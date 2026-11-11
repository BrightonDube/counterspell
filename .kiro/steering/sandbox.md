---
inclusion: fileMatch
fileMatchPattern: ["crates/judge/**", "languages/**", "deploy/**"]
---
# Sandbox boundary

The trusted judge worker owns the tests, expected outputs, comparator, per-test deadlines, and final verdict. It enforces deadlines by killing the sandbox from outside. User code only receives inputs and prints raw answers; output that looks like a verdict is only an answer to compare.

Run sandboxes only through bollard on the separate rootless Docker daemon used for sandboxes. Keep the main Docker daemon, application, database, and secrets out of that boundary. Apply the system-design controls: no network, read-only root, bounded scratch space, non-root user, dropped capabilities, no-new-privileges, seccomp, CPU/memory/PID/wall-time limits, bounded output, and cleanup/reaping.

Do not run user, reference, or MCP code directly on the host.

Source:
#[[file:docs/system-design.md]]
#[[file:docs/requirements.md]]
