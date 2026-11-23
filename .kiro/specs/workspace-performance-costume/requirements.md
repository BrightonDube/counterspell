# Workspace Performance and Costume Requirements

## Goal

Make the coding workspace feel immediate while preserving and improving its Steel Night costume presentation.

## Requirements

- Typical mobile time to interactive must be under three seconds on the agreed test profile.
- Problem text, editor shell and primary controls are the critical first paint.
- Monaco core, selected language support and required worker load before optional editor features.
- Console history, hints, Familiar, results and decorative components initialise only when needed.
- Decorative media is preserved, lazy-loaded or deferred, explicitly sized, responsively encoded and cached.
- Components that are off-screen or collapsed must not all paint or initialise during the critical path.
- Existing layout order remains stable unless measured responsive improvements require bounded changes.
- Editing, run, test, submit, console, hints, ghost states and result feedback retain full functionality.
- Motion is atmospheric, compositor-friendly and disabled or simplified by `prefers-reduced-motion`.
- Every slice records before and after interaction, Lighthouse, bundle, transfer and layout-shift evidence.

## Out of scope

- Removing costume media as a performance shortcut.
- Replacing Monaco with a less capable editor.
