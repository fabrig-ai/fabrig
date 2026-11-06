+++
id = "no-figures"
part = "figures"
kind = "feature"
needs = []
files = [
  "temper/docs/help/how-temper-builds-temper.md",
  "temper/crates/temper-client-view/src/pages/reference.rs",
  "temper/crates/temper-client-view/src/look.rs",
  "temper/crates/temper-client-view/src/tests/reference.rs",
  "temper/crates/temper-client-view/src/tests/look.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-docs/src/tests/rules.rs",
  "temper/tools/gallery/src/tests.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 30
+++

# The reference has no figures

## Goal

The user, of "How temper builds temper": "just remove those, we'll use the site or people can go to the public facing version once in place". Section 6, "In numbers", leaves the page, and with it everything that exists only to draw it. No link, no placeholder, nothing in its place. Sections 1 to 5 keep their numbers. This cut has already moved the design; the task brings the page and the code to it.

Acceptance:

- `temper/docs/help/how-temper-builds-temper.md` ends on section 5's closing note, "Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change."
- The pane draws five sections, in both shells; section 5's closing note is still muted, the page's one muted note.
- `git grep -n -E 'In numbers|Figures to come|temper-doc-figures|DOC_FIGURES|placeholder_figures' -- ':!temper/docs/plan'` finds nothing.
- The two **Not yet** entries named below are gone from the status files.
- `python3 -I -B tools/citations/citations.py --all` finds nothing, run on your commit.
- A browser check: the gallery's `configure-reference` and `configure-reference-draft`, each at 1440 and at 420, show the pane, as step 5 says.
- No commit carries a `Look-request` trailer.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *temper's own way, beside the page*: *Its parts* and *The terminal*.
- `temper/docs/design/client/browser.md`, 2.5, *temper's own way*; 7.1, what the gallery prints.
- `temper/docs/design/draft/own-way.status.md`, for why.
- `temper/docs/plan/ui-no-figures/inputs/request.md`, *For every task of this plan*: its rules hold here, its render proofs above all.

## Owns and may change

Owns the files in the header, for the edits in the steps. Two of them depart from the implementer's text, which closes `docs/help/` and `docs/design/`:

- `temper/docs/help/how-temper-builds-temper.md`: removing section 6 is this task. The page is compiled into the client and read by tests, so it moves with them.
- `temper/docs/design/client/README.status.md` and `browser.status.md`: remove these entries and touch nothing else in them. The change closes them, so the task does:
  - `README.status.md`: **Not yet:** 3.5, *temper's own way, beside the page*: the pane still draws the stock page's section 6, ...
  - `browser.status.md`: **Not yet:** 2.5, *temper's own way*: `temper.css` still holds the rules of `.temper-doc-figures`, ...

Stay out of:

- The rest of `docs/design/`, which this cut has written.
- `temper/crates/temper-service/src/stock/configuring.txt`: read for the request; it names the page and no figures, and `model-catalogue-1` is changing it.
- `temper/tools/gallery/names.txt` and `samples.rs`: both states stay, under their names.
- `.temper-doc-flow`'s rules in `temper.css`, whatever `browser.status.md` says of them.
- `Role::Term` and `Role::Description`, and `look::DOC_MUTED`: others draw them.

Options, each pre-approved:

- **Generated files.** No print, golden or `docs/help/reference/*` holds the section or its class today: a journey's print links `temper.css` and doesn't copy it. So expect nothing to regenerate. If a test names one stale, regenerate it with its own tool and say so in the report.
- **A horizontal overflow at 420** in step 5: print the same state from the base commit. There too, it isn't this task's: name it under `## Left`. Only here, fix it, under the sheet's *The stock reference beside Configure*.

## Steps

1. **The page.** Remove `## 6. In numbers` to the end of the file. One newline ends it.

2. **The view,** `pages/reference.rs`. `figures` goes, `closing` loses its `numbers`, and section 5 alone goes through it:

   ```rust
   } else if section.heading.starts_with(b"5. ") {
       closing(b, section.bytes);
   } else {
   ```

   ```rust
   pub(crate) fn closing(b: &mut Builder, bytes: &[u8]) {
       let text = core::str::from_utf8(bytes).expect("stock page is UTF-8").trim_end();
       if let Some((body, note)) = text.rsplit_once("\n\n")
           && !note.is_empty()
           && !note.starts_with(['#', '-', '|', '`'])
       {
           skein_ui::markdown::read(body.as_bytes(), 1, b);
           b.open(Role::Paragraph);
           b.style(look::DOC_MUTED);
           b.text(Run::Plain, note.as_bytes());
           b.close();
       } else {
           skein_ui::markdown::read(bytes, 1, b);
       }
   }
   ```

3. **The look.**
   - `look.rs`: the const `DOC_FIGURES` goes. `NAMES` is positional, so its name becomes `b"temper-reserved-455"`, as its neighbours 456 to 459 are reserved. No token moves.
   - `tests/look.rs`: the row `(DOC_FIGURES, b"temper-doc-figures")` goes, from `configuring_opening_and_needs_have_their_css_names`.
   - `temper.css`: the four `.temper-doc-figures` rules go, where they stand, under *The stock reference beside Configure*.

4. **The tests.**
   - `temper-client-view`'s `tests/reference.rs`:
     - `SECTION_HEADINGS` becomes `[&str; 5]`, without `"6. In numbers"`.
     - `MUTED_NOTES` is 1, its comment "Section 5 ends in a closing note."
     - In `opening_draws_the_stock_reference_in_both_shells`, `parts` has five `Role::Slot`s. The block from `let figures = styled(tree, look::DOC_FIGURES);` to the end of its `for` goes. The muted texts are the table's note alone.
     - In `terminal_prints_reference_after_composer_in_section_order`, `SECTION_HEADINGS[5]` and `"Figures to come."` give way to `"Each row is a setting, unless it says otherwise."`, the last words printed in order.
     - `incomplete_figure_sections_are_read_whole` becomes `a_closing_section_without_a_note_is_read_whole`: its first row alone, the table with no note, read whole with nothing muted and "Yours" kept. It no longer needs a table of rows, or the assertion on `DOC_FIGURES`.
   - `temper-docs`' `tests/rules.rs`: `the_way_shows_placeholder_figures_and_no_address` becomes `the_way_names_no_address`, its `://` assertion alone. That rule outlives the figures (`AGENTS.md`, section 2).
   - `temper/tools/gallery/src/tests.rs`: in the `Holds` of `configure-reference` and `configure-reference-draft`, the `Part::Text` becomes the note whole, `"Each row is a setting, unless it says otherwise. Tell the chat what to keep, and what to change."`. `has_text` compares a whole run, so a part of the sentence fails.

   No new test pins a new rule, so no bite is due: `sections.len()` against `SECTION_HEADINGS` is what keeps a sixth section out, and it stands.

   Narrow first: `heavy cargo nextest run -p temper-client-view --lib -E 'test(reference) | test(look)'`, then `heavy cargo nextest run -p temper-docs`.

5. **The browser check.** A gallery state is enough: `samples.rs` steps these two by the person's `Act::ShowReference`, as a press would.
   - `heavy cargo run -p temper-client-gallery -- ~/src/fabrig/ui-no-figures/gallery`, outside any worktree.
   - Drive `chromium` over CDP, from a throwaway script, untracked and gone before the push. Open `OUT/configure-reference.html` and `OUT/configure-reference-draft.html`, each at 1440x900 and at 420x900, by `Emulation.setDeviceMetricsOverride`.
   - At each, read: `.temper-doc` has a non-empty box; there are five `.temper-doc-sec`; the last one's text ends with the note; there is no `.temper-doc-figures`; `document.documentElement.scrollWidth` is at most `innerWidth`.
   - Screenshot each, with `.temper-doc` scrolled to its end, into `~/src/fabrig/ui-no-figures/`, and look at them: the pane must close on the note, with no hole or stray box under it.

6. **The checks.**
   - `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`.
   - `heavy cargo clippy -p temper-docs -p temper-client-view -p temper-client-gallery -p temper-client-shell-web --all-targets -- -D warnings`.
   - `heavy cargo nextest run -p temper-docs -p temper-client-view -p temper-client-gallery -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world -p temper-playground-world`.
   - This departs from the implementer's text on direct dependents: `temper-service`, `temper-engine-smith`, `temper-client-tools` and `temper` aren't run here. `temper_docs::Page` keeps its shape, none of them reads this page's sections (`temper-service`'s `tests/stock.rs` asks only that the page exists), and the trunk's end check runs them.

7. Commit, saying why; push; then run the acceptance's `git grep` and `citations.py --all` on what you pushed.

## Report

As the implementer's text asks. Give what step 5 read at each state and width, and the screenshots' paths. Name anything regenerated, or say nothing was.
