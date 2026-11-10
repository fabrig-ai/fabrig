# A goal and its landings, step by step

## 1. A goal, as tasks

A person, P, reaches a plan for adding OAuth login to temper:

```
P, a person
├─ T0  a chat with P about temper          agent, a chat
└─ T1  add OAuth login                     agent, a coordinator's charter; tracked;
   │                                       T0 proposed it, P accepted it
   ├─ T2  spike: library X                 agent, read-only: a report
   ├─ T3  spike: a hand-rolled flow        agent, read-only: a report
   ├─ T4  choose       after T2, T3        person P: a choice
   ├─ T5  design       after T4            the forge's change, a design review as its gate
   ├─ T6…T9  changes   T5 as their input   the forge's change, made by T1
   │                                       once it read T5's result
   │     T7 amended while live: a memory-safety review as a gate
   └─ T10 try it       after T6…T9         agent: a report on what landed
```

- **T1 is proposed, explicitly.** Its authority (a budget, landing into
  main) is beyond T0's, so T0's `delegate` is refused, naming what it
  lacked, and T0 proposes it (jig's `domain/tasks/proposals.md`). P
  accepts it and funds it from P's pool. A goal a chat proposes is
  requested by the person who accepts it (jig's `domain/model.md`, 6.2),
  so T1 is P's, and outlives T0, which keeps a reference to it. Within
  T1's authority, its own delegations need nobody's acceptance.
- **T1 is a goal.** Its issue shows the plan as T1 revises it
  (`issues.md`); its changes queue for main at its priority (`queues.md`);
  landings in temper are news for it (`README.md`, section 5).
- **Results flow along dependencies:** T2's and T3's reports reach T4,
  which P answers in the client; T4's choice reaches T5's brief.
- **T6…T9 are a revision,** made by T1 once T5 landed, within its
  authority; T5 has ended, so its result reaches them as an input, not a
  dependency.
- **The memory-safety review is one amendment:** T1 adds a gate to T7
  (`change/gates.md`, 1). The change procedure runs a review task at each head of T7 its
  verdict does not carry over to, and holds the merge until it passes.
- **Under each change,** the procedure makes its own tasks: producing,
  repairs, reviews, resolving conflicts (`change/README.md`).
- **Had P asked for a small fix,** T0 would have made it itself and
  handed its branch to one change task (`change/README.md`, 1), and there would be no T1.
- **Had P closed T0** while T1 ran, T1 would go on: it is P's.

How T6…T9 land while another goal lands into the same branch is 15.2.

## 2. Landing, step by step

One repository, temper, with main at M0, and two goals in flight:

- **goal A:** change A1 is ready to land (CI green on its head a1,
  reviewed), and A has steps still to come;
- **goal B:** change B1 is ready to land (CI green on b1, reviewed), and
  change B2 is being produced by a run that started from M0.

A1 and B2 both change `limits.rs`; nothing else overlaps. A has the
higher priority.

| When | Who | Does | Afterwards |
|---|---|---|---|
| t0 | Engine | A1 is first in main's queue, and its head holds M0: merges A1 at exactly a1, through Forgejo | main is M1 |
| t0 | Engine | news for A (its own step landed) and B (A1 landed in temper, overlapping neither B1 nor anything B has pushed): neither wakes | B1 and B2 untouched |
| t1 | Engine | B1 is first, and its head lacks M1: asks Forgejo to update it | |
| t1 | Forgejo | merges M1 into B1's branch, cleanly | B1's head is b1′ |
| t1–t2 | CI | passes on b1′ | |
| t2 | Engine | B1's review carries over the clean update; merges B1 at exactly b1′ | main is M2 |
| t2 | Engine | news for B (its own step) and A (B1 overlaps nothing of A's): neither wakes | |
| t3 | Agent, worker | B2's producing run finishes; the worker pushes it | B2's head is b2, built on M0 |
| t3 | Engine, CI, agent | opens B2's pull request; CI passes on b2; a review task approves it | B2 is ready |
| t4 | Engine | B2 is first, and its head lacks M2: asks Forgejo to update it | |
| t4 | Forgejo | refuses: B2 and A1 both changed `limits.rs` | a conflict |
| t4 | Engine | B2 is resolving: a task for an agent; news for B, which does not wake it while the task resolves it | |
| t4 | Worker, agent | the worker prepares the merge in progress; the agent resolves `limits.rs` and runs the checks; the worker commits the merge, with both parents, and pushes it | B2's head is b2′ |
| t5 | CI, agent | CI passes on b2′; a review, asked again since an LLM resolved a conflict, approves it | |
| t5 | Engine | merges B2 at exactly b2′ | main is M3 |
| t5 | Engine | news for B (its own change landed: kept) and A (no overlap: kept); B2's result leaves B nothing to await, so jig makes B's closer and wakes no run (jig's `domain/tasks/lifecycle.md`, 11) | |
| t5 | Closer B | B holds no goal branch: no close step and no landing; its report, B1's and B2's pull requests and Left sections and their CI selection, ends goal B (`branches.md`, 2), and the issue's last milestone carries it | goal B is done |
