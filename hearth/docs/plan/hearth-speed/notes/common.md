# hearth-speed: rules every task names

The request is `hearth/docs/plan/hearth-speed/inputs/request.md`. The budget is `hearth/docs/design/domain/testing.md`, 9.1. The figures and the profile are in `hearth/docs/plan/hearth-speed/figures.md`, once `figures-before` has written them.

## What doesn't move

- **Every assertion and every case stays.** No seed, scenario or count shrinks. No test leaves the focused suite, and the filters in `.config/nextest.toml` don't change. A world's seed still replays. What a test proves doesn't change, and neither does what hearth does for a user.
- **The budget stays:** `tools/tiers/budgets.toml` doesn't change.
- **A memory ceiling doesn't move.** If a change fails a ceiling test, or any test that reads a peak, that's a finding: take the change out and say so in the report.
- **skein, smith, jig and temper change in no file.** A saving that would need one, or a profile line for one of their crates, goes under `## Left` in your report with its measured saving and build cost. That's a question for after this plan, not your task's.
- **A failing test is a finding first.** Read the log and find what changed. Don't weaken a test to get past it.

## The aim

The figure that counts is the queue's, as `tools/tiers/budgets.toml`'s `[gate.hearth.default]` records it: 31.7 CPU s over 2,189 tests before this plan, and the aim is 27. No task can run the queue, and a builder reads higher: 41.7 by the method below on 11-06. Why the two differ is unknown, and it isn't yours to resolve.

So a task's aim is a ratio: **the method's figure at most 85% of the same machine's figure at the base commit** (27 / 31.7 is 0.852). A figure from one machine is never compared with another's without saying so. **Stop once the aim is met:** what's left unsaved isn't a reason to go on.

## The base commit

`483dfed6aa8639dc7df93a4e381c6ae2458c4468`, this plan's start. Every "before" is measured there, whatever the trunk has merged since: in your slot, `git checkout --detach` it, measure, and return to your branch with `git checkout`. No file changes on the way.

## The method

`hearth/docs/design/domain/testing.status.md`'s **Departs** on 9.1, which is `tools/tiers/design.md`, 7, 7.3 and 10.2 with a caller of its own:

- **A throwaway driver, pre-approved:** a short untracked script in your slot builds the wrapper once (`heavy cc -std=c11 -O2 -static -Wall -Wextra -Werror` from `tools/tiers/cpu-wrapper.c`), loads `tools/tiers/cpu.py` with `runpy.run_path`, calls `prepare(DIR, "default", WRAPPER)`, appends the arguments to `heavy cargo nextest run --profile default -p <each package>`, then calls `execute` and `summary`. A fresh DIR for each run. It's deleted before the push and named in the report.
- **The environment:** `NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none`.
- **The packages:** from `cargo metadata --no-deps --format-version 1`, every workspace member whose manifest is under `hearth/` (34 on 11-06). No test filter and no concurrency override.
- **Build first** (`heavy cargo nextest run --no-run` with the same packages), then measure, so that a run waits on no compile. The figure excludes compilation anyway.
- **Twice on each side,** back to back on one machine. Give both figures, and compare the lower of each side.
- **The families** are the ones the status names: `hearth-builder-world`'s `suite` binary (the builder suite), `hearth-tree-world`'s `suite` (the tree suite), `hearth-builder-world`'s `real` (the real-git binary), `hearth-render-world`'s `suite` (the render suite), and all the rest.
- **While searching,** a family's binary alone is enough (`-p PKG --test BIN`): the same collector or nextest's own times, labelled. What goes in a commit and a report is the method's figure.
- **If the collector can't run through heavy,** say what stopped it, take the figures from a plain `heavy cargo nextest run` at the same profile (`--message-format libtest-json` or JUnit), and label them as that.
- **A figure you can't get is `unknown`,** with why. No figure is a guess. The machine comes from `hostname` and the time from `date -u`.

## The end of every task

- `cargo fmt`, and `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` for what you changed.
- From `hearth/`, `python3 -I -B scripts/old_forms.py`, with `--write` only to lower a count for what your task removed.
- From the root, `python3 -I -B tools/citations/citations.py --all` stays at 0.
- `tests/boundary/tests/outward.rs` (package `fabrig-boundary`) keeps passing. Files outside `docs/plan/` name no machine, tool or session of whoever runs this development: `hostname`'s answer stays in `figures.md`, a commit and a report.
- **Each commit says what it saved, measured:** the method's figures before and after, with the machine. A commit that saves nothing measurable doesn't land: take it out, and say so in the report.
