+++
id = "public-http"
part = "jig-public"
kind = "feature"
needs = ["public-component"]
files = ["jig/crates/jig-protocol-engine-clients-http/src/public.rs", "jig/crates/jig-protocol-engine-clients-http/src/dispatch.rs", "jig/crates/jig-protocol-engine-clients-http/src/lib.rs", "jig/crates/jig-protocol-engine-clients-http/src/call.rs", "jig/crates/jig-protocol-engine-clients-http/src/tests.rs", "jig/crates/jig-protocol-engine-clients-http/src/tests/**", "jig/crates/jig-client-protocol-http/src/public.rs", "jig/crates/jig-client-protocol-http/src/lib.rs", "jig/crates/jig-client-protocol-http/src/tests.rs", "jig/crates/jig-client-protocol-http/src/tests/**", "jig/tests/engine-clients/src/peer/public_http.rs", "jig/tests/engine-clients/tests/suite/public_http.rs"]
effort = "high"
minutes = 120
+++

# Public reads over HTTP: the `GET` route, its fixed answers and headers, and the client's call

## Goal

The default HTTP mapping carries public reads: the engine end parses
`<prefix>/v1/public/<owner>/<version>/<kind>[/<id>[/<id>]]` strictly, reads no credential,
checks `Host` first, holds a calls' room of its own, and answers with fixed statuses and
headers whose `max-age` is shortened by the time since the answer's decision; the client end
sends the `GET` with no bearer and reads back the answer and its freshness
(`jig/docs/design/protocol/client.md`, 17.5; `notes/contracts.md`, sections 4 and 6).

Acceptance:

- **The path** (`jig-protocol-engine-clients-http/src/public.rs`):
  `pub fn public_path(path: &[u8]) -> Option<kit::PublicRead>`, `path` the target after the
  prefix: `/v1/public/` then owner, version and kind, each a decimal with no sign and no
  leading zero (`0` alone allowed) within `u16`, then at most two ids of exactly 16
  lowercase hexadecimal digits; `None` for anything else (a query or `#`, a trailing slash,
  an empty, `.`/`..` or percent-encoded segment, upper-case or short ids, an overflowing
  number, a third id). The route is `dispatch::Route::Public(read)` for `GET` alone; any
  method but `GET` under `<prefix>/v1/public/` (`OPTIONS`, `HEAD`, `POST` among them) and any
  path there `public_path` refuses is `404` with `Cache-Control: no-store`.
- **No credential on these routes:** for a public route the mount reads no cookie and no
  bearer, applies neither the browser-header nor the content-type guard, and hands the
  component `FromTransport::PublicRead { exchange, read }`; a public call that declares a
  body is `404`. `Host` is checked first, as for every route (`421`). No other route's guard
  changes.
- **Its room:** at most `limits.public_calls` public calls in flight, apart from the other
  calls (`Mount::room()` keeps counting the others as now); a public call past it is `503`
  busy with its head read and nothing more, never handed on.
- **Its answers** (`PublicReply` mapped), each with exactly these headers and no others:
  - `Document`: `200`, its bytes, `content-type: application/octet-stream`,
    `cache-control: public, max-age=<rest>, must-revalidate`, `x-content-type-options:
    nosniff`; `rest` = `max_age` less the whole seconds, rounded up, since `decided` by the
    mount's `Env.now`, saturating at zero;
  - `Unknown`: `404`, no body, the same `cache-control` with the whole `max_age` (it carries
    no decision), and `x-content-type-options`;
  - `Busy` (the component's, or the room's): `503`, no body, `cache-control: no-store`,
    `retry-after: 1`, `x-content-type-options`;
  - `NotServed { record }`: `200`, its record, `cache-control: no-store`, content type and
    nosniff;
  - a malformed path or method: `404` as above.
  None sets a cookie or carries `vary`, `etag`, `last-modified`, `age`, any
  `access-control-` header or a stale directive. The mount's `Settings` gains
  `pub max_age: u32` (`clients.public.max_age`), filled by each service from the
  component's limits.
- **The client end** (`jig-client-protocol-http/src/public.rs`, re-exported from `lib.rs`):

  ```rust
  pub const PUBLIC_KEEP: &[&str] = &["cache-control", "age", "retry-after"];
  pub struct PublicPath { pub owner: u16, pub version: u16, pub kind: u16, pub ids: List<u64> }
  /// The `GET`, no bearer, no body, no content type (protocol/client.md, 17.5).
  pub fn public(mapping: &Mapping, path: &PublicPath) -> Call;
  /// The path `public` prints, after the prefix.
  pub fn public_target(path: &PublicPath) -> Box<[u8]>;
  pub struct Freshness { pub fresh: u32, pub retry_after: Option<u32> }
  /// max-age less Age, saturating at zero; no-store, a missing or malformed max-age or a
  /// malformed Age give 0; Retry-After's whole seconds, None when absent or malformed.
  pub fn freshness(headers: &[(String, String)]) -> Freshness;
  pub enum PublicHeard { Document { body: Box<[u8]>, fresh: u32 }, Unknown { fresh: u32 },
                         Busy { retry_after: Option<u32> }, Lost }
  pub fn public_answered(status: u16, body: Box<[u8]>, headers: &[(String, String)]) -> PublicHeard;
  ```

  `200` is `Document`, `404` `Unknown`, `503` `Busy`, anything else `Lost`; `headers` are the
  kept names, lower case, as `notes/contracts.md`, section 6 gives them.
- Step tests, engine end (`src/tests/public.rs`, a module line in `src/tests.rs`):
  `canonical_paths_parse_and_others_do_not` (each malformed case of client.md 19's list by
  name), `options_head_and_post_are_404_with_no_access_control_header`,
  `each_answer_has_its_status_and_exact_headers`, `a_late_answer_gets_the_rest_of_its_max_age`
  (decided 2.1 s ago with `max_age` 5: `max-age=2`; decided 9 s ago: `max-age=0`),
  `a_credential_on_the_route_is_never_read` (a cookie without the browser header and a bearer:
  the same `PublicRead` handed on, no `403`), `host_is_checked_before_the_path`,
  `public_calls_past_their_room_are_503_and_signed_in_calls_are_not`.
  Client end (`src/tests/public.rs`): `the_public_call_is_a_get_with_no_bearer_or_body`,
  `the_printed_target_is_the_canonical_path`, `freshness_reads_present_absent_and_malformed_headers`
  (`max-age=5` with `age: 2` gives 2, `age: 9` gives 0, no `cache-control` 0, `no-store` 0,
  `max-age=x` 0, `age: -1` 0, `retry-after: 1` gives `Some(1)`, `retry-after: soon` `None`),
  `statuses_map_to_their_answers`.
- World stories (client.md 19's mapping list) in `jig/tests/engine-clients/tests/suite/public_http.rs`
  (a module line in `main.rs`), through a peer method in `src/peer/public_http.rs` (a module
  line in `src/peer.rs`) that sends real HTTP bytes through the mount:
  `public_reads_with_no_credential_and_with_a_signed_in_cookie_are_answered_alike` (byte for
  byte), `each_public_answer_has_its_status_and_exact_headers`,
  `a_late_answer_carries_a_shortened_max_age`, `malformed_public_paths_are_404`,
  `options_head_and_post_are_404_with_no_access_control_header`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 4
  and 6.
- `jig/docs/design/protocol/client.md`, 17.4 (calls' room), 17.5, and section 19's Testing
  list; `jig/docs/design/protocol/browser.md`, 12.2 (`Host`).
- As they stand: `jig-protocol-engine-clients-http/src/{lib.rs, dispatch.rs, guards.rs,
  call.rs, credential.rs}` and `src/tests.rs`; `jig-client-protocol-http/src/{lib.rs, call.rs}`;
  `public-component`'s boundary types; `jig/tests/engine-clients/src/peer.rs`.

## Owns and may change

- Owns the files in the header.
- **The mount's `Settings` literals,** each edited only to add `max_age` (a test fixture's
  figure, or the component's `limits.public.max_age` where a service builds it):
  `jig/tests/engine-clients/src/fixtures.rs`, `jig/tests/client-protocol/src/fixtures.rs`,
  `jig/testing/jig-test-service/src/clients.rs`; temper (a required field):
  `temper/crates/temper-service/src/clients.rs`, `temper/crates/temper-service/src/tests/{fixtures.rs,
  sign_in_configuration.rs}`. Any other the compiler names: the same.
- `jig/tests/engine-clients/src/peer.rs` and `tests/suite/main.rs`: a module line each.
- **The services' room check:** `temper/crates/temper-service/src/clients.rs` (`server_requests`)
  and `jig/testing/jig-test-service/src/clients.rs` hand a claimed call to the mount only while
  `mount.room() > 0`, which would hold public calls behind the signed-in calls' room. Give the
  mount a pure `pub fn room_for(&self, head: &Head) -> u32` (or the name you choose, reported)
  that answers the public room for a public read's head and `room()` otherwise, and change
  those two checks to it, nothing else in either file; a story shows a public read answered
  while every other call's room is full, and a signed-in call answered while the public room is
  full.
- Pre-approved: if the server's response type adds a header the list above lacks (a
  `content-length`), keep it and say so; if `skein_http`'s call head gives the target with its
  query split off, refuse a call whose query is present all the same. If an `Unknown` must
  carry a decision's time for its `max-age`, it carries the whole `max_age` as above and the
  report says so.

## Steps

1. The parser and the route; the answers and headers; the room;
   `heavy cargo check -p jig-protocol-engine-clients-http --tests`.
2. The client end; `heavy cargo check -p jig-client-protocol-http --tests`.
3. The literals until `heavy cargo check -p jig-test-service -p temper-service -p jig-engine-clients-world -p jig-client-protocol-world --tests`
   passes.
4. The step tests, each run alone first
   (`heavy cargo nextest run -p jig-protocol-engine-clients-http --lib -E 'test(/public::/)'`,
   `-p jig-client-protocol-http --lib -E 'test(/public::/)'`), then the world's
   (`heavy cargo nextest run -p jig-engine-clients-world --test suite -E 'test(/public_http::/)'`).
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and
   `python3 -I -B temper/scripts/old_forms.py`; clippy on every package you changed; the
   focused tests of both crates, `jig-engine-clients-world`, `jig-client-protocol-world`,
   `jig-test-service`, `temper-service`; the citations check.
6. Commit (why: public reads over HTTP with fixed answers and freshness, and the client's
   call); push your branch.

## Report

As the implementer's text says, and also: each answer's exact header list as sent, the room's
accounting beside `Mount::room()`, and every literal outside the header's files.
