# whole-story: execution

2026-11-11. How [whole-story.md](whole-story.md) and the four drafts
written with it, `standing-roles.md`, `what-reaches-you.md`, `follows.md`
and `surfaces.md`, reach the design and the code by v0.1.0, in tracks
that run side by side. It designs nothing: each milestone writes its own
design from its draft as it runs (`DESIGN.md`, section 3). A milestone
that meets a question its draft leaves open takes the draft's default,
or the nearest thing, and says so in that draft's status.

## 1. In one page

- **M0 first, small and alone:** the mockups join the design
  (`whole-story.md`, section 9).
- **Must, should, nice** (the user, 2026-11-08), each milestone's rank
  in section 3: must, as E's and V's bars stand on it; should, cheap and
  keeping a page true; nice, built last, and it may follow v0.1.0. From
  the mockups' audit, which also added N4 to N6, P3e, P3f, P7 and P8 and
  sent the rest Later (`whole-story.md`, section 13).
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
  (`../client/web/journeys.md`; `../client/tui/journeys.md`).
- **Its walk** follows its landing and blocks nothing.
- **Its state** is section 3's table, kept current. When every
  milestone is done and the user has taken the pages and frames, the
  drafts' statuses hold no lag of what v0.1.0 needs, and this file goes
  (`DESIGN.md`, section 3).

## 3. The milestones

| | Milestone | Executes | Needs | Owner | Rank | State |
|---|---|---|---|---|---|---|
| M0 | The mockups join the design | `whole-story.md`, 9; `surfaces.md`, 13 | | temper/draft/ux | must | done |
| P1 | The rail and Work | `surfaces.md`, 2, 5, 8 | M0 | temper/ui | must | landed; its render check and walk follow (section 5) |
| P2 | Home and Needs you across projects, and the brief | `surfaces.md`, 3, 4 | M0 | temper/ui | must | landed; its render check and walk follow (section 5) |
| P3a | How work runs: the page and the path, on today's settings | `surfaces.md`, 7, but for the parts below; `stock-ways.md`, 4 | M0 | temper/ui | must | landed; its render check and walk follow (`../client/README.md`, 5) |
| P3b | How work runs: What reaches you | `surfaces.md`, 7 | P3a, N1 | temper/ui | nice | landed; its render check and walk follow (`../client/README.md`, 5) |
| P3c | How work runs: Rules by path, and the path's stops for a path set, "if core" and "one of you" | `surfaces.md`, 7; `what-reaches-you.md`, 7 | P3a, N2 | temper/ui | should | landed; its render check and walk follow (`../client/README.md`, 5) |
| P3d | How work runs: What follows a landing, and a role's budget | `surfaces.md`, 7 | P3a, F1, R1 | temper/ui | nice | landed; its render check and walk follow (`../client/pages/settings.md`, 3) |
| P3e | Words saved with a setting: a charter's line on Roles, How work runs' headline, a tier's description | `surfaces.md`, 7; its entry below | P3a | temper/ui | should | landed; its render check follows (`../client/README.md`, 5) |
| P3f | The path's "design" and "close-out", from what a goal's template says its coordinator does | `stock-ways.md`, 4; its entry below | P3a | temper/ui | should | done |
| P4a | Team: its people, what waits on each, and the project's triager | `surfaces.md`, 6 | M0 | temper/ui | must | landed and repaired; its render check and walk follow (`../client/README.md`, 5) |
| P4b | Team: a role's card, but for "Talk to it" | `surfaces.md`, 6 | P4a, R1 | temper/ui | must | landed; its render check and walk follow (`../client/README.md`, 5) |
| P4c | Team: areas and their owners, "Hand an area to a role" and "Talk to it" | `surfaces.md`, 6 | P4b, R2; R3 for "Talk to it" | temper/ui | nice | not started |
| P5a | The phone, Ask, and Tell temper filing for the project's triager | `surfaces.md`, 9, 10 | P2 | temper/ui | should | landed but for Tell temper, which waits (section 5) |
| P5b | Tell temper's routing to an area's owner | `surfaces.md`, 9 | P5a, R2 | temper/ui | nice | not started |
| P6 | The drafter's charter | `surfaces.md`, 11.2, its instructions | | temper/ui | must | done |
| P7 | A change's size on its approval card: its files, additions and deletions at the head reviewed | `surfaces.md`, 4; its entry below | P2 | temper/ui | nice | landed; its render check and walk follow (`../client/README.md`, 5) |
| P8 | Work's drafts landed and ready to start a goal | `surfaces.md`, 5; its entry below | P1, P6 | temper/ui | nice | landed; its render check follows (`../client/README.md`, 5) |
| N1 | Three classes, Away, why and Stop asking, Decided for any task | `what-reaches-you.md`, 2 to 6 | R1 for a standing role's case | temper/ui | must | landed, R1 a standing role's case; its repair's web part landed, its render check and walk following (section 5) |
| N2 | Rules by path | `what-reaches-you.md`, 7 | N1 | temper/migration | must | landed; its walk follows (section 5) |
| N3a | Only you | `what-reaches-you.md`, 8 | N1 | temper/ui | nice | landed but for what waits on a login and Only you's other kinds, which wait on reads (section 5) |
| N3b | Being told: a push, a mail | `what-reaches-you.md`, 9 | N1 | temper/ui | must for a push, nice for a mail | not started |
| N4 | A person's request for changes on the forge starts a repair with their remarks, as a blocking review's does | `../domain/forge/change/gates.md`, 1; its entry below | | temper/migration | should | done |
| N5 | Who can still sign: the people a waiting sign-off can still come from, on its card | `what-reaches-you.md`, 7; its entry below | N2 | temper/ui | should | not started |
| N6 | Sign-offs kept at landing: who signed, when and at which head, for Work, Team and the brief | `what-reaches-you.md`, 7; its entry below | N2 | temper/migration | nice | not started |
| R1 | A standing role and its sessions | `standing-roles.md`, 2, 3, 5, 9 | | temper/migration | must | landed; its walk follows R2's and R3's (*R1 to R3*, below) |
| R2 | Areas in words, the triager's routing, outside parties, stock roles | `standing-roles.md`, 4, 8, 10 | R1 | temper/migration | must | not started |
| R3 | Talking with a role, and its reports | `standing-roles.md`, 6, 7 | R1 | temper/migration | must for talking, nice for reports | in flight: goal 712 |
| F1 | A follow, across projects | `follows.md`, 2 to 6 | R1 for a standing role as its owner; a person owns it until then | temper/migration | must | landed for a person as owner; a standing role as owner waits on R1 (section 5) |
| F2 | Depends on | `follows.md`, 7 | F1 | temper/migration | must | landed; its walk follows (section 5) |
| G1 | A goal across two projects' repositories | a draft of its own, not yet written (`whole-story.md`, 5.2) | its draft; F1 | temper/migration, its draft temper/draft/migration | later | low priority, not for v0.1.0 |
| T1 | The launchpad as drawn, and views printed | `surfaces.md`, 11.1, 11.4, but its team lines, `/talk` and `/tell`; 11.5 | M0 | temper/tui | must | landed; its walk follows (`../client/README.md`, 5) |
| T2 | A design session | `surfaces.md`, 11.2 | T1, P6 | temper/tui | must | landed; its walk follows (`../client/README.md`, section 5) |
| T3 | Talking with a role, and the launchpad's team | `surfaces.md`, 11.1's team lines, 11.3, 11.4's `/talk` and `/tell` | T1, R2, R3 | temper/tui | nice | not started |
| E | The whole story on a box, a part for each step (section 4) | `whole-story.md`, 11 | each part, its step's milestones | each owner writes its part of the story | must | not started |
| V1 | The pilot, vsr alone: adopted, its way Draft first, its checks as CI, a goal from a draft | `whole-story.md`, 5.2 | GitHub's rungs (`v0.1.0.md`, 2.2); a worker with vsr's C toolchain | temper/migration, with the user | must | not started |
| V2 | The pilot: vsr's protocol core waiting for the person's sign-off | `whole-story.md`, 5.2 | V1, N2 | temper/migration, with the user | must | not started |
| V3 | The pilot: Prover owns vsr's tests and triage | `whole-story.md`, 5.2 | V1, R2 | temper/migration, with the user | must | not started |
| V4 | The pilot: vsr-rs follows vsr, Porter owning it | `whole-story.md`, 5.2 | V1, F1, R2 | temper/migration, with the user | must | not started |
| V5 | The pilot: cowsql, and a week of the person's use | `whole-story.md`, 5.2 and 11 | V1 to V4, F2 | temper/migration, with the user | must | not started |

### M0. The mockups join the design

- **Lands** the mockups of `whole-story.md`, section 9, exactly as drawn,
  with a `README.md` in each directory saying what they are and their
  precedence over the greenfield mock (`surfaces.md`, section 13). Mockups
  under `docs/` are documents, whatever their format (the user,
  2026-11-08), so they land as the drafts do.
- **Then** the drafts and `../client/web/greenfield/README.md` may cite them
  as paths.
- **Done** when it lands. No code.

### P1 to P6. The pages and the drafter

- **Design:** `../client/README.md`'s rail, pages and their parts, and
  `../client/web/README.md` where the browser differs; the greenfield
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

- **Design:** `../domain/forge/attention.md`, 4 widened from goals to any task,
  the classes as settings (`../protocol/configuration/README.md`), rules by
  path in `../domain/landing.md` and `../domain/forge/README.md`'s landing
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

- **Design:** jig's `domain/tasks/README.md` and `jig/docs/design/domain/engine/README.md` for a task
  that stands, its sessions and a budget per period; temper's
  `../domain/agent/README.md` for a role's charter and its Now;
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
  `../domain/forge/README.md` beside tiers that follow branches
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

### N4 to N6, P3e, P3f, P7 and P8. From the mockups' audit

Each fills what the mockups draw and no draft delivered (the user,
2026-11-08). Its milestone writes its design, as the others do.

- **N4:** a person who asks for changes on the forge's review page holds
  the change today (`../domain/forge/change/gates.md`, 1). Their remarks become a
  repair's brief, as a blocking review's are, so You approve's loop closes
  on the forge.
- **N5:** a sign-off counts people of one role (`../domain/forge/change/gates.md`, 1).
  The card names who can still give the next one: the role's members with
  a forge identity, less those who signed this head. It replaces "whose
  turn", which nothing assigns.
- **N6:** the reviews counted at a change's landing kept with it: the
  person, the head and when. Work's "signed by", Team's counts and the
  brief read them.
- **P3e:** words a person or Ask temper writes with a setting, kept with
  it: a charter's line on Roles, How work runs' headline and lede, and a
  tier's description. Without one, Roles shows the instructions' first
  sentence.
- **P3f:** a goal's template says whether its coordinator designs first
  and whether it makes a close-out, so the path draws "design" and
  "close-out" from the settings rather than guessing from instructions.
- **P7:** the forge's own counts of a change's files, additions and
  deletions at the head reviewed, kept with the change; a count that is
  cut says so.
- **P8:** a draft landed in a repository, named with its repository,
  path and commit, its chat and its goal once started, and ready when
  its writer says so.

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
| 1. The new shape, on today's engine | P1, P2, P3a, P3e, P3f, P4a, P5a, P6, T1, T2, P7, P8 | Move around the new rail; read Home and Needs you across projects; find any work in Work; read a project's real path on How work runs, from its settings as they stand; see its people on Team; use the phone; write a draft in a design session in the terminal, and start a goal from it | E: Pair. V1 |
| 2. What reaches you | N1, N3a, P3b, N3b | See why a need reached them, and stop asking; go Away; read what was decided for them, on any task; see what only they can do; read What reaches you on How work runs; then get a push for a blocking need | E: what reaches a person |
| 3. Sign-offs by path | N2, N4, N5, P3c, N6 | Say that some paths need their sign-off, see that stop on the path, and watch a change wait for it while others land | E: Team. V2 |
| 4. Standing roles | R1, P4b, R2, P4c, P5b, R3, T3 | Start a role and read its Now on Team; hand it an area, and see a red reach it and get repaired; have Tell temper file with an area's owner; talk with a role | E: standing roles. V3 |
| 5. Follows | F1, P3d, F2 | Have one project follow another, and see a landing become a change there, its lag back at zero | E: follows. V4, then V5 |

- **By rank within each step** (section 3): must and should first. A
  nice milestone starts once places are free of the must ones, in any
  step, and may land after v0.1.0.
- **Each milestone is usable alone:** one that finishes early shows
  through today's pages, as N1's why and Stop asking do before P3b draws
  its columns.
- **G1 is low priority** (the user, 2026-11-08): it doesn't block
  v0.1.0, and runs if there's room. Until it has, work on vsr and on
  cowsql runs as a goal per repository.
- **Routing by path is Later** (the user, 2026-11-08; `standing-roles.md`,
  section 13): in R2 the project's triager routes by judgement, from the
  areas' words. Sign-offs by path stay, in step 3: a landing's rule can't
  be left to judgement.
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

As read on 2026-11-08 at 17:14Z, from the areas' work in flight,
P3's rows at 19:45Z, as P3a retired, P6's at 22:21Z, as it retired, and
P1's and P8's at 22:55Z, as P1 retired, and P2's, P5a's and P7's at
23:32Z, as P2 retired, and P3a's and P3e's on 2026-11-09 at 00:10Z,
as P3a's repair, `look-round-25`, retired, and P1's at 00:42Z, as
`ui-away-all` retired, and P1's and P4a's at 01:02Z, as P4a retired,
and P1's, P5a's and P5b's at 01:30Z, as P5a retired, and P3e's and
P3f's at 02:25Z, as P3f retired, and P3e's at 03:18Z, as it retired, and
N1's, N2's, N3a's, N3b's and P3b's at 04:11Z, as N1 retired, and
N3a's at 15:34Z, as it retired, and P3b's at 15:59Z, as it retired, and
P4a's on 2026-11-11, as its repair retired, and
P3a's and P3e's on 2026-11-11, as goal 310, their repairs, landed, and
P1's that day, as goal 368, its repair, landed, and
P2's, as goal 338, its repair's wording, landed, and
N1's that day, as goal 367, its repair's web part, landed, and
R1's, R2's, R3's, P4b's and N1's on 2026-11-11, as R1 landed, and
P1's, P4b's and P4c's that day, as goal 713, P4b, landed, and
R3's that day, as goal 712 designed it, and
N1's, N2's, N5's, N6's and P3c's that day, as N2 retired, and
P3c's that day, as goal 835 cut it, and
F1's, F2's and P3d's on 2026-11-11, as F1 landed, and
P7's that day, as goal 493 landed, and
P8's, as goal 638 landed, and
T1's that day, as goal 761, its repair, landed, and
P3d's on 2026-11-12, as goal 1016 landed, and
F2's on 2026-11-12, as goal 1015 landed, and
P2's and F2's that day, as goal 1243, P2's repair for What follows
what, landed. Plans
land meanwhile, so read what a row waits on afresh before starting it.

| | State | Waits on, and why |
|---|---|---|
| M0 | done | |
| P1 | landed; its render check and walk follow | `ui-p1-rail` drew the rail as this draft stood at its cut; its repair, goal 368, draws To begin's rail with its hairline and "How work runs", turns the words that still sent a person to Settings to How work runs and Deployment, and keeps "Home" and "Needs you" across projects under All projects, Needs you's count across them (`../client/view.md`, 3); the phone's "Needs" landed with P5a; Away across projects landed by `ui-away-all`; Team in the rail landed with P4a, for a second person, and with P4b, for a standing role; its render check and walk, and `project-landing`'s new recording of To begin's rail, follow landing (`../client/README.md`, 5) |
| P2 | landed; its render check and walk follow | `ui-p2-home` drew Home and Needs you across projects; its repair, goal 338, words the byline across projects as `surfaces.md`, 3 does and gives the phone's headline `phone-home.html`'s words; a second repair, goal 1243, draws What follows what on Home on All projects, a box each follow and each dependency of the first four projects read, from their `follows` documents, within a shipped client's watches raised to 26 (`../client/pages/home.md`, 1); a project's row's Now, tiers, a follow's parity and "owned" wait on R1, R3 and F1, by their needs, and Decided for you's who and a card's asker are Needs you's (`surfaces.status.md`, 3 and 4); its render check and walk follow landing (`../client/README.md`, 5) |
| P3a | landed; its render check and walk follow | `ui-p3a-how` drew the path in lanes, and its repair, `look-round-25`, draws it as Ask temper's ways do, their agent's star and its rise in ink, and Roles; a second repair, goal 310, gives the drafter, in the stock since P6, its place among Roles; their render check and walk follow landing (`../client/README.md`, 5) |
| P3b | landed; its walk follows | `ui-p3b-reaches` drew `surfaces.md`, 7's What reaches you from the project's classes, three columns in `how.html`'s words with what Away changes, then the question route and "Away until", in place of the classes' checks, a class changing by proposal; the mockups' lines no class holds, a landing rule's approvals and an agent's own repairs, stay the path's and Rules by path's, P3c (`surfaces.status.md`, 7 and 13); its render check and walk follow landing (`../client/README.md`, 5) |
| P3c | landed; its render check and walk follow | goal 835 derived a role's sign-off as one stop, "you", "one of you" or the role's name, with "if NAME" or "by path" over a stop that only rules by path ask, and its card's "Only for", "Asks" and "Counts" (`../domain/forge/change/branch-path.md`); the `ways` read's rules by path, which `see ways` prints; and How work runs' Rules by path, as `how.html` and `team/how.html` draw it; the heading's chief departure is the project's words, which Ask temper writes; its render check and walk follow landing (`../client/README.md`, 5) |
| P3d | landed; its render check and walk follow | goal 1016 draws `surfaces.md`, 7's What follows a landing on How work runs: a box a follow of the project's, from the forge's `follows` document, its words made from its setting, its lag and parity, and who acts' chips, then a box of its path's tiers after landing; Budgets' rows for the project's live standing roles, each its title, what it has left and its budget, under the project's open period, "a week · renews DAY", then "Goals"; and Roles the charters its standing roles and follows run (`../domain/forge/change/branch-path.md`, *The project's roles*). Where they depart from `how.html` is `surfaces.status.md`, 7 and 13; their render check and walk follow landing (`../client/pages/settings.md`, 3) |
| P3e | landed; its render check follows | `ui-p3e-words` keeps a charter's words, kind 9, and a project's, kind 10, which Ask temper writes with what it proposes, the stock's for each stock charter, a headline for each stock way, which How work runs' heading and lede say, the project's own where it runs none, and a tier's description, which its card says; the stock holds no tier, so none of its words. Its repair, goal 310, carries kind 9 on the `roles` view's cards, after when the role runs, its instructions' first sentence where none is kept, as `how.html` does, and has `see ways` say each way's headline; its render check follows landing |
| P3f | done | |
| P4a | landed and repaired; its render check and walk follow | `ui-p4a-team` drew Team's people, and its repair gives a box the role "Owner", People's head the role its people share, "owners of shop", and a line's count in words (`../client/pages/team.md`, 1); the triager as Who routes, under the areas, is P4c's; a line's sign-offs wait on N2, N5 and N6 (`surfaces.status.md`, 6); its render check and walk follow landing (`../client/README.md`, 5) |
| P4b | landed; its render check and walk follow | goal 713 drew a project's standing roles on Team, a row each with its Now's first line and its period's spend, and a role's card: its head with Pause and Resume, its Now, its rules, what wakes it and its sessions this week, and Team in the rail for a standing role (`../client/pages/team.md`, 1); a card's "owns AREA", "Talk to it", "Give the area back" and weekly report are P4c's, and a forge topic's words, a charter's kept words and a session's line wait for reads no milestone above builds (`surfaces.status.md`, 6); its render check and walk follow landing (`../client/README.md`, 5) |
| P4c | waits | R2 and R3, by its needs, P4b having landed |
| P5a | landed; Tell temper waits | `ui-p5a-phone` drew the phone's tabs from one pure rule, "Needs" with its count and the Ask tab's tabs under Ask temper's page and its chats, Needs you's phone bar, and every target of the frame, Home, Needs you, Work and a chat's bar and composer at 44 px; its render check and walk follow landing (`../client/README.md`, 5). Tell temper is designed and drawn nowhere: it waits for the engine to file a person's word as a finding and for a setting naming the project's triager, which no milestone above builds, and follows as a milestone of its own once they land (temper/ui's answer to the plan's question `tell-temper-findings`; `surfaces.status.md`, 9) |
| P5b | waits | R2, by its needs, and Tell temper's filing (P5a) |
| P6 | done | |
| P7 | landed; its render check and walk follow | goal 493 reads the forge's own counts of a pull request's files, additions and deletions with it (skein's `forge/README.md`, 3.6), keeps them with the change's approval evidence at the head reviewed (`../domain/forge/change/gates.md`, 2), and draws them on the approval's card on Needs you and Home, "+142 -18 in 4 files", a count past 32 bits marked "+" (`../client/forge/README.md`, 4.1); `pair/needs.html`'s table of files, the CI and repair chips and the launchpad's line wait (`surfaces.status.md`, 4 and 11); its render check and walk follow landing (`../client/README.md`, 5) |
| P8 | landed; its render check follows | goal 638 keeps a project's landed drafts (`../domain/forge/drafts.md`): a writer says one ready by `temper ctl draft ready`, temper reads it at the landing branch's head and keeps its commit, title, chat and, once started, its goal; Work lists the ready ones under Drafts and starts a goal from each (`../client/pages/work.md`, 1); saying ready on the web or in a design session, and a goal's card naming its draft, are those documents' open questions; its render check follows landing (`../client/README.md`, 5) |
| N1 | landed; its render check and walk follow | `ui-reaches` built `what-reaches-you.md`, 2 to 6, in the engine and the web client, and R1 a standing role's case; judging the paths an asker's change touches at its head, read since N2, is N1's repair's (`what-reaches-you.status.md`, 4); its repair, goal 367, draws Needs you's Decided head with "Limits" on a filtered page, a words offer's sentence over its field and Home's Away spell with no row showing, and the why line on the signal ground, over the choices, with "Keep asking" and the narrower rule on it, as `needs.html` does, saying "agents" where it says "goals" (`surfaces.status.md`, 4); the TUI's walk says a need's reason since T1's repair, goal 761 (`../client/tui/pages/launchpad.md`, 6); its render check and walk follow landing (`../client/README.md`, 5) |
| N2 | landed; its walk follows | goal 187 built `what-reaches-you.md`, 7, in the engine and the web client: a landing rule's `sets` of the project's path sets, through settings files and ctl; what a change touches, from its merge base to its head, judged at that exact head by every reader of the rules and by the turn's merge; a change whose sign-off waits out of its branch's queue, so the rest of the branch lands; and Needs you's card saying the path sets it signs off, with "Sign off on Forgejo ↗"; the domain world's story holds a path rule's change for its sign-off while another lands. The path's stop for a path set is P3c's (`what-reaches-you.status.md`, 7); its walk follows landing |
| N3a | landed; its later parts wait | `ui-n3a-only` built `what-reaches-you.md`, 8, in the web client: a person's own sign-in signed out, or a login read as expiring, listed apart on Needs you with "Sign in again" to Accounts, which offers an expiring source its renewal; the line saying what waits ("Two agents on vsr wait for this login."), a forge's token to replace and a person task asking an act wait for reads no milestone above builds, and the terminal's inbox is temper/tui's (`what-reaches-you.status.md`, 8); a repair, or a look request, draws an item's from line, question and gap as `needs.html` does (`surfaces.status.md`, 4); its render check and walk follow landing |
| N3b | starts | |
| N4 | done | goal 147, PR #41 |
| N5, N6 | start | |
| R1 | landed; its walk follows | goal 63: jig's task that stands, its sessions and its Now (jig's `domain/tasks/standing.md`), and temper's standing role, its stock charter and template, its person and `role start`, `role list` and `role show` (`../domain/agent/standing.md`); its walk is R2's and R3's |
| R2 | starts | R1 has landed |
| R3 | in flight | goal 712: jig's talk (jig's `domain/tasks/standing.md`, 5.6), the Now's `NowWritten` line and period, and temper's talk, `role talk`, the weekly report and the roles' Nows in Home's brief's words (`../domain/agent/standing.md`, sections 7 and 8); Team's and the terminal's drawing of them are P4c's and T3's |
| F1 | landed for a person as owner | a follow owned by a person, its queue, its follow task, the owner's acts in `temper ctl`, its check and its lag and parity (`../domain/forge/follows.md`); a standing role as owner, its acts as a tool and its questions, wait on R1 (`follows.status.md`); its walk follows landing |
| F2 | landed; its walk follows | goal 1015: kind 13, a project's dependency on another project of the deployment, judged by the root, carried in the forge's `follows` document beside the follows and printed by `temper ctl follow list`, "dependency 1: fabrig depends on vsr-rs", acted on by nothing (`../domain/forge/dependencies.md`); Home's What follows what draws follows and dependencies from that document since P2's repair, goal 1243 (`../client/pages/home.md`, 1); its walk follows landing |
| G1 | later | low priority (section 4). It needs its draft first, which holds open design: a goal branch per repository, two landings and one of them refused, whose rules and people govern the goal; and F1 |
| T1 | landed; its walk follows | `tui-whole-t1` drew the launchpad and printed views; its repair, goal 761, reads the brief, Where you left off and Needs you across the person's projects, says why a walked need reached the person, when each need asked and a left-off row's day, and draws the blank row under the header (`../client/tui/pages/launchpad.md`); Stop asking in the terminal stays the web's (`surfaces.status.md`, 11); its walk follows landing (`../client/README.md`, 5) |
| T2 | landed | goal 172: `../client/tui/pages/design-session.md`, 1, on skein's side region (`skein/docs/design/ui/screen.md`, 5.2); its walk follows landing |
| T3 | waits | T1, R2 and R3, by its needs |
| E | part by part | each part, its step's milestones |
| V1 | waits | GitHub's rungs; a worker with vsr's C toolchain |
| V2 to V5 | wait | V1, and each its needs in section 3 |

- **Next to start:** N3b, must; N5 and N6, N2 having landed.
