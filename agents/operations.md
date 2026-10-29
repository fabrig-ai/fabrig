# Operations

Provisional, 2026-10-28. The machines' and the pipeline's facts every role works with: where things run, how a task is launched, slots, surveys, the queue. Rules are in `AGENTS.md` and each role's file; this file says how.

## 1. In one page

- **hel8 runs sessions and surveys, never builds:** the coordinator's and the leads' sessions, their clones, and read-only Codex surveys. Builders (hetzner3, 6, 7) build in slots; hetzner8 runs the merge queue, and drained hetzner2 is reserved for its second prover. The prover remains disabled until the train code is ready.
- **A lead** is a Claude session on hel8 named `lead/<area>`, started from a directory Claude trusts (under `~/src/rust/`), with its scratch and clones in `~/src/fabrig/leads/<area>/`: `repo/` (read main, mark branches ready) and `survey/` (surveys).
- **Implementers are Codex runs** on a builder, one task in one slot, launched by `codex-run`; their end is posted to whoever launched them.
- **Leads are ephemeral:** each is started for one part from `~/src/rust/tmp/supervise/leads/<area>` and starts warm from the files in its scratch (`agents/lead.md`, 4).
- **Slots are for builds:** a fixed share per lead, the rest a free pool claimed with `claim` from PATH on hel8; `codex-run` trims a writable task's slot before Codex starts (`tools/slots/trim`).
- **Surveys take no slot:** a read-only run builds nothing and runs on hel8 in a survey clone moved to the commit it reads.
- **The queue restarts only through `tools/queue/restart`,** in a login shell on hetzner8.

## 2. Launching a task

- One writer per checkout: the coordinator edits in a clone of its own (`~/src/fabrig/coord/<name>`), and each Codex run has its own checkout. Do not edit, stage or commit in a checkout while its run is writing there.
- The task file and anything it must read go to the builder (`~/src/fabrig/tasks/<area>/`), written with `ssh HOST 'umask 077; cat > PATH' < FILE`; the report and log land in `~/src/fabrig/notes/<area>/` there.
- `ssh HOST 'bash -lc "codex-run task --effort high --to lead/<area>@hel8 TASK SLOT REPORT </dev/null"'`; `--effort medium` for mechanical work; never xhigh. A question or a repair resumes the same run: `--resume <run id>`, a new task file and a NEW report path.
- Remote commands run in a login shell (`bash -lc`), or `cargo`, `heavy`, `codex-run` and `post` are not on PATH.
- Hosts and their slots: `tools/slots/hosts`; post routes: `~/src/rust/tmp/post/machines`.

## 3. Slots

- Run installed `claim` from PATH on hel8, not from a lead's clone: it reads main's `tools/slots/hosts`. `claim list` shows shares and holders; `claim list --free OWNER` lists free slots of that share, then the pool.
- `claim claim HOST N OWNER WHAT` before a task takes a slot, `claim release HOST N OWNER` when its report is in. Another lead's share needs its owner's agreement and `--lent-by SHARE_OWNER`; a free slot in that share is not pool space.
- A guard on each builder (`tools/slots/guard --timer`) trims idle slots when free space falls below 80 GB.
- A slot's target stays warm across tasks: a task switches the slot's checkout in place, never a fresh clone. Stale test executables fill a builder's disk (one reached zero bytes on 10-28, and heavy then fails its own bookkeeping), so `codex-run` trims a writable task's slot before Codex starts (`tools/slots/trim`; by hand: `ssh HOST 'bash -lc "python3 -I - SLOT"' < tools/slots/trim` from hel8).

## 4. Surveys

- `cd ~/src/fabrig/leads/<area>/survey && git fetch -q origin && git switch -q --detach <commit>`, then `codex-run task --read-only --effort medium --to lead/<area>@hel8 TASK <survey clone> REPORT` with the task and report in the lead's scratch. A read-only run cannot fetch: the clone is moved first and the task names the commit.

## 5. The merge queue

- The current trial uses batches of 16. Preserve it on every restart with `tools/queue/restart --repo ~/src/fabrig/queue --batch 16`; omitting the option restores the default of 8. The operator keeps or reverts the trial from measured throughput and bisection costs.
- `tools/queue/design.md`. Branches `<lead>/<task>`, ready as `ready/<lead>/<task>`, marked by the lead from `repo/` after a skim of the diff; a rebased branch gets a new name. The queue posts a rejection (step, log path on hetzner8) to the branch's lead; a rejection is a repair task, a conflict a rebase task.
- Tasks need not lower `scripts/old-forms.txt`: after merging, the queue runs each touched tree's `old_forms.py --write` and commits lower baselines before checks. A raised count rejects or bisects the batch. Branch baseline changes still merge; a conflict confined to baselines merges by itself through a recount, which refuses a raise.
- When the queue's own code is wrong in a way that stops it landing its fix (every batch fails a step), the operator runs the fixed `tools/queue/` from a copy outside the clone (`mq run --repo <clone>`) until the fix lands, then restarts it normally.

## 6. Later

- A periodic run of each package's tests alone, to catch tests that pass only after a workspace build.

## 7. Succession of long-running sessions

- **Who:** every session that outlives its tasks: the coordinator and the operator (leads end instead: the last bullet). A session's context is finite; a session that compacts loses what it knew in its own words.
- **When:** at about three quarters of the context window the session starts its successor, which usually comes before any compaction; if a compaction happens first, the handover starts right after it, never after a second. It never waits for the window to fill.
- **How:**
  1. The incumbent brings its handover file up to date (where things stand, what is in flight with run ids and branches, decisions pending and whose, what it was about to do) and starts the successor with its role file, the handover and the same mailbox name to inherit.
  2. **Warm-up, read-only:** the successor reads the handover, the role and operations files, the plans and the ledger, and follows the mailbox without acting (reading delivered messages' copies, never `post read` of the incumbent's box); it asks the incumbent what is unclear under a temporary name (`<role>-next`), and says when it is ready.
  3. **Takeover:** the incumbent stops its waiter and says so; the successor starts its own on the inherited name, tells the parties it talks to that the role changed hands, and the incumbent stops.
- **The operator's successor** warms up under `ops-next` like any other; since one `codex-run owner` holds a name's mail, the takeover stops the incumbent owner first, then starts `codex-run owner --effort high ops BRIEF` with the handover in its brief.
- **Leads are ephemeral, not succeeded:** a lead runs one part (at most one wave) of its area and keeps its `state.md` current after every event; its area's lasting facts live in its `common.md` and the designs. At the same threshold, or when its part is done, it writes those files current and stops, and the next lead starts warm from them on the same mailbox, with no warm-up overlap (`agents/lead.md`, 4).
