# resume-told: the run's binding calls

Each task builds to its own entry. Entries 3 and 4 are exact words: other tasks' tests and pages agree with them.

## 1. A producer's clean merge (`temper/docs/design/domain/worker.md`, section 5, **A merge in progress**)

Only a `Start::Merge` with a `landing`, whose base its branch does not contain, whose `Op::Merge` ends `Done::Merged`:

- `Op::Commit { parent: ours, merging: Some(base), title: merge_title(base), body: empty, identity }`, then `Op::CheckOut { commit: that, clean: true }`, then the next repository or ready.
- After it: `Tips { start: ours, head: that, pushed: Some(ours), merging: None, .. }`. `start` stays `ours`: a save with nothing new still saves the merge commit, landing `Landed`, as today.
- A later push with nothing new: `Op::Commit { parent: that, merging: None }`, `Unchanged`, then pushes `that`.
- `Prepared::Ready { conflicts: [] }`, nothing on the forge moved.

A `Start::Merge` with no `landing` (a resolution, a merge job) and a producer's `Done::Conflicted` stay as they are.

## 2. `from_saved` (`temper/docs/design/domain/worker.md`, section 5, **Saved work**)

In `temper-worker-domain/src/workspace.rs`, where `from_saved` is computed:

```rust
wire::Start::Saved { .. } => true,
wire::Start::Merge { branch, .. } => repository.save.as_deref() == Some(&**branch),
wire::Start::Base { .. } | wire::Start::Branch { .. } | wire::Start::Commit { .. } | wire::Start::Merged { .. } => false,
```

The wire record, `temper-service/src/translate.rs`'s `kept` and the engine's hearing stay as they are.

## 3. What a run with conflicts is told (`smith/docs/design/domain/run.md`, 8.3)

In `render_checkout`, once, after the directories' lines, when any directory has `conflicts`, these words and a newline:

> A directory with initial merge conflicts holds a merge in progress: `HEAD` is your work so far and `MERGE_HEAD` the tip the host merged into it. Every other staged file is what that merge brought in, not stray work: leave those files in place and restore nothing. Settle the files named; a delivery commits the whole tree as the merge, with both parents.

The fragment ``initial merge conflicts: `path` `` stays as it is.

## 4. What the charters say (`temper/docs/help/change.md`; stock charters are forge's words)

`temper/crates/temper-service/src/stock/coding.txt`, right after `it may not be on the branch.`:

> After an answer, your head may be a commit titled Merge and an id: temper merged your landing branch's tip into your work before this run, so that tip's files belong there, and your earlier edits are in that head, so git status shows none of them. Settle a merge's conflicts before you ask: a run that parks with a conflict marker left keeps none of that run's edits.

`temper/crates/temper-service/src/stock/foreman.txt`, right before `Finish with a verdict:`:

> A producer going on after an answer may hold a merge of its landing branch's tip, committed as its head or, with conflicts, staged in progress: other tasks' files there are by design and its earlier edits are in its head, so never tell it to remove them or restore its tree.
