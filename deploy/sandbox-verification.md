# Sandbox verification (results only)

## Run 1: LOCAL EQUIVALENT, not the VPS (14 Nov 2026)

Host: DESKTOP-P3OVGMG (Brighton's PC), WSL2 kernel 6.18.33.2-microsoft-standard-WSL2, Ubuntu 26.04.1 LTS, systemd as PID 1.
Date (UTC): 2026-11-14 19:46. Docker version: 29.4.3 (runc 1.3.5, libseccomp 2.6.0).
Run by: Brighton's assistant, on Brighton's instruction to unblock judge-sandbox while no VPS exists.

**Important difference from the target:** this is the normal (rootful) Docker daemon in WSL, not a rootless
sandbox daemon. Rootless Docker cannot be set up here yet because the `uidmap` package (newuidmap) is missing
and installing it needs Brighton's sudo password. Every rootless-specific answer below is therefore
NOT TESTED, and the whole table must be re-run on the VPS (T02 stays open as "re-verify on VPS", T37 is the gate).
Only trusted probe programs were run. Nothing hostile ran anywhere.

Image used: `python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90` (linux/amd64 resolved locally).

## T02 answers

| # | Command (shortened) | Output (trimmed) | Result |
|---|---|---|---|
| host | `date -u; uname -a; os-release` | see header | PASS |
| 2 | `docker info --format '{{json .SecurityOptions}} {{.CgroupVersion}} {{.CgroupDriver}} {{.DockerRootDir}}'` | `["name=seccomp,profile=builtin","name=cgroupns"] 2 systemd /var/lib/docker` | cgroup v2 + systemd + seccomp PASS. `name=rootless` absent and root dir is `/var/lib/docker`: rootless **NOT TESTED** (expected, rootful local daemon) |
| 3 | `cat .../user@1000.service/cgroup.controllers` | `cpu memory pids` | PASS (cpu, memory, pids delegated to the user; `cpuset io` not delegated, drop-in not applied) |
| 4 | trusted probe with `--network none --read-only --tmpfs /box:...noexec... --user 65534:65534 --cap-drop ALL --security-opt no-new-privileges:true --memory 256m --memory-swap 256m --pids-limit 64 --cpus 1` | `268435456` / `64` / `100000 100000`; `CapEff: 0000000000000000`, `CapBnd: 0`, `NoNewPrivs: 1`, `Seccomp: 2`; `/` mounted `ro`; `/box` tmpfs `rw,nosuid,nodev,noexec,size=65536k,mode=700,uid=65534,gid=65534` | PASS (all expected values) |
| 5.1 | `tar -cf - hello.py \| docker run -i --rm --read-only --tmpfs /box ... sh -c 'cd /box && tar -xf -'` | `hello.py` owned by nobody, `python3 hello.py` printed `hi`, exit 0 | PASS. Also PASS as `docker exec -i` into a running read-only container (rc 0) |
| 5.2 | `docker cp - <id>:/box < probe.tar` into a running read-only container | `Error response from daemon: container rootfs is marked read-only`, rc 1; `/box` stayed empty | FAIL (archive upload does not work with a read-only root) |
| 5.3 | `--memory 64m`; `docker exec <id> python3 -c 'x=bytearray(200*1024*1024)'` | exec rc 137; `memory.events`: `max 22`, `oom 1`, `oom_kill 1`; `OOMKilled=true` | PASS: `oom_kill` counts a non-PID-1 kill, and Docker 29 also reports `OOMKilled=true` |
| 5.4 | `for c in sleep tar cat touch; do command -v $c; done` | `/usr/bin/sleep /usr/bin/tar /usr/bin/cat /usr/bin/touch` | PASS |
| 5.5 | `--read-only --tmpfs /box:rw,mode=0700,uid=65534,gid=65534 --user 65534:65534 ... stat -c "%u %g %a" /box` | `65534 65534 700` | PASS on rootful. Rootless **NOT TESTED** |
| 5.6 | `cargo tree -p counterspell-judge -i bollard --features sandbox -e normal` | `bollard v0.21.1` used by `counterspell-judge v0.1.0` | PASS |

## Decision: file delivery = tar over exec

Reason: tar over exec works (5.1), and archive upload into a read-only container's tmpfs is refused by the daemon (5.2). This matches design answer 21. Re-check 5.1 and 5.5 on the rootless VPS daemon.

## T03 digest: python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90

- Source: Docker Hub registry, manifest for `library/python:3.13-slim` (OCI image index, multi-arch), header `Docker-Content-Digest`, read 14 Nov 2026 21:45 SAST. Docker Hub shows the tag last updated 2026-11-08 02:07 UTC.
- Check: `docker pull python:3.13-slim@sha256:e48f...` then `docker image inspect --format '{{json .RepoDigests}}'` gave `["python@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90"]` on the local daemon. Same digest.
- Refresh: Renovate (`renovate.json`) opens a weekly digest-only pull request for `languages/languages.toml`; a human merges it; never auto-merged (judge-sandbox amendment A3). Renovate only starts once the free Renovate GitHub app is installed on the repository.
- On the VPS: pull by this digest into the rootless sandbox daemon and repeat the inspect check (T37).

## Run 2: official digest pins for Node, Temurin and GCC (15 Nov 2026)

Host: same WSL Docker daemon as run 1. Date: 15 Nov 2026 00:51 SAST.
Method: same as T03. For each tag, the Docker Hub `Docker-Content-Digest` (image index) was read, then `docker pull <name>@<digest>` and `docker image inspect --format '{{json .RepoDigests}}'` confirmed the same digest.

| Image | Digest | Inspect |
|---|---|---|
| `node:22-slim` | `sha256:e9f565d32859f0cbfb52125eafe8be39390f24dac302dc3cdf566e8ba3ab21ee` | `["node@sha256:e9f565d32859f0cbfb52125eafe8be39390f24dac302dc3cdf566e8ba3ab21ee"]` |
| `eclipse-temurin:21-jdk` | `sha256:0a3f329821dcadf54b2a3033a15e2e9f007069a9db43d7f51897e6b80e726b07` | `["eclipse-temurin@sha256:0a3f329821dcadf54b2a3033a15e2e9f007069a9db43d7f51897e6b80e726b07"]` |
| `gcc:14` | `sha256:a6f425a380ee03e682df3b436fc9077c544fb0fea0a9cab943c3b8320db10765` | `["gcc@sha256:a6f425a380ee03e682df3b436fc9077c544fb0fea0a9cab943c3b8320db10765"]` |

These are recorded in `languages/languages.toml`. The JavaScript, Java and C++ entries stay `enabled = false` until their drivers exist. No container from these images was used to run untrusted code. Rootless behaviour is still NOT TESTED (see run 1).


## Run 3: local rootless daemon for user brighton (15 Nov 2026 01:00 SAST)

Host: same WSL machine as run 1. Brighton had installed `uidmap`. The rootless extras package was already present, so `dockerd-rootless-setuptool.sh install --force` was used (without sudo, and without stopping Docker Desktop). The tool created CLI context `rootless` and briefly selected it; the default context was immediately restored to `unix:///var/run/docker.sock`. Desktop containers (`cs-dev-pg`, `cs-dev-valkey`, and the unrelated bizpilot/evorayn containers) stayed up. No `DOCKER_HOST` was written to a shell rc. Linger was already yes.

- User service: `systemctl --user enable --now docker` (active, enabled). Engine 29.4.3, rootlesskit 2.3.6, slirp4netns, storage overlayfs, data root `/home/brighton/.local/share/docker`.
- Socket: `/run/user/1000/docker.sock` (also `$XDG_RUNTIME_DIR/docker.sock`).
- `docker --context rootless info`: SecurityOptions include `name=rootless`; CgroupVersion=2; CgroupDriver=systemd; DockerRootDir=`/home/brighton/.local/share/docker`.
- Warnings (identity check will refuse this daemon until they are gone): `No cpuset support`; `No io.weight support`; `No io.weight (per device) support`; `No io.max (rbps/wbps/riops/wiops) support`.
- `user@1000.service` cgroup.controllers: `cpu memory pids` only. `cpuset` and `io` are not delegated. The sudo drop-in for `Delegate=cpu cpuset io memory pids` is recorded in `brighton-decisions.md`; it was not applied because it needs Brighton's sudo.
- `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock` is set in `~/.config/counterspell/dev-test.env` (outside the repo).
- `sbx_*` tests: none present in the tree yet, so they were not run. A compile-only `cargo test -p counterspell-judge --features sandbox -- --ignored sbx` in an isolated `/tmp` target filled the 3.9G tmpfs and was deleted; the repo `target/` and kiro-impl were not used.
- T02 and T37 stay open for the real VPS. This run is not a substitute for T37.

## Run 4: target VPS `shared-vps` (T02 re-verify, 19 Nov 2026)

Host: `shared-vps` (`Linux shared-vps 7.0.0-43-generic x86_64`, Ubuntu, GLIBC 2.43).
User: `sandbox` (uid 1002), socket `unix:///run/user/1002/docker.sock`.
Date (UTC): 2026-11-19 11:43. Docker version: 29.9.3.

| # | Command / Check | Output (trimmed) | Result |
|---|---|---|---|
| 2 | `docker info --format "{{json .SecurityOptions}} {{.CgroupVersion}} {{.CgroupDriver}} {{.DockerRootDir}}"` | `["name=seccomp,profile=builtin","name=rootless","name=cgroupns"] 2 systemd /home/sandbox/.local/share/docker` | **PASS** (rootless, cgroup v2, systemd, seccomp) |
| 3 | `cat /sys/fs/cgroup/user.slice/user-1002.slice/user@1002.service/cgroup.controllers` | `cpuset cpu io memory pids` | **PASS** (all 5 controllers delegated) |
| 4 | Trusted probe (`--network none --read-only --tmpfs /box:... --user 65534:65534 --cap-drop ALL --security-opt no-new-privileges:true --memory 256m --memory-swap 256m --pids-limit 64 --cpus 1`) | `memory.max=268435456`, `pids.max=64`, `cpu.max=100000 100000`, `CapEff: 0`, `CapBnd: 0`, `NoNewPrivs: 1`, `Seccomp: 2`, `box_stat=65534 65534 700` | **PASS** |
| 5.1 | `tar -cf - hello.py \| docker exec -i <cid> sh -c "cd /box && tar -xf - && python3 hello.py"` | `hi`, exit 0 | **PASS** (tar over exec works in rootless mode) |
| 5.3 | `--memory 64m --memory-swap 64m`; `python3 -c "x=bytearray(200*1024*1024)"` | exec rc `137`, `memory.events`: `max 37`, `oom 1`, `oom_kill 1`, `OOMKilled=true` | **PASS** |
| 5.4 | `for c in sleep tar cat touch; do command -v $c; done` | `/usr/bin/sleep /usr/bin/tar /usr/bin/cat /usr/bin/touch` | **PASS** |
| 5.5 | `stat -c "%u %g %a" /box` | `65534 65534 700` | **PASS** (rootless tmpfs uid/gid/mode honored) |
