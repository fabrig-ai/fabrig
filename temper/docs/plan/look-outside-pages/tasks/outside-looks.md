+++
id = "outside-looks"
part = "outside-looks"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-view/src/look.rs", "temper/crates/temper-client-view/src/outside.rs", "temper/crates/temper-client-view/src/lib.rs", "temper/crates/temper-client-view/src/root.rs", "temper/crates/temper-client-view/src/frame.rs", "temper/crates/temper-client-view/src/words.rs", "temper/crates/temper-client-view/src/tests/look.rs", "temper/crates/temper-client-view/src/tests/frame.rs", "temper/crates/temper-client-shell-web/temper.css", "temper/tools/gallery/src/tests.rs", "temper/tests/client-browser/src/lib.rs"]
effort = "high"
minutes = 70
+++

# look-outside-pages: the outside looks, the column, and the pages not about the work in it

## Goal

Give the client the looks every page outside a project builds on: the eight tokens, the
request's overlay rules in `temper.css`, and `outside.rs`'s column. Its first user is the
pages not about the work: signed out or out of date, the main becomes the column, the
link's state its level-1 heading and the rest of its words under it (the request's
`outside-pages/07.html`). Other tasks then draw the approval, the sign-in pages, the top
bar and no project yet on these.

Acceptance:

- `look.rs` has the eight constants of contracts, 1, on tokens 500 to 507, named there;
  nothing else of `NAMES` moves.
- `temper.css` holds every rule of the request's overlay block `outside-pages` (common.md
  says how to read it), selectors and declarations as the overlay has them, under a new
  marker `/* ==== ui: outside ==== */` placed just above `/* ==== ui: public ==== */`. No
  old rule is removed here.
- `outside.rs` exists with `column` and `words` (contracts, 3); `lib.rs` declares it.
- Signed out with no way of signing in, signed out with a link's `not_signed_in` reason,
  and out of date, in both shells, the main is `Main [OUTSIDE]`: `Heading { level: 1 }`
  "Signed out" or "Out of date", then one paragraph: the browser's `SignedOutMain`, the
  reason's paragraph as today, or `OutOfDateHow`; the terminal's signed-out column is its
  heading alone (contracts, 5). A task's reading paragraph, drawn there today after the
  words (`root.rs`, the `pages::task::reading` call), stays after them, inside the column.
- `SignedOutMain`'s browser words and the new `OutOfDateHow` are contracts, 5's; the
  TUI's words are unchanged.
- `heavy cargo nextest run -p temper-client-view`, `-p temper-client-gallery`, `-p
  temper-client-shell-web`, `-p temper-client-world` pass; `heavy cargo check -p
  temper-client-browser-world --tests` and `-p temper-client-tui --tests` compile.

## Inputs

- `temper/docs/design/client/README.md`, 5.1: *the main* and *The pages not about the
  work* (the column, the headings, the words).
- `temper/docs/design/client/browser.md`, 2.5, *Outside a project*; 2.4, *Signed out*.
- `temper/docs/plan/look-outside-pages/notes/contracts.md`, 1, 3 and 5;
  `temper/docs/plan/look-outside-pages/notes/common.md`.
- The request's `outside-pages/07.html` and its overlay block.

## Owns and may change

- Owns: `temper/crates/temper-client-view/src/outside.rs` (new), `look.rs` (the eight
  constants and their names only), the unavailable branch of `root.rs`'s `build` and
  `frame.rs`'s `unavailable_main` (which may move into `outside.rs`), the two words in
  `words.rs`, the marker and its rules in `temper.css`.
- May change, the module line only: `temper/crates/temper-client-view/src/lib.rs`.
- May change, assertions only, keeping what each checks for everything else:
  - `temper/crates/temper-client-view/src/tests/frame.rs`: the unavailable mains'
    tests (`signed_out_the_main_says_so_and_nothing_else`,
    `signed_out_the_browser_main_says_how_to_sign_in`,
    `a_full_link_says_the_pool_and_both_moments_in_either_shell`,
    `each_link_reason_has_its_paragraph_in_either_shell`,
    `out_of_date_the_browser_main_keeps_its_words`,
    `an_engine_not_serving_jigs_version_says_temper_is_out_of_date`,
    `terminal_with_google_keeps_its_signed_out_main`): each reads the heading, then the
    paragraph. The banner's own tests there are not yours.
  - `temper/crates/temper-client-view/src/tests/look.rs`:
    `each_pages_main_carries_the_main_token`'s signed-out and out-of-date rows give
    `[OUTSIDE]`; add the eight names as rows of `the_looks_names_pass_skeins_check`'s
    table.
  - `temper/tools/gallery/src/tests.rs`: the `signed-out` and `out-of-date` rows of
    `each_state_holds_what_its_name_says` (the heading, then the words).
  - `temper/tests/client-browser/src/lib.rs`: the `signed_out` finder's words (compile
    it, don't run it).
- Read, never change: `temper/crates/temper-client-tui` (it reads `SignedOutMain`'s and
  `OutOfDateMain`'s terminal words, unchanged); `sign_in.rs` (its use of `SignedOutMain`
  takes the new browser words as they come: if a test of it reads the old words, update
  that assertion and say so).
- Options, pre-approved:
  - If a client story (`temper/tests/client`) or a terminal golden reads the old
    paragraph, change the assertion to the heading and the new words and say so.
  - If `fixtures::main_text` joins the heading and paragraph so that a test reads
    awkwardly, compare the heading and the paragraph apart.
  - If an overlay rule needs a token no task gives, keep the rule and name it in the
    report.

## Steps

1. `look.rs`: the eight constants beside the `APPROVE_*` ones, each `Style { owner:
   owner(), token: N }`, a doc comment citing `client/README.md, 5.1` and `browser.md,
   2.5`; in `NAMES.styles`, slots 500 to 507's `temper-reserved-N` become the names.
2. `temper.css`: the marker and the overlay's `outside-pages` block below it, in its
   order.
3. `outside.rs`: `column(b, heading, extra)` and `words(b, words, word, holes)`, as
   contracts, 3 gives them.
4. `root.rs` and `frame.rs`: the unavailable main as the column (contracts, 3 and 5),
   the task's reading paragraph after the words as today.
5. `words.rs`: `SignedOutMain`'s browser words; `OutOfDateHow` beside `OutOfDateMain`.
6. Tests as above; one test must bite: break the heading's word and see the frame test
   fail.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; the
   focused runs of Acceptance; commit with the trailer (common.md), push.

## Report

As the implementer's text says. `## Account` rows: each overlay rule landed (by selector group: the
root's tokens and body, the top bar, the column, the acts, the quiet link, the form, the
approval's code and field, the phone width); 07.html's heading and words; the terminal's
column.
