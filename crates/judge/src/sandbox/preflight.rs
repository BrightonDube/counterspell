//! Preflight evaluation: compares what a probe container reports with the policy it was
//! built from (judge-sandbox T22, design section 13, R4.1 to R4.6).
//!
//! This file is pure. It reads the text of the probe commands and returns one check per
//! control. Running the probe container and keeping the latest result are separate steps.
//! A check that cannot be read counts as failed, so missing output never passes.

use super::policy::ContainerPolicy;

/// The sandbox user's numeric UID, as the policy runs it.
const SANDBOX_UID: u64 = 65534;
/// The CFS period the quota is written against, in microseconds.
const CPU_PERIOD_US: i64 = 100_000;
/// The one-CPU quota in nano-CPUs.
const NANO_CPUS_PER_CPU: i64 = 1_000_000_000;
/// The number of capability sets that must all be empty.
const CAPABILITY_SETS: [&str; 5] = ["CapInh", "CapPrm", "CapEff", "CapBnd", "CapAmb"];

/// The text the probe commands printed, plus the exit code of the write test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProbeOutputs {
    /// `/proc/self/status` of the probe process.
    pub proc_status: String,
    /// `/proc/self/net/dev` of the probe process.
    pub net_dev: String,
    /// `/sys/fs/cgroup/memory.max`.
    pub memory_max: String,
    /// `/sys/fs/cgroup/memory.swap.max`.
    pub memory_swap_max: String,
    /// `/sys/fs/cgroup/pids.max`.
    pub pids_max: String,
    /// `/sys/fs/cgroup/cpu.max`.
    pub cpu_max: String,
    /// `/proc/mounts`.
    pub mounts: String,
    /// The exit code of `touch` on the read-only root. Non-zero is correct.
    pub touch_exit: i64,
}

/// One control and its outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightCheck {
    /// A stable name for the control, used in the report and in tests.
    pub name: &'static str,
    /// The value the policy requires.
    pub expected: String,
    /// The value the probe reported, or a note when nothing could be read.
    pub observed: String,
    /// True only when the observed value matches the expected one.
    pub ok: bool,
}

/// All checks from one probe run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    /// One entry per control, in a fixed order.
    pub checks: Vec<PreflightCheck>,
}

impl PreflightReport {
    /// True when every check passed.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.checks.iter().all(|check| check.ok)
    }

    /// The failing checks, in report order.
    #[must_use]
    pub fn failures(&self) -> Vec<&PreflightCheck> {
        self.checks.iter().filter(|check| !check.ok).collect()
    }
}

/// Compares the probe with the policy and returns one check per control.
#[must_use]
pub fn evaluate(outputs: &ProbeOutputs, policy: &ContainerPolicy) -> PreflightReport {
    let checks = vec![
        flag_check("no_new_privs", "NoNewPrivs", "1", &outputs.proc_status),
        flag_check("seccomp_filter", "Seccomp", "2", &outputs.proc_status),
        capabilities_check(&outputs.proc_status),
        uid_check(&outputs.proc_status),
        network_check(&outputs.net_dev),
        exact_check(
            "memory_limit",
            policy.limits.memory_bytes.to_string(),
            outputs.memory_max.trim(),
        ),
        exact_check(
            "swap_disabled",
            "0".to_owned(),
            outputs.memory_swap_max.trim(),
        ),
        exact_check(
            "pids_limit",
            policy.limits.pids_limit.to_string(),
            outputs.pids_max.trim(),
        ),
        exact_check(
            "cpu_quota",
            format!("{} {CPU_PERIOD_US}", quota_for(policy.limits.nano_cpus)),
            outputs.cpu_max.trim(),
        ),
        root_check(&outputs.mounts),
        scratch_check(&outputs.mounts, policy),
        write_refused_check(outputs.touch_exit),
    ];
    PreflightReport { checks }
}

/// Reads a size such as `16777216`, `64k`, `64M` or `1g` into bytes. Returns `None` when
/// the text is not a size, or when it does not fit in 64 bits.
#[must_use]
pub fn parse_size(text: &str) -> Option<u64> {
    let text = text.trim();
    let unit = text.chars().last()?;
    let (digits, multiplier) = match unit {
        'k' | 'K' => (&text[..text.len() - 1], 1024),
        'm' | 'M' => (&text[..text.len() - 1], 1024 * 1024),
        'g' | 'G' => (&text[..text.len() - 1], 1024 * 1024 * 1024),
        _ => (text, 1),
    };
    digits.parse::<u64>().ok()?.checked_mul(multiplier)
}

/// The CFS quota for a CPU limit, given the period in `CPU_PERIOD_US`.
fn quota_for(nano_cpus: i64) -> i64 {
    nano_cpus.saturating_mul(CPU_PERIOD_US) / NANO_CPUS_PER_CPU
}

/// The value of one `Key:` line from a `/proc/self/status`-style text.
fn status_field<'a>(text: &'a str, key: &str) -> Option<&'a str> {
    text.lines().find_map(|line| {
        let (name, value) = line.split_once(':')?;
        (name == key).then(|| value.trim())
    })
}

fn flag_check(name: &'static str, key: &str, want: &str, status: &str) -> PreflightCheck {
    exact_check(
        name,
        want.to_owned(),
        status_field(status, key).unwrap_or("missing"),
    )
}

fn capabilities_check(status: &str) -> PreflightCheck {
    let zero = "0000000000000000";
    let observed: Vec<String> = CAPABILITY_SETS
        .iter()
        .map(|set| format!("{set}={}", status_field(status, set).unwrap_or("missing")))
        .collect();
    let ok = CAPABILITY_SETS
        .iter()
        .all(|set| status_field(status, set) == Some(zero));
    PreflightCheck {
        name: "capabilities_empty",
        expected: format!("every set {zero}"),
        observed: observed.join(" "),
        ok,
    }
}

fn uid_check(status: &str) -> PreflightCheck {
    // The first field of `Uid:` is the real UID.
    let real = status_field(status, "Uid")
        .and_then(|value| value.split_whitespace().next())
        .unwrap_or("missing");
    exact_check("sandbox_uid", SANDBOX_UID.to_string(), real)
}

fn network_check(net_dev: &str) -> PreflightCheck {
    // Only the loopback interface may exist. Each interface line has a colon after its name.
    let names: Vec<&str> = net_dev
        .lines()
        .filter_map(|line| line.split_once(':').map(|(name, _)| name.trim()))
        .collect();
    let ok = !names.is_empty() && names.iter().all(|name| *name == "lo");
    PreflightCheck {
        name: "network_interfaces",
        expected: "lo only".to_owned(),
        observed: names.join(","),
        ok,
    }
}

/// The root filesystem must be mounted read-only.
fn root_check(mounts: &str) -> PreflightCheck {
    let root = mount_for(mounts, "/").map(|(_, options)| options);
    let ok = root.is_some_and(|options| options.split(',').any(|opt| opt == "ro"));
    PreflightCheck {
        name: "root_read_only",
        expected: "/ mounted ro".to_owned(),
        observed: root.unwrap_or("/ not listed").to_owned(),
        ok,
    }
}

/// `/box` and `/tmp` must be tmpfs mounts with `nosuid` and `nodev`, and `/box` must have the
/// policy's size.
fn scratch_check(mounts: &str, policy: &ContainerPolicy) -> PreflightCheck {
    let box_mount = mount_for(mounts, "/box");
    let box_options = box_mount.map(|(_, options)| options);
    let want_box = policy
        .tmpfs
        .iter()
        .find(|mount| mount.path == "/box")
        .map(|mount| mount.size_bytes);
    let box_ok = match (box_mount, want_box) {
        (Some((fs_type, options)), Some(want)) => {
            fs_type == "tmpfs" && scratch_options_ok(options, want)
        }
        _ => false,
    };
    let tmp_ok = mount_for(mounts, "/tmp").is_some_and(|(fs_type, options)| {
        fs_type == "tmpfs" && has_tokens(options, &["nosuid", "nodev"])
    });
    PreflightCheck {
        name: "scratch_tmpfs",
        expected: "/box and /tmp tmpfs, nosuid,nodev; /box sized by policy".to_owned(),
        observed: format!(
            "/box: {} | /tmp: {}",
            box_options.unwrap_or("not a mount"),
            mount_for(mounts, "/tmp").map_or("not a mount", |(_, options)| options)
        ),
        ok: box_ok && tmp_ok,
    }
}

fn scratch_options_ok(options: &str, want_size: u64) -> bool {
    let tokens: Vec<&str> = options.split(',').collect();
    let size = tokens
        .iter()
        .find_map(|token| token.strip_prefix("size="))
        .and_then(parse_size);
    has_tokens(options, &["nosuid", "nodev"]) && size == Some(want_size)
}

fn has_tokens(options: &str, wanted: &[&str]) -> bool {
    wanted
        .iter()
        .all(|token| options.split(',').any(|opt| opt == *token))
}

fn write_refused_check(touch_exit: i64) -> PreflightCheck {
    PreflightCheck {
        name: "read_only_write_refused",
        expected: "non-zero exit".to_owned(),
        observed: touch_exit.to_string(),
        ok: touch_exit != 0,
    }
}

/// The filesystem type and option text of the mount at `path`, from a `/proc/mounts`-style text.
fn mount_for<'a>(mounts: &'a str, path: &str) -> Option<(&'a str, &'a str)> {
    mounts.lines().find_map(|line| {
        let mut fields = line.split_whitespace();
        let _device = fields.next()?;
        let mount_point = fields.next()?;
        let fs_type = fields.next()?;
        let options = fields.next()?;
        (mount_point == path).then_some((fs_type, options))
    })
}

fn exact_check(name: &'static str, expected: String, observed: &str) -> PreflightCheck {
    PreflightCheck {
        name,
        ok: observed == expected,
        expected,
        observed: observed.to_owned(),
    }
}
