# landing-fixes: the plan

From `inputs/request.md`, what a real plan's landing met at `7913a87678`. Ranked, with a ceiling of three or four tasks. Everything below was judged from reading the code on 2026-11-06. No test has run yet, so each task reproduces first.

## Parts and tasks

| Request's item | Part | Task | Needs |
|---|---|---|---|
| 1, CI before the update | `update-first` | `update-first` | |
| 2, the receipt and the detached checkout | `delivery-words` | `delivery-words` | |
| 3, an uncommitted merge | none | none: fixed since the deployment's commit | |
| 4, a question to its own task | `question-target` | `question-target` | |
| 5, a resume's empty merge | to come in the next batch | | |
| 5, a branch left after its goal | to come: read, then say | | |
| 6, 7, 8 | to be settled with the next batch | | |

**Critical path:** no task needs another, so the longest alone: `update-first`, about 75 implementer-minutes.

## What was decided

- **1. Yes, the update can come first.** `checking` waits on CI with no look at `contains_base`, which each step already reads afresh (5.3). `Checking -> Updating -> Checking` already exists, for a red head behind its base (`repair_due`), so this only triggers it earlier: no new state, field or read.
  - **A head already green:** nothing changes. It goes on to its gates and its turn, and updates there. Under rules asking up to date a behind head can't land as it is, so its green can't land it, but it does carry it through gating, and nothing is gained by throwing it away early.
  - **A CI result still good:** a conclusion read at the head is never overridden, and CI not read at the head is treated as no conclusion, so an unread green isn't lost. Pending isn't a result.
  - **If the rule doesn't ask:** the root already gives `contains_base` `Passed`, so nothing changes.
  - **What it doesn't save:** temper has no effect that stops a run the forge has started. If the forge's CI starts on the push or the pull request's opening, the old head's run may still spend its machine time unless the repository's workflow cancels superseded runs. The pull request has to exist before the forge can update it, so the update can't precede that either. What's saved is the change's wait, which is what the run reported. So 5:26 is saved of the change's time, not necessarily of the CI's.
  - **The bound:** an early update counts against `forge.change_policy.updates` (3 on the stock) and is never its last, so it makes no hold of its own. Where the base keeps moving during one change, `Hold::Updates` can still come one update sooner. Not counting it would make "bounds update attempts" untrue of the bound, and raising the figure is the user's. temper/qa's next landing plan is where this shows.
- **2. The receipt is a bare head.** A run reads `delivered` and `receipt directory=0 text="<64 hex>"`. Its words now say `pushed <head> to branch <branch>; your checkout stays detached, as expected`. They go in temper's receipt and not smith's `delivered` line, since what delivering means is the host's (smith's `domain/run.md`, 8.2). The evidence's format is unchanged, so this is no stored-format or protocol change: the receipt is opaque bytes to smith and jig, and the engine hears the typed commit. The charter half landed with `parallel-faults/charter-words`, after this deployment's commit; the receipt reaches a project's own charter too.
- **3. Already fixed: no task.** The code was wrong and `hold.rs`'s header right. `parallel-faults/merged-start` (`f6b4c408a7`, 2026-11-06 10:11 UTC) checks out each merge it commits, which is after the deployment's `7913a87678` (01:33 UTC). Its tests pin it: `merged_heads_commit_each_merge_in_order`, and `hosted_git_refuses_a_second_merge_until_a_checkout_clears_the_first` on a fake that refuses as git does. temper/qa's next landing plan is the proof.
- **4. The bare `; ` isn't a lost reason.** `authority_findings` writes `; ` after every finding, so every authority refusal ends that way. It's a separator bug, and the reason, `task reference`, was there all along. Which target gets said in both places, because a project's own charter gets no stock words: `coding.txt` for a coding run, and the refusal for any run.

## Overlaps not avoided

- `update-first` and `question-target` each add a `mod` line to `temper/tests/engine/domain/tests/suite/forge_routes.rs`, and `sign-in-new-tab/return-goes` changes `temper/tests/engine/domain/**`. Their stories are in new files to keep hunks apart.
- `question-target` touches `temper/crates/temper-service/src/tests/stock.rs`, which `model-catalogue-1/stock-seven` and `model-check/trial-charter` change. It adds one test, beside the others.
- `update-first`'s readers include `temper/crates/temper/tests/suite/forge_land.rs`, which `stock-seven` changes. They're read-only unless a story's order has to move.
- No task touches `temper-client-*`.
