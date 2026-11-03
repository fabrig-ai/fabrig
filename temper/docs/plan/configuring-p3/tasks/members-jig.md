+++
id = "members-jig"
part = "pages"
kind = "interface"
needs = ["limits-jig"]
files = ["jig/crates/jig-protocol-config/src/members.rs", "jig/crates/jig-protocol-config/src/lib.rs", "jig/crates/jig-protocol-config/src/read.rs", "jig/crates/jig-protocol-config/src/*.rs", "jig/crates/jig-protocol-config/src/llm/**", "jig/crates/jig-protocol-config/src/clients/**", "jig/crates/jig-protocol-config/src/tests/members.rs", "jig/crates/jig-protocol-config/src/tests.rs"]
effort = "high"
minutes = 150
+++

# members-jig: jig's document members by shape, beside their decoders

## Goal

jig gives every member of the configuration document's `engine` and a worker's shapes a row
of a members table (contract C4), its decoders reading their members through it, so that
temper renders them as the reference page `configuration-engine` (`configuration-pages`).
The members gone to the store (`engine.charters`, `templates`, `policies`, `rules`: temper's
`protocol/configuration.md`, 13.5) are rows saying where they went.

Acceptance:

- `jig_protocol_config::members` with C4's `Shape`, `Member`, `SHAPES`, `MEMBERS` and `names`,
  its module doc citing `jig/docs/design/protocol/configuration.md` and
  `temper/docs/design/protocol/documentation.md`, 8; a row for every member each of jig's
  decoders reads (`lib.rs`' `engine`, `acts.rs`, `defaults.rs`, `projects.rs`, `llm.rs` and
  `llm/`, `hosts.rs`, `clients.rs` and `clients/`, `domain.rs`, `charters.rs`, `templates.rs`,
  `worker.rs`, and the rest the reader reaches), each shape's `paths` every path it is read at.
- `read.rs`' object reader is given its shape where it is made and looks each name asked up in
  `MEMBERS` (C4), every decoder's construction site naming its shape; nothing a decoder takes
  or refuses changes.
- A gone member's row carries, as `gone`, where it went in the words its refusal gives (for
  an application that passes its own defaults); jig's bridge, which still reads them for an
  application that passes none, keeps its rows' types and lines too.
- Lines: one sentence each, at most 160 bytes, from the member's design text in jig's
  `protocol/configuration.md` or the field's doc comment, generic to jig's applications.
- Step tests in `src/tests/members.rs`: `every_row_is_a_member_its_decoder_takes` (a document
  holding each shape's rows, with values from the crate's existing fixtures, decodes with no
  `Unknown` at that shape), `a_member_a_decoder_asks_for_is_a_row` (the debug lookup, by a
  decoder of a document every fixture of the crate holds), `each_line_is_within_its_bound`.
- The focused tests of `jig-protocol-config` and every package that decodes a configuration
  document through it (`heavy cargo check --workspace --all-targets` clean; the focused tests
  of `jig-protocol-config`, `jig-test-service`, `temper-service`) pass.

## Inputs

- `jig/docs/design/protocol/configuration.md` (its shapes and members).
- `temper/docs/design/protocol/documentation.md`, 8.
- `temper/docs/design/protocol/configuration.md`, 13.5 (the members gone).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C4.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files` (the decoders' modules, for each reader's shape
  name and nothing else of them; `overrides.rs` is `limits-jig`'s and is left as it is).
- This crate is also changed by `public-reading-build` on its trunk: keep each decoder's hunk
  to its reader's construction line.
- Options, pre-approved:
  - If the reader is made in one place and handed down, give the shape name where each nested
    object's reader is made, and say how.
  - A member with no design words and no doc comment: write its line from what its decoder
    sets, and list it.

## Steps

1. `members.rs`: the types, then the rows shape by shape, from each decoder.
2. The reader's shape and lookup; each construction site.
3. The tests.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-config --all-targets -- -D warnings`; the focused tests above; the workspace
   check.
5. Commit ("jig-protocol-config: the document's members by shape, beside their decoders
   (documentation.md, 8)") and push.

## Report

As the implementer's text says, and the table's length by shape.
