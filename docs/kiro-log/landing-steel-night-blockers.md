# landing-steel-night: blockers (run of 15 Nov 2026)

Tasks left unchecked on purpose. Each one needs a browser, a running stack or a decision from Brighton. None of them is a failing test.

## T13: Lighthouse mobile median (open)
- **Why open:** the task runs Lighthouse against a served production build in Chrome. That is a localhost browser run, which the test rules put out of scope (no browser or localhost end-to-end testing).
- **Unblock:** run `npx lighthouse http://localhost:8080/ --preset=mobile --only-categories=performance` three times against the built app served by `counterspell-server`. Record the three scores, the median, LCP, CLS, TBT and the JS transferred in `docs/kiro-log/landing-steel-night-lighthouse.md`. If the median is under 90, cut the far layer, then the bats, then the smoke, then the parallax (task text).
- **Pre-measurement state:** first-load assets are listed in `docs/kiro-log/landing-steel-night-title-screen.md`. The poster is now WebP (88 KB desktop, 38 KB phone).

## T14: accessibility, privacy and Compose smoke check (open)
- **Why open:** the keyboard path, focus visibility and contrast checks are browser checks. The Compose smoke test (`http://localhost/` through Caddy) needs the full stack running.
- **Unblock:** Brighton runs the keyboard walk-through and the Compose smoke test on the local stack, then records the result in the Lighthouse log file.

## Decision for Brighton: decorative overlays on the title screen (not a blocker)
- The living loop already paints bats and fog. The planner still adds the `bat-swarm` and `fog-layer-cold` overlays (T16 tests cover them). They may duplicate the painting. I kept them to avoid changing tests the spec does not require me to change.
- **Question:** after a visual check, should the bat-swarm overlay be removed? If yes, the T16 Hero and scene tests change in the same commit with a `test-change` marker.

### verify all, 16 Nov 2026: T13 stays open: no Lighthouse median recorded (docs/kiro-log/landing-steel-night-lighthouse.md is required).
