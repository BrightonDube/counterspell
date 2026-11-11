# Hook questions and choices

The playbook's A.4 table specifies the 17 event names, matchers, and command behaviour, but it does not show a complete v1 JSON example or state whether the event field is named `trigger.type` or `when.type` in the installed Kiro build. Choice for this change: use one JSON object per file with `version: "v1"`, `trigger.type`, optional `trigger.matcher`, and `action.type: "command"` plus `action.command`. Verify the field names in the target IDE/CLI hook schema before enabling the hooks.

A.4 says `stop-session-log` uses `confirm` and `confirmCommand`, but the task also requires command actions everywhere and this run is non-interactive. Choice: the command script checks for tracked changes and logs only when changes exist; it does not add an unverified interactive confirmation field. If the installed schema supports confirmation metadata, add it after checking the official schema.

The `UserPromptSubmit` event's prompt field is not named in A.4. Choice: `prompt-secret-filter.sh` checks the complete stdin JSON and the optional `USER_PROMPT` environment variable, so it works with either payload shape without depending on a single field name.

The write/read payload path field is also unspecified. Choice: `guard-secrets.sh` uses jq to inspect `path`, `filePath`, `target`, and `uri` values recursively, then falls back to the complete event JSON. `.env.example` is explicitly allowed; other `.env*`, key, and `secrets/` paths are blocked.

The PostTaskExec payload field names for the spec/task ID and verification result are not specified. Choice: `task-done-changelog.sh` searches string values for task/spec and verification text and records `not reported` when the event does not provide a value. It records the current Git commit when Git is available and never invents a pass result.

The table says `{{filePath}}` works for file triggers. Choice: file-trigger commands pass that placeholder as an argument to the script, while the script also reads stdin and uses common JSON path fields as a fallback.
