# configuring-p2: settings through ctl

Carry out phase **P2** of `temper/docs/design/draft/configuring.execution.md`. Its entry is the scope, quoted below. It runs as a reference milestone does (`temper/docs/design/draft/reference-case.execution.md`, section 2): design and code go through this plan as one pipeline, its designers writing the design with the specs that cite it (`DESIGN.md`).

## This plan starts early, on M3's trunk

- **Its trunk** is made from `reference-m3/integration`, where M3's cut has released its design. So this plan reads M3's design and builds on M3's code as it lands there. It is no second copy of temper's kinds as settings: M3 builds those.
- **Its designers write only** the sections the entry names: jig's `domain/engine.md` 17.2, 17.6 (the part the entry gives), 17.9 and 17.10; jig's `domain/people.md` 5.2; temper's `protocol/configuration.md` 13.1; `client/tools.md` 5.3. Never M3's: 17.5 to 17.7, `configuration.md` 13.2 to 13.4, `agent.md` 4.3.1. What 17.5 needs from restore is written after M3 lands, as the entry says.
- **It lands after M3** (plan option `after = ["reference-m3"]`). Until M3's trunk lands, this trunk is never readied. M3's later commits reach it by catch-up once M3 is on main.

## The entry

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

## Old code, for reference only

The entry's Old code lives in hel8's mirrors `~/src/fabrig/attic/temper/mirror.git` (`git -C ... show <branch>:<path>`); the assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`. It is a reference, never a plan. Whether to use it, what of it and how is the designers' and implementers' choice, case by case: adapting old code may cost more than writing it fresh. Implementers cannot reach the attic, so whatever a designer takes goes into the specs or notes.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one.

## Size

One cut is at most 90 minutes. Release early, family by family. If the whole cannot be designed in one cut, release what holds together and end with a `Question:` naming the rest.

## Walk

The entry's walk, by fabrig/qa once this lands: a settings file of each kind proposed, shown, accepted and restored through `temper ctl`, and P1's document reaching a run's brief. Mark that slice `walk = true`.
