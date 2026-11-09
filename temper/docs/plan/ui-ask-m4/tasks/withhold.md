+++
id = "withhold"
part = "withhold"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-transcript/src/lib.rs", "jig/crates/jig-protocol-transcript/src/withhold.rs", "jig/crates/jig-protocol-transcript/src/tests.rs", "jig/crates/jig-protocol-transcript/src/tests/**"]
effort = "high"
minutes = 120
+++

## Goal

Give `jig-protocol-transcript` the projection that withholds a turn's
private parts from every party but its task's requester
(`jig/docs/design/domain/engine.md`, 10.1, *Its runs' parts are its
party's*): a pure function over smith's turn bytes, which the clients'
translation and temper's `see` call on a turn the core marks withheld.
Nothing calls it yet.

Acceptance:

- `Private`, `Private::none`, `withhold` and `WITHHELD` exist in
  `jig_protocol_transcript` as `temper/docs/plan/ui-ask-m4/notes/contracts.md`,
  section 3 gives them, with their doc comments, in a module of their own,
  `src/withhold.rs`, re-exported from `lib.rs`.
- `withhold` decodes as `decode` admits (`smith_transcript::decode_archived`,
  version 6 or 7, within `limits.smith` and `limits.bytes`), refusing what
  `decode` refuses with the same `Undecodable`; projects as section 3 says;
  and encodes the result as the current version, within the same limits.
- One table test, `src/tests/withhold.rs` (its `mod` line in `tests.rs`),
  each row a turn made by `tests.rs`'s builders (`text`, `call`, `result`,
  `message`, `source`, `encode`), the `Private` and the turn expected after,
  compared decoded:
  - a section in the `system` text, with three numbered lines, withheld,
    the next `## ` heading and its lines kept;
  - the same section in an opening user message's text, `None kept.` alone,
    withheld, the person's words after it in the same block kept;
  - a line `## Kept about this person` followed at once by a line that is
    neither empty, `None kept.` nor numbered: the heading, then the empty
    line, `WITHHELD` and the empty line, then that line, kept;
  - a private tool's call and its result withheld, its `error` kept, a
    refused one among them, beside another tool's call and result kept
    whole;
  - a changed text's `replay` dropped, an unchanged text's kept;
  - an empty title: no section withheld, a private call still withheld;
  - `Private::none()`: the turn's blocks unchanged;
  - a version 6 turn admitted and returned as version 7;
  - bytes `decode` refuses, refused alike (`Malformed`, `Version`,
    `TooLarge`).
- The table bites: break the section rule once (stop at the heading) and
  the call rule once (keep the input), see their rows fail, and restore.

## Inputs

- `temper/docs/plan/ui-ask-m4/notes/common.md`; `notes/contracts.md`,
  section 3.
- `jig/docs/design/domain/engine.md`, 10.1, *Its runs' parts are its
  party's*.
- smith's transcript: `smith/crates/smith-transcript/schema/transcript-v7.schema`
  (`Turn`, `Message`, `Block`, `Text`, `Call`, `ToolResult`, `Returned`,
  `Said`), `smith/crates/smith-transcript/src/lib.rs`, `decode_archived`;
  the generated `encode` (`src/generated/v7.rs`).
- The pattern to follow: `jig/crates/jig-protocol-transcript/src/lib.rs`,
  `decode` and `decode_source`; its tests' builders, `src/tests.rs` lines 1
  to 130.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change:** `jig/crates/jig-protocol-transcript/Cargo.toml`, a
  dependency line if encoding needs one smith's crate already offers.
- **Options, pre-approved:** if `smith_transcript` offers no whole-turn
  encoder, encode as `tests.rs`'s `encode` does (`Turn::encode` into a
  `Writer` sized by the turn's measured length) and say so. If a version 6
  turn cannot be re-encoded as version 6, return version 7, as the note
  says.

## Steps

1. `src/withhold.rs`: `Private`, `WITHHELD`, `withhold`; its re-export.
2. The table; the two breaks.
3. `cargo fmt`; `heavy cargo clippy -p jig-protocol-transcript --all-targets
   -- -D warnings`; `heavy cargo nextest run -p jig-protocol-transcript
   --lib`. Commit and push.

## Report

As the implementer's text asks; name the rules broken and the rows that
failed.
