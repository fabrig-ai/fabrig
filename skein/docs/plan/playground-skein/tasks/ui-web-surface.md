+++
id = "ui-web-surface"
part = "ui-web"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui-web/src/bridge.rs", "skein/crates/skein-ui-web/src/lib.rs", "skein/crates/skein-ui-web/src/log.rs", "skein/crates/skein-ui-web/src/platform.rs", "skein/crates/skein-ui-web/src/platform/web.rs", "skein/crates/skein-ui-web/src/platform/worker.rs", "skein/crates/skein-ui-web/src/fetch.rs", "skein/crates/skein-ui-web/src/tests.rs", "skein/crates/skein-ui-web/src/tests/bridge.rs", "skein/crates/skein-ui-web/src/tests/log.rs", "skein/crates/skein-ui-web/src/tests/worker.rs", "skein/crates/skein-ui-web/src/tests/fixtures.rs", "skein/crates/skein-ui-web/src/tests/transport.rs"]
effort = "high"
minutes = 75
+++

# skein-ui-web: the bridge's messages, the platform's two duties, and the modules laid down

## Goal

`skein-ui-web` gains the bridge's messages and their encoding, portable, so the page and the
worker share them (`skein/docs/design/playground.md`, 5), and its `Platform` gains the two
duties fetch alone took, abandoning an exchange and reading the stream on
(`skein/docs/design/ui-web.md`, 18). It also lays down the empty modules two later tasks of
this crate fill, so they own disjoint files.

Acceptance:

- `skein_ui_web::bridge` holds exactly `notes/contracts.md`, section 1: `Speed`, `Start`,
  `Input`, `ToWorld`, `Stream`, `Printed`, `Stamp`, `Clock`, `ToPage`, `Malformed`, `READY`,
  and `encode`/`decode` on `ToWorld` and `ToPage`, with the encoding rules there.
- `fetch::Exchange` derives `Clone, PartialEq, Eq` besides `Debug`; `fetch::Report` derives
  `Clone` besides its own.
- `platform::Platform` gains `abandon(&mut self, exchange: u32)` and `read_on(&mut self)`
  (contracts, section 2); `platform::Browser<R>` implements them through its `fetch`, whose
  `pub` field stays.
- The modules of contracts, section 3, exist with their comments alone.
- Step tests in `src/tests/bridge.rs` pass:
  - `every_message_to_the_world_round_trips`: each `ToWorld` variant, each `Input` variant
    (an exchange with headers and a body, an empty body, an abandon, the stream opened with
    headers, closed, a read-on), a `Start`, a pace at 0, 1000 and `u32::MAX`, decodes to
    itself;
  - `every_message_to_the_page_round_trips`: each `ToPage` variant, each `fetch::Report`
    variant, a `Printed` of each stream, a `Clock` with and without a next due instant, the
    largest values of each field, `Failed` with empty and non-empty words;
  - `no_message_encodes_as_ready`: no encoded message of the two tests above is empty;
  - `a_short_or_long_message_is_malformed`: every proper prefix of a few encoded messages,
    each with a byte appended, an unknown tag, and a length past the input, all `Malformed`;
    a length of `u32::MAX` in a short input allocates nothing (it is refused before).
- `src/tests/transport.rs` or `src/tests/fixtures.rs`: the fake platform records
  `Call::Abandon(u32)` and `Call::ReadOn`, and one test
  `the_platform_takes_abandons_and_read_ons` calls both through the trait and reads them back.
- `heavy cargo nextest run -p skein-ui-web --lib` passes, its count in the report;
  `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings` and
  `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings` are
  clean; the readers below build.

## Inputs

- `skein/docs/plan/playground-skein/notes/common.md`, `notes/contracts.md`, sections 1 to 3.
- `skein/docs/design/playground.md`, 5 (the bridge); `skein/docs/design/ui-web.md`, 10 (fetch)
  and 18 (the worker's platform, its first bullet).
- `skein-ui-web`'s `src/fetch.rs`, `src/transport.rs`, `src/platform.rs`,
  `src/platform/web.rs`, `src/tests/fixtures.rs` as they stand.

## Owns and may change

- Owns the files in the header.
- **Readers of the `Platform` trait**, each edited only to implement the two new methods:
  - `jig/crates/jig-client-shell-web/src/tests/fixtures.rs`, its `Platform`: `abandon` and
    `read_on` each push their name to `calls` (`"abandon"`, `"read_on"`), as its other
    methods do. Nothing else of the file changes; run
    `heavy cargo nextest run -p jig-client-shell-web --lib`.
  - Any other `impl … Platform for` in skein, jig or temper that the compiler names: the same
    one-line bodies, listed in the report.
- **The reference shell** `skein/examples/ui/shell-web/src/lib.rs`: its one
  `self.platform.fetch.read_on();` becomes `self.platform.read_on();` (the trait's), with the
  trait's `use` if the compiler asks; nothing else. Check it with
  `heavy cargo clippy -p skein-ui-example-shell-web --target wasm32-unknown-unknown -- -D warnings`
  (the package name as its `Cargo.toml` gives it).
- Read only: `jig/testing/jig-client-test-shell-web` and `temper/crates/temper-client-shell-web`,
  which keep `platform.fetch.read_on()`; check they build:
  `heavy cargo clippy -p jig-client-test-shell-web -p temper-client-shell-web --target wasm32-unknown-unknown -- -D warnings`.
  If one fails for a reason outside this task, report it under `## Left`.
- Pre-approved: if a derive on `fetch::Exchange` meets a field that cannot derive it, say so
  and stop (it should not: its fields are integers, boxes, strings and vectors). If the
  encoding wants a small private reader and writer, write them in `bridge.rs`; skein-lib's
  `Writer` may be used.

## Steps

1. `fetch.rs`'s derives; `bridge.rs` with its types and encoding; `lib.rs`'s declarations.
2. The empty modules of contracts, section 3, and their declarations.
3. The trait's two methods, `Browser`'s bodies, the fake platform's calls; the jig fixture;
   the reference shell's call.
4. The step tests; run them alone
   (`heavy cargo nextest run -p skein-ui-web --lib -E 'test(/bridge::/)'`), then the crate's
   lib tests and the jig crate's.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py` and
   `python3 -I -B jig/scripts/old_forms.py`; the clippy runs above; the citations check.
6. Commit (why: the bridge's messages and the platform's duties, which the worker's platform
   and the bridge in memory build on); push your branch.

## Report

As the implementer's text says, and also: the encoding's layout in a few lines (tags and
field order), and every `Platform` implementation you edited.
