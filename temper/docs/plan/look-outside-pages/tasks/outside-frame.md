+++
id = "outside-frame"
part = "outside-frame"
kind = "feature"
needs = ["outside-looks"]
files = ["temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/outside.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/pages/chats.rs", "temper/crates/temper-client-view/src/pages/new_chat.rs", "temper/crates/temper-client-view/src/tests/frame.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-view/src/tests/person_menu.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/crates/temper-client-view/src/tests/chats.rs", "temper/crates/temper-client-view/src/tests/new_chat.rs", "temper/crates/temper-client-view/src/tests/fixtures.rs", "temper/crates/temper-client-shell-terminal/src/tests/render.rs", "temper/tools/gallery/src/tests.rs", "temper/tools/gallery/src/samples.rs", "temper/tools/gallery/src/worlds.rs", "temper/tools/gallery/names.txt", "temper/docs/design/client/journeys/welcome-set-up/**", "temper/docs/design/client/journeys/welcome-playground/**"]
effort = "high"
minutes = 180
+++

# look-outside-pages: the top bar outside a project, and no project yet in the column

## Goal

Wherever the rail isn't, the frame becomes the top bar: temper's mark, a link to `/`, and,
signed in, the person, "Sign out", "Offline" and "Behind" at its right; nothing else (the
request's `outside-pages/01.html` to `08.html`). The banner, its "Menu", its switcher,
navigation and corner are drawn nowhere. A person signed in with no project sees, at every
address but the sign-in, approval, authorization and unavailable places and `Welcome`,
the column "No project yet" with the form "New project" (`08.html`), or with the welcome
its "Set up temper" and "Try the playground". The welcome's frame is the top bar too.

Acceptance:

- `root::build` draws `outside::top` (contracts, 4) wherever it drew `frame::banner`, in
  both shells: the tree is contracts, 4's, the link region included in every state, and the
  browser rule of contracts, 4 under the outside marker in `temper.css`. `frame::banner`,
  `frame::navigation`, `Place::Banner` (and `Place`, if the rail is all that is left) and
  what only they drew are gone; `BANNER`, `MENU` and `CORNER` are drawn nowhere (their
  tokens leave in a later task).
- The brand link is named and worded "temper" and goes to `Address::Inbox { filter:
  Filter::ALL }`, printed `/`, on `Welcome` too.
- On `Welcome`, the top bar holds temper box's line and the welcome's steps as the banner
  did, no `TOP_END`; the welcome's rules in `temper.css` that select the welcome's banner
  (`#app:has(> .temper-welcome) > .temper-banner` and its phone rule) select
  `.temper-top` instead, where they stand, so the welcome draws as it does today.
- With `me` read and naming no project (`root.pages().project()` is `None` once `me` is
  read, `pages.rs`'s `project_changed`), every address but the sign-in page, the
  `Authorize` and approval places, the pages not about the work and `Welcome` draws the
  column, and no accounts' banner (the playground's line while it is leaving stays):
  - no welcome (`root.forge().deployment()` read, `welcome` false): `Heading` "No project
    yet", `NoProjectLine`, then `Form` "New project" { `TextField` "Project name" with the
    switcher's value, enter and pending state; `Button` "Create" [`OUTSIDE_PRIMARY`];
    the switcher's refusal `Alert` }, sending the switcher's acts as the rail's form does;
  - with the welcome: "No project yet", `NoProjectSetUp`, then "Set up temper"
    [`OUTSIDE_PRIMARY`] to the welcome as the welcome's own "Set up temper" goes and,
    while the playground is offered and none is read, "Try the playground"
    [`OUTSIDE_SECONDARY`] with the welcome's playground act, both in `Slot
    [OUTSIDE_ACTS]`, or the first alone;
  - the deployment not yet read: the address's page as today.
  Chats' and `NewChat`'s own no-project bodies go.
- Words: `NoProjectLine`, `ProjectName`, `NoProjectSetUp` as contracts, 5.
- A gallery state `no-project` (`temper/docs/design/client/browser.md`, 7.2).
- The terminal's first line keeps the link's state outside a project
  (`no_project_reads_the_same_in_the_terminal` prints its new page).
- The journeys `welcome-set-up` and `welcome-playground` are reprinted (`heavy cargo run
  -p temper-client-journeys`, no arguments, reprints the saved recordings; if one reports
  its recording stale, `again NAME` records it, common.md's retries holding), and `heavy
  cargo nextest run -p temper-client-journeys` passes.
- `heavy cargo nextest run` passes for `-p temper-client-view`, `-p
  temper-client-gallery`, `-p temper-client-shell-terminal`, `-p temper-client-shell-web`,
  `-p temper-client-world`, `-p temper-client-journeys`; `heavy cargo check --tests`
  compiles `-p temper-client-browser-world`, `-p temper-one-process-world`, `-p
  temper-client-tui`, `-p temper-client-shell-tui` and `-p temper`.

## Inputs

- `temper/docs/design/client/README.md`, 5.1 (*the top bar*, *the main*), 3.5 (*With no
  project yet*; The welcome, *Its frame*), 5.3 (*No navigation outside a project*, *No
  switcher outside a project*), 6.4 and 6.6.
- `temper/docs/design/client/browser.md`, 2.5, *Outside a project* and *No frame but the
  top bar outside a project*; 7.2, `no-project`.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1, 3, 4 and 5;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/03.html` (the top bar signed in) and `08.html`.

## Owns and may change

- Owns: `frame.rs` (the banner's code goes; the rail stays as it is), `outside.rs` (adds
  `top`), `root.rs` (the frame's call and the no-project dispatch), `pages/chats.rs` and
  `pages/new_chat.rs` (their no-project bodies go), the three words, the rule and the
  welcome's selectors in `temper.css`, the gallery's `no-project` state (a world with no
  project may be added to `worlds.rs`), the two journeys' reprinted pages.
- May change, assertions and helpers, keeping what each checks for everything else (the
  survey's list; each one reads the banner, its menu, switcher, navigation or corner):
  - `tests/frame.rs`: `the_navigation_is_named_main_navigation`,
    `the_navigation_ends_with_accounts`, `the_banner_says_the_links_state_and_the_persons_name`,
    `a_navigation_without_projects_says_no_project_yet`,
    `the_banner_says_connecting_until_me_then_live_then_offline`,
    `signed_out_the_banner_says_the_links_state_alone`,
    `out_of_date_the_banner_says_the_links_state_alone`, `each_link_shows_its_name_as_its_text`,
    `the_navigation_is_inside_the_banner_after_projects_before_the_link`,
    `the_signed_in_corner_groups_link_person_and_sign_out_in_order`,
    `the_unavailable_corner_holds_only_the_link_region`,
    `navigation_shows_the_inbox_count_only_when_nonzero`,
    `the_switcher_is_named_projects_before_a_project_is_open`,
    `the_banner_menu_keeps_navigation_and_corner_while_connecting`,
    `the_unavailable_banner_menu_holds_its_corner_alone`,
    `the_brand_links_to_chats_with_its_accessible_label`,
    `renaming_replaces_the_open_project_row_in_the_banner`,
    `opening_rename_takes_focus_from_a_shown_document_section`, and the helpers
    `assert_corner`, `assert_switcher`, `assert_menu`. A test of a state the change
    removes (the navigation outside a project, the switcher before a project is open, the
    banner's rename) is deleted, or ported to the rail where the rail has no test of that
    case; a test of what stays (the link's state, the person, Sign out, the lost sign-out)
    reads the top bar. Fold what you keep into few tests; name each deleted or ported one.
  - `tests/look.rs`, `the_frame_carries_its_tokens` (`Role::Banner` → `[TOP]`);
    `tests/person_menu.rs`, `a_project_less_accounts_page_keeps_the_banners_sign_out`
    (now the no-project column under the top bar: Sign out in `TOP_END`);
    `tests/welcome.rs`, `the_welcome_frame_is_the_brand_the_box_line_and_the_steps` and
    `an_unavailable_welcome_keeps_the_signed_out_and_out_of_date_frame`;
    `tests/chats.rs`, `no_welcome_and_no_project_says_so` and
    `welcome_or_unknown_deployments_keep_the_chats_page` (the welcome's no-project
    column); `tests/new_chat.rs`, `path_links_lead_through_chats_with_or_without_a_project`
    (its no-project half becomes the column); `tests/fixtures.rs`'s
    `navigation_link_names` and `frame_statuses`.
  - `temper/tools/gallery/src/tests.rs`: `print_writes_every_page_and_the_bundle`'s
    frame branch: a page with neither rail nor terminal holds `class="temper-top"` and no
    `temper-menu`; Welcome's branch likewise; the `no-project` row of
    `each_state_holds_what_its_name_says`.
  - `temper/crates/temper-client-shell-terminal/src/tests/render.rs`,
    `no_project_reads_the_same_in_the_terminal`: its new print.
  - Slow tiers, compiled not run: `temper/tests/client-browser/**`,
    `temper/tests/one-process/**`, `temper/crates/temper/tests/**`,
    `temper/tests/real-loop/**`, where one reads the banner, its menu or its navigation;
    the region kept means their link-state helpers hold.
- Read, never change: `sign_in.rs`, `approve.rs`, `public.rs` (other tasks'); the
  rail's code and tests (`tests/rail.rs` holds as it is: run it);
  `temper-client-tui` and `temper-client-shell-tui` (their own launchpads).
- Options, pre-approved:
  - A word only the banner said (`BrandChats`, `Menu`, `Projects`' switcher name) is left
    in the phrasebook unless a test flags it unused; then remove it and say so.
  - If a client story (`temper/tests/client`) walks the banner's navigation or switcher
    with no project, make it use the column's form or the address directly, keeping what
    it checks, and name it.
  - If `project_landings_start_is_welcome_set_ups_recorded_acts` or the journeys' replay
    flags an act the top bar no longer offers, record the journey again (`again NAME`)
    and say so.

## Steps

1. `outside.rs`: `top(b, root, words, bindings)` (contracts, 4), the welcome's box line and
   steps moved from `frame::banner` (or called from it).
2. `root.rs`: `crate::outside::top` in place of `frame::banner`; the no-project dispatch
   before the address's page, after the unavailable branch; no accounts' banner then.
3. `frame.rs`: remove `banner`, `navigation`, the banner's place of the switcher and what
   only they used; `pages/chats.rs` and `pages/new_chat.rs`: remove their no-project
   bodies.
4. `outside.rs`: the no-project column (contracts, 3), the form reusing the switcher's
   acts and refusal (`frame.rs`'s `new_project_form` may take the field's name, or the
   column draws its own form on the same bindings).
5. `temper.css`: the rule of contracts, 4 under `/* ==== ui: outside ==== */`; the
   welcome's banner selectors to `.temper-top`.
6. The gallery's `no-project` state; the tests above; the terminal print. Bite: draw
   `Disclosure` "Menu" again and see the frame test fail; drop the form and see the
   no-project test fail.
7. Reprint the journeys; `heavy cargo nextest run -p temper-client-journeys`.
8. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view -p temper-client-gallery -p temper-client-shell-terminal
   --all-targets -- -D warnings`; the focused runs and checks of Acceptance; commit with
   the trailer (common.md), push.

## Report

As the implementer's text says, naming each test deleted, ported or folded. `## Account`
rows: the top bar (01 to 08: mark, person, Sign out; Offline, Behind; no menu, switcher,
navigation, Signed out, Connecting, Live), the top bar while `me` is read, the welcome's
top bar, 08.html (No project yet, the form), no project with the welcome, the terminal's
first line.
