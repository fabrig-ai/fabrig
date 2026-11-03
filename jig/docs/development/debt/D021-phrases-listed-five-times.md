# D021: Each of the view's phrases is listed four or five times

- **Kind:** duplication
- **Where:** `jig-client-view/src/words/phrases/jig.rs`, `jig-client-view/src/words/phrases/english.rs`, `jig-client-view/src/words`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T8, A9 F3: "tables would save about 6k lines")
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list ("Client view (A9)"; "one new outcome means editing the view's fixed-capacity phrase arrays")
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a phrase added in one place; the view's largest files shrink)
- **Size:** M
- **Status:** open
- **Paid when:** a phrase's identity, shape and English words are written once, in one table row, and the enum, `ALL`, `shapes` and `id` are derived from it.
- **Measure:** `wc -l < jig/crates/jig-client-view/src/words/phrases/jig.rs` (3,075 at base)

## What

`JigPhrase` has 647 variants (`jig.rs`, line 8), then the same names again in
`ALL: [JigPhrase; 647]` (line 2418), in `id` (line 1074), in `shapes` (line 3069) and in
English's `language` (`english.rs`, 1,630 lines). The words modules beside them
(`words/*_words.rs`, 12,289 lines in `jig-client-view/src/words`) add per-family lists.

## Why it matters

A new phrase is five edits in two 2,000-line files, and the fixed-capacity arrays must be
resized by hand.

## What pays it back

One table (a const array of rows, or generated from a phrase list), from which the
enum's order, the shapes and the language are read.
