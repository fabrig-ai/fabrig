+++
id = "tui-views"
part = "views"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-tui/src/view/printed.rs",
  "temper/crates/temper-client-tui/src/view/chat.rs",
  "temper/crates/temper-client-tui/src/view/chat/read.rs",
  "temper/crates/temper-client-tui/src/tests/chat.rs",
  "temper/crates/temper-client-tui/src/tests/printed.rs",
  "temper/crates/temper-client-tui/src/tests/limits.rs",
  "temper/crates/temper-client-tui/src/tests/golden/chat-views-80x24-dark.txt",
  "temper/crates/temper-client-tui/src/tests/golden/chat-views-80x24-light.txt",
  "temper/crates/temper-client-tui/src/tests/golden/chat-views-110x32-dark.txt",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 120
+++

# A view in a reply prints in the TUI's chat as lines of its own text

## Goal

Today the TUI's chat takes an agent's committed text straight to Markdown
(`view/chat/read.rs`, `c::Block::Text` → `Entry::Agent`), so a `view` block
shows as code holding its address. Split the text at its view blocks as the
browser does and print each view in its place, as lines of the terminal's
own text: a path a stop a line with its actor's mark, code's stops indented,
a branch's bar closing its part; a way, a proposal, a charter or rule card, a
`not_drawn`; "Drawing…" and "Couldn't draw…" while asking or failed. Nothing
to press.

Acceptance (`temper/docs/design/client/README.md`, 6.14, *A chat*, *a view*;
`notes/contracts.md`, section 1):

1. **`view/printed.rs`,** new (`pub(crate) mod printed;` in `view.rs`, one
   line beside `mod chat;`), with `Printed`, `read` and `draw` exactly as the
   contract has them:
   - `read(address, shown)`: `None` or `Some(Shown::Asking)` →
     `Printed::Asking(address)`, `Some(Shown::Failed)` → `Failed(address)`,
     `Some(Shown::Drawn(view))` → `Drawn(view.clone())`.
   - `draw`, by `w::View` variant, each listed (no `_`):
     - *a path* (`print_path(b, &w::BranchPath)` inside the module): a stop
       a paragraph, in `stops()` order; by `w::Actor`: `Person` → the mark
       `○` in `look::SIGNAL`, then " " and its words (no style); `Agent` →
       `✦` with no style, then " " and its words; `Code` → four columns in,
       `▪` and its words, both in `look::HINT`; `Bar` → four columns in,
       `┃` in `look::FAINT` and its words (the branch's name) with no
       style. The marks are the module's glyph constants (`const PERSON:
       &[u8] = "○".as_bytes();` and so on), cited to 6.14; the words are
       the wire's, never spelled.
     - *a way:* a paragraph of its `name()` (`look::NAME`) and, when
       `current()`, the view's `Word::ViewNow` (`look::HINT`); a paragraph
       of its `line()` (`look::HINT`); then its path, a blank row between
       the line and the first stop.
     - *a path* view (`w::View::Path`): its stops alone.
     - *a proposal:* `Word::ViewBefore` (`look::HEAD`), its `before()`
       path; a blank row; `Word::ViewAfter` (`look::HEAD`), its `after()`.
     - *a charter or a rule:* its `card()`: `name()` (`look::NAME`),
       `who()` (`look::HINT`), then each fact a `Role::Term` (`label()`)
       and `Role::Description` (`value()`), as `temper-client-view`'s
       `views::facts` lays them.
     - *a `not_drawn`:* its `words()`, a paragraph.
     - `Asking(address)`: the view's `Word::ViewAsking` with the address,
       in `look::HINT`; `Failed(address)`: `Word::ViewFailed`, no style.
     - No binding, no act, no drawing node.
2. **The chat's entries** (`view/chat.rs`, `view/chat/read.rs`): `Entry`
   gains `Viewed(Box<[Piece]>)` and `Piece { Text(Box<[u8]>),
   View(Printed) }` (the contract). In `committed`, an assistant's text
   whose `temper_client_domain::views::parts` holds a `Part::View` becomes
   `Entry::Viewed`, its parts in order, each view `printed::read(address,
   chat.view(address))`; text with no view block stays `Entry::Agent` as
   today; the provisional text (`growing`) is read as today. `entry()`
   draws `Viewed` as `Agent` is drawn, `look::AGENT` on the item and one
   group under `look::BULLETS`, each `Piece::Text` read by
   `skein_ui::markdown::read(bytes, 2, b)` and each `Piece::View` by
   `printed::draw`, in order. The UTF-8 borrow for `parts` is as
   `temper-client-view`'s `views::parts` makes it (a scoped
   `#[expect(clippy::disallowed_methods, reason = ...)]`).
3. **Tests,** a failing test first for each (`notes/common.md`):
   - `tests/printed.rs`, new (`mod printed;` in `tests.rs`), one table test
     `each_view_prints_its_lines`, a row per case, each drawing
     `printed::draw` into a tree and comparing its text and styles in
     order (the shape `tests/chat.rs`' rows use): a way (the request's
     *You approve* path: you ask, produce, check, repair, check, you
     approve, update, resolve, merge, main), the current way (" · now"),
     a path with a goal's bar in a stop's place, a path view, a proposal,
     a charter, a rule, a `not_drawn`, asking, failed. Build the wire
     values as `temper-client-view`'s `tests/views.rs` does (`card()`,
     `path()`, `way()`).
   - `tests/chat.rs`' `the_chat_draws_each_entry`: rows "a reply with a
     view between two paragraphs" (text, the printed way, text, in order,
     in one entry) and "a reply with a view still asking"; the root answers
     the view's read as `temper-client-view`'s `tests/views.rs`' `root`
     does. A row "provisional text with a view block" keeps the block as
     Markdown code.
   - `tests/chat.rs`' `the_chats_are_their_goldens`: rows
     `chat-views-80x24-dark`, `chat-views-80x24-light` and
     `chat-views-110x32-dark`: the request "How does a change land?", a
     reply "Under You approve:" then the way's view then "Nothing lands
     without you.", built as `Entry::Viewed` from data, the goldens
     written by hand from 6.14's illustration, then checked.
   - `tests/limits.rs`' `the_trees_fit_their_limits`: a row of a chat whose
     reply holds `temper_client_domain::Limits::chat_views()` views, each
     a proposal of two paths at `paths::STOPS` stops of
     `paths::WORDS`-byte words, fits `limits()`; if it doesn't, raise the
     TUI's tree limits in `limits.rs` by what the printed view takes, and
     say so (that file then joins this task's).
4. **Status:** delete the **Not yet** bullet naming `tui-views` in
   `temper/docs/design/client/README.status.md`. No other design line.
5. **Runs,** through `heavy`, counts in the report: `cargo nextest run -p
   temper-client-tui`; `cargo check -p temper-client-shell-tui --tests`.
   Every existing golden stays as it is.

## Inputs

- `temper/docs/plan/tui-whole-t1/notes/common.md`; `notes/contracts.md`,
  section 1; `inputs/request.md`, *Views printed (11.5)*.
- `temper/docs/design/client/README.md`, 6.14, *A chat* (the conversation's
  entries, *a view*), 6.15 (the tokens' table), 5.16 (*The block*, *Read
  when shown*, *Drawn*).
- `temper/docs/design/draft/surfaces.md`, 11.5 and 13 (the marks' colours).
- The code: `temper-client-tui`'s `view/chat.rs` (`Entry`, `entry`),
  `view/chat/read.rs` (`committed`); `temper-client-view`'s `views.rs`
  (`parts`, `read`, `drawn`, `path`, `card`, `facts`), `paths.rs` (`STOPS`,
  `WORDS`); `temper-client-domain`'s `views.rs` (`parts`, `Part`),
  `pages/chat/views.rs` (`Shown`, `Chat::view`);
  `temper_client_protocol_wire::v1`'s `View`, `WayView`, `BranchPath`,
  `Actor`, `ProposalView`, `Card`; tests `temper-client-view/src/tests/views.rs`
  (wire builders, `root`), `temper-client-tui/src/tests/chat.rs`,
  `tests/goldens.rs`, `tests/limits.rs`.

## Owns and may change

Owns the files listed. May add `pub(crate) mod printed;` to `view.rs` and
`mod printed;` to `tests.rs`, one line each, nothing else there: the
launchpad's tasks change `view.rs`, `words.rs`, `look.rs`, `theme.rs`,
`state.rs`, `view/launchpad.rs` and `view/foot*` in the same weeks. No new
word, token or theme entry (the contract). Not `temper-client-view` or
`temper-client-domain`: if `Chat::view` or `views::parts` lacks what the
reader needs, say so in the report and build on what is there.

Options, each decided:

- **Code's indentation:** four columns by a nested `Role::Description` (two
  columns each) where skein lays it so; else two columns of a Description
  and two spaces before the mark in the text. Say which; the goldens show
  four.
- **A blank row** inside the view where the design asks one: a `look::SPACED`
  group where it lays the rows right; else an empty paragraph. Say which.
- **A Term's punctuation:** as skein's screen lays a Term (it may add its
  own); the goldens show what it lays.

## Steps

1. The failing rows and tests, seen failing; then `printed.rs`, the reader
   and `entry`; the narrowest test after each, e.g. `heavy cargo nextest run
   -p temper-client-tui -E 'test(/printed::each_view_prints_its_lines/)'`.
2. The three goldens by hand from 6.14's illustration, then the golden test
   green.
3. The status edit; the runs of acceptance 5; `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui
   --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py
   --base origin/main`.
4. Commit (why: a view in a reply printed as the terminal's own lines, not
   its raw address); push.

## Report

As the implementer's text says, with each row added and the rule broken by
hand for it (for instance code's stop not indented, or a view block left as
Markdown), the indentation and blank-row options taken, and how the limits
row came out.
