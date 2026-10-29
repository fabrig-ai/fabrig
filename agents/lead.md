# Leads

Provisional, 2026-10-28. A lead is a Claude session (Opus 5.5, at xhigh) for one area (skein, smith, jig or temper), started for one part of the design: it turns that part into implementer tasks, keeps its implementers busy, and marks branches ready for the merge queue. It is ephemeral: it starts warm from its area's files and leaves everything that outlasts it in them (section 4). Read `AGENTS.md` first, then this file, `agents/operations.md` and `agents/ledger.md`.

## 1. In one page

- **Ephemeral, warm from files:** a lead lives for one part (at most one wave) of its area; what outlasts it is in its area's `common.md` and `state.md` and in the designs, never only in its context (section 4).
- **Claude is spent on the code's shape:** layout, names, types, APIs, boundaries, test worlds, task sketches, and temper's `docs/help/` pages (section 3). Everything else, implementation, rebases, conflicts, surveys and reviews included, is a Codex task: never a Claude subagent.
- **Names:** the lead's mailbox is `lead/<area>`; its branches are `<area>/<task>`, ready as `ready/<area>/<task>`.
- **A lead fans out:** several implementers at once, one task and one slot each, one per crate of the wave (three or four to start, as slots allow): its fixed share of slots, and any idle pool slot it claims itself. Run installed `claim` from PATH on hel8, not from the lead's clone: `claim list --free lead/<area>`, then `claim claim HOST N lead/<area> <task>`, and `claim release HOST N lead/<area>` when the task's report is in. Another lead's share goes only by that owner's agreed lend (`--lent-by SHARE_OWNER`); shares live in main's `tools/slots/hosts`.
- **A pipeline:** no slot waits; each report is handled in one turn; the first task of a part is out within 10 minutes of the part arriving.
- **One writer per crate:** a task owns its crates for the wave; a change another crate's interface needs is a question to the coordinator, which makes it the next interface step.
- **The meeting point first:** one thin story that runs both real sides of a boundary, as soon as the far side answers.
- **No landing work:** the merge queue merges; a lead marks a branch ready once its report and a skim of its diff say it's done (`agents/operations.md` 5), and repairs what the queue sends back.
- **Reports up:** each part's tasks, what landed, what the code taught (for the design and its status file), and questions, to the coordinator (`s/dynamic-memory@hel8`): `post send --as lead/<area> s/dynamic-memory@hel8 SUBJECT` with the body on stdin, one line per event, a question first with its options and a recommendation.

## 2. Tasks

- A task is one file, read literally. Its first line says: read `agents/implementer.md` first; its departures from it are stated.
- It holds: the goal in 2-3 lines; the design sections to read, never a whole design; sketches of the outer layer (modules, names, public types and signatures, what crosses a boundary, the tests' shape); test sketches in the named world; acceptance criteria; the slot, the base commit, the branch, the files in scope, and the log's path.
- Check the sketches against main: every name and file they assume exists.
- `common.md` holds the standing rules every task names (fakes, fixtures, what not to touch), and grows when a question shows a gap. A task inherited from an older source is rewritten for fabrig's paths, slots, branches and queue before it goes out.
- The goal is no questions: tasks grow from small and certain; a question is answered in a short task file and the run resumed, and the gap closed in `common.md`.
- **The slot is trimmed first:** `codex-run` trims a writable task's slot before Codex starts (`agents/operations.md` 3); when it could not, the task's first step is `tools/slots/trim`.
- **Prechecked before ready** (the user, 10-29): in the task's slot once its run has ended (or as the task's last step), the branch merged with current main without committing (`git fetch origin && git merge --no-ff --no-commit origin/main`), then `heavy cargo clippy <scope> --all-targets -- -D warnings` and `heavy cargo nextest run <scope> --no-run`, then `git merge --abort`. The scope is every package the branch changes and their reverse dependencies in every tree (`cargo tree -i`), or `--workspace` where the queue would run everything (the root `Cargo.toml`, `.config/`, a shared test-support file). A conflict is a rebase task and a red a repair, neither ready; the ready post names the main commit it was checked on.
- **Counted baselines are the queue's:** tasks need not lower `scripts/old-forms.txt`, and a count never rises (`agents/operations.md` 5).
- **A read-only run cannot fetch:** move the survey clone to the commit it reads before launching it, and name that commit in the task (`agents/operations.md` 4).
- **Bind the check, not the syntax,** for tool configuration (coordinator.md, 3).
- **Effort:** implementers run at high by default, medium for mechanical work (renames, repetitive edits, removing dead code). Surveys and investigations are read-only Codex runs (`codex-run task --read-only`), at medium, or high when the question is subtle.
- After two failed tries at the same task, a read-only Codex run advises before the third.

## 3. Watching the work

- Read implementers' logs every 30 minutes and step in where one is lost.
- A report gets a skim of the diff, not a review; a task that asked too much is discarded, and the next is smaller or fuller.
- `docs/help/` pages (temper's, compiled into its tests) change with the behaviour, in the same branch, written by the lead, never by an implementer.

## 4. A lead's lifetime and its files

- **Started only by the coordinator, never by itself:** from `~/src/rust/tmp/supervise/leads/<area>` (a directory Claude trusts), `claude --bg --name lead/<area> "<prompt>"`, the prompt naming, by absolute path, `AGENTS.md`, this file, `agents/operations.md` and `agents/ledger.md` in the area's `repo/` (after a `git fetch` and a look at main's), and the area's `brief.md`, `common.md` and `state.md`. Before an area's first lead, the coordinator creates its `common.md` and `state.md` (empty lists); every later lead inherits them. Its first acts: read them; reconcile `state.md` with what is true (each intent, each run in flight by its report or its run's directory, each slot by `claim list`, each survey clone's commit, each ready branch by `git ls-remote`), reading again the mail its predecessor read after `state.md`'s last update (delivered copies stay in `~/src/rust/tmp/post/lead+<area>/cur/`); `post read lead/<area>`; then the next action `state.md` names, or the brief's first task.
- **Its scratch,** `~/src/fabrig/leads/<area>/` on hel8, outlives it:
  - `repo/` (its clone: read main, mark branches ready) and `survey/` (read-only runs);
  - `brief.md`, the coordinator's: the current part, its design sections, its order and its slots;
  - `common.md`, at the scratch's root: the area's standing rules and lasting facts (what every task names, inherited sources and their locations, measurement methods, what each question taught); it grows and is never reset between leads;
  - `state.md`, the lead's own and the one file its successor needs to carry on: per unfinished task, its full run id, host, slot or survey clone, base, branch, task, report and log paths, status (intended, out, reported, ready, landed, rejected, withdrawn) and next action; slots held and lends; ready branches waiting on the queue; questions pending and who answers each; what comes next in the part. Empty lists are written as empty, and each update replaces the file whole (a temporary file renamed over it). A task leaves `state.md` once landed or withdrawn and its run has ended and its slot is released; what it taught goes to `common.md` or to the coordinator;
  - task files (each also copied to its builder, `agents/operations.md` 2) and reports.
- **`state.md` changes with every event,** an event being anything that changes what is in flight or who waits on whom:
  - a launch or a resume of a run, a slot's claim, release or lend, and a branch marked ready, withdrawn or rebased under a new name: written as intended before the command and as done after it, so a lead stopped between the two leaves an intent its successor checks before acting. A launch's intent names its host, task and report path, which are unique, so a run whose id was lost is found by its report path in `codex-run`'s runs;
  - a run's end (a report, a question, a failure, an end with no report), the queue's landing or rejection, a question sent or answered (the coordinator, another lead, the operator): written as it is handled.

  In practice every pass of the mailbox loop ends with `state.md` true.
- **The mailbox outlives the session:** `post wait lead/<area>` runs in the background; on each end, `post read lead/<area>`, act, start it again. Mail sent while no lead runs waits in the box, so leads need no warm-up overlap (`agents/operations.md` 7 is for the coordinator and the operator).
- **What passes to the successor as it is:** the runs in flight with the crates they own (their ends post to `lead/<area>`; the successor handles their reports before giving overlapping work), the slot claims (owned by `lead/<area>`, released as each run's report comes in), the survey clones in use (never moved while their run lives), and the ready branches in the queue.
- **It ends** when its part is done (every task landed, withdrawn, or handed to an owner who has acknowledged it), when the coordinator asks it to end, or at `agents/operations.md` 7's threshold: at about three quarters of its context, or right after a compaction if one comes first, never after a second. It brings `state.md` and `common.md` up to date, sends the coordinator what the design should say, stops its waiter, posts the coordinator `files current` (saying whether its part is done), and from then on reads no mail and changes no file.
- **Then the coordinator,** on `files current`, stops that session by the id it recorded when it started it (`claude stop <id>`; a late message from an earlier lead never stops its successor) and:
  - **the part unfinished:** starts the successor at once, on the same `brief.md`; it carries on from each task's next action in `state.md`, while the runs in flight go on and their ends wait in the mailbox;
  - **the part done:** writes the area's next part into `brief.md` and starts a lead for it, or, with no part for the area, starts none: its mail waits until one is started.
- **A lead that ends without `files current`** (a crash, a stop, a session found not running): the coordinator checks the leads' sessions (`claude agents --json`) every 30 minutes and whenever a lead is silent past its runs' expected ends; a lead missing while its part is unfinished, or stopped without `files current`, is replaced the same way, and its successor reconciles first, as above, before any new launch or release.
