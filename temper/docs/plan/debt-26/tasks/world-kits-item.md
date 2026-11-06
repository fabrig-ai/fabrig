+++
id = "world-kits-item"
part = "world-kits-item"
kind = "docs"
needs = []
files = ["jig/docs/development/debt/D044-worlds-are-shims-over-kits.md", "jig/docs/development/debt/README.md", "skein/docs/design/testing.md"]
effort = "medium"
minutes = 20
+++

# Open jig's item for worlds that are shims, and say how far skein's gate reaches

## Goal

D031 was largely a rename: open the item that says so, and don't pay it here. And `skein/docs/design/testing.md`'s "in any tree" reads wider than the gate does: say what the gate reaches.

Acceptance:

- `jig/docs/development/debt/D044-worlds-are-shims-over-kits.md` exists in the README's form, `open`; the README's highest ID is D044. If D044 is taken on your base, take the next free ID and say so.
- `skein/docs/design/testing.md`'s *A world's library is its own* says the gate's real reach, in the words below.
- `python3 -I -B tools/debt/debt.py` exits 0, `python3 -I -B tools/citations/citations.py --all` stays at 0.

## Inputs

- `temper/docs/plan/debt-26/inputs/debt24-review.md`, finding 2 of the first review.
- `jig/docs/development/debt/README.md`, the item's form and the rule on its highest ID.
- `skein/docs/design/testing.md`, *A world's library is its own* (about line 386), and `tests/boundary/tests/skein_worlds.rs`'s `world_edges`.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the new item, the README's highest-ID line, and that one paragraph of `testing.md`: a stated departure from the implementer's text. No code changes, and no kit moves: this item isn't paid here.

## Steps

The cut has read these against the trunk, on 2026-11-06. Recount each number as you write it, and write what you see:

- `jig/tests/client`, `channel`, `engine-clients` and `tasks` hold a `src/lib.rs` of 2 or 3 lines, `pub use <kit>::*`. skein's `tests/llm` does the same.
- `jig/testing/jig-client-kit` holds about 19,181 lines and 158 `#[test]`s, and hasn't `test = false`. `jig/testing/jig-tasks-kit` holds about 3,208 lines; outside `jig/tests/tasks` only two temper files read it (`rg -l jig_tasks_kit`).
- The review's other facts are by reading, and you cite them as the review's: 46,295 lines moved into kits for net +79, and the client kit's modules are mutually cyclic, so a fixture change still breaks dependents.

The item:

```markdown
# D044: Worlds are shims over kits that hold the worlds

- **Kind:** tests
- **Where:** `jig/tests/client/src/lib.rs`, `jig/tests/channel/src/lib.rs`, `jig/tests/engine-clients/src/lib.rs`, `jig/tests/tasks/src/lib.rs`, `jig/testing/jig-client-kit`, `jig/testing/jig-tasks-kit`
- **Seen:** 2026-11-06 at `683f4e8b92`, in the spot review of debt-24 (finding 2)
- **Checked:** 2026-11-06
- **Value:** medium (a fixture change stays in its story's world)
- **Size:** L
- **Status:** open
- **Paid when:** a kit holds only what a second package reads, under `test = false`; its tests are back in their world's suite; and a kit's modules aren't mutually cyclic, so a fixture change breaks only its readers.
- **Measure:** `rg -c '#\[test\]' jig/testing/jig-client-kit/src | awk -F: '{s+=$2} END {print s+0}'` (158)

## What
## Why it matters
## What pays it back
```

**What** holds the numbers above and the review's real gains: the directory predicts what's a kit, no aliases, no shipping crate depends on a kit (`jig/tests/boundary/src/tests/kits.rs`). **Why it matters**: D031 moved the worlds without narrowing what a fixture change breaks. **What pays it back**: move kit tests back to world suites; `test = false` as the rule. Cite the review by its name and merge, never by a path under `temper/docs/plan/`: a plan's files leave main when it retires. The item says nothing of skein's `tests/llm` beyond the one sentence above.

The paragraph in `testing.md` becomes (the rest of it as it stands on your base):

> **A world's library is its own.** No package, whichever tree it's in, depends on a package under `skein/tests/`: what a second crate needs from a world lives in a kit under `testing/` (testing-strategy.md, 7), and the world's own suite reads it through its library, which re-exports the kit. No `#[path]` under `skein/` reaches outside its package. `tests/boundary/tests/skein_worlds.rs` holds both, for `skein/` alone: it doesn't read the worlds under smith's and temper's `tests/`, and packages still depend on those.

Check that last clause against `world_edges` and `path_crossings`, and against the edges in `cargo metadata --format-version 1 --no-deps` (it builds nothing). On 2026-11-06 the cut counted about 67 onto `smith/tests` and 26 onto `temper/tests`. Commit, push.

## Report

As your text says: the numbers you recounted, and that no test ran because no code changed.
