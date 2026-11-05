+++
id = "away-client"
part = "away-client"
kind = "feature"
needs = ["client-base", "away-wire", "client-zone"]
files = [
  "temper/crates/temper-client-domain/src/forge/away.rs",
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/away.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/away.rs",
  "temper/crates/temper-client-domain/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# The client watches the switch and sends it

## Goal

The client's forge side holds the party's `Away` rows, and an act moves one: `temper/docs/design/client/README.md`, 3.5 (*Away*,
but its drawing), 4.6. No view yet.

Acceptance:

- `notes/contracts.md`, *The client*: `forge/away.rs`, `Domain`'s three, `pages/away.rs`, `Toasted`'s first three.
- **The watch** opens once for a session, as `dismissed::configure` does, and a watch refused or ended stays so: `away` none, and
  `Toggle` does nothing.
- **`away_shown`:** a goal in the project (`has_goals`), the row on at the wall, or `forge().ways(project)`'s `away`.
- **`next_eight`,** pure: at 07:59 local today's 08:00; at 08:00 and after, tomorrow's; at zones +120, -300, +345, and across
  the epoch's day.
- **`Toggle`:** to off, `on` false, the row's `until` and `kinds`; to on, the row's `kinds` or `DEFAULT_KINDS`; `until` none if a row
  holds none, otherwise `next_eight(wall, zone_minutes)`. A second act while one is out does nothing.
- **`Kinds`, `Until`:** `on` as `away_on` reads, the other field kept; `Until { eight: true }` sends `next_eight`, `false` none. With
  no row: `DEFAULT_KINDS`, `next_eight`.
- **`Limits`:** goes to `Address::Settings { project }` and sets a one-shot request (`Pages::away_focus(&mut self) -> bool`) for the
  section's focus, which `settings-away` reads.
- **The toast,** on the answer, in Needs' slot: on, `AwayOn { project, until }`; off, `AwayOff`; refused or unknown, `AwayRefused`.
- `README.status.md`'s *Away* stays *Not yet* for what's undrawn.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 3.5 (*Away*), 4.6, 5.12.
- The models: `forge/dismissed.rs` (`configure`, `told`, `handle`), `route.rs` where it calls them; `pages/home.rs`' `skip` for a request
  and its refusal; `pages/needs.rs`' `toast`; `pages/goals.rs`' `HasGoals`, for `pages.rs`' facts.

## Owns and may change

Owns, as `files` lists them. May also change: compiler-forced arms for `Toasted` in `temper-client-view/src/frame.rs`, drawing nothing
new (`away-switch` draws them); `Act`'s arms in `temper-client-view` where it matches them whole; `forge::me`'s handle numbers for
`away::handle` (`me.rs`' `DISMISSED` is the model); `temper-client-view`'s and the gallery's fixtures for a new `Side` field.

Options, each pre-approved:

- **`Side` or `Pages`:** rows on `Side`, acts on `Pages`, as `dismissed` and `skip`. The pending key on `Pages`.
- **A watch's handle** beside `me::DISMISSED`, the next free number.

## Steps

1. `forge/away.rs`, `Side`'s fields, `route.rs`' calls. Tests in `src/tests/away.rs`, shaped as `src/tests/dismissed.rs`:
   `the_watch_opens_once_for_a_party`, `a_refused_watch_stays_refused`, `rows_are_delivered_by_project`,
   `on_at_reads_off_and_until`, `ended_is_off_or_until_once_past`.
2. `next_eight`: `next_eight_is_todays_before_eight_and_tomorrows_after`, `next_eight_reads_the_zone`.
3. `pages/away.rs`, its `Act`, its answers and toasts: `toggle_on_sends_defaults_and_the_next_eight`,
   `toggle_on_keeps_a_row_whose_until_is_none`, `toggle_off_keeps_the_row`, `a_second_toggle_waits`, `kinds_keep_on_as_it_reads`,
   `until_sends_the_choice`, `past_until_toggle_turns_on`, `the_toast_follows_the_answer`, `limits_focuses_once`,
   `shown_by_goal_by_on_or_by_ways`.
4. The check of the implementer's text, `temper-client-view` and `temper-client-gallery` as dependents. Push.

## Report

As `notes/common.md`, Your report.
