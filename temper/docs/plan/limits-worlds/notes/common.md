# limits-worlds: the rules every task names

The request is `temper/docs/plan/limits-worlds/inputs/request.md`: read *For every task of this plan*, which binds whatever your spec says. `notes/measuring.md` says how a story is profiled and written down, and `notes/contracts.md` holds the shapes that cross tasks. All three paths are under `temper/docs/plan/limits-worlds/`.

The repair's order is `skein/docs/foundation/testing-strategy.md`, 8.1: does the test belong here; if so, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. **What a test checks decides its place, never its cost.**

## Who owns what

One writer per crate. A cause the profile puts in another task's crates is that task's: name it under `## Left`, with its figures, and go on.

| Task | Owns |
|---|---|
| `figures-before` | its figures file, and no code |
| `cuts-fork`, then `conformance-causes` | `skein/testing/skein-fork`, `jig/testing/jig-conformance`, `temper/tests/conformance` |
| `stories-go` | `temper/tests/real-loop/tests/suite/fuzzy/**`, the case's files `temper/tests/real-loop/src/case.rs` and `src/case/**`, what those stories alone read in `temper/tests/real-loop/src/` |
| `real-loop-causes` | `temper/tests/real-loop/src/**`, after `stories-go` |
| `public-causes` | `temper/tests/one-process/**` |
| `engine-causes` | the crates under the worlds: `temper/crates/**`, `jig/crates/**`, `smith/crates/**`, `skein/crates/**`, and the kits they share in `temper/testing/**`, `jig/testing/**` and `skein/testing/**` but for `cuts-fork`'s two |
| `close` | the account in `temper/docs/development/limits-worlds.md` |

## Departures from the implementer's text

- **A tool the task lacks is otherwise a question.** Valgrind is the request's own method, so `notes/measuring.md` says how to get it with no root and no change to the machine. Only if that fails is it a `Question:`.
- **A local, uncommitted raise of the real loop's patience** is allowed while profiling, as `notes/measuring.md` says, and never reaches a commit. It's the one edit that may sit in your slot while a run goes.
- **`docs/design/`:** where your spec names a design edit (a status entry to remove, a row of `temper/docs/design/domain/testing.md`), make it, in the same task; the request asks for this. No other design text.

## Every fix

- **Profile first, quote before and after.** No change without the figure that put the cause there.
- **Largest first; stop when the next cause's gain is under about 2% of a story.** Say what you stopped at, with its share.
- **No change of behaviour, and a focused test of its own** for a product change, at the size the cause needs. A test that stops doing work it needed isn't a saving: what it checked stays checked.
- **No budget, timeout, deadline or build setting changes, and test builds stay unoptimized.** A per-crate override is the user's. A cost no test can avoid goes in your report, and stays.
- **A change of meaning runs its readers' tests:** if a shared item returns something else or keeps another order, run the tests of each package that reads it.
- **A figure that fell by less than two runs of the same code differ isn't a gain.** Say so.
- **Before calling a failure outside your change,** run it once on the pristine base. If main passes and your branch fails, it's yours.

## Not this plan

- Other plans take the fixtures of the many small tests, and the other heavy tests. Touch only what your spec names and what a profile of a named story puts on its path.
- **No test moves into the focused suite in this plan,** and no test moves to escape a budget. A test past its guide isn't merged with another or named an exception (`temper/docs/design/domain/testing.md`, 6.1).
- Only focused and fuzzy run. No browser, process or live story runs, and no `tools/tiers/run`. A slow-tier story you touch is compiled, not run (`--no-run`), and you say so.

## Beside this plan

Other drafts hold files near here. Keep hunks small, don't reformat, and don't move code you needn't change. If a merge meets another draft in a file your spec names, keep their change, and say so in your report.

- `model-catalogue-2/process-fixtures` holds `temper/tests/real-loop/tests/fixtures/**`.
- `goal-delivery/goal-real-story` holds `temper/tests/real-loop/tests/suite/goal_delivery.rs`.
- `model-catalogue-2/stock-models` holds `temper/crates/temper-fake/src/**`, `temper/crates/temper-playground/**` and `temper/crates/temper-service/src/startup*`.
- `model-catalogue-2/catalogue-clients` and `goal-delivery` hold `temper/crates/temper-engine-domain/src/route/**`.
- `model-catalogue-2` and `ui-model-next-turn` hold `jig/crates/jig-domain-core/src/settings/**`, `jig/crates/jig-domain-tasks/src/**`, `jig/crates/jig-protocol-store/src/**` and their generated code.
