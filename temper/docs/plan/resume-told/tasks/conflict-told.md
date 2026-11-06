+++
id = "conflict-told"
part = "conflict-told"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-domain-run/src/prompt.rs",
  "smith/crates/smith-domain-run/src/tests/workspace.rs",
  "smith/docs/design/domain/run.status.md",
]
effort = "medium"
minutes = 25
+++

# A run that starts in a merge in progress is told what it holds

## Goal

The request's item 1, its conflict case. `clean-merge` commits a producer's clean merge; a conflict stays in progress, with `MERGE_HEAD` and other tasks' files staged. The run's start names the files in conflict and says nothing of the rest. Tell it, in its system text, before it looks.

Acceptance:

- With any directory holding `conflicts`, the system text and a child's prompt hold the words of `notes/contracts.md`, 3, once, after the directories' lines.
- With none, the text is byte for byte what it is today.
- `initial merge conflicts: ` and its paths stay as they are.
- A resumed run's system text holds them too (`prompt::system` takes `resumed`).

## Inputs

- `smith/docs/design/domain/run.md`, 3.2, 8.3.
- `temper/docs/plan/resume-told/notes/common.md`, `temper/docs/plan/resume-told/notes/contracts.md`, 3.

## Owns and may change

Owns the files in the header. Smith only: no temper or jig file, and these words name no temper term.

Reader and neighbour files, read-only: `smith/tests/agent/tests/suite/initial_merge.rs` and `smith/crates/smith-inline-agent/src/workspace.rs` read conflicts: run the first. `temper-worker-domain/src/workspace.rs`, `directories`, supplies them as it does.

Departs from the implementer's text: delete from `smith/docs/design/domain/run.status.md` the `Not yet` entry beginning `8.3:`, and nothing else there.

Options, each pre-approved:

- The words go through `Text`, inside `render_checkout`'s host span, so `Text::measuring` counts them. If a constant counts this section's fixed bytes, add their length, and say so.
- If a measure's test or a limit no longer fits, stop: that's a question, with the numbers.

## Steps

1. **Reproduce,** in `smith/crates/smith-domain-run/src/tests/workspace.rs`, beside the test that asserts the `initial merge conflicts: ` fragment: `a_merge_in_progress_says_what_its_staged_files_are`, asserting the words on `opening.system` and on `prompt::child`, for a fresh start and a resumed one; and that a mount with no conflicts holds neither `MERGE_HEAD` nor `merge in progress`. It fails today.
2. **Fix:** in `render_checkout`, after the loop over directories, before `text.host = false`.
3. **Bite:** break the rule once, as your text asks, and record the failure.
4. Fmt, clippy on `smith-domain-run`, focused tests of `smith-domain-run` and `smith-agent-world`, then commit and push. `smith/docs/design/**` is read by tests (`.config/gate/read-by-tests.txt`): this task's design words are already on the trunk.

## Report

As your text says.
