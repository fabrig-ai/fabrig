+++
id = "tui-walk"
part = "tui-journeys"
kind = "feature"
needs = ["tui-replay"]
files = ["temper/crates/temper-client-shell-tui/src/journey/walk.rs", "temper/crates/temper-client-shell-tui/src/journey/walk/tests.rs"]
effort = "high"
minutes = 75
+++

## Goal

The walk of a TUI journey, natively and without io (`temper/docs/design/client/tui/journeys.md`, 8.1): its `walk` file read as steps, the check that a new recording follows it, and the two readings `again` makes of a save: whether its frame shows a text, and its last `browse` address.

Acceptance:

- Contracts, section 8, holds as written in `src/journey/walk.rs`, re-exported from `src/journey.rs` (one `pub use` line there).
- `walk`: 8.1's records on 3.2's syntax, `walk 1` first and `end` last and once; another version, a record out of its place, a second `end`, none, an unknown record, a `key` unreadable as 3.2's key, each unreadable, naming the line.
- `follows`: the recording's `key` records are the walk's `key` steps, in order and no others, and a `browse` stands before each `approve`'s place (after the keys before it); else the first step, from 1, that differs.
- `shows`: the save replayed to its end (`journey::Replay`), the host's frame's `Frame::text` holding the text; a stale or unreadable save is its refusal.
- `browsed`: the last `browse` line's address, or none.
- Tests in `src/journey/walk/tests.rs`: a walk read back, each record among them, and the unreadable rows; `follows` true for `sign-in-approved`'s walk (`walk 1`, `see "then press Approve."`, `mark`, `approve`, `see "Connected to"`, `end`) against an approval walk recorded in the host's world, and naming the step for a key added, a key missing and a recording with no `browse`; `shows` and `browsed` on the same walk's saves.
- `heavy cargo nextest run -p temper-client-shell-tui` passes; clippy clean with and without `--features record`.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, 8.1 and 3.2.
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 5, 6 and 8 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `src/journey/line.rs`, `record.rs`, `replay.rs` and the host world's helpers in `src/journey/tests.rs` (merged); the browser's `temper-client-shell-web/src/journey/setup.rs`, the model of a walk's file.

## Owns and may change

- Owned: the files above, and one `pub use` and `mod` line in `src/journey.rs` (`tui-redact` and `tui-record` work beside you: keep your hunk to those lines).
- Options:
  - A `key` step written exactly as a recording's `key` field without its event (default).

## Steps

1. Read the inputs. `WalkStep` and `walk`; `follows`; `shows` and `browsed`.
2. The tests above. Break a rule once by hand (`follows` ignoring order; a second `end` read), see a test fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`, and with `--features record`; `heavy cargo nextest run -p temper-client-shell-tui`.
4. Commit and push.

## Report

The implementer's report, with: the tests; each broken rule's failure.
