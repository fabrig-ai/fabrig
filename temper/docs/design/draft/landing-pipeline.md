# The landing pipeline

Draft, 2026-11-01. Nothing here is adopted until the user approves it.
How a ready branch lands on main, and what follows: its verdict after
landing, and QA. Written as jig's primitives and temper's configuration
and mechanisms on them. What comes before ready is `code-pipeline.md`;
the two are independent but for ready and the news that comes back.
`reference-case.md` runs both.

The source is the landing pipeline fab runs today (the agents repository
at 1dc102c); section 6 is fab's way as configuration, section 7 where
fab departs. Machines, slots and sign-ins are left out. Unqualified,
`landing.md`, `after-landing.md`, `findings.md` and `forge.md` are
temper's `../domain/` documents; jig's are named as jig's.

## 1. In one page

- **The landing is temper's:** its procedures and records are the forge
  connector's (`landing.md`, `forge.md` 9), on jig's primitives:
  procedures for the mechanics, jobs for checks, agent tasks for
  judgement, news to the goals concerned (jig `core.md` 3).
- **Ready to landed:** a change proves its own gates before ready; the
  landing queue lands ready changes in candidates, reusing what is
  proved, rerunning a failed test alone before blaming anyone, bisecting
  a red candidate, and fast-forwarding main.
- **After landing:** tiers run on main's newest tip; a confirmed red is
  news to the goals whose changes it may be; their agents decide the
  repair. A red no open goal owns is news to a chat.
- **QA:** a ladder is a goal; its rungs are walk tasks with
  dependencies; findings are temper's records; repairs are changes.
- **An LLM for judgement, code for mechanisms:** each section says
  which.

## 2. Ready (code)

- **Ready is a change's state** in the store (`landing.md` 3): its
  branch and tip, never a ref in the repository.
- **Its own check before ready:** the change procedure proves the
  change's gates at its tip merged onto main, as a job in its workspace
  (`check_before_ready`), and the project's change-alone gates
  (`gate_set`: a counted baseline, code style). A red is a repair. Its
  results are receipts (`landing.md` 6), which the queue reuses.

## 3. Landing (code; who is at fault is LLM)

### 3.1 The landing queue

- **Candidates:** each cycle the queue takes up to `candidate_size`
  ready changes, oldest tip first, prepares them onto main in order
  (`prepare`: merge, or rebase for a candidate of one; the project's
  tooling recounts counted baselines, `landing.md` 5.3), and proves the
  candidate (`landing.md` 4 to 7).
- **What it proves** is selected by key (`landing.md` 6): the packages
  the candidate reaches, with paths that always run; a
  documentation-only candidate only `docs_gates`; every `full_every`th
  candidate in full. A candidate of one whose receipt is green for the
  same tip and main runs only `receipt_gates`.
- **Gates run as jobs** in a lane of their own, CPU measured; starved
  tests stay green; a budget the person sets is reported.
- **Green:** the turn fast-forwards main to the candidate (`landing.md`
  8); each change's branch is deleted; its goal hears it landed.
- **A fault of the gate** (short of disk, a refused push) rejects
  nobody, and the candidate goes again (`landing.md` 7.4).

### 3.2 A flake at the gate

- **A failed test is rerun alone** when at most `gate_reruns` failed and
  each passed in every member's receipt; all passing, nobody is
  rejected, and the candidate goes again (`findings.md` 5 designs the
  rerun). A tip with `gate_flakes` such flakes has its next red stand.

### 3.3 Bisecting

- **A red candidate is halved,** the left half first, each half proved
  again; a green half lands before the right is proved, until each red
  is one change's, which is rejected. After a red full candidate, the
  halves run in full.

### 3.4 A rejection

- **A conflict** is a resolution (`forge.md` 8.4); **a red** goes back
  to its change: a verdict task under the examiner's charter reads the
  failing gate and log and says fix (a repair, `forge.md` 8.4, with what
  to change) or retry (not the change's: the same tip goes back, once).
  Which follows a refusal is code's, from the gate.
- **A goal's branch** rejected is updated, checked again and readied
  again (`code-pipeline.md` 3.4).

## 4. After landing

### 4.1 Tiers follow main (code)

- **Tier jobs** run the project's slow suites on main's newest tip, one
  run at a time (`after-landing.md` 2, 3); tips landed meanwhile are the
  next run's. Live tiers run on accounts the run is granted (jig
  `accounts.md`).
- **Suspects** are the landings since each test last passed.
- **A red is confirmed** by rerunning each failed test alone: failing
  again, it is red (new, standing, or unattributed if it never ran);
  passing, a flake. A pass after a red is recovered.
- **A run's verdict:** red (a confirmed red, or over a set budget), else
  partial (a tier that cannot run, a fault), else green. A partial run's
  tip is not run again; a red run with a fault runs its tip again.
  Starved tests are counted apart, never red.

### 4.2 A red reaches its owner (code; the decision is LLM)

- **Grouped once,** per tip and suspect set, with its evidence.
- **News to each open goal among the suspects** (jig `tasks.md` 8.1,
  `after-landing.md` 7): the goal's agent, or a verdict task under the
  examiner's charter first, decides: a repair change from main
  (reproduce in the lowest tier that shows it, then fix), or not the
  code's.
- **Closed by the same checks:** when every test recovers, or is absent
  from a later run without a fault; a covering run that still has it
  makes it unfixed, news again.
- **A red no open goal owns,** an unfixed one, or a tier that cannot run
  is news to the chat its template names (`code-pipeline.md` 4).

### 4.3 The goal's verdict (code)

- **A goal ends after its verdict:** once its last landing is covered by
  a tier run, while none of its reds is being decided or repaired
  (`after-landing.md` 6 designs a change living to cleared).

## 5. QA walks and findings

### 5.1 A ladder (jig's tasks; temper's records)

- **A ladder is a goal;** its rungs are walk tasks, each a story on a
  backend, ordered by dependencies (a rung after the rungs below it).
  The rung's brief is the walk task's spec; an edition is a revision of
  that spec. A walk at the same edition is "rerun unchanged".
- **A rung's state** is its tasks': passed, walking, waiting on a lower
  rung, holding on an open finding or a person's action, or due.

### 5.2 A walk (LLM; its frame is code)

- **A walk task** runs under a tester's charter on what is deployed at
  main's tip (a job builds it), with the accounts its backend needs
  granted for the walk (jig `accounts.md`), under a time cap. Its result
  contract is a verdict (pass, fail, void) and findings: title,
  severity, area, whether it blocked, reproduction, expected, seen,
  evidence, and the open finding it met again.

### 5.3 Findings and triage (code; triage is LLM)

- **A finding is temper's record** in the store (`findings.md` designs
  it as a forge issue; here the store keeps it): new, routed, backlog,
  dropped, asked, merged, left, or in repair. Met again after its repair
  landed, it is filed again.
- **Triage is the ladder goal's agent** (or a verdict task under a
  triager's charter): at once for a blocking finding, else at a count or
  an age (`triage_at`, `triage_after`). Each finding: on the path (a
  repair, a change from main, by the owner the triager names), backlog,
  a question to the person, the brief at fault (the goal revises the
  rung's spec, a new edition), merged into another, or dropped.
- **The backlog** is findings in that state, shown in `temper ctl`; a
  project may also have it rendered into a document of its own
  (`backlog_document`), a projection never read back (jig `core.md`
  8.1).

### 5.4 The pass (code)

- **A rung passes** when a walk at its edition passes after an earlier
  walk of that edition that passed or failed, nothing holding it, its
  lower rungs passed. A failed walk reopens it.

## 6. fabrig's configuration

| Choice | fabrig's value |
|---|---|
| the queue | `candidate_size` 8, `full_every` 5, `prepare` merge, selection by packages and their reverse dependencies |
| gates | counted baselines (`<tree>/scripts/old-forms.txt`, recounted by `old_forms.py --write`), formatting, lints, the build, the `default` and `fuzzy` tests; `docs_gates` the touched counted baselines; `receipt_gates` counted baselines and formatting; `gate_set` the counted baseline |
| `check_before_ready` | on |
| `gate_reruns`, `gate_flakes` | 5, 2 |
| tiers | the browser tiers of jig, temper and skein, temper's process tier; live: skein's |
| examiners | verdict tasks, Codex at medium |
| QA | one ladder goal; backends ChatGPT with an owned Forgejo or GitHub, Claude Code with an owned Forgejo; triage at 4 findings or 30 minutes; `backlog_document` on (`temper/docs/design/debt/v0.1.0-ladder.md`) |
| fab's own repository | candidates of one by rebase, its whole suite under 120 s, 3 push attempts, the legacy kit and the person's drafts refused, no check before ready, no reruns, code-style gates (no changed signature but for an interface or migration, no function over 60 lines added, no banned name, docs on public items, no repeated block of 8 lines) |

## 7. Where fab departs (fab's gaps)

- **By hand:** walks run by a session (its build, its account loans,
  `fab qa walk`, `fab qa result`); a rung's brief edited by hand;
  signals read and cleared by a session.
- **Data in the repository:** the queue's work list as `ready/*` refs;
  rung briefs as files.
- **Code doing judgement:** a red's repair goes to the newest suspect's
  plan, whatever owner its examiner names.
- **Its own roles:** a red foreman and a finding foreman (here verdict
  tasks or the goal's agent); a triage call (here the ladder goal's
  agent); signals on a page (here news to a chat).
- **A fault:** the queue never takes a ladder's backlog branch (it takes
  ready names of two segments only).

## 8. What it needs

**jig: knobs, no new primitive.** Jobs (designed, not built), for gates
and tiers; a walk task's spec revised as an edition (spec revisions, jig
`tasks.md` 7); grants of an application's credentials to a run, as of
accounts.

**temper: its forge connector.** Candidates of several changes, with
selection, full runs on a cadence and bisecting (`landing.md` has the
candidate and the turn); the check before ready and its receipt reused;
a flake at the gate (`findings.md` 5, designed); tiers and their reds
routed as news (`after-landing.md`, designed); findings and rungs as
records (`findings.md`, designed as issues); the charters of examiners,
walkers and triage.

## 9. Future ideas

**Not to be carried out now.** Receipts by key per package with audits
by full runs; receipts reused for candidates of several; speculative
candidates (a train) or a second gate lane; budgets that turn the gate
red; bisecting after landing; a second repair of an unfixed red; QA
passes on landings marked for QA; a passed rung's story joining the live
tier; findings matched by a normalised key; landings across
repositories; a forge's native merge queue as an accelerator.
