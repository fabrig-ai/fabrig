# A brief's pull section fits its deadline

Draft, 2026-11-14. On a deployment, a change whose pull request had 196 changed files could not start any agent that reads the pull request. Its CI repair's brief failed "ci: not gathered by its deadline" 26 times. Its conflict resolution's brief failed "pull: not gathered by its deadline" 13 times and was held. No run started, so no agent could work around it. The goal routed around it by cutting a separate catch-up change.

## 1. What happens

- jig gathers a brief under one deadline, `root.brief.gather_ms` (default 30 s, `temper/crates/temper-engine-service-vocabulary/src/limits.rs`). A required section that hasn't arrived when the deadline fires fails the whole brief as an infrastructure failure (`jig/crates/jig-domain-brief/src/orchestrated.rs`, `fire` and `late`). The task then retries with backoff, up to its allowance, and holds.
- temper's forge owner gathers the pull section page by page, with each file's before and after (`temper/crates/temper-engine-domain-forge/src/brief.rs`, the `BriefStage::Files` reads). It stops early only at the section's byte room, or at a page too large, with "[more files omitted]" and the way to read the rest (`read_pull_files`). It never stops for time. A pull request whose pages take longer than the deadline therefore fails the brief, however much it had already gathered.
- The CI section has the same shape.

## 2. Wanted

- **A section that can be cut is cut in time, never late.** A connector section that has gathered part of its content before the brief's deadline ends with what it has, and its usual marker says what was left out and how to read it (for the pull section: the files omitted, from which page, and that `read_pull_files` lists them). It doesn't miss the deadline. Its owner learns the time it has left from the plan, or the brief asks its owners to finish ahead of the deadline: the goal's choice, by `domain/engine/briefs.md`, 1.
- **A brief fails only when a required section has nothing usable:** for example, the pull request's head or metadata couldn't be read at all.
- **The words stay honest:** a brief cut for time says so in the section ("cut at the brief's deadline: N of M files"), separately from the byte room's cut.
- Apply the same to the CI section's logs, and to any other connector section that reads in pages.
- Update `jig/docs/design/domain/engine/briefs.md`, 1, and temper's change documents where they describe the pull and CI sections, so the docs say what the code does.

## 3. Out

- Raising the default deadline: the deployment's override is separate.
- How many files a pull request may have.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- A focused test in jig's brief domain: a required connector section that reports a partial result before the deadline yields a brief, not a failure. One with nothing by the deadline still fails, as today.
- A focused story in temper's engine domain tests (`temper/tests/engine/domain/tests/suite/`): a change with a pull request whose files take more pages than the deadline allows gets a resolution brief. Its pull section is cut for time, with the marker, and the resolution's run starts.

Run only the new and affected tests, by name.
