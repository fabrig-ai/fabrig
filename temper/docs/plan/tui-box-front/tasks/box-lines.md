+++
id = "box-lines"
part = "box-front"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-box/src/front.rs",
  "temper/crates/temper-box/src/lib.rs",
  "temper/crates/temper-box/src/tests.rs",
  "temper/crates/temper-box/src/tests/front.rs",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "medium"
minutes = 45
+++

# The box's lines to its TUI in front: their words and bytes

## Goal

With its TUI in front, the box tells the TUI its printed link and its log on
descriptor 4, one line each (`temper/docs/design/protocol/setup.md`, 7.6,
*The box's lines*). This task writes temper-box's pure writer of those
lines, a module `front`, with nothing of the host: the tasks after it
(`box-front`, `front-lines`) send what it makes.

Acceptance, by setup.md, 7.6 and `notes/contracts.md`, section 6:

1. `temper/crates/temper-box/src/front.rs`, `pub mod front;` in `lib.rs`,
   holding exactly the public items of *Steps* below, step code under the
   crate's strict subset (no indexing, no unchecked arithmetic, no `as`, no
   `unwrap`, every `match` whole).
2. `Line::bytes` gives the word, one space, the field and one line feed;
   never more than `LINE_BYTES` (8,192) bytes; a field past the room cut at
   the last UTF-8 boundary that fits (a cut never ends inside a multi-byte
   sequence); a field holding a line feed never arises (`logs` splits at
   them) and is not checked.
3. `logs` gives one `Line::Log` per nonempty line of its bytes, in order,
   each without its line feed, a last line with no line feed included.
4. `state` gives `Restarting` for `Fact::Restarting`, `GaveUp` for
   `Fact::GivingUp`, each named by the fact's `step` as `steps` names it
   (`Placed::step.name()`), and `None` for every other fact.
5. `block_words` gives the first block's lines (`Formatter::block`'s
   stderr items) without the link's line and without blank lines, untimed,
   each ending in its line feed.
6. `Line`'s `Debug` writes a `Link`'s kind alone, never its URL.
7. The unit tests of *Steps* pass, each seen failing first.
8. The **Not yet** bullet naming `box-lines` in
   `temper/docs/design/protocol/setup.status.md` is deleted.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`;
  `notes/contracts.md`, section 6 (the lines) and section 5 (where they go).
- `temper/docs/design/protocol/setup.md`, 7.6, *The box's lines* and *The
  terminal is the front's*; 7.2, *The sign-in block* (the block's words),
  *Its terminal* (the timed lines), *A failure's tail*.
- The code: `temper/crates/temper-box/src/terminal.rs`: `Fact` (about line
  12), `Formatter::block` (about line 131, its items: `PRELUDE` on stderr,
  the link on stdout, the footer on stderr, or `FRESH` and the link once a
  block was admitted), `Formatter::notice` (about line 190: each line
  timed `HH:MM:SSZ `), `Tail::append` (about line 285: a failure's lines);
  `layout.rs`, `Placed` (its `step`); `lines.rs`, `joined`;
  `src/tests/terminal.rs` (the shape of a formatter's tests, `labelled`).

## Owns and may change

Owns `front.rs` and `src/tests/front.rs`, and one line each in `lib.rs`
(`pub mod front;`) and `src/tests.rs` (`mod front;`). Reads and does not
change `terminal.rs`, `layout.rs`, `lines.rs`, `host.rs`: no host change,
no caller of the new items in this task. The status edit of acceptance 8.

Options, each decided:

- **`logs`' return:** `skein_lib::List<Line>` (`List::uncapped`); if a
  `Box<[Line]>` reads better at its callers, use it and say so.
- **`block_words`' argument:** the URL without its line feed; it adds the
  line feed `Formatter::block` expects. Called for the first block alone,
  before `Formatter::admitted_block`: once a block was admitted it gives
  nothing but the fresh form's blank line, which it leaves out, so an
  empty result; document that on the function.

## Steps

1. The module, its head citing setup.md, 7.6:

   ```rust
   /// A line's most bytes, its line feed among them (setup.md, 7.6).
   pub const LINE_BYTES: usize = 8192;
   /// The most lines the box holds that its front has not taken (setup.md, 7.6).
   pub const HELD: u32 = 256;

   /// One of the box's lines to its front (setup.md, 7.6).
   #[derive(Clone, PartialEq, Eq)]
   pub enum Line {
       Link { url: Box<[u8]> },
       Log { text: Box<[u8]> },
       Restarting { step: Box<[u8]> },
       Running { step: Box<[u8]> },
       GaveUp { step: Box<[u8]> },
   }
   impl Line {
       /// `link`, `log`, `restarting`, `running` or `gave-up`.
       #[must_use]
       pub fn word(&self) -> &'static [u8];
       /// The word, a space, the field cut to fit at a UTF-8 boundary, and a line feed.
       #[must_use]
       pub fn bytes(&self) -> Box<[u8]>;
   }
   impl core::fmt::Debug for Line { /* Link { .. }; the others with their field */ }

   /// Each nonempty line of whole timed lines, as `Formatter::notice` and
   /// `Tail::append` make them, as a `log` line.
   #[must_use]
   pub fn logs(bytes: &[u8]) -> List<Line>;
   /// The line a fact gives the front beside its `log` lines.
   #[must_use]
   pub fn state(fact: Fact, steps: &[Placed]) -> Option<Line>;
   /// The first block's lines but its link and its blank lines, for box.log.
   #[must_use]
   pub fn block_words(formatter: &Formatter, url: &[u8]) -> Box<[u8]>;
   ```

   The cut: room is `LINE_BYTES - word - 2`; while the byte at the room
   (`field.get(room)`) is a continuation byte (`0b10xx_xxxx`), step the
   room back by one, with `checked_sub`; the field is `field.get(..room)`.
2. `src/tests/front.rs`, failing first, then the module:
   - `each_line_is_its_word_its_field_and_a_line_feed`, a table of
     `(Line, expected bytes)`: one row per kind (`link
     http://local/first?secret=fixture\n`, `log 00:00:01Z Stopping\n`,
     `restarting forgejo-3\n`, `running forgejo-3\n`, `gave-up
     forgejo-3\n`); a `log` whose field holds spaces, kept as is; a `log`
     field of exactly the room (8,187 bytes: 8,192 in all); one byte past
     it (cut to 8,187); and three rows each with a 2-, 3- and 4-byte
     character (`é`, `€`, `𝄞`) whose first byte sits at the field's last
     byte that fits, cut before it. Every row: at most `LINE_BYTES`, one
     line feed, last.
   - `items_and_facts_become_the_fronts_lines`: `logs` of one timed line,
     of a failure block (`01:02:03Z serve exited 1; restarting in 0.5 s`,
     `01:02:03Z     serve: boom`, `01:02:03Z     Log:
     /data/box/logs/serve.log`, three lines, in order), and of `a\n\nb`
     (two lines, `a` and `b`); `state` for every `Fact` variant over
     `labelled`'s steps (copy that helper's shape: one step `forgejo-3`),
     `Restarting` and `GivingUp` named, the seven others `None`;
     `block_words` with `LinkForm::Bare` and `LinkForm::Osc8`, word
     `temper`, URL `http://local/first?secret=fixture`: exactly
     `Open this link in a browser on this machine.\nIt signs you in as
     this temper's owner, once, within 2 hours.\nOpened after that, it
     prints a fresh one here.\nIn a terminal: temper http://local\n`, and
     never `secret`; and empty after `admitted_block`.
   - `a_links_debug_names_no_url`: `format!("{:?}", Line::Link { .. })`
     holds no byte of the URL; a `Log`'s holds its text.
   Break each rule by hand once (e.g. cut at the room without stepping
   back) and see its test fail; say which in the report.
3. Runs, through `heavy`, counts in the report:
   `heavy cargo nextest run -p temper-box --lib -E 'test(/tests::front::/)'`,
   then `heavy cargo nextest run -p temper-box --lib`.
4. The status edit (acceptance 8); `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-box
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main`.
5. Commit (why: the box's TUI in front reads the box's link and log as
   lines of a fixed shape, written in one place), with
   `Look-request: tui-box-front` directly above the other trailers; push.

## Report

As the implementer's text says: each test added, the rule broken by hand
for it and the failure seen, the runs' counts, and either option taken
otherwise than its default.
