# hearth-speed: the plan

hearth's focused suite under its 30 CPU s, aiming for 27 on the queue's figure, with every assertion and case kept. The design is `hearth/docs/design/domain/testing.md`, 9.1, which doesn't change: only its status does. The standing rules, the aim, the base commit and the method are in `notes/common.md`.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `before` | `figures-before` | nothing |
| `profiles` | `dev-profiles` | nothing |
| `git-setup` | `git-setup` | nothing |
| `repeats` | `repeats` | `before`, `profiles`, `git-setup` |
| `after` | `figures-after` | `repeats` |

The critical path is `dev-profiles`, `repeats` and `figures-after`: about 150 implementer-minutes, and far less if `repeats` finds the aim met. Three tasks start at once.

## What was decided

- **No machine here can profile,** because a call builds nothing, so the first task does. `dev-profiles` and `git-setup` don't wait for it: each measures itself, on its own slot, by experiment, which says more about a profile line than a profile does.
- **Profile lines come first, and they're the likely result.** Every hearth crate builds at `opt-level` 0, and the tree suite is pure computation: 20,000-iteration loops and generated layers over step code. No test is over 1.3 CPU s, so the cost is spread, and a line reaches all of it. Only hearth's crates get lines, and `sha1`, so no other tree builds any differently but for `sha1` itself. `opt-level` alone changes: `overflow-checks` and debug assertions stay on.
- **The aim is a ratio.** No task can run the queue, and a builder reads higher than the queue (41.7 against 31.7, for a reason nobody has found). So a task aims at 85% of its own machine's figure at the base commit, which is 27 / 31.7. That leaves room for a builder's figure to stay over 30 while the queue's is under, and `figures-after` says which it is.
- **Every "before" is at one pinned commit,** `483dfed6aa`, the trunk's base, so that it doesn't depend on which task merged first. `figures-after` measures both sides itself, because the request asks for one machine and a slot can't choose its machine.
- **`repeats` is a task whether or not it's needed.** The plan can't see `dev-profiles`' result, since a cut doesn't wait on a run. So `repeats` starts by measuring, and if the aim is met it records that and changes no code. That's the request's "stop once the aim is met".
- **`git-setup` is small and certain.** Measured here without building, `git` 2.47.3 in a scratch directory: 8.0 ms per repository of `THIN`'s size, 5.5 without the template and automatic maintenance. Repository setup is about 8 of the real-git binary's 46 ms per test (two different machines, so roughly), so most of that binary's cost isn't setup, and `hearth` itself runs at `opt-level` 0 in it.
- **The in-process repository writer is the last option of `repeats`.** It saves up to about 5 ms a test, but it's the one change here with real risk to what a fixture means, so it needs its own measurement and its own test.
- **A checked-in template repository isn't an option.** Many fixtures build different file sets, git won't track a nested `.git`, and `next` still needs a real one.
- **What a world lays per seed is split, not cut.** `Shared::build` takes the seed for `Machine::new` and `Universe::draw`, and only the fake repository is the same each time. A search finds nothing `Machine::new` allocates up front by its `Sizes`. So there's no obvious hidden cost in the world's setup, and `repeats` goes by the profile.

## The survey

Readers of what changes, from the trunk at this cut:

- **The root `[profile.dev.package.*]` tables:** nothing under `tools/`, `tests/`, `.config/` or `scripts/` reads them.
- **`sha1`:** under `hearth/`, only `hearth-fake-git` names it (`hearth/testing/hearth-fake-git/src/objects.rs`).
- **`Repository::make_on`:** only `make`, and `a_scratch_repository_without_git_on_the_path_fails_by_name_not_by_skip`.
- **`git init` in fixtures:** `hearth-test-repository/src/lib.rs` and `hearth/tests/builder/src/fixtures.rs`'s `work_tree`. Nothing under `hearth/tests/`, `hearth/testing/` or `hearth/crates/hearth/` reads `hooks`, `info/exclude`, `.git/description`, `gc.auto` or `maintenance`.
- **The status lines:** `hearth/docs/design/domain/testing.status.md`'s **Departs** on 9.1, and the clause in `hearth/docs/design/STATUS.md`. Neither is in `.config/gate/read-by-tests.txt`.
- **`tests/boundary/tests/outward.rs`** skips `*/docs/plan/<draft>/`, so a machine's name may stand in `figures.md`, but not in a status file.

## Overlaps

No draft beside this one lists a file this plan owns. The nearest is `debt-26/hearth-spot`, on `hearth-protocol-builder/src/lib.rs`, `keep.rs` and `read.rs`: a profile line builds that crate differently but doesn't edit it, and `repeats`' spec keeps any edit there to its own hunk. The root `Cargo.toml` is listed by no draft; a draft that adds a crate would edit its members, far from the profile tables.

## What this plan can't do

The request asks for the gate's record after landing. Nothing in a plan runs after it lands, so no task can write that figure. `figures-after` says so in `figures.md`. If the method's figure stays at 30 or more on a builder while the queue's record comes in under 30, the **Departs** on 9.1 stays until someone edits it after landing. The report names this.
