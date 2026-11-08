+++
id = "serve-sign-in"
part = "linked"
kind = "feature"
needs = ["jig-sign-in", "forges-linked"]
files = [
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper-service/src/settings.rs",
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/system.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/tests/clients.rs",
  "temper/crates/temper-service/src/tests/fixtures.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/docs/design/client/README.status.md",
  "temper/docs/design/protocol/setup.status.md",
]
effort = "high"
minutes = 75
+++

# Serve takes the box's first local link, and says on the forges document that the printed link was used

## Goal

temper's engine side of the box's front: `temper internal serve --box`
reads the printed link's first secret and, when the box's TUI is in front,
the first local link's, and gives both to jig; and the engine sets the
forges document's `linked` the first time jig says `LinkSignedIn`, so that
every client's forges watch hears that a browser used the printed link.

Acceptance, by `temper/docs/design/protocol/setup.md`, 7.1 (*The first
link's secret*), `temper/docs/design/client/README.md`, 4.6 (`linked`), and
`notes/contracts.md`, sections 3 and 4:

1. **Serve's stdin:** `lib.rs`' `given_secret` (about line 678) reads to the
   end, at most 65 bytes, and gives the given secrets: none for no bytes;
   the printed link's for 32; the printed link's and the local link's for
   64 (the first 32, then the next 32); any other length refused,
   `--box: stdin: expected the first link's secret`. `serve` (about line
   710) carries both to `prepare_with_limits` and `Seeds`.
   `temper-service`'s `Seeds` gains `pub local_secret: Option<Secret>`
   beside `link_secret`; `clients.rs` (about line 216) gives it to jig's
   `kit::Settings { local_secret, .. }` (the field `jig-sign-in` added,
   which it filled with `None` there).
2. **`linked`:** `Clients` keeps `linked: bool`, false at start; the arm
   `kit::Request::LinkSignedIn` in `advance` (about line 530; `jig-sign-in`
   left it empty) sets it, and when it turns true the engine marks the
   forges document (`root::Domain::mark_forges`, as
   `temper-service/src/forge/origins.rs:71` does), so that each forges
   watcher is delivered again. `route.rs`' `forge_rows` (about line 401)
   gives the flag to `family::forges` (about line 1055), which encodes it in
   `w::ForgesParts { forges, linked }` in place of `forges-linked`'s
   `false`. A restart starts it false again (a new `Clients`).
3. **Tests,** a failing test first for each:
   - `temper/crates/temper/src/lib.rs`' box tests,
     `a_boxed_serve_reads_none_or_a_whole_secret` (about line 785): rows
     for 64 bytes (both secrets, in order) and for 33, 63 and 65 bytes
     (refused).
   - `temper-service/src/tests/clients.rs`: a `with_given_secrets()` beside
     `with_given_secret()` setting `seeds.local_secret` too; one new test,
     `a_given_local_link_signs_in_and_is_never_kept_or_printed`: the given
     local link (`first_link_url` of the local secret) signs the owner in;
     `local_links()` held nothing before it (no `LocalLink` for a given
     one) and holds the next local link after it, differing from both given;
     `links()` stays empty; the given printed link still signs in after it.
   - the same file, `the_local_link_is_kept_apart_and_signs_in_in_either_order`
     gains: the forges document a watcher is delivered says `linked` false
     before either sign-in, still false after the local link's, and true
     after the printed link's (read it as `tests/family.rs`'
     `forges_encode_the_domains_rows_without_settings` decodes one, or
     through `forge_rows` on the engine; a helper the two need goes at the
     bottom of `tests/clients.rs`).
4. **Status:** delete the **Not yet** bullet naming `serve-sign-in` in
   `temper/docs/design/client/README.status.md` and the one in
   `temper/docs/design/protocol/setup.status.md`. No other design text.

## Inputs

- `temper/docs/plan/tui-box-front/notes/common.md`; `notes/contracts.md`,
  sections 2 to 4.
- `temper/docs/design/protocol/setup.md`, 7.1, *The first link's secret*;
  `temper/docs/design/client/README.md`, 4.6, *The deployment's forges*,
  `linked`; jig's `jig/docs/design/protocol/client.md`, section 8, *A secret
  given* and *Signed in by it, said*.
- The code: `temper/crates/temper/src/lib.rs:678-693` (`given_secret`),
  `:698-764` (`serve`), `:449` and `:518` (`prepare_with_limits`, `Seeds`),
  `:785-805` (its tests); `temper-service/src/settings.rs:84`;
  `clients.rs:216` (jig's settings), `:510-560` (`advance` and its arms);
  `route.rs:140-160` and `:401-403`; `family.rs:1055-1061`;
  `temper-engine-domain/src/domain.rs:262-267` (`mark_forges`);
  `temper-service/src/forge/origins.rs:64-71` (a document marked after a
  change); `src/tests/clients.rs:1-150`.

## Owns and may change

Owns serve's stdin reading, `Seeds.local_secret`, `Clients`' `linked` and
its marking, the forges encoding's flag, the tests above and the two status
edits. Reader edits: a `Seeds { .. }` literal that the compiler forces gains
`local_secret: None` (`temper-service/src/tests/fixtures.rs`,
`temper-playground/src/documents.rs`, `temper/crates/temper/tests/support/owned.rs`,
`temper/tests/one-process/src/fixtures.rs`, and any other `rg 'link_secret'`
finds), the field alone. Not `box_command.rs` or `temper-box` (the box's
64 bytes are `box-front`'s), not `temper-client-*` (the client reads
`linked` in `domain-front`).

Options, each decided:

- **Where the mark is made:** if `Clients::advance` can't reach the root's
  domain, keep a `linked_changed` flag the engine's step reads after
  `clients.advance` and marks then; say where.
- **`forge_rows`' signature:** take the flag from `engine.clients`; a getter
  `Clients::linked(&self) -> bool` is fine.

## Steps

1. The failing tests; then `lib.rs`, `settings.rs`, `clients.rs`, `route.rs`,
   `family.rs`; the narrowest test after each:
   `heavy cargo nextest run -p temper --lib -E 'test(/a_boxed_serve_reads_none_or_a_whole_secret/)'`,
   `heavy cargo nextest run -p temper-service -E 'test(/tests::clients::/)'`.
2. `heavy cargo nextest run -p temper-service`; `heavy cargo nextest run -p temper --lib`;
   `heavy cargo nextest run -p temper --test suite -E 'test(/box_listeners::/)'`
   (the box still gives 32 bytes; its stories must stay green);
   `heavy cargo check --tests` for every package whose `Seeds` literal you
   filled. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `heavy cargo clippy -p temper -p temper-service --all-targets -- -D warnings`
   and each other package touched; `python3 -I -B tools/citations/citations.py --base origin/main`.
3. The status edits; commit (why: the box can hand its TUI the first local
   link, and every client hears when a browser used the printed link) with
   the trailer `Look-request: tui-box-front`; push.

## Report

As the implementer's text says, with each test added or extended and the
rule broken by hand for it, and where the forges document is marked.

## Rebase (2026-11-08, step merge, foreman tui-box-front/serve-sign-in/foreman-1)

Resolve setup.status.md by retaining both adjacent completed-task deletions.
The merge conflicts in `temper/docs/design/protocol/setup.status.md`.
Task commit `dade69f4c9` deletes the 7.1 `serve-sign-in` **Not yet** bullet.
Integration commit `6ed0cf48b1`, merged by `d0aa6ad42c`, deletes the immediately following 7.6 `box-lines` bullet.
Rebase onto the current integration tip and resolve by deleting both bullets.
Preserve all remaining status entries, including the `box-front` bullet and main’s 13.4 `serve.log` coverage gap from `25aa7cea1b`.
Keep this task’s deletion of the `serve-sign-in` bullet in `temper/docs/design/client/README.status.md`, which auto-merged.
The refusal identifies no code or test failure; this resolution requires no implementation change.
