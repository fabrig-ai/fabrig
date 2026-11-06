+++
id = "menu-outside"
part = "menu-outside"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/tests/browser/tests/suite/folds_away.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 40
+++

# A press outside an open menu closes it

## Goal

A menu of skein's page part closes on Escape and on an item's press, and a press anywhere else leaves it open. temper's rail is about to rely on it. The gap is generic, so it closes in skein, before its consumer, and the rail's "+ New chat" menu gains it too.

Acceptance:

- A press outside an open `Role::Menu` closes it, in real Chromium, and the press still goes to what it lands on: a bound button outside it reports its act, in the same turn.
- The focus returns to the menu's button after a press on something that takes no focus. A press that took the focus to another control leaves it there.
- A press on the menu's button still toggles it. A press on an item still closes it and reports its act. A press on a disabled item leaves it open.
- A press in a `[data-component]` and a press outside the page part's mount close it too, and a component's press keeps the focus it took.
- With no menu open, a press changes nothing: no op, no focus move.
- Escape's order stands (an open menu, then a disclosure that folds away, then a form's Escape).
- Each new test was red first, and the report says what it showed.
- `skein/docs/design/ui-web.status.md` is back to `No gap.`

## Inputs

- `skein/docs/design/ui-web.md`, 7 (*Menus and disclosures are the page part's to fold*).
- `temper/docs/plan/ui-owner-menu/notes/common.md`, its departures and its catch-up. The catch-up may bring `Op::Focus` changes to `page/web.rs` and `Role::Disclosure` in `page.rs`'s `focusable`: neither is this task's.
- `skein/docs/foundation/testing-strategy.md`, 8.1, for the browser tier's budget.

## Owns and may change

Owns, as `files` lists them.

May also change:

- `skein/crates/skein-ui-web/src/tests/golden/*`, only if the new tests need a new golden. `menu-open.html` and `menu-closed.html` should not change: a change there is a red to explain.
- No fixture: `skein/examples/ui/shell-web/src/folds_fixture.rs` is rewritten by `ui-rename-focus`, and its panel's menu "Actions", "Inside panel" and "Outside" already give the story below.
- `skein/docs/design/STATUS.md` stays untouched. This task's gap lives on this plan's trunk alone and closes before it lands, and `ui-rename-focus` rewrites that line.
- Nothing in temper. `temper/docs/design/client/journeys/skein-ui.css` doesn't change, since the sheet doesn't.

Options, each pre-approved:

- **Where the close sits:** a helper in `page.rs` that `Page::act` reaches for `Event::Click` and for the new event, as below, or an equivalent. A press on a drawing, which `read` turns into another event, closes the menu too: if that makes the click's arm the wrong place, move it up.
- **How a component's press keeps its focus:** the new event may carry a field read in `page/web.rs` (`document.active_element()`), or the listener may close with no focus move. Either way, the model stays testable natively.

## Steps

1. **The tests, red,** in `skein/crates/skein-ui-web/src/tests/page/events.rs`, beside `pressing_a_menu_item_closes_the_menu` and not at the file's end, on its `menu` builder and a second builder holding a bound button beside the menu:

   ```rust
   #[test]
   fn a_press_outside_closes_an_open_menu_and_returns_the_focus() {
       // open MENU, page.focused(FIRST_MENU_ITEM), Event::Click on the main:
       // no act; html is the menu-closed golden; ops end in Op::Focus { node: MENU }.
   }
   #[test]
   fn a_press_outside_goes_on_to_what_it_lands_on_and_keeps_its_focus() {
       // open the menu, page.focused(BUTTON), Event::Click on BUTTON:
       // Some((BUTTON, Act::Press)); the menu closed; no Op::Focus in ops.
   }
   #[test]
   fn a_press_past_the_page_closes_an_open_menu() {
       // open MENU, Event::PressOutside: closed, the focus back on MENU.
   }
   #[test]
   fn a_press_with_no_open_menu_moves_nothing() {
       // Event::Click on the main and Event::PressOutside: no ops.
   }
   #[test]
   fn a_press_on_a_disabled_item_leaves_the_menu_open() { /* its builder's disabled item */ }
   ```

2. **The model,** `skein/crates/skein-ui-web/src/page.rs`. `Event` gains `PressOutside`, for a press the mapping can't name a node for:

   ```rust
   pub enum Event {
       // ...
       PressOutside,
   }
   ```

   Then a press whose node is neither the open menu nor inside it closes the menu before it goes on as it does today. `close_menu` pushes `Op::Focus { node: menu }`. Keep that where `self.focused` is unset, the menu or inside it, and leave it out where the press moved `self.focused` to another node: the browser's `focusin` comes before its `click`.

3. **The platform half,** `skein/crates/skein-ui-web/src/page/web.rs`, the document's `click` listener: where it returns for a press outside the mount, and for `in_component`, give the model the press first and apply its ops. Keep `fold_away`'s call and its order: a menu inside a panel that folds closes with it.

4. **The story in real Chromium,** a new test at the end of `skein/tests/browser/tests/suite/folds_away.rs`, in that file's steps, leaving `a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape` as it is:

   ```rust
   #[test]
   fn a_press_outside_closes_a_menu_and_goes_on() {
       // Go to /test/folds-away. disclosure(b"Panel"), press(b"Actions"),
       // check [role=menu]:not([hidden]); press(b"Inside panel"), see Pressed 2,
       // check [role=menu][hidden], check the panel open,
       // Step::Focused(Button "Inside panel").
       // press(b"Actions"), press(b"Outside"), see Pressed 4,
       // check [role=menu][hidden], check the panel folded.
       // disclosure(b"Panel"), press(b"Actions"), press(b"Component outside"),
       // check [role=menu][hidden], and the component's data-pressed as the file's story reads it.
   }
   ```

   Run it with `heavy tools/tiers/run skein browser -- -E 'test(=NAME)'`, which prepares the bundle.

5. **Bite:** remove the close from step 2, see the new tests fail for that reason, and restore it. The report says what each showed.

6. **Close the status:** `skein/docs/design/ui-web.status.md` back to `No gap.`, in the last commit.

7. **Check,** the implementer's text, and because a press means something new for every shell with a menu: `heavy cargo nextest run -p skein-ui-web -p skein-ui-web-tests -p skein-browser-world -p jig-client-shell-web -p temper-client-shell-web -p temper-client-gallery`, adjusting the names by `cargo metadata` should one differ. `heavy tools/tiers/run skein browser` once, whole, since the page part changed, reporting its time against its budget. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

The implementer's report, with: the red each test showed; how a component's press keeps its focus; whether you merged `origin/main`; the browser tier's time.
