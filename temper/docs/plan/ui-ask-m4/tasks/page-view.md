+++
id = "page-view"
part = "page"
kind = "feature"
needs = ["page-domain"]
files = ["temper/crates/temper-client-view/src/pages/remembered.rs", "temper/crates/temper-client-view/src/pages.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/pages/settings.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/tests.rs", "temper/crates/temper-client-view/src/tests/remembered.rs", "temper/crates/temper-client-view/src/tests/frame.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tests/client-browser/tests/browser_all/browser_sign_in.rs", "temper/docs/design/client/journeys/**"]
effort = "high"
minutes = 200
+++

## Goal

Draw "What temper remembers about you": its page in Deployment's column,
a row a line with "Change" and "Delete", its field, its refusals, its
band with none kept and its line when full; and open it from the person's
menu, between "Accounts" and "Sign out", and on a phone from the
`Accounts` tab's last row (`temper/docs/design/client/README.md`, 5.17;
5.3, the person's menu; 3.5, *Settings and Deployment*, *Sign out*). It
draws what `page-domain`'s state holds, with no placeholder.

Acceptance:

- `pages/remembered.rs` draws 5.17 from `pages::remembered::Remembered`'s
  accessors alone (`notes/contracts.md`, section 6): the level-1 heading
  and the lede; nothing under them until `answered`; then the rows, each
  its line, its small line ("Ask temper chat 14 · 9 Nov" or "Changed by
  you · 9 Nov", the date by the view's times, 5.12), "Change" and "Delete"
  bound to `PageAct::Remembered(Act::Change { note })` and `Act::Delete {
  note }`; the open row's field with its words, "Save" and "Cancel"
  (Escape cancels); a refusal's line under its row in 5.17's words; with
  none, the quiet band "Nothing kept yet" and its line; at
  `notes.party_entries` lines, the full line. `root.rs` dispatches
  `Remembered` to it, replacing `page-domain`'s empty arm; no rail entry is
  current on it.
- The person's menu (`frame.rs`, `person`) holds three items, "Accounts",
  "What temper remembers about you" to `Address::Remembered`, and "Sign
  out", the item where the comment marks its place; the `Accounts` tab's
  last row (`pages/settings.rs:829`, "Signed in as NAME") gains the link
  "What temper remembers about you" before "Sign out", drawn where that row
  is drawn.
- Every word in `words.rs` beside Deployment's (`:6696`), each its own
  `Word` with a stable identifier (`:3722`); its styles in `look.rs` and
  `temper.css` under `/* ==== ui: settings ==== */` (`:2580`), never at
  the file's end: the rows Deployment's (3.5, *The rows*), the acts under
  the line at 760 px and under, each at least 44 pixels tall.
- `src/tests/remembered.rs`, one table test, its rows a domain state each
  (none answered, none, three, twelve, a field open, a refusal moved and
  one past the bound) and the tree's words and acts expected; and in
  `src/tests/frame.rs`, the person's menu's three items in order with
  their acts.
- The browser story that opens the person's menu
  (`temper/tests/client-browser/tests/browser_all/browser_sign_in.rs:47`)
  expects its three items; edited and compiled (`heavy cargo check -p
  temper-client-browser-world --tests`), never run.
- If a journey's prints now differ (the person's menu is drawn in
  `journeys/chat-draft/01.html`, `project-landing/01.html` and
  `welcome-playground/02.html`), regenerate them with `temper-client-journeys
  again` for those journeys (generation, not a walk: `inputs/request.md`),
  retried as the request says on "a key mark caught a moving page".
- Delete `temper/docs/design/client/README.status.md`'s **Not yet** entry
  `5.17, whole`.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  section 6.
- `temper/docs/design/client/README.md`, 5.17 whole; 5.3, the person's
  menu; 3.5, *Settings and Deployment* (its column, *Sign out*, *The rows*);
  5.12 (times); `temper/docs/design/client/browser.md`, 2.5, *What a phone
  presses*.
- The greenfield mock's language for Deployment's rows:
  `temper/docs/design/client/greenfield/mock.css`, `settings.html`.
- Patterns: `temper-client-view/src/pages/deployment.rs:16` (Deployment's
  frame and column), `src/pages/builders.rs:12`, `src/pages/settings.rs:124`
  and `:829`; `src/frame.rs:654` (`person`); `src/tests/deployment.rs`,
  `src/tests/builders.rs:126`.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change:** the journeys' prints and recordings that the menu's
  item makes stale, by their tool alone, and `temper/docs/design/client/journeys.status.md`'s
  line on their refresh if that tool writes it; any view test whose
  expected menu or Accounts row the new item changes, its row only.
- **Options, pre-approved:** if the `Accounts` tab's row is drawn at a
  phone's width alone, draw the link there alone, as the row is. If the
  small line's date has no form in `words.rs`, use the form chat lines use
  for a day.

## Steps

1. The page; its dispatch; the menu's item and the Accounts row's link;
   words and styles.
2. The tables; break one rule (draw rows before `answered`) to see a row
   fail, and restore.
3. The browser story, compiled; the journeys, if stale.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-view -p
   temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-view --lib`; `heavy cargo nextest run -p
   temper-client-shell-web --lib`; `heavy cargo check -p
   temper-client-browser-world --tests`; `heavy cargo check -p
   temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks; name the journeys regenerated, if any.
