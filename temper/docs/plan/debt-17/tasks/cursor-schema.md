+++
id = "cursor-schema"
part = "cursor-schema"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients/schema/**", "jig/crates/jig-protocol-engine-clients/golden/**", "jig/crates/jig-protocol-engine-clients/src/generated/**", "jig/crates/jig-protocol-engine-clients/src/lib.rs", "jig/crates/jig-protocol-engine-clients/src/translate/cursor.rs", "jig/crates/jig-protocol-engine-clients/src/tests/reads.rs", "jig/tests/codec-families.rs", "jig/tests/engine-clients/tests/suite/**", "jig/docs/design/protocol/client.status.md", "jig/docs/development/debt/D027-engine-clients-cursor-format.md"]
effort = "high"
minutes = 120
+++

# jig: the owner cursor is a schema's record, with goldens

## Goal

Pay `jig/docs/development/debt/D027-engine-clients-cursor-format.md`. The cursor's hand-placed bytes become the family
`jig-engine-cursor`, generated, with goldens under `jig/tests/codec-families.rs`'s drift check.

**The bytes change, on purpose, and in place.** A schema can't hold today's: its integers are big-endian, its version a `u16` and
its variants number from 0 without gaps (`skein/docs/design/codec.md`, 3). This cut decided that isn't a stored format or a
protocol change: see `plan.md`. A client holds the bytes as opaque (`jig/docs/design/protocol/client.md`, 6), no store keeps
them, encodings change in place before the first release (`jig/docs/design/protocol/README.md`, 7), and a cursor from before
is refused rather than misread, so its read is answered unknown.

Acceptance:

- `jig/tests/codec-families.rs` has a row for the family, and both drift tests pass
  (`-p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`).
- `rg -n 'struct Writer|to_le_bytes|from_le_bytes' jig/crates/jig-protocol-engine-clients/src/translate/cursor.rs` finds nothing.
- `wc -l < jig/crates/jig-protocol-engine-clients/src/translate/cursor.rs` falls from 652 to at most 450.
- `cursor::encode`, `decode`, `encode_tree` and `decode_window` keep their signatures, and what they accept and refuse: every
  `Scope` that refuses to encode still does (`TaskText`, `EndedTasks`, `TaskRows`, `SettingsHistory`); a cursor past
  `CEILINGS.cursor_value` is refused; trailing bytes are refused; a tree's cursor doesn't read as an owner's or as a window's;
  `encode` refuses a limit one byte short.
- Every test of `jig-protocol-engine-clients` and `jig-engine-clients-world` passes, with only the exception below.
- The item is deleted, and `client.status.md`'s first `6:` entry leaves.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `jig/docs/development/debt/D027-engine-clients-cursor-format.md`
- `jig/docs/design/protocol/client.md`, 6 (this cut wrote the family) and `jig/docs/design/protocol/README.md`, 7
- `skein/docs/design/codec.md`, 3, 4, 6 and 7.1
- The nearest family: `jig/crates/jig-protocol-accounts` (`schema/moved-v1.schema`, `src/generated/v1.rs`, `golden/`) and its row in
  `jig/tests/codec-families.rs`; `jig/tests/store-codecs/tests/suite/fuzzy/decode.rs`, for arbitrary and mutated family bytes

## Owns and may change

- **Owns:** the files in this spec's header.
- **The one approved test change:** `a_cursor_that_does_not_decode_is_answered_unknown`'s three literals, `[255]`, `[1, 255]` and
  `[1, 1, 10]`, are in today's layout. Spell each of its labels in the new one: `unknown format` a version that isn't 1,
  `unknown variant` version 1 and a tag past `Form`, `truncated scope` a cut within a scope. The assertion stays as it is, and so
  do the labels.
- **Keys stay the store codec's:** `bytes` fields filled by `jig_protocol_store::keys::encode`.
- **Bounds:** no field need bound past `CEILINGS.cursor_value` (1,024), which `decode` checks first as it does today.
- **`jig-client-test-records`' `cursor_value` is 64,** and a cursor grows by at least the version's one byte. If a test's cursor now
  passes its limit and an outcome changes, stop and ask, with the sizes.
- **The tree's cursor is left as it behaves.** A tree writes form `Tree`, and `decode_window` reads a bare `DocumentCursor`. So a tree's
  `next` given back is refused, today and after. That looks like a fault: report it under `## Left`, and don't fix it here.
- If `cursor.rs` can't reach 450 without losing behaviour, keep the behaviour and say how close it came: this cap is the cut's estimate.
- Nothing outside the crate reads `cursor.rs`: `translate.rs` declares it `pub(crate)`. If a reader turns up, stop and ask.

## Steps

1. `schema/cursor-v1.schema`, `family jig-engine-cursor 1`, mirroring today's layout type for type:

   ```
   versioned record Held { form: Form }
   enum Form { Owner: Cursor, Tree: Tree }
   record Tree { task: u64, window: u32, after: DocumentCursor }
   enum Cursor { Live: LiveCursor, Store: StoreCursor }
   record LiveCursor { scope: Scope, after: DocumentCursor, store_after: option Position }
   record StoreCursor { scope: Scope, after: option Position }
   enum Scope { Task, Proposal: .., Escalation, Question: .., PersonTask, Result, Reply: .., Turn: .., EndedDelegates,
                History: .., Activity, Inbox: .., Notes: .., ScopedNotes: .., Spend: .., Tasks: .. }
   enum Position { Owner: OwnerKey, Inbox: InboxPosition }   # a key: `bytes`, in the store's codec
   enum DocumentCursor { Number: .., Inbox: .. }
   ```

   Declare each before its use (`codec.md`, 7.1), with a record for each variant that carries more than one value. Today's layout
   (little-endian, all of it going), with `opt` = `0 | 1 value` and `blob` = `u32 len, bytes`:

   ```
   owner:  01 | Cursor                     tree: 02 | task u64 | window u32 | DocCursor     window: DocCursor, bare
   Cursor: 00 Scope DocCursor opt(Position) | 01 Scope opt(Position)
   Scope:  0 Task, 1 Proposal{Proposer,u64}, 2 Escalation, 3 Question{u64,u64}, 4 PersonTask, 5 Result, 6 Reply{u64,u64},
           7 Turn{u64,u64,u32}, 8 EndedDelegates, 9 History{u64,bool}, 10 Activity, 11 Inbox{Group,Filter},
           14 Notes{u32}, 15 ScopedNotes{NoteScope,opt blob}, 16 Spend{u32}, 17 Tasks{Query}
   Position: 0 Owner(key) | 1 Inbox{opt wall, u64, InboxObject, key}
   DocCursor: 0 Number{u64} | 1 Inbox{u64, InboxObject, opt wall}
   InboxObject: 0 Question, 1 Proposal, 2 Escalation, 3 PersonTask, 4 Reply, 5 Result
   Proposer 0/1 u64; Group 0-7; NoteScope 0-3; Last 0/1 blob; Filter opt u32, opt u64
   Pattern: u32 count, blobs, Last
   Query: opt u32, requester tag 0/1/2, opt u64 x2, opt u32 x4, u32 order
   ```

   A wall is a `u64` of nanoseconds. Read `put_scope`, `put_note_scope` and `put_query` for each field's meaning and for what's
   in `NoteScope` and `Query`. Keep the schema as small as it can be: the item asks for "a small schema".

2. `src/generated/cursor_v1.rs` and, in `src/lib.rs`, the `#[rustfmt::skip] #[path]` module as `jig-protocol-accounts` has it.
   Make `src/generated/` by hand first, then write the module and goldens with
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`, after adding
   this row to `FAMILIES`:

   ```rust
   Family {
       name: "engine cursor",
       root: "crates/jig-protocol-engine-clients",
       schemas: &["schema/cursor-v1.schema"],
       generated: &["src/generated/cursor_v1.rs"],
       golden: "golden",
       fixtures: &[],
   },
   ```

3. `translate/cursor.rs` keeps only the mappings between `reads::Cursor` and `documents::Cursor` and the generated records:
   `Writer`, the readers and the hand tags go. `encode` measures, refuses past `limit`, then writes; `decode` checks the ceiling, reads
   `Held` and takes `Form::Owner`; `decode_window` reads a bare `DocumentCursor` with nothing left over.

4. A cursor comes from a client, so it's read as hostile: `jig/tests/engine-clients/tests/suite/fuzzy/cursor.rs`, under a `mod fuzzy` in
   that suite's `main.rs`, after `jig/tests/store-codecs/tests/suite/fuzzy/decode.rs`: arbitrary and mutated bytes through `Held::decode`
   never panic, and what decodes writes back the same bytes. It reads the generated record, since `translate::cursor` is
   crate-private. Break the drift check by hand once: alter a golden byte, watch it go red.

5. `heavy cargo nextest run -p jig-protocol-engine-clients -p jig-engine-clients-world -p jig-client-wire-world -p
   jig-store-codecs-world`. A cursor's bytes are a shared item's meaning, so run its readers too, `-p jig-client -p jig-client-protocol-world`,
   and `heavy cargo check --tests -p temper-client-tools`, whose hex `read on` prints and takes these bytes without reading them.

6. The status line, then delete the item. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-engine-clients -p jig-engine-clients-world --all-targets -- -D warnings`; the citations check. Commit and push.

## Report

The common note's items, `cursor.rs`'s lines, the schema's and the generated file's, what you broke by hand and what failed, the
fuzzy test's seeds and time, and the tree's cursor under `## Left`.
