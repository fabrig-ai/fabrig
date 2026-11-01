# The reference case

Draft, 2026-11-01. Nothing here is adopted. One goal, from a person's
draft to its last verdict on main, condensed from a real run of the
reference workflow: 64 tasks in 6h23, here 8, with each mechanism at
play met once and the repetition gone. temper must be able to run it.
The rules are `reference-code-production.md` (CP) and
`reference-landing.md` (LD); this document applies them and adds none.
Times are the real run's where a step happened in it. That run stopped
at a green branch, so the landing steps come from other runs of the same
workflow.

## 1. The goal

The draft makes a repository's forges records, removable, and puts a
playground on them. One repository holds three trees: skein, the io
kit; jig, the agentic kit; temper, the application. A dependency points
one way between them.

```
a person
└─ G  the goal                                 designer; tracked
   ├─ c0  the shared shapes                    change into B, handed G's run branch
   ├─ A   the forge's records and removal      designer
   │  ├─ a1  store a forge as a record         change into B
   │  ├─ a2  remove a forge                    needs a1, later b2
   │  └─ a3  the playground                    needs a2
   ├─ K   skein's and the service's            designer
   │  ├─ k1  the connection at runtime         change into B
   │  └─ k2  a retire event                    added by an answer
   └─ L   the landing change                   B into main; needs c0, A, K
      ├─ r1  a repair for a red on B           change into B, made by code
      └─ u1  a conflict with main              change into B, made by code
```

B is `temper/<root>/g<L>`. Designers run Opus 5.5 at xhigh, their surveys
sub-agents at high; implementers run the coding model at high. On main
the rule asks the full gate; on B it asks nothing. A check follows B
every 10 landings or 45 minutes, and L keeps B up with main every 30.

In each table, *who* is code unless it names an LLM's role, and the last
column is the gap a step needs: none means built.

## 2. Production

### 2.1 Intake

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 1 | T+0:00 | The person starts G from the draft's path | a person | `goal`, a designer's charter | CP-12 |
| 2 | T+0:02 | G's first run reads the draft and judges: a branch of its own, two parts, the shapes that cross them. It writes those on its run branch. One question for the person, which no work waits for, goes in its report | designer | a coordinator's fresh run | CP-12 |
| 3 | T+0:18 | G's one batch: L, c0 handed the run branch, A, K. L starts kept, and its first step creates B at main's tip | designer, then code | a batch; a handed branch; kept; a goal's branch | CP-2, CP-5 |
| 4 | T+0:20 | c0 opens and lands into B: nothing to prove | code | a landing of one, no gates | CP-3, CP-4 |

### 2.2 Growing

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 5 | T+0:21 | A's first run judges a survey worthwhile: a read-only sub-agent says what reads a forge today. It delegates a1 at once, a2 and a3 as it writes them, each with the paths it owns and its options with defaults. a2 and a3 chain on `needs`: one writer of one crate. K's run delegates k1 the same way | designer | sub-agents; `delegate` in batches, one run many times | CP-10 |
| 6 | T+0:27 | a1's and k1's producers start from B's tip, and both designers are still writing | code | the change, producing | |
| 7 | T+0:50 | a1 lands into B, and a2's producer starts from the new tip | code | a dependency met | |
| 8 | T+0:52 | a2 has pushed a head and has no pending dependency: a3 starts early, from B's tip with a2's head merged in, kept | code | an early start | CP-5, CP-6 |

### 2.3 A question grows the plan

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 9 | T+1:20 | a2's implementer finds skein has no retire event, which removal needs and no task owns. It pushes what it has and ends: it needs a decision, with options | implementer | a typed end | CP-1 |
| 10 | T+1:20 | The change is held at producing, and the hold escalates past it to A, which it wakes. a1, k1 and a3 go on | code | a hold, an escalation | CP-1 |
| 11 | T+1:29 | A's fresh run reads the question in its brief and judges: a new task. It delegates k2 and amends a2's spec with the answer and its dependencies with k2, then releases it. a2 is kept | designer | a batch; an amendment; a release | CP-11 |
| 12 | T+1:30 | a3, started early, finds removal unfinished and fails saying so. It asks no one: having started early once, the change waits for a2 and runs again | implementer, then code | the wait after an early start | CP-6 |
| 13 | T+1:50 | k2 lands. a2's fresh producer starts from its pushed head with B's tip merged in, its spec carrying the answer | code | a release's fresh producer | CP-1 |
| 14 | T+2:40 | a2 reaches its bound of four runs without landing, and is held the same way. A judges in a minute: stay, and finish what the answer said | code, then designer | a bound | CP-1 |

### 2.4 The branch stays green

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 15 | T+1:05 | 45 minutes after the first landing, the check runs on B's tip. Three tests fail; rerun alone, one passes, a flake, counted | code | a tier on a goal's branch; its cadence | CP-7 |
| 16 | T+1:12 | The two left are new. L's step makes r1, a change into B: the failing output, and a3 named as not landed, with its paths to leave alone. No LLM has been asked | code | a red makes a repair | CP-7 |
| 17 | T+1:40 | r1's implementer asks whether a stall in a real-loop test is its own. L is a procedure, so the hold reaches G: "you own it: it is red on the branch." r1 lands | implementer, designer | a hold, past a procedure | CP-1 |
| 18 | T+2:00 | The next check runs after r1's landing, and is green | code | a tier | CP-7 |

### 2.5 Meeting others

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 19 | T+3:00 | k1 finishes with its acceptance held and one list in its result's body: left, a fixture that a3 owns. It lands | implementer | a result's body | |
| 20 | T+3:30 | Main has moved, and 30 minutes have passed. L merges main into B, cleanly, by the forge | code | an update on a cadence, while kept | CP-8 |
| 21 | T+4:00 | Main moved again and a2 lowered a counted file that main lowered too. The project's tooling recounts it, with no LLM | code | `prepare`, on a job | CP-9 |
| 22 | T+4:30 | The next update conflicts in one crate, past what tooling knows. L's step makes u1, a change into B from main merged into B's tip, its files marked; an implementer keeps both sides' intent; it lands, and landings into B never waited. L sees B holds main's tip | code, implementer | a resolution as a change into B | CP-8 |
| 23 | T+4:40 | a3's landing conflicts with k2 on B. The forge refuses the update, and a3's change makes a resolution under its own hold. It lands | code, implementer | a resolution (`forge.md`, 8.4) | |

### 2.6 Ready

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 24 | T+6:00 | A wakes on the last of its delegates, reads what they left and judges nothing is missing; K the same. Each finishes with its report | designer | a coordinator's last run | |
| 25 | T+6:01 | Every part has ended, so L is no longer kept. No LLM marks it: B is ready to land | code | a dependency on a subtree | |

## 3. Landing

| # | When | What happens | Who | By | Needs |
|---|---|---|---|---|---|
| 26 | +0:00 | L's landing prepares: B lacks main's tip, main is merged in, and the head is the candidate | code | prepare, by a merge | LD-1 |
| 27 | +0:01 | The full gate is dispatched at the candidate. One test fails, and fails again alone: a real red | code | proof by dispatch; a gate's rerun | LD-2 |
| 28 | +0:06 | A repair under L, its brief carrying the failed step and the log's end. Nothing else writes B now. Its new head prepares again | code, implementer | a repair | |
| 29 | +0:30 | The gate's job dies in its set-up step: a runner's fault, read from how it ended. It is retried, and goes green | code | a fault (`landing.md`, 7.4) | |
| 30 | +0:35 | The turn reads main afresh: another goal landed first. Stale, L prepares again, and the gate runs only because the tree changed | code | a stale proof | |
| 31 | +0:40 | Green; the turn fast-forwards main to the candidate. B is deleted and G hears | code | the turn | |
| 32 | +0:50 | The slow tier runs on main's newest tip, covering both landings. Its failures rerun once: one is a flake, one is new, and with two landings in range both goals are told | code | a tier; `tell_all` | CP-7, LD-5 |
| 33 | +0:52 | G, live and subscribed to the verdict, wakes and judges: a fix. It delegates one change into main, with no branch; an implementer, the full gate, a turn | designer, implementer | a landing with no goal's branch | LD-4 |
| 34 | +1:40 | The tier goes green over the fix: the verdict. G's last run writes its status files and retires the draft: one change into main, documentation, which every gate ignores, so it is proved with no run | designer | keys; ignored paths | LD-4 |
| 35 | +1:50 | G reports to the person, with the question of step 2 answered or still open, and ends | designer | a goal's result | |

A red found later, when G has ended, goes to a finding and its triager,
an LLM, which makes it a new goal (LD-6).

## 4. Had it gone worse

Two paths the run met elsewhere, each a sentence.

- **A third red at the gate** holds L past its bound, and the hold wakes
  G (LD-3), which may delegate a part and amend L's dependencies: L is
  kept again, and landings into B go on.
- **A question that is the person's** G passes up: only the task that
  asked parks, and the person answers in their inbox.

## 5. What each gap costs the case

| Gap | Steps that stop without it |
|---|---|
| CP-1, a question as a hold | 9 to 14, 17 |
| CP-2 to CP-4, a goal's branch and landing on it | 3, 4, every landing into B |
| CP-5, CP-6, kept and early starts | 3, 8, 12 |
| CP-7, tiers | 15 to 18, 32 |
| CP-8, keeping up | 20, 22 |
| CP-9, jobs and `prepare` | 21, which then costs a resolution; the case still runs |
| CP-10 to CP-13, delegation, amendment, charters, limits | 1, 2, 5, 11 |
| LD-1 to LD-5, prepare's style, reruns, holds, verdicts | 26, 27, 32 to 34; with the forge's rebase, 26 runs |
| LD-6, findings | section 3's last line |

The first walk may stop at step 25, a green branch unlanded, as the real
run did.

## 6. Left out

- **Repetition:** 56 more tasks, 7 more reds, 39 more questions, the
  implementers in flight beyond the four here. Each repeats a step.
- **Numbers and noise of the real run:** the runs a judge spent on what
  kept and a mechanical wait now settle; merges that only carried specs;
  the event log's own losses.
- **Where things run:** hosts, workers, runners, capacity.
- **Options off in the workflow** (`reference-code-production.md`,
  10.4): skeletons, reviews, surveys beyond one, slices, QA walks.
