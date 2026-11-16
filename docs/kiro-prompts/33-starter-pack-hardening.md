# 33 - Starter Pack v1 Hardening (from Brighton, 14 Nov 2026)

Run with the workspace and global skills, strict TDD, and read .kiro/steering/brighton-decisions.md first. This brief is Brighton's review of grimoire-packs T28: packs stay `draft` until he approves the result.

Objective

Review, improve and validate the three existing starter packs: "count-pairs", "diagonal-total", "longest-run-length". These are the first beginner exercises for Counterspell, an algorithm-practice and debugging platform. The important design choice is to make learning the primary experience and grading the supporting mechanism.

Do not redesign the application or introduce unnecessary infrastructure. Work with the existing architecture, pack format, execution engine and validation conventions. Inspect the actual repository before implementation. Preserve working contracts, and identify any required compatibility changes.

1. Improve problem descriptions
For each pack: use "integers" rather than "whole numbers" when negative values are permitted; clearly state the function signature and arguments; add at least one worked example explaining the expected result; explain relevant edge cases; explain what the solution must return; keep language appropriate for beginners.
- count-pairs: clarify that pairs refer to distinct positions, not unique values. Explain "i < j", duplicate values and why reversed position pairs are not counted separately.
- diagonal-total: explain rectangular matrices, main-diagonal indexing, empty grids and zero-column grids.
- longest-run-length: explain consecutive-character runs and empty-string behavior.

2. Improve testing
Retain the current public samples, reference solutions and known ghosts unless correctness requires modification. Add deterministic regression cases.
- count-pairs: empty list; single-element list; repeated values; multiple valid pairs; negative numbers; zero target; no matching pairs; maximum input length; the existing same-index and empty-min ghost inputs.
- diagonal-total: empty grid; one or more empty rows; single cell; negative diagonal values; wide rectangular matrix; tall rectangular matrix; square matrix; maximum supported dimensions; both existing ghost inputs.
- longest-run-length: empty string; single character; all identical characters; all alternating characters; longest run at beginning; longest run at end; equal-length runs; maximum string length; both existing ghost inputs.
Keep generated tests, but do not rely on them for edge-case coverage. Curated tests must be deterministic and reproducible. Ensure generated matrices are rectangular and all generated inputs respect their declared constraints.

3. Validate all pack contracts
Create or extend the existing pack-validation workflow to check: TOML validity; required metadata fields; supported data types; valid comparator and limits; function name and parameter agreement; input/output schema consistency; unique slugs and ghost IDs; valid referenced program paths; sample inputs satisfying constraints; correct expected outputs; consistent status and version metadata. Avoid creating a second validation framework if one already exists. Reject malformed packs with actionable error messages.

4. Verify reference and incorrect solutions
Automatically verify that: (1) each reference implementation passes every curated and generated test; (2) every declared ghost input satisfies the input schema; (3) each ghost's recorded output or exception matches its actual execution; (4) every incorrect implementation fails at least one valid test; (5) ghost explanations accurately describe the demonstrated failure. Use the existing secure execution mechanism. Never execute untrusted learner code directly in the application process.

5. Improve educational hints
Retain the four-rung hint approach, with progressive disclosure: (1) conceptual nudge; (2) relevant state, indexing or data-structure idea; (3) algorithmic guidance; (4) near-solution guidance without revealing complete code. Hints must be revealed individually, on request. Do not expose reference solutions or hidden tests before the appropriate learning stage.

6. Add lightweight educational metadata
Where compatible with the existing pack schema, include estimated completion time, learning objectives and prerequisite concepts. Provide a short editorial explanation of the reference solution with expected time and auxiliary space complexity: Count the Pairs O(n^2) time, O(1) auxiliary space; Diagonal Total O(min(rows, columns)) time, O(1) auxiliary space; Longest Run O(n) time, O(1) auxiliary space. Keep internal editorial content separate from learner-visible content where appropriate.

7. Preserve MVP scope
Do not add: AI-generated personalized tutoring; new databases or services; complex adaptive difficulty algorithms; leaderboards; new code-execution infrastructure; multiple language implementations; unnecessary new dependencies. Use the existing runner, data models, APIs and frontend components.

8. Acceptance criteria
Complete only when: all three packs parse successfully; all three reference solutions pass all tests; each known ghost is verified against its declared failing case; all test inputs satisfy their declared schema; invalid packs fail validation with useful error messages; beginners can understand the task and expected behavior without reading TOML; existing pack consumers continue to work or receive the necessary compatible updates; the normal CI workflow runs pack validation; existing relevant tests remain passing. Where the application already supports public sample execution, hidden-test submission, progressive hints, known-ghost explanations and reference-solution review, verify them end to end rather than rebuilding them (no browser/E2E tests for now: use the existing Rust/Python test harness).

9. Deliverables
Updated files for all three packs; added regression tests; pack-validation changes if needed; automated verification results; a concise changelog (docs/kiro-log/starter-pack-v1-hardening.md) explaining each change; any remaining issues and limitations. Commit locally on brighton/kiro-setup in small TDD commits. Do not push. Do not claim that a feature works without executing its relevant tests.

Priority: release a small, reliable, educational starter pack rather than expanding the platform's feature set.