# D001: smith-events keeps every record in four hand-written hierarchies

- **Kind:** duplication
- **Where:** `smith-events/src/records.rs`, `smith-events/src/write.rs`, `smith-events/src/read.rs`, `smith-events/src/limits.rs`
- **Seen:** 2026-10-31 at `aa95945cc`, in the smith code-quality review (`reviews/2026-10-31-quality.md`: theme "`smith-events` is five hand-kept hierarchies"; `reviews/2026-10-31-quality-areas.md`: report 04, finding 1)
- **Seen:** 2026-10-31 at `679eacb3c`, in the quality run's closing report (outside the repository): helpers taken, the JSON-line generator left out
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (a new field costs one edit, not one per hierarchy; sizing stops being hand arithmetic)
- **Size:** L
- **Status:** open
- **Paid when:** each event record is declared once, and its writer, reader, validator and size bounds come from that declaration (generated, or one table per record), so adding a field touches one place and its fixtures.
- **Measure:** `cat smith/crates/smith-events/src/records.rs smith/crates/smith-events/src/read.rs smith/crates/smith-events/src/write.rs smith/crates/smith-events/src/limits.rs | wc -l` is 5,372

## What

`smith-events` writes the event stream as JSON lines. Each record exists
by hand four times: its type in `records.rs` (1,145 lines, 82 public types),
its writer in `write.rs` (1,612 lines, 52 `write_`/`encode_` functions and
37 validators), its reader in `read.rs` (1,625 lines, 84 `read_`/`decode_`
functions) and its size bounds in `limits.rs` (990 lines, 61 size
functions). The fixtures repeat it again.

The quality run shared the word spellings (`spelling.rs`, one `spelled`
match per vocabulary) and one sample and maximum builder (`Fill`), and cut
`read.rs` from 1,917 lines. It left the generator out: the schema would need
word spellings and forward values, absent against null fields, object-shaped
lists, capture selection, and bounds per line.

## Why it matters

A new field, or a changed bound, is several hand edits that must agree.
`read.rs` and `write.rs` are each over 1,600 lines. The wire formats beside
it show the other way: three `.schema` files generate smith's binary codecs,
held together by their drift tests.

## What pays it back

Either a JSON-line backend for skein's codec generator, declared from the
records, or one table per record that the four walks read. The first is the
larger change and needs the schema features listed above; decide it before
starting. Neither changes the event stream's bytes, which its goldens check.
