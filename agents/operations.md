# Operations

Provisional, 2026-10-28. The machines' and the pipeline's facts every role works with: where things run, how a task is launched, slots, surveys, the queue. Rules are in `AGENTS.md` and each role's file; this file says how.

## 1. In one page

- **hel8 runs sessions and surveys, never builds:** the coordinator's and the leads' sessions, their clones, and read-only Codex surveys. Builders (hetzner3, 6, 7) build in slots; hetzner8 runs the merge queue, and drained hetzner2 is reserved for its second prover. The prover remains disabled until the train code is ready.
- **A lead** is a Claude session on hel8 named `lead/<area>`, started from a directory Claude trusts (under `~/src/rust/`), with its scratch and clones in `~/src/fabrig/leads/<area>/`: `repo/` (read main, mark branches ready) and `survey/` (surveys).
- **Implementers are Codex runs** on a builder, one task in one slot, launched by `codex-run`; their end is posted to whoever launched them.
- **Slots are for builds:** a fixed share per lead, the rest a free pool claimed with `claim` from PATH on hel8; every task trims its slot first (`tools/slots/trim`).
- **Surveys take no slot:** a read-only run builds nothing and runs on hel8 in a survey clone moved to the commit it reads.
- **The queue restarts only through `tools/queue/restart`,** in a login shell on hetzner8.

## 2. Launching a task

- One writer per checkout: the coordinator edits in `~/src/fabrig/coord/mine`, and each Codex run has its own checkout. Do not edit, stage or commit in a checkout while its run is writing there.
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

- `tools/queue/design.md`. Branches `<lead>/<task>`, ready as `ready/<lead>/<task>`, marked by the lead from `repo/` after a skim of the diff; a rebased branch gets a new name. The queue posts a rejection (step, log path on hetzner8) to the branch's lead; a rejection is a repair task, a conflict a rebase task.
- Tasks need not lower `scripts/old-forms.txt`: after merging, the queue runs each touched tree's `old_forms.py --write` and commits lower baselines before checks. A raised count rejects or bisects the batch. Branch baseline changes still merge; a conflict confined to baselines merges by itself through a recount, which refuses a raise.
- When the queue's own code is wrong in a way that stops it landing its fix (every batch fails a step), the operator runs the fixed `tools/queue/` from a copy outside the clone (`mq run --repo <clone>`) until the fix lands, then restarts it normally.

## 6. Later

- `codex-run` trimming the slot itself, so no task can forget.
- A periodic run of each package's tests alone, to catch tests that pass only after a workspace build.
