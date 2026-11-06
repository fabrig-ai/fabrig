# landing-fixes: the plan

From `inputs/request.md`, what a real plan's landing met at `7913a87678`. Ranked, with a ceiling of three or four tasks. Everything below was judged from reading the code on 2026-11-06. No test has run yet, so each task reproduces first.

## Parts and tasks

| Request's item | Part | Task | Needs |
|---|---|---|---|
| 1, CI before the update | `update-first` | `update-first` | |
| 2, the receipt and the detached checkout | `delivery-words` | `delivery-words` | |
| 3, an uncommitted merge | none | none: fixed since the deployment's commit | |
| 4, a question to its own task | `question-target` | `question-target` | |
| 5, a resume's empty merge | `resume-merge` | `resume-merge` | |
| 5, a branch left after its goal | none | none: not by design, recorded in `forge.status.md` | |
| 6, a repair's brief | none | dropped | |
| 7, a refused worker | none | dropped | |
| 8, a flake | none | dropped | |

Four tasks, the request's ceiling. No task needs another.

**Critical path:** the longest alone, `update-first`, about 75 implementer-minutes.

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
- **5. A resume's empty merge.** From reading, the cause is that a change that started early, or lands into a branch whose rules ask no update, gives its next producer a merge in progress of the landing tip (8.11). Git answers "Already up to date" when the branch already holds that tip, and by `git.md` 4.6 and 4.7 the next commit still records both parents. So the push makes an empty merge, and the head a run reported is replaced. `resume-merge` asks first, with the `IsAncestor` op a merged start already uses. It separates a producer's start from a resolution's by the `landing` field already on the wire, so nothing new crosses a boundary, and a resolution or merge job still always merges. A merge that does take in a moved tip still makes a new head, so a report written before it can name an older head. That's by design, and `agent.md`, section 6 covers it.
- **5. A branch left after its goal isn't by design.** `forge.md`, section 11 says a task's saved work is deleted as its task closes. From reading, `temper/<root>/s<task>` is never a hold, and a close releases holds alone (`continue_release`), so nothing deletes it. The request asks only whether it's by design, and a fix would be a fifth task, so it goes in `forge.status.md` as `Not yet`. I didn't run this, and I can't confirm `s10` and `s14` were saved-work branches of closed tasks beyond their names.
- **Dropped from the bottom, as the request says: 6, 7 and 8.** These weren't traced to a cause and no design section was written for them. What I did see, for whoever picks them up:
  - **6:** a resolution's brief opens `Resolve the conflict from change head` (`temper-engine-domain-forge/src/brief.rs`, chosen by `PullMode`), and a CI repair's `Repair the failed check at change head` (`brief_ci.rs`). So the words do exist, and which path gave conflict words with no cause is unknown.
  - **7:** `temper/crates/temper-service/src/route.rs:324` does send the fixed `not admitted`, as the request says.
  - **8:** not read beyond the test itself, which doesn't take a port directly.

## Overlaps not avoided

- `update-first` and `question-target` each add a `mod` line to `temper/tests/engine/domain/tests/suite/forge_routes.rs`, and `sign-in-new-tab/return-goes` changes `temper/tests/engine/domain/**`. Their stories are in new files to keep hunks apart.
- `question-target` touches `temper/crates/temper-service/src/tests/stock.rs`, which `model-catalogue-1/stock-seven` and `model-check/trial-charter` change. It adds one test, beside the others.
- `update-first`'s readers include `temper/crates/temper/tests/suite/forge_land.rs`, which `stock-seven` changes. They're read-only unless a story's order has to move.
- `update-first`, `question-target` and `resume-merge` all write `temper/docs/design/domain/forge.status.md`, each deleting its own entries only. The entries are on separate lines.
- No task touches `temper-client-*`.
