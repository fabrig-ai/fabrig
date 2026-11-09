+++
id = "differs-says"
part = "differs-says"
kind = "feature"
needs = []
files = ["skein/crates/skein-playground/src/replay.rs", "skein/crates/skein-ui-web/src/log.rs", "skein/crates/skein-ui-web/src/log/text.rs", "skein/crates/skein-ui-web/src/tests/log.rs", "skein/tests/playground/tests/suite/replay.rs", "skein/docs/design/playground.status.md"]
effort = "medium"
minutes = 60
+++

## Goal

A native replay that differs from its session log says what differed, so
the next failure of the browser story shows its cause without another
run (`skein/docs/design/playground.md`, 6.2, *What a difference says*):
`Differs` names the line's input, and a digest that differs gives the
log's digest, the replay's, and the replay's last lines before it, at
most 8, in the log's own text.

Acceptance:

- `skein_playground::replay::Differs` and `Why` have the shapes below;
  `replay` fills them as 6.2 says. Nothing else of the replay changes:
  where it steps, where it fails and the line it names stay as now.
- `skein_ui_web::log` gives the 6.1 texts the replay folds as public
  functions (below), and `log::Said`'s `Debug` prints its 64 lowercase
  hex digits, as the log writes a digest.
- `skein-playground-world`'s replay tests hold the new shape: each
  existing `Differs` assertion names the input and both digests, and one
  test holds the last lines (Steps, 4).
- The **Not yet** entry 6.2, *What a difference says*, in
  `skein/docs/design/playground.status.md` is removed, its lines alone,
  and the date line set to the day you land. This departs from the
  implementer's text, which closes `docs/design/` to you: the entry is
  this task's.

## Inputs

- `skein/docs/design/playground.md`, 6.1 (the log's texts) and 6.2 (the
  replay's loop; *What a difference says*, which this task builds).
- `temper/docs/plan/red-87-replay/inputs/request.md`, item 2.
- `temper/docs/plan/red-87-replay/notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- Reader files, unchanged unless they fail to compile (then the call
  sites alone): `temper/tests/client-browser/tests/browser_all/browser_playground.rs`,
  `temper/tests/playground/tests/suite/walk.rs`,
  `temper/crates/temper-playground/src/tests/world.rs`,
  `temper/tests/one-process/src/replay.rs`. Each prints a `Differs` with
  `{:#?}` or keeps it in an enum; the new fields print there as they are.
- Also owns a `#[cfg(test)]` test of the cut (Steps, 4) in
  `skein-playground`: `src/replay.rs`'s own `mod tests` by default, or
  `src/tests.rs` with its `mod` line in `lib.rs`.
- Not yours: the replay's loop (`Fold::until`'s stepping and its checks),
  `slices.rs`, the bridge, the log's version and its 6.1 texts
  themselves, any `docs/design/` but the status entry above.

## Steps

1. **The log's texts, public,** in `skein-ui-web`'s `log.rs`, each a
   thin wrapper over `log/text.rs`, which `text::entry` and `Saying`
   keep using:

   ```rust
   /// An input as its log line ends (playground.md, 6.1): `exchange id=…`,
   /// `abandon id=…`, `open path=…`, `close` or `read-on`.
   #[must_use]
   pub fn input_text(input: &bridge::Input) -> String;

   /// A fetch report's line as a digest folds it, its newline included (6.1).
   #[must_use]
   pub fn report_line(report: &fetch::Report) -> String;

   /// A printed line's line as a digest folds it, its newline included (6.1).
   #[must_use]
   pub fn printed_line(printed: &bridge::Printed) -> String;
   ```

   `text::entry` writes its input through the same function as
   `input_text`, so the two can't drift. `Said` drops `Debug` from its
   derive for an impl that writes `Said(<64 hex>)` through
   `text::digest`.

2. **The shapes,** in `skein-playground`'s `replay.rs`:

   ```rust
   /// Where a replay first differed from its log, and what differed (playground.md, 6.2).
   #[derive(Clone, Debug, PartialEq, Eq)]
   pub struct Differs {
       /// The log's line, from 1: the input's, or the last line's.
       pub line: u32,
       pub build: Option<Box<[u8]>>,
       /// The line's input in the log's text (`log::input_text`); none on the last line.
       pub input: Option<String>,
       pub why: Why,
   }

   #[derive(Clone, Debug, PartialEq, Eq)]
   pub enum Why {
       /// The world reached the stamp's step at another instant, or the instant at another step.
       Moment { expected: Stamp, found: Stamp },
       /// What the world said differs from the line's digest: the log's, the replay's, and
       /// the replay's last lines before it, oldest first.
       Said { expected: log::Said, found: log::Said, last: Vec<String> },
   }

   /// The most lines a `Why::Said` keeps, and the most bytes of each.
   pub const LAST_LINES: usize = 8;
   pub const LAST_LINE_BYTES: usize = 200;
   ```

3. **The last lines,** kept by `Fold` beside its `Saying`, at most
   `LAST_LINES`, the oldest dropped first (a `VecDeque<String>`), across
   inputs: never reset when an input enters. Each is, in the order the
   replay meets it:
   - an input it entered: `input step=<u64> instant=<ns> <input>`, its
     stamp in decimal and `log::input_text`;
   - a report it folded: `log::report_line` without its newline;
   - a printed line it folded: `log::printed_line` without its newline;
   - the stop, when it folds it: `stopped`.

   A line longer than `LAST_LINE_BYTES` keeps its first
   `LAST_LINE_BYTES` bytes and gains `...` (the texts are ASCII, so the
   cut is on a character). `Fold::until`'s digest check returns
   `Why::Said` with the entry's digest as `expected`, `self.saying.said()`
   as `found`, and the lines as they stand; `replay` sets `input` from the
   entry it is checking (none for `log::End::At`). Inside these shapes
   the code is yours: keep the loop's own lines as they are.

4. **The tests,** in `skein/tests/playground/tests/suite/replay.rs`:
   - Every existing assertion on a `Differs` keeps what it checks and
     gains the new fields: `input` is `Some(log::input_text(&entry.input))`
     for an input's line and `None` for the last line; a `Why::Said` names
     `expected` (the digest the test wrote, `log::Said([0; 32])`) and
     `found` (the digest recorded there before the test changed it). Its
     `last` is checked by its bound alone there (`len() <= LAST_LINES`);
     destructure rather than repeat the lines.
   - One new test, `a_said_difference_shows_its_last_lines`: for each
     seed, the recorded log with its end's digest zeroed (as
     `the_last_line_checks_its_moment_and_digest` does) differs with
     `last` holding exactly `LAST_LINES` lines (the site's 8 inputs, 4
     answers and 3 printed lines are more), every line at most
     `LAST_LINE_BYTES + 3` bytes, the last log entry's input line
     (`input step=… instant=… ` and its `input_text`) among them, and
     every line either an `input step=` line, one of
     `log::report_line`'s or `log::printed_line`'s texts for what the
     record loop heard (it already collects the reports; collect the
     printed lines beside them), or `stopped`.
   - The cut, in `skein-playground`'s own test of the helper that keeps
     the lines: a 300-byte line is kept as its first `LAST_LINE_BYTES`
     bytes and `...`; a line of exactly `LAST_LINE_BYTES` is kept whole;
     a ninth line drops the first. A longer input in the world's
     recorded log would change its steps, so the world's test doesn't
     carry it.
   - In `skein-ui-web`'s `src/tests/log.rs`, beside
     `the_text_is_the_designs`: each kept line of the golden log ends with
     `log::input_text` of its input, and `format!("{:?}", said)` is
     `Said(` and the 64 hex digits the text writes.

   Break each rule once by hand and see its test fail for that reason:
   keep 9 lines, then drop the cut, then reset the lines when an input
   enters. Restore, and name each in the report.

5. **Runs,** checking each count:
   - `heavy cargo nextest run -p skein-playground-world --test suite -E 'test(/^replay::/)'`
   - `heavy cargo nextest run -p skein-playground-world --test suite -E 'test(/^slices::/)'`
   - `heavy cargo nextest run -p skein-ui-web --lib -E 'test(/log::/)'`
   - the readers compile: `heavy cargo check -p temper-playground -p temper-playground-world -p temper-one-process-world -p temper-client-browser-world --tests`.
   The browser story is not run here.

6. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo
   clippy -p skein-playground -p skein-ui-web -p skein-playground-world
   --all-targets -- -D warnings`; `python3 -I -B
   tools/citations/citations.py --base origin/main` after committing;
   commit saying why (a replay that differs says what differed, so the
   browser story's next failure shows its cause); push.

## Report

As the implementer's text has it, and: the shapes as built; where the cut's
test sits; each rule broken by hand and the failure seen; the runs' counts. The
browser story is not run here: say so.
