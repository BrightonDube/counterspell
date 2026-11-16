# judge-sandbox T00 — trusted/untrusted boundary review (for Brighton's sign-off)

Prepared by Devin (driver/reviewer) on 14 Nov 2026. This document is the T00
input required by `.kiro/specs/judge-sandbox/tasks.md`: a review of the
trusted/untrusted boundary, the threat model, isolation boundaries, resource
limits and network policy, plus a recommendation for every open decision in
`design.md`.

**Nothing in judge-sandbox may start until T00 is signed off.** Sign-off means
Brighton answers the required open questions and the answers are recorded as a
"Review decisions" note appended to `.kiro/specs/judge-sandbox/design.md`, or T00
is ticked in `tasks.md`. If any answer weakens or changes an isolation control,
`requirements.md` must be updated before T01. (T00 also asks for a click of
**Analyze Requirements** on `requirements.md` in the Kiro IDE — the CLI could
not do it.)

Sources reviewed: `requirements.md`, `design.md` (all sections, especially 1–4,
7, 9, 13, 16–20 and the open questions), `tasks.md`, `deploy/rootless-sandbox-daemon.md`,
`.kiro/steering/{security,sandbox}.md`, `docs/system-design.md` §5–§7.

---

## 1. What is being approved

A Python-only execution service inside `counterspell-judge` that runs hostile
submitted code and produces verdicts user code cannot forge. The trusted judge
worker owns tests, expected outputs, the comparator, deadlines and the final
verdict. Submitted programs only read one input at a time and print one raw
answer. All execution happens in throwaway containers on a **separate rootless
Docker daemon** used for nothing else.

The spec is unusually well-armoured for a first sandbox: verdict authority is
concentrated in two pure functions (`compare` mints `Pass`, `verdict` mints
`Accepted`), every Docker request is built from typed values and proven
independent of submitted source (P4), a kernel-facts probe verifies the limits
were actually applied (not just configured), and 27 real-sandbox scenarios plus
14 model properties are specified as evidence.

## 2. Threat model (design.md §2, condensed)

**Assets:** verdict integrity; confidentiality of hidden tests/expected
outputs; the VPS host, its secrets and app data; availability until 29 Jan 2027;
isolation between concurrent users' runs.

**Adversaries:** A1 a learner submitting arbitrary Python; A2 the same after a
partial container escape; A3 a different concurrent user's run sharing the
kernel/daemon; A4 an agent driving `counterspell-mcp` (treated exactly like A1);
A5 a pack author (semi-trusted; references still run inside the sandbox).
Out of scope: compromised judge host, Postgres, or VPS root, malicious
Docker/kernel maintainer.

**Key attack → control pairs** (the full table has 20 rows):

| Attack | Control | Backstop |
|---|---|---|
| Forge a verdict/answer | `Pass` mintable only in `compare`; `Accepted` only in `verdict`, from worker-held facts | Strict one-line protocol; wrong id/extra line = protocol violation |
| Infinite loop / sleep bomb | Worker-side deadline, kill from outside, run cap | PID 1 `sleep` self-expires container |
| Memory exhaustion | cgroup `memory.max` = `mem_mb`, swap = memory (no swap) | Kernel `oom_kill` counter read by worker |
| Fork/thread bomb | cgroup `pids.max` = 64, CPU quota 1 CPU | **No `nproc` ulimit** (see Q1); S18 proves slot isolation |
| Output flood | Bounded reader: 1 MiB stdout per call, 64 KiB stderr per session | Log driver `none` |
| Network access | Network mode `none` | Probe sees only `lo`; S12 host-listener evidence |
| Write outside scratch | Read-only rootfs, `nosuid,nodev` tmpfs, all caps dropped, no-new-privileges | Daemon filesystem-change report must be empty |
| Steal hidden tests/expected outputs | They never enter the container; hidden results carry no text | Redaction-canary property P12 |
| Leak data to next run | Fresh container per run, never pooled or reused | Scratch is tmpfs, dies with container |
| Starve the other slot | Per-container quotas; no shared `nproc`; container cap | Aggregate limits on `sandbox` user's systemd slice |
| Orphan containers after crash | PID 1 self-expires (≤ ~25 s); sweeper removes labelled containers > 40 s every 10 s | Start-up sweep; bound ≈ 50 s |
| Infra fault masquerading as wrong answer | `InfraError` is a separate type, cannot become a test verdict | Fault-injection property P9 |
| Trick judge into running code on host | No process spawning (Clippy ban + source test); typed requests only | `counterspell-mcp` shares the same path |
| Injection via names/arguments | File names validated, fixed argv, source only as stdin data | Request-independence property P4 |
| Escape via daemon API | Unix socket path pinned by env, no `DOCKER_HOST`, rootless daemon | Policy read-back + kernel-facts probe |
| Keyword evasion | **No blocklist exists** — sandbox is the boundary | S27 positive control |
| Compile-time abuse | Size limit, compile budget, compile inside sandbox | `compile_error` verdict |

## 3. Isolation boundaries (design.md §1)

Five zones, strictly one-directional for trust:

- **Z0 browser** — untrusted; sends code only.
- **Z1 `counterspell-server`** — semi-trusted; validates, rate-limits, enqueues.
  Must never touch Docker, hold the sandbox socket, depend on `bollard`, or
  write a verdict. (Enforced by boundary test T36.)
- **Z2 `counterspell-judge`** — fully trusted; holds tests/expected
  outputs/comparator/limits; the only verdict authority. No inbound port; only
  the limited Postgres role; no Valkey/OAuth/email/LLM secrets.
- **Z3 sandbox daemon** — rootless dockerd owned by unprivileged `sandbox`
  user, own socket, sandboxes only, separate data root.
- **Z4 container** — hostile; reads stdin, writes stdout/stderr, bounded by
  quota. Must never learn an expected output, reach a network, write outside
  scratch, or declare a verdict.

**Information flow:** the container never receives expected outputs, the
comparator, the Python reference (in the *user's* container — the reference
gets its own container during ghost hunts), secrets, bind mounts or sockets.
Everything crossing Z4→Z2 is untrusted bytes read through size-bounded readers.

## 4. Resource limits (design.md §7, essentials)

One function, `ContainerPolicy::for_language`, builds every container — no
other code path creates one. The policy is plain data, unit-tested, and checked
twice: before the request is sent and again on the daemon's read-back
(`inspect`); any difference removes the container and yields `InfraError`.

| Control | Value |
|---|---|
| Image | Digest-pinned only (`@sha256:` + 64 hex); inspected before use; never pulled at run time |
| PID 1 / Cmd | `sleep <remaining run cap + 5 s>` — user code never runs as PID 1; self-expires if worker dies |
| User | `65534:65534` for container, driver and every exec |
| Network | `none` + networking disabled |
| Rootfs | read-only; scratch only on tmpfs: `/box` 64 MiB (`noexec` for Python), `/tmp` 16 MiB, `/dev/shm` 16 MiB — all `nosuid,nodev` |
| Caps / privileges | drop ALL, add none; `no-new-privileges`; default seccomp (never `unconfined`, no custom profile); privileged off; no devices/binds/volumes/extra hosts/links; private PID/IPC/cgroup namespaces |
| CPU | quota 1 CPU (`NanoCpus = 1e9`) — a quota, not pinning (documented as such) |
| Memory | `mem_mb` (256 for Python); memory+swap equal → no swap; tmpfs counts against it |
| PIDs / fds / files | `pids.max` 64; `nofile` 64; `fsize` 16 MiB; core dumps off; **no `nproc`** |
| Misc | log driver `none`; no restart/auto-remove; fixed env (`PATH`, `LANG`, `HOME` only); managed labels + run/worker/expiry labels; generated names contain no user text |

**Time bounds:** per-call limit = problem `time_ms` else language default,
clamped 100 ms–10 s; call verdict ≤ limit + `KILL_SLACK` (1 s); run cap =
`min(20 s, tests × limit + compile budget + 2 s)`; verdict ≤ cap + slack;
fail-fast after 3 consecutive abnormal tests; fresh session after every
abnormal end while the cap allows.

**Output bounds:** 1 MiB stdout per call, 64 KiB stderr per session — either
breach kills the container with `output_limit`. Worker memory stays bounded
during floods (P5, S09–S11).

## 5. Network policy

- Every sandbox: network mode `none`, networking disabled. No interfaces but
  `lo` (probe verifies via `/proc/self/net/dev`).
- S12 is the proof: hostile programs attempt TCP, UDP, DNS, IPv6, raw sockets,
  link-local/metadata addresses, bridge gateway and host loopback; pass =
  **zero** connections at host listeners. A trusted control run with networking
  deliberately enabled proves the listener *can* see a connection — if the
  control is inconclusive on the VPS the report says so rather than passing
  silently.
- The judge itself reaches the daemon only via `SANDBOX_DOCKER_SOCKET` (Unix
  socket, required, no default, refuses `/var/run/docker.sock`,
  `/run/docker.sock`, and anything non-Unix; `DOCKER_HOST` is never read).
- Sandboxes never pull images; no package installation, ever.

## 6. Preflight (the "limits must actually work" check)

Docker **silently ignores** cgroup flags when v2 delegation is missing, so the
design verifies behaviour, not configuration. At start-up, every 60 s, and via
`counterspell-judge preflight`, a probe container built by the *same* policy
function reads kernel facts from inside: `NoNewPrivs`, `Seccomp: 2`, zeroed
capability sets, non-root UID, only `lo`, `memory.max`/`swap.max`/`pids.max`/
`cpu.max` equal to policy, read-only root, expected tmpfs options, and a
write-to-`/usr` that must fail. Daemon `info` must show rootless, cgroup v2,
non-`none` driver, own data root, registered runtime, no limit-support
warnings. **Any failure → every run is `internal_error`; there is no skip or
downgrade flag.** A negative test proves the probe fails against a deliberately
weakened policy.

## 7. Deviations from `docs/system-design.md` (all need awareness; none remove a control)

| # | Change | Assessment |
|---|---|---|
| D1 | No `nproc` ulimit | Correct call — see Q1. PID cgroup is the control; S18 proves it. |
| D2 | `AutoRemove` off | Needed to read exit state/OOM evidence for honest verdicts; cleanup is explicit + self-expiry + sweeper. |
| D3 | Python gets worker-run `py_compile` | Makes `compile_error` a worker-observed fact. **Changes `language-drivers`' `compile = []` — coordinate.** |
| D4 | Strict digest pinning, no override | Per the brief; adds a weekly digest-refresh chore (T38 documents it). |
| D5–D8 | `noexec` Python scratch; `/box` owned 0700; tar-over-stdin delivery; per-call/per-session output units | All tighten or clarify; tar method is spike-verified in T02. |
| D9 | tmpfs counts against the 256 MiB memory limit | Documented; ~96 MiB max scratch, ≥160 MiB heap remains. Adjustable. |
| D10–D11 | Sweeper at 40 s/10 s; fresh container per run, never pooled | Both tighten cleanup/isolation vs the docs' looser wording. |

## 8. Open questions — recommendations

T00 requires answers to **questions 1–7 and 16** (the ⚠ isolation items).
Questions 8–15 and 17–25 are listed for completeness; their assumed choices
are reasonable and none weakens a control.

### Required (⚠)

| # | Question | Assumed choice | My recommendation |
|---|---|---|---|
| 1 | No `nproc` ulimit | Rely on `pids.max` = 64; prove with S18 | **Accept.** `nproc` is per-UID in the user namespace and all slots share UID 65534 — it would let one fork bomb starve the other slot. The cgroup limit is per-container and correct. |
| 2 | One UID (65534) for PID 1, driver, execs | Yes; a program may end its own container | **Accept.** Killing PID 1 only kills that run's own container → `runtime_error`. A separate PID-1 UID adds moving parts for no isolation gain (same namespaces). |
| 3 | Python worker-run `py_compile` | Yes; whichever spec lands second updates the other | **Accept.** Verdict integrity is asset #1 — `compile_error` must be a fact the worker observed, not something program output can claim. Note: this overrides `compile = []` in `language-drivers`; judge-sandbox lands first in the priority order, so language-drivers must adopt `py_compile`. |
| 4 | Strict digest pinning, no override | Strict, per the brief | **Accept.** The cost is operational (weekly digest refresh, T38), not security. A tag override would make image integrity unverifiable. |
| 5 | Fresh container per run, never pooled | Yes | **Accept.** Cold-start cost is small next to the contamination risk of pooling across users. A pool would need its own threat analysis — not worth it now. |
| 6 | tmpfs counts against 256 MiB memory | Accept | **Accept.** Bounded and documented; per-language sizes are configurable. |
| 7 | Preflight fail-closed, no skip flag | Fail closed; use VPS/CI for `sbx_` tests | **Accept.** This is the correct posture for a security boundary — a host that can't prove its limits gets no user code. Known cost: on a misconfigured host, runs end `internal_error` after retries. Optional future improvement (not required): a run-lifecycle "judge not ready" gate so users don't burn attempts. |
| 16 | `AutoRemove` off | Accepted | **Accept.** Without the exit state you cannot distinguish "user killed their own container" from "daemon failed" — that distinction is what keeps infra faults out of user verdicts. Explicit remove + self-expiry + sweeper covers cleanup (S16 proves the sweeper alone). |

### Informational (assumed choices look right; confirm or adjust)

| # | Topic | Recommendation |
|---|---|---|
| 8 | Fail-fast after 3 abnormal tests | Accept — cuts hostile-run cost ~20 s → ~7 s; configurable. |
| 9 | stderr > 64 KiB → `output_limit` | Accept — matches system design. |
| 10 | Unexecuted tests omitted from events/`test_results` | Accept — no `not_run` verdict exists in run-lifecycle; `passed`/`total` show the gap. |
| 11 | VAL-1–4 stay implemented in run-lifecycle | Accept — one implementation each; this spec adds re-check + shared vectors only. |
| 12 | `memory_kb` stays empty | Accept — `memory.peak` from inside the container is a possible later addition. |
| 13 | Judge re-check failure → `compile_error` + fixed message | Accept — defence in depth; should be unreachable for API-passed requests. |
| 14 | `sbx_` CI job non-blocking; VPS run authoritative | Accept — honest; GitHub runners may lack cgroup delegation. |
| 15 | No socket-filtering proxy | Accept **with** the stated deployment mitigations (sandbox user's systemd slice CPU/mem/task limits, empty home, app secrets mode 0600 owned by another user). Residual blast radius = the `sandbox` OS user. |
| 17 | Driver persists between tests of one run | Accept — the interactive ghost protocol needs it; statefulness is documented residual risk #5. |
| 18 | `python -I` hash randomisation | Accept — documented; ghost double-check absorbs it. |
| 19 | `SandboxOracle` `Handle::block_on` inside `spawn_blocking`; rejects async context | Accept — matches ghost-engine; the async-context check prevents deadlock. |
| 20 | gVisor selectable but untested | Accept — reported as untested until S23 passes on the VPS. Never claim it. |
| 21 | tar-over-exec vs archive upload for file delivery | Defer to T02 spike — correct. |
| 22 | Per-call clamp 100 ms–10 s | Accept. |
| 23 | Image supplies `sleep`/`tar`/`cat` | Defer to T02 verification for the pinned image. |
| 24 | `ProblemSpec.reference_python` used only by `SandboxOracle` | Accept — preserves the information-flow rule (reference never enters the user's container). |
| 25 | Reconciliations with run-lifecycle/language-drivers | Informational — noted. |

## 9. Reviewer's overall recommendation

**Approve the spec as designed, with all assumed choices accepted.** The
design is conservative where it matters: verdict authority is concentrated in
two pure, property-tested functions; the container policy is a single checked
constructor with read-back verification; the probe proves limits empirically;
fail-closed preflight has no bypass; and the evidence plan (P1–P14, S01–S27,
honest `verification-report.md`) refuses to claim what wasn't measured.

Points Brighton may want to weigh before signing:

1. **Q3/D3 is the only deviation that changes another spec.** `language-drivers`
   currently says Python `compile = []`; this spec replaces it with a
   worker-run `py_compile`. Since judge-sandbox is ahead of language-drivers in
   the priority order, language-drivers must adopt the new contract. No action
   needed now — just don't be surprised when its spec is updated.
2. **Q4 means a weekly digest-refresh chore** on the VPS (T38 documents the
   procedure). Missed refreshes don't break security — the pinned digest keeps
   working — they just delay image updates.
3. **Q7's cost is honest but real:** a broken sandbox host turns user runs into
   `internal_error` after retries. T02 (host spike) and T22/T37 (preflight +
   VPS verification) are `[HUMAN]` tasks — they need you on the VPS. Plan for
   that.
4. **Residual risks accepted by design** (§18): kernel/runc escape limited to
   the `sandbox` user; no side-channel mitigation; stateful programs within one
   run; timing noise on a shared VPS; preflight is point-in-time (60 s window,
   narrowed by per-container read-back).

## 10. What happens on sign-off

1. Brighton answers Q1–Q7 and Q16 (and optionally adjusts others), and the
   answers are appended to `design.md` as the "Review decisions" note — or T00
   is ticked. Any answer that weakens a control requires a `requirements.md`
   update first.
2. Kiro may then start T01 (judge deps/config — model work) and T02 (your VPS
   host spike — `[HUMAN]`). T03 (image digest pin) follows T02.
3. Implementation runs SUPERVISED where marked; wave 3+ needs the real sandbox
   daemon on the VPS.

— Devin, driver/reviewer. Questions or changes to this review: reply and I'll
update the document.
