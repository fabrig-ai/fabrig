+++
id = "builders-tab"
part = "builders-tab"
kind = "feature"
needs = ["flow-paths"]
files = [
  "temper/crates/temper-client-domain/src/address.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/builders.rs",
  "temper/crates/temper-client-domain/src/tests/address.rs",
  "temper/crates/temper-client-domain/src/tests/pages/project_pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/builders.rs",
  "temper/crates/temper-client-view/src/pages.rs",
  "temper/crates/temper-client-view/src/pages/builders.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests.rs",
  "temper/crates/temper-client-view/src/tests/builders.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/builders.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 110
+++

# Builders

## Goal

A fifth tab, at `/projects/<project>/settings/builders`, reads jig's `deployment` document and draws the deployment's hosts, read-only. It's the plan's one new behaviour. jig changes nothing.

Acceptance:

- The tab is `temper/docs/design/client/README.md`, 3.5, *Builders*, its head in *Settings*' table, and "Builders" the fifth of the tabs: a section, "Builders · N"; a line per host, its name, or "Builder CHANNEL" with none; `small` "2 slots · 1 in use" ("1 slot" for one); `.when` "Not connected", "Draining" or "Connected", in that order of asking; then `.facts`, "Each agent’s role names which builders it runs on. Tell Configure temper to change it."
- With no host connected, `.empty`, "No builder is connected.", above any lines left, and "Builders · 0" with no lines.
- With no role in the deployment (`me.deployment_role` is `None`), one line, "Builders are seen by the deployment’s owners.", no section, and no watch. Until `me` and the document are read, or if the read ended, the title "Builders" alone, claiming nothing. A cut document says "Some builder facts were omitted."
- The watch opens on the tab with a role, as `pages/accounts.rs` does for an owner, and closes on leaving it or on losing the role.
- `Builders { project }` round-trips through `address.rs`. Choosing a project on it stays on it. "Settings" is current in the rail. `missing` holds it as it holds the other project tabs.
- A world's story through the real engine: the owner reads the fixture's one worker; a person with no role is told.
- The proof, by `temper/docs/plan/ui-settings-tabs/notes/common.md`, *The proof*: the tab at the three widths; `builders`, `builders-not-owner`, `builders-none`. `settings`' gallery sample and the terminal's `settings` golden show the fifth tab.
- `temper/docs/design/client/README.status.md` and `browser.status.md` carry no line this plan closed. `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken. The tests pass and bite.

## Inputs

- `temper/docs/plan/ui-settings-tabs/notes/common.md`; `temper/docs/plan/ui-settings-tabs/notes/contracts.md`, *`Builders { project }`*.
- `temper/docs/design/client/README.md`, 3.5, *Settings*, and 3.5's table. `temper/docs/design/client/browser.md`, 2.3; 2.5, *Settings' look*; 7.2.
- `temper/crates/temper-client-domain/src/pages/accounts.rs`, `configure`, `leave`, `deployment`. `jig/crates/jig-client-domain/src/contract/deployment.rs`, `Deployment` and `Host`: `name`, `channel`, `slots`, `used`, `draining`, `connected`.
- `temper/tests/client/tests/suite/account_reads.rs`: the fixture's worker on channel 7.

## Owns and may change

Owns `files`. Compiler-forced arms for `Address::Builders { .. }`, which fall where `Address::Repositories { .. }` does, in: `temper-client-domain`'s `departure.rs`, `route.rs`, `pages/connect_model.rs`, `pages/switcher.rs`; `temper-client-view`'s `choice.rs`, `frame.rs`, `pages/connect_model.rs`, `root.rs`, `tests/fixtures.rs`, `tests/rail.rs`, `tests/frame.rs`. In `pages/settings.rs`, `Tab`, `open`'s arrays and heading matches, and `main` and `missing` becoming `pub(super)`: the rest is `how-work-runs`'.

`Tab` gains a variant: its readers are `pages/settings.rs` and the tabs' test, `every_tab_draws_its_head_and_tabs_in_order`, which gains its row.

Pre-approved:

- `limits.rs`: a page of as many hosts as the document's line bound holds must fit the tree's bound. If it doesn't, price the tab there, as its neighbours are, and a view test draws a full document. That's a bound the code derives, not a budget.
- The gallery's `deployment(w)` lists no hosts today: `builders` gets three, connected, draining and not connected, for its world alone, by a small `World` test or a new world variant. If that's more than a few lines, deliver them to the sample by its `Then`, with a new `Then::Hosts`. The other samples' hosts stay empty.
- `builders-not-owner`: a world's `me` with `deployment_role: None`, by the same means.
- A golden the fifth tab moves is regenerated by `SKEIN_CODEGEN_WRITE=1`, as `temper-client-shell-terminal`'s `tests/render.rs` reads it. Its diff is the new link and its numbers.
- If the client world can't remove the deployment role for the second person, make a second person with none: `temper/tests/client/tests/suite/accounts_edges.rs` shows the fixtures.

## Steps

1. **`temper-client-domain`:** `Address::Builders { project }`, its print and parse, as `Address::Repositories`; `pages/builders.rs`, by the contract; `pages.rs` holds a `builders: builders::Builders` and `builders_open`, sets it where `accounts_open` is set, configures it where `pages.accounts.configure` runs, leaves it where `pages.accounts.leave` runs, and gives `pub fn builders`; the project's switch and `project()` arms.
2. **`temper-client-view`:** `pages/builders.rs`:

   ```rust
   //! Settings' Builders tab (temper/docs/design/client/README.md, 3.5, Builders).
   pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32);
   ```

   It opens through `settings::main` and `settings::open(.., Tab::Builders)`, holds `settings::missing`, and draws through `settings::section_head`, `look::SETTINGS_LINES` and `look::SETTINGS_WHEN`. New tokens, the next free, under the marker, from `mock.css`: `SETTINGS_EMPTY`, `.empty`; `SETTINGS_FACTS`, `.facts`. The line's `small` takes `look::SETTINGS_DETAIL`. `root.rs` routes it. New words: `SettingsBuildersTab`, `SettingsBuildersHeading`, `SettingsBuildersLede`, `BuildersCount`, `BuildersSlot`, `BuildersSlots`, `BuilderUnnamed`, `BuilderConnected`, `BuilderDraining`, `BuilderNotConnected`, `BuildersFacts`, `BuildersNone`, `BuildersNoRole`, `BuildersCut`, in the design's words.
3. **Tests:**
   - `temper-client-domain`'s `tests/address.rs`: the path round-trips. `tests/pages/builders.rs`: `the_watch_opens_for_a_role_and_closes_on_leaving`, `no_role_watches_nothing_and_reads_no_role`, `losing_the_role_closes_it`, `an_ended_read_is_unknown`. `tests/pages/project_pages.rs`: choosing a project stays on the tab.
   - `temper-client-view`'s `tests/builders.rs`: `each_host_is_a_line_with_its_slots_and_state`, for connected, draining, not connected, and not connected while draining; `one_slot_says_slot`; `a_host_with_no_name_says_its_channel`; `no_host_connected_says_so_above_any_lines`; `no_role_is_one_line`; `an_unread_document_claims_nothing`; `a_cut_document_says_so`; `the_tab_has_no_act`; `the_page_says_builder`, no "host" or "worker" in its words.
   - `tests/settings.rs`: `every_tab_draws_its_head_and_tabs_in_order` gains Builders.
   - `temper-client-shell-terminal`: a `builders` golden beside `settings`, on the fixture's `jig_client_world::stories::deployment()`.
   - `temper/tests/client/tests/suite/builders.rs`: `an_owner_reads_the_engines_worker_on_builders`, `a_person_with_no_role_is_told_who_sees_them`; `mod builders;` in `suite/main.rs`.
4. **The gallery:** `builders`, `builders-not-owner`, `builders-none`, `names.txt` sorted, each with `Holds`; the `settings` sample's `Holds` if it counts the tabs.
5. **The proof.**
6. **The status:** take out `temper/docs/design/client/README.status.md`'s 3.5, *Builders* line, and `temper/docs/design/client/browser.status.md`'s 2.3 and 7.2 Builders entries and whatever remains of its 2.5 line. If anything this plan's request asks is still open, it stays there, and goes in the report.
7. `cargo fmt`; old forms; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(builders) or test(account) or test(settings)'`; `heavy cargo check -p temper-client-browser-world --tests`; `heavy cargo check -p temper --tests`; `heavy cargo nextest run -p temper-client-journeys`, reprinting if it's stale (`notes/common.md`, *Journeys*), this task taking up any a task before it left; commit and push.

## Report

As the implementer's text says: which builder; each screenshot, by name; each difference and why it stays; what stayed in the status; `limits.rs`' decision.
