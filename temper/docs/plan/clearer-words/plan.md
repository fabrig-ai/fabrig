# clearer-words: three small gaps a person meets in temper's help and ctl

Two tasks side by side, one for the help's words and one for ctl's refusal. Neither needs the other.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `help-words` | `help-words` | | the request's 1 and 2: the merge command's contract in `temper help configuration`, `change`'s `resolving` pointing to it, and a cancelled early start in `change` and `task` |
| `answer-bound` | `answer-bound` | | the request's 3: `task answer`'s refusal names the task's bound, and its help says a task may ask for less |

Critical path: either task alone; they share no file.

## What was decided

- **1 and 2 are one task.** Both edit `change.md`, whose bound leaves no room for two writers to size it apart.
- **The contract is a section, not the line.** The request asks the `merge_job` line to say it, but `the_rows_have_unique_names_and_bounded_lines` holds a member's line to 160 bytes. The line points to a closing section, `The merge command`, which search also finds. Raising the bound for one row would have spent a rule to save a pointer.
- **`run from the repository root` goes, because it is untrue.** The worker's `written_job` spawns the command in the job's directory: the path is relative to the repository, the working directory is not. A person writing a command from the old words would run `git` in the wrong place.
- **The section reads its numbers.** The result bound, the deadline and `PATH` come from `limits.rs`, so the page can't drift from them.
- **The pages are written here, to the byte.** `change.md` and `task.md` sit 12 and 2 bytes under 8,192. `notes/help-words.md` holds each replacement, worked out against the trunk: 8,186 and 8,168 bytes after. What they remove is said elsewhere on the page, or is a line end. The implementer applies them rather than choosing what to lose. Two removals to read in review: `change`'s `Bounded work` bullet, which the Held list says in full, and `task`'s `(no tool)` clause, which its Limits says for the acts it lists.
- **The refusal stays the engine's; ctl adds what it already read.** Carrying the bound in jig's refusal would change a stored form and the protocol, which are the user's and which "nothing else changes" rules out. `task answer --rev` reads the person task before it asks, so the contract is in hand.
- **The bound is named only where it is the cause.** `LimitKind::Contract` also covers a wrong form and an unknown choice, so ctl names the bound only where the text is past it. Otherwise, and for `--cannot`, `--approve` and a restored request, the bare word stands. Those have no bound the task asks for, and a restored request read nothing.
- **JSON gets `bound` too.** The JSON is the contract for scripts (`temper/docs/design/client/tools.md`, 6.2), and a script meets the same gap.
- **A joined story beside the step tests.** Only the real engine shows the refusal really is `Contract`; a step test's refusal is written by hand. It is one more story in a world that exists.

## Overlaps

`help-words` touches no file a draft beside it changes: no other draft writes `change.md`, `task.md`, `members.rs` or `reference.rs`, though `ui-settings/ways-ctl` regenerates under `docs/help/reference/`. `answer-bound` could not avoid `note-parts/task-notes` and `ui-settings/ways-ctl`, both merged on their own trunks: `rows.rs` and `ctl-task.md`, and for `task-notes` also `render/**` and `tests/jig/**`. It adds two lines of notes to one tool and a few lines to `render/outcome.rs`, and `ctl-task.md` regenerates. Neither task depends on those drafts.

## Design written

- `temper/docs/design/client/tools.md`, 5.5, with its **Not yet** in `tools.status.md`, which `answer-bound` closes.
- `temper/docs/design/protocol/documentation.md`, 8, with its **Not yet** in `documentation.status.md`, which `help-words` closes.

No draft's status changes: no draft speaks to these.
