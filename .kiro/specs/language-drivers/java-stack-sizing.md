# Java `-Xss` sizing evidence (T14)

Deep-recursion fixture: `crates/judge/tests/fixtures/solutions/deep_recursion.java`
(linear recursion `depth(n)`, called with n = 200 000–300 000 through the real
`Driver.java` inside the digest-pinned `eclipse-temurin:21-jdk` image,
`-Xmx256m -XX:+UseSerialGC`, same flags as `languages.toml`).

| `-Xss` | Result at depth 200k | Result at 250k–300k |
|---|---|---|
| 8m | intermittent StackOverflowError (2/3 pass) | fail |
| 10m | fail (0/3) | fail |
| 12m | intermittent (2/3 pass) | fail |
| 13m | intermittent (2/3 pass), fails at 300k | fail |
| 14m | pass 3/3 | pass 250k and 300k |
| 16m | pass 3/3 | pass 250k and 300k |

**Decision: `-Xss14m`** — the smallest candidate that passed every designated
fixture run (three repeats at 200k, plus 250k and 300k margin checks).
8–13 MiB are nondeterministic at 200k and must not be chosen.

`-Xss` only sets per-thread stack space; it does not weaken the `mem_mb = 512`
or `time_ms_per_test`/`max_time_ms_per_test` sandbox limits. Recorded in
`languages.toml` `run` for the still-disabled `[java]` entry.
