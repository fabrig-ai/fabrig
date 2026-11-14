# Say why a commit failed

Draft, 2026-11-14. A deployment's engine stopped at every start with only `temper-engine serve: commit 685204 failed`. Finding why took an instrumented build replaying a copy of its store: skein-kv had refused the commit as `Full` (its map at 268,433,350 of a 268,435,456-byte budget). The words should have said so.

## 1. What happens

- The store's `Event::Failed { number }` (`jig/crates/jig-protocol-store/src/directory.rs`, from a skein-kv `Refused` or `Failed`, or a `Committed` whose owner isn't its number, and from `commits.rs`'s `Refusal::Failed`) carries only the commit's number. The engine stops with `Stop::CommitFailed { number }` (`temper/crates/temper-engine-service-component/src/stop.rs`), and its line (`temper/crates/temper-engine-shell/src/lines.rs`) prints only that.

## 2. Wanted

- **The stop line names the cause,** bounded and with no record's contents: which of the store's failures it was (skein-kv refused it, and its refusal's kind: `Full` with the map's used bytes, its budget and the commit's projected bytes; a value or a frame too large; an owner mismatch; a commit the store's own checks refused, naming the check), for example `temper-engine serve: commit 685204 failed: the store's map is full (268,433,350 of 268,435,456 bytes; this commit needs 268,435,862)`.
- The cause travels in `Stop::CommitFailed` (and the store's `Failed` event) as data, not text built in the store.
- Nothing else changes: the engine still stops at a failed commit.

## 3. Out

- Raising the budget, retention, what fills the map.

## 4. Tests

Cheap, focused and fuzzy suites only; walk = false. A focused test in jig-protocol-store makes skein-kv refuse a commit as full and sees the cause in `Failed`; one in temper-engine-shell's lines prints the stop line's words. Run only the new and affected tests by name.
