+++
id = "conformance-causes"
part = "conformance"
kind = "feature"
needs = ["cuts-fork"]
files = [
  "jig/testing/jig-conformance/**",
  "jig/testing/jig-core-kit/src/conformance.rs",
  "temper/tests/conformance/**",
  "temper/docs/plan/limits-worlds/figures/conformance-causes.md",
]
effort = "high"
minutes = 120
+++

# What a crash sweep's replays still cost

## Goal

With `cuts-fork`, a sweep no longer repeats its prefix, but each child still restarts and finishes its story. Profile what's left, and fix what the profile puts in the harness, the referee and temper's adapter, largest first, until the next one's gain is under about 2% of a story. The referee keeps every observation it makes today, and the fourteen sweeps keep every commit.

Acceptance:

- `temper/docs/plan/limits-worlds/figures/conformance-causes.md` holds, in `notes/measuring.md`'s form, instructions before and after for `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` and `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit`, with the largest functions by self and by inclusive instructions over the parent and its children, and ends with what you stopped at.
- Each fix has its own commit saying why, and what the referee sees is unchanged: a mutated observation it caught before is still caught, by the tests that hold that (`jig/tests/conformance/tests/suite/referee.rs`, `jig/testing/jig-conformance/src/referee/tests.rs`).
- `heavy cargo nextest run -p temper-conformance-world --profile fuzzy` passes all fifteen, and the focused and fuzzy suites of `jig-conformance-world` and `jig-ops-world`, and the focused suite of `jig-core-world`, pass.

## Inputs

- `temper/docs/plan/limits-worlds/notes/common.md`, `temper/docs/plan/limits-worlds/notes/measuring.md`.
- `temper/docs/plan/limits-worlds/figures/cuts-fork.md`: your *before*.
- `jig/docs/design/domain/testing.md`, 5, 6 and 10 ("A commit's rows for the core's observer").
- `jig/docs/development/debt/D034-world-replay-cost.md`: what's been measured here, and what paying it takes.
- `temper/tests/conformance/src/lib.rs`, `observed`; `temper/tests/conformance/src/observations.rs`; `jig/testing/jig-conformance/src/core.rs`, `durable_at`.

## Owns and may change

Owns the files in the header. Also, each only as named:

- **jig's adapters, `jig/tests/conformance/src/**` and `jig/tests/ops/src/conformance/**`:** what a change to the harness's or the kit's interface forces, and no more.
- **Design, a departure from the implementer's text:** if you pay D034, change `jig/docs/design/domain/testing.md`, 10's "A commit's rows" entry and `jig/docs/design/domain/testing.status.md`'s `debt-25` entry to say so, and delete `jig/docs/development/debt/D034-world-replay-cost.md`, as that directory's `README.md` has a paid item go, fixing the citations to it. No other design text.

Where to look first, and what you decide. The profile decides whether any of these is the cause:

- **The whole store is projected at each commit:** `Temper::observed` runs `observations::snapshot(store)` and walks every row, and `durable_at` rebuilds its snapshot. D034 measured about 10% of jig's fuzzy conformance suite here, and says what a row feed needs: keyed snapshot accounting and commit-level observations, checked against the whole projection. Default: if this is the largest cause and over a tenth of the story, pay it as D034 says, with the whole-store equivalence check as its focused test. Otherwise list it with its share.
- **Temper's own walk in `observed`** over every row for `Result` and `Words` evidence may be cheaper to fix alone than D034: fix it first if the profile shows it apart.
- **A view built for every inbound event** (`Observer::inbound`), and `{event:?}` strings kept by a neighbour.
- **What a cold start repeats per cut:** `A::build`, `A::start` and the restore's pages run in every child, and the sweep's check is the restart itself: **that stays as it is.** Setup the restart doesn't need is fair.
- **A referee check that scans:** a lookup by key does the same work, with the same verdict.

Options, each decided:

- **A check of the referee's would change:** don't, whatever it saves. List it.
- **The cause is the engine's, the store's or a fake's:** `## Left`, for `engine-causes`' list and `close`.

## Steps

1. Profile the two sweeps on your base, summing the parent's and children's files, as `notes/measuring.md` says.
2. Fix the largest, re-profile, repeat.
3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`, and for temper likewise; `heavy cargo clippy` naming every package you changed, `--all-targets -- -D warnings`; the runs in the acceptance; `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

Instructions and CPU seconds before and after, each cause and what it saved, what you left and whose it is, whether D034 is paid, and your implementation and check minutes apart.

## Rebase (2026-11-07, step merge, foreman limits-worlds/conformance-causes/foreman-1)

reconcile main’s lazy observer views with the branch’s keyed projection
The merge conflicts with `fd51f31222` (“Build core observer views only for row readers”) and design changes in `e9d0ada6ed` and `c91d31e7fa`.
Resolve all six conflicting paths, preserving both changes:
- `jig/testing/jig-conformance/src/core.rs`: retain main’s lazy `inbound` and `submitted` interfaces. Call `self.projection.submitted(&pending)` for every submitted commit before main’s no-erasure early return. Preserve cold invalidation and keyed snapshots in `durable_at`, together with main’s combined evidence/accepted-decision pass.
- `jig/testing/jig-core-kit/src/conformance.rs`: retain main’s closure-based views, translated newest commit, named revisions and view tests. In its extracted `core_write`, translate `CopySettingsRevision` and `DropTranscript` to `Write::Refresh`, as the branch requires.
- `jig/testing/jig-conformance/src/core/snapshot.rs`: retain main’s combined naming/row pass and the branch’s `resolve` separation. Keep cross-row tracking and acceptance resolution out of per-row projection; avoid duplicate acceptance collection.
- `jig/docs/design/domain/testing.md`: preserve main’s reader-only fresh-view contract and coverage/budget additions, plus the branch’s forked-sweep text. Resolve section 10’s “A commit’s rows” entry to the implemented keyed projection, retaining cold/current-row and bulk-rebuild guarantees.
- `jig/docs/design/domain/testing.status.md`: remove the paid `debt-25` gap while retaining main’s unrelated gaps, including section 9’s budget verification.
- `jig/docs/development/debt/D034-world-replay-cost.md`: keep the branch’s deletion; main’s modification describes the intermediate whole-projection implementation. Fix any surviving citations introduced on main.
Review the automatically merged `core/evidence.rs` for both the `commit` helper and `Write::Refresh`, and preserve main’s matching intent and ops-adapter interfaces.
Validate keyed/whole-projection equivalence and lazy-view tests, then the spec’s referee checks and focused/fuzzy acceptance suites through heavy. Run formatting, affected-package clippy and citation checks.
