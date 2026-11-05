+++
id = "ways-session"
part = "ways-session"
kind = "feature"
needs = ["ways-kind", "settings-frame"]
files = [
  "temper/crates/temper-client-domain/src/forge.rs",
  "temper/crates/temper-client-domain/src/forge/ways.rs",
  "temper/crates/temper-client-domain/src/forge/me.rs",
  "temper/crates/temper-client-domain/src/forge/family.rs",
  "temper/crates/temper-client-domain/src/route.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/pages/home.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/forge_ways.rs",
  "temper/crates/temper-client-domain/src/tests/pages/home.rs",
  "temper/crates/temper-client-domain/src/tests.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-view/src/tests/fixtures/**",
  "temper/crates/temper-client-shell-terminal/src/tests/fixtures.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/samples.rs",
]
effort = "high"
minutes = 140
+++

# The session's facts: a project's ways and questions; the rail and the tab bar by them

## Goal

The client holds jig's `settings` watch for the session and reads kind 7 of the open project, and kind 6 on the tab. The rail and the tab bar draw from them, and a hidden entry
goes. How work runs' controls are `how-work-runs`', on these facts.

Acceptance:

- **The watch** is session-long, as `dismissed` and `forges` are (`forge/dismissed.rs`, the model: `configure`, `end`, an ended watch stays ended until the next sign-in). Home no longer opens
  its own: `pages/home.rs`' `Home.settings` is `Side::settings_handle()`, and it pushes no `Watch` or `Unwatch` for it. Its facts read the same document.
- **The reads:** `forge/playground.rs`'s `read` and `told` are the model. `Readable::Family`, `scope: c::Scope::Project(project)`, `w::Read::Setting(SettingRead { kind, scope: project, number: 1 })`. Kind 7 as the open project
  changes; kind 6 as the tab opens or the project changes on it; each again when the document's `SettingLine` for that `Setting::Application` shows a new `last` or `follows`, and when the link goes live again. No line (no change, jig
  lists only what was set) is the stock. Only one read of a kind out at a time: a change seen meanwhile re-reads at its answer.
- **Their values:** an answer with no value is `Ways::STOCK`, `Route::None`. A malformed value, or a refused or unserved read, is `None`, as a read not answered yet; the rail then hides, and the tab's controls show none.
- **`route::ways_progress`** calls `forge::ways::progress(side, open, on_tab, limits, out)` beside `route::playground_progress`, in `step` and `fire`: `open` being `pages.project()`, `on_tab` whether the address is `Settings`. Choosing another project drops the last project's values.
- **`forge::Act::Ways` sends:** `ChangeSettings { against, items: [one], running: Running::Stays }`, `against` the document's `newest`, `Setting::Application { kind, scope: project, number: 1 }`, `SettingEdit::Set(SettingSet { value: SettingValue::Application(bytes) })`, the bytes the wire's `ProjectWays` or
  `ProjectQuestions`. While one of a `Control`'s is out, a second of it sends nothing. A `stale` refusal (`Refusal::Stale`) resends once, `against` its `newest`; a second is said as a refusal. Answered changed, the value asked is held as read and read again. `ways_asked`, `ways_refused` as `notes/contracts.md`.
- **`Domain`** gains nothing but what the view needs: `root.forge().ways(project)`.
- **The rail** (`frame.rs`, `entries`), by `temper/docs/design/client/README.md`, 5.3. Read the trunk at your base:
  - If `Address::Goals` is there: "Goals", shown only while `ways.goals`, or on `Address::Goals` or on a goal. Replace any `root.has_goals(project)` test of the rail by this one; `Domain::has_goals` itself stays, others read it. If not there: nothing to draw, and `ui-goals`' request says it reads kind 7 if this has landed.
  - If `Address::Changes` is there: "Changes", shown only while `ways.changes`, or on `Address::Changes`.
  - If an Away switch is drawn (`ui-away`'s): shown only while `ways.away`, or while Away is on, in the rail and as the phone's chip, and it replaces that plan's has-goals test.
- **The tab bar,** browser alone: Home, Needs you, then Goals and Chats while `ways.goals`, Chats and Settings otherwise; and with no `Address::Goals` on the trunk, always Home, Needs you, Chats, Settings. Its Settings keeps the dots. The rail's foot keeps Settings whatever.
- **The switcher's** "Settings for PROJECT" stays: it is how a phone with Goals shown reaches Settings.
- **`pages::chats::drafts_offered`,** by `notes/contracts.md`. If the trunk has a "+ Draft" (or `drafts_offered` by another signature), it reads this; if nothing calls it, it stays public and its test holds it.
- `README.status.md`: what this closes of *3.5, 5.3, 5.13* goes: the watch and reads, the rail and tab bar, `drafts_offered`.
- The tests below pass and bite.

## Inputs

- `temper/docs/plan/ui-settings/notes/common.md`; `temper/docs/plan/ui-settings/notes/contracts.md`, *The session's facts*.
- `temper/docs/design/client/README.md`, 5.3 and 5.13, *What the client holds*, *Its acts*.
- `temper/docs/design/client/forge.md`, section 9, the row of temper's settings.
- The mock's `navItems()`, `rail()`, `tabbar()` in `temper/docs/design/client/greenfield/app.js`.

## Owns and may change

Owns `files`. May also change:

- **The client's limits:** `temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`, `temper-client-shell-terminal/src/limits.rs`: `watches` by one, if the session's count trips; Home's leaves room, so it may not.
- **Fixtures that answer watches and reads** (`temper-client-view/src/tests/fixtures*`, `temper-client-shell-terminal/src/tests/fixtures.rs`, `temper/tools/gallery/src/worlds.rs`, `temper-client-domain/src/tests/fixtures.rs`, `temper/tests/client/**`, `temper/tools/journeys/**`): a world must answer the new family read, and its `settings` watch must be asked of the session. A default that lets existing tests stand: with no answer given, kind 7 reads as all shown only where a test's rail needed Goals or Changes, or as the stock. Goldens and prints regenerate.
- **A gallery sample whose rail shows Goals or Changes** (`home-with-goals`, `goals-*`, `changes*`, if on your trunk): its world gives kind 7 shown; say which in the report.
- **Design status:** `temper/docs/design/client/README.status.md`.

Pre-approved:

- If `Told::Answered` reaches `forge::told`'s family-only `answer` first: route a key of this module's to `forge::ways::told` beforehand, as `playground::told` comes first in `route.rs`.
- A change of meaning: Home's watch and the rail's entries change, so run `temper-client-world`'s and `temper-client-gallery`'s tests too, and `heavy cargo check -p temper-client-browser-world --tests`.
- If kind 6 or 7 read is `Why::Unknown` to a project member, the fault is `ways-kind`'s read: say so as a `Question:` with the failing case; it is not worked around here.

## Steps

1. `forge/ways.rs`: types, `configure`, the reads, `told`, `progress`, the act; `forge.rs`: the fields on `Side`, its handle's number in `me.rs` (`SETTINGS`), `Act::Ways`.
2. `route.rs`, `domain.rs`: `ways_progress`. `pages/home.rs` onto the session's watch; `tests/pages/home.rs` follow.
3. `frame.rs`: a single function `shown(root, project) -> Shown` holding the rules; `entries` and `tabbar` read it. `pages/chats.rs`.
4. Tests, domain, new `tests/forge_ways.rs` (`mod` in `tests.rs`):
   - `the_watch_is_held_for_the_session_and_home_reads_it`;
   - `kind_seven_is_read_as_a_project_opens_and_dropped_as_it_closes`; `kind_six_is_read_on_the_tab_alone`;
   - `no_value_is_the_stock`; `a_refused_read_hides_and_a_malformed_value_too`;
   - `a_changed_line_reads_again`; `one_read_at_a_time`; `the_link_back_reads_again`;
   - `a_set_sends_one_change_of_one_item_against_the_newest`; `a_second_set_of_a_control_sends_nothing_while_one_is_out`; `a_stale_refusal_is_sent_once_more_against_its_newest`; `a_second_stale_is_said`; `a_refusal_is_kept_until_the_next_act`; `an_answer_holds_the_value_asked`.
5. Tests, view, `tests/rail.rs`: `the_rail_shows_what_kind_seven_shows`, table of each value, on and off; `until_read_the_optional_entries_are_hidden`; `a_page_keeps_its_entry_while_on_it`; `the_tab_bar_by_goals`, `a_hidden_page_keeps_its_address`.
   `drafts_offered_reads_drafts`.
6. `cargo fmt`; old forms; clippy and nextest of `temper-client-domain`, `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`, `temper-client-world`, `temper-client-tools`; `heavy cargo check -p temper --tests`. Commit and push.

## Report

As the implementer's text says, with each "if on the trunk" as you found it, and the fixtures' default for kind 7.

## Rebase (2026-11-05, step merge, foreman ui-settings/ways-session/foreman-1)

rebase by combining Changes support with session settings, then regenerate story A’s recording.
The merge refused four content conflicts; this is the branch’s red.
Upstream Changes work comes from `d0fe151eed` (domain) and `5e9c187b7b` (view), incorporated through `96752e47c7`.

- `temper/crates/temper-client-domain/src/pages.rs`: retain Changes state, dispatch and teardown. Keep this branch’s `Home::new()` and `pages.home = None`; do not restore Home’s separate settings watch or handle allocation.
- `temper/crates/temper-client-view/src/frame.rs`: retain Changes navigation and exhaustive address handling. Extend this branch’s single `shown(root, project)` calculation so Changes appears when kind 7 enables `ways.changes` or the current address is Changes. Keep Changes out of the phone tab bar and keep Settings accessible.
- `temper/crates/temper-client-shell-terminal/src/tests/fixtures.rs`: retain both upstream’s family Changes-watch response and this branch’s family Setting-read response in `populate_page`; combine their scoped lint expectation.
- `temper/tests/playground/tests/fixtures/story-a.log`: regenerate from the combined code; do not splice recorded hashes. Upstream `68bad859e7` and branch `5458294a79` both refreshed this recording. Preserve the branch’s startup-settling changes.

Update `temper-client-view/src/tests/rail.rs`: its assertion that Changes never exists is now obsolete. Cover enabled, disabled, unread and refused kind 7, plus keeping Changes visible while on its page and retaining a hidden page’s address.
Give Changes gallery samples explicit kind 7 values where their rails require Changes; regenerate affected goldens and prints.

Regenerate story A with `SKEIN_CODEGEN_WRITE=1` through heavy, selecting `story_a_walks_from_the_welcome_to_the_csv_change_landed` in `temper-playground-world`’s `suite` binary; then verify `story_a_kept_log_replays_natively`.
Run the affected domain, view, terminal, gallery and client-world checks, plus the spec’s browser-world check, through heavy with `TEMPER_BUNDLE=none`.
