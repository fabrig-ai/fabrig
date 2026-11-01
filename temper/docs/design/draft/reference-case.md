# The reference case

Draft, 2026-11-01, fourth edition: the third (10-30), brought loosely in
line with how fabrig's lab runs today, at the user's word. Nothing here
is adopted. One invented story that applies `code-pipeline.md` (C) and
`landing-pipeline.md` (L) to a process like the one temper's own
development runs. It holds every mechanic of the two that such a process
uses, and is as small as that allows. temper must be able to run it.
Machines, slots and sign-ins are left out for now.

## 1. In one page

- **What it is:** the model at work in one project. Each step is
  something that happens, the sections that handle it, and what temper
  and jig lack for it.
- **The cast** (section 3): the person and their chat, a director;
  Export, a goal on a draft, landing through its own branch; Themes, a
  goal landing each change into main; Fix, a goal of a fix; the ladder,
  a goal of QA walks; and their delegates: producers, reviewers, verdict
  tasks, walkers. One repository, one prover, a tier runner, one owned
  Forgejo.
- **Five acts, 18 steps, about 9 hours:** a goal planned and landing
  through its branch (section 4); landing under load (section 5); after
  landing and QA (section 6); the person (section 7); a hold and the
  ends (section 8).
- **What it asks** (section 10): the gaps of C 6 and L 15.4 it uses, by
  id. Nobody works by hand: the person talks to the chat in `temper ctl`
  and answers their inbox there. Section 9 maps the story onto fab.

## 2. How to read it

- **Each step** says what happens, then **Model:** the sections that
  handle it, and **Needs:** what temper and jig must have, by gap id;
  *exact* where they have it today.
- **Time** is in hours from the start. On fakes, timers and bounds are
  scaled down, and each judgement is scripted as the oracle; on real
  backends, a QA driver types the person's words in `temper ctl`.

## 3. The cast and the start

- **The person** directs the work and answers what is theirs.
- **The director** is the person's chat, on Claude, made from the
  director template; a timer wakes it every 2 hours for its check.
- **Export** is a goal on Claude: its agent plans the draft "Export",
  which spans the packages kit and app, and lands through its own
  branch. **Themes** is a goal on Claude in the package ui, landing each
  change into main. **Fix** is a goal of one fix. **The ladder** is the
  project's QA goal: rungs r0 and r1, r1 after r0.
- **The delegates:** producers, reviewers, verdict tasks and walkers are
  agent tasks on GPT.
- **The repository,** works: packages kit; ui and cli, which read kit;
  app, which reads kit and ui; `tools/`, the project's tools with their
  suite; `debts.txt`, a counted baseline that may only go down; `docs/`,
  which no gate reads. temper's data lives in its store, never here.
- **The gates:** deterministic, per package: `fmt`, `lint`, `build`; of
  the repository: `count` and `tools`; measured, per package: `test`.
  **The tier:** `browser`, on app.
- **The settings** are fabrig's (C 5, L 13), but Themes lands each
  change into main, with its check before ready, and to show the
  mechanisms `before_ready` is `keys`, and main has an idle audit.
- **At hour 0:** Themes is running, nothing is ready, no finding is
  open, `browser` is green at main.

## 4. Act 1: a goal planned, landing through its branch (hours 0 to 2)

### 4.1 The director starts Export, which plans

The person: "Export's draft is in; it's next." The director starts
Export with its `goal` tool. Export's agent writes the shapes that cross
parts first, then delegates the plan's first batch, the changes that
depend on nothing: k1, the record in kit (an interface), and k3; while
it plans the rest, which it delegates in two more batches: k2 (after
k1), a1 and a2 (after k1). Its brief listed Themes' changes and their
paths.

- **Model:** C 2.1, C 2.6.
- **Needs:** a goal from the chat (exact), carrying its settings (J15);
  planning in batches with typed specs (L24); the goals beside it listed
  (L22).

### 4.2 Plain merges, an early start, a check before ready

k1 changes what a kit item means. Its producer finishes, and it lands
into Export's branch by a plain merge, with no gates: the branch's
checks come later (4.4). k2 depends on k1 and started beside it, from
k1's pushed branch; k1's final branch is merged into it again, and it
lands after k1. k3 finishes, leaving a lint in ui that its change caused
outside its paths, for its owner.

t1, a change of Themes' landing into main, runs the check before ready:
its head merged onto main, uncommitted, in its workspace; the
deterministic gates over ui and its readers. `lint` fails; the producer
fixes it, and the check passes; each key's result is a receipt, and so
are the tests' (`tests_before_ready`).

- **Model:** C 2.2, C 2.6, C 3, L 4.
- **Needs:** Export's branch and its plain merges (L21); a task started
  beside its dependency (J13); what a change left (L25); the check
  before ready (L11), keys per package (L5, L4), gate classes (L6).

### 4.3 Questions, the options, the person

a2 asks whether a field is escaped or doubled; its spec's options say
doubled. The change starts a verdict task, which finds that, and a2 runs
again. a1 asks whether it may change ui's menu, which Themes owns: the
verdict task passes it on, Export's agent wakes at once, and says leave
it to its owner. a1 pushes its work and asks again about a ui test; the
agent accepts it as it stands.

- **Model:** C 2.3.
- **Needs:** a change relaying its producer's question, a verdict task
  first (L25).

### 4.4 Export's branch: checks along the way, an update, ready

Forty-five minutes after its last check, a job checks Export's branch
without the fuzzy tests: red in app. A repair of the branch fixes it.
Thirty minutes after its last update, main has moved, and Export's
branch takes it by an update; kit's record conflicts with a change of
Themes'; a resolution settles the files and the procedure commits. Once
all have landed, the final check is green, and Export's branch is ready.

- **Model:** C 2.6.
- **Needs:** checks of a goal's branch as jobs (L21, J17), timed updates
  (L21); resolutions (exact).

## 5. Act 2: landing under load (hours 2 to 4)

### 5.1 A batch lands

Export's branch, Themes' t1 and t2, and Fix's f1 are ready together.
They join one candidate, oldest first: prepared by a merge chain on
temper's branch, keys per package; the keys their checks before ready
proved hold, and the rest run in one dispatch to the prover. It is
green; the turn fast-forwards main; each change's branch is released by
a deletion naming the head it landed; each goal hears its landing.

- **Model:** L 3, L 4, L 5.1, L 5.2, L 8.
- **Needs:** a batch (L1); a candidate on temper's branch (L3); the
  prepare and key job (L4); keys per package (L5), gate classes (L6); a
  runner's certificate (L7); the serializer (L9).

### 5.2 A red in a big batch

Candidate 4 holds six changes. An app `test` fails, and fails again
alone. Four of the six reach app's key, so only they are suspects; the
other two land without them. The four are bisected, left first: two
land; of the other two, one lands, and t5 alone is red. A verdict task
says it is the change's: a repair, with the failing step and log.

- **Model:** L 5.2, L 7.
- **Needs:** a measured failure rerun once (L8); suspects by key and
  bisection (L1); a verdict task on a rejection (L23).

### 5.3 Fresh by key, a flake at the gate

t6 was checked before ready at hour 2; since then Export landed, and ui
reads kit, so t6's ui keys changed: temper checks it again before it
joins. f3, a change of Fix's to `tools/`, keeps its receipt, its only
key unchanged. In the next candidate a fuzzy test fails that passed in
both members' receipts, and passes alone: a flake at the gate, nobody
rejected, and the candidate goes again.

- **Model:** L 3, L 7.
- **Needs:** freshness as key equality (L5, L11); a flake at the gate
  (L23).

### 5.4 Conflicts, by rule and by resolution

t7 and f2 each lower `debts.txt`; prepared together, the tooling
recounts it, as the candidate's own commit. t8 needs a hook in ui's menu
that t9, ahead in the same batch, changed: t8 is left out, the conflict
staged with the attempt, and a resolution under Themes settles it; t8 is
checked again and joins a later candidate.

- **Model:** L 5.1, L 5.2, C 2.6.
- **Needs:** the prepare's own commits (L3, L4); a conflict staged with
  its attempt (L3); a check again before ready (L11); a resolution
  (exact).

### 5.5 A fault, a red main already has, a key gap

The prover refuses candidate 9 short of disk: a fault, retried, nobody
rejected. Then a full proof of main is red in app's `test`: a fixture's
certificate expired. Candidate 10 fails in the same test; the red is
main's, so its changes return to the ready set, and the news goes to the
director, which starts a goal whose change renews the fixture; it lands,
and main's key is green. Later a full proof is red in cli where
selections passed: a cli test reads app's fixtures by path. Bisected in
full, it is a change of Themes'; the key gap is news to the director,
whose goal adds app's fixtures to cli's key in the tooling.

- **Model:** L 4, L 7, C 2.7.
- **Needs:** a fault kept apart (L7, L8); full proofs that audit, a red
  main already has, a key gap (L5, L8); news to a chat (J10).

## 6. Act 3: after landing and QA (hours 4 to 6)

### 6.1 Tiers follow main

`browser` is running on an older main when two candidates land; when it
ends, it runs the newest tip, covering both. A run lost when its runner
restarts is a fault, retried.

- **Model:** L 9.
- **Needs:** tiers following main, coalescing (L13); a tier's fault
  (L8).

### 6.2 A red reaches its owner

One run covers Export's branch, t10 and f4. Two tests fail, and fail
again alone. The red is news to each open goal among the suspects; a
verdict task reads the evidence and names the owner: Export's k2.
Export's agent starts a repair from main, which reproduces it in a unit
test and fixes it. The next run shows the same two failures, standing,
and nobody hears; the run after the repair passes them: recovered, and
Export hears the red is closed.

- **Model:** L 9, C 2.7.
- **Needs:** reds as news to the suspects' goals, by role (J10), a
  verdict naming the owner, standing reds, recovery (L13); test ids from
  the run (F1).

### 6.3 A flake, and its finding

A `browser` run fails one test that passes alone: a flake. Finding #7 is
recorded in the store with its signature and routed to Themes, whose
agent triages it to a repair. Before it lands, a candidate's `test`
fails in the same test and passes alone: counted on #7. The repair
lands, naming #7, which closes.

- **Model:** L 7, L 10.1.
- **Needs:** a rerun alone (L8), a flake at the gate (L23); findings as
  temper's records, matched by signature, triaged, closed (L15).

### 6.4 QA: a walk, its findings, the pass

r0 is due. The ladder starts a walk task on r0's spec, at main's tip,
holding a ChatGPT account for its length. It reports fail: #9, blocking,
Export's CSV header is wrong; #10, a page is slow. The ladder's agent
triages: #9 to a repair, owner Export; #10 to the backlog. Export's
repair lands; r0 is due again at a main holding it, its rerun at the
same edition passes, and #9 closes. r1's walk is void: the walker beat
its check by guessing a pull request's number. The brief is at fault: a
question to the person, who says how to tighten it; r1's spec is
revised, a new edition, and r1 is due.

- **Model:** L 10.2, L 10.1.
- **Needs:** a ladder goal of walks with editions, rung states, the pass
  (L16); triage to a repair and the backlog (L15).

## 7. Act 4: the person (hours 6 to 7)

### 7.1 A question to the person

t11 asks how long a suite may take: a budget. Themes, landing each
change, wakes for its questions at 4 or after 30 minutes, then asks the
person in their inbox. The person answers "under a minute" in `temper
ctl`; the answer reaches Themes' agent, which answers t11, and the
director hears a notice.

- **Model:** C 2.3.
- **Needs:** waking on questions by count and age (J14); any task asking
  the person, the answer reaching the asker (exact).

### 7.2 Questions to the chat stay with the chat

Themes asks the director, its requester: "dark mode first, or high
contrast?" The director answers from its notes. Later it asks "may ui's
API break the old callers?"; the director asks its person, who answers,
and the director answers Themes in its own words.

- **Model:** C 2.3.
- **Needs:** a question to the chat staying with it (J7); the chat asks
  its person (J8).

### 7.3 Parked until the person's draft

Themes' next part needs the person's draft on palettes. Themes parks it
until a landing touches `docs/drafts/palettes.md`. The person gives the
director the draft; the director lands it as a change, which reaches no
key and lands on receipts, and the part wakes.

- **Model:** C 2.4, L 4.
- **Needs:** a park lifted by an event (J4).

## 8. Act 5: a hold and the ends (hours 7 to 9)

### 8.1 The person pauses Themes

The person drains Themes' subtree in `temper ctl`: running runs finish
their turn, a change already in a candidate lands, nothing new starts.
Later the director tries to lift it, cannot (the hold is the person's),
and proposes the lift; the person accepts it.

- **Model:** C 2.4.
- **Needs:** a drain over a subtree (J2); owned holds and proposals
  (J1).

### 8.2 The ends

Export's branch has landed and its tier run is green, with none of its
reds open; in its closing run, Export's agent brings the designs' status
files current as one last change, and the goal ends. A tier's red after
that is news to the director.

- **Model:** C 2.7, L 9.
- **Needs:** a goal ending after its verdict (L14).

## 9. The story in fab's terms

| Story | fab |
|---|---|
| the director, a chat | the director, a session; the user's page |
| Export planning and delegating batches | the cut and its early release (started by hand: a gap) |
| plans and specs in the store | plan.toml and task specs in the repository (a gap) |
| producers, the change procedure | Codex implementer runs in slots |
| a verdict task, the goal answering | the foreman, the answer call |
| Export's branch, its checks, updates | the integration trunk, checks along the way, catch-ups, the end check |
| started beside a dependency | early starts |
| a candidate, bisection, a flake at the gate | the merge queue's batch, its bisection, a collateral red |
| ready, a record | a `ready/*` ref (a gap) |
| tiers, a red as news, a verdict naming the owner | after-landing, red routing, the red foreman |
| news to the director | signals on the page, read by a session (a gap) |
| the ladder, walks, triage | the QA ladder plan, walks by a session (a gap), the triage call |
| the closing run | the retire call |

## 10. What the case asks of temper and jig

| Gap | Steps |
|---|---|
| L1 batches | 5.1, 5.2 |
| L3 candidates on temper's branches | 5.1, 5.4 |
| L4 the prepare and key job | 4.2, 5.1, 5.4 |
| L5 keys per package | 4.2, 5.1, 5.3, 5.5 |
| L6 gate classes | 4.2, 5.1 |
| L7 runners | 5.1, 5.5 |
| L8 outcomes | 5.2, 5.5, 6.1, 6.3 |
| L9 the serializer | 5.1 |
| L11 checks before ready | 4.2, 5.3, 5.4 |
| L13 tiers following main | 6.1, 6.2 |
| L14 a goal ending after its verdict | 8.2 |
| L15 findings | 6.3, 6.4 |
| L16 QA: a ladder of walks | 6.4 |
| L19 views and acts | every step, through `temper ctl` |
| L21 a goal's own branch | 4.2, 4.4 |
| L22 the goals beside | 4.1 |
| L23 a flake at the gate; a verdict on a rejection | 5.2, 5.3, 6.3 |
| L24 `goal` and `change` options | 4.1 |
| L25 a change relaying questions; what it left | 4.2, 4.3 |
| J1, J2 owned holds, drain | 8.1 |
| J4 park until an event | 7.3 |
| J7, J8 questions to the chat | 7.2 |
| J10 an address by role | 5.5, 6.2 |
| J13 started beside a dependency | 4.2 |
| J14 waking on questions by count and age | 7.1 |
| J15 application kinds of setting | 4.1 |
| J17 jobs | 4.4 |
| F1 the forge's run reads | 6.2 |

None asks for a new jig primitive: J13 to J17 are knobs on jig's tasks,
wakes and settings, or jobs, which jig designs already.

## 11. Left out, and why

- **Machines, slots and sign-ins** (the user, 11-01): workspaces as
  claimed resources (J12), provers' capacity, account custody.
- **Trains, a repair first, the reservation** (L2, L10, L18): Later in L
  14; fab dropped its train.
- **Leases** (L12): fab has none; overlaps are shown instead (4.1).
- **Moving a live goal, successors, a fresh run from notes, a project's
  holds and holds over admission** (J3, J5, J6, J9, J11): jig's, not
  used by the lab's way of running; the third edition's acts 5 and 6
  show them (052e21597).
- **Mechanics the two documents keep from revision 4 that this story
  does not reach,** shown in the third edition or by fab's own runs:
  overlaps between two goal branches (only Export has one here); a
  change past its tries; a stop of a goal's branch; a review gate on a
  change; pinned heads, side-by-side proving, a stale candidate;
  certificates, the lock, the outbox and recovery; a tier judged by its
  receipt, starvation, unfixed, a red no goal owns, a verdict of "not
  the suspects'"; triage's other decisions; questions on the forge (off
  for now).
- **Marks and QA passes** (L17): Later; the ladder's walks are the lab's
  QA.
- **Landings across repositories, GitHub** (L20, F2 to F5): one
  repository, an owned Forgejo first.
