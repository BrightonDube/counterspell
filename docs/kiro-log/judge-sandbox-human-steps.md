# judge-sandbox: human steps (run on the VPS, not by Kiro)

Kiro cannot reach the VPS. These steps are for Brighton (or whoever runs the sandbox host). Record every output in `deploy/sandbox-verification.md` (create it from the template at the bottom). Never paste a secret or a `.env` value into that file.

Work only on the **rootless sandbox daemon**. Do not run anything against the main Docker daemon. Do not run hostile programs on the host. The probe programs below are trusted and only print facts.

## T02: host spike (blocks T03, T21, T22)

Run as the `sandbox` user unless a step says otherwise. Replace `$SOCK` with the sandbox daemon socket, for example `/run/user/$(id -u)/docker.sock`.

1. Record the host and date:
   ```bash
   date -u; uname -a; cat /etc/os-release | head -3
   ```
2. Rootless daemon facts:
   ```bash
   DOCKER_HOST=unix://$SOCK docker info --format '{{json .SecurityOptions}} {{.CgroupVersion}} {{.CgroupDriver}} {{.DockerRootDir}} {{.Runtimes}}'
   ```
   Expected: `SecurityOptions` contains `name=rootless`; `CgroupVersion` is `2`; `CgroupDriver` is `systemd`; `DockerRootDir` is not `/var/lib/docker`.
3. Delegated controllers:
   ```bash
   cat /sys/fs/cgroup/user.slice/user-$(id -u).slice/user@$(id -u).service/cgroup.controllers
   ```
   Expected: includes `cpu memory pids` (and `cpuset io` if the drop-in is applied). If `cpu` is missing, apply the drop-in (`Delegate=cpu cpuset io memory pids`) with `systemctl --user edit docker.service`, restart, and record that the drop-in was needed.
4. Trusted probe (prints facts only; pinned Python image, no network, read-only root, limits as designed):
   ```bash
   DOCKER_HOST=unix://$SOCK docker run --rm --network none --read-only \
     --tmpfs /box:rw,nosuid,nodev,noexec,size=64m,mode=0700,uid=65534,gid=65534 \
     --user 65534:65534 --cap-drop ALL --security-opt no-new-privileges:true \
     --memory 256m --memory-swap 256m --pids-limit 64 --cpus 1 \
     --entrypoint sh python:3.13-slim -c 'cat /sys/fs/cgroup/memory.max /sys/fs/cgroup/pids.max /sys/fs/cgroup/cpu.max; grep -E "Cap|NoNewPrivs|Seccomp" /proc/self/status; cat /proc/mounts | grep -E " /box | / "'
   ```
   Record the output. Expected: memory `268435456`, pids `64`, cpu `100000 100000` (one CPU), `CapEff: 0000000000000000`, `NoNewPrivs: 1`, `Seccomp: 2` (filter mode), `/box` mounted with `noexec`.
5. Open questions. Run each and record the output and whether it worked:
   - (1) `tar -x` exec with stdin EOF on the image:
     ```bash
     tar -cf - -C /tmp/probe hello.py | DOCKER_HOST=unix://$SOCK docker run -i --rm --network none --read-only --tmpfs /box:rw,mode=0700,uid=65534,gid=65534 --user 65534:65534 --entrypoint sh python:3.13-slim -c 'cd /box && tar -xf - && ls -la /box'
     ```
     (Create `/tmp/probe/hello.py` with `print("hi")` first. Expected: `hello.py` listed and the command exits 0.)
   - (2) archive upload into a running read-only container's tmpfs: start a container with the same flags and `sleep 60`, then `docker cp` a tar into `/box`. Record success or the exact error.
   - (3) OOM counter: in a container with `--memory 64m`, run `python3 -c 'x=bytearray(200*1024*1024)'` as a non-PID-1 exec. Then `cat /sys/fs/cgroup/memory.events` inside the container (or from the host path of its cgroup) and `docker inspect -f '{{.State.OOMKilled}}' <id>`. Record both values.
   - (4) tools present: `docker run --rm --entrypoint sh python:3.13-slim -c 'command -v sleep tar cat touch'`. Expected: four paths.
   - (5) tmpfs options in rootless mode: `docker run --rm --read-only --tmpfs /box:rw,mode=0700,uid=65534,gid=65534 --user 65534:65534 --entrypoint sh python:3.13-slim -c 'stat -c "%u %g %a" /box'`. Expected: `65534 65534 700`.
   - (6) bollard version and types: record `cargo tree -p counterspell-judge -i bollard` output from the repository checkout.
6. Decide and record: tar over exec (answer 21) or archive upload. If neither (1) nor (2) works, stop and write the blocker to `docs/kiro-log/judge-sandbox-blockers.md`.
7. Mark anything not run as `NOT TESTED`. Commit `deploy/sandbox-verification.md` (results only, no secrets).

## T03: pin the Python image digest (after T02)

1. On the sandbox daemon:
   ```bash
   DOCKER_HOST=unix://$SOCK docker pull python:3.13-slim
   DOCKER_HOST=unix://$SOCK docker image inspect --format '{{index .RepoDigests 0}}' python:3.13-slim
   ```
   Expected: `python@sha256:<64 hex>` (or `python:3.13-slim@sha256:...`).
2. Put the digest into `languages/languages.toml` as `image = "python:3.13-slim@sha256:<64 hex>"`. Only the digest changes.
3. Re-check: `DOCKER_HOST=unix://$SOCK docker image inspect --format '{{index .RepoDigests 0}}' python:3.13-slim` gives the same digest as step 2.
4. Record the digest and date in `deploy/sandbox-verification.md`. Tick T03 only after the digest matches.

## Template for `deploy/sandbox-verification.md`

```markdown
# Sandbox verification (results only)
Host: <hostname>   Date (UTC): <date>   Docker version: <docker version>
## T02 answers
| # | Command | Output (trimmed) | Result |
|---|---|---|---|
| 1 | ... | ... | PASS / FAIL / NOT TESTED |
## Decision: file delivery = tar over exec | archive upload (reason)
## T03 digest: python:3.13-slim@sha256:<digest>
```
