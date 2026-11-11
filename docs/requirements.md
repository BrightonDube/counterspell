# Counterspell: product requirements
*Written Thu 12 Nov 2026, 00:05 SAST (UTC+2) by Brighton's Research Assistant. It's the input brief for Kiro's spec generation; Kiro turns these into its own `requirements.md` files per spec. Companion docs: `system-design.md` (architecture), `palette.md` (look), `licence-rules.md`, `kiro-playbook.md`. Updated Thu 12 Nov 2026, 00:43 SAST: added E14 / VAL-1–VAL-5 (submission input validation). Updated Thu 12 Nov 2026, 01:45 SAST: applied Brighton's decisions from the grill-me review of the ChatGPT critique (see system design "Decision log 2026-11-12"); changes are marked **[D 12 Nov]**.*

**Notes:**
- **Placeholder names are examples:** Thandi, Sipho, Ms Naidoo, Kwame and "Judge J" are invented personas, not real users. Brighton is the real builder and the first job-seeker user.
- **Labels:** **[D]** = decided by Brighton; **[E]** = estimate; **[U]** = unverified; **[GK]** = general knowledge, not from a cited source.

---

## 1. Purpose and scope
**Purpose:** help people practise coding-interview problems and learn *why* their code is wrong. When a solution fails, Counterspell finds an input that breaks it and shrinks it to a **minimized counterexample** (the "ghost input": the smallest failing input it can find within a time budget, not a proven minimum) [D 12 Nov], explains it, offers graded hints, and shows how efficient a correct solution is through a reactive mascot (the Efficiency Familiar) whose mood comes only from measured results. It's wrapped in a polished horror theme ("Steel Night").

**Scope for the hackathon** (submit by Fri 4 Dec 2026 14:00 PT = Sat 5 Dec 00:00 SAST; feature freeze Wed 2 Dec):
- a web app (desktop-first, usable on tablet);
- accounts;
- 10 polished original problems [D 12 Nov];
- a browser IDE in Python and JavaScript (Java and C++ are stretch) [D 12 Nov];
- server-side sandboxed judging with live results;
- the ghost input, with a visible shrink animation and a Bug Autopsy card [D 12 Nov];
- hints;
- the familiar;
- a dashboard;
- a landing page;
- `counterspell-mcp` for Kiro, plus a Kiro dogfooding demo [D 12 Nov].

Stretch, only after the core is freeze-safe [D 12 Nov]: Java and C++, spaced review (FSRS), the familiar's live typing guess, timed trials and the break room.

**Hackathon category:** **Costume Contest** (one main category). Frankenstein-style elements may appear in the build, but Frankenstein is not a second category [D 12 Nov]. Judges must be able to use it **free until Fri 29 Jan 2027 23:00 SAST**.

---

## 2. Personas (examples)
| Persona | Who | Goals | Pain today |
|---|---|---|---|
| **P1 Job-seeker**, e.g. **Brighton** (real) | A developer preparing for coding interviews and online assessments | Pass timed assessments; understand failures quickly; build a habit | Hidden-test failures say "Wrong Answer" with no reason; hard to know what to review |
| **P2 Student**, e.g. **Thandi** (example) | A 2nd-year CS student on a mid-range laptop and a capped mobile data plan | Learn data structures by doing; get unstuck without being handed the answer | Long editorials; gives up when stuck; heavy sites on slow connections |
| **P3 Bootcamp instructor**, e.g. **Ms Naidoo** (example) | Runs a 12-week bootcamp cohort | See which concepts the cohort struggles with; assign practice | No cheap way to see *why* students fail |
| **P4 Problem author**, e.g. **Kwame** (example) | Writes original problems (might be Brighton, or a contributor using Kiro) | Write problems with generators and reference solutions quickly and safely | Writing test data by hand; no tools to check edge cases |
| **P5 Hackathon judge**, e.g. **Judge J** (example) | A Kiroween judge with ~10 minutes | Sign in fast, see the core idea working, check how Kiro was used | Broken demos, sign-up friction, paywalls |

---

## 3. Epics and user stories
Format: *As a … I want … so that …* IDs are `EPIC-n`. Stories marked ★ have EARS acceptance criteria in §4 (priority is in §8).

### E1 Accounts and sign-in
- ★ **ACC-1:** As a new visitor, I want to sign up with GitHub, Google or email, so that I can start practising in under a minute.
- ★ **ACC-2:** As a returning user, I want to stay signed in on my device, so that I don't log in every visit.
- ★ **ACC-3:** As a signed-in user, I want to link GitHub, Google or an email address myself, with verification, so that I can use either to sign in, and nobody can take over my account just by having a matching email [D 12 Nov].
- **ACC-4:** As a user, I want to sign out of all devices, so that a lost laptop isn't a risk.
- **ACC-5:** As a user, I want to delete my account and data, so that I control my information.
- ★ **ACC-6:** As a judge, I want a demo account (credentials in the Devpost form), so that I can test without creating an account.

### E2 Problem browsing
- **PRB-1:** As a learner, I want to browse problems by track ("Crypt"), concept and difficulty, so that I practise what I need.
- **PRB-2:** As a learner, I want to see which problems I've solved, attempted or have due for review, so that I know what to do next.
- **PRB-3:** As a learner, I want a clear statement with examples, constraints and the expected function signature per language, so that I can start coding immediately.

### E3 IDE and Run/Submit
- ★ **IDE-1:** As a learner, I want a code editor with syntax highlighting and a language picker (Python and JavaScript at launch; Java and C++ stretch [D 12 Nov]), so that I can write in the language I'll be interviewed in.
- ★ **IDE-2:** As a learner, I want **Run** to test my code on the sample tests and show input/expected/got, so that I can debug quickly.
- ★ **IDE-3:** As a learner, I want **Submit** to judge my code on hidden tests with per-test results appearing live, so that I'm not staring at a spinner.
- **IDE-4:** As a learner, I want my draft saved automatically per problem and language, so that I never lose work.
- **IDE-5:** As a keyboard user, I want shortcuts (Run, Submit, switch panels) and visible focus, so that I can work without a mouse.
- **IDE-6:** As a learner, I want clear verdicts (Accepted, Wrong Answer, Time Limit, Memory Limit, Runtime Error, Compile Error) with the relevant stderr or compiler message, so that I know what kind of failure it is.

### E4 Ghost input
- ★ **GHO-1:** As a learner whose submission failed, I want a small input that breaks my code (a minimized counterexample: the smallest the engine found within its budget [D 12 Nov]), with expected vs my output, so that I understand the bug instead of guessing.
- **GHO-2:** As a learner, I want a one-line plain-English explanation of the ghost ("You returned [0,0]; expected [0,1] for nums=[0,0], target=0"), so that I know where to look.
- **GHO-3:** As a learner, I want to copy the ghost input into a custom Run, so that I can step through it myself.
- **GHO-4:** As a learner, I want to see when no ghost could be found within the time budget, so that I'm not misled.
- ★ **GHO-5:** As a learner, I want to watch the failing input shrink step by step down to the minimized counterexample, so that I see how the ghost was hunted and trust it [D 12 Nov].
- ★ **GHO-6:** As a learner, I want a Bug Autopsy card with the failing input, an expected vs actual diff and the conceptual mistake behind it, so that I learn the concept, not just the fix [D 12 Nov].

### E5 AI hints
- ★ **HNT-1:** As a stuck learner, I want graded hints (nudge → approach → pseudo-code → targeted fix), so that I get unstuck without being handed the solution.
- **HNT-2:** As a learner, I want an optional AI hint that refers to my ghost input and my code, so that the hint is specific to my bug.
- **HNT-3:** As the operator, I want a daily AI-hint cap per user and a fallback provider, so that costs stay predictable and hints still work if one provider fails.

### E6 Efficiency Familiar (mascot)
- ★ **FAM-1:** As a learner, I want the jack-o'-lantern to react on Run/Submit to how efficient my correct solution is (5 moods, furious → laughing), set only from measured results, so that I learn to care about complexity and can trust the reaction [D 12 Nov].
- **FAM-2:** As a learner, I want a gentle live *guess* while I type (clearly labelled a guess, and never changing the mood), so that I notice nested loops early. *(Stretch [D 12 Nov].)*
- **FAM-3:** As a learner, I want to mute or hide the familiar, so that it never distracts me.
- **FAM-4:** As a motion-sensitive user, I want no shakes or bursts when reduced motion is on, so that the app is comfortable.
- **FAM-5:** As a curious learner, I want "How does it judge me?" to explain the measurement and its limits, so that I trust it.
- ★ **FAM-6:** As a learner, I want the familiar's speech bubble to explain my result in plain words (an AI hint, clearly labelled), based only on my code and what was measured, so that the feedback is specific without being made up [D 12 Nov].

### E7 Dashboard, progress and spaced review
- ★ **DSH-1:** As a learner, I want a dashboard with solved counts, streak, weak concepts and recent submissions, so that I see progress at a glance.
- **DSH-2:** As a learner, I want concepts I struggled with scheduled for review (FSRS), with a fresh sibling problem, so that I retain what I learned. *(Stretch [D 12 Nov].)*
- **DSH-3:** As a learner, I want 3 suggested next problems, so that I don't waste time choosing.

### E8 Timed trials ("Night Shift", stretch after the freeze-safe core)
- **TRL-1:** As a job-seeker, I want a timed 2-problem, 90-minute trial that mimics an online assessment, so that I practise under pressure.
- **TRL-2:** As a job-seeker, I want a results report after the trial (score, time per problem, ghosts hit), so that I know what to improve.

### E9 Grimoire packs (problem authoring)
- ★ **PCK-1:** As a problem author, I want to write a problem as files (statement, metadata, input schema, generator, reference solution, comparator), so that problems live in git and can be reviewed.
- **PCK-2:** As a problem author, I want a validator that checks the pack (the reference passes its own tests, the generator respects constraints, the reference complexity matches its benchmark), so that broken problems never ship.
- **PCK-3:** As the operator, I want only original problems (no copied HackerRank/LeetCode text), so that we respect their IP.

### E10 Break room (stretch after the freeze-safe core; built last; first to cut)
- **BRK-1:** As a learner taking a break, I want a daily chess puzzle (from the CC0 Lichess puzzle set), so that I rest my brain without leaving.
- **BRK-2:** As a learner, I want to play a quick game against a simple chess bot, so that I have a short fun break.
- **BRK-3:** As a learner, I want one small brain game (e.g. a pattern or memory game), so that there's a non-chess option.

### E11 Pricing and subscription (later; NOT in the hackathon build)
- **PAY-1:** As a heavy user, I want a paid tier with more AI hints and timed trials, so that I can support the product. *(Later; judges must test free.)*
- **PAY-2:** As an instructor, I want a cohort plan with a class dashboard, so that I can track students. *(Later.)*

### E12 Admin and moderation
- **ADM-1:** As the operator, I want to import, publish and unpublish problem packs, so that I control the catalogue.
- **ADM-2:** As the operator, I want rate-limit and abuse dashboards (runs per user, sandbox kills, queue depth), so that I can spot abuse.
- **ADM-3:** As the operator, I want to suspend an account, so that abusers can't keep using compute.
- **ADM-4:** As the operator, I want error and audit logs without user code in plain logs, so that I can debug without leaking data.

### E13 MCP for Kiro (developer-facing)
- ★ **MCP-1:** As a developer using Kiro, I want a `counterspell-mcp` server with tools like `validate_pack`, `find_ghost_input`, `shrink_input`, `explain_verdict` and `fit_complexity`, so that Kiro can author and debug problems with the same engine.
- **MCP-2:** As a developer, I want a Kiro Power that bundles the MCP server with "author a problem" and "hunt a ghost" skills, so that anyone can install it in one click.
- ★ **MCP-3:** As a hackathon judge watching the video, I want to see Kiro use `counterspell-mcp` to find a counterexample in a deliberately buggy reference or implementation, fix it and re-verify, so that I see the ghost engine and Kiro working together [D 12 Nov].

### E14 Submission validation (pre-sandbox)
**Principle:** browser checks are for quick feedback only. Anyone can skip them and call the API directly, so the server repeats every check. These checks are a first filter, not the main protection. The sandbox (§6; no network, a read-only filesystem, a non-root user, and caps on CPU, memory, processes and time) is the real security boundary. The system does not use keyword or API blocklists (for example `os` or `system`), which are easy to bypass. Valid code can still be hostile; the sandbox has to contain it.

- ★ **VAL-1:** As a learner, I want a submission over the size limit rejected at once with a clear message, so that I know to shorten my code instead of waiting on the queue.
- ★ **VAL-2:** As a learner, I want the editor and the server to check that my code parses as the language I selected, so that a pasted shell script or the wrong language is rejected before it is queued.
- ★ **VAL-3:** As the operator, I want only a single plain-text source file accepted (valid UTF-8, no file uploads), so that binary or multi-file payloads never reach the sandbox.
- ★ **VAL-4:** As the operator, I want per-user and per-IP rate limits on Run and Submit, plus a cap on queued jobs per user, so that one person or IP can't starve the queue.
- ★ **VAL-5:** As a learner, I want program output and test results shown as plain text (never as HTML), so that nothing a program prints can run in my browser or anyone else's.

---

## 4. Acceptance criteria for core stories (EARS)
EARS patterns: *Ubiquitous* (THE SYSTEM SHALL …), *Event* (WHEN … THE SYSTEM SHALL …), *State* (WHILE …), *Unwanted* (IF … THEN …), *Optional* (WHERE …).

**ACC-1 Sign up**
1. WHEN a visitor chooses GitHub or Google, THE SYSTEM SHALL complete OAuth with a `state` value and PKCE, and create or sign in the account.
2. WHEN a visitor submits an email address, THE SYSTEM SHALL send a single-use sign-in link valid for 15 minutes.
3. IF a sign-in link is used twice or after expiry, THEN THE SYSTEM SHALL reject it and offer a new link.
4. IF OAuth returns an email that matches an existing account not already linked to that provider identity, THEN THE SYSTEM SHALL NOT link or sign in to that account automatically, and SHALL tell the user to sign in with their original method and link the provider from settings [D 12 Nov].

**ACC-3 User-initiated linking** [D 12 Nov]
1. WHEN a signed-in user starts linking GitHub or Google, THE SYSTEM SHALL bind a link-intent `state` to that session and link the identity only after the OAuth flow completes for that session.
2. WHEN a signed-in user adds an email address, THE SYSTEM SHALL link it only after the user opens a single-use verification link sent to that address.
3. IF the provider identity is already linked to another account, THEN THE SYSTEM SHALL refuse the link and say why.

**ACC-2 Session**
1. THE SYSTEM SHALL keep sessions in HttpOnly, Secure, SameSite=Lax cookies for 30 days with rolling renewal.
2. IF a request carries an invalid or expired session, THEN THE SYSTEM SHALL treat it as signed out and return 401 for API calls.

**ACC-6 Judge access**
1. THE SYSTEM SHALL provide a demo account, and all features in the video SHALL work without payment until 29 Jan 2027 23:00 SAST.
2. THE SYSTEM SHALL let a judge sign in to the demo account from the sign-in page with the credentials given in the Devpost private field [D 12 Nov].
3. THE SYSTEM SHALL NOT let the demo account link providers, change its credentials or delete itself, and SHALL reset its data daily [E].

**IDE-1 Editor**
1. THE SYSTEM SHALL offer Python and JavaScript with a starter signature per problem. WHERE the Java and C++ stretch drivers ship, THE SYSTEM SHALL offer them the same way [D 12 Nov].
2. WHEN the user switches language, THE SYSTEM SHALL keep each language's draft separately.

**IDE-2 Run**
1. WHEN the user presses Run, THE SYSTEM SHALL execute the code in a server sandbox on the sample tests and stream one result per test.
2. THE SYSTEM SHALL show input, expected and actual output for sample tests.
3. IF the code is larger than 64 KB, THEN THE SYSTEM SHALL reject the run with a clear message.
4. IF the user exceeds the run rate limit (e.g. 30 runs / 10 min [E]), THEN THE SYSTEM SHALL return a 429 with the wait time.

**IDE-3 Submit**
1. WHEN the user presses Submit, THE SYSTEM SHALL queue the run and return a run id within 300 ms (p95, excluding network) [E].
2. WHILE a run is in progress, THE SYSTEM SHALL stream per-test verdicts over SSE within 1 s of each test finishing.
3. THE SYSTEM SHALL NOT reveal hidden test inputs or expected outputs.
4. IF the SSE connection drops, THEN THE SYSTEM SHALL replay every stored event with a sequence number after the client's `Last-Event-ID`, in order, from Postgres; the client SHALL discard duplicates by sequence number [D 12 Nov].
5. WHEN the run finishes, THE SYSTEM SHALL store the submission, per-test results and the final verdict.
6. THE SYSTEM SHALL create the submission and its queue job in one transaction, so a saved submission always has a job [D 12 Nov].
7. THE SYSTEM SHALL give every run exactly one terminal state and one final result, fetchable after a page reload, even if a worker crashes and the job is retried [D 12 Nov].
8. IF the sandbox infrastructure fails, THEN THE SYSTEM SHALL report an internal error and SHALL NOT report it as the user's wrong answer [D 12 Nov].

**GHO-1 Ghost input**
1. WHEN a submission fails and the problem has a reference solution, THE SYSTEM SHALL search for a failing input and shrink it to a minimized counterexample within 8 s total.
2. THE SYSTEM SHALL only show a ghost that reproduces the same failure predicate (same verdict kind and exception class) as the original failure, verified against the Python reference by the trusted judge.
3. THE SYSTEM SHALL only accept a shrink step that strictly reduces a deterministic size metric.
4. THE SYSTEM SHALL produce the same ghost for the same code, problem, language and engine version (deterministic seed), and SHALL record the seed, generator version, problem version, runtime image digest and shrink steps with it.
5. THE SYSTEM SHALL only show inputs that satisfy the problem's constraints.
6. THE SYSTEM SHALL describe the ghost as the smallest failing input *found*, and SHALL NOT claim it is the smallest possible input [D 12 Nov].
7. IF no failing input is found within budget, THEN THE SYSTEM SHALL say so and fall back to the first failing sample (if any).

**GHO-5 Shrink animation** [D 12 Nov]
1. WHEN a ghost is shown, THE SYSTEM SHALL animate the recorded shrink steps in order, from the first failing input found down to the minimized counterexample.
2. THE SYSTEM SHALL only animate steps the engine actually recorded, and SHALL NOT invent intermediate inputs.
3. IF there are more recorded steps than the display cap (e.g. 20 [E]), THEN THE SYSTEM SHALL show the first and last steps and summarise the rest.
4. WHERE reduced motion is on, THE SYSTEM SHALL show the steps as a static list instead of an animation.

**GHO-6 Bug Autopsy card** [D 12 Nov]
1. WHEN a ghost is shown, THE SYSTEM SHALL show the failing input, the expected output and the actual output with their differences highlighted, using text and icons as well as colour.
2. WHERE the pack has an authored explanation for a matching known ghost, THE SYSTEM SHALL show it as the conceptual mistake; otherwise THE SYSTEM SHALL show an AI explanation labelled as an AI hint.
3. THE SYSTEM SHALL NOT reveal hidden test inputs or expected outputs on the card.

**HNT-1 Hints**
1. THE SYSTEM SHALL reveal authored hints one level at a time, in order.
2. WHERE AI hints are enabled, WHEN the user requests one, THE SYSTEM SHALL include the ghost input and the user's code in the prompt and SHALL NOT output a full solution.
3. IF the primary AI provider errors or takes over 8 s, THEN THE SYSTEM SHALL retry once with the fallback provider.
4. IF the user has reached the daily AI-hint cap, THEN THE SYSTEM SHALL show the cap and the next authored hint instead.

**FAM-1 Familiar (measured)**
1. WHEN a Submit passes all tests and the problem has a bench spec, THE SYSTEM SHALL time the solution at growing input sizes (5 repeats per size, median) within a 4 s budget.
2. THE SYSTEM SHALL classify growth only when the slope, size-ratio and best-fit methods agree; otherwise it SHALL mark confidence as low.
3. THE SYSTEM SHALL map the result to one of 5 moods (Furious, Angry, Uneasy, Pleased, Laughing) per system design §8b, using only measured facts: correctness and observed scaling against the reference complexity, with confidence [D 12 Nov].
4. THE SYSTEM SHALL word results as observations (e.g. "looks quadratic, medium confidence") and SHALL label the top mood "Matches reference complexity", never "optimal" [D 12 Nov].
5. IF confidence is low, THEN THE SYSTEM SHALL NOT lower the mood by more than one step.
6. WHEN a run fails or times out, THE SYSTEM SHALL show the crack-and-burst (or a static badge under reduced motion).
7. WHERE the live typing guess (FAM-2, stretch) is enabled, WHILE the user types, THE SYSTEM SHALL show only a labelled guess and SHALL NOT change the mood [D 12 Nov].

**FAM-6 Familiar speech bubble (AI)** [D 12 Nov]
1. WHEN a Run or Submit finishes, THE SYSTEM SHALL request at most one speech-bubble hint for that run from the hint providers (gpt-6-luna, then Groq gpt-oss-20b), and SHALL NOT call the LLM on keystrokes.
2. THE SYSTEM SHALL ground the prompt only on the user's code, the measured results (verdicts, ghost, timings, fitted class, confidence) and the reference complexity.
3. THE SYSTEM SHALL label the bubble as an AI hint.
4. THE SYSTEM SHALL set the verdict and mood before the bubble is requested, and the bubble SHALL NOT change them.
5. IF both providers fail or the daily AI cap is reached, THEN THE SYSTEM SHALL show a templated line built from the measured facts.

**DSH-1 Dashboard**
1. THE SYSTEM SHALL show solved counts by difficulty, the current streak, the 10 latest submissions, weak concepts and the next due review in one request.
2. THE SYSTEM SHALL only ever show the signed-in user's data.
3. WHEN a submission finishes, THE SYSTEM SHALL reflect it on the dashboard by the next page load.

**PCK-1 Packs**
1. THE SYSTEM SHALL reject a pack whose reference solution fails any of its own tests.
2. THE SYSTEM SHALL reject a pack whose generator produces an input violating its input schema (checked on ≥ 200 samples).
3. WHERE a pack declares `reference_complexity`, THE SYSTEM SHALL check it by benchmarking the reference, and warn on mismatch.

**MCP-1 MCP server**
1. THE SYSTEM SHALL expose the listed tools over MCP stdio using the same engine crate as production.
2. THE SYSTEM SHALL execute code for `run_reference`, `find_ghost_input` and any other code-running tool only through the same sandbox and trusted judge as production (same images, limits and comparator), and SHALL NOT run user or reference code directly on the developer's machine [D 12 Nov].

**MCP-3 Kiro dogfooding demo** [D 12 Nov]
1. THE SYSTEM SHALL include a deliberately buggy reference or implementation in a test pack used only for the demo.
2. WHEN Kiro calls `find_ghost_input` on it, THE SYSTEM SHALL return a minimized counterexample through the sandbox.
3. WHEN Kiro has fixed the bug, THE SYSTEM SHALL re-verify it (the pack validator and `find_ghost_input` find no counterexample within budget), and the session SHALL be recorded for the video and the Kiro write-up.

**Sandbox (applies to IDE-2, IDE-3, GHO-1, FAM-1)**
1. THE SYSTEM SHALL run user code only inside a container with no network, a read-only root filesystem, a non-root user, all capabilities dropped, and CPU, memory, process and wall-time limits.
2. IF user code exceeds a limit, THEN THE SYSTEM SHALL return TLE, MLE, RE or OLE and SHALL NOT return Accepted.
3. THE SYSTEM SHALL remove every user container after the run, including on worker crash (reaper within 60 s).
4. THE trusted judge worker SHALL hold the test cases, expected outputs, comparator, per-test time limits and the final verdict. The user program SHALL only receive inputs and print answers (or the driver harness returns raw answers) [D 12 Nov].
5. THE SYSTEM SHALL enforce per-test time limits by killing the program from outside the sandbox.
6. IF the user program prints anything that looks like a verdict, THEN THE SYSTEM SHALL treat it only as an answer to compare, and SHALL NOT let it change the verdict [D 12 Nov].
7. THE SYSTEM SHALL run sandbox containers on a separate rootless Docker daemon used only for sandboxes; the judge SHALL NOT have access to the main Docker daemon [D 12 Nov].

**Submission validation (applies to VAL-1–VAL-5)**
1. THE SYSTEM SHALL repeat every client-side validation check on the server before queuing a run. Client-side checks are for feedback only.
2. THE SYSTEM SHALL NOT use keyword or API blocklists as a security control.
3. THE SYSTEM SHALL run accepted user code only inside the sandbox (see Sandbox above).

**VAL-1 Size limit** (applies to IDE-2, IDE-3)
1. THE SYSTEM SHALL enforce a maximum source size in bytes, configurable per language, default 64 KB. THE SYSTEM SHALL NOT use a line-count limit as the cap.
2. IF the source is larger than the limit for the selected language, THEN THE SYSTEM SHALL reject the Run or Submit on the server, before queuing, with a clear message that states the limit and the actual size.
3. WHEN the source exceeds the limit in the editor, THE SYSTEM SHALL show the same kind of rejection in the browser for immediate feedback. THE SYSTEM SHALL still enforce the limit on the server.

**VAL-2 Language parse** (applies to IDE-2, IDE-3)
1. WHEN a user presses Run or Submit, THE SYSTEM SHALL parse the source on the server with tree-sitter, using the grammar for the selected language, before queuing.
2. IF the source does not parse as that language, THEN THE SYSTEM SHALL reject it with a clear error and SHALL NOT queue the run.
3. THE SYSTEM SHALL run the same parse check in the browser for immediate feedback. THE SYSTEM SHALL still enforce the check on the server.

**VAL-3 Plain text only** (applies to IDE-2, IDE-3)
1. THE SYSTEM SHALL accept only a single source file as plain text in the request. THE SYSTEM SHALL NOT accept file uploads.
2. IF the source is not valid UTF-8, or contains a NUL byte, or contains a control character other than tab (U+0009), newline (U+000A) or carriage return (U+000D), THEN THE SYSTEM SHALL reject it before queuing with a clear message.

**VAL-4 Rate limits** (applies to IDE-2, IDE-3)
1. THE SYSTEM SHALL enforce per-user and per-IP rate limits on Run and Submit in Valkey, and a maximum number of queued jobs per user (counted from the Postgres job queue).
2. IF a user or IP exceeds a rate limit, or a user already has the maximum queued jobs, THEN THE SYSTEM SHALL return 429 with the wait time and SHALL NOT queue another run.

**VAL-5 Output escaping** (applies to IDE-2, IDE-3, GHO-1)
1. THE SYSTEM SHALL render program stdout, stderr and test output as plain text in the UI, never as HTML.
2. THE SYSTEM SHALL truncate output sent to the client to a display cap (e.g. 64 KB [E]). The sandbox still kills the run if stdout exceeds 1 MB or stderr exceeds 64 KB (§6).

---

## 5. End-to-end walkthroughs (plain language; names are examples)

### W1: Thandi solves Two Sum in JavaScript (example)
1. Thandi (student) signs in with Google and opens **Two Sum** in the "Crypt of Arrays".
2. She picks **JavaScript** and types a double `for` loop. The jack-o'-lantern in the corner stays calm: its mood only changes when something is measured. *(Stretch: a small labelled guess bubble may say "Guess from reading your code: looks like O(n²). Run it for the real verdict.")*
3. She presses **Run**. Her code goes to our server, not her laptop. It's queued, picked up by the judge worker and run in a locked-down Node.js container. The 3 sample tests pass, and each tick appears live.
4. She presses **Submit**. Results stream in: **3 of 12 hidden tests fail** (Wrong Answer). The familiar **cracks and bursts into crimson embers**, then re-forms.
5. A "ghost hunting…" bar runs for about 2 s. The **ghost card** replays the shrink: the first failing input the engine found, then smaller ones, down to `nums = [0, 0], target = 0` (an example ghost). The **Bug Autopsy** shows expected `[0, 1]`, got `[0, 0]`, and the mistake: *"You used the same element twice: your inner loop starts at i instead of i + 1."*
6. She opens **Hint 1** ("Can an index pair with itself?") and fixes `j = i + 1`. She re-submits: **12/12 pass**. The familiar measures her solution: it looks O(n²) against the reference O(n), which gives **Angry** (passes, looks 2 classes worse). Its speech bubble, labelled AI hint, says the nested scan is what makes it quadratic. The popover shows the timing chart.
7. She opens **Hint 3** (approach: "store what you've seen in a Map") and rewrites the solution. Submit: 12/12, looks O(n), within 2× of the JavaScript baseline. The familiar **laughs** ("Matches reference complexity"; ember ring + a small green spark).
8. Her **dashboard** updates: Two Sum solved, streak = 1 day, "Hash maps" shown as a practised concept. *(Stretch: with FSRS, the next review would be due in about 3 days.)*

### W2: Brighton practises before an interview (real user, example flow)
1. Brighton opens the dashboard. "Due for review: Sliding window." He clicks the suggested sibling problem.
2. He writes Python and submits. A **Time Limit** verdict on 2 hidden tests (he re-sorts inside the loop). The ghost engine finds no wrong *answer*, so it reports the timeout kind and the size at which it first timed out.
3. The familiar is **Furious**, and its speech bubble (gpt-6-luna, labelled AI hint) points to the sort inside the loop. He fixes it and passes. Mood: **Pleased** (same class as the reference, slower than the baseline). *(Stretch: the review is rescheduled further out.)*
4. Stretch: he starts a **Night Shift** trial (2 problems, 90 min). At the end he gets a report with time per problem and the ghosts he hit.

### W3: Kwame authors a problem with Kiro (example)
1. Kwame opens the repo in Kiro with the Counterspell Power installed and asks it to *"author a problem: count islands in a grid."*
2. Kiro (problem-author agent) writes `packs/count-islands/` (statement, schema, generator, reference, comparator) and calls `validate_pack` over MCP. The validator flags that the generator can produce a 0×0 grid that the constraints forbid. Kiro fixes the schema.
3. Kiro calls `find_ghost_input` with a deliberately wrong solution (diagonal neighbours). The call runs through the sandbox, not on Kwame's laptop. It gets a tiny 2×2 minimized counterexample, which Kwame adds as a "known ghost" seed with an authored Bug Autopsy explanation.
4. The pack passes validation, and the operator imports and publishes it (ADM-1).

### W4: Judge J tests the submission (example)
1. Judge J opens the Devpost link and signs in with the demo account from the private submission field.
2. They submit a deliberately wrong Python answer to a starter problem and see the shrink animation, the ghost card and the Bug Autopsy in under 10 s, then the familiar's reaction.
3. They open the GitHub repo: the `LICENSE` (MIT) shows in About, `/.kiro` contains specs, hooks, steering, agents and MCP settings, and `docs/KIRO_USAGE.md` explains how Kiro was used.

---

## 6. What happens when a user runs code (plain English)
**The short version:** your code never runs in your browser and never runs "loose" on our server. It runs inside a tiny, throwaway, locked box on the server. The box is destroyed afterwards.

**Step by step:**
1. **You press Run or Submit.** The browser first checks size, language parse and plain text for instant feedback, then sends your code, the language and the problem id to our server. Those browser checks are only for UX; they can be skipped with a direct API call.
2. **The server checks you and the code.** It confirms you're signed in, under the per-user and per-IP rate limits (and under the queued-job cap), that the code is a single plain-text source within the byte limit for that language (default 64 KB), and that it parses as the selected language (tree-sitter). Only then does it save a "queued" submission and a job ticket in Postgres in one step (one transaction), so a ticket can't get lost [D 12 Nov]. It answers immediately with a run id. Your browser then opens a live connection (Server-Sent Events) to receive results.
3. **A judge worker picks up the ticket.** It's a separate, trusted program from the website. It claims the ticket with a time-limited lease, so if it crashes another worker can retry. It creates a fresh **Docker container** for your language from a pre-built image (Python or Node; Temurin Java and GCC are stretch), on a separate rootless Docker daemon that is used only for sandboxes [D 12 Nov].
4. **The container is locked down:**
   - no network at all;
   - a read-only system disk with only a small scratch folder;
   - runs as an unprivileged "nobody" user with all special Linux permissions removed;
   - Docker's default system-call filter;
   - one CPU's worth of time (a quota), 256 MB memory (512 MB for stretch Java), at most 64 processes;
   - a hard wall-clock kill;
   - output capped at 1 MB.
5. **Your code runs against the tests** through a small per-language "driver". The worker sends one test input at a time; the driver calls your function and prints only its raw answer. The worker times each test with its own clock, kills the program from outside if it runs too long, and compares each answer with the expected one *outside* the container. Your code never sees the answers, and nothing it prints can declare a verdict [D 12 Nov].
6. **Results stream back**, one per test, to your screen. Each result is stored with a sequence number, so a dropped connection picks up where it left off. Output is shown as plain text (never HTML) and truncated to a display cap.
7. **If it failed,** the ghost engine runs your code and the reference solution side by side (still sandboxed) on generated inputs, finds one where they disagree, and shrinks it to a minimized counterexample. The shrink steps are kept for the animation.
8. **If it passed,** bench mode times it at growing sizes for the familiar.
9. **Cleanup:** the container is deleted. A "reaper" deletes any container older than 60 s, in case something crashed.

**Risks and mitigations:**
| Risk | What could happen | Mitigation |
|---|---|---|
| Infinite loop | Ties up a CPU forever | Per-test time budget + wall-clock kill → **Time Limit** verdict |
| Memory bomb | Server runs out of RAM | Container memory cap (no swap) → **Memory Limit** |
| Fork bomb | Thousands of processes | `pids-limit 64` |
| Network abuse (attacking others, leaking tests) | Data exfiltration, spam | `--network none` |
| Reading secrets or other users' code | Data breach | Separate container per run; no host mounts; secrets never inside the container; tests compared outside |
| Container escape (kernel exploit) | Full server compromise | Non-root, capabilities dropped, no-new-privileges, default seccomp, patched host. Sandboxes run on a **separate rootless Docker daemon used only for sandboxes**; the app, database and secrets aren't on it, and the judge can't reach the main daemon [D 12 Nov]. Optional **gVisor** runtime later. The judge has no inbound port. |
| Faked verdict or timing | A wrong solution shows as Accepted | The trusted worker holds the answers, compares, times and decides; the program only prints answers; a hostile test prints fake verdict JSON [D 12 Nov] |
| Lost or stuck run | A submission stays "running" forever | Postgres job queue with atomic insert, leases, a reaper, max attempts and a single terminal write; infrastructure failures show as internal errors [D 12 Nov] |
| Huge output | Disk or memory fill; huge payload in the browser | Sandbox kills at 1 MB stdout / 64 KB stderr; UI truncates to a display cap (e.g. 64 KB) and renders as plain text, never HTML (VAL-5) |
| Too many runs (abuse or bots) | Queue starvation, cost | Per-user and per-IP rate limits in Valkey, max queued jobs per user (counted in the Postgres queue; 429), code size cap in bytes (VAL-1, VAL-4), account suspension (ADM-3) |
| Hostile or non-source payload (wrong language, binary, control characters, uploads) | Unexpected data reaches the queue or worker | Server-side tree-sitter parse for the selected language; valid UTF-8 and plain-text checks; single source field; no uploads (VAL-2, VAL-3). Client checks are UX only. The sandbox, not a keyword blocklist, is the execution boundary. |
| XSS via program output | Script in stdout/stderr runs in a user's browser | Render stdout, stderr and test output as plain text, never HTML (VAL-5) |
| Noisy timing (familiar) | Wrong mood | Medians, size ratios, confidence levels; low confidence never drops the mood more than one step; "looks quadratic" wording, never "optimal" [D 12 Nov] |

**How HackerRank and LeetCode do it:**
- **HackerRank (sourced):** its Execution Environment page (support.hackerrank.com, opened 11 Nov 2026) says:
  - each language has its own time and memory limits, e.g. **Python 3: 10 s, 512 MB; C++: 2 s, 512 MB; Java 8: 4 s, 512 MB**;
  - "The total CPU time includes execution across all threads";
  - "The maximum allowed size for a code submission is 50 KB."
  - We use a similar idea (per-language limits, a size cap of 64 KB).
- **LeetCode (sourced):** its Help Center article "What is the difference between Time Limit Exceeded (TLE) and Timeout?" (opened 11 Nov 2026) says TLE can mean "an underlying infinite loop", "too slow… high time complexity" or an invalid returned data structure. "Timeout simply means the server queue is busy." That's the same queue-plus-limits model.
- **Internals [GK]:** neither company publishes its internal sandbox design on the pages I opened. That online judges generally run each submission in an isolated container or VM with cgroup limits, no network and a worker queue is **general knowledge**, not a cited fact about either company.

---

## 7. Non-functional requirements
- **Security:**
  - Untrusted code only in the sandbox (§6). OWASP basics: parameterised SQL, CSRF protection on forms, strict CSP on the app, HttpOnly cookies, Argon2id for any passwords.
  - Secrets only in server env vars. Ownership checks on every query.
  - No user code in plain logs. Dependency licence checks in CI.
- **Performance [E]:**
  - Run (Python/JS) under 1.5 s p95 with a warm image; stretch languages: C++ Submit under 3 s, Java under 3.5 s [U].
  - Ghost hunt ≤ 8 s hard cap. First per-test event ≤ 1 s after the test finishes.
  - Landing page Lighthouse performance ≥ 90 on mobile. IDE page interactive in < 3 s on a mid-range laptop over a 10 Mbps link [E].
- **Capacity and hosting:**
  - One **OVH VPS-2** (4 vCPU, 8 GB, 75 GB NVMe; **€8.49/month ex. VAT**, with VAT and backups extra) [D 12 Nov]. **2 concurrent sandbox runs by default** until benchmarked [D 12 Nov], with the rest queued in Postgres.
  - It must survive about 20 concurrent submits (load test on freeze day) without failures, only queueing.
  - Docker Compose + Caddy (HTTPS).
- **Availability:** reachable and **free for judges until Fri 29 Jan 2027 23:00 SAST** (Judging Period ends 29 Jan 2027 1:00 pm PT). Daily Postgres backup. Uptime monitor.
- **Accessibility:**
  - WCAG 2.2 AA contrast (palette v2 checked);
  - a full keyboard path for the IDE;
  - `prefers-reduced-motion` honoured (no GSAP scenes, familiar badges instead of bursts);
  - `aria-live` for verdicts and familiar mood;
  - no information by colour alone (icons + text for pass/fail).
- **Privacy:** minimal data (email, provider id, submissions). Account deletion removes personal data. AI hints send code + ghost only, no personal data.
- **Cost:**
  - The VPS is the only fixed cost (system design §11).
  - AI hints cost about $0.03 per active user per month [E], with a daily cap per user.
- **Maintainability:**
  - Languages as config (`languages.toml`); problems as files (packs);
  - property-based tests for the engine and the sandbox limits;
  - MIT licence; `/.kiro` committed.
- **Determinism:** the ghost seed is derived from (problem, code hash, language, engine version), and the result is stored with the seed, generator version, problem version, image digest and shrink steps.

## 8. MoSCoW priority (hackathon build)
| Priority | Items |
|---|---|
| **Must** | ACC-1, ACC-2, ACC-6 (judge demo account) · PRB-1, PRB-3 · IDE-1 (Python + JavaScript), IDE-2, IDE-3, IDE-6 · **VAL-1, VAL-2, VAL-3, VAL-4, VAL-5** · **GHO-1, GHO-2** · HNT-1 · **FAM-1** (measured mood) · DSH-1 · PCK-1, PCK-2 · sandbox criteria (worker-owned verdicts, rootless sandbox daemon) · landing page · MCP-1 · **10 polished original problems** in Python + JS [D 12 Nov] |
| **Should** | **GHO-5 (shrink animation), GHO-6 (Bug Autopsy), MCP-3 (Kiro dogfooding demo)** [D 12 Nov] · FAM-6 (AI speech bubble) · IDE-4, IDE-5 · GHO-3, GHO-4 · HNT-2, HNT-3 · FAM-3, FAM-4, FAM-5 · DSH-3 · PRB-2 · ACC-3 (user-initiated linking), ACC-4 · ADM-1 · MCP-2 (Power) |
| **Could (stretch; won't start before the freeze-safe core)** [D 12 Nov] | Java + C++ · DSH-2 (FSRS) · FAM-2 (live typing guess) · TRL-1, TRL-2 (Night Shift) · BRK-1, BRK-2, BRK-3 (break room) · ADM-2, ADM-3, ADM-4 · ACC-5 · gVisor runtime |
| **Won't (this time)** | PAY-1, PAY-2 · cohort/instructor dashboards · mobile apps |

Cut order if behind (from system design §14): brain game → chess bot → puzzle → Night Shift → FSRS → live-typing guess → Java/C++ → familiar speech bubble → whole familiar. **Never cut:** the ghost loop, login, IDE with live results, dashboard, landing polish.

## 9. Out of scope
- Payments and subscriptions (designed later, see E11).
- Proctoring, webcam monitoring, plagiarism detection.
- Company hiring tools, candidate pipelines, interview scheduling.
- Native mobile apps (the web works on tablet).
- Copying HackerRank/LeetCode problems, text or trade dress (patterns only; original problems).
- Running user code in the browser (all judging is server-side).
- Multiplayer duels, community-submitted packs without review, light theme (tokens are ready, but it isn't built).
- Languages beyond Python and JavaScript at launch, with Java and C++ as stretch (config makes later additions cheap).
