+++
id = "growing-reader"
part = "reader"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/markdown.rs",
  "skein/crates/skein-ui/src/tests/markdown.rs",
  "skein/crates/skein-ui/src/tests/markdown/**",
  "skein/docs/design/ui.status.md",
]
effort = "high"
minutes = 90
+++

# Skein's Markdown reader reads a text still being written

## Goal

A long answer is read as it arrives, so its last block is usually unfinished. `read` shows a delimiter never closed as the characters it is, which is right for finished text and is the raw markers the user saw. Add `read_growing`, which reads the last block as if it were closed, and leaves every closed block exactly as `read` reads it.

Acceptance:

- `read_growing` is public in `skein_ui::markdown`, with the signature in `notes/contracts.md`, and each case of `skein/docs/design/ui.md`, section 10, *text still being written*, has a test that was red first.
- For every text kept beside the tests (`skein/crates/skein-ui/src/tests/markdown/*.md`), at every blank line outside a fence, the nodes `read_growing` makes for the text before that line, `end`'s aside, are the nodes `read` makes for it.
- For every such text cut at **every** byte on a UTF-8 boundary: nothing panics, and the node count stays within `max_nodes` of the bytes read, plus what `end` adds.
- A text that ends closed reads as `read` reads it, `end`'s nodes aside.
- `read` makes exactly what it made: its existing tests pass unchanged.
- `end` runs exactly once for a text holding a word, and never for an empty one or one holding only a marker still being written.
- The **Not yet** line `10: read_growing` is gone from `skein/docs/design/ui.status.md`.

## Inputs

- `skein/docs/design/ui.md`, section 10, all of it, and section 15 for how an owner counts the reader's nodes.
- `temper/docs/plan/chat-reading-a/notes/common.md`.
- `temper/docs/plan/chat-reading-a/notes/contracts.md`, `read_growing`.
- `skein/docs/foundation/programming-model.md`, 10: this is a step crate.

## Owns and may change

Owns the crate's Markdown reader and its tests, as `files` lists them. `skein/crates/skein-ui/src/tests/markdown/**` is read by tests: add a text there if a case needs one, and its test with it.

May also change: nothing else. No caller of the reader changes: `read` keeps its behaviour, and `temper-client-view` takes `read_growing` in a later task.

Options, each pre-approved:

- **Sharing the reader:** thread a private growing flag through `line` and `inline`, or split the last block's bytes and read it apart, whichever keeps `read`'s code plainer. If `read` would have to change what it makes, stop: that is a `Question:`.
- **A table's last row:** if placing `end` inside the last cell makes `row` much harder to read, place it after the table as its next sibling, and say so in your report.
- **A last line of digits:** a paragraph's last line that is only digits waits as a marker might, even where it turns out to be a number. It shows when the next byte arrives. Keep that.

## Steps

1. **Red first,** in `skein/crates/skein-ui/src/tests/markdown.rs`, in the file's idiom, one test a case:

   ```rust
   #[test]
   fn a_bold_still_open_is_bold() // "a **bold wo" reads a strong run "bold wo"
   #[test]
   fn a_delimiter_alone_at_the_end_makes_no_text() // "word **" and "word `" show "word "
   #[test]
   fn a_link_still_being_written_shows_its_label() // "[label](htt" and "[label" show "label"
   #[test]
   fn a_marker_being_written_waits_for_its_words() // "##", "## ", "-", "- ", "4", "4.", "4. ", ">", "> " make no node
   #[test]
   fn a_fence_still_open_is_code() // "```\nlet a" is a Code node holding "let a"
   #[test]
   fn a_fence_being_opened_or_closed_shows_no_backticks() // "```ru", and "```\nlet a\n``"
   #[test]
   fn a_delimiter_row_being_written_waits() // "| a | b |\n|--" shows the header line as a paragraph's
   #[test]
   fn the_end_stands_after_the_last_word_in_the_block_that_holds_it() // paragraph, heading, item, quote, code
   #[test]
   fn closed_blocks_read_as_read_reads_them() // the kept texts, at each blank line
   #[test]
   fn every_cut_of_the_kept_texts_stays_in_its_bounds() // the kept texts, at every boundary
   ```

2. **`read_growing`** beside `read`. Keep the step machine: one pass over the lines, no recursion. It allocates no more than `markdown_bound` prices.
3. **Fmt, clippy and tests:** `heavy cargo nextest run -p skein-ui`, and `heavy cargo clippy -p skein-ui --all-targets -- -D warnings`. The whole-cut test holds to the tier's budget: if it's slow, cut the kept texts' byte range, not the tier.
4. Commit and push.

## Report

As the implementer's text says, and: each case's red, the option you took for a table's last row, and the cost of the every-cut test.
