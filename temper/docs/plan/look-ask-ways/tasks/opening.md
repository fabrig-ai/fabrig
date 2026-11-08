+++
id = "opening"
part = "opening"
kind = "feature"
needs = ["opening-state", "path-look"]
files = ["temper/crates/temper-client-view/src/pages/ask_page.rs", "temper/crates/temper-client-view/src/tests/ask.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/**"]
effort = "high"
minutes = 260
+++

## Goal

Draw Ask temper's opening as the look request `ask-temper-ways` draws
it, in both shells, from the domain's `StockWays`: no heading, the lede,
the ways' tabs, the shown way's line, its path pressed, a stop's card,
the legend and its act, and a refusal; and remove what it replaces: the
question, "What it sets" and its note, "Ways to begin" with its
starters and "Not sure yet?", and temper's own way's card with "See how
it works" and "Start from it" (`temper/docs/design/client/README.md`,
3.5, *Ask temper's page* and *temper's own way*).

Acceptance:

- The page's tree is `notes/contracts.md`, section 3, in its order, with
  its roles, names, tokens and acts, every way's word, path and card
  from `StockWays` (nothing written by hand): the tabs from `tabs()`, the
  shown one `Selected` from `shown()`, the panel of the shown tab alone,
  the path `paths::path` with `Pressed` (`open()` the domain's), the card
  of the open stop, the act from `act(index)` disabled while
  `sending()`, the refusal from `refused()`.
- A press on a stop in the drawing (`skein_ui::Act::Drawing { data }` on
  the drawing node) decodes in `frame.rs` to `pages::Act::WayStop {
  project, stop }` through `paths::pressed_stop`; a fallback button and a
  tab press their own acts.
- What the request replaces is gone from the page, its tree and its
  terminal print: the heading "How should temper run PROJECT?", the old
  lede, "What it sets" and its note, "Ways to begin", the three starters
  and "Models by role", "Not sure yet?", the own way's card, "See how it
  works", "Start from it", and the reference's mode group and pane on
  this page. Their words, look tokens (renamed `temper-reserved-N` in
  `NAMES`, as freed tokens are) and `temper.css` rules go with them where
  nothing else draws them; the composer's hint is the chat's.
- `temper.css`: the request's opening rules (its `overlay.css` from
  `.temper-ways-lede` to `.temper-ways-acts > .temper-ways-use`, the
  night rule and the phone part's opening lines, but the path's, which
  `path-look` landed) under `/* ==== ui: settings ==== */`, where
  `.temper-starter`'s stand, as `notes/common.md` says of selectors
  skein draws otherwise: a tab is `[role=tab]`, the selected one
  `[aria-selected=true]`; the legend's kinds by their tokens; the card's
  caret is the drawing's (`path-look`), so `.temper-ways-zoom::before`
  goes and the card follows the figure with no margin; the refusal's
  rule is new, in the request's terms (13.5 px, `--gf-error`). The rules the request
  replaces go where they stand (the sets, note, head, unsure, own way
  and its parts, the Split mode's starters, the terminal's own way rule).
  The report lists each rule landed, translated or dropped.
- `src/tests/ask.rs`, whose tests walk the removed parts, is rewritten
  with the page, as tables where it can: the tree's order and tokens;
  the tabs' order with "now" on the current; a tab's press showing its
  panel; a stop's press opening its card (an agent's facts, a link "Open
  the reviewer ›" to Settings), pressed again closing it; the act per
  state ("Use this", "Keep it", none for "Yours", none when `may_change`
  is false, none once configured), disabled while sending; the refusal
  in words; the loading and ended states; and the terminal's print in
  order, the current tab "NAME, now", the path a stop a line. Its panels
  are hand-made `WaysPanel`s in the test (as `temper-client-domain`'s
  `tests/forge_stock_ways.rs` makes them), the request's three paths'
  words and actors among them.
- The gallery: `ask-reference` and `ask-reference-draft` go (their entries
  in `src/tests.rs`, `names.txt` and their prints and goldens); the other
  Ask temper states are regenerated with its own tool. Its new `ask-ways-*`
  states are `opening-gallery`'s.
- Journeys: if the gate's focused replay check reports a journey stale,
  it is recorded again with `temper-client-journeys again`
  (`notes/common.md`); none is expected to be.
- The view's, the gallery's and the shells' focused tests pass.

## Inputs

- `temper/docs/plan/look-ask-ways/notes/common.md`;
  `temper/docs/plan/look-ask-ways/notes/contracts.md`, sections 1 to 4.
- `temper/docs/design/client/README.md`, 3.5, *Ask temper's page* (the
  conversation, the composer, *temper's own way*), 5.13, *A way's acts*,
  and 5.16 (this cut's revisions).
- `temper/docs/design/client/browser.md`, 2.5, *Ask temper's ways*.
- `temper/docs/design/draft/look-requests.md`, section 3, steps 3 to 5.
- The look request's message, its four pages and its `overlay.css`, as
  `notes/common.md` says to read them.

## Owns and may change

- Owns: `temper-client-view`'s `src/pages/ask_page.rs` and
  `src/tests/ask.rs`; the opening's words in `src/words.rs` (numbers from
  4302) and tokens in `src/look.rs` (after `path-look`'s); the opening's
  rules in `temper.css`.
- `src/frame.rs`: the decode arm for the drawing's act; nothing else.
- `src/tests/look.rs`: rows naming a token this task frees or adds.
- `temper/tools/gallery`: removing the two reference states and
  regenerating the others' prints and goldens, `names.txt` kept sorted.
- Left in place, for `starters-out`: the client domain's `Act::Starter`,
  `Act::ShowReference`, `chats.reference` and its modes,
  `pages/reference.rs` and `src/tests/reference.rs`, and the words and
  tokens only the pane uses. The Ask page no longer reaches them.
- Not yours: the client domain (but compiler-forced arms), `paths`.
- Options: if a removed word or token is still drawn elsewhere, keep it
  and say where; if a gallery state other than the two reference ones
  walks a removed part (a starter pressed), change it to the nearest
  state the page still has, keeping its name, and say so; if the drawing
  node cannot take a binding as `paths` gives it, that is `path-look`'s
  to have made possible: report it as a `Question:`.

## Steps

1. Read 3.5's *Ask temper's page*, the contracts' section 3, the request's
   message and `01.html` to `04.html`'s opening, and `ask_page.rs` whole.
2. Rewrite the opening in `ask_page.rs`; remove the replaced parts, their
   words and tokens; add the decode arm.
3. The rules in `temper.css`.
4. Rewrite `src/tests/ask.rs` as Acceptance lists. Break a rule once by
   hand for each new row (the act shown while `may_change` is false, the
   card left open on another tab) and see it fail.
5. The gallery: remove the two reference states, regenerate the rest.
6. `heavy cargo nextest run -p temper-client-view --lib -E 'test(/ask/)'`;
   then the package's focused tests; `heavy cargo nextest run -p
   temper-client-gallery`; `heavy cargo nextest run -p
   temper-client-world --test suite -E 'test(/ways/) | test(/ask/)'`;
   `heavy cargo check -p temper-client-tui --tests`; `heavy cargo
   nextest run -p temper-client-shell-terminal`.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (`--write`
   only to lower); `heavy cargo clippy -p temper-client-view -p
   temper-client-gallery -p temper-client-world --all-targets -- -D
   warnings`. Commit with the trailer `Look-request: ask-temper-ways` and
   push.

## Report

As the implementer's text says; list the CSS rules as landed, translated
or dropped, every word and token removed or kept with why, and every
gallery file removed or regenerated.
