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
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper-service/src/tests/qa_box_link.rs",
  "temper/crates/temper-service/src/tests/system.rs",
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
  paths are written absolute, as the record holds them, except an owned
  entry's Forgejo or runner binary whose record fetch flag is set,
  written `"fetched"` (protocol/setup.md, 8).
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

## Answers

2026-11-01. May this task change `temper-service/src/system.rs` for the
box link's file io? Yes (option A); `system.rs` is now in `files`. The
service owns the kernel and routes every completion by its token's
prefix (`Service::complete`), so the link's file requests cannot be
driven from `box_link.rs` alone. In `system.rs`, only: open `<data>`
(the pre-approved default) when `adopt_box` makes the `Link`, and hand
the link its directory; submit the link's file requests beside
`box_io`'s pipe requests; route completions bearing the link's token
(`tokens.rs`) to the link, ahead of `io::up`; carry the write's
deadline and close its descriptors on shutdown, as the store's are.
Rendering and the write's states stay in `route/group_file.rs`, the
ask's state in `route/box_link.rs`. `fr-setup` also edits `system.rs`:
keep the change to those places so the merge stays small.

2026-11-01. Where does the link get `<data>`, and how is a fetched
binary written? Option A for both. (1) The data directory: in
`temper/crates/temper/src/lib.rs`, in the boxed branch only, open
`directories.data` (already made private by `Directories::open`) with
`skein_shell::make_root(.., 0o700)`, push it to `owned`, and pass it
with the effective user to `adopt_box`, which now takes `(requests,
answers, data: Fd, effective_user: u32, line_bytes)` and hands both to
the link, as `adopt_store` hands the deployment root to the store; on
refusal the caller keeps the descriptor. The link opens its own
directory beneath that root (`OpenDirectory`, as `store.rs` does) and
closes its descriptors at shutdown. Repair the callers of `adopt_box`
(`system.rs`'s own test, `tests/system.rs`, `tests/qa_box_link.rs`;
now in `files`) with a fixture descriptor, and their expected io where
a box start now writes its file first; nothing else in those files.
(2) Fetched binaries: `render` writes `"fetched"` for
`mode.owned.forgejo` and for the runner's binary when the record's
`fetch.forgejo` / `fetch.runner` is set, the stored absolute path
otherwise: that is the entry's form in protocol/setup.md, 8, and reads
back to the same `Entry`. The reader and the file format are unchanged.
