+++
id = "fr-group-file"
part = "forge-records"
kind = "feature"
needs = ["fr-forge-added", "bx-kinds"]
files = [
  "temper/crates/temper-service/src/route/box_link.rs",
  "temper/crates/temper-service/src/route/group_file.rs",
  "temper/crates/temper-service/src/tests/group_file.rs",
  "temper/crates/temper-service/src/tests/box_link.rs",
]
effort = "high"
minutes = 50
+++

## Goal

Under the box, serve's box link writes a forge's group file,
`<data>/box/groups/N`, from the forge's record before it sends `start N`
(contracts, section 3), so the box can run a forge its document does not
name. The file is exactly `{"setup": ENTRY, "playground": BOOL}`, ENTRY
the record's setup entry in `application.setup[]`'s form with `"forge":
N` (and `"start": "asked"` for an owned Forgejo), which
`temper_forge_setup::file::read` (`bx-kinds`) reads back to the same
`Entry`.

Acceptance:

- `group_file::render(number: u16, setup: &record::Setup, playground:
  bool) -> Box<[u8]>` gives bytes that `temper_forge_setup::file::read`
  accepts and reads to the `Entry` the record stands for, for an owned
  (with runner, mirror and public origin), a fake and a managed entry;
  paths are written absolute, as the record holds them.
- The box link's ask for a forge whose held record has a setup entry
  (`forge::table::record`, `fr-forge-added`) first writes the file:
  `box/groups` made 0700 if absent, the file stored whole (skein-io's
  `file::Request::Store`, a synced temporary renamed) at 0600; only once
  it is stored does `start N` go out. A write that fails answers the ask
  as the box's `failed` would (`SetUpFailed`, why `group file not
  written`), and no `start` goes out.
- A forge with no setup entry, or a box link not under the box, writes
  nothing; today's asks for the document's forges behave as before (the
  file is written for them too, and the box prefers the document's
  entry, contracts 3).
- The tests below pass; `temper-service`'s focused box-link tests pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 3).
- `temper/docs/plan/playground-reset-r4/notes/progress-box.md` and the
  merged `bx-kinds` report (`file::{read, path, GroupFile, BYTES}`).
- `temper/docs/design/protocol/setup.md`, 7.1 (the pipe) and 8 (the
  entry's members).
- skein's `skein/docs/design/io.md`, the file layer's `Store` and
  `MakeDirectory`; the service's file use today in
  `temper-service/src/store.rs` (`OpenDirectory`) as the model of
  opening a directory and keeping its token.
- `skein-json`'s writer (`skein/crates/skein-json/src/writer.rs`).

## Owns and may change

Owned: the files in the header (`route/group_file.rs` and
`tests/group_file.rs` new; the `mod` lines in `route.rs`'s neighbour
`route/` parent and `tests.rs`).

Neighbours: `temper-service/src/route.rs` (the `mod group_file;` line),
`engine.rs` (a file owner token or a directory token kept on the engine,
if the link needs one), `tokens.rs` (a token constant for the link's
file requests): those lines only.

Pre-approved options:

- If the service holds no data directory token under the box, open
  `<data>` once, when the link is made (`Link::new` under the box), as
  the store opens its own, and keep it; default.
- If `file::read` refuses an absolute path somewhere, write that member
  relative to its directory, and say which.

## Steps

1. `group_file.rs`: `render` (above) with `skein_json`'s encoder, the
   members in `Setup::read`'s order; `pub(crate) fn write(engine, number,
   bytes)` issuing the io requests and its completion handling.
2. `box_link.rs`: the ask's new first step (file, then `start`), as a
   state of the `Ask` it holds; the box link's one-ask-at-a-time rule
   holds across the write.
3. **Tests**: `tests/group_file.rs`:
   `an_owned_record_renders_an_entry_file_read_reads_back`,
   `a_fake_record_renders_with_playground_true`,
   `a_managed_record_renders_without_start`;
   `tests/box_link.rs`: `a_record_forge_start_writes_its_file_first`
   (the io requests in order, then the `start N` line),
   `a_failed_write_answers_set_up_failed_and_sends_no_start`.
4. `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets --
   -D warnings`; `temper-service`'s focused tests; `heavy cargo check
   --tests -p temper -p temper-one-process-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `render`, `write`, the new ask
state, the `failed` why, and the file's mode bits as set.
