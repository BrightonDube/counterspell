# Judge crate guidance

`crates/judge/` is the security boundary. The trusted worker owns tests, expected outputs, comparison, deadlines, sandbox execution, cleanup, and final verdicts. Keep untrusted code and all code-running MCP requests inside the separate rootless sandbox daemon.
