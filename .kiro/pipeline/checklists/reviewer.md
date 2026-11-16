# Reviewer checklist

Use this checklist to score one review round. Score each category from 1 to 5. For every score, cite a file and a pointer, such as `hints.toml rung 3` or `hidden.toml /case/3`. Never quote a hidden input or expected value (PIP-22). Each finding has an ID, a severity (`blocker`, `major` or `minor`), a category and a pointer (PIP-24).

A round passes only when every category scores 3 or more, the mean of the seven scores is 4 or more, correctness scores 5, originality scores 4 or more with no `ORG-*` finding, and no finding is a `blocker` (PIP-23).

## Correctness
- `RV-correctness-1` The samples and the reference agree, as the S10 sandbox report shows.
- `RV-correctness-2` Every hidden expected value follows from the statement.
- `RV-correctness-3` The reference handles the empty input and the maximum input that the statement describes.

## Clarity
- `RV-clarity-1` The call, its arguments and its return type are named in the task section.
- `RV-clarity-2` The worked example matches sample 1.
- `RV-clarity-3` Edge cases are stated in words.

## Originality
- `RV-originality-1` The originality record has all five sections.
- `RV-originality-2` No originality marker appears in any file (`ORG-01` passes).
- `RV-originality-3` No trade dress and no third-party title appear.

## Difficulty
- `RV-difficulty-1` The concept count and the reference length match the `difficulty` value.
- `RV-difficulty-2` At least one hidden case needs the key idea of the problem.

## Hints
- `RV-hints-1` Rung 1 is a nudge only.
- `RV-hints-2` Rung 3 is pseudo-code of three lines at most.
- `RV-hints-3` No rung names a bug the learner may not have. The leak check (`HNT-03`) passes.

## Ghosts
- `RV-ghost-1` There are at least two known ghosts, and each is a realistic beginner mistake.
- `RV-ghost-2` Each explanation matches its incorrect program.
- `RV-ghost-3` The S11 report shows every known ghost found and the reference not found.

## Access
- `RV-access-1` Sentences are under about 25 words where possible.
- `RV-access-2` No idiom or jargon appears without a definition.
- `RV-access-3` No meaning depends on colour alone.
