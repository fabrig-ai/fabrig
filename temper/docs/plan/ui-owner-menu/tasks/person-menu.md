+++
id = "person-menu"
part = "person-menu"
kind = "feature"
needs = ["menu-outside"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/tests/person_menu.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client-browser/src/lib.rs",
  "temper/tests/client-browser/tests/browser_all/browser_sign_in.rs",
  "temper/tests/client-browser/tests/browser_all/browser_public.rs",
  "temper/tests/client-browser/tests/browser_all/browser_attacks.rs",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/names.txt",
  "temper/docs/design/client/journeys/**",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# The rail's foot row opens a menu with Sign out

## Goal

Sign out is too buried at the foot of the Accounts tab. In a browser, the rail's foot row, the person's name, becomes the button of a small menu above it, with "Accounts" and "Sign out". Over 760 px Sign out leaves the Accounts tab; at a phone's width, with no rail's foot, it stays there. The terminal doesn't change.

Acceptance:

- **In a browser, on a page with the rail:** the foot holds a `Role::Menu` named by the person's name, inside a `Role::Slot` styled `look::RAIL_PERSON`, and no `Role::Status` with that name. Its items, in order: `Role::MenuItem` "Accounts", giving `Act::Go { address: Address::Accounts }`, and `Role::MenuItem` "Sign out", giving `Act::SignOut`.
- **The failure line:** after a lost sign-out, "Sign out did not reach the engine." stands on the foot, after the menu, as today, and the menu still offers "Sign out".
- **The Accounts tab:** its Sign out form stays in the tree, with its failure line, and carries `look::SETTINGS_SIGN_OUT`. `temper.css` hides it over 760 px and draws it as today at 760 px and under.
- **A project-less Accounts page:** nothing changes. It has no rail, and its Sign out is the banner's, as `frame.rs`' `banner` draws it. The request calls that "keeps it there".
- **The terminal:** every print, command and act as today. No golden under `temper-client-shell-terminal` changes.
- **A click isn't lost:** the menu's "Sign out" keeps its element while accounts arrive, as the form's does.
- **In real Chromium, at the tier's 1280 px:** the row opens the menu; Escape closes it, the focus on the row; a press outside closes it, the focus on the row; "Accounts" goes to the Accounts tab, which shows no Sign out button; "Sign out" there signs out, a reload stays out. `signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` keeps every assertion it has.
- **The look:** the proof below, at 1440, 1000 and 420, open and closed, beside the mock.
- Each new test was red first, and the report says what it showed.
- The **Not yet** entries this task closes are gone: `temper/docs/design/client/README.status.md` (3.5, 5.3) and `temper/docs/design/client/browser.status.md` (2.5, 7.2). The **Design departs** entry for `ui-owner-menu` stays.

## Inputs

- `temper/docs/design/client/README.md`, 5.3 (the foot and the person's menu; the switcher's menu for its look) and 3.5 (*Sign out*).
- `temper/docs/design/client/browser.md`, 2.5.
- `skein/docs/design/ui-web.md`, 7 (*Menus and disclosures are the page part's to fold*).
- `temper/docs/plan/ui-owner-menu/notes/common.md`: all of it. Its catch-up comes first.
- `temper/docs/design/client/greenfield/` (`mock.css`'s `.rail-foot`, `app.js`' rail): the row closed is the mock's row.

## Owns and may change

Owns, as `files` lists them, with no new `Word`: the menu's name is the person's, "Accounts" is `Word::SettingsAccountsTab` and "Sign out" is `Word::SignOut`.

May also change, each as said:

- `temper/crates/temper-client-view/src/tests.rs`: the `mod person_menu;` line.
- `temper/crates/temper-client-view/src/tests/look.rs`: the new styles' rows in its name and role tables.
- `temper/crates/temper-client-view/src/tests/rail.rs`: only what reads the browser's person as the foot's last named child or a `Role::Status`, today the `foot.last()` assertion of the link-state test, which keeps it for the terminal and finds the `Role::Menu` for the browser. `rename-row` of `ui-rename-focus` edits this file too: keep to those lines.
- `temper/crates/temper-client-view/src/tests/frame.rs`, `settings.rs`: as `rail.rs`, should one read the browser's person so. Their terminal assertions and every "Sign out" `Role::Button` assertion stand: those are the tab's and the banner's, and still true.
- `temper/crates/temper-client-view/src/tests/accounts.rs`: one test, beside `sign_out_keeps_its_element_when_deployment_accounts_arrive`, below.
- `temper/tools/gallery/src/tests.rs`: the new sample's `Holds` row, the two tests below. On a catch-up, regenerate as the request says.
- `temper/docs/help/**`: nothing. No page names the Accounts tab's Sign out.

Options, each pre-approved:

- **A story that sees `tier.states().live` on a page with no project** (the banner's `Role::Status` "Owner"; `jig/testing/jig-client-fake-person-browser/src/attacks.rs` and `stories.rs` read `states.live`): give that step its own `Find::exactly_named(Role::Status, b"Owner")`, in temper's world if the step is temper's. If it is jig's shared story, make `live()` fit that story's page and say so.
- **The press outside:** `first_chat(b"Hello")`, the chat's heading, is the inert thing to press. If that heading turns out to be bound, press any inert text of the page and say which.
- **An element clipped at a short window:** `#app > .temper-rail` scrolls. If the proof shows the open menu clipped, add `#app > .temper-rail:has(> .temper-rail-foot [aria-expanded=true]) { overflow: visible; }` beside the new rules, as the New menu's rule does. Otherwise add nothing.
- **Look token numbers:** take the next free ones, in the array's order. A catch-up that meets another plan's tokens renumbers yours.
- **A journey's reprint that won't replay** is a red to explain. The acts are recorded as events, so adding the row should reprint only. If it can't, re-record under the request's *Recording flakes*.

## Steps

1. **The catch-up** (`notes/common.md`).

2. **The tests, red,** in a new `temper/crates/temper-client-view/src/tests/person_menu.rs`, on `fixtures` (`browser_page_view`, `page_view`, `root_at`, `no_project_root`), one each:

   ```rust
   #[test]
   fn the_rails_person_is_a_menu_in_a_browser() { /* the Slot, the Menu named Pat, two items in order; no Status Pat in the rail */ }
   #[test]
   fn the_person_menus_items_go_to_accounts_and_sign_out() { /* crate::act on each: Pages(Go { Accounts }), Pages(SignOut) */ }
   #[test]
   fn the_terminal_keeps_the_persons_line_and_has_no_menu() { /* page_view: Status Pat in the foot, no Menu named Pat, no MenuItem Sign out */ }
   #[test]
   fn a_lost_sign_out_keeps_its_line_after_the_menu_which_still_offers_it() { /* browser, on Home: the foot's last child the Alert, the item's act still SignOut */ }
   #[test]
   fn the_accounts_tabs_sign_out_is_marked_for_the_sheet() { /* browser and terminal, with a project: the form holds SETTINGS_SIGN_OUT and SETTINGS_SETTING */ }
   #[test]
   fn a_project_less_accounts_page_keeps_the_banners_sign_out() { /* browser: Button Sign out present, no MenuItem Sign out, no Menu named by the person */ }
   ```

   And beside `sign_out_keeps_its_element_when_deployment_accounts_arrive` in `tests/accounts.rs`, on its `owner_root` and `deliver_deployment`:

   ```rust
   #[test]
   fn the_menus_sign_out_keeps_its_element_when_deployment_accounts_arrive() { /* as its neighbour, for the MenuItem */ }
   ```

3. **The view.** `temper/crates/temper-client-view/src/frame.rs`, in `rail`, where it draws `me`'s name today:

   ```rust
   person(b, root, w, bindings);
   if root.client().link().sign_out_lost { /* as today */ }
   ```

   ```rust
   /// The rail's person: a line in the terminal, a menu in a browser
   /// (temper/docs/design/client/README.md, 5.3).
   fn person(b: &mut Builder, root: &Domain, w: &crate::words::Words, bindings: &mut Bindings) {
       let Some(me) = root.pages().me(root.client()) else { return };
       match w.shell {
           Shell::Terminal => text(b, Role::Status, me.party.name.bytes(), None),
           Shell::Browser => { /* Slot RAIL_PERSON, Menu named me.party.name, two MenuItems as the New menu's are bound */ }
       }
   }
   ```

   `look.rs` gains `RAIL_PERSON` (`temper-rail-person`) and `SETTINGS_SIGN_OUT` (`temper-settings-sign-out`). `pages/settings.rs`' `sign_out` adds the second to its form, keeping `SETTINGS_SETTING`. Fix its comment only if it no longer holds.

4. **The sheet,** `temper/crates/temper-client-shell-web/temper.css`, every new rule under `/* ==== ui: settings ==== */`, none at the file's end:

   - `#app > .temper-rail > .temper-rail-foot > .temper-rail-person { position: relative; }`
   - its `> [aria-haspopup]`: the foot's row as the mock draws it, `padding: 6px 10px`, `border: 0`, `border-radius: 6px`, no background, the foot's muted colour and size, full width, left-aligned; on hover `#f0f0ee` and ink, as the foot's links; `[aria-expanded=true]` the hover's grey and ink. No `outline: 0`.
   - its `> [role=menu]`: `position: absolute; bottom: calc(100% + 6px); left: 0; right: 0; z-index: 30;` and the switcher's panel, as `.temper-projects-menu` draws it: flex column, `padding: 6px`, the line's border, `border-radius: 10px`, the surface, its shadow. `> [role=menu][hidden] { display: none; }`. Its items take `#app > .temper-rail [role=menuitem]` as they are; check the weight against the switcher's rows, which are not bold.
   - `@media (min-width: 761px) { .temper-settings-setting.temper-settings-sign-out { display: none; } }`

   Keep the row's closed look to the pixel of today's plain row: read `getBoundingClientRect` before and after.

5. **The gallery:** a sample `rail-person-menu` (world `Solo`, `Address::Home { project: 1 }`, `Shown`), `samples::N` up one, `names.txt` sorted, `open_menu` generalised to name its menu by sample, and in `temper/tools/gallery/src/tests.rs` its `Holds` row and:

   ```rust
   #[test]
   fn the_person_menu_sample_opens_above_its_row() { /* as the_rail_menu_sample_opens_the_menu: aria-expanded true, its menu not hidden, both items */ }
   #[test]
   fn the_accounts_tabs_sign_out_hides_over_a_phones_width_alone() {
       // temper.css holds the hiding rule for .temper-settings-sign-out inside its
       // `@media (min-width: 761px)` block, and no other display: none for that class.
   }
   ```

   The browser tier opens no phone's width, so this test holds the phone's half and the proof shows it once.

6. **The browser stories.** In `temper/tests/client-browser/src/lib.rs`, `live()` becomes `Find::exactly_named(Role::Button, b"Owner")`, the row. Then, with `row` that button and `item(name)` `Find::exactly_named(Role::MenuItem, name)`:

   - `browser_sign_in.rs`, where it presses Settings, Accounts and Sign out today, keeping every step around them:

     ```rust
     press(row), see(item(b"Accounts")), see(item(b"Sign out")),
     Step::Key(Key::Escape), Gone(item(b"Sign out")), Step::Focused(row),
     press(row), press(first_chat(b"Hello")), Gone(item(b"Sign out")), Step::Focused(row),
     press(row), press(item(b"Accounts")),
     see(Find::exactly_named(Role::Heading { level: 1 }, b"What agents run on.")),
     Gone(Find::exactly_named(Role::Button, b"Sign out")),
     press(row), press(item(b"Sign out")),
     see(tier.states().signed_out), // and on as today: reload, go, reopen
     ```

   - `browser_public.rs`, line 137 on: `press(row)`, `press(item(b"Sign out"))`, in place of Settings, Accounts and the button.
   - `browser_attacks.rs`: the page's `Gone` reads the row, since a hidden item would pass with nothing drawn; each `Settings`, `Accounts`, `see(live)` becomes `press(row)`, `see(item(b"Sign out"))`, `Step::Key(Key::Escape)`.

   Run each alone with `heavy tools/tiers/run temper browser -- -E 'test(=NAME)'`, then `heavy tools/tiers/run temper browser` once, whole, since `live()` is read throughout, reporting its time against its budget.

7. **The prints:** 23 journey pages hold the foot. `heavy cargo run -p temper-client-journeys` reprints them; the diff is the row and nothing else.

8. **The proof:** a throwaway CDP script on a fake box (`temper/docs/plan/ui-final-walk/driver/`, on `origin/ui-final-walk/integration` if not yet on your base, shows how to bring one up with `--fake-backends`; the request's *Render proofs* line binds 420). At 1440 and 1000: Home with the menu closed and open, each beside the mock's `home.html` at that width, and the Accounts tab, no Sign out. At 420: Home with no row, and the Accounts tab with its Sign out. At each width read and report `document.activeElement` after Escape and after a press outside, and the row's `getBoundingClientRect` closed.

9. **Bite,** the implementer's text: break each rule once. The report names what each new test showed.

10. **Close the status** (Acceptance's last line), in the last commit.

11. **Check,** the implementer's text: `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world -p temper-client-gallery -p temper-client-journeys -p temper-client-browser-world`. `python3 -I -B temper/scripts/old_forms.py`. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

The implementer's report, with: the red each test showed; whether you merged `origin/main`, and what `ui-rename-focus` had landed by then; any story that needed its own `live`; the journeys' diff in a line; the proof's screenshot paths and the values read at each width; what still differs from the mock beyond the entry in the status file; the browser tier's time.
