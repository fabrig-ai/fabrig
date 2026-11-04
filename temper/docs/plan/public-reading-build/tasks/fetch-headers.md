+++
id = "fetch-headers"
part = "skein-fetch"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui-web/src/fetch.rs", "skein/crates/skein-ui-web/src/fetch/**", "skein/crates/skein-ui-web/src/bridge.rs", "skein/crates/skein-ui-web/src/log/**", "skein/crates/skein-ui-web/src/tests/**", "jig/crates/jig-client-shell-web/src/adapter.rs", "jig/crates/jig-client-shell-web/src/tests.rs", "jig/crates/jig-client-shell-web/src/tests/**"]
effort = "high"
minutes = 75
+++

# skein's fetch: an exchange's cache and an answer's kept headers, and jig's web adapter carrying them

## Goal

A web application can ask for an exchange the browser's cache neither answers nor keeps, and
read the answer headers it names, so jig's public reads in a browser know their freshness
and when to ask again (`skein/docs/design/ui-web.md`, section 10, "An exchange's cache and an
answer's headers"; `notes/contracts.md`, section 6). jig's web adapter carries both, every
existing exchange keeping the browser's cache and no header.

This task departs from "a branch stays in its tree": it changes skein's `fetch` and jig's
web adapter (`jig-client-shell-web/src/adapter.rs`) together, since the adapter is the one
mapping of skein's exchange into jig's and carries exactly the new fields; every other reader
in jig and temper is edited for the fields alone.

Acceptance:

- `skein_ui_web::fetch`: `pub enum Cache { Browser, NoStore }` (deriving as `Exchange` does);
  `Exchange` gains `pub cache: Cache` and `pub keep: &'static [&'static str]`;
  `Report::Answered` gains `headers: Vec<(String, String)>`, the kept names alone, lower
  case, in `keep`'s order, each value at most the exchange's `bound` bytes (a longer value
  or a header past them fails the exchange, as a body past its bound does), a header the
  answer lacks absent.
- On wasm32 (`fetch/web.rs`): `Cache::NoStore` sets the request's cache mode to
  `RequestCache::NoStore` (enable the `web-sys` feature if the crate lacks it); the kept
  headers are read from the response's `Headers` by name. The native half and the fake
  platform report what the exchange's fake answer gives.
- The bridge's encoding (`bridge.rs`) and the session log's (`log/`) carry `cache`, `keep`
  and `headers` both ways; `keep` crosses as names and is mapped back to the exchange's own
  `&'static` names on the side that holds them, or the bridge carries owned names if the
  types require it (say which).
- `jig_client_shell_web::adapter`: `Asked::Exchange` gains `cache: fetch::Cache` and
  `keep: &'static [&'static str]`, passed into `fetch::Exchange`; `Heard::Answered` gains
  `headers: Vec<(String, String)>`, from `Report::Answered`.
- Step tests:
  - skein-ui-web `src/tests/fetch.rs`: `kept_headers_are_reported_by_name_in_order`,
    `an_unkept_header_is_never_reported`, `a_kept_header_past_the_bound_fails_the_exchange`,
    `a_missing_kept_header_is_absent`;
  - skein-ui-web `src/tests/bridge.rs`: the round-trip tests cover an exchange with each
    `Cache` and a non-empty `keep`, and an answer with headers;
  - jig-client-shell-web `src/tests.rs`: `an_exchange_carries_its_cache_and_kept_names` and
    `an_answer_carries_its_kept_headers`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, section 6.
- `skein/docs/design/ui-web.md`, section 10 (exchanges; their cache and kept headers) and
  section 18 (the worker's platform, whose bridge carries reports).
- As they stand: `skein-ui-web`'s `fetch.rs`, `fetch/web.rs`, `bridge.rs`, `log/`,
  `src/tests/fetch.rs`, `src/tests/bridge.rs`, `src/tests/fixtures.rs`;
  `jig-client-shell-web/src/adapter.rs` and its tests.

## Owns and may change

- Owns the files in the header.
- **Readers, each edited for the new fields alone** (`cache: Cache::Browser`, `keep: &[]`,
  `headers` ignored or `Vec::new()`):
  - skein: `skein/crates/skein-playground/src/visitor.rs`, `skein/examples/ui/shell-web/src/lib.rs`,
    `skein/examples/ui/view/src/{state.rs, tests/fixtures.rs, tests/stream.rs, tests/adds.rs}`,
    `skein/tests/playground/src/scripted.rs`, `skein/tests/playground/tests/suite/{memory.rs, memory/support.rs, replay.rs, visitor.rs, slices/support.rs}`,
    `skein/tests/ui-web/tests/suite/pages.rs`;
  - jig: `jig/testing/jig-client-test-shell-web/src/host.rs`;
  - temper: `temper/crates/temper-client-shell-web/src/{host.rs, host_tests.rs, host_tests_authorize.rs, host_tests_departure.rs, journey/line.rs, journey/record.rs, journey/tests.rs, journey/walk.rs}`,
    `temper/crates/temper-client-shell-web/tests/suite/host/walk.rs`.
  Any other the compiler names in skein, jig or temper: the same, listed in the report.
- Pre-approved: if skein-playground's visitor (a world program speaking HTTP as a browser's
  fetch) can honour `keep` from the HTTP answer it parses, do so and test it in
  `skein/tests/playground`'s visitor suite; otherwise report no header and say so under
  `## Left`. If the session log's format is versioned, bump it as its own rules say.

## Steps

1. `fetch`'s types and both halves; the bridge and the log; skein-ui-web's tests
   (`heavy cargo nextest run -p skein-ui-web --lib`), and
   `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings`.
2. jig's adapter and its tests (`heavy cargo nextest run -p jig-client-shell-web --lib`).
3. The readers until `heavy cargo check -p skein-playground -p skein-playground-world -p jig-client-test-shell-web -p temper-client-shell-web --tests`
   passes (package names as their `Cargo.toml`s give them), and the wasm32 clippy of
   `jig-client-shell-web`, `jig-client-test-shell-web` and `temper-client-shell-web`.
4. `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py` from each tree you touched;
   clippy on every package you changed; their focused tests and their direct dependents'
   (`skein-playground`'s world among them); the citations check.
5. Commit (why: an exchange the browser's cache leaves alone and the answer headers an
   application names, which public reading in a browser reads its freshness from); push your
   branch.

## Report

As the implementer's text says, and also: how the bridge carries `keep`, and every reader you
edited.
