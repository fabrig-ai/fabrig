# The reference case

Draft, 2026-11-01, fourth edition, not adopted until the user approves
it. One invented story, as small as it can be while it runs every
mechanism of `landing-model.md` (revision 5) under fabrig's
configuration (its section 14), with fab's cast, so that someone who
knows fab recognises each role and step. It is what temper and jig must
run to replace fab. Section 9 maps the story onto fab and the model;
section 11 says, mechanism by mechanism, whether temper and jig provide
it as a generic mechanism, provide it fixed, or lack it: the input to
the execution plan. Designs citing the third edition are repointed on
adoption.

## 1. In one page

- **One project, Harbor,** one repository on an owned Forgejo.
- **Ines and her chat,** which reads the inbox and does the hand work
  fab leaves to sessions; every other role is the engine's (3.2).
- **Three goals:** Export and Themes, from two drafts, each on a goal
  branch, and the ladder, Harbor's QA.
- **Five acts, 26 steps, about 10 hours,** overlapping in time: the
  start (section 4), the goal branches (section 5), landing (section 6),
  after landing (section 7), and QA (section 8).

## 2. How to read it

Each step says what happens, then **Model:**, the model's sections it
shows. Times are hours from the start. On fakes, timers and bounds are
scaled down and each result is scripted as the oracle; on real backends,
a QA driver types Ines's words.

## 3. The project, the cast and the start

### 3.1 Harbor

- **The repository,** `harbor`:
  - packages: `kit`; `ui`, which reads kit; `app`, which reads kit and
    ui;
  - `debts.txt`, a counted baseline that only falls, recounted by
    `tools/count`;
  - `docs/`, with the drafts and each goal's plan.
- **The gates:** formatting, lints, the build, and the `default` and
  `fuzzy` tests: a change's check before ready runs them in its
  workspace, the landing queue in its gate lane, the final check whole.
- **The tiers:** `browser-app` (CPU recorded), and `live-app`, which
  needs a ChatGPT account.
- **The machines:** the hub, which runs the engine, the agent tasks, the
  inbox, custody and the status page, and builds nothing; workers B1
  (four slots, five workspaces), B2 (four slots, four workspaces) and G,
  which holds the gate lane and the tier runner (two slots).
- **Custody:** a Codex and a Claude account lent to each machine;
  codex-9 and chatgpt-1 free.
- **The settings** are fabrig's (model section 14).

### 3.2 The cast

Ines and her chat; the engine; and its agent tasks and runs, each as fab
has it: the planner, the producers (Codex runs in workspaces), the
examiners, the answerer, the reviewer, the reviser and the arbiter, the
scaffolds, the structure review, the closing task, the triager, the
reply task, the landing queue on G and the tier runner. The walker is an
agent of Ines's chat, as fab's walks are still walked by hand. Section 9
maps each onto fab.

### 3.3 At hour 0

Main is green in every tier; no goal is listed. The ladder's plan is on
main with rungs r0 and r1 (r1 after r0), each on ChatGPT with the owned
Forgejo, sharing `kit.md`.

## 4. Act 1: the start (hours 0 to 1)

### 4.1 Ines starts Export and Themes

Ines starts Export (`docs/drafts/export.md`) with fabrig's settings: a
goal branch, ready, review off, scaffold off, early starts on. Then
Themes, with `review` and `scaffold` on. Each start runs a planner and
lists the goal.

- **Model:** 3, 4.1, 4.4, 10.1.

### 4.2 Export's planner releases early

The planner writes the export record's shape first, then pushes the plan
with three changes that depend on nothing: e1, the record in kit; e2,
the CSV writer in app; e3, the menu entry in ui. It wakes the engine,
which merges the push into Export's goal branch; e1 to e3 start at once,
across B1 and B2 (most free slots first), while the planner writes e4 to
e9 and pushes them in two more batches. Its brief listed Themes' changes
and paths; its plan names the overlap it could not avoid: both goals
touch ui's menu.

- **Model:** 4.2, 4.3, 5.1, 6.1, 6.6.

### 4.3 Themes' planner, reviewed, and a part kept

Themes' planner writes two scaffolded parts, palette and dark, and a
question for Ines: does dark mode follow the system? It names part dark.
The reviewer refuses the plan's branch (a spec names a package that does
not exist); the reviser fixes it, the reviewer approves, and the plan
merges into Themes' goal branch. Dark is not designed while the question
is open. Ines answers "follow the system"; her chat writes the answer
into the plan.

- **Model:** 4.2, 10.2, 10.5, 11.1.

### 4.4 Palette's scaffold: the forge, an arbiter, a rebase

Palette's scaffold ends with a question: keep the old colour names as
aliases? The subject is kept, and an issue opens on the forge. Ines
comments "what do the aliases cost?"; the reply task answers with the
two options and a recommendation. Ines comments "drop them"; the next
reply task records it as the answer, and the issue closes. The scaffold
runs again with the answer in its brief.

It writes palette's types with empty bodies and the part's specs. The
engine checks its tip (gates and tests), and t1 to t3 start from it
while the reviewer reads it. The reviewer refuses it (a name it
dislikes); the reviser renames it; the reviewer refuses again (a doc
comment it calls missing), which holds it, and the arbiter says land:
the remark is style, left for later. It merges. t1 to t3, finished from
the first tip, each get a rebase onto the revised one, outside their
tries, and then merge.

Dark's scaffold, approved at once, conflicts with palette's when it
merges into the goal branch: that rejection counts as a refusal, with no
revision yet, so the reviser rebases it, the reviewer approves it again,
and it merges.

- **Model:** 10.1, 10.2, 10.5, 10.6, 11.1, 11.3.

## 5. Act 2: the goal branches (hours 1 to 4)

### 5.1 As wide as the slots

Each goal's target is the workers' slots, ten. Each run starts in a
claimed workspace from its goal branch's tip, on a compiled brief;
before each start, the worker's code moves to the engine's version. B1
is calm (low load, no disk waits), so it takes e9 as a fifth run in its
fifth workspace. Later, e7's start, aimed at B2, finds its last free
workspace claimed by hand by Ines's chat, and hands on to B1, which has
a free slot.

- **Model:** 5.1, 5.2, 12.1.

### 5.2 A question the options settle, and early starts

e2 asks whether a quote in a field is doubled or escaped; its spec's
options say doubled. An examiner answers; the answer joins e2's spec,
and e2 runs again from its pushed branch.

e4, the export command in app, depends on e2. While e2 runs again with
its branch pushed, e4 starts early: its workspace is the goal branch
with e2's branch merged in, and its brief names e2 to merge again before
it pushes. e4 merges only after e2. e5 depends on e3, but e3's branch
and the goal branch both lowered `debts.txt`, and an early start's merge
recounts nothing: e5's early start is refused, and it waits for e3 to
merge.

- **Model:** 5.1, 6.2, 10.3.

### 5.3 Finished with something left, and merges

e1 finishes, leaving a lint in ui that its type change caused outside
its paths. e1 and e3 merge; both lowered `debts.txt`, so the second
merge conflicts there, and the engine takes the goal branch's side and
recounts. No one hears of it. e3's merge makes e5 runnable, so the round
that made it wakes itself to start it.

- **Model:** 3, 5.1, 6.1.

### 5.4 A merge conflict, a disk wait, a failure

- **e6 and e5** both change app's command table. e5 merges first; e6's
  merge conflicts and is a rejection. An examiner writes a rebase
  section, and a rebase resolves it.
- **e7's** run waits on its worker's disk: the build reclaims idle
  workspaces, waits 20 minutes, and fails as transient. The engine
  starts it again, spending no try; it finishes.
- **e8's** run fails (its fixture generator crashes). It is kept for the
  answerer at once, which says the generator is e9's and to stub the
  fixture; e8 runs again and finishes.

- **Model:** 5.1, 6.1, 10.3, 10.4, 10.5, 12.1.

### 5.5 Questions outside the change

e9 asks whether it may change ui's theme hook, which Themes owns. The
spec's options do not settle it, so the examiner queues it, and the
answerer runs at once (a goal branch's): leave ui to its owner and list
it as left. e9's next run pushes its work but asks again, about a ui
test it cannot make green. The examiner says accept: the change is
finished, its branch kept, and it merges.

- **Model:** 10.3, 10.4.

### 5.6 An interim check, and its repair

At the tenth merge, the engine checks Export's goal branch in an idle
workspace, without the fuzzy tests: the default tests are red in app.
`repair-1` is made from the records, its words the failing gates and the
unmerged changes' paths. Its run asks which of two failing tests is
right; with no written spec, the question queues for the answerer, whose
answer joins repair-1's words. The next run finishes and merges.

- **Model:** 4.3, 6.3, 10.3, 10.4.

### 5.7 Goals beside each other

Each round the overlap probe merges both goal branches with main in
memory, then with each other: ui's menu conflicts. It records the
overlap once, and the status page shows Export and Themes with a count
of one. Nothing waits on it.

- **Model:** 6.6, 13.

### 5.8 Themes is ready first

Four hours in, Themes' structure review reads the digests of the changes
merged so far and writes one consolidation change, merging two colour
helpers; it starts ahead of Themes' other changes. Themes' last change
merges; the engine updates the goal branch from main (main has not
moved), runs the final check, green, and readies it.

- **Model:** 5.2, 6.4, 6.5, 10.2.

## 6. Act 3: landing (hours 3 to 6)

### 6.1 A candidate with a hand-pushed branch, bisected

Ines's chat readies `chat/readme` by hand, a fix to a doc comment in
`ui/src`. The landing queue merges it and Themes' goal branch onto main
as one candidate: the lint gate is red. The candidate is halved, left
first: Themes' goal branch alone is green and lands. The chat's branch
alone is red and is rejected. No goal owns it, so an alert opens in the
inbox with its gate and log. The chat fixes the comment and pushes a
newer tip; when the queue tries it, the alert clears, and it lands.

- **Model:** 7.1, 7.4, 7.5, 11.2.

### 6.2 Updates, a resolution, a stop

Main has moved under Export. Thirty minutes after its last update, the
engine merges main into Export's goal branch: ui's menu conflicts, as
the probe said. A resolution merges that main into a branch from the
goal branch, keeping both sides' intent, fixes the app code Themes'
palette broke, and merges; the engine checks the goal branch now
contains that main and records a clean update. The overlap closes,
Themes having landed.

Main moves again, and the next update conflicts in a lock file. The
second resolution merges, but its run had reset its branch, so the goal
branch still lacks that main: a refusal at a main whose resolution has
merged, and the goal stops. Ines's chat merges main by hand; the next
round records a clean update, and the stop clears.

- **Model:** 6.4, 6.5, 6.6.

### 6.3 Export's final check, red, then ready

The last change merges. The final check is red: a fuzzy test in app.
`repair-2` fixes it; the check runs again, green, and the goal branch is
ready.

- **Model:** 6.5.

### 6.4 The gate's error, then a rejection

The candidate holding Export's goal branch stops at the build: G's disk
wait ran out. It is an error: nobody is rejected, and it goes again. Its
next attempt is red: a ladder repair (8.2) that landed meanwhile changed
kit's error type, and Export's app code no longer builds on it. The
engine updates the goal branch, checks it (red, `repair-3`, green), and
readies it again. It lands.

- **Model:** 6.5, 7.1, 7.5.

## 7. Act 4: after landing (hours 4 to 8)

### 7.1 Themes' red, a flake at the gate, unfixed

The tier runner runs Themes' landing: a contrast test in `browser-app`
fails, and fails again alone. The engine opens `red-1`, suspect Themes'
goal branch, with its evidence, and an examiner reads it: fix, owner t2.
`red-1`'s repair starts from main, reproduces in a unit test, fixes,
passes its check before ready, and is readied.

It shares a candidate with `finding-1` (8.2). A fuzzy test in app fails,
which passed in both members' receipts; run alone, it passes: a flake at
the gate. Nobody is rejected; it goes again and lands. The tier run
covering the repair still lists the contrast test: `red-1` is unfixed,
an alert Ines's chat takes up.

- **Model:** 7.2, 7.3, 8.1, 8.2, 10.3, 11.2.

### 7.2 One run for two tips

The tier runner was busy when Export's goal branch and the chat's readme
fix landed. When it ends, it runs the newest tip once, with both merges
as suspects:
- two export tests fail, and fail again alone: new reds;
- one test passes on its rerun: a flake;
- one story runs past its wall limit: starved, counted apart, not red;
- `live-app` cannot borrow a ChatGPT account (chatgpt-1 is lent to a
  walk): the tier faults, the run is partial, and an alert opens.

- **Model:** 8.1, 11.2, 12.2.

### 7.3 Export's red reaches its owner

The engine opens `red-2` for the two tests; Export's goal branch is
among the suspects, so an examiner reads it: fix, owner e4, a header row
dropped under one locale. `red-2`'s repair runs from main and is
readied. Alone in its candidate, its receipt green for that tip and
main, it lands on the receipt after the counted baselines and
formatting. The next tier run passes both tests: recovered, and `red-2`
closes. chatgpt-1 is back in custody, `live-app` runs, and the fault's
alert clears.

- **Model:** 7.2, 8.1, 8.2, 11.2.

### 7.4 A red no goal owns

A later run turns the ui menu test red behind the chat's second
hand-pushed branch alone: `red-3` is an alert. The chat reads the
evidence, fixes it, and clears the alert with a reason.

- **Model:** 8.2, 11.2.

### 7.5 Export closes

Export's parts are done and its goal branch has landed. The closing task
waits for a tier run covering it with no red of Export's open, which
`red-2`'s closing gives. It brings the status files of the designs
Export changed current, and the goal closes.

- **Model:** 8.3, 10.6.

## 8. Act 5: QA (hours 4 to 10)

### 8.1 A walk, its findings, triage

The inbox's ladder says r0 is due, a first walk. Ines's chat records its
start at main's tip, borrows chatgpt-1, hands the brief to its walker,
and enters the result: fail, with two findings. f1, blocking: the
export's CSV header is wrong. f2: the history page is slow.

f1 is blocking, so the triager runs at once: f1 path, owner app; f2
backlog. The engine renders the backlog file from the records and
readies it. The queue rejects it on a lint flake; the next round readies
it again at the same render, and it lands. A second rejection of that
render would have raised an alert.

- **Model:** 9.1, 9.2, 9.3, 12.2.

### 8.2 The repair, and the pass

An examiner reads f1's record and the walk's report: fix, the header
comes from the wrong locale. `finding-1` starts from main, reproduces in
a world test, fixes, and is readied (7.1). r0 holds until it lands; then
r0 is due again at a main containing it.

The chat walks r0 again at the same edition: pass, with one sighting,
`same` f2, merged into it. The engine records the pass; f1 is fixed.

- **Model:** 9.3, 9.4, 10.3.

### 8.3 A brief at fault, a new edition

r1's first walk is void: the box's setup failed. Its second fails: the
walker's model beat r1's gate by guessing a pull request's number, and
f3 says so. The triager says brief r1: a question to the ladder's owner,
Ines, in the inbox and as an issue. Ines answers "make the gate name the
pull request's opening time"; f3 is dropped with that reason, and the
chat lands the new brief. r1's edition changes, and its first walk at it
is due.

- **Model:** 9.1, 9.3, 11.1, 11.3.

### 8.4 Again, a repair's tries, not the code's

r1's walk at the new edition fails with two findings:
- **f4** is the header again, under another locale. The walker names f1,
  whose repair landed, so f4 is filed `again`, and f1 is unfixed. The
  triager puts f4 on the path, an examiner says fix, and `finding-4`
  runs. The queue rejects it at the lints; the ladder, with an adaptive
  flow, halves its target. An examiner says fix, the section joins the
  repair's words, and the second run is rejected again. Out of tries, it
  is kept; the answerer points it at the locale table's generator, and
  its fresh run finishes. On its next ready, a port test fails, and
  fails again alone: rejected. An examiner says retry, not the change's,
  and the same tip is readied again and lands.
- **f5,** not blocking: the box timed out reaching the forge. The
  triager puts it on the path; its examiner says not the code's, an
  alert.

r1's rerun at a main holding f4's repair passes, and f5's alert clears
with the pass.

- **Model:** 5.1, 5.2, 7.3, 7.5, 9.2, 9.3, 9.4, 10.3, 10.4, 11.2.

### 8.5 The machines and the reports

- **A swap:** custody's half-hourly check shows B1's Codex account at
  92% of its weekly limit; the chat swaps B1 onto codex-9, and the other
  goes back.
- **A lost wake:** B2 cannot reach the hub when a run ends; the next
  backstop round collects it and notes the lost wake. The run had gone
  with a restart of B2, so its workspace's claim is stale, and the
  engine releases it.
- **The reports:** next morning the chat runs the code and landing
  reports (runs, questions, idle gaps, weights; candidates, flakes at
  the gate, bisecting, reds, walks and passes); the status page showed
  the overlap, the questions waiting and the accounts' pace throughout.

- **Model:** 3, 12.1, 12.2, 13.

## 9. The story in fab's terms

| Story | fab | Model |
|---|---|---|
| Ines; Ines's chat | the user; the director (and ops, for the machines) | 11, 12 |
| the engine; a round | the tick, `fab tick` | 3 |
| a goal, its plan, its settings (4.1) | a plan, `plan.toml`: `landing`, `ready`, `reviews`, `direct`, `early` | 4.4 |
| Ines starts a goal (4.1) | `fab call cut --plan ... --source ...` | 4.1 |
| the planner's early pushes (4.2) | the cut's early release, `fab wake cut` | 4.2 |
| e1 to e9 and their specs (4.2) | tasks and their specs | 4.3 |
| repair-1, a resolution, red-1, finding-1 | `repair-<n>`, `catch-up-<n>`, `red-<n>`, `finding-<n>`, folded from the log | 4.3 |
| reviewer, reviser, arbiter (4.3, 4.4) | the call review run, `amend`, `settle` | 10.2 |
| a rejection counted as a refusal (4.4) | the trunk merge's rejection of `calls/<id>` | 10.2 |
| a scaffold, its checked tip, the rebases (4.4) | the skeleton call, `call.checked`, a task rebase | 10.6 |
| Ines's answers (4.3, 4.4, 8.3) | plan.toml's `answer`; `fab issues sync`, the reply call | 11.1, 11.3 |
| producers' runs in workspaces (5.1) | Codex implementer runs in builder slots | 5.1 |
| slots, a calm worker, a start handed on (5.1) | places, `flow.measure`, "no idle slot" | 5.2 |
| an early start, and one refused (5.2) | `early_starts`, `early.refused` | 6.2 |
| a merge and a recount (5.3) | the merge into `<draft>/integration`, `old_forms.py --write` | 6.1 |
| a round waking itself, a lost wake (5.3, 8.5) | `fab wake`, `tick.want`, the backstop timer | 3 |
| finished, left, transient, failed (5.3, 5.4) | `Done:`, `## Left`, `Infra:`, `Failed:` | 5.1 |
| kept (5.4) | parked: `question.asked` with `park` | 10.5 |
| examiners, accept (5.2, 5.4, 5.5, 7.1, 8.2, 8.4) | the question, refusal, red and finding foremen; `Accept:` | 10.3 |
| the answerer (5.4 to 5.6, 8.4) | the answer call | 10.4 |
| an interim check (5.6) | `fab check start --default`, `MID_EVERY` | 6.3 |
| the overlap probe (5.7) | `trunks.overlap` | 6.6 |
| the structure review (5.8) | the `review` call, reviews on | 10.2 |
| updates, a resolution, a stop (6.2) | catch-ups, `catch-up-<n>`, the stop | 6.4 |
| the final check (5.8, 6.3) | `fab check` (`fab precheck --full`) | 6.5 |
| the landing queue, a candidate, bisecting (6.1) | `fab queue`, a batch, its bisection | 7.1, 7.4 |
| the gate's error (6.4) | a batch `error` | 7.1 |
| a goal branch readied again (6.4) | catch-up, re-check, a new ready | 6.5 |
| a flake at the gate (7.1) | a collateral red, `reds.classify` | 7.3 |
| a receipt reused (7.3) | the precheck receipt carried at ready | 7.2 |
| a retry at the same tip (8.4) | `fab queue untry` | 7.5 |
| the tier runner (7.1 to 7.4) | `fab after-landing run` | 8.1 |
| alerts (6.1, 7.1 to 7.4, 8.1, 8.4) | the page's signals: `queue:`, `red-`, `fault:`, `finding-`, `backlog:` | 11.2 |
| the closing task (7.5) | the retire call | 8.3, 10.6 |
| a walk entered (8.1) | `fab qa walk`, `fab qa result` | 9.2 |
| the triager, the backlog (8.1, 8.3, 8.4) | the triage call, `ladder/<draft>/backlog-<n>` | 9.3 |
| the pass (8.2, 8.4) | `rung.passed` | 9.4 |
| the inbox | `FAB_HOME/user.md`, `fab page answer`, `fab page clear` | 11.1 |
| workspaces, sweeps, stale claims (5.1, 8.5) | slots, `fab slot`, `heavy` | 12.1 |
| custody (7.2, 8.1, 8.5) | the login shelf, `fab logins` | 12.2 |
| the reports, the status page (5.7, 8.5) | `fab report daily`, `fab serve` | 13 |

## 10. Left out, and why

- **A landing of one by rebase** (model 7.6): fab lands its own
  repository so; a candidate of one (7.3) shows the landing temper
  meets.
- **A replay** (`ready` no): a measurement, not a way to land.
- **The decision task:** started by hand; its probe is not built.
- **A shared-shape scaffold:** 4.4's scaffolds show its mechanics.
- **The flow growing** (model 5.2): only on a first-attempt landing with
  the target full, which the ladder never fills.
- **fab's two faults** (model 10.6): the case shows the intended
  behaviour, one rebase and a closing task whose branch lands.

## 11. What temper and jig provide

For each mechanism: **generic** (a mechanism a deployment configures
with settings, charters or templates), **fixed** (provided, but its
behaviour is code, not configurable), or **lacks** (*designed* where a
design has it). Evidence at fabrig 2075ff38b; unqualified documents are
temper's `../domain/` ones, jig's are named.

| Mechanism | Model | Today | What is missing |
|---|---|---|---|
| One writer, derived state, wakes, timers | 3 | generic: one engine, the store's only writer; inbox and queue derived; wake policies, timers (jig's `domain/engine.md` 5, `domain/tasks.md` 8.3) | record kinds a deployment declares (walks, findings, reds, overlaps) |
| A goal from a draft; the planner | 4.1, 4.2 | generic goal (`agent.md` 4.3.2; jig's `domain/core.md` 6.2); lacks the planner | an agent task whose result is a plan of parts, slices and specs as files, released in pieces |
| Changes, specs with options, repairs from records | 4.3 | fixed: the change procedure, its producer and repairs (`forge.md` 8); compiled briefs generic (jig's `domain/engine.md` 9) | options settling questions; repairs made from records |
| A goal's settings | 4.4 | lacks: landing settings are per project (`landing.md` 13.1, *designed*) | settings per goal |
| A run, its results, tries | 5.1 | generic charters (model, effort, turns, spend); results and tries fixed (jig's `domain/tasks.md` 5.5) | a transient result spending no try; a list of what was left |
| Slots, the flow, the order of starts | 5.2 | generic in part: slots, placement, priority (jig's `domain/engine.md` 8) | a target of runs in flight, fixed or adaptive; calm; repairs first |
| The goal branch and its merges | 6.1 | lacks (*designed*: a branch per goal, `forge.md` 11.1) | merges of each finished change, the recount |
| Early starts | 6.2 | lacks (*designed*: started kept beside a dependency, jig's `domain/connectors.md` 6.1) | a start on a dependency's pushed branch |
| Interim checks | 6.3 | fixed: a red base pausing the queue for one repair (`forge.md` 9) | checks of a goal branch on a cadence |
| Updates, resolutions, a stop | 6.4 | fixed: lazy updates, a conflict as a resolution (`forge.md` 10, 8.4) | timed updates of a goal branch; a stop cleared by hand |
| The final check, ready again | 6.5 | fixed: a change's gates and bounded repairs (`forge.md` 8.2 to 8.4) | a whole check before ready; readied again after a rejection |
| The overlap probe | 6.6 | lacks: only news to overlapping goals (`forge.md` 7.3) | a merge probe of goal branches, shown |
| The landing queue: candidates, selection, a gate lane, errors | 7.1 | fixed: a queue per landing branch, or the turn (`forge.md` 9; `landing.md` 8) | candidates of several changes, a full selection on a cadence, an exclusive lane, an error rejecting nobody |
| A check before ready, a receipt reused | 7.2 | fixed: receipts by key, the turn only (`landing.md` 6) | a check in the change's workspace, and its receipt |
| A flake at the gate | 7.3 | lacks (*designed*: `findings.md` 5); faults kept apart (`landing.md` 7.4) | a failed test rerun alone, with its bound |
| Bisecting a candidate | 7.4 | lacks (*designed* for tiers: `after-landing.md` 5.2) | bisecting at the gate |
| Who hears a rejection | 7.5 | fixed in part: a failure as a repair (`landing.md` 7.3) | an examiner choosing fix or retry; an alert for an unowned branch |
| Tiers following main | 8.1 | lacks (*designed*: `after-landing.md` 2 to 5) | tiers built, CPU, starvation, unattributed, live tiers on lent accounts |
| A red reaching its owner | 8.2 | lacks (*designed*: `after-landing.md` 7) | grouping, an examiner, a repair from main made by the engine, unfixed |
| A goal closing after its verdict | 8.3, 10.6 | lacks (*designed*: a change lives to cleared, `after-landing.md` 6) | a goal waiting for its verdict; the closing task |
| The ladder, editions, walks | 9.1, 9.2 | lacks: a process outside (`../qa.md`); a walk *designed* (`findings.md` 8) | rungs, editions and walks as records |
| Findings, triage, the backlog | 9.3 | lacks (*designed*: `findings.md` 2, 4, 6) | merges by the walker, again, the person's and the brief's decisions, a backlog landed |
| A finding's repair, the pass | 9.4 | lacks (*designed*: `findings.md` 6, 7) | an examiner, a repair from main, rung states, a pass on an unchanged rerun |
| Agent tasks | 10.1 | generic: finite tasks under charters with budgets (jig's `domain/engine.md` 9; `agent.md` 4.6) | peak context measured |
| Review, reviser, arbiter, structure review | 10.2 | fixed: a review gate, changes requested as a repair (`forge.md` 8.3) | review of any agent task's output, the reviser, the arbiter, a rejection counted |
| The examiner | 10.3 | generic in part: escalations, a person's choice (jig's `domain/tasks.md` 9) | settling from options with no one asked; accept; a judge of rejections, reds, findings |
| The answerer; kept | 10.4, 10.5 | generic in part: a question keeps its task; wakes batched (jig's `domain/tasks.md` 5.7, 8.3) | one task answering a goal's queued questions; a part kept on a question |
| Scaffolds | 10.6 | lacks | scaffolds, checked tips, changes started from them and rebased |
| The inbox and alerts | 11.1, 11.2 | generic in part: a party's inbox (jig's `domain/people.md` 6, 8) | alerts from records; a plan's question answered in the plan |
| Questions on the forge | 11.3 | lacks (*designed*: `forge.md` 12.1, Later) | a reply task that converses first |
| Workspaces and disk | 12.1 | fixed in part: a checkout cache (`worker.md` 5) | disk admission, sweeps, lanes |
| Custody | 12.2 | generic in part: accounts with holders and states (jig's `domain/accounts.md`) | lending to a run, returning, swaps, limit checks |
| Reports and the status page | 13 | generic in part: views, spend per period (jig's `domain/engine.md` 11, `domain/authority.md` 7) | the two reports, a public page, weights |

