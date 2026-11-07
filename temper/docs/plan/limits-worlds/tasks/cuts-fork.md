+++
id = "cuts-fork"
part = "cuts"
kind = "feature"
needs = []
files = [
  "skein/testing/skein-fork/**",
  "jig/testing/jig-conformance/src/harness.rs",
  "jig/testing/jig-conformance/src/lib.rs",
  "jig/testing/jig-conformance/Cargo.toml",
  "temper/tests/conformance/tests/suite/fuzzy/stories.rs",
  "temper/tests/conformance/tests/suite/fuzzy/shared.rs",
  "temper/tests/conformance/tests/suite/stories.rs",
  "temper/tests/conformance/README.md",
  "temper/docs/plan/limits-worlds/figures/cuts-fork.md",
]
effort = "high"
minutes = 120
+++

# A crash at every live commit holds its prefix once

## Goal

`temper-conformance-world`'s `fuzzy::stories` and `fuzzy::shared` run a whole fresh deployment for each of their live commits, so a cut repeats every commit before it. Fork the run at each commit instead: one baseline, each cut's child going on from there through the same cold restart. **Every commit stays checked, with no sampling,** by the same referee and the same story checks. Most of 44 CPU s of the fuzzy suite is here.

Acceptance:

- `skein/testing/skein-fork`, `Cut::EachCommitFrom`, `Outcome::forked` and `cut_passed` exist as `notes/contracts.md` has them.
- All fourteen sweeps in `fuzzy/stories.rs` and `fuzzy/shared.rs` use them, under their test names as they stand, each asserting that `forked` is every commit from the first live to the last, and `a_shrinking_pool_is_not_applicable` stays as it is.
- A failing cut fails its test, naming the commit. You've seen that happen once by breaking a story's check by hand, and the report says what you broke and what it printed.
- A new focused test in `temper/tests/conformance/tests/suite/stories.rs`, `a_forked_cut_ends_as_a_fresh_cut_does`, on the cheapest native story by commits (`Story::IssueCreation`, unless the numbers say otherwise): for each live commit it runs the fresh `Cut::AfterCommit(cut)` first and keeps its `Outcome`, then runs `Cut::EachCommitFrom`, with each child comparing its whole `Outcome` to its cut's before it ends. It stays within the focused guide of 1 s (`skein/docs/foundation/testing-strategy.md`, 8).
- Focused tests of the kit: a child's `passed` reaches the parent as `Exit::Code(0)`, a child's panic as `Exit::Code(101)`, and a child's writes don't reach the parent.
- `temper/docs/plan/limits-worlds/figures/cuts-fork.md` holds instructions before and after for `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` and `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit`, in `notes/measuring.md`'s form, each after-figure the sum over its children's files.
- `heavy cargo nextest run -p temper-conformance-world --profile fuzzy` passes all fifteen, and the focused suites of `temper-conformance-world`, `jig-conformance-world` and `jig-ops-world` pass.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `notes/measuring.md`, and `notes/contracts.md`, "A sweep's fork": the sketch binds.
- `jig/docs/design/domain/testing.md`, 5, "Crashing at every commit".
- `temper/docs/design/domain/testing.md`, 5.2, on the conformance world.
- `skein/docs/design/testing.md`, 6, its line for `testing/skein-fork`.
- `tools/lints/design.md`, 2.4 and 3; `skein/docs/foundation/programming-model.md`, 2.1, on the module allowed `unsafe`.
- `temper/tests/conformance/README.md`, its last two paragraphs.

## Owns and may change

Owns the files in the header. Also, each only as named:

- **Root `Cargo.toml` and `Cargo.lock`:** the new member, its `[workspace.dependencies]` line, and `libc` for it.
- **`skein/testing/skein-fork`:** takes `ring` with `python3 -I -B tools/lints/lints.py --set ring skein/testing/skein-fork`, which also gives it its `clippy.toml`. If the tool wants the package's path otherwise, follow its `--help`.
- **`tools/lints/design.md`, 3:** add `skein-fork` to `ring`'s bullet, since it's the first kit under `testing/` there, and say why in that one line.
- **`jig/testing/jig-conformance/Cargo.toml`:** its `skein-fork` dependency. A world names no fork: `temper/tests/conformance/Cargo.toml` gains nothing.
- **Status entries, a departure from the implementer's text:** remove the **Not yet** entries this plan wrote for this work: `jig/docs/design/domain/testing.status.md` at 5 (`Cut::EachCommitFrom`), `temper/docs/design/domain/testing.status.md` at 5.2 and `skein/docs/design/testing.status.md` at 6 (`testing/skein-fork`). No other design text.
- **Readers of `Cut` and `Outcome`,** `jig/tests/conformance/**` and `jig/tests/ops/**`: compiler-forced arms only. `Harness::from_snapshot` gains the field. jig's own sweeps stay as they are.
- **`temper/tests/conformance/README.md`:** say that the fuzzy sweep forks each cut from one baseline. Its counts stay.

Options, each decided:

- **The prefix is small after all.** Baseline Ir, times the number of cuts, over two estimates what's shared. If that's under a fifth of the test's Ir, don't build the fork: write the figures, name what the cost is instead, and report `Done:` with that. This is the request's *find the cause first*.
- **A story's run differs between parent and child,** a commit it doesn't reach, say, or an `Outcome` that doesn't match its fresh cut: that's a finding about determinism, not a thing to paper over. Read it. If it's a fake holding state outside the world (a global, a time read), fix that when small and say so; otherwise `Question:` with the two outcomes.
- **The referee's heap check moves** with the parent's `forked` list: the list is a few hundred bytes against a ceiling three times a story's peak, so it shouldn't. If it does, read it before anything else.
- **`libtest`'s main thread:** the child never returns to it. That's why `passed` and the panic path use `_exit`; don't add a way back.
- **macOS:** `fork` and `waitpid` from `libc` build there as they do on Linux. Don't fence the kit by platform.

## Steps

1. Profile the two sweeps on your base, as `notes/measuring.md` says, once each: simulated time, no raised patience.
2. Build the kit with its tests, then `Cut::EachCommitFrom`, then the sweeps and the focused comparison.
3. Break a story's check by hand and watch the sweep name the cut; restore it.
4. Profile the two again: callgrind follows a fork with no flag, and each child writes its own `%p` file.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`, and for temper and skein likewise; `heavy cargo clippy -p skein-fork -p jig-conformance -p temper-conformance-world --all-targets -- -D warnings`; the runs in the acceptance; `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

Instructions and CPU seconds before and after for both, the fuzzy suite's whole `temper-conformance-world` CPU before and after if you have them, what you broke by hand and what it printed, what the fork itself costs (per cut, from the profile), and your implementation and check minutes apart.

## Answers

2026-11-07 (temper/tiers). The first: cuts-fork is accepted with the guide unmet and named in its report, and conformance-causes owns getting a_forked_cut_ends_as_a_fresh_cut_does within 1 s. Its first look: whether the comparison at the first, a middle and the last live commit shows what comparing at every commit does, since the fuzzy sweeps run every commit through the fork anyway; if so, that's the fix, and otherwise the profile's cause. Nothing raised, nothing sampled in the sweeps.
