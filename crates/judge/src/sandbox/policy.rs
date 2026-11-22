//! The container policy: every control a sandbox container must have (design.md
//! section 7). This module is pure data and checks. It never talks to Docker, so
//! the checker can be mutation-tested. The Docker backend converts this value into
//! a daemon request and reads the container back to run the same checker again.

use std::collections::BTreeMap;
use std::time::Duration;

use thiserror::Error;

use super::config::OciRuntime;
use crate::languages::{LanguageDefinition, PinnedImage};

const MIB: u64 = 1024 * 1024;
pub(super) const SANDBOX_USER: &str = "65534:65534";
pub(super) const WORKING_DIR: &str = "/box";
const NANO_CPUS_ONE: i64 = 1_000_000_000;
const PIDS_LIMIT: i64 = 64;
const NOFILE_LIMIT: i64 = 64;
const FILE_SIZE_LIMIT: i64 = 16 * 1024 * 1024;
const SCRATCH_BYTES: u64 = 64 * MIB;
const TMP_BYTES: u64 = 16 * MIB;
const SHM_BYTES: u64 = 16 * MIB;
/// PID 1 sleeps for the run cap plus this many seconds, so a container expires by itself.
const IDLE_EXTRA_SECS: u64 = 5;
const NO_NEW_PRIVILEGES: &str = "no-new-privileges:true";
const MANAGED_LABEL: &str = "counterspell.managed";

/// Whether the scratch directory may be executed from. Only compiled languages need it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScratchExec {
    /// `noexec` is set on the scratch directory. This is the default for Python.
    Forbidden,
    /// No `noexec` flag. Used only by a language that compiles to a native binary.
    Allowed,
}

/// Which seccomp profile applies. The production value is always `Default`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SeccompProfile {
    /// Docker's default profile. No seccomp entry is written.
    Default,
    /// A custom profile file. Not allowed by the hardening checker.
    Custom(String),
    /// No filter at all. Not allowed by the hardening checker.
    Unconfined,
}

/// One `tmpfs` mount inside the container.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TmpfsMount {
    /// The mount point inside the container.
    pub path: String,
    /// The size limit in bytes. The mount counts against the memory limit.
    pub size_bytes: u64,
    /// Mount options, for example `noexec` or `uid=65534`.
    pub options: Vec<String>,
}

/// One resource limit set with `ulimit`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Ulimit {
    /// The limit name, for example `nofile`.
    pub name: String,
    /// The soft limit.
    pub soft: i64,
    /// The hard limit.
    pub hard: i64,
}

/// Which namespaces the container has of its own. All three must be private.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Namespaces {
    /// A private process-ID namespace.
    pub pid: bool,
    /// A private IPC namespace.
    pub ipc: bool,
    /// A private cgroup namespace.
    pub cgroup: bool,
}

/// CPU and memory limits.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Limits {
    /// Memory limit in bytes.
    pub memory_bytes: u64,
    /// Memory plus swap in bytes. Must equal `memory_bytes`, so swap is never used.
    pub memory_swap_bytes: u64,
    /// CPU quota in billionths of a CPU. One CPU is `1_000_000_000`.
    pub nano_cpus: i64,
    /// The most processes and threads the container may have.
    pub pids_limit: i64,
}

/// Every control a sandbox container must have. Fields are public because this is
/// plain data, and the checker is what makes a weakened value visible.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContainerPolicy {
    /// The digest-pinned image. Inspected before use, never pulled.
    pub image: PinnedImage,
    /// The entrypoint. Must be empty, so PID 1 is the idle command below.
    pub entrypoint: Vec<String>,
    /// The idle command that is PID 1.
    pub command: Vec<String>,
    /// The user for the container and every exec.
    pub user: String,
    /// The working directory.
    pub working_dir: String,
    /// The network mode. Must be `none`.
    pub network: String,
    /// Whether the root filesystem is read-only.
    pub read_only_rootfs: bool,
    /// The `tmpfs` mounts. Exactly `/box` and `/tmp` are allowed.
    pub tmpfs: Vec<TmpfsMount>,
    /// The size of `/dev/shm`.
    pub shm_size_bytes: u64,
    /// Whether the scratch directory may be executed from.
    pub scratch: ScratchExec,
    /// Capabilities to drop. Must be `ALL`.
    pub cap_drop: Vec<String>,
    /// Capabilities to add. Must be empty.
    pub cap_add: Vec<String>,
    /// Security options. Must be exactly `no-new-privileges:true`.
    pub security_opts: Vec<String>,
    /// The seccomp profile. Must be `Default`.
    pub seccomp: SeccompProfile,
    /// Whether the container is privileged. Must be false.
    pub privileged: bool,
    /// Devices, binds, volumes, extra hosts or links. Must be empty.
    pub host_resources: Vec<String>,
    /// Which namespaces are private.
    pub namespaces: Namespaces,
    /// CPU, memory and process limits.
    pub limits: Limits,
    /// Other `ulimit` values. Must include `nofile`, `fsize` and `core`, and must not include `nproc`.
    pub ulimits: Vec<Ulimit>,
    /// The log driver. Must be `none`, because output arrives over the exec stream.
    pub log_driver: String,
    /// The restart policy. Must be `no`.
    pub restart_policy: String,
    /// Whether the daemon removes the container on exit. Must be false.
    pub auto_remove: bool,
    /// Environment variables. Exactly the fixed set.
    pub env: Vec<(String, String)>,
    /// Labels. `counterspell.managed` must be `true`.
    pub labels: BTreeMap<String, String>,
    /// The OCI runtime.
    pub runtime: OciRuntime,
}

/// One control that is missing or weakened. Each variant names one control.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum PolicyViolation {
    /// PID 1 is not the fixed idle command.
    #[error("PID 1 must be the fixed idle command")]
    IdlePidOne,
    /// The container does not run as the unprivileged sandbox user.
    #[error("the container must run as user 65534:65534")]
    NonRootUser,
    /// The working directory is not `/box`.
    #[error("the working directory must be /box")]
    WorkingDirectory,
    /// The network is not disabled.
    #[error("the network mode must be none")]
    NetworkEnabled,
    /// The root filesystem is writable.
    #[error("the root filesystem must be read-only")]
    WritableRootFs,
    /// The `/box` scratch mount is missing or weaker than required.
    #[error("the /box mount is not locked down: {0}")]
    ScratchMount(&'static str),
    /// The `/tmp` mount is missing or weaker than required.
    #[error("the /tmp mount is not locked down: {0}")]
    TmpMount(&'static str),
    /// A `tmpfs` mount exists that the policy does not allow.
    #[error("an unexpected tmpfs mount is present")]
    UnexpectedMount,
    /// `/dev/shm` is not the fixed size.
    #[error("the /dev/shm size must be 16 MiB")]
    ShmSize,
    /// Not every capability is dropped.
    #[error("every capability must be dropped")]
    CapabilitiesNotDropped,
    /// A capability is added back.
    #[error("no capability may be added")]
    CapabilityAdded,
    /// `no-new-privileges` is not set.
    #[error("no-new-privileges must be set")]
    NoNewPrivilegesMissing,
    /// A security option other than `no-new-privileges` is set.
    #[error("an unexpected security option is set")]
    UnexpectedSecurityOption,
    /// The seccomp profile is not Docker's default.
    #[error("the seccomp profile must be Docker's default")]
    SeccompOverridden,
    /// The container is privileged.
    #[error("the container must not be privileged")]
    Privileged,
    /// A device, bind, volume, extra host or link is present.
    #[error("no host device, bind, volume, extra host or link may be present")]
    HostResourceMounted,
    /// A namespace is shared with the host.
    #[error("the {0} namespace must be private")]
    SharedNamespace(&'static str),
    /// The CPU quota is not one CPU.
    #[error("the CPU quota must be one CPU")]
    CpuQuota,
    /// The memory limit is missing.
    #[error("a memory limit must be set")]
    MemoryLimit,
    /// Swap is allowed beyond the memory limit.
    #[error("memory plus swap must equal the memory limit")]
    SwapAllowed,
    /// The process limit is not 64.
    #[error("the PID limit must be 64")]
    PidsLimit,
    /// The open-file limit is not 64.
    #[error("the nofile limit must be 64")]
    NoFileLimit,
    /// The file-size limit is not 16 MiB, or it is missing.
    #[error("the fsize limit must be 16 MiB")]
    FileSizeLimit,
    /// Core dumps are not disabled.
    #[error("core dumps must be disabled")]
    CoreDumpsAllowed,
    /// An `nproc` limit is set. It is left out on purpose (design open question 1).
    #[error("the nproc limit must not be set")]
    ProcLimitSet,
    /// The log driver is not `none`.
    #[error("the log driver must be none")]
    LogDriver,
    /// The restart policy is not `no`.
    #[error("the restart policy must be no")]
    RestartPolicy,
    /// The daemon would remove the container on exit.
    #[error("auto-remove must be off")]
    AutoRemove,
    /// The environment is not the fixed set.
    #[error("the environment must be the fixed set")]
    UnexpectedEnvironment,
    /// A required label is missing or wrong.
    #[error("the label {0} is missing or wrong")]
    MissingLabel(&'static str),
}

impl ContainerPolicy {
    /// Builds the policy for one language. This is the only place in the judge that
    /// describes a sandbox container, so every other caller gets the same controls.
    #[must_use]
    pub fn for_language(lang: &LanguageDefinition, run_cap: Duration, runtime: OciRuntime) -> Self {
        let memory = lang.mem_mb() * MIB;
        let (scratch, box_options) = if lang.exec_scratch() {
            (ScratchExec::Allowed, scratch_options(false))
        } else {
            (ScratchExec::Forbidden, scratch_options(true))
        };

        Self {
            image: lang.image().clone(),
            entrypoint: Vec::new(),
            command: vec![
                "sleep".to_owned(),
                (run_cap.as_secs() + IDLE_EXTRA_SECS).to_string(),
            ],
            user: SANDBOX_USER.to_owned(),
            working_dir: WORKING_DIR.to_owned(),
            network: "none".to_owned(),
            read_only_rootfs: true,
            tmpfs: vec![
                TmpfsMount {
                    path: WORKING_DIR.to_owned(),
                    size_bytes: SCRATCH_BYTES,
                    options: box_options,
                },
                TmpfsMount {
                    path: "/tmp".to_owned(),
                    size_bytes: TMP_BYTES,
                    options: strings(&["rw", "noexec", "nosuid", "nodev"]),
                },
            ],
            shm_size_bytes: SHM_BYTES,
            scratch,
            cap_drop: strings(&["ALL"]),
            cap_add: Vec::new(),
            security_opts: strings(&[NO_NEW_PRIVILEGES]),
            seccomp: SeccompProfile::Default,
            privileged: false,
            host_resources: Vec::new(),
            namespaces: Namespaces {
                pid: true,
                ipc: true,
                cgroup: true,
            },
            limits: Limits {
                memory_bytes: memory,
                memory_swap_bytes: memory,
                nano_cpus: NANO_CPUS_ONE,
                pids_limit: PIDS_LIMIT,
            },
            ulimits: vec![
                Ulimit {
                    name: "nofile".to_owned(),
                    soft: NOFILE_LIMIT,
                    hard: NOFILE_LIMIT,
                },
                Ulimit {
                    name: "fsize".to_owned(),
                    soft: FILE_SIZE_LIMIT,
                    hard: FILE_SIZE_LIMIT,
                },
                Ulimit {
                    name: "core".to_owned(),
                    soft: 0,
                    hard: 0,
                },
            ],
            log_driver: "none".to_owned(),
            restart_policy: "no".to_owned(),
            auto_remove: false,
            env: fixed_env(),
            labels: BTreeMap::from([(MANAGED_LABEL.to_owned(), "true".to_owned())]),
            runtime,
        }
    }

    /// Lists every control that is missing or weakened. An empty list means the policy
    /// is hardened. The backend runs this before creating a container, and again on the
    /// configuration the daemon reports back.
    #[must_use]
    pub fn hardening_violations(&self) -> Vec<PolicyViolation> {
        let mut found = Vec::new();
        self.check_identity_and_entry(&mut found);
        self.check_privileges(&mut found);
        self.check_resources(&mut found);
        self.check_mounts(&mut found);
        self.check_runtime_settings(&mut found);
        found
    }

    fn check_identity_and_entry(&self, found: &mut Vec<PolicyViolation>) {
        if !self.entrypoint.is_empty() || self.command.len() != 2 || self.command[0] != "sleep" {
            found.push(PolicyViolation::IdlePidOne);
        }
        if self.user != SANDBOX_USER {
            found.push(PolicyViolation::NonRootUser);
        }
        if self.working_dir != WORKING_DIR {
            found.push(PolicyViolation::WorkingDirectory);
        }
        if self.network != "none" {
            found.push(PolicyViolation::NetworkEnabled);
        }
        if !self.read_only_rootfs {
            found.push(PolicyViolation::WritableRootFs);
        }
    }

    fn check_privileges(&self, found: &mut Vec<PolicyViolation>) {
        if self.cap_drop != ["ALL"] {
            found.push(PolicyViolation::CapabilitiesNotDropped);
        }
        if !self.cap_add.is_empty() {
            found.push(PolicyViolation::CapabilityAdded);
        }
        if !self.security_opts.iter().any(|o| o == NO_NEW_PRIVILEGES) {
            found.push(PolicyViolation::NoNewPrivilegesMissing);
        }
        if self.security_opts.iter().any(|o| o != NO_NEW_PRIVILEGES) {
            found.push(PolicyViolation::UnexpectedSecurityOption);
        }
        if self.seccomp != SeccompProfile::Default {
            found.push(PolicyViolation::SeccompOverridden);
        }
        if self.privileged {
            found.push(PolicyViolation::Privileged);
        }
        if !self.host_resources.is_empty() {
            found.push(PolicyViolation::HostResourceMounted);
        }
        if !self.namespaces.pid {
            found.push(PolicyViolation::SharedNamespace("pid"));
        }
        if !self.namespaces.ipc {
            found.push(PolicyViolation::SharedNamespace("ipc"));
        }
        if !self.namespaces.cgroup {
            found.push(PolicyViolation::SharedNamespace("cgroup"));
        }
    }

    fn check_resources(&self, found: &mut Vec<PolicyViolation>) {
        if self.limits.nano_cpus != NANO_CPUS_ONE {
            found.push(PolicyViolation::CpuQuota);
        }
        if self.limits.memory_bytes == 0 {
            found.push(PolicyViolation::MemoryLimit);
        }
        if self.limits.memory_swap_bytes != self.limits.memory_bytes {
            found.push(PolicyViolation::SwapAllowed);
        }
        if self.limits.pids_limit != PIDS_LIMIT {
            found.push(PolicyViolation::PidsLimit);
        }
        if self.ulimits.iter().any(|u| u.name == "nproc") {
            found.push(PolicyViolation::ProcLimitSet);
        }
        if !ulimit_is(&self.ulimits, "nofile", NOFILE_LIMIT) {
            found.push(PolicyViolation::NoFileLimit);
        }
        if !ulimit_is(&self.ulimits, "fsize", FILE_SIZE_LIMIT) {
            found.push(PolicyViolation::FileSizeLimit);
        }
        if !ulimit_is(&self.ulimits, "core", 0) {
            found.push(PolicyViolation::CoreDumpsAllowed);
        }
        if self.shm_size_bytes != SHM_BYTES {
            found.push(PolicyViolation::ShmSize);
        }
    }

    fn check_mounts(&self, found: &mut Vec<PolicyViolation>) {
        for mount in &self.tmpfs {
            if mount.path != WORKING_DIR && mount.path != "/tmp" {
                found.push(PolicyViolation::UnexpectedMount);
            }
        }
        let duplicated = self.tmpfs.iter().enumerate().any(|(i, mount)| {
            self.tmpfs[..i]
                .iter()
                .any(|earlier| earlier.path == mount.path)
        });
        if duplicated {
            found.push(PolicyViolation::UnexpectedMount);
        }
        match self.tmpfs.iter().find(|m| m.path == WORKING_DIR) {
            None => found.push(PolicyViolation::ScratchMount("missing")),
            Some(scratch) => self.check_scratch(scratch, found),
        }
        match self.tmpfs.iter().find(|m| m.path == "/tmp") {
            None => found.push(PolicyViolation::TmpMount("missing")),
            Some(tmp) => check_tmp(tmp, found),
        }
    }

    fn check_scratch(&self, scratch: &TmpfsMount, found: &mut Vec<PolicyViolation>) {
        let has = |option: &str| scratch.options.iter().any(|o| o == option);
        if !has("nosuid") {
            found.push(PolicyViolation::ScratchMount("nosuid missing"));
        }
        if !has("nodev") {
            found.push(PolicyViolation::ScratchMount("nodev missing"));
        }
        if !(has("uid=65534") && has("gid=65534")) {
            found.push(PolicyViolation::ScratchMount("owner"));
        }
        if !has("mode=0700") {
            found.push(PolicyViolation::ScratchMount("mode"));
        }
        if scratch.size_bytes != SCRATCH_BYTES {
            found.push(PolicyViolation::ScratchMount("size"));
        }
        if self.scratch == ScratchExec::Forbidden && !has("noexec") {
            found.push(PolicyViolation::ScratchMount("noexec missing"));
        }
    }

    fn check_runtime_settings(&self, found: &mut Vec<PolicyViolation>) {
        if self.log_driver != "none" {
            found.push(PolicyViolation::LogDriver);
        }
        if self.restart_policy != "no" {
            found.push(PolicyViolation::RestartPolicy);
        }
        if self.auto_remove {
            found.push(PolicyViolation::AutoRemove);
        }
        if self.env != fixed_env() {
            found.push(PolicyViolation::UnexpectedEnvironment);
        }
        if self.labels.get(MANAGED_LABEL).map(String::as_str) != Some("true") {
            found.push(PolicyViolation::MissingLabel(MANAGED_LABEL));
        }
    }
}

fn check_tmp(tmp: &TmpfsMount, found: &mut Vec<PolicyViolation>) {
    let has = |option: &str| tmp.options.iter().any(|o| o == option);
    if !has("noexec") {
        found.push(PolicyViolation::TmpMount("noexec missing"));
    }
    if !has("nosuid") {
        found.push(PolicyViolation::TmpMount("nosuid missing"));
    }
    if !has("nodev") {
        found.push(PolicyViolation::TmpMount("nodev missing"));
    }
    if tmp.size_bytes != TMP_BYTES {
        found.push(PolicyViolation::TmpMount("size"));
    }
}

/// True when the named ulimit appears exactly once, with soft and hard both equal to `value`.
/// A second entry with the same name is refused, so a weaker duplicate cannot hide behind
/// a correct one.
fn ulimit_is(ulimits: &[Ulimit], name: &str, value: i64) -> bool {
    let mut matching = ulimits.iter().filter(|u| u.name == name);
    match (matching.next(), matching.next()) {
        (Some(only), None) => only.soft == value && only.hard == value,
        _ => false,
    }
}

fn scratch_options(noexec: bool) -> Vec<String> {
    let mut options = vec!["rw".to_owned(), "nosuid".to_owned(), "nodev".to_owned()];
    // Docker's default tmpfs mount set includes noexec; a language that compiles to a
    // native binary has to ask for exec explicitly or its output cannot run.
    options.push(if noexec {
        "noexec".to_owned()
    } else {
        "exec".to_owned()
    });
    options.extend(strings(&["uid=65534", "gid=65534", "mode=0700"]));
    options
}

pub(super) fn fixed_env() -> Vec<(String, String)> {
    vec![
        ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
        ("LANG".to_owned(), "C.UTF-8".to_owned()),
        ("HOME".to_owned(), WORKING_DIR.to_owned()),
    ]
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}
