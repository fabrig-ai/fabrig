+++
id = "skein-keys"
part = "skein-keys"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui/src/tests/tree.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/tests/**",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 45
+++

# Three small mechanisms a chat's document needs of skein

## Goal

`Ctrl+\` as a shortcut, a keyed slot, and a menu item with a line under its name. Each is a generic mechanism, so it goes into skein before temper's page uses it.

Acceptance:

- **The key `\`,** with the primary modifier alone: `Builder::shortcut`'s assertion allows `Shortcut { key: b'\\', modifier: Primary }`, and `skein-ui-web`'s `shortcut_key` takes `\`'s key
  and its `Backslash` code. A primary shortcut fires inside a field, as today. Its mapping's words, for `aria-keyshortcuts`, follow `mapping.rs`' `Primary` form. A plain `\` or an Alt `\` is
  still refused by the assertion.
- **A key on a slot:** `Builder::key`'s assertion also allows `Role::Slot`. `diff.rs` matches siblings by key as it does items'; add its test: of three keyed slots, changing the second's content
  keeps the first's and third's node ids, and reordering keeps each slot's.
- **A menu item's description:** a `Role::MenuItem` with a description in `skein-ui-web`'s `mapping.rs` is drawn as `<span>NAME</span><span>DESCRIPTION</span>`, its accessible name the name alone
  (`aria-label`, or by `aria-describedby` to the second span, as `mapping.rs` does a description). The terminal's mapping (`skein-ui-text`) prints the name, then the description as a line of its own, if
  a description prints in its other roles.
- The status lines those close, in `ui.status.md` and `ui-web.status.md`.

## Inputs

- `temper/docs/plan/ui-chat-draft/notes/common.md`.
- `skein/docs/design/ui.md`, 3.4 and 6.1; `skein/docs/design/ui-web.md`, sections 6 and 7.

## Owns and may change

Owns, as `files` lists them. May also change `skein/crates/skein-ui/src/diff.rs` and its tests, `skein/crates/skein-ui-text/src/**` and its tests for the menu item's words, and
`skein/tests/browser/**` only if a test it holds names what changed.

Options, each pre-approved:

- **`Slot` and a diff's keys:** if a keyed slot's siblings aren't matched by key by the code paths items take, generalise that path rather than add one. If an unkeyed sibling among keyed ones is refused for items, it's refused the same here.
- **A browser's `\`:** on keyboards where `\` has no key of its own, `code` reads `Backslash` or `IntlBackslash`; take both.

## Steps

1. The three, with their tests, each having bitten: `a_primary_backslash_is_a_shortcut`, `a_plain_backslash_is_refused`, `shortcut_key_takes_backslash`, `a_keyed_slot_keeps_its_node_as_its_sibling_changes`,
   `a_menu_item_shows_its_description_in_a_span_of_its_own`.
2. `heavy cargo nextest run -p skein-ui -p skein-ui-web -p skein-ui-text`.
3. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
