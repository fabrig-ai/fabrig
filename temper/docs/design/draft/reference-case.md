# Reference case

Draft, 2026-11-02. One small goal through the reference, from a draft to
cleared, showing each mechanism of `reference-code-production.md` and
`reference-landing.md` once. It is condensed from a real run of the
`playground-reset.md` draft in this directory. It keeps that run's
names and kinds of event and drops its repetition: 64 changes become
nine, 54 questions four, and nine checks along the way one.
`reference-notes.md`, 2.3 maps each step to the run, and its N numbers
are the design changes each step needs.

## 1. In one page

- **The goal:** P, a person, starts a goal from the draft. G, its
  coordinator, cuts it into nine changes in four families, released in
  three batches into a goal branch, B.
- **Act 1, the cut** (section 3): surveys and designers as sub-agents,
  contracts first, and three batches as soon as each part is ready.
- **Act 2, changes into the goal branch** (section 4): the plain way; a
  question settled by the foreman, by G and by P; a new change made by
  an answer; early starts, a merge held for its dependency, an early
  start refused; a Left list; a hold; conflicts settled by the resolver
  and by an agent.
- **Act 3, along the way** (section 5): a red of the goal branch's tier
  repaired while merges go on; a catch-up with main.
- **Act 4, landing** (section 6): B closed, checked whole, repaired once,
  updated and merged at its head.
- **Act 5, after landing** (section 7): a red of main's tier, triaged
  and repaired by a change naming it; B cleared; the retire.

## 2. The cast

- **P,** the person who wrote the draft.
- **G,** the goal: an agent task under the designer's charter, Opus 5.5
  at xhigh, its sub-agents Opus 5.5 at xhigh and high; tracked.
- **B,** the goal branch: the forge's change into main, collecting, its
  branch `temper/<root>/g<B>` (N3).
- **N,** the plan's notes: a change into B's branch, handed G's run
  branch, carrying `notes/common.md` and `notes/contracts.md`.
- **The changes,** each into B's branch, its words its spec, its
  producer a coding agent at gpt-6.1-sol:

| | Change | Family | After | Shows |
|---|---|---|---|---|
| C1 | sv-skein-setup | sv, skein and service | N | the plain way |
| C2 | bx-doc | bx, the box | N | a question G settles by widening the words |
| C3 | bx-init | bx | C2 | an early start; a merge held for its dependency |
| C4 | fe-remove | fe, the forge's engine | N | a question G settles with a new change |
| C5 | sv-skein-retire | sv | N | a change an answer made; the resolver alone |
| C6 | fe-world-remove | fe | C4 | an early start that fails; the foreman's `finish`; a Left list |
| C7 | fe-demo-gone | fe | C6 | a conflict the resolver cannot settle |
| C8 | fe-world-reset | fe | C7 | an early start refused; a hold P decides |
| C9 | pd-run-credential | pd, jig | N | a change made from a Left list |

- **R1,** a repair of B's branch, made by B (N5); **R2,** a repair into
  main after landing; **D,** the retire.

```
P
└─ G   implement playground-reset.md            agent, designer's charter; tracked
   ├─ B   the goal branch, into main            change, collecting
   │  └─ R1  repair of B's branch               change into B's branch
   ├─ N   the plan's notes                      change into B's branch
   ├─ C1…C9                                     changes into B's branch
   │     producers, foremen, the resolver, resolutions under them
   ├─ R2  repair of a red after landing         change into main
   └─ D   the retire                            change into main
```

## 3. Act 1: the cut

| # | Who | What happens | Shows |
|---|---|---|---|
| 1 | P | starts a goal from the project's template for drafts, words: "Implement `temper/docs/design/draft/playground-reset.md`." | a goal, its template's charter and LLM choice (`reference-code-production.md`, section 4) |
| 2 | G | its first run reads the draft, the guides and the designs it touches; asks four inspect-only sub-agents at high, side by side, what the code does today (forges as records, the playground, deleting a project, the box's groups); writes their findings into the status note | surveys as sub-agents (N10) |
| 3 | G | writes `notes/common.md` (what every change may do without asking, what is not its own, how it reports) and `notes/contracts.md` (the shared shapes, each name exact) in its workspace; puts the families and the critical path in its notes | contracts first (N12) |
| 4 | G | first batch: B, collecting into main; N, handed its run branch; C1, which needs no designer, after N. N merges into B's branch at once; C1's producer starts from B's tip | early release; the goal branch; notes landed first (N3, N12) |
| 5 | G | asks one designer sub-agent per family at xhigh, side by side, each briefed with the contracts and the surveys; the bx designer asks for one contract more, which G adds to the bx batch's words. Second batch, as the designers answer: C2, C3, C4, C6, C7, C8, with their dependencies. G's run ends; G parks | designers as sub-agents; batches; dependencies (N10) |

## 4. Act 2: changes into the goal branch

| # | Who | What happens | Shows |
|---|---|---|---|
| 6 | C1 | its producer writes the code and its tests, runs `.temper/pre-pr`, pushes, finishes: the head and its report, Left empty. C1 opens its pull request into B's branch, needs no check, is first, merges at its head. Landed and done; its result waits in G's inbox | the plain way (N6) |
| 7 | C3 | depends on C2, whose producer has pushed a first head: C3 starts kept, its producer from B's tip with C2's head merged in | an early start (N2) |
| 8 | C2 | its producer asks, options given: "may bx-doc add a read-only accessor in `temper-forge-setup/src/problem.rs`, outside its files (A, recommended), or should bx-steps supply it (B)?" C2's foreman: the words do not authorise it: escalate. C2 asks G; G wakes, amends C2's words with the file and answers A. C2 relays it; its producer goes on | a question through the foreman to G; words widened (N1) |
| 9 | C3 | its producer finishes done while C2 is still producing: C3 is kept, its merge waits. C2 lands; C3 is ready and merges | a merge held for its dependency (jig's `domain/tasks.md`, 5.7) |
| 10 | C4 | its producer asks for a skein API no change makes. Its foreman escalates; G makes C5, sv-skein-retire, adds it as C4's dependency, and answers that C4 waits for it. C5 lands; C4's producer goes on, B's tip merged in first | a new change from an answer; a dependency added (N1, N2) |
| 11 | C5 | its merge into B's branch conflicts only in `scripts/old-forms.txt`, which C1 lowered too. The resolver job recounts the baseline, its delivery the merge | the resolver (N7) |
| 12 | C6 | starts kept on C4's pushed head; its producer fails: C4's head still holds stubs. By the once rule the next producer waits for C4 to land | an early start that fails (N2) |
| 13 | C6 | its next producer, from B's tip, writes the four removal stories; two fail in jig's run code (a stale credential), outside its files. It asks; its foreman answers `finish`: the stories hold, the failures are Left. It pushes and finishes with a Left list naming them; C6 merges | the foreman settles; a Left list (N1, N14) |
| 14 | C7 | its merge conflicts in nine files that C6 and C1 changed. The resolver leaves them; a resolution, an agent in a merge in progress, resolves them and runs the checks; the worker commits the merge and pushes it; C7 merges | a resolution (`forge.md`, 8.4) |
| 15 | C8 | would start early on C7's pushed head, but it conflicts with B's tip: the early start is refused, and C8 waits for C7 to land | an early start refused (N2) |
| 16 | C8 | its producer fails for good: "project isolation for the playground account needs an upstream fix". C8 is held at producing, escalated past it to G. G reads the draft: whether to rely on disjoint model policies changes what the draft promises, so G asks P. P answers in the inbox: "disjoint policies, as the draft's 6.1 says". G amends C8's acceptance and releases it; a fresh producer finishes; C8 merges | a hold; a question to the person (jig's `domain/tasks.md`, section 9) |
| 17 | G | woken by B's word that nothing into its branch is live, reads the results waiting: C6's Left list names jig's run credential, owned by no change. G makes C9, pd-run-credential, its words the failing stories; it lands | a change from a Left list (N14) |

## 5. Act 3: along the way

| # | Who | What happens | Shows |
|---|---|---|---|
| 18 | tier | B's branch's tier, the gate without its slowest suites, runs at B's tip after step 11: three tests fail. Rerun alone, one passes: a flake. B makes R1, a repair into its branch, its words the two failures, the log's end, and C6 to C8 with what they own. One failure lies in C7's files: R1 leaves it. Changes keep merging into B's branch meanwhile | checked along the way; one repair (N5) |
| 19 | R1 | its producer asks whether the other failure, a stalled chat in the service's workers, is its own. Its foreman escalates; R1 asks B, which relays to G; G answers: "it is yours: no pending change owns those files". R1 repairs it and merges; the tier's next run is green | a repair's question relayed through B (N1) |
| 20 | B | main has moved (another goal landed in `temper-service`), and the span has passed: B updates, the forge merging main into its branch, and meets a conflict in `workers.rs`. The resolver leaves it; B's resolution settles it, and the merges into B's branch wait for its writer slot meanwhile | catching up along the way (N4) |

## 6. Act 4: landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 21 | G | woken again by B once C9 has landed: every change landed, every Left list made into a change, the tier's last run green. G closes B by amending `collect` | closing (N3) |
| 22 | B | opens its pull request into main; CI, the full gate, fails one fuzzy test at its head. A repair, an agent on B's branch given CI's failing step and its log's end, pushes a new head; CI passes | the goal checked whole; a repair at the gate (`forge.md`, 8.4) |
| 23 | B | queued, then first in main's queue. A person's small fix landed meanwhile, so B lacks main's tip: B is updated cleanly, CI passes at the new head, and B merges at exactly that head, a merge commit. Landed, then verifying | the queue; a lazy update; a merge at its head (`forge.md`, sections 9 and 10) |

## 7. Act 5: after landing

| # | Who | What happens | Shows |
|---|---|---|---|
| 24 | tier | main's browser tier runs at the new tip; two tests fail. Rerun alone, both fail; neither failed before; B's landing is the only one in the range, so they are its | a tier following main; a verdict (`after-landing.md`, section 5) |
| 25 | G | wakes on the red as news: the tier, the tests, the log's end. It triages: the playground page still names the removed demo journey, in files C7 owned. It makes R2, a change into main naming B's red landing, its words the two tests and the cause | a red to its goal; triage (`after-landing.md`, section 7) |
| 26 | R2 | its producer fixes the page; R2 lands through main's queue; the tier's next run passes both tests. B's landing is cleared, repaired by R2; B gives its result | cleared by a repair (`after-landing.md`, 5.3 and 6) |
| 27 | G | its last run writes the status files beside the designs the goal touched and the designs' Later sections, and hands its run branch to D, a change into main. D lands; G reports, and the goal is done | the retire (N15) |

## 8. What each primitive did

| Primitive or mechanism | Steps |
|---|---|
| a goal, tracked, from a template with its LLM choice | 1 |
| sub-agents within a run, each naming its LLM | 2, 5 |
| batches with dependencies, several from one run | 4, 5 |
| a dependency added to a live task | 10 |
| the change procedure into a goal branch, light rules | 6 to 17 |
| a kept change, its merge refused until its dependency lands | 7, 9 |
| a producer's early start, refused, failed, once | 7, 12, 15 |
| a question in the question form, its asker kept | 8, 10, 13, 19 |
| a foreman's verdict: escalate, finish | 8, 10, 13, 19 |
| an answer, an amendment, a new change from a question | 8, 10, 16 |
| a hold escalated past a procedure to the coordinator | 16 |
| a question from the coordinator to its person, answered in the inbox | 16 |
| a result's Left list read at the coordinator's next wake | 17 |
| the resolver job, a resolution in a merge in progress | 11, 14, 20 |
| a goal branch's tier, its repair, merges going on | 18, 19 |
| a goal branch's update along the way | 20 |
| a collecting change closed by amendment | 21 |
| CI at a head, a repair, the queue, a lazy update, a merge at a head | 22, 23 |
| tiers following main, a rerun, a verdict, a red as news | 24, 25 |
| a change naming the red it repairs; a change living to cleared | 26 |
| a run branch handed to a change | 4, 27 |

## 9. What it rests on

Every step from 4 on runs in or around B, so each needs the collecting
change (N3) and the light rules of goal branches (N6). Beyond those:

- **Built, configuration alone:** the goal and its template (1); a
  producer's failure holding its change, the escalation past it, and the
  coordinator's question to its person (16); a resolution (14); CI at a
  head, a repair, the queue, the update and the merge (22, 23).
- **Designed, not built:** a change that starts kept (7, 9); jobs, for
  the resolver (11, 14, 20); after landing (24 to 26).
- **Changes to designs** (`reference-notes.md`): N1, a change's questions
  relayed (8, 10, 13, 19); N2, early starts (7, 10, 12, 15); N4, the
  catch-up (20); N5, the goal branch's tier (18, 19); N7, the resolver
  (11, 14, 20); N10, sub-agents on Claude Code (2, 5); N12, notes landed
  first (3, 4); N14, the Left list (13, 17); N15, the run branch made
  again (27); N18, each producer's LLM choice (every change).
