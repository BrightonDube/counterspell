---
inclusion: always
---
# Counterspell product

Counterspell is a horror-themed coding practice platform. When a solution fails, it finds and shrinks a **minimized counterexample** (the “ghost input”), then explains the bug and offers graded hints.

## Audience and launch
- Job-seekers, students, bootcamp graduates, and authors, including people on slow connections.
- Launch languages: Python and JavaScript. Java and C++ are stretch goals.
- MVP: accounts, 10 original problems, IDE, server-side judging with live SSE, ghost engine, hints, dashboard, landing page, and the measured Efficiency Familiar.
- Judges use a free demo account; payments, proctoring, mobile apps, and copied problem content are out of scope.

## Core experience
1. Choose a problem, write code in Monaco, and run samples or submit hidden tests.
2. The trusted judge streams per-test results. A failed run can produce a minimized counterexample, shrink trace, Bug Autopsy, and hints.
3. A passing run can measure observed complexity against the reference and update the Familiar. AI speech is labelled and cannot change verdict or mood.
4. Progress updates on the dashboard.

## Product rules
- Describe ghosts as minimized counterexamples, never as proven global minima.
- The Familiar's mood comes only from measured facts; use observations such as “looks quadratic”.
- Build the landing and visuals first, but keep the real fail-to-ghost checkpoint on the VPS.
- Brighton reviews Kiro-written work; every Rust change includes a short Rust lesson.

Source of truth:
#[[file:docs/product-brief.md]]
#[[file:docs/requirements.md]]
#[[file:docs/system-design.md]]
