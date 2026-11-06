+++
id = "board-words"
part = "board-words"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/public/pages.rs",
  "temper/crates/temper-client-view/src/public/forge.rs",
  "temper/crates/temper-client-view/src/tests/public.rs",
  "temper/crates/temper-client-view/src/tests/public/rules.rs",
  "temper/crates/temper-client-view/src/tests/public/records.rs",
  "temper/tests/client-browser/tests/browser_all/browser_public.rs",
  "temper/docs/design/client/browser.status.md",
]
effort = "medium"
minutes = 30
+++

# The board says what it doesn't hold

## Goal

A stranger reading "No goals yet." or "Nothing has landed yet." on a project with years of work is told something false.
Add the board's line of what it doesn't hold, change the two empty words, and pin that a queue merge's records, which
`queue-merges` makes, draw in a landing's form. The view's code for lines and landing pages does not change.

Acceptance:

- **The words, as `notes/contracts.md` has them, byte for byte:** the board's line under its existing lede, in
  `look::PUBLIC_LEDE`, whether or not the board has goals; `No goals to show yet.`; `No landings to show yet.` on the board
  and the landings page.
- **Where they don't go:** the Projects page, a task's page and a landing's page are as they were. `Word::GoalsNone` and the
  signed-in Goals page keep "No goals yet.".
- **A queue merge's form:** a test renders the records of `notes/contracts.md` and reads the line and the landing page there.
- **The browser story** compiles, and holds for an empty page or a merge. It is not run.
- **The geometry story's measuring is untouched** (`notes/common.md`).

## Inputs

- `temper/docs/plan/ui-public-history/notes/common.md` and `notes/contracts.md`.
- `temper/docs/design/client/browser.md`, 8.3 (*The board*, *The landings*) and 8.4.
- `jig/docs/design/domain/people.md`, 13.9 (*Ended goals since it was built*).

## Owns and may change

- **Owns:** `temper-client-view`'s `src/words.rs`, `src/public/pages.rs`, `src/public/forge.rs`, `src/tests/public.rs`,
  `src/tests/public/rules.rs` and `src/tests/public/records.rs`.
- **May change, and how:**
  - `temper/tests/client-browser/tests/browser_all/browser_public.rs`: only `check_page_fields`' `Page::Landings` arm, and a
    private helper for it. `red-50` is repairing this file's geometry story: touch nothing of it.
  - `temper/docs/design/client/browser.status.md`: **a departure from the implementer's text, which closes `docs/design/` to
    you:** when your acceptance holds, remove the `**Not yet:** 8.3: the board's line of what it doesn't hold ...` entry.
    Nothing else in `docs/design/`.
- **Not yours:** `temper-engine-domain-forge` and the wire (`queue-merges` owns the first; the second doesn't change).
- **Pre-approved:**
  - `PublicNothingLanded` no longer says its word, so rename it `PublicNoLandings`, at each of its uses in `words.rs`,
    `public/pages.rs` and `public/forge.rs`. If another tree's file names it, change that line too and say so.
  - The new word is `PublicBoardHeld`, next to `PublicBoardLede`. Take its number as the table's next free one and raise the
    `PHRASES` and `SEGMENTS` counts as their neighbours do. If a counted baseline or print goes stale, regenerate it by its
    tool and name it.
  - The `.temper-public-intro` flex column already spaces two paragraphs; add no CSS. If a throwaway render shows otherwise,
    put the rule by `.temper-public-lede` in `temper-client-shell-web/temper.css` and say so.

## Steps

1. **`words.rs`:** `PublicNoGoals`' and `PublicNoLandings`' texts; the new `PublicBoardHeld`, in its enum, its list and its
   number table, and with its text in the phrase table.

2. **`public/pages.rs`, `board`:** under the lede's paragraph, inside `PUBLIC_INTRO`:

   ```rust
   b.open(Role::Paragraph);
   b.style(crate::look::PUBLIC_LEDE);
   self.text(b, Word::PublicBoardHeld);
   b.close();
   ```

3. **View tests,** `src/tests/public/rules.rs` and `src/tests/public.rs`:
   - `the_board_says_what_it_is_and_when_it_has_no_goals`: the new words; the new line on an empty and a populated board,
     straight after the lede; not on the projects page.
   - `landings_show_below_the_goals_only_with_a_queue_or_a_landing`: `No landings to show yet.`
   - `the_view_shows_only_what_the_records_carry`: the new line in `allowed`.
   - **New,** `a_queue_merges_line_and_landing_read_as_a_landing`, with a `records::queue_merge()` fixture holding the
     contract's `PublicLanding` and `records::landings_with(false, true)`-style line with `turn: None`:
     the list's line reads `landed`, its task and its seven digits, both links to `Public::Landing`; the page's member row
     holds `Its change: 789ada1` and no gate row; "Holds the turn" is absent.

4. **The browser story,** `browser_public.rs`, `check_page_fields`, `Page::Landings`. The demo lands by the stock queue and
   the read may come before or after its merge, so the page holds either form:

   ```rust
   if matches!(self.page, Page::Landings) {
       // The demo lands by the stock queue: the merge shows once it has landed (temper/docs/design/client/browser.md, 8.3).
       let shown = self.text.join("");
       let merged = shown.starts_with("temperLandingslanded") && <its last seven characters are lowercase hexadecimal>;
       assert!(shown == "temperLandingsNo landings to show yet." || merged, "public landings page: {shown}");
   }
   ```

   `Page::Landings` must not read a queue or a turn; if the old assertion's exact form held something else out, keep it held
   out. Compile it, `heavy cargo check -p temper-client-browser-world --tests`. **Don't run it.**

5. **Each new test bites:** remove the board's new line and invert the merge's "no gate row" oracle once each; see them fail
   for that reason. The report names them.

6. **Checks:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p
   temper-client-browser-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view`, and its direct
   dependents that read public pages: `heavy cargo check -p temper-client-shell-web --tests` and
   `heavy cargo check -p temper-client-gallery --tests`.

7. `python3 -I -B tools/citations/citations.py --base origin/main`, then commit and push.

## Report

As the implementer's text asks. Name: the render check if you made one (a throwaway, gone before the push), the browser
story as compiled and not run, and any generated file regenerated.
