+++
id = "tui-lines"
part = "tui-journeys"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-shell-tui/Cargo.toml", "temper/crates/temper-client-shell-tui/src/journey.rs", "temper/crates/temper-client-shell-tui/src/journey/line.rs", "temper/crates/temper-client-shell-tui/src/journey/line/tests.rs"]
effort = "high"
minutes = 120
+++

## Goal

The TUI's recording file as lines (`temper/docs/design/client/tui/journeys.md`, 3.2): `temper-client-shell-tui`'s `journey` module, built with the crate's new `record` feature and in its tests, starts with its strict byte codec, every record of 3.2's table written and read back, on the browser's syntax (`temper/docs/design/client/journeys.md`, 3.2). It is the record layout the recorder, the replay, the redaction, the print and the walk all build on.

Acceptance:

- `temper-client-shell-tui` has a feature `record` (no dependency yet), and `src/lib.rs` declares `#[cfg(any(test, feature = "record"))] pub mod journey;`. With the feature off, the crate builds as today.
- Contracts, section 5, holds as written in `src/journey/line.rs`, re-exported from `src/journey.rs`: `Line`, `Streamed`, `Sent` (with `Sent::of`), `Unreadable`, `Line::write`, `Line::kind`, `lines`.
- The text is 3.2's exactly: a record's kind, then its fields, each after one space; numbers decimal; words bare; bytes in double quotes with `\\`, `\"` and `\xHH` (two upper-case hex digits) for a byte outside 0x20 to 0x7e; one `\n` after each line; no other line. Times and walls are nanoseconds; an exchange `Token::raw()`; a size rows then columns; a scheme `dark` or `light`; `none` where 3.2 says so.
- A key is 3.2's: the bare words for the keys with no text, and `char`, `paste`, `control`, `alt` with their bytes; a `char` or `alt` field holds exactly one UTF-8 character and a `control` field one ASCII letter, or the line is unreadable.
- A message's kind is 3.2's kebab-case word; `sign-in-service` and `sign-out` carry `""` and refuse other bytes.
- Tests in `src/journey/line/tests.rs`, one table: every record of 3.2 written and read back to itself, every key and every message kind among them, bytes with each escape (`\\`, `\"`, `\x00`, `\x7F`, `\xFF`) and a UTF-8 character outside ASCII; and unreadable rows naming their line: another version (`journey 3`), an unknown kind, an unknown word in a field, a field missing, a field over, a number past its type, an unterminated quote, a bad escape, a blank line, a `char` of two characters, a body on `sign-out`.
- `heavy cargo nextest run -p temper-client-shell-tui` passes; `heavy cargo check -p temper-client-shell-tui --features record`; clippy clean with and without the feature.

## Inputs

- `temper/docs/design/client/tui/journeys.md`, 3.2 (the table, a key, a message's kind, unreadable) and 3.3.
- `temper/docs/design/client/journeys.md`, 3.2 (the shared syntax).
- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 5 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code to read for the syntax, never to depend on: `temper/crates/temper-client-shell-web/src/journey/line.rs` (`Text`, its reader, `Unreadable`). The types: `skein_ui_screen_terminal::Key`, `skein_ui_screen::Size`, `temper_client_tui::Scheme`, `jig_client::Message` (`jig/crates/jig-client-protocol-link/src/boundary.rs`), `skein_lib::{Time, Wall, Token}`.

## Owns and may change

- Owned: the files above, and the one `mod` line in `src/lib.rs` (`approval-host` edits `lib.rs` for its `opener` module at the same time: keep your hunk to your line).
- Not yours: `host.rs`, `run.rs` and their tests (`approval-host`'s). The module reads no host type yet; the recorder and replay come in `tui-replay`.
- Options:
  - The codec's own writer and reader in `line.rs` (default), copying the browser's syntax rules; never a dependency on `temper-client-shell-web` here (`tui-redact` adds that under `record`).
  - `Line`'s `Debug`: none (default), or a hand-written one printing kinds and lengths alone.
  - The unreadable rows' `why` texts are yours, short and naming no bytes.

## Steps

1. Read the inputs. Write `Line` and its parts, `write`, `kind`, then `lines`.
2. The table, then the unreadable rows. Break a rule once by hand (an escape written in lower-case hex; a reader that takes `journey 3`), see a row fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-tui --all-targets -- -D warnings`, and again with `--features record`; `heavy cargo nextest run -p temper-client-shell-tui`.
4. Commit and push.

## Report

The implementer's report, with: the rows; each broken rule's failure; any record of 3.2 whose fields were unclear and how you read them.
