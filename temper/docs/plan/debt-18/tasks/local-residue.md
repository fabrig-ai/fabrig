+++
id = "local-residue"
part = "smith-review"
kind = "feature"
needs = ["effort-pin"]
files = ["smith/tests/local/src/world.rs", "smith/crates/smith-local-domain/src/parent_tests.rs", "smith/crates/smith-local-domain/src/person.rs", "smith/crates/smith-local-protocol/src/terminal.rs", "smith/tests/machine/tests/suite/stories.rs", "smith/tests/tools/tests/suite/simulation.rs", "smith/tests/tools/src/fixture.rs"]
effort = "medium"
minutes = 35
+++

# smith: the local world reads notices by type, and the review's residue goes

## Goal

`temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, findings 3 and 4. The cut re-read both. They hold, with two
corrections, stated below: merging the local world's `Notice` arms must not widen what its referee sees, and the clips in
`person.rs` stay, with a row.

Acceptance:

- `smith/tests/local/src/world.rs` reaches `Goal::Parked` by matching `local::Notice::Answer(local::Said::Parked)`, not by
  comparing rendered words; `rg 'Chat parked|Run cancelled' smith/tests/local` finds nothing.
- `parent_tests.rs`'s two `settled`-answer checks (at about lines 808 and 950) accept `Notice::Answer(_)` alone, and go red
  if the answer's notice is removed.
- `stories.rs` has no useless import, and it and `smith/tests/tools/tests/suite/simulation.rs` share one `ALL` and one
  `exited`, in `smith_tools_world::fixture`.
- `tool_terminals_keep_their_verdict_words` is gone from `terminal.rs`; `a_turn_shows_the_tool_name_and_its_terminal` has an
  `Ended(Failed)` line and goes red if either verdict's word changes.
- A new row in `person.rs`'s test calls `turn_lines` with a small cap and goes red if either clip is removed.
- The branch's net lines fall by at least 15 (the review's estimates: -11, -10, -9; the row adds a few). Every other test
  passes unchanged.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/inputs/smith-debt14-review.md`, findings 3 and 4

## Owns and may change

- **Owns:** the files in `files`, in these places only.
- **The local world's arms.** The second `Request::Notice` arm also pushes to `self.notices` and calls `self.saw(..)`,
  which the first doesn't do: `Turn` and `Answer` notices must keep not reaching them, since the referee would see more.
  Merge into one arm guarded on that, if it's shorter; if not, keep two and report it. `self.shown` keeps its rendered
  text, so the world keeps `smith_local_protocol::notice_text`. The `b"interrupted: the run was cancelled\n"` comparison in
  the second arm stays: it isn't in the finding.
- **`Run cancelled` is dead:** no smith code says those words, and `Said` has no such variant. If a `Said::Text` fixture
  does say them, keep the check and report it.
- **The clips stay.** The review offers dropping `cap` or adding a row. Dropping it makes a `Notice::Turn` copy a whole
  segment rather than `show_bytes` of it, which moves what a notice holds, so add the row.
- **Leave the other copies of `ALL`** (in step crates' tests and the benchmark grader): they can't read a world.
- **`shown_requests` in `parent_tests.rs`** counts both notices, on purpose: it stays.

## Steps

1. The local world (finding 3).

2. `parent_tests.rs`'s two narrowings, and `person.rs`'s row: for instance `turn_lines(&turn, 4)` gives
   `TurnLine::Text(b"Work")` and `TurnLine::Tool { name: b"insp" }`.

3. `stories.rs`: drop `use self::ALL;` and whatever the inner `use`s repeat of the file's own (`Token`, `FromDomain`,
   `ToDomain`). `pub const ALL` and `pub fn exited` go into `smith/tests/tools/src/fixture.rs`, and both suites read them.

4. `terminal.rs`'s tests: the row, the deletion.

5. Break each by hand and restore: `Said::Parked`'s match in the world; a verdict's word; each clip; the answer's notice
   for the two narrowed checks. Report what failed.

6. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `heavy cargo clippy -p smith-local-world -p smith-local-domain -p
   smith-local-protocol -p smith-machine-world -p smith-tools-world --all-targets -- -D warnings`; `heavy cargo nextest run` on
   the same packages; the citations check. Commit and push.

## Report

The common note's items, what you broke and what failed, and whether the world's arms merged.
