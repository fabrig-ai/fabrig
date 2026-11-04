+++
id = "bundle-part"
part = "bundle-helpers"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-engine-clients-http/src/bundle.rs", "jig/crates/jig-protocol-engine-clients-http/src/bundle/tests.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/testing/jig-test-service/src/server/bundle.rs", "jig/testing/jig-test-service/src/server.rs"]
effort = "high"
minutes = 50
+++

# jig: ship what serving a bundle has in common

## Goal

`jig/testing/jig-test-service/src/server/bundle.rs` and `temper/crates/temper-service/src/clients/bundle.rs` copy file
kinds, digest-path lookup and the reply's chunks. Ship those in `jig_protocol_engine_clients_http::bundle`, and have
the reference service compose them. A later task of this plan does temper's side. Behaviour stays unchanged.

Acceptance:

- `jig_protocol_engine_clients_http::bundle` has exactly the surface `notes/contracts.md` gives, and holds no `Answer`,
  no `Bundle`, no route and no cache field.
- `jig-test-service`'s `server/bundle.rs` holds no copy of `kind`, `target`, `field`, `demand`, nor the body of
  `Bundle::file` or `Pages::send`'s chunking; `File` and `kind` are re-exported, so `jig/testing/jig-test/src/bundle.rs`
  and `server.rs`'s `pub use` keep working.
- Every test of `jig-test-service`'s `tests/server.rs` passes unchanged, and the new tests below bite.
- `jig/docs/design/protocol/service.status.md` is left alone: its **Not yet** line for 11.4 goes with temper's side.
- **Size cap:** net at most +30 lines, tests included: the module and its tests add about 110, the service loses about
  80.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `temper/docs/plan/debt-10/notes/contracts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.4, on the trunk ("What serving a bundle has in common").
- skein's `skein/docs/design/ui-web.md`, 4.4.
- `jig/docs/development/debt/D005-accounts-host-copied.md`: this task doesn't pay it alone.

## Owns and may change

- Owns: the new `bundle.rs` and its tests, its `pub mod bundle;` line in the crate's `lib.rs`; the reference's
  `server/bundle.rs` and `server.rs`.
- `jig-protocol-engine-clients-http` is step code: the strict subset holds (`AGENTS.md`, 2). `chunk`'s arithmetic uses
  what the two copies use. The service's copies may read as they do; the shipped one may not, so `usize::try_from` and
  `.expect` become the crate's forms where clippy says so.
- If `demand` or `chunk` needs one more argument to fit both services, **stop and ask**: temper's task builds on this
  surface.

## Steps

1. New `bundle.rs` to the surface in the note. `kind`, `File`, `target` and `field` move as they stand. `file` takes
   `Bundle::file`'s body. `demand` and `chunk` take `Pages::demand`'s and `Pages::send`'s, with `offset` in and out
   and the body given.
2. In the reference: `Bundle::file` calls `bundle::file(&self.files, &self.digest, path)`; `Pages::demand` and
   `Pages::send` call the shipped ones, `reply.offset = bundle::chunk(..)`; `pub use` of `File` and `kind` from the
   shipped module. `use` lines tidy.
3. Tests, in `src/bundle/tests.rs`, `#[cfg(test)] mod tests;`:
   - `kind` on each listed extension, on a dotless name, on `a.b/c`, on an extension off the list.
   - `file` finds under its digest and refuses another digest, a path with no name, and an unknown file; `/robots.txt`
     names `files/robots.txt`.
   - `chunk` on a body longer than `send`: the offsets walk it, the last sends `Finish`, and each earlier one demands
     what's left. A `Server` for this exists in `src/tests.rs`' fixtures or `tests/server.rs`' shape; if the crate has
     none cheaply, drive it from `jig-test-service`'s `tests/server.rs`, which already serves a bundle, and say so.
4. Bite: an off-by-one in `chunk` and a dropped digest check. `heavy cargo nextest run -p jig-protocol-engine-clients-http
   -p jig-test-service`. fmt, clippy, old forms, citations. Commit, push.

## Report

As the implementer's text says, with `notes/common.md`'s table and the net lines.
