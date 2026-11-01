# The reference case

Draft, 2026-11-01, fourth edition, not adopted until the user approves
it. One invented story, as small as it can be while it runs every
mechanism of `code-pipeline.md` (C) and `landing-pipeline.md` (L) under
fabrig's configuration (C 5, L 6; Themes turns two settings on), with
fab's cast, so that someone who knows fab recognises each role and step.
Nobody works by hand: Ines talks to her chat through `temper ctl` and
answers her inbox there; the rest is jig's tasks and temper's connector.
Section 9 maps the story onto fab; section 11 maps each mechanism onto
jig's primitives and temper, and says what is missing: the input to the
execution plan. Machines, slots and sign-ins are left out for now.

## 1. In one page

- **One project, Harbor,** one repository on an owned Forgejo.
- **Ines and her chat,** a director; every other role is a jig task: the
  goals' agents, producers, reviewers, verdict tasks (examiners),
  walkers, and temper's procedures and jobs.
- **Four goals:** Export and Themes, from two drafts, each on a goal
  branch; Docfix, a small fix; and the ladder, Harbor's QA.
- **Five acts, 26 steps, about 10 hours,** overlapping in time.

## 2. How to read it

Each step says what happens, then **Shows:**, the sections of the two
pipelines it shows. Times are hours from the start. On fakes, timers and
bounds are scaled down and each agent's result is scripted as the
oracle; on real backends, a QA driver types Ines's words in `temper
ctl`.

## 3. The project and the start

- **The repository,** `harbor`: packages `kit`; `ui`, which reads kit;
  `app`, which reads kit and ui; `debts.txt`, a counted baseline
  recounted by `tools/count`; `docs/`, the project's designs and drafts.
  temper's data lives in its store, never here.
- **The gates:** formatting, lints, the build, the `default` and `fuzzy`
  tests. **The tiers:** `browser-app`, and `live-app`, which needs a
  ChatGPT account granted for its run.
- **At hour 0:** main is green in every tier. The ladder goal, which
  Ines's chat started from the QA kit, is open: rungs r0 and r1 (r1
  after r0), each on ChatGPT with the owned Forgejo.

## 4. Act 1: the start (hours 0 to 1)

### 4.1 Ines asks for Export and Themes

Ines tells her chat that Export's draft is in, and then Themes'. The
chat starts two goals with its `goal` tool: Export with fabrig's
settings (its own branch, early starts, review off, no scaffolds), and
Themes with `review` on and scaffolds, which its template allows.

- **Shows:** C 2, C 5.

### 4.2 Export's plan, delegated early

Export's agent, under the planning charter, delegates a read-only survey
of kit, writes the export record's shape, then delegates the plan's
first batch: three changes that depend on nothing, with their specs (e1,
the record in kit; e2, the CSV writer in app; e3, the menu entry in ui).
They start at once, while it plans e4 to e9 in two more batches. Its
brief listed Themes' changes and paths; it names the overlap it could
not avoid: both goals touch ui's menu.

- **Shows:** C 3.1, C 3.2, C 3.6.

### 4.3 Themes' plan, reviewed, and a part kept

Themes' agent plans two parts, palette and dark, each with a scaffold
first, and a person task for Ines: does dark mode follow the system?
Dark's changes depend on it. A verdict task reviews the first batch and
refuses it (a spec names a package that does not exist); the agent
revises it, the review approves, and the batch starts. Ines answers in
her inbox.

- **Shows:** C 3.1, C 3.5.

### 4.4 Palette's scaffold asks on the forge

Palette's scaffold producer asks whether to keep the old colour names as
aliases. Themes' agent cannot settle it and asks Ines; the question also
opens an issue on the forge. Ines comments "what do the aliases cost?";
a reply task answers with the options and a recommendation. Ines
comments "drop them"; the next reply task records the answer, and the
issue closes. The scaffold runs again with it.

- **Shows:** C 3.3, C 4.

### 4.5 Two scaffolds: a hold, a resolution

Palette's scaffold lands into Themes' branch after its review: refused
once (a type's doc contradicts its design; a repair fixes it), refused
again (a body left to guesswork), held, and escalated to Themes' agent,
which says land. t1 to t3, started beside it, land after it. Dark's
scaffold conflicts with palette's in the branch: a resolution settles
the conflicting files, with no git writes of its own, and it lands.

- **Shows:** C 3.1, C 3.4, C 3.5.

## 5. Act 2: the goal branches (hours 1 to 4)

### 5.1 Producers, and a question the options settle

Each change's procedure delegates its producer, an agent task in a
workspace, from the goal branch's tip. e2 asks whether a quote in a
field is doubled or escaped; a verdict task finds the spec's options say
doubled, and e2 runs again from its pushed branch.

- **Shows:** C 3.2, C 3.3.

### 5.2 Early starts

e4, the export command, depends on e2 and starts beside it, from the
goal branch with e2's branch merged in; the procedure merges e2's final
branch into it again, and e4 lands after e2. e5 depends on e3, but both
lowered `debts.txt`: e5's early start is refused, and it waits.

- **Shows:** C 3.4.

### 5.3 Finished with something left, and landings

e1 finishes, leaving a lint in ui that its type change caused outside
its paths, and lands into Export's branch. e3 lands too; both lowered
`debts.txt`, and the project's tooling recounts. e3's landing starts e5.

- **Shows:** C 3.2, C 3.4.

### 5.4 A conflict, a lost run, a failure

- **e6** conflicts with e5 in app's command table: a resolution settles
  it, and e6 lands.
- **e7's** worker restarts mid-run: the run is lost, a transient failure
  by its class, and runs again, a try counted.
- **e8's** run fails (its fixture generator crashes): it is held and
  escalated to Export's agent, which says the generator is e9's and to
  stub the fixture; e8 runs again.

- **Shows:** C 3.2, C 3.3, C 3.4.

### 5.5 Questions outside the change

e9 asks whether it may change ui's theme hook, which Themes owns. The
verdict task passes it on; Export's agent wakes at once (a goal
branch's): leave ui to its owner and list it as left. e9 pushes its work
and asks again about a ui test it cannot make green; the agent accepts
the change as it stands, and it lands.

- **Shows:** C 3.3.

### 5.6 An interim check, and its repair

At the tenth landing, a job checks Export's branch without the fuzzy
tests: red in app. A repair of the goal's branch is made, its spec the
failing gates and the unlanded changes' paths. It asks which of two
failing tests is right; Export's agent answers; it lands.

- **Shows:** C 3.3, C 3.4.

### 5.7 Goals beside each other

Each round, both goals' branches are merged with main in memory, then
with each other: ui's menu conflicts. Both goals hear it once; `temper
ctl`'s status shows the overlap. Nothing waits on it.

- **Shows:** C 3.6, C 4.

### 5.8 Themes is ready first

Themes' structure review reads what landed and makes one consolidation
change, which starts first. Themes' last change lands; a job runs the
final check, green, and Themes' branch is ready.

- **Shows:** C 3.4, C 3.5, L 2.

## 6. Act 3: landing (hours 3 to 6)

### 6.1 A small fix, a candidate, bisecting

Ines tells her chat that a doc comment in `ui/src` names the wrong
function. The chat starts Docfix, a goal of one change into main; its
check before ready is green. The queue prepares it and Themes' branch as
one candidate, its fifth, so in full: the doc lint is red, since Themes
renamed the function. Halved, left first, in full: Themes lands.
Docfix's change, alone on the new main, is red; a verdict task says fix;
it is repaired and lands.

- **Shows:** C 2, L 2, L 3.1, L 3.3, L 3.4.

### 6.2 Updates, a resolution, a stop

Thirty minutes after its last update, Export's branch takes main by an
update: ui's menu conflicts, as the overlap said. A resolution settles
it. The next interim check finds app code Themes' palette broke; a
repair fixes it. Main moves again, and three updates conflict in a lock
file the resolution cannot settle: Export's branch is held, escalated to
Export's agent, which starts a resolution that regenerates the lock file
with the project's tool; it lands, and updates go on.

- **Shows:** C 3.4.

### 6.3 Export's final check, red, then ready

The last change lands; the final check is red in a fuzzy test; a repair
fixes it; green, and Export's branch is ready.

- **Shows:** C 3.4, L 2.

### 6.4 A fault, then a rejection

The candidate holding Export's branch stops short of disk: a fault of
the gate, nobody rejected, and it goes again. Next it is red: a ladder
repair (8.2) that landed meanwhile changed kit's error type. Export's
branch is updated, checked (red, repaired, green) and readied again; it
lands.

- **Shows:** L 3.1, L 3.4, C 3.4.

## 7. Act 4: after landing (hours 4 to 8)

### 7.1 Themes' red, a flake at the gate, unfixed

A tier job on Themes' landing: a contrast test fails, and again alone.
The red is news to Themes; a verdict task says fix, owner t2; a repair
from main reproduces it in a unit test, fixes it, and is ready. It
shares a candidate with `finding-1` (8.2): a fuzzy test fails that
passed in both receipts, then passes alone, a flake at the gate; the
candidate goes again and lands. The covering tier run still has the
contrast test: unfixed, news again, and Themes has since closed, so it
goes to the chat, which starts a goal for it.

- **Shows:** L 3.2, L 4.1, L 4.2, C 4.

### 7.2 One run for two tips

The tier job was busy when Export's branch and Docfix landed; its next
run covers both: two export tests and a story never run before fail
again alone (new and unattributed); one test passes alone, a flake; one
story runs past its wall limit, starved, not red; `live-app` cannot be
granted a ChatGPT account (a walk holds it) and faults. The run is red
with a fault, so its tip runs again; the fault is news to the chat,
which waits.

- **Shows:** L 4.1, L 4.2, C 4.

### 7.3 Export's red reaches its owner

The red is news to Export; a verdict task says fix, owner e4; the
repair, from main, is ready alone, its receipt green for that tip and
main, and lands on it. The next tip's run passes all three tests while
`live-app` faults again: partial, the red closes; the run after it, with
the account back, clears the fault.

- **Shows:** L 2, L 3.1, L 4.1, L 4.2.

### 7.4 Export ends

Export's changes have landed and a tier run covers them with no red of
Export's open. In its closing run, Export's agent brings the status
files of the designs it changed current as one last change, and the goal
ends.

- **Shows:** C 3.7, L 4.3.

## 8. Act 5: QA (hours 4 to 10)

### 8.1 A walk, its findings, triage

r0 is due: its walk task runs on what a job built at main's tip, with
chatgpt-1 granted, under the tester's charter. It reports fail with two
findings: f1, blocking, dark mode ignores the system's setting; f2, the
history page is slow. The ladder's agent triages at once: f1 on the
path, owner ui; f2 backlog, shown in `temper ctl` and rendered into the
project's debt document.

- **Shows:** L 5.1, L 5.2, L 5.3.

### 8.2 The repair, and the pass

A verdict task reads f1: fix, kit reads the setting with the wrong error
type. `finding-1`, a repair from main, lands (7.1). r0's rerun at the
same edition passes, meeting f2 again (merged into it); r0 passes and f1
is fixed.

- **Shows:** L 5.3, L 5.4.

### 8.3 A brief at fault, a new edition

r1's first walk is void (the box's setup failed); its second fails: the
walker beat r1's check by guessing a pull request's number. Triage says
the brief is at fault; the ladder's agent revises r1's spec, a new
edition, and r1 is due.

- **Shows:** L 5.1, L 5.3.

### 8.4 Again, tries, not the code's

r1's walk fails with f4, dark mode again on the history page: met after
f1's repair landed, it is filed again, and f1 is unfixed. `finding-4` is
rejected at the lints twice (each time a verdict task says fix); out of
tries, it is held and escalated to the ladder's agent, which points it
at the page's own theme reader; it is ready again, a port test fails
twice alone, a verdict task says retry, and the same tip lands. f5, the
box timing out on the forge, is triaged to the path; its verdict task
says not the code's: news to the chat, cleared when r1's rerun passes.

- **Shows:** C 3.2, L 3.4, L 5.2, L 5.3, L 5.4.

### 8.5 The reports

At midnight a recurring task writes the day's report for the chat;
`temper ctl`'s status showed the overlap and the questions waiting
throughout.

- **Shows:** C 4.

## 9. The story in fab's terms

Kind: code or LLM. "Gap": fab does it by hand, by an agent where this
has code, by code where this has an agent, or with temper's data in the
repository (C 6, L 7).

| Story | Kind | fab | Shows |
|---|---|---|---|
| Ines and her chat (4.1) | LLM | the user and the director; gap: the cut started by hand | C 2 |
| a goal's settings (4.1) | code | `plan.toml` options (gap: in the repository) | C 5 |
| the goal's agent planning (4.2) | LLM | the cut | C 3.1 |
| batches delegated, started (4.2) | code | gap: the cut pushes them and wakes the tick | C 3.1 |
| specs, changes (4.2) | LLM | task specs, tasks | C 3.1 |
| a person task for Ines (4.3) | code | a `[[question]]` (gap: answered in plan.toml) | C 3.1 |
| a plan's review (4.3) | LLM | the call review run | C 3.5 |
| a hold escalated, the holder deciding (4.5) | LLM | settle | C 3.5 |
| a repair of remarks (4.5) | LLM | amend | C 3.5 |
| scaffolds, started beside (4.4, 4.5) | LLM; code | skeleton calls, `call.checked`; gap: task rebases | C 3.1 |
| questions on the forge (4.4) | code; LLM | `fab issues sync`, the reply call | C 4 |
| the change procedure, producers (5.1) | code; LLM | implementer runs; gap: they push themselves | C 3.2 |
| verdict tasks (5.1, 6.1, 7.1, 7.3, 8.2, 8.4) | LLM | the foremen | C 3.3, L 3.4, L 4.2 |
| the goal's agent answering (5.4 to 5.6) | LLM | the answer call | C 3.3 |
| early starts (5.2) | code | `early_starts`; gap: the implementer merges again | C 3.4 |
| landing into the goal's branch, a recount (5.3) | code | the merge into `<draft>/integration` | C 3.4 |
| resolutions (4.5, 5.4, 6.2) | LLM | rebase and catch-up runs; gap: they rebase and push themselves | C 3.4 |
| a transient class from the run (5.4) | code | gap: from the run's own `Infra:` | C 3.2 |
| held past tries (5.4, 8.4) | code | a park at the run limit | C 3.2 |
| interim and final checks (5.6, 5.8, 6.3) | code | `fab check` | C 3.4 |
| overlaps (5.7) | code | `trunks.overlap` | C 3.6 |
| a structure review (5.8) | LLM | the `review` call | C 3.5 |
| a stop, the goal's agent (6.2) | LLM | gap: a note, then a hand merge | C 3.4 |
| Docfix, a goal (6.1) | LLM | gap: a hand-pushed ready ref | C 2 |
| ready, the check before it (5.8, 6.1) | code | gap: `ready/*` refs; the implementer runs `fab precheck` | L 2 |
| the queue, candidates, bisecting (6.1, 6.4) | code | `fab queue`, batches | L 3.1, L 3.3 |
| a flake at the gate (7.1) | code | a collateral red | L 3.2 |
| tier jobs (7.1 to 7.3) | code | `fab after-landing run` | L 4.1 |
| a red as news to its goal (7.1, 7.3) | code | red routing; gap: the repair goes to the newest suspect's plan | L 4.2 |
| news to the chat (7.1, 7.2, 8.4) | code; LLM | gap: signals on a page, read by a session | C 4 |
| the closing run (7.4) | LLM | the retire call (gap: its branch never reaches main) | C 3.7 |
| walk tasks (8.1 to 8.4) | LLM; code | gap: walked by a session, `fab qa walk`, `fab qa result` | L 5.2 |
| triage by the ladder's agent (8.1, 8.3, 8.4) | LLM | the triage call | L 5.3 |
| the backlog (8.1) | code | a backlog branch (gap: never taken by the queue) | L 5.3 |
| a rung's spec revised (8.3) | LLM | gap: brief files edited by hand | L 5.1 |
| the pass (8.2) | code | `rung.passed` | L 5.4 |
| reports (8.5) | code | gap: `fab report daily` by hand | C 4 |

## 10. Left out, and why

- **Machines, slots and sign-ins** (the user, 11-01): run targets, calm
  workers, disk waits, workspace sweeps, account custody and swaps.
- **fab's own repository's landing** (L 6): a configuration, left out at
  the user's word.
- **A replay:** a measurement, not a way to land.
- **Not shown, for brevity:** the bound on flakes at the gate; triage's
  `user` and `merge`; a red not the suspects'; a conflict at `prepare`;
  a backlog document rejected; the queue's bound on rejections.

## 11. jig's primitives, temper, and what is missing

For each mechanism: the jig primitives that carry it (jig's `domain/`
documents), what temper adds (configuration, or its forge connector's
mechanism, `forge.md`, `landing.md`), what is built today at fabrig
2075ff38b, and what is missing: a **jig knob** (no new primitive) or a
**temper** mechanism. Judged for `temper ctl`.

| Mechanism | Kind | jig | temper | Today | Missing |
|---|---|---|---|---|---|
| A chat, a goal from it (C 2) | LLM | a chat from a template; a goal (`people.md` 7, `core.md` 6.2) | director template; `goal` tool | built, template 1 only (`agent.md` 4.3.2) | jig knob: a chat from any template in ctl |
| Planning in batches (C 3.1) | LLM | an agent delegating batches with specs and dependencies (`core.md` 7.1, `tasks.md` 3, 4) | a planning charter | built in jig; temper's goal plans and calls `change` | temper: a planning charter; `change` with typed spec and dependencies in a batch |
| A goal's settings (C 5) | code | settings, scopes (`engine.md` 17) | its kinds | four kinds only | jig knob: application kinds (17.5, designed), a scope under a project |
| A change and its producer (C 3.2) | code; LLM | a procedure delegating an agent; tries per class (`tasks.md` 5.3, 5.5) | the change procedure, producer charter | built (`forge.md` 8) | temper: a list of what was left |
| Questions, answers (C 3.3) | LLM | a question to the requester; kept; wake policies (`tasks.md` 5.7, 8.3) | answering in the goal's charter; a verdict task first | built | jig knob: waking on questions by count and age |
| A person's question (C 3.1, C 4) | code | a person task; the inbox (`people.md` 6, 8) | — | built | — |
| The goal's branch: landing, dependents, checks, updates, bounds (C 3.4) | code; LLM | dependencies; jobs; holds and escalation (`tasks.md` 5.3.1, 5.5, 9) | a branch per goal; checks on a cadence; timed updates | designed (`forge.md` 11.1); updates and resolutions built (`forge.md` 8.4, 10); jobs designed | temper: the goal's branch, its checks and timed updates; jig knob: jobs built |
| Early starts (C 3.4) | code | started kept beside a dependency (`connectors.md` 6.1) | `early_start` | designed | jig knob: built |
| Review, holds (C 3.5) | LLM; code | a verdict task; held and escalated (`tasks.md` 5.5, 9) | review gates, the reviewer's charter | built for changes (`forge.md` 8.3) | temper: a plan's review; a structure review |
| Overlaps (C 3.6) | code | news (`tasks.md` 8.1) | the overlap probe | in part (`forge.md` 7.3) | temper: merging goals' branches in memory |
| The goal's end (C 3.7) | LLM; code | closing (`tasks.md` 5.1) | the end duty in the goal's charter | built | — |
| Alerts, the forge, reports (C 4) | code; LLM | news to a chat's subscription; recurring tasks (`tasks.md` 8.4, 10) | topics; questions on the forge; a report | forge questions designed (`forge.md` 12.1) | temper: the topics, the forge's questions, the report; jig knob: a daily recurrence |
| Ready and its check (L 2) | code | a job (`tasks.md` 5.3.1) | readiness; receipts by key | receipts built (`landing.md` 6) | temper: the check before ready |
| Candidates, selection, the turn (L 3.1) | code | procedures, jobs | the landing queue | the turn built for one landing (`landing.md` 8) | temper: candidates of several, full runs on a cadence |
| A flake at the gate (L 3.2) | code | — | a rerun alone | designed (`findings.md` 5) | temper |
| Bisecting (L 3.3) | code | — | halves | absent | temper |
| A rejection (L 3.4) | code; LLM | a verdict task | repairs, resolutions | built (`forge.md` 8.4) | temper: retry |
| Tiers (L 4.1) | code | jobs; account grants (`accounts.md`) | tier jobs, verdicts | designed (`after-landing.md` 2 to 5) | temper: tiers; jig knob: jobs |
| Reds to their goals (L 4.2) | code; LLM | news; a verdict task | grouping, routing | designed (`after-landing.md` 7) | temper |
| The goal's verdict (L 4.3) | code | a task closing after its delegates | living to cleared | designed (`after-landing.md` 6) | temper |
| A ladder, walks (L 5.1, 5.2) | LLM; code | tasks with dependencies; spec revisions; grants (`tasks.md` 4, 7) | the tester's charter; a build job | designed (`findings.md` 8) | temper: rungs and editions; jig knob: credentials granted as accounts are |
| Findings, triage, the pass (L 5.3, 5.4) | code; LLM | records; the goal's agent | finding records; the backlog | designed as forge issues (`findings.md`) | temper: findings in the store, the pass |
