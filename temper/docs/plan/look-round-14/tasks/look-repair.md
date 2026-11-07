+++
id = "look-repair"
part = "look-repair"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/main.rs",
  "temper/crates/temper-client-domain/src/forge/words.rs",
  "temper/crates/temper-client-domain/src/forge/sheet.rs",
  "temper/crates/temper-client-domain/src/tests/sheet.rs",
  "temper/crates/temper-client-view/src/forge/sheet.rs",
  "temper/crates/temper-client-view/src/forge/connect.rs",
  "temper/crates/temper-client-view/src/pages/settings.rs",
  "temper/crates/temper-client-view/src/pages/connect_model.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/sheet.rs",
  "temper/crates/temper-client-view/src/tests/settings.rs",
  "temper/crates/temper-client-view/src/tests/forges.rs",
  "temper/tests/client/tests/suite/forges_edges.rs",
  "temper/tests/client/tests/suite/playground.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/forge.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "high"
minutes = 90
+++

# What the render check found in `connect-flows` and page 07

## Goal

Make the real pages look like the requests' pages, where a render check found they don't. These are misses, not new looks. Most are a broad rule winning over a landed one: fix the cascade, and add no `!important`.

Acceptance, by reading the rule or the view and by the tests below. No browser runs, here or in any step:

**The select chevron**

1. Every select under `.temper-settings`, the sheet's Forge and Account and Models' Provider included, draws `files/chevron.svg`, 14 px in from the border, with padding-right 36, whatever its own rules say.

**The sheet, pages 16 to 22**

2. `.temper-connect-sources` is one row: flex, 2 px gap, 0 top margin, about 271 × 36.
3. Copy public has a visible caption, "Their addresses, one per line", 14 px, over a textarea 107 px high.
4. `.temper-connect-list` has margins 0.
5. The no-match row has 14 px vertical padding, 49.69 high. It says `Nothing here matches “QUERY”.`, with the search as typed, trimmed.
6. A completed search with no matches shows no count, with or without picks.
7. "More"'s summary reads "More · Owned, it runs CI".
8. The radio and check captions in `.temper-connect-how` (As its admin, Owned, Fork, Context, It runs CI and their siblings) are 15 px, with the 24 or 23.25 px line height they inherit.
9. The Forge and Account captions over their selects are weight 600, line height 21.69, bottom margin +4. "What temper does with it" keeps its own values.

**The connect pages**

10. Add key and Connect your Forgejo (pages 02, 05, 11) are 14.5 px, line height 23.2, ink `rgb(24,24,27)`. The row's description stays 13.5 px and muted.
11. Forges' action says "+ Connect a forge".
12. A model way's path ends with the way's name after "Connect a model": ChatGPT, Claude or OpenAI API key (pages 03, 06, 07). "Connect a model" is a link there, and the phone's "‹" goes to it, as a forge way's do on pages 12 to 14.

**Settings, Models, page 07**

13. Provider, the name and Add sit on one row at every width, 36.39 high. "The model’s name" isn't drawn and stays the field's accessible name.
14. The Provider and "Add a model by name" sections' heads have a 6 px bottom margin. No other Settings section's head changes.

**Keeping the record**

15. Every commit carries its trailer, and the last commit carries the account (Steps, 6). The three `look-round-14` **Not yet** entries are gone.

## Inputs

- `temper/docs/plan/look-round-14/inputs/request.md`: all of it, *Leave alone* and *For every task of this plan* first. It binds whatever this spec says.
- `temper/docs/design/draft/look-requests.md`, 3 to 5: the trailers, the account.
- `temper/docs/design/client/browser.md`, 2.5: the chevron.
- `temper/docs/design/client/README.md`, 3.5: `Forges`, *The `Models` tab*, *The flows' paths*; 4.4: "More", Copy public.
- `temper/docs/design/client/forge.md`, 5.1: a search that finds nothing.
- The request's pages, read with `git show` alone: `86358b6de5:temper/docs/design/client/journeys/connect-flows/` (`connect.css`, `03.html`, `16.html`, `21.html`, `adopt.js`) and `3d0ed33559:temper/docs/design/client/journeys/model-picker/07.html`.

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but this task deletes the entries that name `look-round-14` in `README.status.md`, `forge.status.md` and `browser.status.md`, in its last commit. An entry you could not close stays, and goes first under `## Left`.

Reader edits allowed, for the words alone:

- `temper/crates/temper-client-view/src/tests/forges.rs`, `temper/tests/client/tests/suite/forges_edges.rs`: "+ Add a forge" becomes "+ Connect a forge".
- `temper/tests/client/tests/suite/playground.rs`, line 133's absent-name list: `b"Add a forge"` becomes `b"+ Connect a forge"`, so the check still bites.
- `temper/tools/gallery/src/tests.rs`, another view or world test: a row your correct change trips, keeping what it checks for everything else. Name each in the report.

Leave alone:

- `temper/crates/temper-client-view/src/pages/models.rs` and `skein/`: this task needs neither, since item 13 is the stylesheet's.
- `temper/docs/design/client/mockups/pages.js`: it says "Add a forge", and the mock in `greenfield/` is the target.
- The request's *Leave alone* list.

Options, each decided:

- **`rows=4`:** `skein_ui` carries no such attribute, and this repair stays in temper. Item 3 is a `min-height`, and the account says so.
- **The way rows of item 12's test:** default, in `temper/crates/temper-client-view/src/tests/settings.rs`. If its fixture can't offer ways and setting `me`'s providers there won't do, put them in `temper/crates/temper-client-view/src/tests/connect_model.rs` and say so. A draft beside this one, `account-email/names-by-email`, is changing that file.
- **A stale print:** no recorded journey draws a Settings path, the sheet or Forges' action, so none should change. If `temper-client-journeys`'s focused tests say otherwise, regenerate that journey with `temper-client-journeys again` and say so.
- **`model-catalogue-3`:** if it landed "Read again" or other Models rules first, catch up beside them and keep both.

## Steps

### 1. The stylesheet

In `temper/crates/temper-client-shell-web/temper.css`, change each rule where it stands. These are the values; the comments are yours, in the file's manner.

```css
/* item 1, replacing `.temper-settings select`. The doubled id outweighs every `#app >` scope,
   whose `background` and `padding` would otherwise take the arrow away. */
#app#app .temper-settings select { appearance: none; padding-right: 36px; background-image: url(files/chevron.svg); background-repeat: no-repeat; background-position: right 14px center; background-size: 12px; text-overflow: ellipsis; }

/* item 2: the kit's radiogroup rule, in the away area, no longer catches the chooser. */
.temper-settings [role=radiogroup]:has(> input):not(.temper-connect-sources) { /* its declarations, unchanged */ }

/* item 3, replacing `textarea.temper-connect-addresses`, whose other declarations lose to
   `.temper-settings-text` on the request's own page. 4 × 23.25 + 12 + 2. */
.temper-settings textarea.temper-connect-addresses { min-height: 107px; resize: vertical; }

/* item 4: add `margin: 0;` to `.temper-connect-list`. */

/* item 5, replacing `.temper-connect-none`, which the list's `> li` rule outweighs. */
.temper-connect-list > li.temper-connect-none { padding: 14px 0; font-size: 14px; color: var(--gf-muted); }

/* item 8: in `.temper-connect-how > div > label`, `font-size: 14px` becomes 15px. */

/* item 9, after `.temper-connect-sheet p.temper-connect-title`. A caption directly over a
   control takes a label's values; "What temper does with it" sits over no such control. */
.temper-connect-sheet p.temper-connect-title:has(+ .temper-settings-text) { margin: 0 0 4px; font: 600 14px/1.55 var(--gf-sans); }

/* item 10, replacing `.temper-connect-row span:not(:has(> strong))`. */
.temper-connect-row > div > span:not(:has(> strong)) { font-size: 13.5px; color: var(--gf-muted); }

/* item 13: drop `flex-wrap: wrap;` from `.temper-models-add`, add `min-width: 0;` to its
   `> input`, and hide the label the web shell draws for a named text field, as
   `.temper-settings-switch + label` does. */
#app > .temper-settings .temper-models-add > label { position: absolute; width: 1px; height: 1px; padding: 0; overflow: hidden; clip-path: inset(50%); white-space: nowrap; }

/* item 14, in the block of Settings' list of models. */
.temper-settings-body > .temper-settings-section:has(> .temper-models-source, > .temper-models-add) > .temper-settings-section-head { margin-bottom: 6px; }
```

Leave the existing `!important`s and the Models select's own `padding` and `background`: item 1 has to win over them.

### 2. The domain

- `temper/crates/temper-client-domain/src/forge/words.rs`: `(Word::NoneFound, &[b"Nothing here matches \xe2\x80\x9c", b"\xe2\x80\x9d."], &[Name])`.
- `temper/crates/temper-client-domain/src/forge/sheet.rs`, `count`: a completed search with nothing shown is `Count::Hidden`, otherwise `Count::Found { found, of }` as today.

### 3. The sheet's view

In `temper/crates/temper-client-view/src/forge/sheet.rs`:

- `making`, `Source::Copy`: in the browser shell alone (`w.shell == crate::words::Shell::Browser`, as the sheet's heading), a `Role::Paragraph` styled `look::CONNECT_TITLE` with `Word::Addresses`, directly before the `TextArea`. The area keeps its name. Item 9's rule reaches it because it sits directly over `.temper-settings-text`.
- `picker`: `Word::NoneFound` takes `Hole::Name` of `sheet.find.text.trim_ascii()`. The choice between `NoneReachable` and `NoneFound` also trims, so a blank search doesn't say `“”`.
- `more`: the summary starts `"· "`, inline, as `pages/models.rs` joins `" · "`. The description becomes `· Owned, it runs CI`, and the web shell puts its space after "More".

### 4. The words and the way's path

- `temper/crates/temper-client-view/src/words.rs`: `Word::AddForge => (&[b"+ Connect a forge"], &[])`. The word's name and number stay.
- `temper/crates/temper-client-view/src/pages/settings.rs`: beside `path`, sharing its drawing:

  ```rust
  /// A way under a flow's choices (README.md, 3.5, The flows' paths): the back link
  /// and the trail go through the choices, and the way's name is last.
  pub(crate) struct FlowWay<'a> {
      pub(crate) choices: Word,
      pub(crate) address: Address,
      pub(crate) name: &'a [u8],
  }

  pub(crate) fn way_path(
      b: &mut Builder,
      root: &Domain,
      w: &Words,
      bindings: &mut Bindings,
      project: Option<u32>,
      tab: FlowTab,
      way: &FlowWay<'_>,
  )
  ```

  It draws what `elsewhere_path` draws, for either tab: nothing in the terminal or with no project.
- `temper/crates/temper-client-view/src/forge/connect.rs`: `elsewhere_path` becomes one call of `way_path`, with `Word::PathAddForge`, `Place::Connect.address(None)` and `Word::ConnectManaged`'s words. Its pages don't change.
- `temper/crates/temper-client-view/src/pages/connect_model.rs`, at the `settings::path` call: read `model::way` first. With a way and `accounts::current_me`, call `way_path` with `Word::PathConnectAccount` and `Address::ConnectModel { way: None }`. Otherwise call `path` as today. The way's name, from `names(me, way)`:
  - `Way::SignIn { plan: true, .. }` and `Way::Program { .. }`: `crate::choice::shown(plan, w)`.
  - `Way::SignIn { plan: false, .. }`: `crate::choice::shown(brand, w)`.
  - `Way::Key { .. }`: `named(w, W::ApiKeyBrand, brand)`.

### 5. The tests

Each is a row or a line in what exists, but the one test below. Tests stay cheap.

- `temper/crates/temper-client-shell-web/tests/suite/main.rs`: one table test beside its siblings, by their `split_once` manner. Rows of selector and declarations: item 1's (all six declarations), and the selectors of items 2, 3, 5, 9 and 10, each with its deciding declaration. After the table, `#app#app` occurs exactly once in `temper.css`, so nothing else can outweigh it, and `.temper-connect-row span:not(` occurs nowhere.
- `temper/crates/temper-client-domain/src/tests/sheet.rs`: after `Count::Found { found: 1, of: 9 }`, with its pick still held, a search that finds nothing is `Count::Hidden`.
- `temper/crates/temper-client-view/src/tests/sheet.rs`:
  - `more_keeps_the_forms_fields_and_summary`: the description is `· Owned, it runs CI`.
  - The Copy public test: in a browser view, the paragraph with the addresses' words comes directly before the area and carries `look::CONNECT_TITLE`. A terminal view has no such paragraph.
  - A search with no match says `Nothing here matches “…”.`, its query trimmed, and shows no count.
- `temper/crates/temper-client-view/src/tests/settings.rs`: the flows' paths table, at `for (address, tab, to, here)`, gains a row each for a plan's sign-in, a program and a key. It expects the back link and last trail link to be "Connect a model" and the plain last part to be the way's name. Give the table the column this needs.

Break a rule once by hand for each new test or row family, see it fail for the rule's reason, restore it, and name it in the report.

Then, one command at a time:

- `heavy cargo nextest run -p temper-client-shell-web --test suite -E 'test(=NAME)'` for the new test, and the narrowest test for each other change.
- `cargo fmt`, then `python3 -I -B temper/scripts/old_forms.py`.
- `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-world --all-targets -- -D warnings`.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-world -p temper-client-gallery -p temper-client-journeys`. `Count` and `Word::NoneFound` change meaning, so this set is also their readers' tests.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

Item 13's width and every other rendered value are read from the rules, not measured. temper/ui's render check follows landing.

### 6. Commits and the account

- **Trailers,** one per line above the other trailers:
  - The repairs of the request's sections 2 and 3: `Look-request: connect-flows`.
  - Section 4: `Look-request: model-picker`.
  - The chevron and the last commit: both.
- **The last commit** deletes the status entries and carries the account in its message: one table, `| Item | Result |`, as `temper/docs/design/draft/look-requests.md`, 5.
  - One row per bullet of the request's sections 1 to 4: landed, or differs and why. `rows=4` differs: a `min-height`, the same box.
  - A row for the frame differences left alone: the Settings page's H1 and the project menu's open row.
  - A row for the report's second table standing.
  - A row for `model-catalogue-3`: whichever lands second catches up.
  - A row for the render check: it follows landing, owned by temper/ui, and compiling and print checks establish no rendered equality.
- Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- each reader file you edited beyond the header's;
- any status entry that stands;
- whether a print changed.

Put under `## Left` what the rules alone could not settle. Name no browser run: none is to be made.
