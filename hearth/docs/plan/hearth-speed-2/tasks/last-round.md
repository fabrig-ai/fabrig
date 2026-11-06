+++
id = "last-round"
part = "last-round"
kind = "feature"
needs = ["builder-repeats", "tree-once", "render-repeats", "scratch-objects", "path-bytes"]
files = ["hearth/docs/plan/hearth-speed-2/figures.md", "hearth/crates/**", "hearth/testing/**", "hearth/tests/*/src/**", "hearth/tests/*/tests/suite/**"]
effort = "high"
minutes = 60
+++

# If the aim isn't met, the largest of what's left

## Goal

Every family's repairs have merged. Measure. If the aim is met, record it and stop. If not, take what the profile shows is largest of what's left, across every family, and stop at the aim or when the options end. If they end short of it, the figures say by how much, for the user's ruling.

Acceptance:

- `figures.md` gains `## Repairs`: the method's two figures at the base commit and at your start, on one machine, the ratio, and the families. That's your first commit, whichever way it comes out.
- **If the ratio is at most 85%:** done. No other file changes, and your first line says so.
- **If not:** one commit per repair, each with what it saved, largest share first, until the ratio is at most 85% or the options end. `## Repairs` gets the final figures and what's left, by how much.
- Every test of `hearth/`'s packages passes in `default` and `fuzzy`, with every assertion, case, seed and count as it was.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `## Profile`, `## A test's process` and `## What it found`
- `skein/docs/foundation/testing-strategy.md`, 8.1
- `skein/docs/foundation/programming-model.md`, 10, for a step crate

## Owns and may change

- **Owns:** `figures.md`'s `## Repairs`, and for a repair, `hearth/crates/**`, `hearth/testing/**`, `hearth/tests/*/src/**`, and helpers and loop shape in `hearth/tests/*/tests/suite/**`. You run alone, so every family's files are yours. `hearth/tests/*/tests/suite/**` keeps every `assert` on the same values.
- **Departs from your text:** a run that finds the aim met pushes only `## Repairs`, and adds no test. A profile may be stale by now: take callgrind again on the test you repair, for what remains.
- **Options, pre-approved, in the order the shares put them.** Each is in `hearth/crates/hearth-domain-render/src/`, what a user runs, and keeps every byte, order and refusal unless it says otherwise:
  1. **The per-task listing scans:** `call/files.rs:32-38`, `call/files/sources.rs:74-75` and `kinds/scripts.rs:24,44` call `class()` on every entry per task per target. `Side` gains `by_task: Map<Box<[u8]>, Box<[u32]>>`: entry positions in listing order for tree, template and script classes, built once beside `compose()`. They walk those positions and still call `class()` on each, so order, faults and invalid-UTF-8 handling stay. `hearth/crates/hearth-domain-render/src/tests/fixtures/kinds/sources.rs:59` and `hearth/crates/hearth-domain-render/src/tests/fixtures/scripts.rs:29` call these directly: a `Side` that doesn't pass through `compose()` needs the index built too.
  2. **The exports cache:** `prime` restarts at the start after each pair, and `needed` scans all targets (`exports/cache.rs:36-71,201-222`). A cursor, and a read set built from the reader's row. The same pairs in the same order, one per resume, with the same `limited` check first (`cache.rs:31`), `pub(crate)`.
  3. **`placement()`** clones up to three placements (`compose.rs:524-534,691`): a private `placement_ref -> Option<&Placement>`, with `placement()`'s signature kept. It cheapens what the exports cache repeats per pair.
  4. **Rescans in a call:** `needs` per entry (`call.rs:221-233,258`), a union built in `compose()` and **cleared in `fail()`**, since `fail` empties the targets (`call.rs:295`). `contains` per entry (`call.rs:201,282`, `call/files/sources.rs:103`). `copies` per rendered file (`call/templates.rs:155`, `call/files/sources.rs:94-110`).
  5. **`retain_spec`'s re-walk** (`call/specs.rs:233-262`): a running tally. It must make the exact same admission decisions at the limit, so a test first holds it equal to the full `held`. It's superlinear and matters mainly for the large render, so it's last.
  6. **In the other worlds:** `hearth/tests/config/src/world.rs:86` runs `reference::decode` per seed on the same bytes: a `thread_local` memo by bytes and limits. `hearth/testing/hearth-fake-machine`'s second path resolution, if the shares say so.
  7. **What the profile shows, in hearth's files, that no option names,** on `notes/repairs.md`'s terms.
- **Not options:** a build setting of any kind; `.config/nextest.toml`; the budget; a memory test in a binary of its own (`hearth/docs/design/domain/testing.status.md`, 8.4); a process's floor, unless the profile shows hearth's own code in it.
- **A cost in skein, smith, jig or temper** stays there, and that's the request's rule: a question first, not a task. If the aim is unmet when your options end and `figures.md`'s `### Shared kits` holds a cost that would close the gap, put its measured share under `## Left` and say by how much it would. `close` carries it to the user.

## Steps

1. The method's figures at the base and at your start. Write and commit `## Repairs`. Met: push, report, stop.
2. Otherwise, the largest share first: the repair, its family's figure, then the method's figure. Keep it if it saves, with a commit that says what.
3. Stop at the aim or at the end of the options. `heavy cargo nextest run` over every package under `hearth/` (the driver's list), then `--profile fuzzy` over the same.
4. The end of `notes/common.md`. Delete what's untracked. Push.

## Report

The ratio at your start. Each repair and its saving, or none. The final figures, the machine, what's left by how much, and `## Left`.
