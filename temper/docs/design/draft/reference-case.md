# Reference case

Draft, 2026-11-02. One small goal through the reference, from a draft to
cleared, showing each mechanism of `reference-code-production.md` and
`archive/reference-landing.md` once. It is condensed from a real run of the
`archive/playground-reset.md` draft in this directory. It keeps that run's names
and kinds of event and drops its repetition: 64 changes become nine,
five designers two, 54 questions five, and nine checks along the way
one. `notes.md`'s N numbers are the choices each step rests on.

## 1. In one page

- **The goal:** P, a person, starts a goal from the draft; its branch g
  is made with it, and G, its coordinator, holds it. Everything the goal
  writes lands into g: a shared design, two families' designs, nine
  changes in four families, a design revision, a repair, a catch-up and
  the close-out. Then g lands into main as one change.
- **Act 1, the cut** (section 3): surveys as sub-agents, the shared
  design and the designers as changes into g, a designer's question,
  batches as each family's design lands.
- **Act 2, changes into g** (section 4): the plain way; questions settled
  by the foreman and by G; a new change made by an answer; the design
  revised under live changes; early starts, a merge held for its
  dependency, an early start refused; a Left list; a hold passed up to P;
  conflicts settled by the merge job and by an agent.
- **Act 3, along the way** (section 5): a red of g's tier, news to G,
  triaged and repaired while merges go on; a catch-up with main.
- **Act 4, landing** (section 6): the close-out into g; L, checked whole,
  repaired once on g, updated and merged at its head.
- **Act 5, after landing** (section 7): a red of main's tier, triaged and
  repaired by a change naming it; L cleared; the goal done.

## 2. The cast

- **P,** the person who wrote the draft.
- **G,** the goal: an agent task under the coordinator's charter, Opus
  5.5 at xhigh, its sub-agents Opus 5.5 at high; tracked; it holds g,
  `temper/<root>/g<root>` (N3).
- **S, Dbx, Dfe,** the shared design and the designs of the box and the
  forge's engine: changes into g whose producers run under the
  designer's charter at G's LLM choice (N13). **S2,** a revision of S's
  design, made later the same way.
- **The changes,** each into g, its words its spec citing g's design,
  its producer a coding agent at gpt-6.1-sol:

| | Change | Family | After | Shows |
|---|---|---|---|---|
| C1 | sv-skein-setup | sv, skein and service | | a change G specifies itself; the plain way |
| C2 | bx-doc | bx, the box | Dbx | a question G settles by widening the words |
| C3 | bx-init | bx | C2 | an early start; a merge held for its dependency |
| C4 | fe-remove | fe, the forge's engine | Dfe | a question G settles with a new change |
| C5 | sv-skein-retire | sv | | a change an answer made; the merge job alone |
| C6 | fe-world-remove | fe | C4 | an early start that fails; the foreman's `finish`; a Left list |
| C7 | fe-demo-gone | fe | C6 | the design revised under it; a conflict the merge job cannot settle |
| C8 | fe-world-reset | fe | C7 | an amendment naming a revision; an early start refused; a hold passed up to P |
| C9 | pd-run-credential | pd, jig | | a change made from a Left list |

- **G's others:** R1, a repair along the way, and K1, a catch-up with
  main, both into g; X, the close-out, into g; L, the goal's landing,
  into main with g as its branch; R2, a repair after landing, into main.

```
P
└─ G   implement archive/playground-reset.md    agent, coordinator's charter; tracked; holds g
   ├─ S, S2  the shared design, its revision    changes into g, designer's charter
   ├─ Dbx, Dfe  the families' designs           changes into g, designer's charter, after S
   ├─ C1…C9                                     changes into g
   │     producers, foremen, merge jobs, resolutions under them
   ├─ R1  repair along the way                  change into g
   ├─ K1  catch-up with main                    change into g, the merge job its producer
   ├─ X   the close-out                         change into g, designer's charter
   ├─ L   the goal's landing                    change into main, its branch g
   └─ R2  repair of a red after landing         change into main
```

## 3. Act 1: the cut

| # | Who | What happens | Shows |
|---|---|---|---|
| 1 | P | starts a goal from the project's template for drafts, its words: "Implement `temper/docs/design/draft/archive/playground-reset.md`." The template names *its own goal branch*, resolved to g; G becomes active holding it, and the forge makes g at main's tip | a goal from a template, with its charter, its LLM choice and its own branch (N3) |
| 2 | G | its first run reads the draft, the guides and the designs it touches, and asks four inspect-only sub-agents at high, side by side, what the code does today (forges as records, the playground, deleting a project, the box's groups) | surveys as sub-agents (N10) |
| 3 | G | shapes the plan: two families need a designer (bx, fe); both refuse removals, so how a removal is refused is a shared shape; sv-skein-setup needs none, skein's design already saying it. First batch: S, its words the refusal's shape and where it goes; Dbx and Dfe, after S, each told its family; C1. G parks | batches; the shared design and designers as changes (N13); a change G specifies itself |
| 4 | S | its producer, under the designer's charter, writes the refusal's section into the forge's design and a status entry for it, and pushes. Dbx and Dfe start kept on S, their producers from S's pushed head (N2). S lands into g; their merges may follow | the design written into g (N12); designers starting early |
| 5 | Dfe | asks G, straight up: "does removing a world wait for its live runs (A) or refuse while one runs (B, recommended, as S's shape refuses)?" G answers B. Dfe surveys the engine's worlds with a sub-agent of its own and goes on | a designer's question with no foreman (N1) |
| 6 | Dbx, Dfe | each writes its family's sections, their status entries and its Later into g, pushes and finishes, its report listing its changes: Dbx's C2 and C3, Dfe's C4, C6, C7 and C8, each with what it owns, its dependencies and its effort. Each lands into g; each result, carrying its report, wakes G, which makes that family's batch | a family's design; its report as the result; one batch per family (N15) |

## 4. Act 2: changes into g

| # | Who | What happens | Shows |
|---|---|---|---|
| 7 | C1 | its producer writes the code and its tests, runs `.temper/pre-pr`, pushes, finishes: the head and its report, Left empty. C1 opens its pull request into g, needs no check and no update, is first, merges at its head. Landed and done; its result wakes G, which has nothing to do | the plain way (N4) |
| 8 | C3 | depends on C2, whose producer has pushed a first head: C3 starts kept, its producer from g's tip with C2's head merged in | an early start (N2, N14) |
| 9 | C2 | its producer asks, options given: "may bx-doc add a read-only accessor in `temper-forge-setup/src/problem.rs`, outside its files (A, recommended), or should bx-steps supply it (B)?" C2's foreman: neither the words nor Dbx's design authorise it: escalate. C2 asks G; G amends C2's words with the file and answers A. C2 relays it; its producer goes on | a question through the foreman to G; words widened (N1) |
| 10 | C3 | its producer finishes done while C2 is still producing: C3 is kept, its merge waits. C2 lands; C3 is ready and merges | a merge held for its dependency (jig's `domain/tasks/lifecycle.md`, 8) |
| 11 | C4 | its producer asks for a skein API no change makes. Its foreman escalates; G makes C5, sv-skein-retire, adds it as C4's dependency, and answers that C4 waits for it. C4 holds the answer until C5 has landed; its producer then goes on, g's tip merged in first | a new change from an answer; a dependency added (N1, N2) |
| 12 | C5 | the forge refuses its merge into g: a conflict in `scripts/old-forms.txt` alone, which C1 lowered too. The merge job merges g's tip into C5's branch, recounts the baseline and pushes; C5 merges | the merge job (N4, N7) |
| 13 | C6 | starts kept on C4's pushed head; its producer fails: C4's head still holds stubs. By the once rule the next producer waits for C4 to land | an early start that fails (N2) |
| 14 | C6 | its next producer, from g's tip, writes the four removal stories; two fail in jig's run code (a stale credential), outside its files. It asks; its foreman answers `finish`: the stories hold, the failures are Left. It marks the two as ignored, the fault their reason, pushes and finishes with a Left list naming it; C6 merges | the foreman settles; a Left list (N1, N15) |
| 15 | C7 | its producer asks: S's refusal names the record that blocks a removal, and the demo journey's removal has none to name. Its foreman: the design does not settle it: escalate. G makes S2, a designer's change revising S's section (a refusal names a record or a reason), adds it as C7's dependency and answers that C7 waits for it; it amends C8, whose words cite the same section, naming S2. S2 lands; C7's answer goes down, and its producer merges g's tip and goes on | the design revised under live changes (N12) |
| 16 | C7 | its merge into g conflicts in nine files C6 and C1 changed. The merge job leaves them; a resolution, an agent in that merge in progress, resolves them and runs the checks; the worker commits the merge and pushes it; C7 merges | a resolution (N4, N7) |
| 17 | C8 | would start early on C7's pushed head, but that head conflicts with g's tip: the early start is refused, and C8 waits for C7 to land | an early start refused (N2) |
| 18 | C8 | its producer fails for good: "project isolation for the playground account needs an upstream fix". C8 is held at producing, escalated past it to G. Relying on disjoint model policies instead changes what the draft promises, so G passes the escalation up to P with its reasons. P amends C8's acceptance ("disjoint policies, as the draft's 6.1 says") and releases it; a fresh producer finishes; C8 merges | a hold passed up to the person (jig's `domain/tasks/proposals.md`) |
| 19 | G | C6's result woke it at step 14: its Left list names jig's run credential, owned by no change. G makes C9, pd-run-credential, its words the fault and the two ignored stories to restore; it lands | a change from a Left list (N15) |

## 5. Act 3: along the way

| # | Who | What happens | Shows |
|---|---|---|---|
| 20 | tier | g's tier, the gate without its slowest suites, runs at g's tip after step 12: three tests fail. Rerun alone, one passes: a flake. The two left reach G as news. G triages from the tree: one failure lies in C7's files, still live, so G amends C7's words to say it is C7's; the other, a stalled chat in the service's workers, no live change owns, so G makes R1, a repair into g, its words the failure, the log's end, and C6 to C8 with what they own. G notes the flaky test in the repository's scope. R1 lands while changes keep merging; the tier's next run is green | checked along the way; a red as news; triage; a repair; a note as jig means it (N5, N12) |
| 21 | G | main has moved: another goal landed in `temper-service`, which g's open work touches. Woken by the landing (paced), G makes K1, a catch-up into g. The merge job merges main into a branch at g's tip and meets a conflict in `workers.rs` it cannot settle; a resolution settles it; K1 lands into g | catching up along the way (N6, N7) |

## 6. Act 4: landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 22 | G | woken by C9's result, its last change: every change landed, every Left list made into a change, the tier's last run green. G makes X, the close-out: its producer, under the designer's charter, writes into g the status entries still open (project isolation upstream, from C8), the designs' Later sections, and folds the draft into the forge's design, removing it. X lands into g | the close-out (N16) |
| 23 | L | X's result wakes G, which makes L, a change into main whose branch is g. L opens its pull request; CI, the full gate, fails one fuzzy test at its head. L's repair, an agent at g's tip, fixes it and pushes g under G's hold; CI passes at the new head | the goal checked whole; a repair on g (N3) |
| 24 | L | queued, then first in main's queue. A chat's small fix landed meanwhile, so L lacks main's tip: the forge merges main into g cleanly, CI passes at the new head, and L merges at exactly that head, a merge commit. Landed, then verifying | the queue; a lazy update; a merge at its head (`../domain/forge/queues.md` and `../domain/forge/README.md`, section 6) |

## 7. Act 5: after landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 25 | tier | main's browser tier runs at the new tip; two tests fail. Rerun alone, both fail; neither failed before; L's landing is the only one in the range, so they are its | a tier following main; a verdict (`after-landing.md`, section 5) |
| 26 | G | wakes on the red as news: the tier, the tests, the log's end. It triages: the playground page still names the removed demo journey, in files C7 owned. It makes R2, a change into main naming L's red landing, its words the two tests and the cause | a red to its goal; triage (`after-landing.md`, section 7; N22) |
| 27 | R2 | its producer fixes the page; R2 lands through main's queue; the tier's next run passes both tests. L's landing is cleared, repaired by R2; L gives its result | cleared by a repair (`after-landing.md`, 5.3 and 6) |
| 28 | G | woken by L's result, reports: the goal is done. It closes, and its hold's release deletes g, landed | the goal's end; its branch released (N3) |

## 8. What each primitive did

| Primitive or mechanism | Steps |
|---|---|
| a goal, tracked, from a template with its LLM choice | 1 |
| a goal's own branch, held by it, every change of the goal landing into it | 1, 4, 6, 7 to 22, 28 |
| sub-agents within a run, at the LLM their call names | 2, 5 |
| batches with dependencies, one per wake | 3, 6 |
| a designer as a change's producer, its design landed into g, its report the family's changes | 3, 4, 6, 15, 22 |
| a change's words citing the design in g | 6, 9, 15 |
| a dependency added to a live task | 11, 15 |
| the change procedure into a goal's branch, light rules | 4, 6, 7 to 22 |
| a kept change, its merge refused until its dependency lands | 4, 8, 10 |
| a producer's early start, refused, failed, once | 8, 13, 17 |
| a question relayed by a change, its asker kept; a designer's straight up | 5, 9, 11, 14, 15 |
| a foreman's verdict: escalate, finish | 9, 11, 14, 15 |
| an answer, an amendment, a new change from a question | 9, 11, 15 |
| the design revised, live changes amended naming the revision | 15 |
| a hold escalated past a procedure, passed up to the person | 18 |
| a result carrying its producer's report and Left list | 6, 14, 19 |
| the merge job, a resolution in a merge in progress | 12, 16, 21 |
| a tier following g, its red as news, triage, a repair, merges going on | 20 |
| a note of what a run learned | 20 |
| a catch-up change into g | 21 |
| the close-out into g | 22 |
| a change on its requester's held branch; CI at a head, a repair, the queue, a lazy update, a merge at a head | 23, 24 |
| tiers following main, a rerun, a verdict, a red as news | 25, 26 |
| a change naming the red it repairs; a change living to cleared | 26, 27 |
| a hold released as its goal closes | 28 |

## 9. What it rests on

Every step runs in or around g, so each rests on the goal's own branch
(N3), and every change G makes on its `change` with all their
parameters (N22) and their agents' LLM choices (N19). Where the design
still lags each step is `reference-case.status.md`.
