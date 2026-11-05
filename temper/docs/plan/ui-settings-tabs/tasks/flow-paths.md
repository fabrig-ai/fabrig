+++
id = "flow-paths"
part = "flow-paths"
kind = "feature"
needs = ["accounts-tab"]
files = [
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/forge/page.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/connect_forge.rs",
  "temper/crates/temper-client-view/src/tests/connect_model.rs",
  "temper/crates/temper-client-view/src/tests/forge_page.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-domain/src/forge/forges.rs",
  "temper/crates/temper-client-domain/src/tests/connect.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/forge.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 100
+++

# The flows' paths, and `Adopt` in the language

## Goal

`ConnectForge`, `ConnectModel` and `Adopt` say where they are in Settings and lead back to its tabs. `Adopt` leaves its card chrome, and "Start over" from `ConnectForge` returns to `Forges`. The welcome draws as it did and the terminal prints as it did.

Acceptance:

- The paths are `temper/docs/design/client/README.md`, 3.5, *The flows' paths*: PROJECT / Settings / Forges / Add a forge; PROJECT / Settings / Accounts / Connect an account; PROJECT / Settings / Repositories / Adopt a repository. PROJECT links to `Home { project }`, Settings to `Settings { project }`, the tab to `Forges`, `Accounts` or `Repositories { project }`; the last part is no link. At 760 px and under, "‹" alone, a link to the tab. "Settings" stays current in the rail, as `frame.rs` already has it.
- With no project there's no path. The terminal draws none: `adopt_*` and every other terminal golden hold unchanged, and `ConnectForge`'s, `ConnectModel`'s and `Adopt`'s numbers with them.
- `ConnectForge` and `ConnectModel` keep the welcome's look as built. `Adopt` takes the `.col`, the `.text` and the `.btn`: `look::PAGEBAR` and `look::CARD` leave `forge/page.rs`, its main is `look::MAIN` and `look::SETTINGS`, and `look::ADOPTION_PAGE` and every rule under `.temper-adoption-page` go.
- "Start over" on a forge cut short, once removed, leaves for `/forges` from `ConnectForge`, and for `/welcome?at=forge` from the welcome. Grep for any other ending of the three flows that ends on no page of its own: one that does returns to its tab, and the report names it. Ready and "Adopted" are pages of their own, and stay.
- The proof, by `temper/docs/plan/ui-settings-tabs/notes/common.md`, *The proof*: each flow at the three widths; every `connect-*`, `forge-admin-details`, `forge-details`, `forge-setting-up-middle`, `forge-setup-failed`, `adoption`, `adoption-filled`, `adoption-refused`.
- `python3 -I -B tools/citations/citations.py --base origin/main` shows none broken. The tests pass and bite.

## Inputs

- `temper/docs/plan/ui-settings-tabs/notes/common.md`; `temper/docs/plan/ui-settings-tabs/notes/contracts.md`, *The language*.
- `temper/docs/design/client/README.md`, 3.5, *Settings*, its *The flows' paths*, and 4.4. `temper/docs/design/client/forge.md`, 5.4, `ConnectForge`'s first bullet. `temper/docs/design/client/browser.md`, 2.5, *Settings' look*.
- `temper/crates/temper-client-view/src/pages/configure.rs`, `top`: the path it takes after, on `look::CRUMBS`, `look::BACK` and `look::TRAIL`.

## Owns and may change

Owns `files`. `Configure`'s path and its `.temper-session` rules stay as they are. `pages/configure.rs` isn't this task's.

Readers that may change, each only to follow a signature or a variant:

- `forge::forges::Act::StartOver`: `temper/crates/temper-client-view/src/tests/connect_forge.rs` (~399), `temper/crates/temper-client-domain/src/tests/connect.rs`, and `rg 'StartOver|start_over\b' temper/crates temper/tests temper/tools` for any since. Forge acts' bindings in `forge/bindings.rs`, if they copy the act.
- `connect_model::main`'s new argument: `root.rs` alone.
- `look::ADOPTION_PAGE`: `root.rs` alone; its name goes from `look.rs`.

Pre-approved:

- If a test in `temper/tests/client/tests/suite/` reads a look token of `Adopt`, it follows. Its names stay.
- If the rules of `.temper-main.temper-page` get in the way of the path on `ConnectModel`, whose main is `look::PAGE`, scope the path's rule to win; `ConnectModel`'s other rules stay.
- If a `Role::Choice` or `Role::Check` on `Adopt` needs a rule to sit in the column, write it under `.temper-settings`, on the mock's values.

## Steps

1. **The path,** in `pages/settings.rs`, beside `open`:

   ```rust
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub(crate) enum FlowTab { Forges, Accounts, Repositories }

   /// A flow's path at its head (README.md, 3.5, The flows' paths): nothing with no project, or in the terminal.
   pub(crate) fn path(
       b: &mut Builder,
       root: &Domain,
       w: &Words,
       bindings: &mut Bindings,
       project: Option<u32>,
       tab: FlowTab,
       here: Word,
   );
   ```

   It draws `Slot (SETTINGS_PATH)` holding `Navigation (CRUMBS, named by Word::ConfigureWhere)`, the `look::BACK` link to the tab, then the `look::TRAIL` slot of PROJECT, Settings and the tab, then `here` as plain text. `look::SETTINGS_PATH` is the next free token, `temper-settings-path`, and its rules, under the marker, take `Configure`'s crumbs' values. New words: `Word::PathAddForge`, "Add a forge"; `Word::PathConnectAccount`, "Connect an account"; `Adopt`'s last part is `Word::Adoption`.
2. **The callers:** `forge/connect.rs` `main` and `pages/connect_model.rs` `main`, right after the main opens; `root.rs` `adoption_main`, inside the column, with `Some(form.project)`. `connect_model::main` takes `&mut frame::Bindings` and `Option<u32>`, as `pages/accounts.rs` `main` does. `ConnectForge` takes its project from `root.rs`' `rail_project`.
3. **`Adopt`:** `root.rs` `adoption_main` opens `look::MAIN`, `look::SETTINGS`, `look::SETTINGS_COL`. `forge/page.rs` `page`: the level-1 heading on `look::SETTINGS_HEADLINE`, no page bar; its slots no card; `Role::TextField` on a new `look::SETTINGS_TEXT`; "Adopt" on `look::SETTINGS_BTN` and `look::SETTINGS_PRIMARY`. The tree's names, roles and order stay, so its goldens hold. `.temper-adoption-page`'s and `:is(.temper-adoption-page ...)`'s rules go where they stand.
4. **"Start over":**

   ```rust
   // temper-client-domain, forge/forges.rs
   #[derive(Clone, Copy, PartialEq, Eq, Debug)]
   pub enum StartOverEnd { Welcome, Forges }
   // Act::StartOver { forge: u16, end: StartOverEnd }; the removal's `start_over: Option<StartOverEnd>`
   ```

   `forge/connect.rs` `unfinished` names it by `root.pages().address()`. `removal_told` leaves for `/welcome?at=forge` or `/forges`.
5. **Tests:**
   - `tests/settings.rs`: `each_flow_draws_its_path_in_the_browser_and_none_in_the_terminal`, for the three, its links' addresses and its last part; `no_project_draws_no_path`.
   - `tests/connect_forge.rs`: `start_over_names_where_it_ends`, at `ConnectForge` and at the welcome.
   - `temper-client-domain`'s `tests/connect.rs`: `a_removed_start_over_leaves_for_its_end`, both ends. Today no test holds the leave: this one does.
   - `tests/forge_page.rs`: `adopt_draws_no_card_chrome_and_takes_the_language`.
6. **The gallery:** `Holds` follow if any reads what moved. Keep every sample.
7. **The proof.**
8. **The status:** take out `temper/docs/design/client/README.status.md`'s 3.5, *The flows' paths* line; `temper/docs/design/client/forge.status.md`'s 5.4, at Choose entry on "Start over"; and from `temper/docs/design/client/browser.status.md`'s 2.5 line, `Adopt`'s and a flow's path, and the old rules, now gone.
9. `cargo fmt`; old forms; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-gallery -p temper-client-shell-terminal -p temper-client-shell-web`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(adoption) or test(connect_forge) or test(connect_model) or test(first_repository) or test(welcome)'`; `heavy cargo check -p temper-client-browser-world --tests`; `heavy cargo check -p temper --tests`; `heavy cargo nextest run -p temper-client-journeys`, reprinting if it's stale (`notes/common.md`, *Journeys*), this task taking up any a task before it left; commit and push.

## Report

As the implementer's text says: which builder; each screenshot, by name; each difference and why it stays; each ending you changed or found none to change; the rules removed from `temper.css`.

## Rebase (2026-11-05, step merge, foreman ui-settings-tabs/flow-paths/foreman-1)

rebase the flow style and word registries onto the integration branch’s allocations
The merge conflicts are in `temper/crates/temper-client-view/src/look.rs` and `temper/crates/temper-client-view/src/words.rs`.
Integration commit `cbdead9c7b` brought main’s Goals and Needs you changes alongside Settings; preserve both sets of changes.

In `look.rs`, keep integration’s existing allocations, including Settings tokens 299–320.
Assign `SETTINGS_PATH` and `SETTINGS_TEXT` tokens 321 and 322; their branch tokens 275 and 276 now identify Goals styles.
Append `temper-settings-path` and `temper-settings-text` to `NAMES.styles` in that order.
Preserve this task’s removal of `ADOPTION_PAGE` and its `temper-reserved-61` placeholder.

In `words.rs`, retain integration’s vocabulary and add both path words throughout the enum, `ALL`, key mapping and English definitions.
Assign `PathAddForge` and `PathConnectAccount` keys 3360 and 3361; 3260 and 3261 now belong to `GoalLandWhole` and `GoalLandWholeLine`.
Combine the counts: `PHRASES = 1485`, `SEGMENTS = 2582`, and `ALL: [Word; 888]`.
Preserve integration’s word limits and hole handling.

Keep the automatically merged flow callers, CSS, tests and status changes.
Run the focused view tests for style names, vocabulary and flow paths, then the spec’s required checks.
Preserve the acceptance requirement that terminal flow numbers and goldens remain unchanged.
