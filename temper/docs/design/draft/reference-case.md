# The reference case, third edition

Draft, 2026-10-30, from sessions with the user. Nothing here is
adopted. One invented story that applies `landing-model.md` to a
process like the one temper's own development runs. The story holds
every mechanic of the model that such a process uses, and none that it
doesn't. It is as small as that allows. temper must be able to run it.
The names are invented; `landing-model.md` 13 maps the real process.

## 1. In one page

- **What it is:** the model at work in one project. Each step is
  something that happens, the model's sections that handle it, and what
  temper and jig lack for it.
- **The cast** (section 3):
  - the person, with the director as their chat;
  - C, a coordinator, and the area goals under it: K and A, then K2,
    K's successor;
  - F, a goal of a fix;
  - P, an area goal started before coordinators were the rule, and L,
    the coordinator it is moved under;
  - Op, the project's operator;
  - implementers, reviewers, an advisor and a tester, as delegates.

  There is one repository of four packages, two builders of
  workspaces, two provers, a tier runner, two model plans and one owned
  Forgejo.
- **Six acts, 24 steps, about 9 hours:**
  1. a feature designed and landed (section 4);
  2. under load (section 5);
  3. after landing (section 6);
  4. the person (section 7);
  5. holds (section 8);
  6. the tree changes, and the ends (section 9).
- **What it asks** (section 11): every gap in `landing-model.md` 15.4,
  except those the case leaves out (section 12).

## 2. How to read it

- **Each step** says what happens, then gives two lists:
  - **Model:** the sections of `landing-model.md` that handle it;
  - **Needs:** what temper and jig must have, by the gap ids of that
    model's 15.4, whose table grades them; *exact* where they have it
    today.
- **Time** is in hours from the start. On fakes, timers and bounds are
  scaled down.
- **Run two ways:**
  - **On fakes,** with each judgement scripted as the oracle. The
    stories take the step's number: `fuzzy_case3_X_Y`.
  - **On real backends,** as v0.1.0's top rung. The person's words are
    a script that a QA driver types through `temper ctl`, and each step
    is checked by its effects.
- **The person acts directly or through the director.** Where both
  are possible, the step says which.
- **The case is written to the model's recommendations** for its
  choices of primitives (`landing-model.md` 17, H1 to H9); section 13
  says which steps change if one flips.

## 3. The cast and the start

- **The person** directs the work, sets budgets and priorities, and
  answers what is theirs to answer.
- **The director** is the person's chat, on Claude:
  - it coordinates under the coordinators' charter;
  - it is the QA chooser, and takes findings that have no coordinator;
  - a timer wakes it every 2 hours for its check.
- **C** is a coordinator goal on Claude: the coordinator of "Export",
  which spans the packages kit and app. It designs, has each part
  reviewed once, and starts the area goals.
- **K** (Export's kit side) and **A** (its app side) are area goals on
  Claude under C. **K2** takes on what K leaves, in act 6.
- **F** is a goal of a fix, on Claude, which C starts from a finding.
- **P** owns "Themes" in the package ui, on Claude. The director
  started it before the case, when features had no coordinator. **L**
  is Themes' coordinator, born in act 6.
- **Op** is the project's operator, a goal on GPT under the director:
  it hears faults and key gaps, and it alone sets a priority.
- **The delegates:**
  - implementers, reviewers and the tester are agents on GPT;
  - an advisor is a one-shot agent on Claude, called once two tries
    have missed.
- **The repository,** works, the project's only one:
  - **packages:** kit; ui and cli, which read kit; app, which reads kit
    and ui;
  - `tools/`, the project's own tools with their suite;
  - `debts.txt`, a counted baseline that may only go down;
  - `docs/`, which no gate reads; its `STATUS.md` is generated from one
    file per part.
- **The gates:**
  - deterministic, per package: `fmt`, `lint`, `build`;
  - deterministic, of the repository: `count` (over `debts.txt`) and
    `tools` (over `tools/`);
  - measured, per package: `test`.
- **The tier:** `browser`, on app, measured, reporting its failing
  tests by id.
- **The tooling:** `prepare` merges each member onto the parent in
  order, recounts `debts.txt` and regenerates `STATUS.md` as the
  candidate's own commits, and computes each gate's keys from the
  build's graph.
- **The machines:** two builders with eight workspaces, shared as C 4,
  the director 2, the free pool 2; two provers, each warm with its own
  cache; a tier runner. All are self-hosted runners with labels of
  their own.
- **The forge:** an owned Forgejo with Actions. Every workflow is
  dispatch-only, and "autodetect manual merge" is on.
- **The settings** (`landing-model.md`, 14):

  | Setting | Value |
  |---|---|
  | before ready | `keys` |
  | keys | per package, by the tooling |
  | batch | by measure, at most 8 |
  | provers; train depth | 2; 2 |
  | full proofs | every 5 attempts, hourly, idle after 10 minutes |
  | gate retries | 2 |
  | priority | the operator, with a hold after it at its choice |
  | `browser` | coalesce beyond 2 waiting; bisect within 45 minutes and 16 runs |
  | a tier's red goes to | the suspects' coordinators |
  | cleared | off |
  | findings on the forge | off |
  | QA | marks from a starting main, 5 a pass, a pass's cap 120 minutes, a walk's 180 |
  | the operator | Op |
  | leases | on |
  | handover | at 75% of a charter's context |

- **At hour 0:**
  - P is running on Themes, with the lease on ui;
  - nothing is held, and nothing is ready;
  - no finding is open;
  - `browser` is green at main.

## 4. Act 1: a feature designed and landed (hours 0 to 2)

### 4.1 The director starts C, and C's design lands

The person: "Export's draft is in; it's next." The director starts C at
priority 1, with the draft at its commit as its brief.

C takes the lease on `docs/design/export.md` and claims a workspace
from its share. It writes the design of the boundary between kit and
app, and a GPT reviewer reads it once. C amends the design, and its
change into `docs/design/` is ready.

The change touches only paths no gate reads. So:
- its keys equal main's, whose receipts pass, and no run is dispatched;
- the serializer lands it in a candidate of one, fast-forwarding main;
- `browser`'s key is unchanged too, so the tier judges the new main by
  its receipt, with no run.

C hears its landing within a minute.

- **Model:** 3.1, 3.2, 3.3, 5, 6.1, 9, 10.
- **Needs:**
  - a coordinator goal with a one-shot reviewer (exact);
  - a lease on a document (L12);
  - a workspace claimed from a share (J12);
  - receipts for paths no gate reads (L5);
  - the serializer and its turn (L9);
  - a tier judged by its receipt (L13).

### 4.2 C starts its area goals

C starts K and A, each with a brief: the design at its commit, and the
boundary where they meet. Its charter's standing rule says the
interface lands first, alone. K takes the lease on kit for the wave,
and A the lease on app.

K delegates three implementers on GPT, each in a workspace of C's
share: k1, the interface; k2, which depends on k1; and k3. k2 finds
that its sketch doesn't match main and asks K, its requester. K
answers, and k2 goes on.

- **Model:** 3.1, 3.2, 3.3, 3.4.
- **Needs:**
  - goals at depth three, dependencies between delegates, and a
    question to the requester (exact);
  - leases on packages (L12);
  - workspaces claimed for a change's life (J12).

### 4.3 k1, checked before ready

k1 changes what a kit item means. K runs the check before ready: in
k1's workspace, its head is merged onto main's tip, uncommitted, and
the deterministic gates run over kit and every package that reads it.
`tools` and `count` are not reached.

`lint` fails in cli. k1's implementer fixes it in the same workspace,
and the check passes. Each key's result is a receipt, from a
certificate temper accepted.

Since k1 changes a meaning, the readers' tests (`test` in ui, cli and
app) run there too. They pass, and are reported as advisory, with no
receipt. The checkout is restored and verified, K marks k1 ready, and
the workspace's claim is released.

- **Model:** 4.1, 5, 7.
- **Needs:**
  - the check before ready, in the workspace (L11);
  - keys per package (L5), computed by the tooling (L4);
  - gate classes: deterministic receipts from a certified runner (L6,
    L7);
  - the workspace restored before its claim moves on (J12).

### 4.4 The first batch lands

At hour 1, k1 is ready, and so are P's t1 and t2, in ui. Prover 1 is
busy with the idle audit of main after 4.1's landing, so the three wait
and join one candidate, k1 first:
- the prepare job, on a worker, merges them in order onto main, on
  `temper/works/c1`, and reports each gate's keys;
- kit's and cli's deterministic keys are those k1 was checked at, so
  their receipts hold;
- ui's and app's changed with t1 and t2, so their deterministic gates
  run, with every reached package's `test`, in one dispatch to prover
  1, with one build.

It is green. Main still equals the candidate's parent, so the turn
fast-forwards it to the candidate. The forge marks the three pull
requests merged, and each branch is released by a deletion that names
the head it landed. K and P hear their landings, with the commit.

k1's commit says `Taught: export ids must be stable across runs`. C's
wake policy names `Taught:`, so the landing wakes C, which amends its
design. That amendment lands as in 4.1.

- **Model:** 4.2, 5, 6.1, 6.2, 7, 8, 9, 11.2, 12.
- **Needs:**
  - a batch (L1);
  - a candidate on temper's branch by a merge chain (L3);
  - the prepare and key job (L4), and keys per package (L5);
  - a prover (L7), and the serializer's turn (L9);
  - landing news with trailers (L17);
  - Forgejo's dispatch, statuses, a ref updated fast-forward only, and
    pull requests marked merged by containment (F1, F4).

## 5. Act 2: under load (hours 2 to 4)

### 5.1 A train

By hour 2, a dozen implementers are producing, and ready landings wait
longer than a proof. Candidate 2 (k2, k3, a1, t3) proves on prover 1.
Candidate 3 (a2, t4, k4) is prepared on candidate 2's commit, and
proves on prover 2 at once.

Candidate 3's proof ends first. It waits, with no effect. Candidate 2
is green and lands; main then equals candidate 3's parent, and it lands
too.

- **Model:** 6.3, 7, 9.
- **Needs:**
  - a train (L2);
  - provers with affinity, idempotent dispatch, and certificates (L7).

### 5.2 A red in a big batch

Candidate 4 holds six landings, and candidate 5 is prepared on it.
Candidate 4 is red: an app `test` fails.

- **Candidate 5 is invalidated.** Its result arrives later and decides
  nothing, but its certified receipts stay good for their keys. Prover
  2 is free once its commands have ended.
- **Candidate 4 is bisected,** with nothing prepared on it meanwhile:
  1. its left half, three landings, proves against main, green, and
     lands;
  2. its right half is red, and splits: a3 alone is red, and the other
     two are green and land.
- **a3's red is its own:** a repair under A, briefed with the failing
  step and the log.
- **Candidate 5's landings** are prepared again on the new main. kit's
  and cli's keys are those candidate 5 proved, so only app's and ui's
  run.

- **Model:** 5, 6.2, 6.3, 8.
- **Needs:**
  - bisection by halves (L1);
  - successors invalidated, and no speculation in a bisection (L2);
  - receipts kept from a discarded candidate (L5);
  - a red as the member's repair (L8).

### 5.3 Fresh by key

A's a4 was checked before ready at hour 2. Since then k2 landed, and
app reads kit, so a4's app keys changed. At admission, temper claims a
workspace from the pool and checks a4 again: only app's deterministic
gates run, and they pass. a4 joins the next candidate.

Op's o1, a change to `tools/`, was checked before k2 too. Its only key
is `tools`, which k2 didn't change, so its receipt stands, and o1 joins
as it is.

- **Model:** 4.2, 5.
- **Needs:**
  - `before_ready` and the check at admission (L11);
  - freshness as key equality (L5);
  - a workspace claimed for a check (J12).

### 5.4 Conflicts, by rule and by resolution

a5 and t5 each lower `debts.txt`, and each adds a part to `STATUS.md`.
When the batch holding both is prepared, the tooling recounts
`debts.txt`, which refuses a raise, and regenerates `STATUS.md`, as the
candidate's own commits. No conflict reaches anyone.

A's a6 needs a hook in ui's menu. ui is P's for the wave, so P lends
ui to a6, and a6 is ready. But P's t6, ahead of a6 in the same batch,
changed the same function:
- prepare leaves a6 out, and the rest of the batch goes on;
- the conflict is staged with the attempt, and is a6's once the
  attempt's parent is green;
- a resolution under A's goal, on GPT, settles it on a6's branch;
- a6 is checked again before ready, and joins a later candidate.

- **Model:** 3.3, 6.1, 6.2, 8.
- **Needs:**
  - a lease lent to a change (L12);
  - the prepare's own commits, by the project's rules (L3, L4);
  - a conflict staged with its attempt, and its member left out (L3);
  - a resolution under the member's goal (exact).

### 5.5 A fault, and a repair first

At hour 3, prover 2's disk is below its floor. It refuses candidate 9
before running a gate: a fault, retried on prover 1, with nobody
rejected. Op is told once, and trims prover 2's cache.

At hour 3.5, a test fixture's certificate in app expires. The idle
audit proves main in full, and app's `test` is red at main itself. Op
hears it, since no member is to blame:
1. Op holds the landing pipeline's admission;
2. candidate 10, in flight, fails in the same test. The red is main's,
   so nobody is rejected, and its landings return to the ready set;
3. Op's implementer renews the fixture, in o2, and Op gives o2 the
   priority `repair`;
4. o2 passes Op's own hold and proves alone, in full, whatever its
   receipts say. It is green and lands;
5. the next full proof is green, and Op lifts the hold.

- **Model:** 3.5, 5, 6.4, 7, 8, 9.
- **Needs:**
  - a fault before a gate (L7), kept apart from reds (L8);
  - an idle audit of main, and a red main already has (L5, L8);
  - a priority that passes its setter's hold (L10);
  - a hold over the landing pipeline's admission (J9), owned (J1).

### 5.6 A full proof finds a key gap

Candidate 12 is the fifth attempt since the last full proof, so it is
proved in full. cli's `test` is red. A cli test reads kit's fixtures by
path, and candidate 12's k6 changed one. The selections before passed,
since cli's key didn't hold kit's fixtures.

The red is bisected with every half proved in full, to k6: a repair
under K. Op is also told of a suspected key gap, and adds kit's
fixtures to cli's `test` key, a change to the tooling.

- **Model:** 5, 8.
- **Needs:**
  - full proofs that audit the keys, and what a key cannot see (L5);
  - a key gap (L8).

## 6. Act 3: after landing (hours 4 to 5.5)

### 6.1 Tiers follow main

`browser` is running on an older main when candidates 13 and 14 land.
When that run ends, three tips are waiting, more than the bound, so the
older two are coalesced into the newest, and the coverage lost is
recorded. The run is lost when its runner restarts. That is a fault,
not a red: it is retried, and the retry is green.

- **Model:** 10.
- **Needs:**
  - tiers following main, one run at a time on the newest tip, and
    coalescing (L13);
  - a tier's fault kept apart from a red (L8).

### 6.2 A red, bisected, to the coordinator

One `browser` run covers four landings, in order: A's a7, P's t8, C's
second design landing, and K's k7. Two tests fail, and they fail again
when rerun alone. temper bisects, within its bound:
- the design landing reaches no key of the tier's, so it is skipped;
- the two tests fail at a7's commit, so they are a7's.

The red goes to C, the nearest coordinator above a7's change, with the
tests, the log's end and the bisection's answer. C has A repair it:
1. an implementer tries twice and misses;
2. A calls a Claude advisor once, and it finds the cause;
3. the repair lands.

Meanwhile P lands t9. The run after it shows the same two failures:
they are standing, not new, so nobody hears of them. The run after the
repair passes both, and the recovery is recorded.

- **Model:** 3.7, 10.
- **Needs:**
  - the rerun, bisection within a bound, and standing reds (L13);
  - the red routed to a role, the nearest coordinator (J10);
  - test ids from the run's artifacts (F1);
  - a one-shot advisor on another model (exact).

### 6.3 A flake, and its finding

A `browser` run fails one test, and the rerun passes: a flake. Finding
#7 is recorded in temper's store: a flake, in app, with the test's id
and the failure's first line as its signature. app is C's area, so #7
is routed to C.

C triages #7: it starts F, a goal naming #7.

Before F lands, candidate 16's app `test` fails in the same test. The
failure matches #7, so the failed job is rerun once, and passes. No
repair is made.

F's fix lands, naming #7, and #7 closes.

- **Model:** 8, 11.1.
- **Needs:**
  - findings as temper's records, matched by signature, routed,
    triaged and closed (L15);
  - a gate's failed jobs rerun (exact);
  - a goal made from a finding (exact).

### 6.4 QA

k1, a1 and t2 carry `QA:` trailers. Marks are chosen from a starting
main, so all three are. At its 2-hour check, the director, the QA
chooser, moves t2's mark to its backlog, since Themes comes after
Export.

Once the newest main whose tiers ran green holds both marks left, a
pass starts on it. Its walk, under the tester's charter, on GPT:
- claims a workspace and deploys the build at that commit;
- holds a sign-in in custody for its length, and returns it at its end;
- exercises the marked stories as a user would, with no forge effects;
- writes down what it expects before each case.

It records two findings, each for the coordinator of the landing that
carried its mark, C:
- #9: Export's CSV header is wrong, on the happy path;
- #10: a page is slow, which is minor.

At its next wake, C routes #9 to A, which repairs it, and moves #10 to
its backlog.

- **Model:** 3.2, 11.1, 11.2.
- **Needs:**
  - marks, a pass, and a walk with a sign-in in custody (L16);
  - trailers in landing news (L17);
  - triage to a goal and to the backlog (L15).

## 7. Act 4: the person (hours 5.5 to 6.5)

### 7.1 A question to the person

A's app suite takes 70 s against its budget of 60 s. Budgets are the
person's, so A asks the person directly. Person answers are on, so the
person's answer, "don't raise it; make it cheaper", goes straight to A,
and the director hears a notice. A has an implementer make the suite
cheaper.

- **Model:** 3.4.
- **Needs:** any task asking the person, and the answer reaching the
  asker (exact).

### 7.2 Questions to the chat stay with the chat

C asks the director, its requester: "Export's CSV first, or JSON?" The
question is the director's to answer, not the person's. The director
answers from the priorities in its notes.

Later C asks: "may kit's API break ui's old callers?" The director
doesn't know, so it asks its own person, in the inbox. The person
answers. The director answers C in its own words.

- **Model:** 3.4.
- **Needs:**
  - a question to the chat staying with it (J7);
  - the chat asking its person (J8).

### 7.3 Parked until the person's draft

K's next part needs the person's draft on Export's formats, which
doesn't exist yet. C parks K until a landing touches
`docs/drafts/formats.md`; K keeps its lease on kit. The person lands
the draft, which reaches no key and is landed on receipts, and K wakes.

- **Model:** 3.5, 5.
- **Needs:** a park lifted by an event (J4).

## 8. Act 5: holds (hours 6.5 to 7.5)

### 8.1 The person pauses Themes

The person: "pause Themes." The person drains P's subtree, as the
hold's owner:
- P's implementer finishes its turn;
- t10, in a candidate already, lands;
- nothing new starts, and P is held as it goes idle, keeping its lease
  and its workspaces.

Later the person says "use every free workspace". The director reads
that as a resume and tries to lift P's hold, but it cannot, because the
hold is the person's. So it proposes the lift. The person sees the
proposal in their inbox and says yes, and P resumes.

- **Model:** 3.5.
- **Needs:**
  - a drain over a subtree (J2);
  - owned holds, and a proposal to lift one (J1).

### 8.2 A drain, to change the rules

The person wants to change the coordinators' rules while nothing is
half done, so they drain the project:
- running runs finish their turn;
- the candidates in flight are decided, and A's a9, in one of them,
  lands;
- nothing new starts, and no candidate is admitted;
- each goal is held as it goes idle;
- the director stays live.

The person lands the rule change: a new revision of the coordinators'
charter, which live coordinators move to. Then the person lifts the
drain and releases the goals one at a time: C first, then K and A when
C proposes them.

- **Model:** 3.1, 3.5.
- **Needs:**
  - drain (J2), over a project (J3);
  - a charter's new revision reaching live tasks (exact).

### 8.3 A stop over the project

The director relays a plan that the person hasn't approved yet. It
rewrites its priorities, starts a goal for the plan, and that goal
lands one change.

The person: "act on none." This stops the project: runs are cancelled
at once, nothing lands, and new tasks start held. The person cancels
the plan's goal, and asks the director for a revert of the landed
change. The revert's change starts held, so the person releases that
one task, and it lands.

Ninety minutes later the person approves the plan, and lifts the stop
with every hold it placed. The director starts the plan's goal again.

- **Model:** 3.5.
- **Needs:**
  - a stop over the project (J3), owned by the person (J1);
  - one task released under a project hold (J3).

## 9. Act 6: the tree changes, and the ends (hours 7.5 to 9)

### 9.1 A coordinator born, and a live goal moved

The rule changed in 8.2 says every feature gets a coordinator. The
director starts L, Themes' coordinator, and moves P, still running,
under L:
- P's next result goes to L, and L funds its budget's reservation;
- its workspaces stay claimed by P, now within L's share;
- news for the nearest coordinator above P's changes now reaches L;
- the director keeps a reference to it.

- **Model:** 3.1, 3.2, 3.6, 3.7.
- **Needs:**
  - moving a live task under another task (J5);
  - a role's address resolved when news is made (J10).

### 9.2 C hands over

C's context nears its charter's bound. Its run ends with a handover
note, and its next run starts from its brief, its notes and the
handover, not from its transcript. Its lease on the design, its
subscriptions and the answers it waits for stay with it, and no second
session warms up beside it. A red that arrives meanwhile waits in C's
inbox, and wakes the new run.

- **Model:** 3.6.
- **Needs:** a fresh run from a goal's notes and handover (J11).

### 9.3 An area goal ends, and its successor

K's last landing lands, and K ends. Its result says what it did and
what is left: one part, and a question to the person about a format's
default, open for 2 hours.

C starts K2 with K's result as its brief, and names K2 as K's
successor. K's lease on kit passes to K2 with no gap for another
writer. An hour later the person answers K's question, and the answer
goes to K2.

- **Model:** 3.1, 3.3, 3.4, 3.6.
- **Needs:** a successor that receives an ended task's answers and
  leases (J6).

### 9.4 The ends

A and K2 each end once their last landing lands; a tier's red after
that goes to C, as in 6.2. C ends once its goals have ended, its
findings are closed or in its backlog, and its design carries what
their `Taught:` lines said; its leases are released, and their first
waiters told. The director compares their results, and proposes a
change to the rules, which the person accepts.

- **Model:** 3.1, 3.3, 10, 11.1.
- **Needs:**
  - goals that end at landed, with later reds going to their
    coordinator (L13);
  - results and proposals (exact).

## 10. The index

Every mechanic of `landing-model.md` that the case uses, with the steps
that hold it.

| Mechanic | Model | Steps |
|---|---|---|
| A chat as coordinator; coordinator goals; goals at depth three | 3.1 | 4.1, 4.2, 9.4 |
| Delegates by charter and model: implementers, reviewers, an advisor, a tester | 3.1 | 4.1, 4.2, 6.2, 6.4 |
| Periodic checks | 3.1 | 6.4 |
| A standing rule; a charter's new revision | 3.1 | 4.2, 8.2, 9.1 |
| Workspaces: shares, claims, the pool, restored after a check | 3.2 | 4.1, 4.2, 4.3, 5.3, 6.4 |
| Leases: a wave's writer, a lend, waiters, passed to a successor | 3.3 | 4.1, 4.2, 5.4, 9.3, 9.4 |
| Any task asking the person; answers reaching the asker | 3.4 | 7.1 |
| A question to the chat stays with it; the chat asks its person | 3.4 | 7.2 |
| An answer after its asker ended | 3.4, 3.6 | 9.3 |
| Owned holds and proposals to lift them | 3.5 | 5.5, 8.1, 8.3 |
| Stop and drain, over a subtree and over a project | 3.5 | 8.1, 8.2, 8.3 |
| A hold over the landing pipeline's admission | 3.5 | 5.5 |
| Park until an event | 3.5 | 7.3 |
| A fresh run from notes | 3.6 | 9.2 |
| A live task moved under another; a successor | 3.6 | 9.1, 9.3 |
| News to roles: the member's goal, the nearest coordinator, the operator | 3.7 | 4.4, 5.5, 6.2, 9.1 |
| Checks before ready, in the change's workspace; readers' tests | 4.1 | 4.3, 5.4 |
| Fresh by key; a check at admission | 4.2 | 5.3 |
| Keys per package; repository gates; paths no gate reads | 5 | 4.1, 4.4, 5.3, 7.3 |
| Receipts outliving candidates | 5 | 5.2 |
| Gate classes | 5 | 4.3 |
| Full proofs, an idle audit, what a key cannot see | 5 | 5.5, 5.6 |
| A candidate on temper's branch; the prepare's own commits; pinned heads | 6.1 | 4.4, 5.4 |
| A conflict staged with its attempt; a member left out | 6.1, 6.2 | 5.4 |
| Batches; bisection by halves | 6.2 | 4.4, 5.2 |
| A train; invalidation; no speculation in a bisection | 6.3 | 5.1, 5.2 |
| A repair first, alone and in full | 6.4 | 5.5 |
| Provers: exclusive, warm, certified; faults before a gate | 7 | 4.4, 5.1, 5.5 |
| Outcomes: green, red, conflict, fault, key gap | 8 | 5.2, 5.4, 5.5, 5.6 |
| The serializer: the turn, the outbox, expected-head release | 9 | 4.1, 4.4, 5.1 |
| Tiers on the newest main, coalescing, the rerun, bisection, standing reds | 10 | 6.1, 6.2 |
| A tier's red to the suspects' coordinator; goals ending at landed | 10 | 6.2, 9.4 |
| Findings: recorded, matched, routed, triaged, closed | 11.1 | 6.3, 6.4 |
| QA: marks, a pass, a walk with a sign-in in custody | 11.2 | 6.4 |
| Trailers in landing news | 11.2 | 4.4, 6.4, 9.4 |

## 11. What the case asks of temper

Every gap of `landing-model.md` 15.4 but those of section 12, with its
steps:

| Gap | Steps |
|---|---|
| L1 batches | 4.4, 5.2 |
| L2 trains | 5.1, 5.2 |
| L3 candidates on temper's branches | 4.4, 5.4 |
| L4 the prepare and key job | 4.3, 4.4, 5.4 |
| L5 keys per package | 4.1, 4.3, 4.4, 5.2, 5.3, 5.5, 5.6 |
| L6 gate classes | 4.3 |
| L7 provers and runners | 4.3, 4.4, 5.1, 5.5 |
| L8 outcomes | 5.2, 5.5, 5.6, 6.1 |
| L9 the serializer | 4.1, 4.4, 5.1 |
| L10 a priority | 5.5 |
| L11 checks before ready | 4.3, 5.3 |
| L12 leases | 4.1, 4.2, 5.4, 9.3 |
| L13 tiers following main | 4.1, 6.1, 6.2, 9.4 |
| L15 findings | 6.3, 6.4 |
| L16 QA | 6.4 |
| L17 trailers in news | 4.4, 6.4 |
| L19 views and acts | every step, through `temper ctl` |
| L20 one change in one repository | every landing |
| J1 owned holds | 5.5, 8.1, 8.3 |
| J2 drain | 8.1, 8.2 |
| J3 project holds | 8.2, 8.3 |
| J4 park until an event | 7.3 |
| J5 move under a task | 9.1 |
| J6 successor | 9.3 |
| J7 questions to the chat stay | 7.2 |
| J8 the chat asks its person | 7.2 |
| J9 a hold over admission | 5.5 |
| J10 an address by role | 6.2, 9.1 |
| J11 a fresh run from notes | 9.2 |
| J12 workspaces | 4.1, 4.2, 4.3, 5.3 |
| F1, F4 the forge's calls and adoption | 4.4, 6.2 |

F2, F3 and F5, GitHub's, come when the case runs on GitHub. The first
pair of backends is an owned Forgejo.

## 12. Left out, and why

- **Landings across repositories** (`landing-model.md`, H9): the
  process has one repository. A change across repositories lands in
  steps, each parked until the one before lands, which is 7.3's
  mechanic.
- **Cleared** (L14): the process ends goals at landed, and a tier's
  red goes to the coordinator.
- **The reservation** (L18): the process batches, and main moves only
  by the serializer's turn, so no proof goes stale at the turn; it
  serves landings proving side by side.
- **Findings and questions on the forge:** settings the process keeps
  off, and Later's.
- **"Tell all", a tier's red to the landing's goal too, and a red
  holding admission**: settings the process doesn't choose.
- **Judgement gates on a landing:** reviews here are delegates inside
  goals, not gates.
- **A review's follow-up as a finding:** the process records findings
  from QA and from tiers only.
- **A walk on demand, before landing:** 6.4's walk is a pass's; one a
  coordinator starts on a build at a named commit is the same walk.
- **Deeper trains, elastic provers, partitioned full proofs:** Later's.
- **The change of shape** that the first edition's act 2 tells: here a
  rule changes (8.2, 9.1), but the shape of the tree does not.

## 13. Open questions

- **Names:** "coordinator", "area goal" and "operator" are the model's
  roles; temper's charters may name them otherwise.
- **The oracle on fakes:** which judgements are scripted word for word
  (C's triage, the chooser's picks, Op's priority), and which are
  checked by their effects.
- **Load on real backends:** act 2 needs about a dozen producers at
  once to make batches and a train. Whether they run on real models, or
  as scripted producers on the real forge and real provers.
- **Real time:** whether 9 hours fits one sitting on one held login,
  or whether the acts run apart, each from the state the act before it
  leaves.
- **The choices of primitives** (`landing-model.md`, 17): a flip
  changes these steps: H1, 4.3 and 5.3; H2, 4.4 and 5.6; H3, 4.4; H4,
  6.2 and 9.1; H5, 5.5; H6, 4.2 and 5.4; H7, 4.1 to 4.3; H8, 9.2; H9,
  section 12's first item.
