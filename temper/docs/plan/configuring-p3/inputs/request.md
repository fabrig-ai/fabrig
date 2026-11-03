# configuring-p3: the configuring agent

Carry out phase **P3** of `temper/docs/design/draft/configuring.execution.md`. Its entry is the scope, quoted below. It runs as a reference milestone does (`temper/docs/design/draft/reference-case.execution.md`, section 2): design and code go through this plan as one pipeline, its designers writing the design with the specs that cite it (`DESIGN.md`).

P1 (documents, revisions, a chat's template) and P2 (settings through ctl) are on main, as are every reference milestone, M1 to M10. So this plan builds on main.

## The entry

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

## Context

- **Old code** in the named branches is a reference only (`reference-case.execution.md`, 2). Whether and how to use it is the designers' and implementers' call, case by case.
- **The walk** runs on a real model, through `temper ctl`. Mark that slice `walk = true`.
- **Budgets and timeouts are the user's** (skein's `docs/foundation/testing-strategy.md`, 8.1).
- **Beside it:** `public-reading-build` and `playground-browser` are building; leave their files alone.

## Size

One cut is at most 90 minutes. Release early.
