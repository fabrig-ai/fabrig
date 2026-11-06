+++
id = "saved-kept"
part = "saved-kept"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-worker-domain/src/workspace.rs",
  "temper/tests/worker/domain/tests/suite/whole.rs",
  "temper/tests/system/src/composed/parked.rs",
  "temper/tests/system/tests/suite/parked.rs",
  "temper/docs/design/domain/worker.status.md",
]
effort = "high"
minutes = 70
+++

# A parked producer's work survives its answer, and a test shows it

## Goal

The request's item 2. A goal's producer resumes from saved work as a `Start::Merge` of that saved branch, which `temper-worker-domain/src/workspace.rs` reports as `from_saved: false`. `temper-service/src/translate.rs` keeps a saved tag only if the save landed or `from_saved` holds. So, by reading, a save that lands nothing drops the saved work, and the next run starts without it. This is a cause from reading only: reproduce it first, and say if it is wrong.

The two ways it should show, after a producer writes, delivers, writes more, asks and parks, so that its undelivered edits are saved:

- **Nothing new on a contained start.** A later run's saved branch already holds the landing tip, so `landing-fixes/resume-merge` makes no merge. It asks again with nothing new: its save is `Unchanged`, the tag is dropped, and the run after it has lost the undelivered edits.
- **A marker left.** A later run starts in conflict and asks with the marker left: its save is `Conflicted`, and the tag is dropped.

Acceptance:

- Both stories below fail before the fix for the reason above, and pass after it.
- A merge start whose branch is its repository's `save` reports `from_saved: true`; every other merge start reports `false`. `notes/contracts.md`, 2.
- Your report's first lines say plainly whether a parked run's edits can be lost, and how: this is what `answer-words` writes down. Name each way you found that is left, a run parking with a marker left (its own edits, by design: `worker.md`, section 12) and a dead worker among them.

## Inputs

- `temper/docs/design/domain/worker.md`, section 5, **Saved work**; 4.2, step 7; section 12, **A park with markers left**.
- `temper/docs/design/domain/forge.md`, 8.11, **The next producer merges the landing tip first**.
- `temper/docs/design/protocol/channel.md`: `SavedRepository`, unchanged.
- `temper/docs/plan/resume-told/notes/common.md`, `temper/docs/plan/resume-told/notes/contracts.md`, 2.

## Owns and may change

Owns the files in the header. `temper/tests/system/src/composed/parked.rs` and `temper/tests/system/tests/suite/parked.rs` are new.

Reader and neighbour files:

- `temper/tests/system/src/composed.rs`: the `mod parked;` line only. Its private helpers are visible from the child module: `until_producer_answer`, `advance_goal_for_producer`, `answer_producer_question`, `assert_producer_file`, `questioning_config`. Leave `questioning_script` and its story as they are.
- `temper/tests/system/tests/suite/main.rs`: the `mod parked;` line only.
- `temper/tests/worker/domain/src/whole.rs`: if a row of `a_save_says_whether_each_repository_started_from_saved_work` needs a commit or branch the fixture lacks, add it there.
- `temper/crates/temper-service/src/translate.rs`, `temper-engine-domain-forge/src/items.rs`, the wire record: read-only. If the fix seems to need one of them to change, stop: that is a question.

Departs from the implementer's text: delete from `temper/docs/design/domain/worker.status.md` the `Not yet` entry beginning `5, *Saved work*`, and nothing else there.

Options, each pre-approved:

- "Committed" edits here means delivered, the one commit the fake LLM's scripted tools can make. A run's own `git commit` needs a shell the scripts lack, and `../protocol/git.md`, 4.7 takes the worktree's tree whatever its head says: if you can add one cheaply in `temper/tests/worker/checkout`, do; otherwise say in your report that it was not scripted.
- Where the world can read the worker's disk at a run's start, assert the edits are there too. If it can't, the delivered head's files are the proof, and say so.
- If a story needs a fixture path the composed world doesn't have, add it in `parked.rs`.
- If the fake git doesn't conflict where both sides add `second.txt`, have the first delivery write it and both sides change it; the undelivered `kept.txt` stays the proof.

## Steps

1. **Reproduce.** In `temper/tests/system/src/composed/parked.rs`, model `World::questioning_producer`:

   ```rust
   //! A parked producer's delivered and saved edits across a moved landing
   //! branch (temper/docs/design/domain/worker.md, section 5, **Saved work**).
   impl World {
       /// A goal's producer delivers one edit, writes two more, and asks;
       /// the independent change moves the goal to `moved_path`.
       pub fn parked_edits(seed: u64, moved_path: &[u8], script: Script) -> Self
       /// Answers each question in turn, then checks the delivered head.
       pub fn continue_parked_edits(&mut self, files: &[(&[u8], &[u8])])
   }
   ```

   Scripted turns, in `temper/tests/system/tests/suite/parked.rs`'s two tests:

   - `a_parked_producers_edits_survive_a_clean_merge_and_a_contained_resume`: `write first.txt`, `deliver`, `write second.txt`, `write kept.txt`, question, `wait`; the goal moves on `goal.txt`; question, `wait`; question, `wait`; `write third.txt`, `finish`. The delivered head holds `first.txt`, `second.txt`, `kept.txt`, `third.txt` and `goal.txt`, and has the first delivery and the goal's tip as ancestors.
   - `a_parked_producers_saved_work_survives_a_save_refused_for_its_markers`: the same first run; the goal moves on `second.txt`, other content; question, `wait` with the marker left; `write second.txt` settled, `finish`. The delivered head holds `first.txt`, `kept.txt` and the settled `second.txt`, and has the goal's tip as an ancestor. The last run's start is a `Start::Merge` on the saved branch.

   Run each and read its failure: `kept.txt` missing is the reproduction. If either passes before the fix, the cause is wrong for that way: say so, keep the test, and look for what did lose a run's edits.
2. **A row below it,** in `a_save_says_whether_each_repository_started_from_saved_work`: a merge start on `saved` is `true`, a merge start on another branch `false`.
3. **Fix,** as `notes/contracts.md`, 2 has it.
4. **Bite:** break the rule once, as your text asks, and record the failure.
5. **A change of meaning runs its readers:** besides your own packages, `heavy cargo nextest run -p temper-engine-domain-world`'s `saved_retry` and `early_start` tests, and `-p temper-service --lib` for `translate`. Fmt, clippy on every package you changed, focused tests of `temper-worker-domain`, `temper-worker-domain-world` and `temper-system-world`, then commit and push.

## Report

As your text says, first the answer to "can a parked run's edits be lost, and how". Any reader you changed.
