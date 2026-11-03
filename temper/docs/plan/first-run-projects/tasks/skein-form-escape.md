+++
id = "skein-form-escape"
part = "form-escape"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/outline.rs",
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/tests/outline.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 50
+++

# skein-ui: Escape inside a form

## Goal

A field edited in place (temper's banner renames a project in its own row, the draft
`temper/docs/design/draft/first-run-projects.md`, 7) is cancelled by Escape, so skein-ui lets a
node inside a form name Escape, and the web page part presses it on the Escape key while the
focus is in that form, outside every open dialog (`skein/docs/design/ui.md`, 6.1 and section 14;
`skein/docs/design/ui-web.md`, section 7, "Escape"). skein names no consumer: this is the generic
mechanism its consumers use.

Acceptance:

- The outline accepts Escape named by a node inside a `Role::Form` (any depth, the form inside a
  landmark or the page) as it accepts one inside a dialog, and still faults one outside every
  dialog and every form.
- In `skein-ui-web`'s page model, the plain Escape key while the focus is on a node inside a form
  and no dialog is open, and no menu takes it, presses the first bound, enabled node naming
  `Shortcut::ESCAPE` in the innermost form holding the focus, with the key's default prevented,
  whether or not the focus is in a field. With no such node it does nothing. A dialog's Escape
  (its cancel event) is unchanged.
- The tests below pass; the status entries below are gone.

## Inputs

- `temper/docs/plan/first-run-projects/notes/common.md` (its rules hold in skein too).
- `skein/docs/design/ui.md`, 6.1 ("Escape"), section 14 (the outline's rule table), and section
  16's browser mapping row "a shortcut".
- `skein/docs/design/ui-web.md`, section 7: the "Escape" bullet.

## Owns and may change

- `skein/crates/skein-ui/src/outline.rs`: the Escape rule tracks open forms as it tracks dialogs
  (`dialog_end`); the variant `Rule::EscapeInDialog` becomes `Rule::EscapeInDialogOrForm`, its doc
  "Escape named by a node outside every dialog and every form" (its readers are skein-ui's own;
  if `rg` finds one in another crate, edit it in its name alone and say so).
- `skein/crates/skein-ui/src/tree.rs`: `Shortcut::ESCAPE`'s doc says inside a dialog or a form.
- `skein/crates/skein-ui/src/tests/outline.rs`, `FAULTS`: `("escape form", Role::Page,
  &[Role::Main, Role::Form], None)` and `("escape banner form", Role::Page, &[Role::Banner,
  Role::Form], None)` beside `"escape dialog"`; `"escape main"` keeps its fault under the new
  name.
- `skein/crates/skein-ui-web/src/page.rs`, `key` (line ~724): before the `in_field` return, a plain
  `Escape` not taken by `menu_key` looks up the innermost form ancestor of `self.focused` (no open
  dialog holding the focus) and returns `Some((node, Act::Press))` for its first bound, enabled
  node naming `Shortcut::ESCAPE` in preorder. How the page part finds a node's ancestors is yours;
  if the web listener needs the key's default prevented on that path and does not already, the
  edit in `src/page/web.rs` is allowed, and say so.
- `skein/crates/skein-ui-web/src/tests/page/events.rs`, beside
  `escape_presses_the_dialogs_escape_button`, in its fixtures' form:
  `escape_in_a_form_presses_its_escape_button` (a main holding a form holding a field and a
  disabled and an enabled Escape button; the field focused; `plain_key(b"Escape")` presses the
  enabled one), `escape_in_a_form_naming_none_does_nothing`, and
  `escape_outside_a_form_presses_nothing` (the focus on a node of the main outside the form).
- Status, as `notes/common.md` says:
  - `skein/docs/design/ui.status.md`, **Not yet**: remove `6.1, 14: Escape inside a form: the
    outline refuses Escape outside every dialog (\`skein-ui/src/outline.rs\`,
    \`Rule::EscapeInDialog\`).`
  - `skein/docs/design/ui-web.status.md`, **Not yet**: remove `7: Escape in a form outside every
    dialog: the page part presses Escape from a dialog's cancel event alone
    (\`skein-ui-web/src/page.rs\`, \`Event::Cancel\`).`
  - `skein/docs/design/STATUS.md`: ` and \`ui-web.md\` (Escape in a form)` goes from its summary,
    which then reads "A few items lag in `lib.md`, `examples.md` and `ui.md`."

## Steps

1. The outline; `heavy cargo nextest run -p skein-ui --lib -E 'test(outline)'`.
2. The page model; `heavy cargo nextest run -p skein-ui-web --lib -E 'test(escape)'`.
3. The status edits; the citations check.
4. `cargo fmt`; old forms for skein; `heavy cargo clippy -p skein-ui -p skein-ui-web --all-targets
   -- -D warnings`; the focused tests of `skein-ui`, `skein-ui-web` and their direct dependents in
   skein (`skein-ui-text`, `skein-ui-bundle`, `skein/tests/ui-web`'s and `skein/tests/browser`'s
   packages); `heavy cargo check -p jig-client-shell-web -p temper-client-shell-web --tests` for the
   cross-tree readers. Commit the code and tests, then the status text; push.

## Report

As the implementer's text says.
