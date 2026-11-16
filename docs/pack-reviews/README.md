# Pack review summaries

This folder holds one public-safe summary per pack, written for Brighton to read in a few minutes. It is made from the review summary template in `.kiro/pipeline/templates/`.

## What is here

- One summary file per pack, named after its slug, for one review round. The batch step writes it after a pass, and it is replaced when the pack is reviewed again.
- This README. It describes the format and does not review any pack.

## What is never here

- Hidden cases, hidden expected values, reference source, ghost values or any pointer into the private repository.
- Scored reports that point at hidden data. Those stay in the private repository under the review folder for the slug.
- Credentials, tokens or paths to credential files.

A scan (`SUM-01`) checks every summary before a pull request is opened. A summary that fails the scan is not written.

## Format

The template has six parts:

1. What the problem asks, in plain words.
2. Why it tests real skill.
3. What the ghost shows.
4. What could still be wrong.
5. The scores in words.
6. Decisions for Brighton.

## Example layout

The layout only, with no real content:

```
# Review summary: example-slug
Round 1, content hash 0123456789ab
## What the problem asks
...
```

Brighton approves a pack by merging its pull request. He sets `status` to `published` in that same commit. The summary is never the approval.
