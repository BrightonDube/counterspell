# Hints settings (operator notes)

The hints feature (ladder reveals and AI hints) reads its settings once at start-up. AI is **off by default**: `HINT_PROVIDERS` is empty in `docker-compose.yml`, so no provider is called until an operator turns it on. Ladder reveals work with AI off.

Keys and settings come from the host environment at deploy time. They never go in a file in this repository, and `docker-compose.yml` only passes them through with `${NAME:-}`. Empty values count as not set.

## Variables

| Variable | Default | Passed by compose | Notes |
|---|---|---|---|
| `HINT_PROVIDERS` | empty (AI off) | yes | Ordered list, for example `openai:gpt-6-luna,groq:openai/gpt-oss-20b`. The first is the primary, the second the fallback. |
| `OPENAI_API_KEY` | none | yes | Required when `openai` is listed. Never logged. |
| `GROQ_API_KEY` | none | yes | Required when `groq` is listed. Never logged. |
| `AI_STRICT_CONFIG` | `true` | yes | `true`: refuse to start if a listed provider has no key. `false`: leave that provider out and log a warning. |
| `OPENAI_BASE_URL` | `https://api.openai.com/v1` | no | Must start with `https://`. Confirm `gpt-6-luna` is served on Chat Completions before the first real call. |
| `GROQ_BASE_URL` | `https://api.groq.com/openai/v1` | no | Must start with `https://`. |
| `HINT_TIMEOUT_MS` | `8000` | no | Per provider attempt, 1000 to 15000. |
| `HINT_MAX_OUTPUT_TOKENS` | `300` | no | Sent as `max_tokens`, 1 to 300. |
| `AI_DAILY_LIMIT` | `10` | no | AI generations per learner per UTC day. The familiar shares it. |
| `AI_DAILY_SPEND_CAP_USD` | `5.00` | no | All learners together, per UTC day. Up to six decimal places. |
| `HINT_PRICE_OPENAI_IN_PER_M` / `_OUT_PER_M` | `0.10` / `0.50` | no | US dollars per million tokens. |
| `HINT_PRICE_GROQ_IN_PER_M` / `_OUT_PER_M` | `0.075` / `0.30` | no | US dollars per million tokens. |
| `HINT_WINDOW_LIMIT` | `30` | no | Hint requests per learner per 10 minutes. |

Settings marked "no" are not passed by compose. To change one, add a pass-through line for it in the `server` environment, using the `${NAME:-default}` form.

## Valkey persistence

The Valkey service runs with `--appendonly yes`, so the daily quota and spend counters survive a restart. Without it, a restart could reset the counters and allow extra spend on that day.

## Checks before enabling AI

1. Confirm the model names and base URLs against the provider's current documentation.
2. Set `HINT_PROVIDERS` and the matching keys in the host environment, then restart the server.
3. Sign in with the demo account, reveal rung 1, and request one AI hint. Check the `AI hint` label and the quota count on the response.
4. Run `docker compose logs server` and confirm it has no key, prompt or learner code in it.
