+++
id = "team-rail"
part = "rail"
kind = "feature"
needs = ["team-reads", "team-page", "team-prints", "team-joint"]
files = [
  "temper/crates/temper-client-domain/src/pages/rail.rs",
  "temper/crates/temper-client-domain/src/pages/team.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/rail.rs",
  "temper/crates/temper-client-domain/src/tests/pages/team.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/tests/client/tests/suite/team.rs",
  "temper/docs/design/client/README.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 150
+++

# Team in the rail: ui-p1-rail's Team row, given its count

## Goal

Meet `ui-p1-rail` at Team's row in the rail: its `rail::parts` already
shows "Team" while the project has two people or more, but `rail::facts`
gathers no members, so Team never shows. Read the project's document on
every project page, give `rail::facts` Team's count of people, and draw
"Team" in the rail and "Team for PROJECT" in the switcher's menu, to
`Team { project }`, current there.

**Before anything, check the base.** This task builds on ui-p1-rail's
rail, which `needs` can't name, since it is another plan's: on your base,
`temper/crates/temper-client-domain/src/pages/rail.rs` must hold
`rail::facts` and `rail::parts`, and `temper-client-view/src/frame.rs`
must draw the rail from `rail::parts` (`RAIL_NAV` holding "Work" and
"How work runs"). If either is missing, ui-p1-rail hasn't landed or the
trunk hasn't taken main since: change nothing, push nothing, and report
`Failed: ui-p1-rail's rail is not on the base; team-rail runs once it has
landed and the trunk has taken main.`

Acceptance, by `temper/docs/design/client/README.md`, 3.5, *Team*, *In
the rail*, and 5.3 (*The rail*, *The rail's parts*, *Projects in the
rail*), and `notes/contracts.md`, T3 and T5:

1. **The document on every project page:** Team's read of the project's
   document (`team-reads`) is made on every page with a project
   (`address_project` gives one), not on `Team` alone, kept while the
   project stays open and read again on a `me` delivery as before; the
   others' `Tasks` watches stay Team's alone. `people(root, project)` so
   answers on any project page once read.
2. **The rail's count:** `rail::facts` gives `members:
   team::people(root, project)`; `standing_roles` stays `None`. The
   rail's facts test (`tests/pages/rail.rs`) gains a row: a project whose
   document lists two people gives `members: Some(2)`, one with one
   person and a service `Some(1)`, an unread one `None`.
3. **The rail's entry** (`frame.rs`): "Team", after "Work" and before
   "Ask temper", while `parts.team`, a link to `Team { project }`,
   current on `Team`; the tab bar unchanged. The switcher's menu: "Team
   for PROJECT", to `Team { project }`, before "How work runs for
   PROJECT", while `parts.team`. Both shells, the terminal as numbered
   lines as the rail's other entries print. Words in `words.rs` from
   4900 to 4949 (`notes/common.md`), beside Team's.
4. **Rows, no new test function:** `tests/rail.rs`'s current-entry table
   (Team current on `Team`), its entries' table (Team drawn with two
   people, not with one); `tests/frame.rs`'s switcher's menu (its "Team
   for PROJECT" row); the gallery's `team` row in `tests.rs` gains the
   rail's links, "Team" among them, and `team-alone`'s shows none; the
   joint story `suite/team.rs` asserts the rail's "Team" for both people.
5. **What it makes stale is yours** (the request's rules): a new read on
   every project page changes the requests of journeys and stories.
   Regenerate the journeys a recording of which is stale with
   `temper-client-journeys again NAME` (the request's retries hold),
   reprint them (`heavy cargo run -p temper-client-journeys`), and repair
   a client world story or a terminal golden whose correct behaviour the
   read trips, keeping what it checks. Name each in the report.
6. **Design:** in `temper/docs/design/client/README.md`, 5.3, *The rail's
   parts*, the table's "Team" row's fact says the project's people, from
   its document (3.5, *Team*), and standing roles, none gathered; no other
   line of 5.3. In `README.status.md`, delete, whole, this plan's **Not
   yet** entry "3.5, *Team*, *In the rail*, and 5.3: ...", and narrow
   ui-p1-rail's entry "5.3, *The rail's parts*: Team is never shown. ..."
   to the standing roles alone: no read gives the rail a standing role.
   A `docs/help/` page that lists the rail's entries names "Team" too,
   within its bound.

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T3 and T5.
- `temper/docs/design/client/README.md`, 3.5, *Team*, *In the rail*; 5.3,
  as ui-p1-rail landed it.
- ui-p1-rail's own plan, for what its rail leaves Team:
  `temper/docs/plan/ui-p1-rail/notes/contracts.md`, C3, and
  `tasks/rail.md` (on main once landed, else on
  `origin/ui-p1-rail/integration`).
- The code: `pages/rail.rs` (`facts`, `parts`), `pages/team.rs`,
  `pages.rs` (`address_project`, Team's configure), `frame.rs` (the rail's
  entries, the switcher's menu), and their tests.

## Owns and may change

Owns the files in the header, ui-p1-rail's files among them once it has
landed: by then they are main's. Reader files: the journeys' recordings
and prints (`temper/docs/design/client/journeys/**`), the terminal shell's
goldens, the client world's stories and the TUI's tests where the read
changes what they see, each repaired as acceptance 5 says; `docs/help/`
as acceptance 6 says.

Options, each decided:

- **The rail's draw takes no root** where Team's link needs one: pass
  the address `Team { project }` as the rail's other entries pass theirs.
- **A journey still stale after three retries:** widen the mark's wait in
  the recording driver, as the request says, and say so.

## Steps

1. Check the base, as the Goal says. Read the Inputs.
2. The read on every project page (1) and its row in
   `tests/pages/team.rs`; the rail's count (2); bite: give `members` the
   members of any kind, see the person-and-service row fail, restore.
3. The entry and the switcher's row (3), their rows (4); bite: show Team
   at one person, see the entries' row fail, restore.
4. Stale journeys, goldens and stories (5); the design and status lines
   (6).
5. Runs through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-domain`, `-p temper-client-view`, `-p
   temper-client-shell-web`, `-p temper-client-shell-terminal`, `-p
   temper-client-gallery`, `-p temper-client-world`, `-p
   temper-client-journeys`; `cargo check -p temper-client-tui --tests`;
   clippy on every package changed; then as `notes/common.md` says before
   the push. Commit, saying that Team shows in the rail once the project
   has two people, and push.

## Report

The implementer's report: the base's check, where the document is now
read, the rail's count, the entry and the switcher's row in both shells,
the rows and bites with their failures, each journey, golden and story
repaired, the runs' counts, and what is left.

## Answers

2026-11-08. *Run team-rail-2 found no `pages/rail.rs` on its base (ui-p1-rail not
landed): what now?* The stop was right; nothing changes. ui-p1-rail is not on main
yet (`origin/main` 30f6c3e70d has no `pages/rail.rs`). Don't build on
`origin/ui-p1-rail/integration` or copy its rail: that is another plan's
unlanded work. Rerun only once ui-p1-rail has landed and this trunk has taken
main; then do the base check again first, as the Goal says, and go on if it
passes.

2026-11-08. *Run team-rail-3 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 19:36Z `origin/main`
(2220ad96a1) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Wait
for it to land and for this trunk to take main. Then do the base check first and
go on only if it passes; until then each run ends with the same `Failed:` line,
and that is not a question.

2026-11-08. *Run team-rail-4 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 19:47Z `origin/main`
(ba10e9e72a) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-5 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:05Z `origin/main`
(bde6d68af9) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-6 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:17Z `origin/main`
(99c196376d) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-7 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:26Z `origin/main`
(4bc9d78ace) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-8 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:35Z `origin/main`
(4bc9d78ace) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-9 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:43Z `origin/main`
(4bc9d78ace) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-10 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 20:50Z `origin/main`
(4bc9d78ace) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-11 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 21:00Z `origin/main`
(cb577eef0c) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-12 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 21:09Z `origin/main`
(cb577eef0c) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-13 again found no `pages/rail.rs` on its base: what
now?* Same answer: the stop was right; change nothing. At 21:17Z `origin/main`
(e62fc406e0) still has no `pages/rail.rs`, so ui-p1-rail hasn't landed. Don't
rerun until it lands and this trunk takes main; then do the base check first.

2026-11-08. *Run team-rail-14 again found no `pages/rail.rs` on its base: what
now?* The stop was right for its base, but ui-p1-rail has now landed: at 21:25Z
`origin/main` (0d6f7a9b65) holds `pages/rail.rs` with `facts` and `parts`, and
`frame.rs` draws the rail from `rail::parts`. This trunk (dd0550b3d2) hasn't
taken that main yet. Rerun once it has; then the base check passes and you go
on. In the check, `RAIL_NAV` is the style of the slot that holds the rail's
`entries` (`frame.rs`, after the switcher), where "Work" and "How work runs"
are drawn: that is what "holding" means; Team's entry goes in `entries`.
