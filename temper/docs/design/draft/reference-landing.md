# The reference workflow: landing

Draft, 2026-11-01. Nothing here is adopted. It supersedes
`landing-model.md`. How a project developing itself on temper lands a
branch, once `reference-code-production.md` has made it ready, and what
follows on main. `reference-case.md` walks both once. The boundary is not
hard: a repair goes back from here to production (section 6). This is
one way of using temper, reached mostly by configuration, and the
simplest landing that holds: most of it is `landing.md`, built, and
section 9 says what the model leaves out. Section 11 lists what is
missing.

Unqualified, `landing.md`, `after-landing.md`, `findings.md`, `forge.md`
and `worker.md` are temper's `../domain/` documents; jig's are named as
jig's.

## 1. In one page

- **What it is:** everything between a branch ready to land and its
  change being known good on main: one gate on one candidate, one turn,
  then the slow tiers and their verdicts.
- **A landing of one:** one branch, proved alone at the exact commit
  that lands, fast-forward only. No batches, no trains, no provers.
- **A goal lands once.** Its tasks landed into its own branch
  (`reference-code-production.md`, section 3), so main sees one landing
  per goal, and contention at main is between goals, not tasks.
- **By turn, as built:** prepare, prove, turn (`landing.md`). One
  departure: prepare merges main in, never rebases.
- **One mechanism at two levels.** A rule per landing branch says what a
  landing there proves: on main the full gate, on a goal's branch
  nothing (section 3).
- **LLMs judge, code runs the mechanisms.** Preparing, dispatching,
  rerunning, classifying, landing and following main are code. An agent
  repairs a real red and resolves a conflict; a designer judges a hold.
- **Each outcome has one path** (section 4): stale prepares again, red
  repairs, a conflict resolves, a fault retries, a bound holds, and only
  a hold wakes anyone.
- **After landing, tiers follow main:** one run per tier, on the newest
  tip (section 7). A red never stops other landings.
- **Measured** (section 10), and simple on purpose: each thing left out
  has what would bring it back (section 9).

## 2. What the numbers say

From a queue's record of two days, over a real project's development:

| Measured | |
|---|---|
| batches run | 130 |
| batches of one branch | 128 |
| outcomes | 121 green, 3 red, 3 errors of the tools, 2 conflicts, 1 red that was not the branch's |
| landed | 125, of which 36 code; the other 89 carried specs and answers, which on temper are task words |
| the full gate, green | median 3 minutes, 5¼ at most |
| a whole goal's branch, ready to landed | about 4 minutes |
| a full check of a 64-task branch | 4.3 minutes |

- **Thin, and read as such:** 36 code landings in two days is low
  contention, and says nothing of a busy month. What it shows is that a
  gate of minutes and one landing per goal leave room, and that batching
  earned nothing here. Section 9 says when to look again.
- **The serial gate is the bound:** about ten landings an hour on main
  if none goes stale.

## 3. A landing

```
kept ─► ready ─► preparing ─► proving ─► turn ─► landed
           ▲         │           │        │
           │     resolving    repairing   │
           │         │           │        │
           └─────────┴───────────┘        │
                     ▲                    │
                     └────── stale ───────┘
```

`landing.md`, section 3's states, with a keep before them. L is the
change that lands a goal's branch B; any other change takes the same
way. L is ready once no part of its goal is live, and its head is what
landed into B.

### 3.1 Rules by landing branch

- **Which gates, which tiers** follow a rule on a pattern of landing
  branches (`forge.md`, 8.7), the project's:

  | Landing branch | Gates | What a landing costs |
  |---|---|---|
  | `main` | the full gate, blocking | minutes |
  | `temper/*/g*`, a goal's branch | none | seconds |

- **The project lands by turn,** at both levels: `landing.md`, 13.1's
  `by`. One turn per project, held for seconds.
- **A gate here is one workflow of steps:** the repository's own
  formatting, lints, build, counted baselines and tests, dispatch-only,
  running wherever the project's runners are. What each step checks is
  the project's.

### 3.2 Prepare

- **Read afresh:** the head and the landing branch's tip (`landing.md`,
  5.2). A head that contains the tip is the candidate as it is.
- **Otherwise main is merged in,** by the forge's update in merge style
  (`forge.md`, section 10) or, where the project names one, by its
  `prepare` on a job (`landing.md`, 5.3). A branch only moves forward:
  no forced update, commits kept as they are. Main takes the head by
  fast-forward, so its history keeps the merge. This is the project's
  setting; `landing.md`, 17 leaves the default open.
- **A conflict the tooling cannot settle** is a resolution under the
  change (`landing.md`, section 3): main's tip merged into its branch,
  the files in conflict marked, an agent keeping both sides' intent.
  Its merged head contains main's tip and is the candidate as it is.

### 3.3 Prove

- **Dispatched at the candidate** (`landing.md`, 7.1), keyed by its
  tree, without the paths the gate ignores (`landing.md`, 6.1). A key
  with a passed receipt needs no run, so a tree is never gated twice,
  and a documentation landing is proved with none.
- **Green** gives a receipt and takes the turn.
- **Proofs run side by side:** several goals may prove at once. The
  first proved lands first (`landing.md`, section 9), and the rest find
  the base stale.
- **No gates, no proof:** a landing on a goal's branch has nothing to
  dispatch and goes straight to the turn.

### 3.4 The turn

- **The base is read afresh;** a tip that moved makes the proof stale
  (`landing.md`, section 8).
- **Fast-forward only,** at exactly the candidate: main's commit is the
  gated one, and it is never gated again.
- **Landed:** the result names the commit, B is deleted with L's hold
  (`forge.md`, section 11), and the goal hears it.

## 4. Outcomes

Every outcome has one path, by code, and the person hears none of them
unless a bound or a goal passes it up.

| Outcome | Told by | What follows | LLM |
|---|---|---|---|
| green | the gate's run | the turn | none |
| stale base | the turn's read | prepares again; only what changed runs | none |
| a test failing | the gate's report | its failed tests rerun once, alone; those that pass are a flake, kept and counted | none |
| red, confirmed | the rerun | a repair under the change (`forge.md`, 8.4): its brief carries the failed step and its log's end; its new head prepares again | an implementer |
| a conflict | prepare | settled by tooling, or a resolution | an implementer, if any |
| a runner's fault | how the run ended, never its log's words (`landing.md`, 7.4) | retried up to `gate_retries`, never repaired | none |
| a forge's or tool's error | the effect | settled as `forge.md`, section 6 says | none |
| past a bound | the change | held; the hold escalates to the goal, which judges: more repairs, a new part, a cancel | a designer |
| a judge that cannot decide | the goal | passes it up to its person | none |

Bounds, each a setting: repairs at the gate (the third red holds),
resolutions, updates, stale proofs. A rejection names a fault of the
branch it rejects: one record in 130 was not, which the rerun answers.

## 5. Landings with no goal's branch

The same way, into main directly:

- **A fix:** a goal of one change, from a red or a finding, with no
  designer and no branch; an implementer, the full gate, the turn.
- **A goal's last change:** its retiring run's status files and the
  draft's retirement, delegated by G once its branch has landed.
- **A person's chat** asking for a small change, as `forge.md`, 15.1
  says.

Which one a plan uses is its choice: `forge.md`, 11.1 makes the landing
branch a parameter of the change.

## 6. Back and forth with production

- **A red at the gate:** L's repair writes B directly, as a change's
  repair does, since nothing else writes B once its parts have ended.
- **A red beyond a repair:** past its bound L holds, and G wakes. It may
  add a part: it delegates one and amends L's dependencies (jig's
  `domain/tasks.md`, section 7). L is kept again, leaves its landing
  and goes on when the part ends; changes land into B as before.
- **Main moved during proof:** stale, prepare merges it in, and a
  semantic conflict is a red.
- **After landing:** a red is section 7's, and its fix is a new change,
  not a return to B.

## 7. After landing

`after-landing.md`, in its simple form.

- **Tiers follow main** (section 3 there): one run per tier at a time,
  each on the newest tip, so one run covers every landing since the
  last. Its slow profiles are what the gate does not hold.
- **A failing run names its tests** in an artifact (section 4 there). One
  that names none, or never ends, is a fault, not a red.
- **A red's verdict is mechanical:** failed tests rerun once alone;
  passes are flakes, counted; failures of the previous run are
  inherited, blamed on no one new; new ones go to the landing in the
  range, or with several, to all of them (`suspects: tell_all`). No
  bisection in the first version.
- **Where a red goes:** to a live goal of the landing, as news that
  wakes it; otherwise, filed as a finding (`findings.md`, section 4).
  The project's triager, an LLM, judges it: a new goal, usually a fix
  (section 5), the backlog or nothing.
- **Others go on:** landings keep landing; `red_pauses` stays off.
- **A goal's end waits for the verdict:** G subscribes to the verdict on
  its landing's commit, and retires on it. A change ends at landed; its
  living on to cleared (`after-landing.md`, section 6) is not used.
- **A green tip** posts `temper/verdict/<tier>`. A finding's own life,
  and QA, are `findings.md`'s; here a finding only enters as a new goal.

## 8. Configuration

| Where | Member | Value |
|---|---|---|
| `landing.md`, 13.1, kind 4 | `by` | `turn` |
| | `prepare` | the project's tooling, or none |
| | prepare's style | merge (LD-1) |
| | `gate_retries`, `tier_retries`, `stale_proofs` | 2, 2, 2 |
| | `suspects`, `red_pauses` | `tell_all`, no |
| | `dependencies_join`, `pins` | unused: one repository |
| | `findings`, `triager` | on, a coordinator's charter |
| kind 5 | `gates` | the full gate on `main`: its workflow, ignored paths, blocking |
| | `tiers` | the slow tiers on `main`; the check on goals' branches |
| landing rules, kind 1 and 2 | `main` | required: its gate; no review by default |
| | `temper/*/g*` | nothing required |
| repository | workflows | dispatch-only, so no check runs on a pushed head |

## 9. Left out

| Left out | What it was for | What brings it back |
|---|---|---|
| Batches and trains | a queue that outruns one gate at a time | ready branches waiting for an hour, day after day; a gate past 15 minutes |
| Keys per package, selection by receipts | a gate too slow to run whole | the same; today's whole gate is 3 to 5 minutes |
| Checks before ready as receipts | not repeating a change's own checks | first-attempt failures a repository's own check could catch; start with `.temper/pre-pr` (`agent.md`, 4.5) |
| Provers, certificates, gate classes | exclusive runners for measured gates | false reds from load the rerun does not answer |
| The serializer's durability beyond the turn | a process that lands past crashes | jig's outbox and `landing.md`, section 15 hold; only a found fault |
| Priorities on a repair, a pinned repair | a red main with work queued | a red main that holds other work |
| Leases: one writer of a path across goals | two goals in one file | overlaps that the keep-up cannot settle |
| Workspaces as claims, handing over, successors | long-lived area goals | the tree here is short-lived and fresh from briefs |
| Drain, a project's holds, parked until a draft | an operator's control | when a person needs them; jig's 5.9 designs them |
| Full-proof audits, key gaps | keys that can miss | keys per package |
| Bisection after landing, cleared | many landings per tier run | ranges with several landings that are red and unexplained |
| `with`, stacks, pin-only members, the reservation | repositories that pin each other | a second repository |
| The forge's merge queue | a forge that requires it | a repository on such a forge |

`landing.md` and `after-landing.md` keep their designs; this table says
what the reference does not use, and they say the rest. The `queue` way
(`forge.md`, 8 to 10) stays for projects that want it.

## 10. Measured

Generated from records, per hour and per goal:

| Metric | Read from |
|---|---|
| ready to landed, median and 90th percentile; ready and waiting | the change's states; the landing log |
| turns, proofs per landing, stale proofs | `Landing` |
| outcomes by section 4's row, and first-attempt success | `Landing`; repairs |
| false reds: rejected, then landed unchanged | `Receipt`; repairs |
| each gate step and tier against its budget | the forge's runs |
| landing to verdict; verdicts by outcome; flakes | the tier's record |
| repair rounds and their time | changes |

Each off its target is a signal with a first action, and targets are the
person's. A red that cost throughput gets its cause and a remedy where
the next agent reads it; a class seen twice gets a check in the gate,
not another rule.

## 11. What is missing

Rows `reference-code-production.md`, section 13 holds are not repeated:
CP-3 and CP-4, a landing on any branch and rules per pattern; CP-7,
tiers; CP-9, jobs; CP-14, the report.

| # | Owner | Kind | What | Where |
|---|---|---|---|---|
| LD-1 | temper | mechanism | Prepare's style, merge, as a project's setting: `landing.md`, 5.2 rebases, and its keep of earlier heads and its "not drift" fall away for merge | `landing.md`, 5.2, 13.1, 17 |
| LD-2 | temper | mechanism | A gate's rerun of its failed tests alone, whatever findings say: a gate's run must name its tests and take `temper_tests`, as a tier's does | `landing.md`, 7.3; `after-landing.md`, sections 2, 4; `findings.md`, section 5 |
| LD-3 | temper | mechanism | A landing's holds reach the member's goal: `landing.md`, 4.2 sends them to the project's escalation role and tells goals as news, and a person would hear what a designer should judge | `landing.md`, 4.2, 7.4 |
| LD-4 | temper | mechanism | A goal subscribing to a landing's verdict, with `lives to cleared` off | `after-landing.md`, sections 6, 7 |
| LD-5 | skein, temper | mechanism | A run's artifact read, for a run's failing tests | skein's `forge.md`, 3.2; `after-landing.md`, section 4 |
| LD-6 | temper | mechanism | Findings' filing and triage, at least for a red of no live goal | `findings.md`, sections 1 to 6 |
| LD-7 | temper | setting | Kinds 4 and 5 as settings; the document's `landing` member until then | `landing.md`, 13.1 |
| LD-8 | temper | mechanism | The turn on GitHub, where a project's forge is GitHub: refused today, with the ref update and tree and run reads to build | `landing.md`, section 8; skein's `forge-github.md` |
| LD-9 | temper | setting | `.temper/pre-pr` wired to the repository's own checks, for first-attempt success | `agent.md`, 4.5 |

## 12. Later

- **Batches, trains and keys per package,** by section 9's triggers.
- **A second gate at once,** when the turn outruns one.
- **Slow tiers on a goal's branch** before it lands.
- **QA after landing on every marked change.**

## 13. Open questions

- **One gate of steps or a gate per step:** a lint red re-proves less
  with a gate per step; one gate is simpler, and is what this draft
  uses.
- **A merged history on main:** a head that merged main in lands by
  fast-forward, so a first-parent walk follows branches; whether a
  project wants a merge commit on main instead.
- **`temper/*/g*` covers a branch while its goal is live:** whether a
  rule should also bound its age.
