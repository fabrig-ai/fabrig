# resume-told: the plan

From `inputs/request.md`, temper/qa's parallel replay at `5d0044254f`. Judged from reading the code on 2026-11-06; no test has run yet, so each task reproduces first. `landing-fixes/resume-merge` is on the trunk, and this plan stands on it.

## Parts and tasks

| Request's item | Part and task | Needs |
|---|---|---|
| 2, a parked run's work | `saved-kept` | |
| 1, a clean merge | `clean-merge` | |
| 1, the conflict case | `conflict-told` | |
| 3, whoever answers | `answer-words` | the other three |

Four tasks, the request's ceiling. Nothing is dropped.

**Critical path:** `saved-kept` or `clean-merge`, about 70 implementer-minutes, then `answer-words`, about 35.

## What was decided

- **1: commit a clean merge before the run starts; don't tell the run.** Less goes wrong. Telling a run of a clean merge needs a new field on smith's start, a protocol change and so a question first, because `Directory.conflicts` is all that crosses and it is empty. It would also leave dozens of staged files to misread, or to remove: the commit takes the tree as it stands, so a run obeying "clean up" would land a two-parent merge without other tasks' work. A commit first leaves nothing to clean. The steps exist already for a base with heads merged in (`CommitHead`, `CheckOutHead`). The draft agrees (`temper/docs/design/draft/reference-code-production.md`: "merges g's tip into it first; a conflict is left").
- **`temper/docs/design/protocol/git.md`, 4.6 barely moves:** `Merge` is the same operation, and only a bullet says who commits it. `worker.md`, section 5 and `forge.md`, 8.11 move.
- **`tips.start` stays the commit merged into.** So a save with nothing new saves the merge and lands, as today, and the saved tag's hearing doesn't change for the clean case.
- **The conflict case is told in smith's system text,** which already names the files: no protocol change, and a resumed run renders it too.
- **2: they can be lost, by reading.** A goal's producer resumes from saved work as a `Start::Merge`, reported `from_saved: false`, so a save that lands nothing drops the tag. `resume-merge` opened the contained way: two questions in a row with nothing new between. A marker left at a park is the older way. The fix is one match arm, with no wire or stored change, since `save` is already on the item.
- **Not fixed: a run that parks with a conflict marker left keeps none of that run's edits.** Keeping them needs the conflict list to outlive the hold, stored, or a saved tree's markers could be delivered. It is `worker.md`, section 12's open question, and the run is told to settle before it asks.
- **Left as it is:** a resolution's merge stays in progress even if clean, as `git.md`, 4.7 and its tests have it; its brief names the conflict. The request is a producer's.

## Overlaps with drafts beside this one

- `answer-words` touches `temper/crates/temper-service/src/tests/stock.rs`, which `model-catalogue-1/stock-seven` and `configuring-words-2/stock-words` change, and `temper/docs/help/change.md`, under `model-check/trial-close`'s `temper/docs/help/**`. It goes last and adds small hunks.
- `saved-kept` and `clean-merge` both delete an entry of `temper/docs/design/domain/worker.status.md`; the entries sit apart, so a merge finds them apart.
- The early tasks touch no other draft's files.
