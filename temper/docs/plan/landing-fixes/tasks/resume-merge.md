+++
id = "resume-merge"
part = "resume-merge"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain-checkout/src/hold.rs",
  "temper/crates/temper-worker-domain-checkout/src/boundary.rs",
  "temper/crates/temper-worker-domain-checkout/src/tests.rs",
  "temper/crates/temper-engine-domain-forge/src/items.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/early.rs",
  "temper/tests/worker/checkout/src/merges.rs",
  "temper/tests/worker/checkout/tests/suite/merges.rs",
  "temper/docs/design/domain/worker.status.md",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 70
+++

# A producer's start whose branch holds the tip makes no merge

## Goal

The request's item 5, its first half. A change that started early, or lands into a branch whose rules ask no update, gives its next producer `Start::Merge` of the landing tip (`temper-engine-domain-forge/src/items.rs`). When the branch already holds that tip, git answers "Already up to date", and by `git.md` 4.6 and 4.7 the next commit still records both parents. So a resume adds an empty merge, and the head its run reported goes stale. From reading only: reproduce it first.

Acceptance:

- A producer's `Start::Merge` whose base the branch already contains runs no `Op::Merge`. It goes on as the start it was before `items.rs` made it a merge: `Start::Branch` where the branch is the push branch, `Start::Saved` otherwise.
- On the push branch, with nothing new, a finish lands the same head (`Landing::Landed { commit: ours }`) by the guarded push the eligible retry already makes, with no new commit.
- On saved work, the save's commit lands as it is, with no new commit and `merging` `None`, as a `Start::Saved` does: the saved tree is not lost.
- New work after such a start is an ordinary commit, one parent.
- A base not contained merges exactly as today: same ops, two parents, a conflict left in the checkout.
- A `Start::Merge` with no `landing`, a resolution's or a merge job's, asks nothing and always merges. `even_an_unchanged_merge_tree_is_committed_and_a_stale_expected_head_refuses` and the `merges` world's cases hold as they are.

## Inputs

- `temper/docs/design/domain/worker.md`, section 5, **A merge in progress**.
- `temper/docs/design/domain/forge.md`, 8.11, *Early starts*, **The next producer merges the landing tip first**; 8.2, step 1, on the eligible retry.
- `temper/docs/design/protocol/git.md`, 4.6, 4.7.
- `temper/docs/plan/landing-fixes/notes/common.md`.

## Owns and may change

Owns the files in the header.

Reader and neighbour files:

- `temper/crates/temper-engine-domain-forge/src/tests/early.rs`: `an_early_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work` and `a_goal_branch_changes_next_producer_merges_the_landing_tip_into_branch_or_saved_work` assert `item.landing`: it becomes the landing branch.
- `temper/tests/worker/checkout/src/translate.rs` and `temper-fake-git`: call sites only. `IsAncestor` exists for merged starts.
- `temper/tests/engine/domain/tests/suite/forge_routes/{early_start,saved_retry,goal_branch,goal_landing,catch_up,merge_job}.rs` are read-only readers: run them.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs` and `temper-worker-domain/src/translate.rs` carry `landing` through already: read-only. `Access::Writable.landing` is an existing wire field, so this is no protocol change. If a check refuses `landing` on a merge start, that check may change, and say so.

Departs from the implementer's text: delete from `temper/docs/design/domain/worker.status.md` the `Not yet` entry beginning `5, *A merge in progress*`, and from `temper/docs/design/domain/forge.status.md` the entry beginning `8.11, *The next producer merges the landing tip first*`. The `11:` entry stays: it is not this task's.

Options, each pre-approved:

- Rewriting the repository's `start` in `holding.repositories` is the sketch, so `is_saved()` and `landing_branch()` keep working as they read. If something reads the start as it was given, keep a separate marker in `Tips` or the step instead, and say so.
- A second checkout of `ours` on the no-merge path is fine: it is what those paths already do.
- Leave `wire::Start`, `temper-worker-domain`'s `from_saved` and the engine's hearing of saves as they are: `from_saved` is `false` for a merge start today, and stays so. If that loses something a test shows, report it under `## Left`.

## Steps

1. **Reproduce.** In `temper-worker-domain-checkout/src/tests.rs`, beside `merged_head_already_contained_makes_no_merge_or_checkout`, add `a_producers_merge_start_with_its_base_contained_makes_no_merge`, using `merge_spec()` with `landing: Some(..)`. It fails today, the op after `CheckOutMerge`'s `Succeeded` being `Op::Merge`. Then in the checkout world, a `Case::Contained` in `temper/tests/worker/checkout/src/merges.rs`, with `a_resumed_producer_holding_the_tip_pushes_no_merge` in `suite/merges.rs`: on `temper-fake-git`'s branch, the branch's head after a finish with nothing new is the head before it, and `merge_parent` of that head is what it was. Before the fix it shows the empty merge, which proves the request's reading on fake git.
2. **Fix, the engine,** `items.rs`'s `landing` match, for `Delegate::Produce`:

   ```rust
   Start::Merge { .. } => match &push {
       Some(_) => Some(row.base.clone()),
       None => None,
   },
   ```

   `Delegate::Resolve` and `Delegate::Merge` keep `None`. Correct the `landing` comment in `boundary.rs`, "used only by Branch starts on the push target".
3. **Fix, the worker,** `hold.rs`:

   ```rust
   /// A producer's merge start asks whether its branch holds the base (domain/worker.md, 5).
   PrepareStep::CompareBase { repository, ours, base }
   // its op
   Op::IsAncestor { at: place(workspace, nth(holding, repository)), ancestor: base, commit: ours }
   ```

   In `checked_out_merge`, a `Start::Merge` whose spec has `landing` goes to `CompareBase` before any `Tips` is pushed. `IsAncestor { contains: false }` does what that function does today. `contains: true` rewrites the start and continues as the `Fetch` arm does with `ours`: `FetchLanding` for a branch equal to its push, `FetchPush { saved: ours }` otherwise. List the step in every `match` the compiler names, with no `_`. A failed query ends `Failure::Transient`, as `CompareHead`'s does. Update the module header and its transition table.
4. The other tests: saved work lands with `merging` `None`; a base not contained still gives `Op::Merge` and two parents; no `landing`, no `IsAncestor`.
5. Break the rule once (take the `contains: true` arm out, then `items.rs`'s `landing`) and see the tests fail. Restore.
6. fmt, `old_forms.py`, clippy of `temper-worker-domain-checkout`, `temper-engine-domain-forge` and `temper-worker-checkout-world`, then the focused tests of those packages and `temper-worker-domain`, `temper-worker-domain-world`, `temper-worker-protocol-git`, `temper-fake-git-hosted`. This changes what a merge start does, so also run `temper-engine-domain-world`'s tests. Commit and push.

## Report

The implementer's report, plus: whether the empty merge reproduced as read, and what a story saw of the head after a resume.
