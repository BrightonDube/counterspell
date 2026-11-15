//! The Docker side of the policy (judge-sandbox T20, part 1).
//!
//! This file converts a [`ContainerPolicy`] into the daemon's create request, reads a
//! container's configuration back into a policy, and creates a container only after the
//! image digest and the read-back both pass. The start, exec, file and kill calls come in
//! part 2 (T21).
//!
//! Only this file and `daemon.rs` import bollard. The conversion functions are pure, so
//! the model tests check them without a daemon. The async functions talk to the daemon
//! and are checked by the `sbx_` tests on the rootless sandbox daemon.

use std::collections::{BTreeMap, HashMap};
use std::future::Future;
use std::pin::Pin;
use std::time::Duration;

use bollard::container::LogOutput;
use bollard::exec::{StartExecOptions, StartExecResults};
use bollard::models::{
    ContainerCreateBody, ContainerInspectResponse, ExecConfig, HostConfig,
    HostConfigCgroupnsModeEnum, HostConfigLogConfig, ResourcesUlimits, RestartPolicy,
    RestartPolicyNameEnum,
};
use bollard::query_parameters::{
    CreateContainerOptionsBuilder, InspectContainerOptions, KillContainerOptionsBuilder,
    RemoveContainerOptionsBuilder,
};
use bollard::Docker;
use futures_util::{Stream, StreamExt};
use thiserror::Error;
use tokio::io::{AsyncWrite, AsyncWriteExt};

use crate::languages::PinnedImage;
use crate::sandbox::backend::{
    Backend, BackendSession, DriverChannel, DriverEvent, ExecOutput, InfraError,
};
use crate::sandbox::config::OciRuntime;
use crate::sandbox::policy::{
    fixed_env, ContainerPolicy, Limits, Namespaces, ScratchExec, SeccompProfile, TmpfsMount,
    Ulimit, SANDBOX_USER, WORKING_DIR,
};
use crate::sandbox::spec::{ExecSpec, SessionSpec};

/// The working directory and scratch mount, as the policy names them.
const SCRATCH_PATH: &str = "/box";
const SECCOMP_PREFIX: &str = "seccomp=";
const SIZE_PREFIX: &str = "size=";
/// Docker's answer for a namespace mode that is the default private one.
const PRIVATE_MODE: &str = "";
/// The mode the policy uses when it asks for a shared namespace with the host.
const HOST_MODE: &str = "host";

/// Why a daemon's answer cannot be read as a policy. A refused answer counts as a
/// read-back mismatch, so the container is removed.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ReadbackError {
    /// A section the policy needs is absent from the daemon's answer.
    #[error("the daemon did not report {0}")]
    Missing(&'static str),
    /// A value is present but is not one the judge knows how to read.
    #[error("the daemon reported an unrecognised {0}")]
    Unrecognised(&'static str),
}

/// Converts a policy into the daemon's create request.
///
/// The request is only meaningful for a policy that passed
/// [`ContainerPolicy::hardening_violations`]. [`create_checked`] enforces that order.
#[must_use]
pub fn to_create_body(policy: &ContainerPolicy) -> ContainerCreateBody {
    let tmpfs: HashMap<String, String> = policy
        .tmpfs
        .iter()
        .map(|mount| (mount.path.clone(), tmpfs_text(mount)))
        .collect();
    ContainerCreateBody {
        image: Some(policy.image.as_str().to_owned()),
        user: Some(policy.user.clone()),
        working_dir: Some(policy.working_dir.clone()),
        env: Some(
            policy
                .env
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        ),
        cmd: Some(policy.command.clone()),
        entrypoint: Some(policy.entrypoint.clone()),
        labels: Some(policy.labels.clone().into_iter().collect()),
        host_config: Some(HostConfig {
            network_mode: Some(policy.network.clone()),
            readonly_rootfs: Some(policy.read_only_rootfs),
            tmpfs: Some(tmpfs),
            shm_size: Some(to_i64(policy.shm_size_bytes)),
            cap_drop: Some(policy.cap_drop.clone()),
            cap_add: Some(policy.cap_add.clone()),
            security_opt: Some(security_opts(policy)),
            privileged: Some(policy.privileged),
            log_config: Some(HostConfigLogConfig {
                typ: Some(policy.log_driver.clone()),
                config: None,
            }),
            restart_policy: Some(RestartPolicy {
                name: Some(restart_from_text(&policy.restart_policy)),
                maximum_retry_count: None,
            }),
            auto_remove: Some(policy.auto_remove),
            memory: Some(to_i64(policy.limits.memory_bytes)),
            memory_swap: Some(to_i64(policy.limits.memory_swap_bytes)),
            nano_cpus: Some(policy.limits.nano_cpus),
            pids_limit: Some(policy.limits.pids_limit),
            ulimits: Some(policy.ulimits.iter().map(ulimit_to_daemon).collect()),
            pid_mode: Some(namespace_mode(policy.namespaces.pid)),
            ipc_mode: Some(namespace_mode(policy.namespaces.ipc)),
            cgroupns_mode: Some(if policy.namespaces.cgroup {
                HostConfigCgroupnsModeEnum::PRIVATE
            } else {
                HostConfigCgroupnsModeEnum::HOST
            }),
            runtime: Some(runtime_text(policy.runtime).to_owned()),
            ..Default::default()
        }),
        ..Default::default()
    }
}

/// Reads a container's configuration back into a policy.
///
/// Every value comes from the daemon's answer. A missing section, or a value the judge
/// cannot read, is an error rather than a guess. The caller compares the result with the
/// policy it created, and with the hardening checker.
///
/// # Errors
///
/// Returns [`ReadbackError`] when a required section or value is missing or unknown.
pub fn from_inspect(inspect: &ContainerInspectResponse) -> Result<ContainerPolicy, ReadbackError> {
    let config = inspect
        .config
        .as_ref()
        .ok_or(ReadbackError::Missing("config"))?;
    let host = inspect
        .host_config
        .as_ref()
        .ok_or(ReadbackError::Missing("host config"))?;
    let image = config
        .image
        .as_deref()
        .and_then(PinnedImage::parse)
        .ok_or(ReadbackError::Unrecognised("image"))?;
    let runtime = match host.runtime.as_deref() {
        Some("runc") => OciRuntime::Runc,
        Some("runsc") => OciRuntime::Runsc,
        _ => return Err(ReadbackError::Unrecognised("runtime")),
    };
    let (seccomp, security_opts) = split_security(host.security_opt.as_deref().unwrap_or_default());
    let tmpfs = read_tmpfs(host.tmpfs.as_ref())?;
    let scratch = scratch_exec_from(&tmpfs);

    Ok(ContainerPolicy {
        image,
        entrypoint: config.entrypoint.clone().unwrap_or_default(),
        command: config.cmd.clone().unwrap_or_default(),
        user: config.user.clone().unwrap_or_default(),
        working_dir: config.working_dir.clone().unwrap_or_default(),
        network: host.network_mode.clone().unwrap_or_default(),
        read_only_rootfs: host.readonly_rootfs.unwrap_or(false),
        tmpfs,
        shm_size_bytes: to_u64(host.shm_size.unwrap_or(0)),
        scratch,
        cap_drop: host.cap_drop.clone().unwrap_or_default(),
        cap_add: host.cap_add.clone().unwrap_or_default(),
        security_opts,
        seccomp,
        privileged: host.privileged.unwrap_or(false),
        host_resources: host_resources(host, inspect),
        namespaces: namespaces_from(host),
        limits: Limits {
            memory_bytes: to_u64(host.memory.unwrap_or(0)),
            memory_swap_bytes: to_u64(host.memory_swap.unwrap_or(0)),
            nano_cpus: host.nano_cpus.unwrap_or(0),
            pids_limit: host.pids_limit.unwrap_or(0),
        },
        ulimits: ulimits_from(host.ulimits.as_deref().unwrap_or_default()),
        log_driver: host
            .log_config
            .as_ref()
            .and_then(|log| log.typ.clone())
            .unwrap_or_default(),
        restart_policy: restart_to_text(host.restart_policy.as_ref().and_then(|r| r.name)),
        auto_remove: host.auto_remove.unwrap_or(false),
        env: env_from(config.env.as_deref().unwrap_or_default())?,
        labels: config
            .labels
            .clone()
            .unwrap_or_default()
            .into_iter()
            .collect::<BTreeMap<_, _>>(),
        runtime,
    })
}

/// True when one of the daemon's repo digests carries the pinned digest.
///
/// Only the digest is compared. A repo digest names the repository without the tag, so the
/// name part cannot be matched exactly. The digest identifies the image content.
#[must_use]
pub fn image_digest_matches(repo_digests: &[String], pinned: &PinnedImage) -> bool {
    let Some(wanted) = digest_part(pinned.as_str()) else {
        return false;
    };
    repo_digests
        .iter()
        .any(|repo_digest| digest_part(repo_digest) == Some(wanted))
}

/// Creates one container from a policy, with the image and read-back checks around it.
///
/// Order: the policy must pass the hardening checker. The image must exist locally with
/// the pinned digest (nothing is pulled). After create, the daemon's answer must read back
/// to the same policy and pass the checker. A failure after create removes the container.
///
/// # Errors
///
/// Returns [`InfraError`] for a refused policy, a missing or wrong image, a daemon error
/// or timeout, or a read-back mismatch. None of these is a verdict on the user's code.
pub async fn create_checked(
    docker: &Docker,
    policy: &ContainerPolicy,
    name: &str,
    api_timeout: Duration,
) -> Result<String, InfraError> {
    if let Some(first) = policy.hardening_violations().into_iter().next() {
        return Err(InfraError::PolicyRejected(first.to_string()));
    }
    check_image(docker, policy, api_timeout).await?;

    let options = CreateContainerOptionsBuilder::new().name(name).build();
    let created = with_timeout(
        api_timeout,
        docker.create_container(Some(options), to_create_body(policy)),
    )
    .await?
    .map_err(|error| InfraError::DaemonUnreachable(error.to_string()))?;
    let id = created.id;

    match verify_readback(docker, policy, &id, api_timeout).await {
        Ok(()) => Ok(id),
        Err(error) => {
            if let Err(removal) = remove_forced(docker, &id, api_timeout).await {
                // The sweeper removes any labelled container left behind. Log only the id.
                tracing::warn!(container = %id, error = %removal, "could not remove a refused sandbox container");
            }
            Err(error)
        }
    }
}

/// Force-removes a container by name or id. The daemon also stops it.
///
/// # Errors
///
/// Returns [`InfraError`] when the daemon fails or does not answer in time.
pub async fn remove_forced(
    docker: &Docker,
    name: &str,
    api_timeout: Duration,
) -> Result<(), InfraError> {
    let options = RemoveContainerOptionsBuilder::new().force(true).build();
    match with_timeout(api_timeout, docker.remove_container(name, Some(options))).await? {
        Ok(()) => Ok(()),
        // A container that is already gone has been removed, which is the goal.
        Err(error) if is_not_found(&error) => Ok(()),
        Err(error) => Err(InfraError::KillFailed(error.to_string())),
    }
}

async fn check_image(
    docker: &Docker,
    policy: &ContainerPolicy,
    api_timeout: Duration,
) -> Result<(), InfraError> {
    let reference = policy.image.as_str();
    let inspected = tokio::time::timeout(api_timeout, docker.inspect_image(reference))
        .await
        .map_err(|_| InfraError::ApiTimeout)?;
    let image = match inspected {
        Ok(image) => image,
        Err(error) if is_not_found(&error) => {
            return Err(InfraError::ImageMissing(reference.to_owned()));
        }
        Err(error) => return Err(InfraError::DaemonUnreachable(error.to_string())),
    };
    let digests = image.repo_digests.unwrap_or_default();
    if image_digest_matches(&digests, &policy.image) {
        Ok(())
    } else {
        Err(InfraError::DigestMismatch(reference.to_owned()))
    }
}

async fn verify_readback(
    docker: &Docker,
    policy: &ContainerPolicy,
    id: &str,
    api_timeout: Duration,
) -> Result<(), InfraError> {
    let inspect = with_timeout(
        api_timeout,
        docker.inspect_container(id, None::<InspectContainerOptions>),
    )
    .await?
    .map_err(|error| InfraError::DaemonUnreachable(error.to_string()))?;
    let read = from_inspect(&inspect)
        .map_err(|error| InfraError::PolicyReadbackMismatch(error.to_string()))?;
    if let Some(first) = read.hardening_violations().into_iter().next() {
        return Err(InfraError::PolicyReadbackMismatch(first.to_string()));
    }
    if read != *policy {
        return Err(InfraError::PolicyReadbackMismatch(
            "the daemon's configuration differs from the policy".to_owned(),
        ));
    }
    Ok(())
}

/// Runs one daemon call under the API timeout and turns a timeout into [`InfraError::ApiTimeout`].
async fn with_timeout<T>(limit: Duration, call: impl Future<Output = T>) -> Result<T, InfraError> {
    tokio::time::timeout(limit, call)
        .await
        .map_err(|_| InfraError::ApiTimeout)
}

fn is_not_found(error: &bollard::errors::Error) -> bool {
    matches!(
        error,
        bollard::errors::Error::DockerResponseServerError {
            status_code: 404,
            ..
        }
    )
}

fn security_opts(policy: &ContainerPolicy) -> Vec<String> {
    let mut opts = policy.security_opts.clone();
    match &policy.seccomp {
        SeccompProfile::Default => {}
        SeccompProfile::Custom(path) => opts.push(format!("{SECCOMP_PREFIX}{path}")),
        SeccompProfile::Unconfined => opts.push(format!("{SECCOMP_PREFIX}unconfined")),
    }
    opts
}

/// Splits the daemon's security options into the seccomp setting and the rest.
/// More than one seccomp entry is reported as a custom profile, which the checker refuses.
fn split_security(entries: &[String]) -> (SeccompProfile, Vec<String>) {
    let mut seccomp = Vec::new();
    let mut others = Vec::new();
    for entry in entries {
        match entry.strip_prefix(SECCOMP_PREFIX) {
            Some(value) => seccomp.push(value.to_owned()),
            None => others.push(entry.clone()),
        }
    }
    let profile = match seccomp.as_slice() {
        [] => SeccompProfile::Default,
        [only] if only == "unconfined" => SeccompProfile::Unconfined,
        [only] => SeccompProfile::Custom(only.clone()),
        _ => SeccompProfile::Custom("more than one seccomp entry".to_owned()),
    };
    (profile, others)
}

fn tmpfs_text(mount: &TmpfsMount) -> String {
    let mut parts = mount.options.clone();
    parts.push(format!("{SIZE_PREFIX}{}", mount.size_bytes));
    parts.join(",")
}

/// Reads the daemon's tmpfs map into mounts, ordered by path so the result is stable.
fn read_tmpfs(map: Option<&HashMap<String, String>>) -> Result<Vec<TmpfsMount>, ReadbackError> {
    let mut mounts = Vec::new();
    for (path, text) in map.into_iter().flatten() {
        let mut size_bytes = None;
        let mut options = Vec::new();
        for token in text.split(',').filter(|token| !token.is_empty()) {
            match token.strip_prefix(SIZE_PREFIX) {
                Some(value) => {
                    let parsed = value
                        .parse::<u64>()
                        .map_err(|_| ReadbackError::Unrecognised("tmpfs size"))?;
                    size_bytes = Some(parsed);
                }
                None => options.push(token.to_owned()),
            }
        }
        mounts.push(TmpfsMount {
            path: path.clone(),
            size_bytes: size_bytes.ok_or(ReadbackError::Missing("tmpfs size"))?,
            options,
        });
    }
    mounts.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(mounts)
}

/// Reads the scratch mode from the daemon's answer. Without a readable scratch mount, the
/// stricter `Forbidden` is assumed, so the checker sees the stronger requirement.
fn scratch_exec_from(mounts: &[TmpfsMount]) -> ScratchExec {
    match mounts.iter().find(|mount| mount.path == SCRATCH_PATH) {
        Some(scratch) if scratch.options.iter().any(|o| o == "noexec") => ScratchExec::Forbidden,
        Some(_) => ScratchExec::Allowed,
        None => ScratchExec::Forbidden,
    }
}

fn namespaces_from(host: &HostConfig) -> Namespaces {
    Namespaces {
        pid: is_private_mode(host.pid_mode.as_deref()),
        ipc: is_private_mode(host.ipc_mode.as_deref()),
        cgroup: matches!(
            host.cgroupns_mode,
            Some(HostConfigCgroupnsModeEnum::PRIVATE | HostConfigCgroupnsModeEnum::EMPTY)
        ),
    }
}

/// An absent mode is treated as shared, so the checker reports it.
fn is_private_mode(mode: Option<&str>) -> bool {
    matches!(mode, Some(PRIVATE_MODE | "private"))
}

fn namespace_mode(private: bool) -> String {
    if private {
        PRIVATE_MODE.to_owned()
    } else {
        HOST_MODE.to_owned()
    }
}

/// Lists the host resources the daemon reports: binds, devices, hosts, links, shared
/// volumes and every mount (including volumes an image declares).
fn host_resources(host: &HostConfig, inspect: &ContainerInspectResponse) -> Vec<String> {
    let mut found = Vec::new();
    found.extend(
        host.binds
            .iter()
            .flatten()
            .map(|bind| format!("bind {bind}")),
    );
    found.extend(host.devices.iter().flatten().map(|_| "device".to_owned()));
    found.extend(
        host.extra_hosts
            .iter()
            .flatten()
            .map(|host| format!("extra host {host}")),
    );
    found.extend(
        host.links
            .iter()
            .flatten()
            .map(|link| format!("link {link}")),
    );
    found.extend(
        host.volumes_from
            .iter()
            .flatten()
            .map(|source| format!("volumes-from {source}")),
    );
    found.extend(
        inspect
            .mounts
            .iter()
            .flatten()
            .map(|mount| format!("mount {}", mount.destination.as_deref().unwrap_or("?"))),
    );
    found
}

fn ulimits_from(entries: &[ResourcesUlimits]) -> Vec<Ulimit> {
    // An entry without a name, soft or hard value is skipped. The checker then reports the
    // limit as missing, which is the fail-closed result.
    entries
        .iter()
        .filter_map(|entry| {
            Some(Ulimit {
                name: entry.name.clone()?,
                soft: entry.soft?,
                hard: entry.hard?,
            })
        })
        .collect()
}

fn ulimit_to_daemon(limit: &Ulimit) -> ResourcesUlimits {
    ResourcesUlimits {
        name: Some(limit.name.clone()),
        soft: Some(limit.soft),
        hard: Some(limit.hard),
    }
}

/// Reads `KEY=VALUE` strings. A string without `=` cannot be read, so it is an error.
fn env_from(entries: &[String]) -> Result<Vec<(String, String)>, ReadbackError> {
    entries
        .iter()
        .map(|entry| {
            entry
                .split_once('=')
                .map(|(key, value)| (key.to_owned(), value.to_owned()))
                .ok_or(ReadbackError::Unrecognised("environment"))
        })
        .collect()
}

fn restart_from_text(text: &str) -> RestartPolicyNameEnum {
    match text {
        "no" => RestartPolicyNameEnum::NO,
        "always" => RestartPolicyNameEnum::ALWAYS,
        "unless-stopped" => RestartPolicyNameEnum::UNLESS_STOPPED,
        "on-failure" => RestartPolicyNameEnum::ON_FAILURE,
        // Only "no" passes the checker. Any other text becomes an empty name, which the
        // checker then reports as a restart-policy violation.
        _ => RestartPolicyNameEnum::EMPTY,
    }
}

fn restart_to_text(name: Option<RestartPolicyNameEnum>) -> String {
    match name {
        Some(RestartPolicyNameEnum::NO) => "no",
        Some(RestartPolicyNameEnum::ALWAYS) => "always",
        Some(RestartPolicyNameEnum::UNLESS_STOPPED) => "unless-stopped",
        Some(RestartPolicyNameEnum::ON_FAILURE) => "on-failure",
        Some(RestartPolicyNameEnum::EMPTY) | None => "",
    }
    .to_owned()
}

fn runtime_text(runtime: OciRuntime) -> &'static str {
    match runtime {
        OciRuntime::Runc => "runc",
        OciRuntime::Runsc => "runsc",
    }
}

/// The part after the `@`, for `name@sha256:...`.
fn digest_part(reference: &str) -> Option<&str> {
    reference.rsplit_once('@').map(|(_, digest)| digest)
}

fn to_i64(value: u64) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

fn to_u64(value: i64) -> u64 {
    u64::try_from(value).unwrap_or(0)
}

// ---------------------------------------------------------------------------
// Part 2 (T21): start, exec, files, facts, kill and remove.
// ---------------------------------------------------------------------------

/// The most output one exec may return, stdout and stderr together (design section 7).
pub const EXEC_OUTPUT_CAP: usize = 64 * 1024;
/// Where the kernel reports the container's memory events (cgroup v2, private cgroup namespace).
const MEMORY_EVENTS_PATH: &str = "/sys/fs/cgroup/memory.events";
/// The signal the worker uses to stop a container. It cannot be caught or ignored.
const KILL_SIGNAL: &str = "SIGKILL";
/// The daemon answers 409 when a container is not running, which means it is already stopped.
const CONFLICT: u16 = 409;

/// Which output stream a chunk of exec output came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    /// The command's standard output.
    Stdout,
    /// The command's standard error.
    Stderr,
}

/// Collects exec output under one shared cap. Stdout and stderr together never hold more
/// than `cap` bytes. Once a write crosses the cap, nothing more is stored.
#[derive(Debug)]
pub struct OutputCollector {
    cap: usize,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    overflowed: bool,
}

impl OutputCollector {
    /// An empty collector with the given shared cap.
    #[must_use]
    pub fn new(cap: usize) -> Self {
        Self {
            cap,
            stdout: Vec::new(),
            stderr: Vec::new(),
            overflowed: false,
        }
    }

    /// Stores as much of `bytes` as fits. Returns `false` when this write, or an earlier
    /// one, went over the cap. The caller then stops reading the command.
    pub fn push(&mut self, stream: OutputStream, bytes: &[u8]) -> bool {
        if self.overflowed {
            return false;
        }
        let held = self.stdout.len() + self.stderr.len();
        let room = self.cap.saturating_sub(held);
        let kept = &bytes[..bytes.len().min(room)];
        match stream {
            OutputStream::Stdout => self.stdout.extend_from_slice(kept),
            OutputStream::Stderr => self.stderr.extend_from_slice(kept),
        }
        if bytes.len() > room {
            self.overflowed = true;
            false
        } else {
            true
        }
    }

    /// The collected output with the command's exit code.
    #[must_use]
    pub fn into_output(self, exit_code: i64) -> ExecOutput {
        ExecOutput {
            exit_code,
            stdout: self.stdout,
            stderr: self.stderr,
            overflowed: self.overflowed,
        }
    }
}

/// Reads the `oom_kill` count from the text of `memory.events`.
///
/// # Errors
///
/// Returns [`InfraError::Unexpected`] when the line is missing or is not a number. A missing
/// counter is never read as zero, so an OOM kill cannot be hidden.
pub fn parse_oom_kill(text: &str) -> Result<u64, InfraError> {
    let value = text
        .lines()
        .find_map(|line| line.strip_prefix("oom_kill "))
        .ok_or_else(|| InfraError::Unexpected("memory.events has no oom_kill line".to_owned()))?;
    value
        .trim()
        .parse::<u64>()
        .map_err(|_| InfraError::Unexpected("oom_kill is not a number".to_owned()))
}

/// Turns a daemon error into an infrastructure error. Connection and timeout failures keep
/// their own variants. Every other daemon answer is `Unexpected` and keeps its message.
#[must_use]
pub fn map_error(error: &bollard::errors::Error) -> InfraError {
    use bollard::errors::Error;
    match error {
        Error::RequestTimeoutError => InfraError::ApiTimeout,
        Error::IOError { .. }
        | Error::SocketNotFoundError(_)
        | Error::HyperResponseError { .. }
        | Error::HyperLegacyError { .. }
        | Error::HttpClientError { .. } => InfraError::DaemonUnreachable(error.to_string()),
        other => InfraError::Unexpected(other.to_string()),
    }
}

/// The bollard backend: creates sandbox containers on the rootless sandbox daemon.
#[derive(Clone)]
pub struct BollardBackend {
    docker: Docker,
    api_timeout: Duration,
}

impl BollardBackend {
    /// Wraps a connected daemon client. Every call uses `api_timeout`.
    #[must_use]
    pub fn new(docker: Docker, api_timeout: Duration) -> Self {
        Self {
            docker,
            api_timeout,
        }
    }
}

impl Backend for BollardBackend {
    type Session = BollardSession;

    async fn create(&self, spec: &SessionSpec) -> Result<BollardSession, InfraError> {
        let id = create_checked(&self.docker, &spec.policy, &spec.name, self.api_timeout).await?;
        Ok(BollardSession {
            docker: self.docker.clone(),
            id,
            api_timeout: self.api_timeout,
        })
    }
}

/// One live sandbox container, identified by the id the daemon returned at create time.
pub struct BollardSession {
    docker: Docker,
    id: String,
    api_timeout: Duration,
}

impl BackendSession for BollardSession {
    type Driver = BollardDriver;

    fn name(&self) -> &str {
        &self.id
    }

    async fn read_policy(&mut self) -> Result<ContainerPolicy, InfraError> {
        let inspect = with_timeout(
            self.api_timeout,
            self.docker
                .inspect_container(&self.id, None::<InspectContainerOptions>),
        )
        .await?
        .map_err(|error| map_error(&error))?;
        from_inspect(&inspect)
            .map_err(|error| InfraError::PolicyReadbackMismatch(error.to_string()))
    }

    async fn start(&mut self) -> Result<(), InfraError> {
        with_timeout(
            self.api_timeout,
            self.docker.start_container(&self.id, None),
        )
        .await?
        .map_err(|error| map_error(&error))
    }

    async fn exec(&mut self, spec: &ExecSpec, stdin: &[u8]) -> Result<ExecOutput, InfraError> {
        run_exec(&self.docker, &self.id, spec, stdin, self.api_timeout).await
    }

    async fn open_driver(&mut self, spec: &ExecSpec) -> Result<BollardDriver, InfraError> {
        let created = with_timeout(
            self.api_timeout,
            self.docker.create_exec(&self.id, exec_config(spec)),
        )
        .await?
        .map_err(|error| map_error(&error))?;
        let (output, input) = attach(&self.docker, &created.id, self.api_timeout).await?;
        Ok(BollardDriver {
            docker: self.docker.clone(),
            container: self.id.clone(),
            exec_id: created.id,
            output,
            input,
            api_timeout: self.api_timeout,
        })
    }

    async fn kill(&mut self) -> Result<(), InfraError> {
        kill_container(&self.docker, &self.id, self.api_timeout).await
    }

    async fn remove(&mut self) -> Result<(), InfraError> {
        remove_forced(&self.docker, &self.id, self.api_timeout).await?;
        // Removal is only done once the daemon no longer knows the container.
        match with_timeout(
            self.api_timeout,
            self.docker
                .inspect_container(&self.id, None::<InspectContainerOptions>),
        )
        .await?
        {
            Err(error) if is_not_found(&error) => Ok(()),
            Err(error) => Err(InfraError::KillFailed(error.to_string())),
            Ok(_) => Err(InfraError::KillFailed(
                "the container still exists after removal".to_owned(),
            )),
        }
    }
}

type OutputStreamOf = Pin<Box<dyn Stream<Item = Result<LogOutput, bollard::errors::Error>> + Send>>;
type InputOf = Pin<Box<dyn AsyncWrite + Send>>;

/// The driver process's attached stdin and stdout. Its exit is reported by `next_event`.
pub struct BollardDriver {
    docker: Docker,
    container: String,
    exec_id: String,
    output: OutputStreamOf,
    input: InputOf,
    api_timeout: Duration,
}

impl DriverChannel for BollardDriver {
    async fn write_line(&mut self, line: &[u8]) -> Result<(), InfraError> {
        // A closed stdin means the driver has already ended. The end is reported by
        // `next_event` as `Exited`, so the write itself is not an infrastructure error.
        if self.input.write_all(line).await.is_ok() {
            let _ = self.input.flush().await;
        }
        Ok(())
    }

    async fn next_event(&mut self) -> Result<DriverEvent, InfraError> {
        loop {
            match self.output.next().await {
                Some(Ok(LogOutput::StdOut { message })) => {
                    return Ok(DriverEvent::Chunk(message.to_vec()));
                }
                Some(Ok(LogOutput::StdErr { message })) => {
                    return Ok(DriverEvent::Stderr(message.to_vec()));
                }
                Some(Ok(_)) => {}
                Some(Err(error)) => return Err(map_error(&error)),
                None => return self.exited().await,
            }
        }
    }
}

impl BollardDriver {
    /// Reads the exit code and the OOM count after the output stream ends. Any failure is an
    /// error, never an `Exited` event, so a failed read cannot become a user's verdict.
    async fn exited(&mut self) -> Result<DriverEvent, InfraError> {
        let inspect = with_timeout(self.api_timeout, self.docker.inspect_exec(&self.exec_id))
            .await?
            .map_err(|error| map_error(&error))?;
        let code = inspect.exit_code.ok_or_else(|| {
            InfraError::Unexpected("driver finished without an exit code".to_owned())
        })?;
        let oom_kill = read_oom_kill(&self.docker, &self.container, self.api_timeout).await?;
        Ok(DriverEvent::Exited { code, oom_kill })
    }
}

/// Reads the container's `oom_kill` count with a fixed `cat` command.
async fn read_oom_kill(
    docker: &Docker,
    container: &str,
    api_timeout: Duration,
) -> Result<u64, InfraError> {
    let spec = ExecSpec {
        argv: vec!["cat".to_owned(), MEMORY_EVENTS_PATH.to_owned()],
        user: SANDBOX_USER.to_owned(),
        working_dir: WORKING_DIR.to_owned(),
        env: fixed_env(),
    };
    let output = run_exec(docker, container, &spec, &[], api_timeout).await?;
    if output.exit_code != 0 || output.overflowed {
        return Err(InfraError::Unexpected(
            "the OOM counter could not be read".to_owned(),
        ));
    }
    parse_oom_kill(&String::from_utf8_lossy(&output.stdout))
}

/// Runs one command to completion inside a container and collects its output.
///
/// Output past [`EXEC_OUTPUT_CAP`] stops the read and kills the container, so the command
/// stops too. The result then has `overflowed` set, and the caller treats the session as
/// abnormal.
///
/// # Errors
///
/// Returns [`InfraError`] when the daemon fails, times out, or does not report an exit code.
pub async fn run_exec(
    docker: &Docker,
    container: &str,
    spec: &ExecSpec,
    stdin: &[u8],
    api_timeout: Duration,
) -> Result<ExecOutput, InfraError> {
    let created = with_timeout(
        api_timeout,
        docker.create_exec(container, exec_config(spec)),
    )
    .await?
    .map_err(|error| map_error(&error))?;
    let (mut output, mut input) = attach(docker, &created.id, api_timeout).await?;
    input
        .write_all(stdin)
        .await
        .map_err(|error| InfraError::ExecSetup(error.to_string()))?;
    // Half-close: the command sees end of input after the bytes above.
    input
        .shutdown()
        .await
        .map_err(|error| InfraError::ExecSetup(error.to_string()))?;
    drop(input);

    let mut collected = OutputCollector::new(EXEC_OUTPUT_CAP);
    while let Some(item) = output.next().await {
        let item = item.map_err(|error| map_error(&error))?;
        let (stream, bytes) = match item {
            LogOutput::StdOut { message } => (OutputStream::Stdout, message),
            LogOutput::StdErr { message } => (OutputStream::Stderr, message),
            _ => continue,
        };
        if !collected.push(stream, &bytes) {
            kill_container(docker, container, api_timeout).await?;
            return Ok(collected.into_output(-1));
        }
    }
    let inspect = with_timeout(api_timeout, docker.inspect_exec(&created.id))
        .await?
        .map_err(|error| map_error(&error))?;
    let code = inspect.exit_code.ok_or_else(|| {
        InfraError::Unexpected("command finished without an exit code".to_owned())
    })?;
    Ok(collected.into_output(code))
}

/// Starts an exec with stdin, stdout and stderr attached.
async fn attach(
    docker: &Docker,
    exec_id: &str,
    api_timeout: Duration,
) -> Result<(OutputStreamOf, InputOf), InfraError> {
    let options = StartExecOptions {
        detach: false,
        tty: false,
        output_capacity: None,
    };
    let started = with_timeout(api_timeout, docker.start_exec(exec_id, Some(options)))
        .await?
        .map_err(|error| map_error(&error))?;
    match started {
        StartExecResults::Attached { output, input } => Ok((output, input)),
        StartExecResults::Detached => Err(InfraError::ExecSetup(
            "the daemon started the command detached".to_owned(),
        )),
    }
}

/// The daemon request for one exec. The user, working directory and environment come from
/// the spec, which is fixed, and the argv is a list, never a shell string.
fn exec_config(spec: &ExecSpec) -> ExecConfig {
    ExecConfig {
        attach_stdin: Some(true),
        attach_stdout: Some(true),
        attach_stderr: Some(true),
        tty: Some(false),
        privileged: Some(false),
        user: Some(spec.user.clone()),
        working_dir: Some(spec.working_dir.clone()),
        env: Some(
            spec.env
                .iter()
                .map(|(key, value)| format!("{key}={value}"))
                .collect(),
        ),
        cmd: Some(spec.argv.clone()),
        ..Default::default()
    }
}

/// Sends SIGKILL to a container. A container that is already stopped or gone counts as done.
///
/// # Errors
///
/// Returns [`InfraError::KillFailed`] when the daemon refuses for another reason, or
/// [`InfraError::ApiTimeout`] when it does not answer in time.
pub async fn kill_container(
    docker: &Docker,
    container: &str,
    api_timeout: Duration,
) -> Result<(), InfraError> {
    let options = KillContainerOptionsBuilder::new()
        .signal(KILL_SIGNAL)
        .build();
    match with_timeout(api_timeout, docker.kill_container(container, Some(options))).await? {
        Ok(()) => Ok(()),
        Err(error) if is_not_found(&error) || is_conflict(&error) => Ok(()),
        Err(error) => Err(InfraError::KillFailed(error.to_string())),
    }
}

fn is_conflict(error: &bollard::errors::Error) -> bool {
    matches!(
        error,
        bollard::errors::Error::DockerResponseServerError { status_code, .. }
            if *status_code == CONFLICT
    )
}
