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

Being written: `team-prints` (the gallery's states and the terminal's
print), `team-joint` (the client world with two people) and `team-rail`
(the meeting with `ui-p1-rail`'s rail).

**Critical path, as it stands:** `team-surface` (100) → `team-page` (200)
→ the prints; `team-reads` (160) runs beside `team-page`, and the view
draws hand-made `Shown` values in its tests so that neither waits on the
other.

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
- **No stored format, protocol or public interface of the engine
  changes.** The client's `Address` gains `Team { project }`.

## Who hears first

- **The TUI's owner (temper/tui):** `temper-client-domain` gains
  `Address::Team { project }` (`team-surface`), with forced arms in
  `temper-client-tui`'s `view.rs` and `view/launchpad.rs` and
  `temper-client-shell-tui`'s `host.rs`, each as `Builders`'; and the
  module `pages::team` (`Facts`, `gather`, `parts`, `shown`, `people`),
  which T3's launchpad team lines may read.
- **The owner of coding runs:** no change to `temper-engine-domain` or
  `forge_route.rs`.

## Overlaps with drafts beside this one

- **`ui-p1-rail`** (the rail, Work, Deployment): `Address`'s forced arms
  meet its `Work`, `Members` and `Limits` in the same `match`es
  (`pages.rs`, `frame.rs`, the TUI's); the README's pages' table and
  status file. Team's rail entry waits for its rail: see `team-rail`.
- **`ui-p2-home`** (Home and Needs you): `Address::AllHome`'s arms beside
  `Team`'s; `pages/needs.rs` and `pages/home.rs` take Team's arm alone.
- **Every UI plan:** `look.rs`'s style tokens, `words.rs`'s `Word`, the
  gallery's `samples.rs` and `names.txt`, `surfaces.status.md`.
