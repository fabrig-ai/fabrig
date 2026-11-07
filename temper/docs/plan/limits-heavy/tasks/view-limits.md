+++
id = "view-limits"
part = "limits"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/tests/limits.rs",
  "temper/docs/plan/limits-heavy/figures/view-limits.md",
]
effort = "high"
minutes = 90
+++

# The view's limit tests: why does a window at its bound cost seconds?

## Goal

`temper-client-view`'s `tests::limits` is 6 tests and 9.1 CPU s: `a_window_full_of_small_lines_builds` 2.7, `kept_words_at_the_threads_bytes_build` 2.5, `a_window_full_of_long_words_builds` 1.9. They belong where they are (`plan.md`, *Belonging*). Find why each costs what it does, and fix the cause, in the test or in the code it runs.

Acceptance:

- All six tests still exist, reach the same bounds and assert what they assert today. No bound in them is lowered.
- `temper/docs/plan/limits-heavy/figures/view-limits.md` holds each of the six, before and after in instructions, with the cause of each.
- The next cause's gain is under about 0.2 CPU s, or what's left is under *Found and not fixed*.
- A product change has its own focused test, and changes no behaviour.

## Inputs

- `temper/docs/plan/limits-heavy/inputs/request.md`.
- `temper/docs/plan/limits-heavy/notes/common.md`: how to measure, the figures' form and the rules.
- `temper/docs/plan/limits-heavy/plan.md`, *Belonging*.
- `temper/docs/design/client/README.md`, 5.4: what these tests hold.
- `skein/docs/foundation/testing-strategy.md`, 8, *A test is a process* and *A slow test is understood, never hidden*.

## Owns and may change

Owns `temper/crates/temper-client-view/src/tests/limits.rs` and its figures.

Readers: `repeated` is read as `super::limits::repeated` by `temper-client-view/src/tests/chat.rs` and `tests/task.rs`. Its signature stays, so they need no edit. `tests/fixtures/panel.rs` has a `use super::limits` of its own; read it before you change what `limits.rs` exports.

May change, at the size the cause needs and no more:

- `temper-client-view/src/tests/fixtures.rs` and `tests/fixtures/**` (`limits-fixtures` is here too), and `tests/documents*`.
- `temper-client-view/src/**`, `temper-client-domain/src/pages/chat*`, `jig-client/src/thread/**`, `skein-ui/src/**` and `skein-lib`, for a cause the profile puts there. skein's and jig's only if small and clearly theirs.

Options, each decided:

- **The cause is building the test's own input:** make the building cheap in `limits.rs`, keeping every byte the test draws. A helper that writes a run of bytes is fine: in skein-lib if it is small and general, in the test otherwise.
- **The cause is a scan or a repeated computation in the product** (a weight recomputed per line, a clone per row, a quadratic walk over the lines): fix it there, with its own focused test.
- **The cost is the drawing itself, linear in what the bound holds:** that is what the test is for. It stays, and goes under *Found and not fixed* with its count.
- **A fix would change a shared item's meaning** (what it returns, or an order it keeps): run the tests of every package that reads it, and say which in your report.

## Steps

1. Profile `a_window_full_of_small_lines_builds`, `kept_words_at_the_threads_bytes_build` and `a_window_full_of_long_words_builds`, then the other three. Test filters read `tests::limits::NAME`, with `-p temper-client-view --lib`.
2. Starting points, to check against the profile and not findings: `full_window` and `repeated` put one byte at a time through `Writer::put`, in an unoptimized build. `build_window` clones the line for every position and asks its `weight()` three times there, and `a_window_full_of_long_words_builds` makes the same line again for each row. `Limits::of` and `valid` run more than once a test, and `draw_words` builds them for both shells. `kept_words_at_the_threads_bytes_build` calls `thread.sending` once per chunk, and may scan what it holds on each. What does `thread.page` or `conversation` do per line, and is any of it more than linear?
3. Fix the causes, largest first, re-measuring after each. Stop at the rule in the common note.
4. Write the figures.
5. The implementer's text's end of a run: `cargo fmt`, old forms, `heavy cargo clippy -p temper-client-view --all-targets -- -D warnings` with any other package you changed, the focused tests of those packages and their direct dependents. Commit and push.

## Report

The six tests' before and after in instructions, the causes found and not fixed, and any product change with the test that holds it. Your implementation and check minutes apart.
