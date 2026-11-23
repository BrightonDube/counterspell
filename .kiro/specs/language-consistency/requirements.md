# Language Consistency Requirements

## Goal

Make the catalogue, workspace and judge expose one truthful language contract.

## Requirements

- The global language API may list Python, JavaScript, Java and C++ only when the corresponding drivers are production-ready.
- A published problem exposes a language only when it has complete starter code and is accepted by the run path.
- Selecting an enabled language must never produce a blank editor.
- Run and submit requests must reject unsupported problem-language pairs with a clear bounded error.
- Every new pack must include starters for all four languages before publication.
- Contract tests must cover API language data, problem details, editor selection and judge dispatch.

## Out of scope

- Adding a fifth language.
- Weakening sandbox limits for a driver.
