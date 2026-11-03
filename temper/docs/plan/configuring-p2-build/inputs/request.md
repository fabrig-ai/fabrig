# configuring-p2-build: build configuring P2's tasks

Build phase **P2** of `temper/docs/design/draft/configuring.execution.md` (settings through ctl) from the design plan `configuring-p2` released. That plan's cut wrote the design and released no tasks, because its code builds on M3's application settings and temper's kinds as settings, which are now on main.

## What to build from

- **The design** is in this trunk, which starts from `configuring-p2/integration`:
  - jig's `domain/engine.md` 17.2, 17.6, 17.9 and 17.10;
  - jig's `domain/people.md` 5.2;
  - temper's `protocol/configuration.md` 13 and 13.1;
  - `client/tools.md` 5.3;
  - and their status files.
- **The plan's own notes** are in `temper/docs/plan/configuring-p2/`. Its `plan.md`, `notes/contracts.md` and `notes/next-cut.md` sketch the code batch: 13 tasks, including `restore-records` and `restore-request`, with the old code quoted.
- **M3's settings,** now landed on main, are what P2's code builds on: jig's application settings (`apps-*`) and temper's kinds as settings (`kinds-*`). Never a second copy of them.

## Also write, now that M3 has landed

configuring-p2's cut left these for after M3 (its answer, 11-03):
- jig's 17.5 history, and `domain/connectors.md` 17's restore asks;
- temper's `configuration.md` 13.2's and 13.4's own words;
- `client/tools.md` 5.2's `revision show`;
- jig's `client/domain/contract.md` 3.2 and 5.1 (a change's item of a project's policy, `Restore`'s answer), as compatible detail.

## The entry's walk

fabrig/qa walks it once it lands: a settings file of each kind proposed, shown, accepted and restored through `temper ctl`, and P1's document reaching a run's brief. Mark that slice `walk = true`.

A budget or timeout is the user's (skein's `docs/foundation/testing-strategy.md`, 8.1): a role never raises one.

## Size

One cut is at most 90 minutes. The design exists, so this cut writes the specs and the short design addenda above. Release early.
