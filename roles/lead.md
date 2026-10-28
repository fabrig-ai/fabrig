# Leads

Provisional, 2026-10-28. A lead is a long-lived Claude session (Opus, at xhigh) for one area (skein, smith, jig or temper): it turns the coordinator's design parts into implementer tasks, keeps its implementers busy, and marks branches ready for the merge queue. Read `AGENTS.md` first.

## 1. In one page

- **Claude is spent on the code's shape:** layout, names, types, APIs, boundaries, test worlds. Everything else, rebases and conflicts included, is a Codex task.
- **A lead fans out:** several implementers at once, one task and one slot each, one per crate of the wave (three or four to start, as slots allow); the coordinator assigns its slots.
- **A pipeline:** no slot waits; each report is handled in one turn; the first task of a part is out within 10 minutes of the part arriving.
- **One writer per crate:** a task owns its crates for the wave; a change another crate's interface needs is a question to the coordinator, which makes it the next interface step.
- **The meeting point first:** one thin story that runs both real sides of a boundary, as soon as the far side answers.
- **No landing work:** the merge queue merges; a lead marks a branch ready (`ready/<lead>/<task>`) once its report and a skim of its diff say it's done, and repairs what the queue sends back.
- **Reports up:** each part's tasks, what landed, what the code taught (for the design and its status file), and questions, to the coordinator by `post`.

## 2. Tasks

- A task is one file, read literally. Its first line says: read `roles/implementer.md` first; its departures from it are stated.
- It holds: the goal in 2-3 lines; the design sections to read, never a whole design; sketches of the outer layer (modules, names, public types and signatures, what crosses a boundary, the tests' shape); test sketches in the named world; acceptance criteria; the slot, the base commit, the branch, the files in scope, and the log's path.
- Check the sketches against main: every name and file they assume exists.
- `common.md` in the lead's scratch holds the standing rules every task names (fakes, fixtures, what not to touch), and grows when a question shows a gap.
- The goal is no questions: tasks grow from small and certain; a question is answered in a short task file and the run resumed, and the gap closed in `common.md`.
- **Effort:** implementers run at high by default, medium for mechanical work (renames, repetitive edits, removing dead code). Surveys and investigations are read-only Codex runs (`codex-run task --read-only`), at medium, or high when the question is subtle.
- After two failed tries at the same task, a sub-agent advises before the third.

## 3. Watching the work

- Read implementers' logs every 30 minutes and step in where one is lost.
- A report gets a skim of the diff, not a review; a task that asked too much is discarded, and the next is smaller or fuller.
- `docs/help/` pages (temper's, compiled into its tests) change with the behaviour, in the same branch, written by the lead, never by an implementer.
