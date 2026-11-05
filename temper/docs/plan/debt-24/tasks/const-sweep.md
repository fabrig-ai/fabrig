+++
id = "const-sweep"
part = "world-verbs"
kind = "feature"
needs = ["client-protocol-kit"]
files = ["jig/tests/tasks/tests/**", "jig/tests/tasks/src/**", "jig/testing/jig-client-kit/src/**", "jig/tests/client/tests/**", "jig/testing/jig-client-protocol-kit/src/**", "jig/tests/client-protocol/tests/**", "jig/tests/ops/**", "jig/tests/store/tests/**", "jig/tests/client-wire/**", "jig/tests/client-browser/**", "jig/tests/conformance/**"]
effort = "medium"
minutes = 45
+++

# jig's worlds name a fixture once each

## Goal

Jig D035's rest. Within one world, `const PROJECT: u32 = 1` in several files is one name repeated: client 9, client-protocol 7, ops 6, people 3, tasks 2. `TASK` and `PARTY` go the same way where their values match.

Acceptance:

- In each of those worlds, a `PROJECT`, `TASK` or `PARTY` of one value is declared once, in the world's kit or library, and its files import it. A file with another value keeps its own, and it's named for what it is if the same name now means two things in that world. **`SEED` stays per file:** each names its run. `rg -l -t rust '^\s*(pub(\(crate\))? )?const PROJECT:' jig/tests jig/testing | wc -l` falls to about one per world, 25 at most, and your report gives it, `TASK`'s and `PARTY`'s.
- **The tasks world's verbs:** `claim(10, 10)`, `claim(23, 23)` and `lift(100, 1, ..)` read a named task or hold, with a verb where the two numbers always match. The Measure over `jig/tests/tasks` reads at most six, each left in the report with why.
- `jig-ops-world`'s `ask`: if it's the same as `jig_core_kit`'s for its own world, it's named once in that world.
- `heavy cargo nextest run` on `jig-tasks-world`, `jig-client-world`, `jig-client-protocol-world`, `jig-ops-world` and `jig-store-world` passes, with no assertion changed. Tests' net lines fall, since each sweep drops a declaration: at least 20.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D035-bare-numbers-in-worlds.md`

## Owns and may change

- **Owns:** the files in the header, for these fixtures alone. A `use` line may grow.
- **Temper's tests** aren't yours.
- **An exact-literal `sed`** is safe for `const PROJECT: u32 = 1`. Anything else, read.

## Steps

One world at a time, each a commit, with its test. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each changed package `--all-targets -- -D warnings`. Commit and push.

## Report

The common note's items; the three counts before and after; the Measure over tasks. No new rule: nothing to break by hand.
