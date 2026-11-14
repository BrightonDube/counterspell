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
