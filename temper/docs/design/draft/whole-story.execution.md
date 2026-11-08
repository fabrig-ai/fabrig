# whole-story: execution

2026-11-08. How [whole-story.md](whole-story.md) and the four drafts
written with it, `standing-roles.md`, `what-reaches-you.md`, `follows.md`
and `surfaces.md`, reach the design and the code by v0.1.0, in tracks
that run side by side. It designs nothing: each milestone writes its own
design from its draft as it runs (`DESIGN.md`, section 3). A milestone
that meets a question its draft leaves open takes the draft's default,
or the nearest thing, and says so in that draft's status.

## 1. In one page

- **M0 first, small and alone:** the mockups join the design
  (`whole-story.md`, section 9).
- **Then five tracks at once,** each one area's: P, the pages
  (temper/ui); N, what reaches a person (temper/ui, with temper/migration
  for rules by path); R, standing roles (temper/migration); F, follows
  (temper/migration); T, the terminal (temper/tui).
- **Few waits between tracks** (section 4): wave 1 needs nothing but
  M0; the rest wait only for what they draw or call: Team's page for
  areas, talking with a role for roles and the launchpad, How work runs
  for the classes, and rules by path and follows for their parts.
- **Two stories close it:** E on a box through `temper ctl`, and V, the
  vsr pilot on a real deployment (`whole-story.md`, section 11).
- **Speed comes from width:** each track runs its milestones one plan
  at a time, the tracks side by side, with the shared seams in section
  4 told before they change.

## 2. How a milestone runs

- **Its entry here is its input,** its draft the source, the mockups
  its look.
- **Its design and its build together,** landed whole: the design
  documents' sections its draft's status lists as lags, then the code
  and its focused and fuzzy tests.
- **A page or a frame is recorded** as a journey for the user to take
  (`../client/journeys.md`; `../client/tui/journeys.md`).
- **Its walk** follows its landing and blocks nothing.
- **Its state** is section 3's table, kept current. When every
  milestone is done and the user has taken the pages and frames, the
  drafts' statuses hold no lag of what v0.1.0 needs, and this file goes
  (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes | Needs | Owner | State |
|---|---|---|---|---|---|
| M0 | The mockups join the design | `whole-story.md`, 9; `surfaces.md`, 13 | | temper/draft/ux | done: landed by hand in `whole-story/mockups` |
| P1 | The rail and Work | `surfaces.md`, 2, 5, 8 | M0 | temper/ui | not started |
| P2 | Home and Needs you across projects, and the brief | `surfaces.md`, 3, 4 | M0 | temper/ui | not started |
| P3 | How work runs | `surfaces.md`, 7 | M0, N1; N2 and F1 for their parts | temper/ui | not started |
| P4 | Team | `surfaces.md`, 6 | R2; R3 for "Talk to it" | temper/ui | not started |
| P5 | The phone, Ask and Tell temper | `surfaces.md`, 9, 10 | P2; R2 for Tell's routing | temper/ui | not started |
| P6 | The drafter's charter | `surfaces.md`, 11.2, its instructions | | temper/ui | not started |
| N1 | Three classes, Away, why and Stop asking, Decided for any task | `what-reaches-you.md`, 2 to 6 | R1 for a standing role's case | temper/ui | not started |
| N2 | Rules by path | `what-reaches-you.md`, 7 | N1 | temper/migration | not started |
| N3 | Only you, and being told | `what-reaches-you.md`, 8, 9 | N1 | temper/ui | not started |
| R1 | A standing role and its sessions | `standing-roles.md`, 2, 3, 5, 9 | | temper/migration | not started |
| R2 | Areas, routing, outside parties, stock roles | `standing-roles.md`, 4, 8, 10 | R1 | temper/migration | not started |
| R3 | Talking with a role, and its reports | `standing-roles.md`, 6, 7 | R1 | temper/migration | not started |
| F1 | A follow, across projects | `follows.md`, 2 to 6 | R1 for a standing role as its owner; a person owns it until then | temper/migration | not started |
| F2 | Depends on | `follows.md`, 7 | F1 | temper/migration | not started |
| T1 | The launchpad as drawn, and views printed | `surfaces.md`, 11.1, 11.4, but its team lines, `/talk` and `/tell`; 11.5 | M0 | temper/tui | not started |
| T2 | A design session | `surfaces.md`, 11.2 | T1, P6 | temper/tui | not started |
| T3 | Talking with a role, and the launchpad's team | `surfaces.md`, 11.1's team lines, 11.3, 11.4's `/talk` and `/tell` | T1, R2, R3 | temper/tui | not started |
| E | The whole story on a box | `whole-story.md`, 11 | every milestone above | each owner writes its part of the story | not started |
| V | The vsr pilot | `whole-story.md`, 5.2 and 11 | N1, N2, R1, R2, F1, F2; GitHub's rungs (`v0.1.0.md`, 2.2) | temper/migration, with the user | not started |

### M0. The mockups join the design

- **Lands** the mockups of `whole-story.md`, section 9, exactly as drawn,
  with a `README.md` in each directory saying what they are and their
  precedence over the greenfield mock (`surfaces.md`, section 13). Mockups
  under `docs/` are documents, whatever their format (the user,
  2026-11-08), so they land as the drafts do.
- **Then** the drafts and `../client/greenfield/README.md` may cite them
  as paths.
- **Done** when it lands. No code.

### P1 to P6. The pages and the drafter

- **Design:** `../client/README.md`'s rail, pages and their parts, and
  `../client/browser.md` where the browser differs; the greenfield
  mock's README for what it now defers to.
- **Builds:** each page as drawn, from the engine's reads, with what a
  read lacks named in the page's lag; Home and Needs you reading every
  project the person is in; Work's one list; Team from the roles and
  people of a project; How work runs drawing the path from the
  settings (`stock-ways.md`, section 4).
- **Tests:** the gallery's states for each page at a laptop's and a
  phone's width, light and dark; the client's joint world for a page
  across two projects; a fuzz test of each page's parts against
  `surfaces.md`, section 12's rules, across random settings and their
  moves; each page's journey recorded.
- **Walk:** on a box, each page against its mockup, at a laptop's and a
  phone's width.

### N1 to N3. What reaches a person

- **Design:** `../domain/forge.md`, 5.8 widened from goals to any task,
  the classes as settings (`../protocol/configuration.md`), rules by
  path in `../domain/landing.md` and `../domain/forge.md`'s landing
  rules, and the person's devices for being told.
- **Tests:** in temper's worlds, each class's need asked, decided and
  listed; Away moving a class and not "always waits"; a path rule
  holding a change until its sign-offs, at the exact head, while an
  unrelated change lands; a push sent to a fake push service for a
  blocking need alone.
- **Walk:** on a box, the classes changed by proposal, a need asked,
  then decided while away and listed; a path rule's change waiting
  while another lands.

### R1 to R3. Standing roles

- **Design:** jig's `domain/tasks.md` and `domain/engine.md` for a task
  that stands, its sessions and a budget per period; temper's
  `../domain/agent.md` for a role's charter and its Now;
  `../domain/findings.md` for routing by area.
- **Tests:** in jig's worlds, a role's session continued while warm and
  started fresh after its quiet span, its Now written at each end, a
  budget refilled per period; in temper's, a red routed to its area's
  role, triaged and repaired by a change the role makes; a finding filed
  by an outside party through `temper ctl`.
- **Walk:** on a box, an area handed to a standing role, a red routed
  to it, its Now on Team, and a rule added by talking with it.

### F1 and F2. Follows

- **Design:** the follow as a project's setting and its task, in
  `../domain/forge.md` beside tiers that follow branches
  (`../domain/after-landing.md`).
- **Tests:** in temper's worlds on the fake forge, a source landing
  making a follow change, in order, its check run, lag and parity kept;
  a held follow showing its lag; skipped and merged follow changes
  leaving the chain unheld; a follow across two projects.
- **Walk:** on a box, two projects' repositories, a landing followed
  and its lag back to zero.

### T1 to T3. The terminal

- **Design:** `../client/README.md`'s TUI sections and `tui.md`,
  section 3, as `surfaces.md`, section 11 revises them.
- **Tests:** golden frames for each screen at two sizes, light and dark;
  the client's joint world for a design session's pane and a role's
  conversation; each screen's journey recorded.
- **Walk:** on a box, each screen against its mockup, in a real
  terminal.

### E and V. The stories

- **E** is written into `../../development/acceptance.md` by the owners
  of what it walks, driven through `temper ctl` on a box with a real
  model, as the ladder's rungs are (`v0.1.0.md`, section 3).
- **V** is the person's own use: vsr, vsr-rs and cowsql adopted on the
  deployment, the roles started, the follow running. Its findings are
  findings like any other.

## 4. Order and seams

- **Wave 0:** M0.
- **Wave 1, at once:** P1, P2, P6, N1, R1, F1, T1.
- **Wave 2, as each need lands:** P3, P5, N2, N3, R2, R3, F2, T2.
- **Wave 3:** P4, T3.
- **Then:** E, once every milestone has landed; V, once its own needs
  have.
- **Seams told before they change:** temper-engine-domain and
  `forge_route.rs`, which N, R and F all write, between temper/ui and
  temper/migration; `temper-client-domain`, which P and T share,
  between temper/ui and temper/tui; jig's tasks and engine, which R
  changes and N reads.
- **The brief is one area's:** Home's brief across projects is P2's,
  temper/ui's; R3 gives it the owners' Nows to read, and changes nothing
  of it.
- **Wide where it can be:** a track may run two plans at once when
  they share no files, such as P1 and P2, or R2 and R3.
- **Started ahead of the go** (the user, 2026-11-08): the terminal's
  directing work. P6 starts at once. T1 starts when `tui-box-front` and
  `client-binary` have landed, and T2 when T1 and P6 have, beside the
  work already in flight. Every other milestone waits for the user's
  word, which comes no earlier than 2026-11-11.
- **Ask temper around it** (the user, 2026-11-08): M2 and M4, which this
  draft needs (`whole-story.md`, section 8), run ahead of the go, after
  P6. M3b runs after this draft's stories, as nothing here needs its
  views. The TUI's drawing of views is dropped for v0.1.0: T1 prints
  them (`surfaces.md`, 11.5).
