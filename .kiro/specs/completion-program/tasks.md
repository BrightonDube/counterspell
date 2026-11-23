# Counterspell Completion Program Tasks

## Working rules

- Use one branch, worktree and pull request at a time.
- Write a failing behavioural test first for every defect or feature.
- Use diagnosis before changing a reported broken or slow path.
- Use the relevant specialist skill for each slice.
- Merge only after required CI and review pass.
- After merge, clean the branch and worktree, wait for deployment and verify `https://counterspel.online`.
- Do not tick an operator or production evidence task without direct evidence.

## Wave 0: close active work

- [x] **CP00 Live-verify static cache policies**
  - Evidence: [static cache production verification](verification.md).
  - Immutable assets return `public, max-age=31536000, immutable`.
  - Art, fonts and landing assets return `public, max-age=604800`.
  - `robots.txt` returns `public, max-age=3600`.
  - Matching 304 responses retain each policy.
  - Missing immutable assets and encoded traversal requests receive no immutable policy.

## Wave 1: specifications

- [ ] **CP01 Publish this umbrella specification**
- [ ] **CP02 Create the production audit fixes child spec**
- [ ] **CP02a Create the production foundations child spec**
- [ ] **CP03 Create the language consistency child spec**
- [ ] **CP04 Create the accessibility, SEO and authenticated quality child spec**
- [ ] **CP05 Create the workspace performance and costume child spec**
- [ ] **CP06 Create the Lantern Recall polish child spec**
- [ ] **CP07 Create the themed chess experience child spec**
- [ ] **CP08 Create catalogue batch child specs 01 through 20**
- [ ] **CP09 Review the child-spec dependency graph and implementation frontier**

## Wave 2: reliability and product foundations

- [ ] **CP10 Implement production foundations in focused tracer-bullet pull requests**
- [ ] **CP11 Implement language consistency in focused tracer-bullet pull requests**
- [ ] **CP12 Implement accessibility, SEO and authenticated quality in focused tracer-bullet pull requests**

## Wave 3: core product polish

- [ ] **CP13 Implement workspace performance and costume polish**
- [ ] **CP14 Implement Lantern Recall image-backed reveals**
- [ ] **CP15 Implement the responsive themed chess experience**

## Wave 4: catalogue

- [ ] **CP16 Author, review, sandbox-validate and publish batches 01 through 05**
- [ ] **CP17 Author, review, sandbox-validate and publish batches 06 through 10**
- [ ] **CP18 Author, review, sandbox-validate and publish batches 11 through 15**
- [ ] **CP19 Author, review, sandbox-validate and publish batches 16 through 20**

## Wave 5: final evidence

- [ ] **CP20 Run the full public production audit**
- [ ] **CP21 Run the full authenticated production audit**
- [ ] **CP22 Verify the 200 new packs contain 80 easy, 80 medium and 40 hard packs with four-language coverage**
- [ ] **CP23 Record human-gated infrastructure and operational evidence**
- [ ] **CP24 Reconcile every child-spec checkbox against direct evidence**
- [ ] **CP25 Publish the final production verification summary**

## Dependency frontier

- CP01 to CP09 complete before implementation starts.
- Production audit tasks PA01 and PA02 complete before CP10 starts.
- CP10 blocks CP11 to CP15 where they depend on corrected production contracts.
- CP11 completes before new catalogue publication.
- CP12 completes before final UI performance baselines.
- CP13 completes before CP14 and CP15 so shared media and loading patterns are stable.
- CP10 to CP15 complete before CP16.
- CP16 through CP19 are sequential batch groups; each individual batch is reviewed and live-verified before the next.
- CP20 to CP25 start only after all actionable implementation and catalogue tasks finish.
