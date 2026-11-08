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
- **Incremental, not a big bang** (the user, 2026-11-08): five steps
  (section 4), each ending in something a person uses on a box. The
  first asks nothing new of the engine, and each later one adds what
  takes more change. A page shows what exists and grows as the
  milestones that feed it land, so results show along the way.
- **Five tracks side by side,** each one area's: P, the pages
  (temper/ui); N, what reaches a person (temper/ui, with temper/migration
  for rules by path); R, standing roles (temper/migration); F, follows
  (temper/migration); T, the terminal (temper/tui).
- **Few waits between tracks** (section 4): a page's first part needs
  nothing but M0, and its later parts wait only for what they draw: How
  work runs for the classes, rules by path and follows; Team for roles
  and areas; talking with a role for roles and the launchpad.
- **Readiness** is section 5: what runs, and what waits on work in
  flight.
- **The stories grow with the steps:** each step adds its part of E, on
  a box through `temper ctl`, and V, the vsr pilot on a real deployment,
  starts with vsr alone (`whole-story.md`, section 11).
- **Speed comes from width:** each track runs its milestones one plan
  at a time, the tracks side by side, with the shared seams in section
  4 told before they change.

## 2. How a milestone runs

- **Its entry here is its input,** its draft the source, the mockups
  its look.
- **It draws what exists:** what the engine holds when it lands, and no
  more. What a later milestone adds is left out, with no empty section,
  no placeholder and no made-up value, and that milestone draws it. A
  mockup is where a page ends up, not what its first milestone shows.
- **It can be seen:** section 4 says what a person can do on a box once
  its step's milestones have landed.
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
| M0 | The mockups join the design | `whole-story.md`, 9; `surfaces.md`, 13 | | temper/draft/ux | done |
| P1 | The rail and Work | `surfaces.md`, 2, 5, 8 | M0 | temper/ui | in flight |
| P2 | Home and Needs you across projects, and the brief | `surfaces.md`, 3, 4 | M0 | temper/ui | in flight |
| P3a | How work runs: the page and the path, on today's settings | `surfaces.md`, 7, but for the parts below; `stock-ways.md`, 4 | M0 | temper/ui | not started |
| P3b | How work runs: What reaches you | `surfaces.md`, 7 | P3a, N1 | temper/ui | not started |
| P3c | How work runs: Rules by path, and the path's stops for a path set, "if core" and "one of you" | `surfaces.md`, 7; `what-reaches-you.md`, 7 | P3a, N2 | temper/ui | not started |
| P3d | How work runs: What follows a landing, and a role's budget | `surfaces.md`, 7 | P3a, F1, R1 | temper/ui | not started |
| P4a | Team: its people, what waits on each, and the project's triager | `surfaces.md`, 6 | M0 | temper/ui | not started |
| P4b | Team: a role's card, but for "Talk to it" | `surfaces.md`, 6 | P4a, R1 | temper/ui | not started |
| P4c | Team: areas and their owners, "Hand an area to a role" and "Talk to it" | `surfaces.md`, 6 | P4b, R2; R3 for "Talk to it" | temper/ui | not started |
| P5a | The phone, Ask, and Tell temper filing for the project's triager | `surfaces.md`, 9, 10 | P2 | temper/ui | not started |
| P5b | Tell temper's routing to an area's owner | `surfaces.md`, 9 | P5a, R2 | temper/ui | not started |
| P6 | The drafter's charter | `surfaces.md`, 11.2, its instructions | | temper/ui | in flight |
| N1 | Three classes, Away, why and Stop asking, Decided for any task | `what-reaches-you.md`, 2 to 6 | R1 for a standing role's case | temper/ui | in flight |
| N2 | Rules by path | `what-reaches-you.md`, 7 | N1 | temper/migration | not started |
| N3a | Only you | `what-reaches-you.md`, 8 | N1 | temper/ui | not started |
| N3b | Being told: a push, a mail | `what-reaches-you.md`, 9 | N1 | temper/ui | not started |
| R1 | A standing role and its sessions | `standing-roles.md`, 2, 3, 5, 9 | | temper/migration | in flight |
| R2 | Areas, routing, outside parties, stock roles | `standing-roles.md`, 4, 8, 10 | R1 | temper/migration | not started |
| R3 | Talking with a role, and its reports | `standing-roles.md`, 6, 7 | R1 | temper/migration | not started |
| F1 | A follow, across projects | `follows.md`, 2 to 6 | R1 for a standing role as its owner; a person owns it until then | temper/migration | in flight |
| F2 | Depends on | `follows.md`, 7 | F1 | temper/migration | not started |
| G1 | A goal across two projects' repositories | a draft of its own, not yet written (`whole-story.md`, 5.2) | its draft; F1 | temper/migration, its draft temper/draft/migration | not started |
| T1 | The launchpad as drawn, and views printed | `surfaces.md`, 11.1, 11.4, but its team lines, `/talk` and `/tell`; 11.5 | M0 | temper/tui | not started |
| T2 | A design session | `surfaces.md`, 11.2 | T1, P6 | temper/tui | not started |
| T3 | Talking with a role, and the launchpad's team | `surfaces.md`, 11.1's team lines, 11.3, 11.4's `/talk` and `/tell` | T1, R2, R3 | temper/tui | not started |
| E | The whole story on a box, a part for each step (section 4) | `whole-story.md`, 11 | each part, its step's milestones | each owner writes its part of the story | not started |
| V1 | The pilot, vsr alone: adopted, its way Draft first, its checks as CI, a goal from a draft | `whole-story.md`, 5.2 | GitHub's rungs (`v0.1.0.md`, 2.2); a worker with vsr's C toolchain | temper/migration, with the user | not started |
| V2 | The pilot: vsr's protocol core waiting for the person's sign-off | `whole-story.md`, 5.2 | V1, N2 | temper/migration, with the user | not started |
| V3 | The pilot: Prover owns vsr's tests and triage | `whole-story.md`, 5.2 | V1, R2 | temper/migration, with the user | not started |
| V4 | The pilot: vsr-rs follows vsr, Porter owning it | `whole-story.md`, 5.2 | V1, F1, R2 | temper/migration, with the user | not started |
| V5 | The pilot: cowsql, and a week of the person's use | `whole-story.md`, 5.2 and 11 | V1 to V4, F2, G1 | temper/migration, with the user | not started |

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
- **Builds:** each page as far as its row in section 3 goes, from the
  engine's reads, with what a read lacks named in the page's lag; Home
  and Needs you reading every project the person is in; Work's one list;
  Team from a project's people, then its roles and areas; How work runs
  drawing the path from the settings (`stock-ways.md`, section 4).
- **Tests:** the gallery's states for each page at a laptop's and a
  phone's width, light and dark; the client's joint world for a page
  across two projects; a fuzz test of each page's parts against
  `surfaces.md`, section 12's rules, across random settings and their
  moves; each page's journey recorded.
- **Walk:** on a box, each page against the part of its mockup its
  milestones have drawn, at a laptop's and a phone's width.

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
  model, as the ladder's rungs are (`v0.1.0.md`, section 3). Each step
  adds its part (section 4), walked once that step's milestones have
  landed.
- **V** is the person's own use, begun early: vsr alone on the
  deployment, then its sign-off, Prover, vsr-rs and Porter, and cowsql,
  each as what it needs lands (section 3). Its findings are findings
  like any other.

## 4. Order and seams

The steps come first: what a person can do once each has landed. They
set what is built first, and what yields when builders run short. They
hold nothing back: a milestone starts as soon as section 5 clears it,
whatever its step, so that work needing more change starts early and
lands when it's ready.

| Step | Milestones, in order | What a person can do then | Its part of E, and of V |
|---|---|---|---|
| 1. The new shape, on today's engine | P1, P2, P3a, P4a, P5a, P6, T1, T2 | Move around the new rail; read Home and Needs you across projects; find any work in Work; read a project's real path on How work runs, from its settings as they stand; see its people on Team; use the phone; write a draft in a design session in the terminal, and start a goal from it | E: Pair. V1 |
| 2. What reaches you | N1, N3a, P3b, N3b | See why a need reached them, and stop asking; go Away; read what was decided for them, on any task; see what only they can do; read What reaches you on How work runs; then get a push for a blocking need | E: what reaches a person |
| 3. Sign-offs by path | N2, P3c | Say that some paths need their sign-off, see that stop on the path, and watch a change wait for it while others land | E: Team. V2 |
| 4. Standing roles | R1, P4b, R2, P4c, P5b, R3, T3 | Start a role and read its Now on Team; hand it an area, and see a red reach it and get repaired; have Tell temper file with an area's owner; talk with a role | E: standing roles. V3 |
| 5. Follows | F1, P3d, F2, G1 | Have one project follow another, and see a landing become a change there, its lag back at zero; start one goal whose changes land in two projects' repositories | E: follows. V4, then V5 |

- **Each milestone is usable alone:** one that finishes early shows
  through today's pages, as N1's why and Stop asking do before P3b draws
  its columns.
- **Then:** E whole, once its five parts have been walked; V, part by
  part, as its needs land.
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
- **The go** (the user, 2026-11-08): the whole story starts now, each
  milestone as soon as section 5 clears it, in parallel as far as the
  builders allow. Work in flight that helps it carries on. The terminal's
  directing work leads: P6 at once, T1 when `tui-box-front` and
  `client-binary` have landed, T2 when T1 and P6 have.
- **No walk of what will change** (the user, 2026-11-08): until its
  milestone has landed, a walk skips what that milestone replaces: the
  launchpad's layout before T1, the rail and the Goals and Changes lists
  before P1, and the ways' drawing and words and Settings' How work runs
  tab before P3a. A real bug there is still repaired.
- **Ask temper around it** (the user, 2026-11-08): M2 and M4, which this
  draft needs (`whole-story.md`, section 8), run beside it. M3b runs after this draft's stories, as nothing here needs its
  views. The TUI's drawing of views is dropped for v0.1.0: T1 prints
  them (`surfaces.md`, 11.5).

## 5. Readiness

As read on 2026-11-08 at 17:14Z, from the areas' work in flight. Plans
land meanwhile, so read what a row waits on afresh before starting it.

| | State | Waits on, and why |
|---|---|---|
| M0 | done | |
| P1 | in flight | `ui-p1-rail` |
| P2 | in flight | `ui-p2-home` |
| P3a | waits | temper/ui's `look-round-24`, the ways page's drawing, which How work runs takes over |
| P3b, P3c, P3d | wait | P3a, and N1, N2 or F1 and R1, by their needs |
| P4a | starts | |
| P4b, P4c | wait | R1, then R2 and R3, by their needs |
| P5a | waits | P2, by its needs |
| P5b | waits | P5a and R2, by its needs |
| P6 | in flight | `ui-drafter` |
| N1 | in flight | `ui-reaches` |
| N2 | waits | N1, by its needs |
| N3a, N3b | wait | N1, by their needs |
| R1 | in flight | goal 101 |
| R2, R3 | wait | R1, by their needs |
| F1 | in flight | goal 102 |
| F2 | waits | F1, by its needs |
| G1 | waits | its draft, which holds open design: a goal branch per repository, two landings and one of them refused, whose rules and people govern the goal; F1 |
| T1 | waits | temper/tui's `tui-box-front`, which changes how the TUI starts |
| T2 | waits | T1 and P6, by its needs; finding-465's repair, a chat losing its opening question |
| T3 | waits | T1, R2 and R3, by its needs |
| E | part by part | each part, its step's milestones |
| V1 | waits | GitHub's rungs; a worker with vsr's C toolchain |
| V2 to V5 | wait | V1, and each its needs in section 3 |

- **Next to start:** P4a now, as temper/ui's places allow; P3a when
  `look-round-24` lands; T1 when `tui-box-front` does.
