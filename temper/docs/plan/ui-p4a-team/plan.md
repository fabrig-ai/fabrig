# ui-p4a-team: Team, its people, what waits on each, and the triager

Milestone P4a of `temper/docs/design/draft/whole-story.execution.md`:
`temper/docs/design/draft/surfaces.md`, section 6, for a project's people,
what waits on each, and the project's triager, on today's engine. The
design this cut wrote, on the trunk with the first push:

- `temper/docs/design/client/README.md`, 3.5: the pages' table's
  `Team { project }` row, and *Team* (its intro, People, a person's line,
  the triager, its parts, its reads, in the rail).
- `temper/docs/design/client/browser.md`: 2.3's `Team` row; 2.5's *Team's
  look*; 7.2's `team`, `team-alone` and `night-team`.
- Status: `README.status.md`'s and `browser.status.md`'s **Not yet**
  entries, one per task that closes it; `draft/surfaces.status.md`'s lag 6
  narrowed, and a **Design departs** for another person's line.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `model` | `tasks/team-surface.md` | | `Team { project }` and its path; `pages::team`'s `Facts`, `gather`, `parts` | 100 |
| `model` | `tasks/team-reads.md` | team-surface | the project's document, what waits on the reader (Needs you's entries) and on others (a `tasks{query}` watch each) | 160 |
| `page` | `tasks/team-page.md` | team-surface | Team's page in both shells, its words, tokens and `temper.css`'s `ui: team` | 200 |
| `proof` | `tasks/team-fuzz.md` | team-surface | section 12's rules over `gather` and `parts`, random members and moves | 70 |
| `proof` | `tasks/team-prints.md` | team-reads, team-page | the gallery's `team`, `team-alone`, `night-team`; the terminal's `team.txt` | 130 |
| `proof` | `tasks/team-joint.md` | team-reads, team-page | the client's joint world: two people read each other on Team, a question counted on both sides | 120 |
| `rail` | `tasks/team-rail.md` | team-reads, team-page, team-prints, team-joint; ui-p1-rail landed | the project's document read on every project page; `rail::facts`' members; "Team" in the rail and "Team for PROJECT" in the switcher's menu | 150 |

About 930 implementer-minutes in all.

**Critical path:** `team-surface` (100) → `team-page` (200) → `team-prints`
(130) → `team-rail` (150): about ten hours. Beside it `team-reads` (160)
ends before `team-page`, `team-joint` (120) runs beside `team-prints`, and
`team-fuzz` (70) starts with `team-page`. The view's tests draw hand-made
`Shown` values, so `team-page` and `team-reads` run at once on their own
crates; the prints and the joint story wait for both, as they step the
real reads.

**One writer per crate:** `team-surface`, then `team-reads`, write
`temper-client-domain`; `team-page` writes `temper-client-view` (and
`temper.css`); the fuzz and the joint story share `temper-client-world`
on disjoint files (`fuzzy/team.rs`, `suite/team.rs`), each with one `mod`
line in `main.rs`; `team-prints` writes the gallery and the terminal's
tests; `team-rail` comes last and writes across them.

**`team-rail` waits on another plan.** It builds on ui-p1-rail's rail
(`pages::rail`, the rail drawn from `rail::parts` in `frame.rs`), which
`needs` can't name. It needs every other task here, so it starts last;
its spec checks the base first and, without ui-p1-rail's rail, changes
nothing and reports `Failed: ui-p1-rail's rail is not on the base`. It is
then run again once ui-p1-rail has landed and the trunk has taken main.
Until it lands, Team is reached at its address (and from the gallery and
the stories), not from the rail.

## What was decided

- **Team draws its people alone:** a project's areas and owners, a role's
  card, "+ Invite" and "Hand an area to a role" are P4b's and P4c's, and
  nothing stands in their place (the execution's section 2).
- **People are members of kind person**, from jig's `Read::Project`, the
  reader first; a service is Deployment's Members', not Team's, and the
  rail's "second member" counts people.
- **What waits on each:** the reader's line is Needs you's open entries in
  the project by kind, exact as the rail's count; another person's line
  is what the engine says waits on them on the tasks they requested
  (`waits_on` on a `tasks{query}` per person, at most eight), since no
  read gives another party's inbox. A **Design departs** in
  `surfaces.status.md` says so.
- **The project's triager is designed, not drawn:** no setting, record or
  read holds a triager today (`findings.status.md`; `landing.status.md`,
  13.1). Team's model has its fact and its rule, gathered as `None`, and
  the status names the lag; the request's "and the project's triager" is
  met as far as the engine holds it, which is not at all.
- **Team's parts are in `pages::team`,** not in ui-p2-home's
  `pages::parts`, so that this plan keeps off its file; *Team*'s
  paragraph says they are section 12's one function for the page.
- **The lede counts people,** "Two of you.", and says "Two people." where
  the person reading isn't among them, as a deployment's owner may not be.
- **Tests stay cheap:** one table test each in the domain and the view,
  one fuzz test of pure functions (under 0.3 s), three gallery states
  measured before and after, one joint story (under 0.6 s); temper's
  focused and fuzzy suites are over their budgets already
  (`temper/docs/design/domain/testing.status.md`).
- **No walk or render check** is in any spec: both follow landing.
- **No stored format, protocol or public interface of the engine
  changes.** The client's `Address` gains `Team { project }`.

## Who hears first

- **The TUI's owner (temper/tui):** `temper-client-domain` gains
  `Address::Team { project }` (`team-surface`), with forced arms in
  `temper-client-tui`'s `view.rs` and `view/launchpad.rs` and
  `temper-client-shell-tui`'s `host.rs`, each as `Builders`'; and the
  module `pages::team` (`Facts`, `gather`, `parts`, `shown`, `people`),
  which T3's launchpad team lines may read; `needs::Session` gains
  `pub(crate) fn answered` (`team-reads`); and, with `team-rail`, the
  project's document read on every project page and `rail::facts`'
  `members` given.
- **The owner of coding runs:** no change to `temper-engine-domain` or
  `forge_route.rs`.

## Overlaps with drafts beside this one

- **`ui-p1-rail`** (the rail, Work, Deployment): `Address`'s forced arms
  meet its `Work`, `Members` and `Limits` in the same `match`es
  (`pages.rs`, `frame.rs`, the TUI's); the README's pages' table and
  status file; `surfaces.status.md`'s lags; the fuzzy suite's `mod`
  block. Team's rail entry is `team-rail`'s, in its files, once it has
  landed.
- **`ui-p2-home`** (Home and Needs you): `Address::AllHome`'s arms beside
  `Team`'s; `pages/needs.rs` and `pages/home.rs` take Team's arm alone,
  and `needs::Session` one `answered` function (`team-reads`); the client
  world's `src/fixtures.rs`, where `team-joint` may add one function.
- **Every UI plan:** `look.rs`'s style tokens, `words.rs`'s `Word`, the
  gallery's `samples.rs` and `names.txt`, `surfaces.status.md`.
