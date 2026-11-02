# configuring: execution

2026-11-02. How [configuring.md](configuring.md) gets built: the phases
that carry its order (section 9) into the design and the code, until
its acceptance passes (section 8). It designs nothing: each phase
executes a part of the draft and writes its own design as it goes. It
runs beside `reference-case.execution.md`, whose milestones it names as
the reference's M1 to M10.

## 1. In one page

- **Five phases,** P1 to P5 (section 3), set with fabrig/dogfood and the
  user on 2026-11-02 so that no two plans' designers write one section
  at once: the reference's M1 and M3 write jig's `domain/engine.md`,
  17.5 to 17.7, temper's `../protocol/configuration.md`, 13.2 to 13.4
  and the section M1 adds after 13.8, and `../domain/agent.md`, 4.3.1,
  and P2 and P3 follow M3 beside them.
- **P1 now:** what is designed and touches nothing in flight: a
  charter's documents, charters read at their task's revision, and
  `chat start --template`. The reference's M3 takes a charter's
  documents from P1 rather than building them.
- **P2 and P3 after M3:** P2 the settings through ctl, temper's kinds in
  them; P3 the configuring agent. Their designers start once M3's cut
  has released its design into M3's trunk, and build on its text.
- **P4, stage 1** (`configuring.md`, 8.1): the golden and QA's walk
  through `temper ctl`. **P5, stage 2** (8.2): v0.1.0's rung D, once
  rung C passes.
- **Each phase's state** is in section 3's table, kept current. When P5
  passes, the draft's status holds none of these entries, and this file
  goes with the draft (`DESIGN.md`, section 3).

## 2. How a phase runs

As a milestone of the reference runs (`reference-case.execution.md`,
section 2): a plan of its own, `temper/docs/plan/configuring-<phase>`,
started by the user, its entry here its request, this file and
`configuring.md` the cut's sources; design and build in one pipeline;
landed whole through the queue; its walk, where there is one, a pass of
QA through `temper ctl` that blocks nothing. Its designers, its
implementers and its walker are separate sessions. Old branches are a
reference, never a plan, as that section says; each phase names its own
under Old code, assessed in hel8's `~/src/fabrig/coord/old-repos-value/`.

## 3. The phases

| | Phase | Executes (`configuring.md`) | Needs | State |
|---|---|---|---|---|
| P1 | Documents, revisions, a chat's template | 2 (rules reach the runs), 4, 5.4 (`chat start`) | | not started |
| P2 | Settings through ctl | 3.3, 5.4 | P1; the reference's M3 | not started |
| P3 | The configuring agent | 5.3, 6 | P2 | not started |
| P4 | Stage 1 | 8.1 | P3 | not started |
| P5 | Stage 2, rung D | 8.2 | P4; rung C (the reference's M10) | not started |

### P1. Documents, revisions, a chat's template

- **Executes:** `configuring.md`, section 2's "Rules must reach the
  runs" (a charter's documents), its section 4 (a task runs under the
  revision it was made in), and its 5.4's first row.
- **Design:** none new. jig's `domain/engine.md`, 17.3 and 17.9;
  temper's `../protocol/configuration.md`, 13.7; `../domain/agent.md`,
  4.1 and 4.6; `../client/tools.md`, 5.3 (`chat start`'s `--template`).
  A designer finding a section incomplete writes its part, and no
  more.
- **Builds:** jig's charter documents, planned with the brief; temper's
  gathering of each as a brief section; each run reading its charter,
  documents included, at its task's revision; `chat start --template`,
  any template a chat's.
- **Tests:** `../domain/testing.md`, 6.4's rows for a charter's
  documents and for running work keeping its revision.
- **Old code:** jig's and temper's `configuring-revisions/documents-r1`
  (a charter's documents); temper's `configuring-revisions/fleet` and
  `configuring-revisions/story` (a charter's version forwarded to a run;
  running work keeping and moving its revision).
- **Walk:** folds into P2's.
- **Done** when it lands. As P1 starts, fabrig/dogfood notes in M3's
  entry of `reference-case.execution.md` that P1 builds a charter's
  documents, and that M3's charters naming documents wait for P1 to
  land.

### P2. Settings through ctl

- **Executes:** `configuring.md`, 3.3 (a change touching a project's
  own policy; restoring a version) and 5.4's rows but the first, which
  is P1's.
- **Design:** the rules jig's `domain/engine.md`, 17.9 and 17.10 and
  temper's `../protocol/configuration.md`, section 13 cite this draft
  for, in their own words (`configuring.status.md`'s first entry; those
  `../domain/agent.md`, 8.1 cites are P3's); a change of settings
  touching a project's own policy, decided by one person holding both
  roles (jig's `domain/engine.md`, 17.2 and 17.6, jig's
  `domain/people.md`, 5.2, temper's `../protocol/configuration.md`,
  13.1, `../client/tools.md`, 5.3); restore and temper's kinds (jig's
  `domain/engine.md`, 17.10), the status's entry closed one way or
  recorded as a departure the user decides; what that asks of 17.5,
  M3's, written after M3 lands.
- **Builds:** the items of temper's kinds in a settings file and in a
  proposal's review; `settings show landing-rule` and `forge-charter`;
  `revision restore`; a proposal's items touching a project's policy.
  On M3's temper kinds as settings, never a second copy of them.
- **Old code:** temper's `configuring-revisions/restore`,
  `configuring-revisions/set4-r2`, `configuring-revisions/replaced` and
  `configuring-revisions/settings-file`.
- **Walk:** a settings file of each kind proposed, shown, accepted and
  restored through `temper ctl`; P1's document reaching a run's brief.
- **Starts early,** an exception to section 2, fabrig/dogfood's
  (11-02): once P1 has landed and M3's cut has released its design into
  M3's trunk, P2's designers read that trunk and write only the
  sections named above, none of which M3 writes. Its trunk takes M3's
  later revisions by an update from M3's, and it lands after M3.

### P3. The configuring agent

- **Executes:** `configuring.md`, 5.3 and section 6.
- **Design:** `../domain/agent.md`, 8.1, as P2 left the settings, the
  rules it cites this draft for in its own words;
  `../protocol/documentation.md`'s reference pages `configuration`,
  `configuration-engine` and `limits`, describing what M3 and P2 built.
- **Builds:** charter 3 and template 3 in the stock; the `settings`
  family in the stock's policy and rules, offered only where a charter's
  `tools` names it; each charter leaving its brief room, checked at a
  start and at a change's entrances (`../domain/agent.status.md`,
  `../protocol/configuration.status.md`); the three pages.
- **Tests:** `../domain/testing.md`, 6.4's rows for the configuring
  agent.
- **Old code:** temper's `configuring-revisions/agent-tools`; jig's and
  temper's `configuring-revisions/charter-tools-r6`.
- **Walk:** a configuring chat started through `temper ctl` on a real
  model, a small description proposed and accepted.

### P4. Stage 1

- **Executes:** `configuring.md`, 8.1.
- **Design:** the golden, `reference-code-production.md`, section 14
  cut into obligations; its check in code and its judge; the box's
  prerequisites and limits; designed beside 6.4 in
  `../domain/testing.md` unless its designers find a better home.
- **Builds:** the golden, the check and the judge; the walk's brief for
  fabrig/qa.
- **Walk:** 8.1 itself, run again as the reference's milestones land.
- **Done** when 8.1 passes with every obligation of a setting met and
  every gap named, as far as the reference has landed.

### P5. Stage 2, rung D

- **Executes:** `configuring.md`, 8.2.
- **Needs:** rung C passed (`v0.1.0.md`, 2.1), the reference's limits
  from M10.
- **Walk:** rung D.
- **Done** when D passes and `configuring.status.md` is empty; the
  draft's close-out follows (`DESIGN.md`, section 4).
