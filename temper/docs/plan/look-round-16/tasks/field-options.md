+++
id = "field-options"
part = "field-options"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/outline.rs",
  "skein/crates/skein-ui/src/tests/tree.rs",
  "skein/crates/skein-ui/src/tests/outline.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/golden/**",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 110
+++

# skein-ui: a field's options, moved through from the field

## Goal

A text area keeps the focus while a list under it is moved through with the arrows, pressed with Enter and completed with Tab: the chat terminal's `/` commands and its choices. skein-ui has menus that take the focus into their items, and no key but Enter and Escape reaches a field's owner; this adds the state *options* on a menu, its web mapping as a list the field controls, and its keys, generic in skein before temper uses it (`terminal-commands`). The shapes are `notes/contracts.md`, 3, binding; the behaviour is `skein/docs/design/ui.md`, 3.2, *options*, and `skein/docs/design/ui-web.md`, 6 and 7, *A field's options*.

Acceptance:

1. `State::Options` exists; the builder asserts it on a menu alone; `outline::check` faults an options menu whose innermost form has no text field or area before it.
2. The web mapping draws an options menu as `role="listbox"` with `id="skein-ui-options-N"` and no button and not hidden, its items as `role="option"` with `id="skein-ui-option-N"`; its field carries `aria-autocomplete="list"` and `aria-controls` naming the list, and `aria-activedescendant` and the option's `aria-selected="true"` follow the active option.
3. In the page, with the focus in the field and an enabled option in its list: ArrowDown and ArrowUp move the active option, wrapping and skipping disabled ones; plain Enter with no composition presses the active option and the same Enter gives no `Submit`; Tab completes as `ui-web.md`, 7 says, reporting the field's `Input`, and otherwise gives nothing, so the browser moves the focus; Escape presses the form's bound Escape as today; a press on an option presses it.
4. The active option starts at the item marked selected or the first enabled one, stays across an apply while its node stays in the list, and starts again otherwise.
5. Today's menus, Enter, Escape, PageUp and PageDown behave as they do: their tests pass unchanged.
6. `skein-ui` and `skein-ui-web` pass clippy for `wasm32-unknown-unknown` with warnings denied.
7. The two `field-options` **Not yet** entries, in `skein/docs/design/ui.status.md` and `ui-web.status.md`, are deleted.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules. This task is skein's: of the request's rules, the trailer, the runs and the checks hold.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 3: binding.
- `skein/docs/design/ui.md`, 3.2 (the states' table and *options*), 6.1 (shortcuts and Escape), 14 (the outline).
- `skein/docs/design/ui-web.md`, 6 (the table's menu row; *What the page part sets*), 7 (*Escape*, *Menus and disclosures*, *A field's options*, and the log's PageUp and PageDown).

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but the last commit deletes the two `field-options` entries. If one can't close, it stays, and goes first under `## Left`.

Reader edits allowed:

- An exhaustive `match` on `State` elsewhere in any tree (`skein-ui-text`'s render, `skein-ui-web`'s mapping, jig's or temper's views, the test kits): the compiler's arm for `Options`, doing nothing in the text shell.
- `skein/crates/skein-ui-text/src/tests.rs` or a golden there: only if a menu row prints differently, which it should not.
- `skein/docs/design/ui-text.md`: none; the text shell prints an options menu as any menu.

Options, each decided:

- **How Enter's second event is held:** `read` in `page/web.rs` gives `Event::Key` then `Event::Enter` for one keydown (`page/web.rs`, `read`); the page records that the key's branch pressed an option, and `Event::Enter`'s branch gives no `Submit` while that record stands for the same keydown. Any shape that holds acceptance 3 and passes `an_enter_keeps_its_input_after_a_composition` will do.
- **Composition:** an Enter while a composition is under way is the field's, as today; the options take no key then.
- **Where the active option is applied:** an `Op` the platform half carries, setting `aria-activedescendant` on the field and `aria-selected` on the options, as `menu_element` toggles a menu's attributes.
- **A pointer down on an option:** prevent its default so the field keeps the focus, then the click presses as any item's does.
- **The field's ARIA attributes:** set by the page when it applies a tree in which a form holds an options menu after the field, and removed when the menu goes.

Leave alone: today's menu-button menus, `Act` (no new act: a press and an input carry it all), `skein-ui-web`'s platform, store and screen (`web-screen`'s), and the text shell's commands.

## Steps

### 1. The tree

`tree.rs` gains `State::Options` with the contract's doc line; `builder.rs`'s state assertion holds it to `Role::Menu`; `outline.rs`'s `check` faults an options menu with no field before it in its innermost form.

### 2. The web page

`mapping.rs` maps the options menu and its items; `page.rs` keeps the active option beside `menu` and takes the keys in a branch before today's menu handling and before `if in_field && modifier == Modifier::Plain` (`page.rs`, `key`); `page/web.rs` carries the new op and prevents an option's pointer down.

### 3. The tests

Rows of what exists, and one new table test:

- `skein/crates/skein-ui/src/tests/tree.rs`: the builder's state assertion, a row where the test of a role-held state loops (`Required`'s, `FoldsAway`'s); `tests/outline.rs`, a faulting and a passing row.
- `skein/crates/skein-ui-web/src/tests/mapping.rs`: a menu with `Options` and two items in `ROLES` (or `PROPERTIES`), checked by `each_role_maps_to_its_element` and `each_role_has_its_accessible_roles`; `golden/mapping.html` regenerated by its test's own means.
- `skein/crates/skein-ui-web/src/tests/page/events.rs`, **new**, `a_fields_options_take_its_keys`: one table of a form with a text area, its options menu of three items, the middle one disabled or not, and rows `(keys, field words, expected act or none, active option after)`: ArrowDown, ArrowDown wrapping, ArrowUp skipping a disabled one, Enter pressing the active option and no `Submit` after it, Shift+Enter giving none of the options', Tab completing `/mo` to `/model`, Tab with empty words giving none, Tab where the words equal the name giving none, Escape pressing the form's escape, a click on an option, and the active option kept across an apply that keeps its node and reset by one that removes it.

Break each rule once by hand (the Enter hold, the wrap, the Tab condition, the reset), run the test, see it fail for that reason, restore, and name each in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p skein-ui --lib -E 'test(=NAME)'` and `heavy cargo nextest run -p skein-ui-web --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B skein/scripts/old_forms.py`.
- `heavy cargo clippy -p skein-ui -p skein-ui-web -p skein-ui-text --all-targets -- -D warnings`.
- `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui -p skein-ui-web -- -D warnings`.
- `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-text -p skein-ui-tests`, and `heavy cargo check -p jig-client-view -p temper-client-view -p temper-client-shell-web --tests` for `State`'s readers.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailer:** `Look-request: chat-terminal` on every commit, one line above the other trailers.
- The last commit deletes the two status entries. Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- how Enter's second event is held;
- each reader file edited for `Options`'s arm;
- whether the status entries stand.
