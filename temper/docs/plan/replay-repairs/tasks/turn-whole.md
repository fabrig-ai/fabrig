+++
id = "turn-whole"
part = "turn-input"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-protocol-link/src/limits.rs",
  "jig/crates/jig-client-protocol-link/src/translate/turn.rs",
  "jig/crates/jig-client-protocol-link/src/tests/reads.rs",
  "jig/crates/jig-client-protocol-link/src/tests/fixtures/link_reads.rs",
  "jig/crates/jig-client-protocol-link/src/tests/turn_text.rs",
  "jig/docs/design/client/domain/client.status.md",
]
effort = "high"
minutes = 50
+++

# A turn read alone keeps every byte of a call's input

## Goal

`temper ctl task turn` read a goal's call of about 34 KB as two pages, both `cut=true`, 32,549 bytes together, the second saying `bytes_left=0` and `read_on=null`. The engine and the wire carry the turn whole; the client's link decodes a turn read alone (`translate::turn::archive`) within `turn_text`, the live turns' shared 32,768-byte text budget (`temper/crates/temper-client-tools/src/limits.rs`), so the call's input is cut there, and the tools page the cut copy, truthfully by its own measure. `jig/docs/design/client/domain/client.md`, 10 now says a turn read alone is decoded whole. Give `archive` its own limits, as `notes/contracts.md` says, and leave a page's turns as they are. The cut is made in the link, so the step test is the link's.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4).

Acceptance:

1. Failing first: a `w::ReadValue::Turn` row in `each_discriminated_read_value_reaches_its_contract_variant` (`jig/crates/jig-client-protocol-link/src/tests/reads.rs`, rows from `tests/fixtures/link_reads.rs`) whose one call's input passes the fixture's `turn_text` (32 bytes) several times over and ends in a sentinel; its expected contract value holds that input whole, the sentinel last, and the turn not marked cut. Before the fix it fails on the input's cut prefix and `…`.
2. `the_single_turn_decoder_uses_the_archive_bound` passes unchanged, and an archive one byte past its byte bound is still `TooLarge` (`a_pages_archived_turn_is_decoded_at_its_document_bound_and_too_large_one_byte_past` and its siblings pass unchanged).
3. A page's archived turns keep `turn_text`: `translate::pages::carried_turn_fields` still decodes with `archived_turn()`, and the existing page tests pass unchanged; `a_pages_archived_turn_past_the_live_turns_field_is_decoded`'s comment says what's true now (a page's turn is bounded by `turn_text`, a turn read alone is whole).
4. The entry *10, 14: a turn read alone (`Read::Turn`) is decoded within the live turns' `turn_text`...* is gone from `jig/docs/design/client/domain/client.status.md`.
5. This changes what a turn read alone holds, so its readers' tests run: focused tests pass in `jig-client-protocol-link` (`--lib`), `jig-client-domain` (`--lib`), `temper-client-tools` (`--lib`), `temper-client-shell-cli` and `temper-client-world`.

## Inputs

- `temper/docs/plan/replay-repairs/notes/common.md`.
- `temper/docs/plan/replay-repairs/notes/contracts.md`, *A turn read alone*: it binds.
- `jig/docs/design/client/domain/client.md`, 10 (the turn's decoding, and the new sentences on a turn read alone) and 14 (`DomainLimits.turn_text`).
- `jig/crates/jig-protocol-transcript/src/lib.rs`: `translate`'s budget and its row caps, `Budget::bounded_text`.

## Owns and may change

**Owned:** the `files` above.

**Readers:** `translate::turn::archive`'s caller `translate/reads.rs` (unchanged: it passes `limits`); the doc line of `translate/pages.rs`'s `carried_turn_fields` if it no longer says what's true (one line, allowed). temper's tools render an archived turn's input and page it (`temper-client-tools/src/jig/render/turn.rs`, `render/document/turn_window.rs`); they need no change, since a whole input makes their `bytes_left` and `read_on` truthful, and their tests inject decoded turns.

**Options, each with its default:**

- smith's ceilings for a turn's messages and a message's blocks: whichever `jig_protocol_transcript` exposes (`smith::CEILINGS` through its `Limits`). If the link can't name them without linking smith itself, take them from `jig_protocol_transcript::Limits::new(..).smith` or ask the transcript crate for a constructor, a one-line `pub fn` there, and say so.
- `bytes * 2` past `u32`: `Limits::new` of the link is refused (`Unusable::Limits`), as other relations are; no real limits reach it.
- If the client's validity check (`Limits::valid`) or a heap test refuses the new budget: stop with `Question:`, the relation and its figures.
- A journey whose print shows a cut archived turn (`temper/docs/design/client/journeys/*/*.html`): reprint it with `heavy cargo run -p temper-client-journeys`, and say so.

## Steps

1. Acceptance 1's row; run it and see the cut: `heavy cargo nextest run -p jig-client-protocol-link --lib -E 'test(=tests::reads::each_discriminated_read_value_reaches_its_contract_variant)'`.
2. `Receiving::turn_read` and `archive`'s use of it; the row passes. Break it once (`archive` back on `archived_turn()`), see the row fail, restore.
3. Acceptance 3's comment, the status entry, acceptance 5.
4. `cargo fmt`, `python3 -I -B jig/scripts/old_forms.py`, `heavy cargo clippy -p jig-client-protocol-link --all-targets -- -D warnings`, commit and push.

## Report

As the implementer's text says; also where the smith ceilings come from.
