# Reference case

Draft, 2026-11-02. One small goal through the reference, from a draft to
cleared, showing each mechanism of `reference-code-production.md` and
`reference-landing.md` once. It is condensed from a real run of the
`playground-reset.md` draft in this directory. It keeps that run's
names and kinds of event and drops its repetition: 64 changes become
nine, 54 questions four, and nine checks along the way one.
`reference-notes.md`, 2.3 maps each step to the run, and its N numbers
are the design changes each step needs. It replaces the case's third
edition (N23).

## 1. In one page

- **The goal:** P, a person, starts a goal from the draft. G, its
  coordinator, writes the plan's notes and asks two designers for their
  families; nine changes in four families go out in four batches, and
  two more come later, made by an answer and by a Left list.
- **Act 1, the cut** (section 3): surveys as sub-agents, the plan's
  notes, designers as tasks, a designer's question, batches as each
  family is ready.
- **Act 2, changes into the goal branch** (section 4): the plain way; a
  question settled by the foreman and by G; a new change made by an
  answer; early starts, a merge held for its dependency, an early start
  refused; a Left list; a hold passed up to P; conflicts settled by the
  merge job and by an agent.
- **Act 3, along the way** (section 5): a red of the goal branch's tier
  repaired while merges go on; a catch-up with main.
- **Act 4, landing** (section 6): B closed, checked whole, repaired once,
  updated and merged at its head.
- **Act 5, after landing** (section 7): a red of main's tier, triaged
  and repaired by a change naming it; B cleared; the retire.

## 2. The cast

- **P,** the person who wrote the draft.
- **G,** the goal: an agent task under the designer's charter, Opus 5.5
  at xhigh, its sub-agents Opus 5.5 at high; tracked.
- **B,** the goal branch: the forge's change into main, collecting, its
  branch `temper/<root>/g<B>` (N3).
- **Dbx, Dfe,** designers for the box and the forge's engine: agent
  tasks under the designer's charter, narrowed to reads, notes and
  messages.
- **The changes,** each into B's branch, its words its spec, its
  producer a coding agent at gpt-6.1-sol:

| | Change | Family | After | Shows |
|---|---|---|---|---|
| C1 | sv-skein-setup | sv, skein and service | | the plain way |
| C2 | bx-doc | bx, the box | | a question G settles by widening the words |
| C3 | bx-init | bx | C2 | an early start; a merge held for its dependency |
| C4 | fe-remove | fe, the forge's engine | | a question G settles with a new change |
| C5 | sv-skein-retire | sv | | a change an answer made; the merge job alone |
| C6 | fe-world-remove | fe | C4 | an early start that fails; the foreman's `finish`; a Left list |
| C7 | fe-demo-gone | fe | C6 | a conflict the merge job cannot settle |
| C8 | fe-world-reset | fe | C7 | an early start refused; a hold passed up to P |
| C9 | pd-run-credential | pd, jig | | a change made from a Left list |

- **B's changes:** R1, a repair of a red along the way; K1, a catch-up
  with main; R2, a repair of a red at the gate. **G's after landing:**
  R3, a repair into main; D, the retire.

```
P
└─ G   implement playground-reset.md            agent, designer's charter; tracked
   ├─ B   the goal branch, into main            change, collecting
   │  ├─ R1, R2  repairs of B's branch          changes into B's branch
   │  └─ K1  a catch-up with main               change into B's branch
   ├─ Dbx, Dfe  designers                       agents, designer's charter, narrowed
   ├─ C1…C9                                     changes into B's branch
   │     producers, foremen, merge jobs, resolutions under them
   ├─ R3  repair of a red after landing         change into main
   └─ D   the retire                            change into main
```

## 3. Act 1: the cut

| # | Who | What happens | Shows |
|---|---|---|---|
| 1 | P | starts a goal from the project's template for drafts, its words: "Implement `temper/docs/design/draft/playground-reset.md`." | a goal from a template, with its charter and LLM choice |
| 2 | G | its first run reads the draft, the guides and the designs it touches, and asks four inspect-only sub-agents at high, side by side, what the code does today (forges as records, the playground, deleting a project, the box's groups) | surveys as sub-agents (N10) |
| 3 | G | writes the plan's notes in the goal's scope: `plan/strategy`, `plan/status`, `plan/common`, a `contract/<name>` for each shared shape, and `spec/sv-skein-setup`, which needs no designer | the plan's text as notes (N12) |
| 4 | G | first batch: B, collecting into main, and the designers Dbx and Dfe, each told its family and the notes to read. G parks. B makes its branch at main's tip and tells G | batches; designers as tasks; the goal branch made (N3) |
| 5 | G | woken by B's word, makes C1 into B's branch, its words copied from its note (N21) | a change into the goal branch |
| 6 | Dfe | finds no contract for how a removal is refused, and asks G in the question form. G adds `contract/removal-refusal` and answers. Dfe surveys the engine's worlds with a sub-agent of its own, writes its specs as notes and finishes; Dbx finishes too. Each result wakes G, which makes the family's batch: C2 and C3; C4, C6, C7 and C8, with their dependencies | a designer's question; one batch per family |

## 4. Act 2: changes into the goal branch

| # | Who | What happens | Shows |
|---|---|---|---|
| 7 | C1 | its producer writes the code and its tests, runs `.temper/pre-pr`, pushes, finishes: the head and its report, Left empty. C1 opens its pull request into B's branch, needs no check and no update, is first, merges at its head. Landed and done; its result wakes G, which has nothing to do | the plain way (N6) |
| 8 | C3 | depends on C2, whose producer has pushed a first head: C3 starts kept, its producer from B's tip with C2's head merged in | an early start (N2, N13) |
| 9 | C2 | its producer asks, options given: "may bx-doc add a read-only accessor in `temper-forge-setup/src/problem.rs`, outside its files (A, recommended), or should bx-steps supply it (B)?" C2's foreman: the words do not authorise it: escalate. C2 asks G; G amends C2's words with the file and answers A. C2 relays it; its producer goes on | a question through the foreman to G; words widened (N1) |
| 10 | C3 | its producer finishes done while C2 is still producing: C3 is kept, its merge waits. C2 lands; C3 is ready and merges | a merge held for its dependency (jig's `domain/tasks.md`, 5.7) |
| 11 | C4 | its producer asks for a skein API no change makes. Its foreman escalates; G makes C5, sv-skein-retire, adds it as C4's dependency, and answers that C4 waits for it. C4 holds the answer until C5 has landed; its producer then goes on, B's tip merged in first | a new change from an answer; a dependency added (N1, N2) |
| 12 | C5 | the forge refuses its merge into B's branch: a conflict in `scripts/old-forms.txt` alone, which C1 lowered too. The merge job merges B's tip into C5's branch, recounts the baseline and pushes; C5 merges | the merge job (N6, N7) |
| 13 | C6 | starts kept on C4's pushed head; its producer fails: C4's head still holds stubs. By the once rule the next producer waits for C4 to land | an early start that fails (N2) |
| 14 | C6 | its next producer, from B's tip, writes the four removal stories; two fail in jig's run code (a stale credential), outside its files. It asks; its foreman answers `finish`: the stories hold, the failures are Left. It marks the two as ignored, the fault their reason, pushes and finishes with a Left list naming it; C6 merges | the foreman settles; a Left list (N1, N14) |
| 15 | C7 | its merge into B's branch conflicts in nine files C6 and C1 changed. The merge job leaves them; a resolution, an agent in that merge in progress, resolves them and runs the checks; the worker commits the merge and pushes it; C7 merges | a resolution (N6, N7) |
| 16 | C8 | would start early on C7's pushed head, but that head conflicts with B's tip: the early start is refused, and C8 waits for C7 to land | an early start refused (N2) |
| 17 | C8 | its producer fails for good: "project isolation for the playground account needs an upstream fix". C8 is held at producing, escalated past it to G. Relying on disjoint model policies instead changes what the draft promises, so G passes the escalation up to P with its reasons. P amends C8's acceptance ("disjoint policies, as the draft's 6.1 says") and releases it; a fresh producer finishes; C8 merges | a hold passed up to the person (jig's `domain/tasks.md`, section 9) |
| 18 | G | C6's result woke it at step 14: its Left list names jig's run credential, owned by no change. G makes C9, pd-run-credential, its words the fault and the two ignored stories to restore; it lands | a change from a Left list (N14) |

## 5. Act 3: along the way

| # | Who | What happens | Shows |
|---|---|---|---|
| 19 | tier | the goal branch's tier, the gate without its slowest suites, runs at B's tip after step 12: three tests fail. Rerun alone, one passes: a flake. B makes R1, a repair into its branch, its words the two failures, the log's end, and C6 to C8 with what they own. One failure lies in C7's files: R1 leaves it. Changes keep merging into B's branch meanwhile | checked along the way; one repair (N5) |
| 20 | R1 | its producer asks whether the other failure, a stalled chat in the service's workers, is its own. Its foreman escalates; R1 asks B, which relays to G; G answers: "it is yours: no pending change owns those files". R1 repairs it and merges; the tier's next run is green | a repair's question relayed through B (N1) |
| 21 | B | main has moved (another goal landed in `temper-service`), and the span has passed: B makes K1, a catch-up into its branch. The merge job merges main into a branch at B's tip and meets a conflict in `workers.rs` it cannot settle; a resolution settles it; K1 merges into B's branch | catching up along the way (N4, N7) |

## 6. Act 4: landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 22 | G | woken by C9's result, its last change: every change landed, every Left list made into a change, the tier's last run green. G closes B by amending `collect` | closing (N3) |
| 23 | B | drained, opens its pull request into main; CI, the full gate, fails one fuzzy test at its head. B makes R2, a repair into its branch, given CI's failing step and its log's end; R2 merges; CI passes at the new head | the goal checked whole; a repair at the gate (N3) |
| 24 | B | queued, then first in main's queue. A chat's small fix landed meanwhile, so B lacks main's tip: the forge updates it cleanly, CI passes at the new head, and B merges at exactly that head, a merge commit. Landed, then verifying | the queue; a lazy update; a merge at its head (`forge.md`, sections 9 and 10) |

## 7. Act 5: after landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 25 | tier | main's browser tier runs at the new tip; two tests fail. Rerun alone, both fail; neither failed before; B's landing is the only one in the range, so they are its | a tier following main; a verdict (`after-landing.md`, section 5) |
| 26 | G | wakes on the red as news: the tier, the tests, the log's end. It triages: the playground page still names the removed demo journey, in files C7 owned. It makes R3, a change into main naming B's red landing, its words the two tests and the cause | a red to its goal; triage (`after-landing.md`, section 7; N21) |
| 27 | R3 | its producer fixes the page; R3 lands through main's queue; the tier's next run passes both tests. B's landing is cleared, repaired by R3; B gives its result | cleared by a repair (`after-landing.md`, 5.3 and 6) |
| 28 | G | woken by B's result, makes D, the retire: a change into main whose words say which status files and Later sections to write, its producer under the designer's charter. D lands and is cleared at once, every tier ignoring documents. Its result wakes G, which reports; the goal is done | the retire (N15, N21) |

## 8. What each primitive did

| Primitive or mechanism | Steps |
|---|---|
| a goal, tracked, from a template with its LLM choice | 1 |
| sub-agents within a run, at the LLM their call names | 2, 6 |
| notes in the goal's scope, written by G and designers, read by the subtree | 3, 6, 9, 14 |
| batches with dependencies, one per wake | 4, 5, 6 |
| a designer's question in the question form, answered by a note and an answer | 6 |
| a dependency added to a live task | 11 |
| the change procedure into a goal branch, light rules | 7 to 18 |
| a kept change, its merge refused until its dependency lands | 8, 10 |
| a producer's early start, refused, failed, once | 8, 13, 16 |
| a question relayed by a change, its asker kept | 9, 11, 14, 20 |
| a foreman's verdict: escalate, finish | 9, 11, 14, 20 |
| an answer, an amendment, a new change from a question | 9, 11 |
| a hold escalated past a procedure, passed up to the person | 17 |
| a result's Left list read at the coordinator's wake | 14, 18 |
| the merge job, a resolution in a merge in progress | 12, 15, 21 |
| a goal branch's tier, its repair, merges going on | 19, 20 |
| a catch-up change into the goal branch | 21 |
| a collecting change closed by amendment | 22 |
| CI at a head, a repair, the queue, a lazy update, a merge at a head | 23, 24 |
| tiers following main, a rerun, a verdict, a red as news | 25, 26 |
| a change naming the red it repairs; a change living to cleared | 26, 27, 28 |

## 9. What it rests on

Every step from 4 on runs in or around B, so each needs the collecting
change (N3), and every change G makes needs its `change` with all their
parameters (N21) and its producers' LLM choices (N18). Beyond those:

- **Built, configuration alone:** the goal and its template (1); notes
  (3, 6); a designer's question and a person's decision on a hold passed
  up (6, 17); a merge held while kept (10); CI at a head, the queue, the
  update and the merge (24).
- **Designed, not built:** a change that starts kept (8); mid-run
  delivery (8, N13); jobs, for the merge job (12, 15, 21); after landing
  (25 to 27).
- **Changes to designs** (`reference-notes.md`): N1, a change's questions
  relayed (9, 11, 14, 20); N2, early starts (8, 11, 13, 16); N4, the
  catch-up (21); N5, the goal branch's tier, main's tiers following main
  (19, 20, 25); N6, the light rules (7, 12, 15); N7, the merge job (12,
  15, 21); N10, sub-agents on Claude Code (2, 6); N12, the notes' bounds
  (3); N14, the Left list (14, 18); N15, the retire from words (28); N22,
  the coordinator's wake policy (every wake of G).
