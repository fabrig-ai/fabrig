+++
id = "clean-merge"
part = "clean-merge"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain-checkout/src/hold.rs",
  "temper/crates/temper-worker-domain-checkout/src/tests.rs",
  "temper/tests/worker/checkout/src/merges.rs",
  "temper/tests/worker/checkout/tests/suite/merges.rs",
  "temper/docs/design/domain/worker.status.md",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 70
+++

# A producer's clean merge is committed before its run starts

## Goal

The request's item 1. A producer resumed after its landing branch moved finds `MERGE_HEAD` and dozens of other tasks' staged files, reads them as contamination, and may be told to remove them, which would land a merge without them. Commit a clean merge before the run starts, so it sees a clean tree with its own work and one new commit. A conflict stays a merge in progress; `conflict-told` tells that run.

Why this and not telling the run: telling it needs a new field on smith's start, a protocol change, and leaves the staged files there to misread or remove. `CommitHead` and `CheckOutHead` already do this for a base with heads merged in.

Acceptance, `notes/contracts.md`, 1:

- A producer's merge start (`landing` is `Some`), base not contained, merging cleanly: `Op::Merge`, `Op::Commit` with both parents and the title `Merge <base>`, `Op::CheckOut` of that commit with `clean: true`, then ready with no conflicts. Nothing moves on the forge.
- With nothing new, a finish pushes that merge, and adds no commit. A repeat is the same landing.
- With nothing new, a save saves that merge, landing `Landed`.
- New work is an ordinary commit on the merge, one parent.
- No merge state is left in the checkout before the run starts.
- A producer's conflict leaves the merge in progress exactly as today. A merge start with no `landing`, a resolution's or a merge job's, runs exactly today's operations. `even_an_unchanged_merge_tree_is_committed_and_a_stale_expected_head_refuses`, `a_producers_uncontained_base_still_leaves_a_conflicted_two_parent_merge` and the `merges` world's other cases hold as they are.

## Inputs

- `temper/docs/design/domain/worker.md`, section 5, **A merge in progress**, **A base with heads merged in**.
- `temper/docs/design/domain/forge.md`, 8.11, **The next producer merges the landing tip first**.
- `temper/docs/design/protocol/git.md`, 4.5, 4.6, 4.7. No operation changes.
- `temper/docs/plan/resume-told/notes/common.md`, `temper/docs/plan/resume-told/notes/contracts.md`, 1.

## Owns and may change

Owns the files in the header.

Reader and neighbour files:

- `temper/tests/worker/checkout/src/translate.rs`, `temper-fake-git`: call sites only. The commit and checkout already run for merged heads.
- `temper/tests/worker/checkout/tests/suite/memory.rs`: read-only. If a new step passes its ceiling, stop: that's a question, with the measurements.
- `temper/tests/worker/domain/tests/suite/runtime.rs`, `temper/tests/worker/domain/tests/suite/merge.rs`, `temper/tests/system/tests/suite/composed.rs`, `temper/tests/engine/domain/tests/suite/forge_routes/{early_start,saved_retry,goal_branch,goal_landing,catch_up,merge_job}.rs`: read-only readers: run them. If one pins a producer's clean merge left in progress, change its expectation only, and name it.

Departs from the implementer's text: delete from `temper/docs/design/domain/worker.status.md` the `Not yet` entry beginning `5, *A merge in progress*`, and from `temper/docs/design/domain/forge.status.md` the entry beginning `8.11, *The next producer merges the landing tip first*`. Nothing else there.

Options, each pre-approved:

- `Done::Conflicted` from the new commit can't come after a clean merge: `Next::Failed(Failure::Transient)`, not `Failure::Conflicted`, which the engine reads as an early start's conflict.
- Prove no merge state as the world can: a further `Op::Merge` the fake git would refuse in a merge in progress (`worker.status.md`, 5, has the transcript), or the fake's own reading. Say which.
- New steps' names below are a sketch; a shared step with `CommitHead` is fine if it stays plain.

## Steps

1. **Reproduce,** in `temper-worker-domain-checkout/src/tests.rs`, beside `a_producers_uncontained_base_still_leaves_a_conflicted_two_parent_merge`, with `comparing_producer_base(false)`:

   ```rust
   #[test]
   fn a_producers_clean_merge_is_committed_before_it_is_ready() {
       // IsAncestor { contains: false } -> Op::Merge
       // Done::Merged -> Op::Commit { parent: ours, merging: Some(base), title: Merge <hex> }
       // Done::Committed { m } -> Op::CheckOut { commit: m, clean: true }
       // Done::Succeeded -> Prepared::Ready, no conflicts
       // Push -> Op::Commit { parent: m, merging: None }; Done::Unchanged -> Op::Push { commit: m }
   }
   ```

   It fails today: `Done::Merged` is ready at once. Then, in `temper/tests/worker/checkout/src/merges.rs`, a `Case::Resumed`: `Case::Clean`'s fixture with `landing` set, and `a_resumed_producers_clean_merge_is_committed_before_its_run` in `suite/merges.rs`: after the prepare the trace holds the commit and the checkout, no remote moved and no merge state is left; then a save with nothing new lands the merge on `saved`, a push with nothing new lands it on `topic`, and new work on it is one parent.
2. **Fix,** `hold.rs`, `prepared_merge`:

   ```rust
   Done::Merged => match head {
       Some(head) => Next::Step(PrepareStep::CommitHead { repository, head }),
       None if nth(holding, repository).landing.is_some() => {
           Next::Step(PrepareStep::CommitMerge { repository, theirs })
       }
       None => after_merged(repository, last),
   },
   ```

   `CommitMerge`, built like `commit_head`, ends with `tips.head = commit; tips.merging = None`, then `CheckOutMerged { repository }`, built like `CheckOutHead`, then `after_merged`. `tips.start` stays `ours`: not `next_head`'s `tips.start = tips.head`. Update the module's words and its transition table.
3. **Bite:** break the rule once, as your text asks, and record the failure.
4. **A change of meaning runs its readers:** besides your own packages, run `temper-worker-domain-world`, `temper-system-world` and `temper-engine-domain-world`'s `forge_routes` tests. Fmt, clippy on every package you changed, focused tests of `temper-worker-domain-checkout`, `temper-worker-domain` and `temper-worker-checkout-world`, then commit and push.

## Report

As your text says. Whether any reader pinned a clean merge in progress, and whether its change was only an expectation.
