+++
id = "home-goals"
part = "home-goals"
kind = "feature"
needs = ["home-words"]
files = [
  "temper/crates/temper-client-view/src/pages/home.rs",
  "temper/crates/temper-client-view/src/tests/home.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goals.rs",
  "temper/tools/gallery/src/samples.rs",
]
effort = "high"
minutes = 60
+++

# Goals on Home, in Chats' place

## Goal

With a goal in the project, Home's section is Goals, not Chats: `temper/docs/design/client/README.md`, 3.5, *Goals* under *Home on any project*.

Acceptance:

- A region named "Goals", its head the level-2 heading "Goals" and "All N", to `Address::Goals { project, filter: GoalsFilter::All }`, N `goals.count(client)`.
- Its rows: `goals::row`, with its state and report as the Goals page finds them; live first, then ended, `max(5, live)` rows.
- Chats and its sentence aren't drawn then. While To begin shows, the section is as now: To begin's rule.
- **Its reads are the Goals page's:** `pages.goals` opens on `Address::Home` once `has_goals` holds, and closes on leaving, the same `goals::Goals`, filtered `All`. A project with no goal opens nothing.
- `home-with-goals` in the gallery shows it.
- `client/README.status.md`'s *Not yet* loses Goals.

## Inputs

- `temper/docs/plan/ui-home-status/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5.
- The mock: `home` in `greenfield/app.js`, `shown`.
- `temper/crates/temper-client-view/src/pages/goals.rs`, `main` and `row`; `temper/crates/temper-client-domain/src/pages.rs` where `pages.goals` is opened, told, configured and left (`:449`, `:483`, `:615`, `:813`).

## Owns and may change

Owns, as `files` lists them, and the terminal's goldens by regenerating.

Options, each pre-approved:

- **`goals::row` stays `pub(crate)`,** which is public enough; a helper the page and Home both need for a row's state moves into `pages/goals.rs`, its page unchanged.
- **Opening `goals`** may go in `pages::configure` beside `has_goals.configure`, or where you find it simplest; the page's lifetime is what's bound.
- **Rows' words:** the mock's, through `row`, unchanged.

## Steps

1. **`pages.rs`,** then a domain test: `goals_opens_on_home_once_the_project_has_one`, `home_opens_no_goals_with_none`, `leaving_home_leaves_its_goals`.
2. **`pages/home.rs`:** a `goals` section; `main` picks it.
3. **Tests,** `tests/home.rs`: `goals_takes_chats_place`, `goals_rows_are_live_first_and_at_least_five`, with more live than five, `all_n_links_to_goals`, `to_begin_keeps_chats`.
4. **Check,** the implementer's text, with `temper-client-shell-terminal` and `temper-client-gallery`, then push.

## Report

As `notes/common.md`, Your report.

## Answers

2026-11-05 (fabrig/ui). Ruling (rulings.md, 9): Home's Goals section shows when the Goals setting is on or the project has a goal.
