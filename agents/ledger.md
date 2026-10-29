# The remedies ledger

Provisional, 2026-10-28. Kept by the operator (`agents/operator.md`, 5); every role reports its incidents to it. The goal is throughput: (1) a pipeline in which each role schedules the next as soon as it can and plans more while the next is busy, (2) short builds on warm caches, (3) no spurious blocking. Each row is an incident that cost one of them, what it cost, its remedy, and where the remedy lives so the next agent meets it. A new session reads this file before its role's work.

| # | goal | incident | cost | remedy | lives in |
|---|---|---|---|---|---|
| 1 | 3 | the queue rejected good branches because heavy refused below its disk floor | repair rounds | a floor refusal is an error, not a red; the queue waits below its floor and trims its own test executables | `tools/queue/mq` |
| 2 | 3 | a builder's disk reached zero bytes: slots' stale test executables; heavy failed its own bookkeeping | a stalled task, a manual reconcile | every task trims its slot first; fewer test binaries | `tools/slots/trim`; `agents/implementer.md` 1; `agents/operations.md` 3 |
| 3 | 3 | conflicts in `old-forms.txt` between branches that each lowered a count | rebase rounds | the queue merges a conflict confined to it by `--write` | `tools/queue/mq` |
| 4 | 2 | every batch a full workspace run (lockfile and manifest changes forced it) | 7-11 min a batch | lockfile-aware selection; fmt on changed packages; build and test timed apart | `tools/queue/mq` |
| 5 | 2 | ~1,366 test executables per full build; linking dominates the test step | build time, disk | one integration binary per package | `AGENTS.md` 6; the nextest profiles |
| 6 | 3 | a gate red under load (it was a real bug that load exposed) | a batch and a repair | harness deadlines tell starved from failed through one skein mechanism | `skein/docs/foundation/testing-strategy.md` 2.10 |
| 7 | 3 | the queue restarted without a login shell: no cargo or post on PATH, no alert | errored batches | restart only through the tool; `mq run` refuses without its tools | `tools/queue/restart`; `agents/operations.md` 1 |
| 8 | 3 | the queue's output block-buffered under nohup | a false alarm | unbuffered (`-u`) | `tools/queue/restart` |
| 9 | 1 | one branch spanning four trees held three other parts for an hour | idle slots | no branch crosses trees: expand, move, contract | `agents/coordinator.md` 3 |
| 10 | 1 | leads asked the coordinator for every slot | a round trip each | fixed shares plus a pool leads claim | `tools/slots/claim`; `agents/lead.md` 1 |
| 11 | 1 | read-only surveys stopped: their sandbox cannot fetch; they also took slots | round trips, slots | surveys run on hel8 in a survey clone moved first | `agents/operations.md` 4 |
| 12 | 1 | a task bound a tool's syntax that the tool's version refuses | two round trips | a task binds the check, not the syntax | `agents/coordinator.md` 3; `agents/lead.md` 2 |
| 13 | 3 | a test ran another package's binary that only a workspace build makes | a red in a package-scoped run | a test's binaries are its own package's targets | smith's test; `testing-strategy.md` (to come) |
| 14 | 3 | the queue's own build step passed flags nextest refuses together; every building batch failed, including its fix | ~6 min of rejections | the fix runs from a copy until it lands; the queue's tests run its commands' arguments against nextest's rules | `agents/operations.md` 5; `tools/queue/test_mq.py` |
| 15 | 1 | a lead claimed another lead's share "from the pool": shares lived only in briefs; a new pool slot was invisible in leads' clones | a lead without a slot; round trips | shares in `hosts` (an owner per slot or `pool`), `claim` refuses another's share without a lend, installed from main | `tools/slots/hosts`, `tools/slots/claim`; `agents/operator.md` 3 |
| 16 | 3 | tests that relaunch their own binary with `--exact <name>` pass silently after a rename: the child selects nothing and exits 0 | a hidden loss of coverage, caught before it happened | a relaunched child reports having run exactly one test | wave 2.5's recipe; `testing-strategy.md` (to come) |
| 17 | 3 | `tools/slots/trim` refused every slot: a systemd process's fd directory is unreadable | every task's first step failed until told | skip unreadable processes; a tool's task ends with one real, harmless run on a builder | `tools/slots/trim`; `agents/operator.md` 4 |
| 18 | 3 | hetzner7 fell under heavy's floor (35 GB): one slot held 1651 test executables (174 GB), all younger than the two hours a task-start trim keeps | a refused run, a round trip | a guard on each builder trims idle slots fully when free space runs low | `tools/slots/guard` (to come); `agents/operator.md` 3 |
