+++
id = "rename-row"
part = "rename-row"
kind = "feature"
needs = ["focus-target"]
files = [
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/frame.rs",
  "temper/crates/temper-client-view/src/tests/rail.rs",
  "temper/crates/temper-client-domain/src/pages/switcher.rs",
  "temper/crates/temper-client-domain/src/tests/pages/switcher.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/main.rs",
  "temper/crates/temper-client-shell-web/tests/suite/switcher.rs",
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 75
+++

# While renaming, the open project's row is the form

## Goal

Match the mock's rename (`greenfield/app.js` 229-240 and 1136; `mock.css` 291-296), the first half of `inputs/request.md`: the open project's row becomes a name field filling the row and a primary small Save, "Rename NAME" leaves the menu's acts, Enter saves, Escape cancels, an empty name keeps the old one, no visible Cancel. Then prove the request's repro of the stray ring gone end to end, on `focus-target`'s skein.

Acceptance:

- While renaming, in the banner and the rail, in both shells, the open project's row is the form: no pressed row beside it, and in the rail no "Rename NAME" among the acts.
- In the browser the form shows the field (min-height 34px, filling the row) and Save (`.btn.primary.small`'s look), and no Cancel. The terminal still shows its Cancel, since the text shell ignores shortcuts (skein's `ui.md`, 6.1).
- Enter or Save with a new name sends it, without spaces at its ends. Escape closes and sends nothing. Save with an empty name, spaces alone or the name unchanged closes and sends nothing, keeping the old name. A refusal stays in the form, under the field.
- In a real browser: Home, project menu, Rename NAME, then Escape, and again then Save: the focus is on the menu's summary, never the main; `main` draws no outline. The field, Save and the summary keep their ring.
- Opening the form on a page that also asks the focus does not panic (`Builder::focus`'s at most one per tree).
- `temper.css` 119-121's old rules are gone, nothing replaced stays, and no rule for this sits at the end of the file.
- Each fix had its test red first. Proof at 1440, 1000 and 420, each beside the mock.
- `README.status.md`'s **Not yet** entry for 5.3 is gone; its **Design departs** entry for this request stays.

## Inputs

- `temper/docs/design/client/README.md`, 5.3, *Rename* and *Its state*.
- `temper/docs/design/client/greenfield/app.js` 229-240, 981 and 1136; `mock.css` 90-96, 240 and 291-296.
- `skein/docs/design/ui.md`, 6.1 (*Escape*, what a shell with no keys does); `skein/docs/design/ui-web.md`, 7.
- `temper/docs/plan/ui-rename-focus/notes/common.md`, its departures and *What crosses the two tasks*.
- `temper/docs/design/client/browser.md`, 2.5, for how the sheets are laid out.

## Owns and may change

Owns, as `files` lists them. `tests/suite/switcher.rs` is new; `main.rs` gains only its `mod switcher;`, so that the plans appending to `main.rs` merge cleanly.

May also change, for this alone:

- `temper/crates/temper-client-view/src/pages/{chat,document,settings,ask}.rs`: one `root.pages().switcher().renaming().is_none()` guard on each `b.focus`, as `pages/new_chat.rs` has.
- `temper/crates/temper-client-view/src/limits.rs`: a derived count your change moves, which isn't a budget.
- `temper/docs/design/client/journeys/**`: reprinted with `temper-client-journeys`, should a print change; no print holds the form today, so one that changes is a red to explain first.

Stays out: `temper-client-shell-web/src/**`, which `ui-final-walk/fix-flow` holds; `lib.rs`'s `finish` needs no change, the sheet's rule covers it. No new `look.rs` token and no new `Word`: `PROJECT_RENAME_FORM` and the field's name "Rename" stay.

Options, each pre-approved:

- **A focus guard's page you can't reach with a test:** leave that site, and say why in the report.
- **A rule of `temper.css` that still rings a focus target** (`#app > :is(.temper-welcome, .temper-home) .temper-welcome-sheet :focus-visible` is the one to read): add `:not([tabindex="-1"])` to it. `.temper-main:focus-visible` in the selector list at line 143 and `.temper-doc-sec:focus { outline: none; }` at 1284 are then redundant: remove them, if the proof holds.
- **A demo box whose person owns no project,** for the browser story below: use the tier that has one; with none, the proof carries it and the report says so.
- **The menu at 1000:** where it would clip the form, take the mock's behaviour at that width.

## Steps

1. **Reproduce** the request's repro on a fake box, as `focus-target` left it: see the form's look beside the mock. `temper/docs/plan/ui-final-walk/driver/` (on `origin/ui-final-walk/integration` if not yet on your base) shows how to bring up a box and Chromium with `--fake-backends`; the request's *Render proofs* line binds a 420 proof.
2. **The domain,** `pages/switcher.rs`, `Act::Save`, red first in `tests/pages/switcher.rs` (`an_empty_or_unchanged_name_sends_nothing` becomes its closing form, with a spaces-alone row and a row that sends trimmed words):

   ```rust
   let words = self.rename_words.trim_ascii();
   if words.is_empty() || words == row.name.bytes() {
       self.close();
       return;
   }
   let name = c::Text::new(words.into(), NAME_BYTES).expect("bounded rename words");
   ```

   Update `Act::Cancel`'s and `Act::Save`'s doc lines. Its readers' tests: `temper-client-world`.
3. **The view,** `frame.rs`, `switcher`, red first in `tests/rail.rs` (`the_panel_puts_rename_its_form_new_and_settings_after_projects`: renaming, the panel is `[Form, Button, Slot]` and the acts `[Form, Link]`, the first form "Rename Shop", in both shells) and `tests/frame.rs`:

   ```rust
   // In the rows' loop, both places: the open project's row is the form.
   if selected && root.pages().switcher().renaming() == Some(row.project) {
       rename_form(builder, root, words, &name, bindings);
       continue;
   }
   // The row's button; then in the banner, Rename where it is offered.
   // The rail's acts: "Rename NAME" where offered and not renaming.
   ```

   `rename_form` keeps its nodes and their order: field, refusal, Save, Cancel naming Escape. `the_rename_form_binds_its_field_save_and_cancel`, `a_pending_rename_disables_save` and `a_refused_rename_keeps_the_words_and_shows_why` stand; `assert_rail` follows the shape.
4. **The focus guards,** red first: a view test that opens Rename on a chat page whose `composer_focus` moved, in the shape of `tests/new_chat.rs`'s `opening_focuses_the_entry_except_while_renaming`, and on the other three sites as you can reach them.
5. **The sheet,** `temper.css`: remove `.temper-project-rename-form`'s rules at 119-121 and correct the comment at 113; keep line 112's hidden label. A new marker `/* ==== ui: switcher ==== */` just above `/* ==== ui: home ==== */`, holding one set of rules for both places; `--gf-*` is set only on the greenfield pages' frames, not always a banner's, so each `var` carries the mock's value as its fallback. To be tuned against the mock:

   ```css
   .temper-project-rename-form { display: flex; flex-wrap: wrap; align-items: center; gap: 6px; padding: 4px; }
   .temper-project-rename-form > input { flex: 1; min-width: 0; min-height: 34px; padding: 6px 12px; border: 1px solid var(--gf-line-2, #d4d4cf); border-radius: 8px; background: var(--gf-surface, #fff); color: var(--gf-ink, #18181b); }
   .temper-project-rename-form > button { flex: none; min-height: 34px; padding: 5px 12px; border: 1px solid var(--gf-ink, #18181b); border-radius: 8px; background: var(--gf-ink, #18181b); color: #fff; font-size: 13.5px; font-weight: 500; }
   .temper-project-rename-form > button:hover { background: #000; }
   .temper-project-rename-form > button:disabled { opacity: .45; cursor: default; }
   /* Cancel is Escape's alone here: the page part presses it from the tree. */
   .temper-project-rename-form > [aria-keyshortcuts=Escape] { display: none; }
   .temper-project-rename-form > [role=status] { flex: 0 0 100%; order: 1; }
   ```

   The field sets no `outline`: a real control keeps its ring. `tests/suite/switcher.rs`, in the idiom of `main.rs`'s `include_str!` tests, pins the hidden Cancel, the field's min-height and the old rules' absence.
6. **The story in real Chromium,** `browser_pages.rs`, on `Step::Focused` as `history_keeps_the_chat_focus_and_filter_across_reload_and_back` uses it: Home, press the project's disclosure, press "Rename NAME", `Step::Key(Key::Escape)`, `Step::Focused` on that disclosure, a `Step::Keep` of `main:not(:focus)`; press Rename again, type a new name, Enter, see it, and the same two checks. Run it alone with `heavy tools/tiers/run temper browser -- -E 'test(=NAME)'`.
7. **The proof:** a throwaway CDP script on a fake box, at 1440, 1000 and 420: the menu open, the form open, a refusal under the field, and after Escape and after Save; each beside the mock's `home.html` at that width. At each width read and report `document.activeElement` and `getComputedStyle(main).outlineStyle` after closing.
8. **Check,** the implementer's text, and because Save means something new: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world -p temper-client-gallery -p temper-client-journeys`. `python3 -I -B temper/scripts/old_forms.py`. `python3 -I -B tools/citations/citations.py --base origin/main`. Push.

## Report

The implementer's report, with: the red each test showed; which focus guards were reached; the proof's screenshot paths and the values read at each width; what still differs from the mock, by the status file's list or beyond it.

## Answers

2026-11-06 (fabrig/ui). Addition, a third fix the user asked for (via fabrig/temper): a chat shows two scroll bars, the browser's and the conversation pane's, after pressing "Older" twice in a long chat at a laptop's window size. A reader's cause, checked on the user's recorded page at 1280x720 and 1000x600 but not by this plan: the agent message's who-line (div.temper-who, "Temper 00:26") is hidden with position: absolute and no ancestor up to body is positioned, so a who-line below the pane's visible part isn't clipped by the pane's overflow: auto and extends the document (671px in a 633px window). Fix: #app > .temper-session [role=log] { position: relative; }, which brought the page back to 633px. Any other visually hidden absolute part inside a scrolling pane has the same fault: grep for them and fix each, including the draft pane. Test: a long chat's document never exceeds its window at 1280x720 and 1000x600 with Older pressed twice.
