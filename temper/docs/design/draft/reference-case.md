# The reference case

Draft, 2026-11-01, fourth edition. Nothing here is adopted until the
user approves it. One invented story, as small as it can be while it
runs every mechanic of `landing-model.md` (revision 5), the model fab
runs today. temper and jig must be able to run it. Each step cites the
model's sections it shows. Section 9 lists, mechanic by mechanic, what
temper and jig have today and what they lack: the input to the
execution plan that follows approval.

## 1. In one page

- **One project, Harbor,** with one repository on an owned Forgejo.
- **Two actors:** the person, and the person's chat, which reads the
  inbox page and does what has no other owner. Everything else is the
  dispatcher with its runs and calls.
- **Three plans:** Export and Themes, two drafts on trunks of their own,
  and the ladder, Harbor's QA.
- **Five acts, 26 steps, about 10 hours:** the start (section 4), the
  trunks (section 5), landing (section 6), after landing (section 7),
  and QA (section 8). Acts 4 and 5 overlap in time.

## 2. How to read it

- **Each step** says what happens, then **Model:** the sections of
  `landing-model.md` it shows.
- **Time** is hours from the start. On fakes, timers and bounds are
  scaled down.
- **Run two ways:** on fakes, each call's and run's report scripted as
  the oracle; on real backends, with real producers on the real forge,
  and the person's words typed by a QA driver.

## 3. The project and the start

- **The repository,** `harbor`:
  - packages: `kit`; `ui`, which reads kit; `app`, which reads kit and
    ui;
  - `debts.txt`, a counted baseline that only falls, recounted by
    `tools/count`;
  - `docs/`, with the drafts and each plan's directory.
- **The checks:** formatting, lints, the build, and the `default` and
  `fuzzy` tests. A change's check before ready runs them in its
  workspace; the queue runs them in its gate lane.
- **The tiers:** `browser-app` (measured, CPU recorded), and `live-app`,
  which needs a ChatGPT sign-in.
- **The machines:** a hub that runs the dispatcher, the calls, the inbox
  page, the shelf and the status page; builders B1 and B2, four slots
  each; G, the gate machine, whose slot 4 is the tier runner's.
- **The shelf:** codex-1 and codex-2 (lent to B1 and B2), codex-3
  (free), claude-1 (the hub), chatgpt-1 (free).
- **At hour 0:** main is green in every tier; no plan is listed; the
  ladder plan exists on main with rungs r0 and r1 (r1 after r0), each
  on ChatGPT with the owned Forgejo, sharing `kit.md`.

## 4. Act 1: the start (hours 0 to 1)

### 4.1 The person starts Export and Themes

The person starts Export (`docs/drafts/export.md`) with the defaults:
an integration plan, ready, reviews off, direct, early starts. Then
Themes, with `reviews = true` and `direct = false`. Each start runs a
cut call and lists the plan.

- **Model:** 3, 4.1, 4.4, 10.1.

### 4.2 Export's cut releases early

Export's cut writes `notes/contracts.md` (the export record's shape)
first, then pushes plan.toml with three changes that need nothing: e1,
the record in kit; e2, the CSV writer in app; e3, the menu entry in ui.
It wakes the dispatcher, which merges the push into
`export/integration`, and e1 to e3 start at once while the cut writes
e4 to e9 and pushes them in two more batches. Its brief listed Themes'
changes and their paths; its plan.md says one overlap it could not
avoid: both drafts touch ui's menu.

- **Model:** 4.2, 4.3, 6.1, 6.7.

### 4.3 Themes' cut, reviewed, and a part parked

Themes' cut writes two skeleton parts, palette and dark, and a
question for the person: does dark mode follow the system? It names
part dark. The cut's branch gets a review run, which refuses it: a
brief names a package that does not exist. An amend call fixes it, the
review approves, and the plan is merged into `themes/integration`.
Part dark is not designed while the question is open. The person
answers "follow the system", and the chat writes the answer into the
plan on Themes' trunk.

- **Model:** 4.2, 10.2, 10.5, 11.1.

### 4.4 Themes' skeletons: a question on the forge, a settle

The skeleton call for palette ends `Question:`: keep the old colour
names as aliases? Its subject parks, and the question opens an issue on
the forge. The person comments "what do the aliases cost?"; a reply
call answers with the two options and a recommendation. The person
comments "drop them"; the next reply call records it as the answer,
the issue closes, and the skeleton call starts again with the answer
in its brief.

It writes palette's types with empty bodies and the part's briefs. The
dispatcher checks its tip (gates and tests), and t1 to t3 start from
it while its review runs. The review refuses it twice (a name it
dislikes, then a doc comment it calls missing); the second refusal
holds the chain, and a settle call says land: the findings are style,
left as later work. The skeleton merges into Themes' trunk. Dark's
skeleton call, its question answered, follows the same way, unrefused.

- **Model:** 10.1, 10.2, 10.5, 10.6, 11.1, 11.3.

## 5. Act 2: the trunks (hours 1 to 4)

### 5.1 Runs, as wide as the places

The dispatcher's target for each plan is its builders' places, 8: e1
to e3 on B1 and t1 to t3 on B2 start in claimed slots, each from its
trunk's tip, on a compiled brief, repairs first whenever there are
some. When B1 is later full with a slot
the person's chat claimed by hand, e5's start hands on to B2. Before
each start, the builder's fab install moves to the hub's commit.

- **Model:** 5.1, 5.2, 12.1.

### 5.2 A question the options settle, and an early start

e2 asks whether a quote in a field is doubled or escaped; its brief's
options say doubled. A foreman reads the brief and the report and
answers; the answer lands in e2's Answers and e2 runs again, from its
pushed branch.

e4, the export command in app, needs e2. While e2 runs again with its
branch pushed, e4 starts early: its slot is the trunk with e2's branch
merged in, and its brief names e2 as a branch to merge again before it
pushes. e4 merges into the trunk only after e2.

- **Model:** 5.1, 6.2, 10.3.

### 5.3 Done with something left, and merges

e1 is done, with a `## Left`: a lint in ui that its type change caused
outside its files. e1 and e3 merge; both lowered `debts.txt`, so the
merge conflicts there, and the dispatcher takes the trunk's side and
recounts. No one hears of it.

- **Model:** 5.1, 6.1.

### 5.4 A merge conflict, a disk wait, a failure

- **e6 and e5** both change app's command table. e5 merges first; e6's
  merge conflicts and is recorded as a rejection at `merge`. A refusal
  foreman writes a `## Rebase` section, and a rebase run resolves it.
- **e7's** first run waits on B2's disk: `heavy` reclaims idle slots
  and waits 20 minutes, then exits 75. The run reports `Infra: disk`,
  and the dispatcher starts it again; it is done.
- **e8's** run ends `Failed:` (its fixture generator crashes). It parks
  for the answerer at once, which says the generator is e9's and to
  stub the fixture; e8 runs again with fresh runs, and is done.

- **Model:** 5.1, 6.1, 10.3, 10.4, 10.5, 12.1.

### 5.5 Questions outside the change

e9 asks whether it may change ui's theme hook, which Themes owns. The
foreman's options do not settle it, so it queues, and the answerer runs
at once (an integration plan): leave ui to its owner and report it in
`## Left`. e9's next run is done with its work pushed but asks again,
about a ui test it cannot make green. The foreman answers `Accept:`
the change is done, its branch kept, and it merges.

- **Model:** 10.3, 10.4.

### 5.6 A check along the way, and its repair

At the tenth merge, the dispatcher checks Export's trunk in an idle
slot: the default tests are red in app. `repair-1` is made from the
record, its body the failing steps and the unmerged changes' paths. Its
run asks which of two failing tests is right; with no brief, the
question skips the foreman, the answerer answers, and the answer is
folded into repair-1's body. The next run is done, and merges.

- **Model:** 4.3, 6.3, 6.6, 10.4.

### 5.7 Drafts beside each other

Each tick, the overlap probe merges both trunks with main in memory,
then with each other: ui's menu conflicts. It records the overlap once,
and the status page shows Export and Themes with a count of one.
Nothing waits on it.

- **Model:** 6.7, 14.

### 5.8 Themes is ready first

Four hours in, with changes landed on its trunk, Themes' structure
review call reads their digests and writes one consolidation change,
merging two colour helpers; it starts ahead of Themes' other changes.

Themes' last change merges. The dispatcher catches its trunk up with
main (main has not moved), runs the end check, which is green, and
pushes `ready/themes/integration`.

- **Model:** 5.2, 6.4, 6.5, 10.2.

## 6. Act 3: landing (hours 3 to 6)

### 6.1 A batch with a hand-pushed branch, bisected

The person's chat pushes `ready/chat/readme` by hand, a doc fix with a
code sample. The queue takes it and Themes' trunk as one batch, merged
onto main in order, and runs the full checks: the lint step is red.
The batch is halved, left first: Themes' trunk alone is green and
lands. The chat's branch alone is red and is rejected.

Themes' trunk is the dispatcher's; the chat's branch is no plan's, so
the record gains `queue.rejected` and the inbox page opens
`queue:chat/readme` with its step and log. The chat fixes the sample
and pushes a newer tip, which clears the signal, and lands in the next
batch.

- **Model:** 7.1, 7.4, 7.5, 11.2.

### 6.2 Catching up, a conflict, a repair

Main has moved under Export. Thirty minutes after its last catch-up,
the dispatcher merges main into Export's trunk: ui's menu conflicts, as
the probe said. `catch-up-1` merges that main into a branch from the
trunk, keeping both sides' intent, fixes the app code Themes' palette
broke, and merges; the dispatcher checks the trunk now contains that
main, and records a clean catch-up. The overlap closes with Themes'
trunk landed.

Then main moves again, and the next catch-up conflicts in
`Cargo.lock`. `catch-up-2` merges, but its run had reset its branch, so
the trunk still lacks that main: a refusal at a main whose repair has
merged, and the plan stops. The person's chat merges main into the
trunk by hand and pushes it; the next tick records a clean catch-up,
and the stop clears.

- **Model:** 6.4, 6.5, 6.7.

### 6.3 Export's end check, red, then ready

The last change merges. The end check is red: a fuzzy test in app.
`repair-2` fixes it; the trunk is checked again, green, and readied.

- **Model:** 6.5.

### 6.4 The gate short of disk, then a rejection

The batch holding Export's trunk stops at the build: G's disk wait ran
out. The batch is an error, nobody is rejected, and it goes again.
Its next attempt is red: a ladder repair (8.2) that landed meanwhile
changed kit's error type, and Export's app code no longer builds on it.
The trunk is rejected. The dispatcher catches it up with main, checks
it (red, `repair-3`, green), and readies it again. It lands.

- **Model:** 6.5, 7.1, 7.5.

## 7. Act 4: after landing (hours 6 to 8)

### 7.1 One run for two tips

The tier runner was busy on an older main when Export's trunk and the
chat's readme fix landed. When it ends, it runs the newest tip once:
its suspects are both merges. In `browser-app`, two export tests fail,
and fail again alone: new reds. One test passes on its rerun: a flake.
One story runs past its wall limit: starved, counted apart, not red.
`live-app` cannot borrow a ChatGPT sign-in (chatgpt-1 is lent to a
walk): the pair faults, and the run is partial.

- **Model:** 8.1, 12.2.

### 7.2 The red goes to its owner

The dispatcher opens `red-1` for the two tests, with both suspects,
writes its evidence, and starts a red foreman, since Export's trunk is
a plan's. It says fix, owner e4: the export command drops a header row
under one locale. `red-1` starts from main in a slot: it reproduces in
a unit test, red before and green after, then fixes. Its precheck
passes, and it is readied. It asks nothing; had its two runs missed, it
would have gone to the answerer.

The `live-app` fault opens `fault:live-app` in the inbox.

- **Model:** 8.2, 10.3, 11.2.

### 7.3 Collateral, a receipt, and a recovery

`red-1` and `finding-1` (8.2) are ready together and share a batch. A
`fuzzy` test fails in app, which passed in both members' prechecks; run
alone, it passes. The batch is collateral: nobody is rejected, and it
goes again, green, and lands.

The next tier run covers `red-1`: both tests pass, recovered, and
`red-1` closes. The standing reds of the run before started nothing.
chatgpt-1 is back on the shelf, `live-app` runs, and `fault:live-app`
clears.

- **Model:** 7.1, 7.3, 8.1, 8.2.

### 7.4 A red no plan owns, and one unfixed

A later run turns `browser-app`'s menu test red behind the chat's
second hand-pushed branch alone: `red-2` opens as a signal, no change
being a plan's. The chat reads the evidence, fixes it by hand, and
clears `red-2` with a reason.

Earlier, `red-3`, a red the foreman routed to e3, landed its repair,
but the covering run still lists the test: `red-3` is unfixed, a
signal, which the chat takes up.

- **Model:** 8.2, 11.2.

### 7.5 Export retires

Export's parts are done and its trunk has landed. Its retire waits for
a tier run covering the trunk, with no red of Export's open (`red-3`,
a signal, does not hold it). The retire call brings the status files of
the designs Export changed current, marks the plan retired, and its
branch lands through the queue.

- **Model:** 8.3, 10.6.

## 8. Act 5: QA (hours 4 to 10)

### 8.1 A walk, its findings, triage

The chat reads the inbox page's ladder: r0 is due, a first walk. It runs
`fab qa walk` for r0 at main's tip, borrows chatgpt-1, gives the brief
to its walker, and reports with `fab qa result`: fail, with two
findings. f1, blocking: the export's CSV header is wrong. f2: the
history page is slow.

f1 is blocking, so the triage call runs at once: f1 path, owner app;
f2 backlog. The dispatcher renders the backlog file from the record and
readies `ladder/harbor/backlog-1`. The queue rejects it on a lint flake;
the next round readies `backlog-2` at the same render, which lands. A
second rejection of that render would have raised `backlog:harbor`.

- **Model:** 9.1, 9.2, 9.3, 12.2.

### 8.2 The repair, and the pass

A finding foreman reads f1's record and the walk's report: fix, the
header comes from the wrong locale. `finding-1` starts from main,
reproduces in a world test, fixes, and is readied (7.3). r0 holds until
it lands; then r0 is due again at a main containing it.

The chat walks r0 again at the same edition: pass, with one sighting,
`same` f2, merged into it. The dispatcher records `rung.passed`; f1 is
fixed.

- **Model:** 9.3, 9.4, 10.3.

### 8.3 A brief at fault, a new edition

r1's first walk is void: the box's setup failed. Its second fails: the
walker's model beat r1's gate by guessing a pull request's number, and
f3 says so. Triage says brief r1: a question to the ladder's owner, the
person, in the inbox and on the forge. The person answers "make the
gate name the pull request's opening time"; f3 is dropped with that
reason, and the chat lands the new brief. r1's edition changes, and its
first walk at it is due.

- **Model:** 9.1, 9.3, 11.1.

### 8.4 Again, not the code's, and the end

r1's walk at the new edition fails with two findings:
- f4 is the header again, under another locale: the walker names f1,
  whose repair landed, so f4 is filed `again`, and f1 is unfixed.
  Triage puts f4 on the path, and its foreman says fix. `finding-4`'s
  first two runs end `Failed:` (the locale table is generated); at its
  limit of two it parks, the answerer points it at the generator, and
  its fresh run is done. The queue rejects it at the lints: its refusal
  foreman says fix, the section folds into its body, and its next run
  lands it. The ladder, a queue plan, halves its target of runs on that
  rejection. On its next ready a port flake rejects it again; the
  foreman says retry, and the same tip is readied again. It is alone in
its batch, its receipt green for that tip and main, so the queue lands
it on the receipt after the counted baselines and formatting.
- f5, non-blocking: the box timed out reaching the forge. Its foreman
  says not the code's: `finding-5` is a signal in the inbox.

r1's rerun at a main holding f4's repair passes. `finding-5` clears
with the pass.

- **Model:** 5.1, 5.2, 7.2, 7.5, 9.2, 9.3, 9.4, 10.3, 10.4, 11.2.

### 8.5 The machines and the reports

- **A login swap:** the shelf's half-hourly check shows codex-1 at 92%
  of its weekly limit; the chat swaps B1 onto codex-3, and codex-1 goes
  back on the shelf.
- **A lost wake:** B2 cannot reach the hub when a run ends; the next
  backstop tick collects it and notes the lost event. The run's unit
  had gone with a restart of B2, so its slot's claim is stale, and the
  dispatcher releases it.
- **The next morning,** the daily code report shows each plan's runs,
  questions, idle gaps and weights, and the landing report its batches,
  the collateral batch, the bisection, the reds by state and the
  ladder's walks and passes; the status page showed the overlap, the
  questions waiting and the subscriptions' pace throughout.

- **Model:** 3, 12.1, 12.2, 14.

## 9. What temper and jig have, and lack

Mechanic by mechanic, in the model's order. "Have" is what temper and
jig hold today at fabrig 2075ff38b, built unless marked *designed*;
"lack" is what the case needs beyond it. Unqualified documents are
temper's `../domain/` ones.

| Mechanic | Model | Steps | temper and jig have | They lack |
|---|---|---|---|---|
| One writer, state folded from records, event wakes, a backstop | 3 | 4.1, 8.5 | one engine as the store's only writer, each decision a journalled commit; inbox, queue and turn derived, never stored; timers (jig's README 6, `domain/engine.md` 5; `root.md` 5.7) | records of the case's own kinds: walks, findings, reds, overlaps |
| A draft started; the cut: a plan of parts, slices, questions and briefs, released early | 4.1, 4.2 | 4.1 to 4.3 | the chat's `goal`, which plans, asks its requester and makes changes in batches (`agent.md` 4.3.2, 8.2; jig's `domain/core.md` 7.1, `domain/tasks.md` 4); notes (jig's `domain/engine.md` 10) | a one-shot design call whose output is a plan and briefs as files, released in pieces; drafts beside it in its brief |
| Changes, briefs with options, repairs made from records, a plan's options | 4.3, 4.4 | 4.1, 5.6 | a change with its producer, repairs and resolutions (`forge.md` 8); briefs compiled from sections (jig's `domain/engine.md` 9) | pre-approved options in a brief; repairs folded from records; per-plan landing options |
| A run: its outcome line, `Left`, a rerun from its pushed branch, a run limit | 5.1 | 5.1 to 5.5, 8.4 | a producing task on a worker; finished, parked or failed with failure classes; follow-ups; a retried producer continuing on its pushed branch; bounded tries reset by a release (jig's `domain/tasks.md` 5.5, 5.6; `worker.md` 5) | `Infra:` as an outcome restarted without anyone; a `Left` list the trunk's checks take up |
| Places, a flow target, calm builders, the order of starts, a start handed on | 5.2 | 5.1, 8.4 | fleet slots, fenced attempts, placement preferring a warm worker, a busy one placed again (jig's `domain/engine.md` 8) | a target of runs in flight that grows and halves; calm; repairs first |
| Merging into a draft's trunk, a counted baseline recounted, a conflict to a rebase | 6.1 | 4.2, 5.3, 5.4 | a goal's own branch landed whole (`forge.md` 11.1, *designed*); a conflict as a resolution (`forge.md` 8.4); the project's prepare tooling (`landing.md` 5.3, *designed*) | a trunk each done change merges into, dependents starting from it, the recount at a merge |
| Early starts | 6.2 | 5.2 | a procedure started kept beside its dependency (jig's `domain/connectors.md` 6.1, *designed*) | a start on a need's pushed branch, merged again before the push |
| Checks along the way, repairs and their questions | 6.3, 6.6 | 5.6 | a red base pausing the queue for one repair change (`forge.md` 9) | periodic checks of a trunk; repairs naming the unmerged changes; answers folded into a repair |
| Catching up with main, its repairs, a stop | 6.4 | 6.2 | updates by the forge's merge, a conflict as a resolution, bounds that hold a change (`forge.md` 10, 8.4) | a timed catch-up of a trunk; a check that main is contained; a plan's stop cleared by a hand merge |
| The end check, ready, a re-check after a rejection | 6.5 | 5.8, 6.3, 6.4 | a goal branch landed as a change, with CI and bounded repairs (`forge.md` 8.2 to 8.4, 11.1) | a full check of the trunk before ready; a re-check after the queue rejects it |
| An overlap probe between drafts | 6.7 | 4.2, 5.7, 6.2 | a landing waking a goal whose changes' files it overlaps (`forge.md` 7.3, in part) | a merge probe of each pair of trunks, recorded and shown |
| The queue: batches, selection, an exclusive gate lane, the push, an error | 7.1 | 6.1, 6.4, 7.3 | the turn: bases read again, fast-forward only, one per project (`landing.md` 8, on Forgejo); selection by keys and ignored paths (`landing.md` 6.1); a queue per landing branch (`forge.md` 9) | batches of unrelated changes; a full selection every fifth batch; an exclusive lane; an error that rejects nobody |
| The check before ready and its receipt reused | 7.2 | 8.4 | receipts by key, reused by every member, never run twice (`landing.md` 6) | a check before ready in the change's workspace, and a receipt from it |
| Collateral reds | 7.3 | 7.3 | a runner's fault kept apart from a failure (`landing.md` 7.4); a gate's flake rerun when it matches an open finding (`findings.md` 5, *designed*) | a failed test retried alone before anyone is blamed, with its bound |
| Bisection of a red batch | 7.4 | 6.1 | bisection of a red tier run (`after-landing.md` 5.2, *designed*) | bisection at the gate |
| Who hears a rejection: the refusal foreman, a retry, a hand-pushed branch | 7.5 | 6.1, 6.4, 8.4 | a real failure as a repair under the member's change (`landing.md` 7.3); a held change in the person's inbox (`agent.md` 8.2) | a judge choosing fix or retry; an unchanged tip readied again; a signal for a branch no goal owns |
| Tiers following main | 8.1 | 7.1, 7.3 | tiers on the newest tip, failures rerun alone, new, inherited and flaky, faults (`after-landing.md` 2 to 5, *designed*; only `Check::Tier` built) | the tiers built; CPU and starvation; recovered and unattributed; live pairs on borrowed sign-ins |
| A red reaching its owner | 8.2 | 7.2 to 7.4 | a red to the landing's goals, a repair naming it, cleared by a passing run; reds of no goal to findings (`after-landing.md` 5.3, 7, *designed*) | grouping by tip and suspects; a red's judge; a repair from main made by the system; unfixed; signals |
| A plan retired after its verdict | 8.3, 10.6 | 7.5 | a change living to cleared (`after-landing.md` 6, *designed*) | a plan waiting for its trunk's verdict; a retire call |
| The ladder, editions, a walk entered by command | 9.1, 9.2 | 8.1, 8.3 | the ladder as a process (`../qa.md`; `testing.md` 2.7); a walk with a sign-in in custody (`findings.md` 8, *designed*) | rungs and editions as records; a walk's start and result as records |
| Findings, merges, again, triage, the backlog | 9.3 | 8.1 to 8.4 | finding records matched by signature, triaged to a goal, the backlog or a drop (`findings.md` 2, 4, 6, *designed*) | merges by the walker's word; again; the person's and the brief's decisions; a backlog file landed |
| A finding's repair, rung states, the pass | 9.4 | 8.2, 8.4 | a routed finding as news to its goal, closed by its fix once cleared (`findings.md` 6, 7, *designed*) | a judge then a repair from main; rung states; a pass on an unchanged rerun; unfixed |
| Calls | 10.1 | 4.1 to 4.4 | finite agent tasks on a compiled brief, with budgets of turns, spend and time; report-only advisors (jig's `domain/engine.md` 9; `agent.md` 4.3.2, 4.6) | a call's peak context measured |
| Review chains, settle, a landing's refusal, the structure review | 10.2 | 4.3, 4.4, 5.8 | a separate read-only reviewer; changes requested as a repair; a failed review asked once more, then held (`forge.md` 8.3) | settle (land, amend, the person); a landing's refusal counted; a structure review writing consolidations |
| The foreman | 10.3 | 5.2, 5.4, 5.5, 7.2, 8.2, 8.4 | escalations and proposals decided by the nearest holder; a person task choosing among options (jig's `domain/tasks.md` 9, `domain/people.md` 8) | settling from a brief's options with no one asked; `Accept:`; a read-only judge of refusals, reds and findings |
| The answerer and parks | 10.4, 10.5 | 4.3, 4.4, 5.4 to 5.6, 8.4 | a task's own question parks its run, spending no try; wakes batched after N or T (jig's `domain/tasks.md` 5.7, 8.3); a conversation answering its delegates (8.5, *designed*) | one call answering a plan's queued questions; a part parked on a plan's question |
| Skeleton and contract calls | 10.6 | 4.4 | nothing | design calls per part, their tips checked, changes started from them |
| The inbox page, signals, answers and clears | 11.1, 11.2 | 4.3, 6.1, 7.2, 7.4, 8.3, 8.4 | a party's inbox of questions, proposals, escalations, person tasks and results, answered with `temper ctl task answer` (jig's `domain/people.md` 6, 8) | signals as a kind, opened and cleared from records |
| Questions on the forge | 11.3 | 4.4 | a question as an issue, the first comment its answer (`forge.md` 12.1, *designed*, Later) | a reply call that converses before it records |
| Workspaces, disk, lanes, stale claims | 12.1 | 5.1, 5.4, 8.5 | a checkout cache by workstream, one operation at a time, released or discarded (`worker.md` 5; jig's `domain/hosts.md` 7) | disk admission that waits and reclaims; a sweep that bounds a workspace; lanes |
| The shelf | 12.2 | 7.1, 8.1, 8.5 | accounts with holders, sources and states, limited until a reset; a claim records its account; moves out and in (jig's `domain/accounts.md` 1 to 6) | lending to a run and returning; a swap; checks of every limit |
| Reports and the status page | 14 | 5.7, 8.5 | views of tasks and what they wait on (jig's `domain/engine.md` 11); spend per period (jig's `domain/authority.md` 7); `ctl landing show` and `list` | daily code and landing reports; a public page of counts; weights |

## 10. Left out, and why

- **`fab land`** (model 7.6): it lands fab's own repository, one branch
  at a time; a landing of one through the queue, as in 8.4, is the
  mechanic temper meets.
- **A replay** (`ready = false`): a measurement of fab, not a way to
  land.
- **The decision call:** started by hand, and its probe is not built.
