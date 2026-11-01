# The reference case

Draft, 2026-11-01, fourth edition, not adopted until the user approves
it. One invented story, as small as it can be while it runs every
mechanism of `landing-model.md` (revision 5) under fabrig's
configuration (its section 14; Themes turns two settings on), with fab's
cast, so that someone who knows fab recognises each role and step.
Nobody works by hand: Ines talks to her chat through `temper ctl` and
answers her inbox, and the rest is code or agent tasks. temper and jig
must run it to replace fab. Section 9 maps it onto fab and the model,
code or LLM; section 11 says what temper and jig provide of each
mechanism: the input to the execution plan. Designs citing the third
edition are repointed on adoption.

## 1. In one page

- **One project, Harbor,** one repository on an owned Forgejo.
- **Ines talks to her chat** through `temper ctl`, a chat made from a
  director template, and answers her inbox there, or on a question's
  forge issue (model 11.3); nothing else is hers. The chat creates
  goals; each goal's agent and the engine do the rest (3.2).
- **Four goals:** Export and Themes, from two drafts, each on a goal
  branch; Docfix, a small fix; and the ladder, Harbor's QA.
- **Five acts, 26 steps, about 10 hours,** overlapping in time: the
  start (section 4), the goal branches (section 5), landing (section 6),
  after landing (section 7), and QA (section 8).

## 2. How to read it

Each step says what happens, then **Model:**, the model's sections it
shows. Times are hours from the start. On fakes, timers and bounds are
scaled down and each agent task's result is scripted as the oracle; on
real backends, a QA driver types Ines's words through `temper ctl`.

## 3. The project, the cast and the start

### 3.1 Harbor

- **The repository,** `harbor`:
  - packages: `kit`; `ui`, which reads kit; `app`, which reads kit and
    ui;
  - `debts.txt`, a counted baseline, recounted by `tools/count`;
    - `docs/`, the project's designs and drafts. Goals, plans, specs and
      findings live in temper's store, never in the repository.
- **The gates:** formatting, lints, the build, and the `default` and
  `fuzzy` tests: the check before ready runs them in a change's
  workspace, the landing queue in its gate lane, the final check whole.
- **The tiers:** `browser-app` (CPU recorded), and `live-app`, which
  needs a ChatGPT account.
- **The machines:** E, the engine's machine, which runs the engine,
  custody and the Claude agent tasks; workers B1 (run target four, five
  slots), B2 (run target four, four slots) and G (run target two), which
  holds the gate lane and the tier runner.
- **Custody:** a Codex and a Claude account lent to each machine;
  codex-9 and chatgpt-1 free.
- **The settings** are fabrig's (model section 14), with custody's
  `swap_at`, `walk_boxes` and the reports' timer set, which fab lacks.

### 3.2 The cast

Ines; her chat; each goal's agent; the engine; and the agent tasks, each
as fab has it: the planner, the producers (Codex runs in workspaces),
the resolvers, the examiners, the answerer, the reviewer, the reviser
and the arbiter, the scaffolds, the structure review, the wrap-up, the
triager, the walker and the reply task. The landing queue on G and the
tier runner are code. Section 9 maps each onto fab.

### 3.3 At hour 0

Main is green in every tier. The ladder goal, which Ines's chat created
from the QA kit, is open, with rungs r0 and r1 (r1 after r0), each on
ChatGPT with the owned Forgejo, sharing a kit.

## 4. Act 1: the start (hours 0 to 1)

### 4.1 Ines asks for Export and Themes

Ines tells her chat, through `temper ctl`, that Export's draft is in,
and then Themes'. The chat creates two goals: Export with fabrig's
settings (a goal branch, ready, review off, scaffold off, early starts
on), and Themes, with `review` and `scaffold` on, which its template
allows. Each goal's agent starts a planner on its draft.

- **Model:** 3, 4.1, 4.4, 10.1.

### 4.2 Export's planner hands over early

The planner surveys kit with a read-only run, writes the export record's
shape first, then hands over the plan's first piece, three changes that
depend on nothing, recorded in the store with their specs: e1, the
record in kit; e2, the CSV writer in app; e3, the menu entry in ui. The
engine starts them at once, from Export's goal branch, each on the
worker furthest under its run target, while the planner writes e4 to e9
in two more pieces. Its brief listed Themes' changes and paths; it names
the overlap it could not avoid: both goals touch ui's menu.

- **Model:** 3, 4.2, 4.3, 5.1, 5.2, 6.6.

### 4.3 Themes' planner, reviewed, and a part kept

Themes' planner writes two scaffolded parts, palette and dark, and a
question for Ines: does dark mode follow the system? It names part dark.
The reviewer refuses the plan (a spec names a package that does not
exist); the reviser fixes it, the reviewer approves, and the plan is
taken. Dark is not designed while the question is open. Ines answers
"follow the system" in her inbox; the answer is recorded in the store.

- **Model:** 4.2, 10.2, 10.5, 11.1.

### 4.4 Palette's scaffold asks on the forge

Palette's scaffold ends with a question: keep the old colour names as
aliases? The subject is kept, and an issue opens on the forge. Ines
comments "what do the aliases cost?"; the reply task answers with the
two options and a recommendation. Ines comments "drop them"; the next
reply task records it as the answer, and the issue closes. The scaffold
runs again with the answer in its brief.

- **Model:** 10.1, 10.5, 11.1, 11.3.

### 4.5 Two scaffolds: an arbiter, a resolution

Palette's scaffold writes its types with empty bodies and the part's
specs. The engine checks its tip (gates and tests), and t1 to t3 start
from it while the reviewer reads it. The reviewer refuses it: a type's
doc contradicts the design it cites; the reviser fixes it; the reviewer
refuses again: a body the design leaves to guesswork. It is held, and
the arbiter says land: the part's specs settle that body. It merges. The
engine merges the revised scaffold into t1 to t3, finished from the
first tip, as resolutions outside their tries, and they merge.

Dark's scaffold, approved at once, conflicts with palette's when it
merges into the goal branch: that rejection counts as a refusal, with no
revision yet, so the engine merges the goal branch into it, a reviser
resolves the conflict with no git writes of its own, the reviewer
approves it again, and it merges.

- **Model:** 6.1, 10.2, 10.6.

## 5. Act 2: the goal branches (hours 1 to 4)

### 5.1 As wide as the run targets

Each goal's target is the workers' run targets, ten. Each run starts in
a workspace the engine sets up, from its goal branch's tip, on a brief
it assembles; before each start, the worker's code moves to the engine's
version. e7's start, aimed at B2, finds every slot taken (one by a
walk's build, 8.1), and hands on to B1, under its run target. Later B1
is calm, so it takes e9 as a fifth run in its fifth slot.

- **Model:** 5.1, 5.2, 12.1.

### 5.2 A question the options settle, and early starts

e2 asks whether a quote in a field is doubled or escaped; its spec's
options say doubled. An examiner answers; the answer joins e2's spec,
and e2 runs again from its pushed branch.

e4, the export command in app, depends on e2. While e2 runs again with
its branch pushed, e4 starts early: its workspace is the goal branch
with e2's branch merged in. When e4 finishes, the engine merges e2's
final branch into it again, and e4 merges only after e2. e5 depends on
e3, but e3's branch and the goal branch both lowered `debts.txt`, and an
early start recounts nothing: e5's early start is refused, and it waits
for e3 to merge.

- **Model:** 5.1, 6.2, 10.3.

### 5.3 Finished with something left, and merges

e1 finishes, leaving a lint in ui that its type change caused outside
its paths; the engine pushes its branch and merges it. e3 merges too;
both lowered `debts.txt`, so the second merge conflicts there, and the
engine takes the goal branch's side and recounts. No one hears of it.
e3's merge makes e5 runnable, and it starts.

- **Model:** 3, 5.1, 6.1.

### 5.4 A merge conflict, a disk wait, a failure

- **e6 and e5** both change app's command table. e5 merges first; e6's
  merge conflicts and is a rejection. An examiner says fix; a merge
  refused it, so the next run is a resolution: the engine merges the
  goal branch into e6 in a workspace, a resolver settles the command
  table with no git writes of its own, and the engine commits, pushes
  and merges.
- **e7's** build waits on its worker's disk: it reclaims idle
  workspaces, waits 20 minutes, and fails as transient, by its exit. The
  engine runs e7 again without asking, a try counted; it finishes.
- **e8's** run fails (its fixture generator crashes). It is held and
  escalated to the answerer at once, which says the generator is e9's
  and to stub the fixture; e8 runs again and finishes.

- **Model:** 5.1, 6.1, 10.3, 10.4, 10.5, 12.1.

### 5.5 Questions outside the change

e9 asks whether it may change ui's theme hook, which Themes owns. The
spec's options do not settle it, so the examiner queues it, and the
answerer runs at once (a goal branch's): leave ui to its owner and list
it as left. e9's next run pushes its work but asks again, about a ui
test it cannot make green. The examiner says accept: the change is
finished as its branch stands, and it merges.

- **Model:** 10.3, 10.4.

### 5.6 An interim check, and its repair

At the tenth merge, the engine runs a check job on Export's goal branch
in an idle workspace, without the fuzzy tests: the default tests are red
in app. `repair-1` is made from the records, its words the failing gates
and the unmerged changes' paths. Its run asks which of two failing tests
is right; with no written spec, the question queues for the answerer,
whose answer joins repair-1's words. The next run finishes and merges.

- **Model:** 4.3, 6.3, 10.3, 10.4.

### 5.7 Goals beside each other

Each round the overlap probe merges both goal branches with main in
memory, then with each other: ui's menu conflicts. It records the
overlap once, and `temper ctl`'s status shows Export and Themes with a
count of one. Nothing waits on it.

- **Model:** 6.6, 13.

### 5.8 Themes is ready first

Four hours in, Themes' structure review reads the digests of the changes
merged so far and writes one consolidation change, merging two colour
helpers; it starts ahead of Themes' other changes. Themes' last change
merges; the engine runs the final check, green, and readies it.

- **Model:** 5.2, 6.4, 6.5, 10.2.

## 6. Act 3: landing (hours 3 to 6)

### 6.1 A small fix, a candidate, bisecting

Ines tells her chat that a doc comment in `ui/src` names the wrong
function. The chat creates a small goal, Docfix (`landing = each`); its
agent starts one change, whose producer fixes the comment; the engine
runs its check before ready and the counted-baseline gate, green, and
readies it. The landing queue prepares it and Themes' goal branch onto
main as one candidate, its fifth, so the full selection runs: the doc
lint is red, since Themes renamed the function the new comment names.
The candidate is halved, left first, each half in full: Themes alone is
green and lands. Docfix's change alone, on the new main, is red and
rejected; an examiner says fix, its next run renames the reference, and
it lands.

- **Model:** 4.1, 7.1, 7.2, 7.4, 7.5, 10.3.

### 6.2 Updates, a resolution, a stop

Main has moved under Export. Thirty minutes after its last update, the
engine merges main into Export's goal branch: ui's menu conflicts, as
the probe said. A resolution: the engine starts that merge in a
workspace, a resolver settles the menu keeping both sides' intent, and
the engine commits it and merges it; the goal branch contains that main,
a clean update. The overlap closes, Themes having landed. The next
interim check finds app code Themes' palette broke; a repair fixes it.

Main moves again, and the next three updates conflict in a lock file the
resolver cannot settle. At the third refusal since the clean one, the
goal branch's steps stop, escalated to Export's agent, which reads the
record and starts a resolution that regenerates the lock file with the
project's tool; it merges, a clean update, and the steps resume.

- **Model:** 4.1, 6.1, 6.4, 6.5, 6.6.

### 6.3 Export's final check, red, then ready

The last change merges. The final check is red: a fuzzy test in app.
`repair-2` fixes it; the check runs again, green, and the goal branch is
ready.

- **Model:** 6.5.

### 6.4 The gate's fault, then a rejection

The candidate holding Export's goal branch stops at the build: G's disk
wait ran out. It is a fault of the gate: nobody is rejected, and it goes
again. Its next attempt is red: a ladder repair (8.2) that landed
meanwhile changed kit's error type, and Export's app code no longer
builds on it. The engine updates the goal branch, checks it (red,
`repair-3`, green), and readies it again. It lands.

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
an alert to Ines's chat, which creates a goal for it and clears the
alert naming that goal.

- **Model:** 7.2, 7.3, 8.1, 8.2, 10.3, 11.2.

### 7.2 One run for two tips

The tier runner was busy when Export's goal branch and Docfix's change
landed. When it ends, it runs the newest tip once, with both merges as
suspects:
- two export tests fail, and fail again alone: new reds; a story never
  run before fails too, unattributed, in the same group;
- one test passes on its rerun: a flake;
- one story runs past its wall limit: starved, counted apart, not red;
- `live-app` cannot borrow a ChatGPT account (chatgpt-1 is lent to a
  walk): the tier faults. The run is red with a fault, so the tip runs
  again; an alert reaches the chat, which waits for it to clear.

- **Model:** 8.1, 11.2, 12.2.

### 7.3 Export's red reaches its owner

The engine opens `red-2` for the three tests; Export's goal branch is
among the suspects, so an examiner reads it: fix, owner e4, a header row
dropped under one locale. `red-2`'s repair runs from main in Export's
goal and is readied. Alone in its candidate, its receipt green for that
tip and main, it lands on the receipt after the counted baselines and
formatting. The next tip's run, holding the repair, passes all three
tests (recovered, and `red-2` closes) while `live-app` faults again:
partial, so the tip after it covers its suspects. There, chatgpt-1 is
back in custody, `live-app` runs, and the fault's alert clears.

- **Model:** 7.2, 8.1, 8.2, 11.2.

### 7.4 Export ends

Export's parts are done and its goal branch has landed. The wrap-up
waits for a tier run covering it with no red of Export's being decided
or repaired, which `red-2`'s closing gives. It brings the status files
of the designs Export changed current; its branch lands, and the goal
ends.

- **Model:** 8.3, 10.6.

## 8. Act 5: QA (hours 4 to 10)

### 8.1 A walk, its findings, triage

r0 is due, a first walk. The engine records the walk's start at main's
tip, builds Harbor there, lends chatgpt-1, and starts the walker on r0's
brief. It reports fail, with two findings. f1, blocking: dark mode
ignores the system's setting. f2: the history page is slow.

f1 is blocking, so the triager runs at once: f1 path, owner ui; f2
backlog, shown in `temper ctl`; fabrig's settings also render it into a
project document, readied as a change, and it lands.

- **Model:** 9.1, 9.2, 9.3, 12.2.

### 8.2 The repair, and the pass

An examiner reads f1's record and the walk's report: fix, kit reads the
setting with the wrong error type. `finding-1` starts from main,
reproduces in a world test, fixes, and is readied (7.1). r0 holds until
it lands; then r0 is due again at a main containing it.

The engine starts r0's rerun at the same edition: pass, with one
sighting, `same` f2, merged into it. The engine records the pass; f1 is
fixed.

- **Model:** 9.2, 9.3, 9.4, 10.3.

### 8.3 A brief at fault, a new edition

r1's first walk is void: the box's setup failed. Its second fails: the
walker's model beat r1's check by guessing a pull request's number, and
f3 says so. The triager says brief r1; the ladder goal's agent revises
r1's brief in the store to name the pull request's opening time, and f3
is dropped. r1's edition changes, and its first walk at it is due.

- **Model:** 4.1, 9.1, 9.3.

### 8.4 Again, a repair's tries, not the code's

r1's walk at the new edition fails with two findings:
- **f4** is dark mode again, on the history page. The walker names f1,
  whose repair landed, so f4 is filed `again`, and f1 is unfixed. The
  triager puts f4 on the path, an examiner says fix, and `finding-4`
  runs. The queue rejects it at the lints; the ladder, with an adaptive
  flow, halves its target. An examiner says fix, the section joins the
  repair's words, and the second run is rejected again; its examiner
  says fix again. Out of tries, it is held and escalated; the answerer
  points it at the page's own theme reader, and its fresh run finishes.
  On its next ready, a port test fails, and fails again alone: rejected.
  An examiner says retry, not the change's, and the same tip is readied
  again and lands.
- **f5,** not blocking: the box timed out reaching the forge. The
  triager puts it on the path; its examiner says not the code's: an
  alert, which the chat leaves for the rerun.

r1's rerun at a main holding f4's repair passes, and f5's alert clears
with the pass.

- **Model:** 5.1, 5.2, 7.3, 7.5, 9.2, 9.3, 9.4, 10.3, 10.4, 11.2.

### 8.5 The machines and the reports

- **A swap:** custody's half-hourly check shows B1's Codex account past
  `swap_at`; custody swaps B1 onto codex-9, and the other goes back.
- **A lost worker:** B2 restarts while a run is in flight; the run ends
  lost and runs again elsewhere, and the engine releases its workspace's
  stale claim.
- **The reports:** at midnight the daily timer starts the code and
  landing reports for the chat; `temper ctl`'s status showed the
  overlap, the questions waiting and the accounts' pace throughout.

- **Model:** 3, 12.1, 12.2, 13.

## 9. The story in fab's terms

Kind: **code** or **LLM**, the model's. "Gap": fab does it by hand, by
an agent where the model has code, or by code where it has an agent
(model 14.2); temper does not copy it.

| Story | Kind | fab | Model |
|---|---|---|---|
| Ines talks to her chat in ctl (4.1) | LLM | the user, the director; gap: `fab call cut` by hand | 4.1 |
| a goal's settings | code | `plan.toml` (gap: plans and specs in the repository) | 4.4 |
| the engine | code | the tick, `fab tick`; its rounds and backstop are fab's (14.2) | 3 |
| the planner, its survey (4.2) | LLM | the cut, its survey runs | 4.2 |
| its pieces recorded, started (4.2) | code | gap: the cut pushes and runs `fab wake cut` | 4.2 |
| specs (e1 to e9) | LLM | task specs | 4.3 |
| repair-1, red-1, finding-1 made from records | code | `repair-<n>`, `red-<n>`, `finding-<n>` | 4.3 |
| reviewer, reviser, arbiter (4.3, 4.5) | LLM | the call review run, `amend`, `settle` | 10.2 |
| a rejection counted as a refusal (4.5) | code | the trunk merge's rejection of `calls/<id>` | 10.2 |
| scaffolds (4.4, 4.5) | LLM | skeleton calls | 10.6 |
| a scaffold's checked tip, merged in (4.5) | code | `call.checked`; gap: task rebases | 10.6 |
| a plan's answer recorded (4.3) | code | gap: written into plan.toml by hand | 11.1 |
| the forge's issue (4.4) | code | `fab issues sync` (gap: its timer by hand) | 11.3 |
| the reply (4.4) | LLM | the reply call | 11.3 |
| producers | LLM | Codex implementers | 5.1 |
| pushing a run's branch (5.3) | code | gap: the implementer pushes | 5.1 |
| run targets, calm, a start handed on (5.1) | code | places, `flow.measure` | 5.2 |
| early starts (5.2) | code | `early_starts`, `early.refused`; gap: the implementer merges again | 6.2 |
| a merge and a recount (5.3) | code | the merge into `<draft>/integration`, `old_forms.py --write` | 6.1 |
| the resolution's merge (4.5, 5.4) | code | gap: rebase runs rebase and push themselves | 6.1 |
| a resolver (4.5, 5.4, 6.2) | LLM | rebase, catch-up and amend runs | 6.1 |
| a transient failure by its exit (5.4) | code | gap: read from the run's `Infra:` line | 5.1 |
| finished, left, a question, failed | LLM | `Done:`, `## Left`, `Question:`, `Failed:` | 5.1 |
| kept, held (5.4, 8.4) | code | parked; `park` marks fab's own (a failed run, the run limit) | 10.5, 5.1 |
| examiners (fix, retry, accept) | LLM | the foremen | 10.3 |
| a resolution or a repair, by the refused gate | code | `REBASE_STEPS` | 10.3 |
| the answerer (5.4) | LLM | the answer call | 10.4 |
| an interim check (5.6) | code | `fab check start --default` | 6.3 |
| overlaps (5.7) | code | `trunks.overlap` | 6.6 |
| the structure review (5.8) | LLM | the `review` call | 10.2 |
| Docfix, a small goal (6.1) | LLM | gap: a ready ref pushed by hand, no plan's | 4.1 |
| the check before ready (6.1) | code | gap: run by the implementer | 7.2 |
| a candidate, a full selection, bisecting (6.1) | code | `fab queue`, a batch, every fifth full | 7.1, 7.4 |
| updates (6.2) | code | catch-ups (gap: a repair merges main itself) | 6.4 |
| a stop, the goal's agent deciding (6.2) | LLM | gap: a note, then a hand merge | 6.4 |
| the final check (5.8, 6.3) | code | `fab check` (`fab precheck --full`) | 6.5 |
| the gate's fault, readied again (6.4) | code | a batch `error`; re-check, a new ready | 7.1, 6.5 |
| a flake at the gate (7.1) | code | a collateral red, `reds.classify` | 7.3 |
| a receipt reused (7.3) | code | the precheck receipt carried at ready | 7.2 |
| a retry at the same tip (8.4) | code | `fab queue untry` | 7.5 |
| the tier runner (7.1 to 7.3) | code | `fab after-landing run` | 8.1 |
| a red's repair in its owner's goal (7.3) | code | gap: in the newest suspect's plan | 8.2 |
| alerts to the chat (7.1, 7.2, 8.4) | code | gap: signals on the page | 11.2 |
| the chat acting on alerts (7.1) | LLM | gap: read and cleared by a session | 11.2 |
| the wrap-up (7.4) | LLM | the retire call (gap: on a trunk, never reaches main) | 8.3, 10.6 |
| a walk started, built, lent, recorded (8.1) | code | gap: a session, `fab qa walk`, `fab qa result` | 9.2 |
| the walker (8.1) | LLM | the session's walker | 9.2 |
| the triager (8.1) | LLM | the triage call | 9.3 |
| the backlog readied (8.1) | code | `ladder/<draft>/backlog-<n>` (gap: the queue never takes it) | 9.3 |
| a brief rewritten (8.3) | LLM | gap: a question to the person, then a hand edit | 9.3 |
| the pass (8.2) | code | `rung.passed` | 9.4 |
| the inbox | code | `user.md`, `fab page answer` | 11.1 |
| workspaces, a lost run, a stale claim (5.1, 8.5) | code | slots, `heavy` (gap: lost runs and stale claims by hand) | 12.1 |
| workers following the engine (5.1) | code | `_sync_install` | 5.2 |
| custody, a swap (7.2, 8.1, 8.5) | code | the login shelf (gap: lending and `fab logins swap` by hand) | 12.2 |
| reports, status (5.7, 8.5) | code | gap: `fab report daily` by hand | 13 |

## 10. Left out, and why

- **fab's own repository's landing** (a candidate of one by rebase, its
  code-style gates, refused paths, push attempts): a configuration in
  the model's 14.1, left out at the user's word.
- **A replay** (`ready` no): a measurement, not a way to land.
- **Not shown, for brevity:** `gate_flakes`, `review_faults`,
  `producer_allowances`, tier budgets; the arbiter's other verdicts;
  triage's `user`, `merge` and batching; a red not the suspects' (8.4
  shows a finding's); a red no goal owns (an alert, as 7.1's); a
  conflict at `prepare`; a backlog rejected too often; a held scaffold;
  the stops after `final_reds` and `queue_refusals`, as 6.2's.
- **The flow growing:** only on a first-attempt landing with the target
  full, which the ladder never fills.

## 11. What temper and jig provide

For each mechanism: **generic** (a mechanism a deployment configures
with settings, charters or templates), **fixed** (provided, but code
that settings cannot change), or **lacks** (*designed* where a design
has it), judged for `temper ctl` only. Kind is the model's: code or LLM.
Evidence at fabrig 2075ff38b; unqualified documents are temper's
`../domain/` ones.

| Mechanism | Model | Kind | Today | What is missing |
|---|---|---|---|---|
| One writer, derived state, wakes | 3 | code | generic: the engine, the store's only writer (jig's `domain/engine.md` 2); the inbox derived (jig's `domain/people.md` 6); wake policies, timers (jig's `domain/tasks.md` 8.3) | record kinds a deployment declares |
| A chat from a template; tools that create goals | 4.1 | LLM | generic in part: templates and charters as settings (jig's `protocol/configuration.md` 3.1); the chat's `goal`, `change` and `delegate` tools (`agent.md` 4.3.2, 4.3.1, 4.3) | ctl starting a chat from any template; `goal` choosing its template; `alerts_to` |
| The planner | 4.2 | LLM; code | lacks | an agent task whose result is a plan of tasks with specs, in the store, in pieces |
| Changes, specs with options, repairs from records | 4.3 | LLM; code | fixed: the change procedure (`forge.md` 8); briefs from sections (jig's `domain/engine.md` 9) | options settling questions; repairs made from records |
| A goal's settings | 4.4 | code | lacks: landing settings are per project (`landing.md` 13.1) | settings per goal |
| A run, its ends, tries | 5.1 | LLM; code | generic: charters; tries per failure class configured per deployment (jig's `domain/tasks.md` 5.5; `jig-protocol-config` overrides) | tries counted as runs not landed; a list of what was left; fresh tries granted by an answer |
| Slots, run targets, the order of starts | 5.2 | code | generic in part: slots and placement (jig's `domain/engine.md` 8); priority in hold queues (jig's `domain/tasks.md` 6.2) | a run target and a flow target; calm; repairs first |
| The goal branch and its merges | 6.1 | code; LLM | lacks (*designed*: `forge.md` 11.1, landing each change gated); resolutions fixed (`forge.md` 8.4) | ungated merges into a goal branch, the recount |
| Early starts | 6.2 | code | lacks (*designed*: jig's `domain/connectors.md` 6.1) | a start on a dependency's pushed branch |
| Interim checks | 6.3 | code | fixed: a red base pausing the queue (`forge.md` 9); jobs *designed* (jig's `domain/core.md` 3.2) | check jobs of a goal branch on a cadence |
| Updates, resolutions, a stop | 6.4 | code; LLM | fixed: lazy updates, resolutions (`forge.md` 10, 8.4); holds escalate to the goal (jig's `domain/tasks.md` 5.5, 9) | timed updates of a goal branch |
| The final check, ready again | 6.5 | code | fixed: a change's gates, bounded repairs (`forge.md` 8.2 to 8.4) | a whole check before ready; readied again |
| The overlap probe | 6.6 | code | lacks: news to overlapping goals only (`forge.md` 7.3) | a merge probe of goal branches |
| The landing queue | 7.1 | code | fixed: a queue per branch, or the turn (`forge.md` 9; `landing.md` 8) | candidates of several, a full selection on a cadence, a gate lane, `prepare`, `selection`, `gate_set` |
| A check before ready, a receipt reused | 7.2 | code | fixed: receipts by key (`landing.md` 6) | the check in the change's workspace, its receipt |
| A flake at the gate | 7.3 | code | lacks (*designed*: `findings.md` 5) | a failed test rerun alone, bounded |
| Bisecting a candidate | 7.4 | code | lacks (*designed* for tiers: `after-landing.md` 5.2) | bisecting at the gate |
| Who hears a rejection | 7.5 | code; LLM | fixed in part: a failure as a repair (`landing.md` 7.3) | an examiner choosing fix or retry |
| Tiers following main | 8.1 | code | lacks (*designed*: `after-landing.md` 2 to 5; jig's jobs) | tiers, CPU, starvation, partial runs, live tiers on lent accounts |
| A red reaching its owner | 8.2 | code; LLM | lacks (*designed*: `after-landing.md` 7) | grouping, an examiner, a repair from main, unfixed |
| A goal ending after its verdict | 8.3, 10.6 | code; LLM | lacks (*designed*: `after-landing.md` 6) | waiting for the verdict; the wrap-up |
| The ladder and its rungs | 9.1 | code | lacks: a process outside (`../qa.md`) | rungs, editions, states as records |
| A walk | 9.2 | code; LLM | lacks (*designed*: `findings.md` 8) | walks started, builds, accounts lent, results recorded |
| Findings, triage, the backlog | 9.3 | code; LLM | lacks (*designed*: `findings.md` 2, 4, 6) | merges, again, the triager's decisions, a backlog landed |
| A finding's repair, the pass | 9.4 | code; LLM | lacks (*designed*: `findings.md` 6, 7) | an examiner, a repair from main, a pass on an unchanged rerun |
| Agent tasks | 10.1 | LLM | generic: finite tasks under charters (jig's `domain/engine.md` 9; `agent.md` 4.6) | peak context measured; `agent_hosts` |
| Review, reviser, arbiter, structure review | 10.2 | LLM; code | generic in part: review gates per landing rule, the review charter; remarks as a bounded repair (`forge.md` 8.3, 8.4; `protocol/configuration.md` 3) | review of planning outputs, the arbiter, a rejection counted |
| The examiner | 10.3 | LLM | generic in part: escalations (jig's `domain/tasks.md` 9) | settling from options; accept; a judge of rejections, reds, findings |
| The answerer; kept and held | 10.4, 10.5 | LLM; code | generic in part: a question keeps its task, routed to its requester; held past tries (jig's `domain/tasks.md` 5.5, 5.7) | one task answering queued questions; waking on questions by count and age (a bool today); a part kept |
| Scaffolds | 10.6 | LLM; code | lacks | scaffolds, checked tips, changes started from them |
| The inbox and alerts | 11.1, 11.2 | code; LLM | generic in part: a party's inbox in ctl (jig's `domain/people.md` 6, 8) | alerts to a chat |
| Questions on the forge | 11.3 | code; LLM | lacks (*designed*: `forge.md` 12.1, Later) | a reply task |
| Workspaces and disk | 12.1 | code | fixed in part: a checkout cache (`worker.md` 5) | disk admission, sweeps, lanes, stale claims released |
| Custody | 12.2 | code | generic in part: accounts with holders, a run's grant, limited until a reset, export and import (jig's `domain/accounts.md` 3.2, 3.4, 3.5) | swaps, limit checks, credentials lent to walks; moving a program's login between machines conflicts with `accounts.md` 6 |
| Reports and status | 13 | code | lacks for ctl: recurring tasks fire per spend period, and nothing in ctl starts one (jig's `domain/tasks.md` 10) | the two reports on a daily timer; status counts in ctl; weights |











