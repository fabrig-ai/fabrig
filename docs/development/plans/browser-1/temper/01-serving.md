# Session 01: temper serves the browser's bundle under jig's page policy

temper's clients' listener answers today what jig's mapping claims under `/jig`, and every other path not found, with
`Referrer-Policy: no-referrer` on the printed link's route (`client/README.md`, 8). `client/browser.md` 2.2 gives it
a bundle: `application.clients.bundle` names a directory of skein's bundler, which `temper serve` reads whole at start
within two new limits, and which the listener serves by six routes in order, under jig's page policy on every answer.
01.1 adds the key, the limits and start's checks, and the read; 01.2 the routes and their answers, in
`src/clients/bundle.rs`; 01.3 the policy, in `src/clients/policy.rs`, put in at `Server::answer`. Each needs the one
before it. temper 04.1's first story needs 01.2, and 04.5's attack stories 01.3 (decision 2). Everything here is proved
on the host (decision 4).

## Read first

- **temper:** `docs/design/client/browser.md` 2.2 (the routes, the page's policy, the guards, the bundle);
  `docs/design/protocol/configuration.md` 3 (`bundle`), 7 (the `clients.` prefix, the two limits, the checks at start),
  8 (every start), 10 (the example's `"bundle": null`); `docs/design/client/README.md` 8; `docs/design/domain/testing.md`
  2.1 (system worlds), 2.5 (the one-process world), 2.6 (the process tier).
- **jig:** `docs/design/protocol/browser.md` 12, 12.1 (the policy) and 12.2 (the guards) (decision 10). The code to copy:
  `testing/jig-test-service/src/server/bundle.rs` (`File`, `Bundle`, `Reply`, `kind`, `route`, `start`, `demand`,
  `send`); `testing/jig-test-service/src/server/policy.rs` (`FIELDS`, `add`); `testing/jig-test-service/src/server.rs`
  (`Server::answer`, the bundle's event arms); `testing/jig-test/src/bundle.rs` (`read`); the fixtures
  `testing/jig-test-service/src/tests/fixtures/bundle.rs` (`head`, `bundle`) and `testing/jig-test/src/tests.rs`
  (`fixture_bundle`).
- **skein:** `docs/design/ui-web.md` 4.3 (the page and the starter), 4.4 (serving a bundle: the cache fields, the closed
  list of types).
- **skein foundation:** `programming-model.md` 3 (step functions), 5.2 (lifecycle), 6.3 (the worst case), 10 (the Rust
  subset); `testing-strategy.md` 2.1 (step tests), 2.3 (system worlds), 10 (a test's shape).
- **Plan:** the README's "Rules of this plan" (the configuration's new key; Hotspots) and "Process tier";
  `code-organization.md` 3, 4, 5 and 6; decisions.md 2, 4, 6, 7 and 10.
- **Code:** `crates/temper-service/src/startup/application.rs` (`Clients`, `clients`, its two callers, `webhook_secret`'s
  nullable read); `src/startup.rs` (`Named`, `named`, `parse`); `src/limits.rs` (`Limits`, `Root`, `profile`, `server`);
  `src/limits/names.rs` (`apply`); `src/startup/checks.rs` (`limits`, `against`); `src/clients.rs` (`Clients`, `new`,
  `server_requests`, `unclaimed`, `worst_case`); `src/server.rs` (`Server::answer`, `done`); `src/system.rs`
  (`Service::adopt_store`, `memory`); `crates/temper/src/lib.rs` (`prepare`, `read_files`, `unread`, `path`); the tests'
  `src/tests/fixtures.rs`, `src/tests/fixtures/{startup,server,http}.rs`, `src/tests/{startup,checks,limits,server}.rs`;
  `crates/temper/tests/startup.rs` and `tests/support/`.

## What the design asks

- **configuration.md 3:** `application.clients.bundle`, a directory or `null`; `null` serves no browser. 01.1.
- **configuration.md 7:** `clients.bundle_files`, the page among them, and `clients.bundle_bytes`, `small`'s constants;
  both above 0 at start. 01.1, with decision 6's numbers.
- **browser.md 2.2, "The bundle":** read at start, whole, into memory, within the two limits; the start refused past
  them, or when the directory holds no page or no digest directory; after a restart, the old digest's files not found.
  01.1 reads; 01.2 answers.
- **browser.md 2.2, "The routes":** the six, in order, over skein's `ui-web.md` 4.4: the bundle's files immutable, the
  page and `/robots.txt` with an ETag, `no-cache` and `304`, each file typed by the closed list. 01.2.
- **browser.md 2.2, "The page's policy", and jig's browser.md 12.1:** jig's five fields on every answer. 01.3.
- **browser.md 2.2, "jig's example guards", and client/README.md 8:** jig's mapping's guards as they are, and the link's
  `GET` with no bundle not found with `Referrer-Policy: no-referrer`. Kept; from 01.3 the policy carries it.
- **`docs/design/domain/testing.md` 2.6:** `temper serve` as it ships; 01 changes its listener, so the process tier
  runs at the session's end.

## Choices this plan makes

- **The read is the binary's.** `temper-service` is `no_std` outside its tests and reads no file;
  `crates/temper/src/lib.rs`'s `prepare` reads the configuration's files (`read_files`). A new
  `crates/temper/src/bundle.rs` reads the directory after them, and the service adopts the result after `startup`, as
  it adopts its store (`Service::adopt_store`). The service holds the types, the closed list, the worst case, the
  routes and the answers, proved in its unit tests (decision 4); the binary's read is proved by its start tests.
- **The page counts among the files** (configuration.md 7), where jig-test's `read` leaves it out. Each file is read at
  the bytes left, so the file that passes `bundle_bytes` is the one refused. jig-test's 4 MiB a file and 256 entries go,
  as the design names neither: the entries are bounded at twice `bundle_files`, a path at `NAME_BYTES`, 1024, as
  jig-test's, and a directory is read in name order, so a refusal names the same file each run.
- **No bundle comes before the method.** browser.md 2.2 puts route 2 (no bundle: not found) before route 3 (`405`),
  where jig's `route` answers `405` first. temper's `route` takes the design's order.
- **The mapping's claim stays first in `server_requests`.** `bundle::route` routes only what `http::claims` leaves, so
  its `Answer` has no `Mount`.
- **The policy goes in at `Server::answer`,** as jig's test service's `Server::answer` adds it: every answer of the
  clients' listener passes there, the mapping's (its guards' `421`, `415` and `403` among them) and the bundle's. Only
  the clients' listener runs `crate::server::Server`.
- **`webhooks` shares `clients()`.** A `bundle: bool` argument reads the member for the clients' listener alone, so
  `application.webhooks.bundle` stays an unknown member.
- **`"bundle": null` in every configuration,** as configuration.md 10's example. The owned example keeps its
  `memory_bytes`; the bundle's worst case is about 17 MB at decision 6's numbers.
- **The terminal and the CLI read sixteen fields.** The policy adds five fields to every `/jig` answer, which
  `temper-client-shell-terminal`'s `native()` reads with `headers: 8`, as does `temper-client-shell-cli`'s copy, with
  which `temper ctl` reads `/jig` (`host.rs`, `signing.rs`; the real-loop plan's 05.1 wrote it, and its later steps
  read `/jig` through it). 01.3 makes each 16, as jig's `tests/client-protocol` fixture holds. 02.2's web shell copies
  only `client` and `view`, not the header count.

## Rules that bind it

- **The configuration's new key:** 01.1 adds `application.clients.bundle` to every configuration the repository holds
  (README, Rules; code-organization.md 4). The configuration fixtures and `application.rs` are hotspots.
- **Copied, not depended on:** no dependency of jig's test service or of jig-test; their bundle serving, read and policy
  are copied, citing the source (code-organization.md 5).
- **Sizes:** `src/clients.rs` (641 lines) stays under 650; `src/startup/application.rs` (708) gets the key and nothing
  else (code-organization.md 6).

## Increments

### 01.1 The bundle's key, its limits and its read at start
Scope: configuration.md 3, 7 and 8; browser.md 2.2, "The bundle": the key, the limits, start's checks and the read.
Needs: nothing at start: `tools/temper-repin` has landed (temper `8c02b3b9e`).
Files: `crates/temper-service/src/startup/application.rs` (`Clients.bundle`; `clients`'s `bundle` argument and its two
callers); `src/limits.rs` (`Clients`, `CLIENTS`, `Limits.clients`, `profile`); `src/limits/names.rs` (two arms);
`src/startup/checks.rs` (`limits`); `src/startup.rs` (`Named`, `named`); new `src/clients/bundle.rs`; `src/clients.rs`
(`pub mod bundle;`, the field `bundle`, `new`); `src/system.rs` (`Service::adopt_bundle`; `memory`); the binary's
`crates/temper/src/lib.rs` (`pub mod bundle;`, `prepare`) and new `crates/temper/src/bundle.rs`. The configurations,
each application `"clients"` literal `git grep -n '"clients"' origin/main` finds: `docs/examples/owned.json`,
`crates/temper/tests/support/configuration.rs` (`documents`), `tests/real-loop/src/fixtures.rs` (`documents`, real-loop
08.2's copy of it, landed in `94df159d3`), `tests/one-process/src/fixtures.rs`,
`crates/temper-service/src/tests/fixtures/startup.rs` (`APPLICATION`), `crates/temper-service/src/tests/fixtures.rs`
(the `Clients` literal gains `bundle: None`). Hotspots (README, Rules): real-loop 09 changes
`tests/real-loop/src/fixtures.rs`, and 10.1 and 10.2 `configuration.rs`'s neighbours `tests/support/mod.rs` and
`forgejo.rs`; forge-kit's temper 02.1, 03.1 and 05.2 change `owned.json`, the service's `src/tests/fixtures/` and
`tests/one-process`; whichever lands second keeps the other's lines. Tests: `src/tests/startup.rs` (and the text that
`a_missing_clients_member_is_named_by_its_path` replaces), `src/tests/checks.rs`, `src/tests/limits.rs`
(`an_override_sets_the_field_its_dotted_name_names`, rows 5 to 7), `crates/temper/tests/startup.rs`. Unchanged users:
`settings.rs`'s `clients`, `checks.rs`'s loopback check and `tests/checks.rs`'s `apply_shape` read `listener` alone; the
owned tier takes the key from `owned.json`: `tests/support/owned.rs`'s `application`, and `forgejo.rs`'s
`configuration`, which writes `owned::application` and reads only a previous configuration's listeners; the engine's
`clients` (`heartbeat_ms`, `link`) in `owned.json` and `src/tests/fixtures.rs` is another member; `profile`'s callers
(`fixtures/worker.rs`'s `worker_limits`) gain a field.
Shape: `clients` reads `bundle` with `read::required` before `read::finish` when its `bundle` argument is true, and a
non-null value with `config::read::text(document, at, 1_048_576)`, as `webhook_secret` is read; the clients' call passes
true, the webhooks' false. `Limits.clients` is `CLIENTS`, 128 and `16 << 20` (decision 6); `names::apply` gains
`b"clients.bundle_files"` and `b"clients.bundle_bytes"`. `checks::limits`, after its git loop, pushes
`against(problems, b"clients.bundle_files", b"a nonempty bundle")` when it is 0, and the same for `bundle_bytes`. `named`
passes the key and `limits.clients`. `clients/bundle.rs` holds `NAME_BYTES`, jig's `File` and `Bundle`, `kind` (jig's,
cited), `worst_case` (`bundle_bytes`, `Map::<Box<[u8]>, File>::worst_case(bundle_files)`, `bundle_files` names at
`NAME_BYTES`, the digest and the ETag) and `Clients::adopt_bundle`; `memory` adds `worst_case`. The binary's `read`
copies jig-test's `read`, cited, with this plan's bounds (Choices): its refusals read
`application.clients.bundle: <directory>/<entry>: <why>`, as `read_files` prints a file's, with jig-test's words,
`unread`'s for a read that fails, and `past clients.bundle_files` and `past clients.bundle_bytes`; the ETag is
jig-test's FNV-1a over the page. `prepare` reads after `read_files` when `named.bundle` is `Some`, and calls
`adopt_bundle` after `adopt_store`. `"bundle": null` goes into each configuration, `"bundle":null` where it is
written on one line.
Test shape: the service's in `src/tests/startup.rs`, through `fixtures::with` and `crate::named`. The reference test:

```rust
#[test]
fn the_configured_bundle_is_named_for_the_binary() {
    let document = fixtures::with(&fixtures::document(), b"application.clients.bundle", br#""bundle""#);
    let named = crate::named(&document).expect("named");
    assert_eq!(named.bundle.as_deref(), Some(b"bundle".as_slice()));
    assert_eq!(named.bundle_limits, crate::limits::Clients { bundle_files: 128, bundle_bytes: 16 << 20 });
}
```

Beside it: `a_missing_bundle_is_named_by_its_path` (`application.clients.bundle: missing`);
`a_webhook_listener_with_a_bundle_is_refused` (`fixtures::webhook_document()` with `application.webhooks`
`{"listener":{"socket":"127.0.0.1:7200"},"bundle":null}`: `application.webhooks.bundle: unknown member`);
`a_bundle_limit_of_zero_refuses_the_start` (in `tests/checks.rs`, two rows through `application.limits.overrides`, each
`must agree with a nonempty bundle`). The binary's, in `crates/temper/tests/startup.rs`, on a scratch directory with
jig-test's `fixture_bundle` copied (page 31 bytes, `boot.js` 7, `files/robots.txt` 24):
`a_bundle_directory_is_read_with_its_one_digest_directory` (`temper::bundle::read`: the digest, three files, the ETag
`"edaa28e315690734"`); `a_directory_that_is_no_bundle_refuses_the_start` (rows through `temper::prepare_in`: no page, no
digest directory, two digest directories, a file off the closed list, each refusal's line exactly);
`a_bundle_past_a_limit_refuses_the_start` (rows by `support::owned::configuration_value`: `bundle_files` 2 and
`bundle_bytes` 61, each naming `bundle/0123456789abcdef/files/robots.txt`). Ceilings: each unit test under 20 ms; each
start test as its file's others.
Proof: unit: the service's four, `cargo nextest run -p temper-service`. The binary's three run its read and `prepare_in`
in process on a scratch directory, `cargo nextest run -p temper --test startup`, beside
`serve_prepares_once_on_the_owned_example`, which shows the owned example starts with the bundle's worst case counted.
Don't: add to `application.rs` more than the key; read a file in `temper-service`; depend on jig-test; keep jig-test's
4 MiB a file; add a limit the design does not name; split `src/tests/fixtures.rs` (841 lines at the base).
Size: at most 240 lines (`crates/temper/src/bundle.rs` about 120, `clients/bundle.rs` about 60, the rest about 60).
Removes: nothing.

### 01.2 The clients' listener's routes
Scope: browser.md 2.2, "The routes", 2 to 6, over skein's `ui-web.md` 4.4; "The bundle": the old digest not found.
Needs: 01.1.
Files: `crates/temper-service/src/clients/bundle.rs` (`Answer`, `route`, `Reply`, the answers' start and chunks,
`Clients::serve_call`, `Clients::serve_event`); `src/clients.rs` (`unclaimed` becomes `replies`, a field `send`,
`server_requests`'s two arms, `worst_case`'s map); new `src/tests/bundle.rs` (`mod bundle;` in `src/tests.rs`); new
`src/tests/fixtures/bundle.rs` (`pub mod bundle;` in `src/tests/fixtures.rs`). Unchanged: `src/tests/server.rs`'s
`a_path_outside_jig_is_answered_not_found`, `a_get_of_the_printed_links_route_is_not_found_with_no_referrer` and
`a_host_off_the_list_is_answered_421_with_nothing_done`, which hold with no bundle; `system.rs`'s `memory`, which passes
the transport's limits to `clients::worst_case` already.
Shape: `route` is jig's without the mapping's claim, in the design's order: no bundle, `NotFound`; not `GET`,
`NotAllowed`; under `/bundle/`, `File` for a file of the bundle's digest, else `NotFound`; `/robots.txt`, `Root` when
the bundle holds `files/robots.txt`, else `NotFound`; a matching `If-None-Match`, `NotModified`; else `Page`, the link's
route among them. `serve_call` copies jig's `start`, pushing `Discard`, `Respond` and a first `Demand` through
`self.server.answer(call, …, &mut self.server_out)`, with `close: false`, and keeps the `Reply` in `replies`;
`serve_event` copies jig's arms (`Reply(Up::Room)` sends a chunk at `send`, `Done` and `Ended` end the reply, a failure
ends it), each end removing the reply and calling `self.server.done`. A `NotFound` to the link's `GET` keeps
`http::link_page()` until 01.3. `server_requests` calls `serve_call` for what `http::claims` leaves, and
`serve_event` for a call in `replies`.
`clients::worst_case` counts `Map::<Token, bundle::Reply>` and each reply's path at `transport.http.head`, in place of
`Map::<Token, ()>`.
Test shape: in `src/tests/bundle.rs`, with the fixture's `head` and `bundle` (jig's, plus `module.wasm` one byte past a
send, 16,385 bytes) and `body` (a peer's bytes past its head). The reference test, a unit test of `route`:

```rust
const ETAG: &[(&[u8], &[u8])] = &[(b"If-None-Match", b"\"0011223344556677\"")];
const ROUTES: [(bool, Method, &[u8], &[(&[u8], &[u8])], Answer); 10] = [
    (false, Method::Get, b"/", &[], Answer::NotFound),
    (false, Method::Post, b"/", &[], Answer::NotFound),
    (true, Method::Post, b"/bundle/0123456789abcdef/boot.js", &[], Answer::NotAllowed),
    (true, Method::Get, b"/bundle/0123456789abcdef/boot.js", &[], Answer::File),
    (true, Method::Get, b"/bundle/fedcba9876543210/boot.js", &[], Answer::NotFound),
    (true, Method::Get, b"/bundle/0123456789abcdef/missing.js", &[], Answer::NotFound),
    (true, Method::Get, b"/robots.txt", &[], Answer::Root),
    (true, Method::Get, b"/jig/v1/sign-in/link?secret=AAAA", &[], Answer::Page),
    (true, Method::Get, b"/chats/7", &[], Answer::Page),
    (true, Method::Get, b"/chats/7", ETAG, Answer::NotModified),
];
#[test]
fn each_head_is_routed_in_the_designs_order() {
    let bundle = fixtures::bundle::bundle();
    for (bundled, method, target, headers, answer) in ROUTES {
        let head = fixtures::bundle::head(method, target, headers);
        assert_eq!(route(bundled.then_some(&bundle), &head), answer, "{method:?} {target:?}");
    }
}
```

Beside it, each through `fixtures::started()`, `Service::adopt_bundle`, `fixtures::request` and `fixtures::answered`:
`a_bundle_file_is_served_immutable_with_its_kind` (`text/javascript`, `public, max-age=31536000, immutable`, the body);
`the_page_is_served_with_its_etag_for_revalidation` (`/chats/7`: `text/html; charset=utf-8`, `no-cache`, the ETag, the
page); `a_matching_etag_is_answered_not_modified` (`304`, no body); `a_file_longer_than_one_send_arrives_whole`
(`module.wasm`); `the_printed_links_route_serves_the_page` (with a bundle: `200`, the page, nothing reached the root).
Ceilings: the unit test under 20 ms; each world test under 0.5 s.
Proof: unit: `each_head_is_routed_in_the_designs_order`. world: the service's system world
(`docs/design/domain/testing.md` 2.1), the five above and `src/tests/server.rs`'s three,
`cargo nextest run -p temper-service`.
Don't: put a route or an answer in `clients.rs`; route the mapping's paths in `route`; answer a non-`GET` `405` with no
bundle; hold a bundle file twice.
Size: at most 150 lines (`clients/bundle.rs` about 140; `clients.rs` loses about 25 and gains about 15).
Removes: `unclaimed` and the unclaimed call's `404` with `close: true`.

### 01.3 jig's page policy on every answer
Scope: browser.md 2.2, "The page's policy"; jig's browser.md 12.1: the five fields on every answer of the clients'
listener: the page, the bundle's files, `304`, `404`, `405`, `/jig`'s answers, its guards' refusals among them.
Needs: 01.2.
Files: new `crates/temper-service/src/clients/policy.rs` (`pub(crate) mod policy;` in `src/clients.rs`);
`src/server.rs` (`Server::answer`); `src/clients/bundle.rs` (the link's `NotFound` loses `http::link_page()`);
`src/tests/server.rs`; `src/tests/fixtures/bundle.rs` (`POLICY`); `crates/temper-client-shell-terminal/src/limits.rs`
(`native()`'s `headers`); `crates/temper-client-shell-cli/src/limits.rs` (`native()`'s `headers`, its copy). Unchanged
users of the terminal's limits, which read `native()`: `crates/temper/src/open.rs`, `tests/one-process/src/fixtures.rs`
and `stories.rs` (`client_limits`), `tests/one-process/src/referee/client.rs`, `crates/temper/tests/support/person.rs`,
`crates/temper/tests/configuration.rs`, `crates/temper-client-tools/src/tests/limits.rs`. Unchanged users of the CLI's:
its `host.rs` (`native()`, `transport`), `signing.rs` (`settings`), `src/tests.rs` and `src/tests/fixtures/recovery.rs`,
and `pending.rs` and `src/tests/fixtures/local.rs`, which read the domain's limits alone (temper `fa7c66141`).
Neither shell has a unit test that reads `headers`, so neither gains one.
Shape: `policy.rs` copies jig's `policy.rs`, cited: `FIELDS` and `add`, which appends each field the response lacks, its
name compared ignoring case. `Server::answer` takes `mut request` and calls `crate::clients::policy::add` on a
`Request::Respond`'s response, as jig's test service's does. The terminal's `headers` becomes 16, and the CLI's.
Test shape: in `src/tests/server.rs`, the fixture's `POLICY` holding the five fields, names lowercased as `answered`
gives them, values from browser.md 2.2. The reference test:

```rust
const ANSWERED: [(bool, &[u8]); 8] = [
    (true, b"GET /chats/7 HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (true, b"GET /bundle/0123456789abcdef/boot.js HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (true, b"GET /robots.txt HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (true, b"GET / HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nIf-None-Match: \"0011223344556677\"\r\n\r\n"),
    (true, b"GET /bundle/fedcba9876543210/boot.js HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (true, b"DELETE /chats/7 HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (false, b"GET /jig/v1/sign-in/link?secret=AAAA HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n"),
    (false, b"POST /jig/v1/read HTTP/1.1\r\nHost: evil.test:7100\r\nContent-Type: application/octet-stream\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"),
];
#[test]
fn each_answer_carries_the_policys_five_fields() {
    for (bundled, bytes) in ANSWERED {
        let mut service = fixtures::started();
        if bundled {
            service.adopt_bundle(fixtures::bundle::bundle());
        }
        let mut peer = fixtures::request(&mut service, bytes);
        let (_, headers) = fixtures::answered(&mut service, &mut peer);
        for (name, value) in fixtures::bundle::POLICY {
            assert_eq!(fixtures::header(&headers, name), Some(value), "{name:?} on {bytes:?}");
        }
    }
}
```

Beside it: `the_policy_keeps_a_field_the_answer_already_holds` (a unit test of `add`: a response holding
`referrer-policy: same-origin` holds that one `Referrer-Policy` field after it). Ceilings: the world test under 0.5 s;
the unit test under 20 ms.
Proof: unit: `the_policy_keeps_a_field_the_answer_already_holds`; the CLI's tests, which read its `native()`,
`cargo nextest run -p temper-client-shell-cli`. world: `each_answer_carries_the_policys_five_fields`
(`cargo nextest run -p temper-service`); then the one-process world, whose terminal reads `/jig`'s answers with the
five fields:
`cargo nextest run -p temper-one-process-world -E 'test(=a_person_signs_in_by_the_printed_link_and_a_chat_is_answered)'`.
process: at the session's end, `scripts/proof-one-process.sh` in a checkout of temper's `origin/main`
(README, "Process tier"), where `temper_open_signs_in_prints_the_inbox_and_exits_on_q` reads them over `temper serve`'s
listener. The browser's own proof of the policy is temper 04.5's `a_framed_page_shows_nothing`.
Don't: add an origin or a directive to the policy (browser.md 2.2); add the policy anywhere but `Server::answer`; change
the webhooks' listener's answers; raise any limit of the terminal's or the CLI's `native()` but `headers`.
Size: at most 45 lines (`policy.rs` about 30, `server.rs` about 5, the terminal 1, the CLI 1).
Removes: `bundle.rs`'s `http::link_page()` for the link's `NotFound`, which the policy's `Referrer-Policy` gives.

## Interface sketches

The service owns the bundle once it is read: the binary reads and checks it, the service holds it whole and answers from
it, and its worst case is in the start's memory check (programming-model.md 6.3). The sketch compiles under temper's
workspace lints, `clippy.toml` and `rustfmt.toml`, with `clippy --all-targets -D warnings`, in a scratch crate with
stand-ins for `config::Address`, `Document`, `Path`, `Problems`, `Map`, `http::Call`, `Response` and `Service`, and stub
bodies.

```rust
// crates/temper-service/src/startup/application.rs

/// The clients' listener (protocol/configuration.md, section 3).
#[derive(Debug)]
pub struct Clients {
    pub listener: config::Address,
    /// `application.clients.bundle`: the bundle's directory; `None` serves no browser.
    pub bundle: Option<Box<[u8]>>,
}

/// Read a listener with jig's address reader, refusing non-socket forms; and `bundle` when asked (the clients').
#[must_use]
pub fn clients(document: &Document, at: u32, path: &Path, bundle: bool, problems: &mut Problems) -> Option<Clients>;

// crates/temper-service/src/limits.rs

/// The clients' listener's own limits: the browser's bundle (protocol/configuration.md, 7).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Clients {
    /// The bundle's files, its page among them.
    pub bundle_files: u32,
    /// The bundle's files' bytes in all, held in memory while `temper serve` runs.
    pub bundle_bytes: u32,
}

pub struct Limits {
    // … the other owners' limits, unchanged.
    pub clients: Clients,
}

// crates/temper-service/src/startup.rs

pub struct Named {
    // … the other members, unchanged.
    /// `application.clients.bundle`, for the binary's read.
    pub bundle: Option<Box<[u8]>>,
    /// The bundle's limits, overrides applied.
    pub bundle_limits: crate::limits::Clients,
}
```

```rust
// crates/temper-service/src/clients/bundle.rs

/// A bundle path's bound (jig's testing/jig-test/src/bundle.rs).
pub const NAME_BYTES: u32 = 1024;

/// One digest-directory file, typed by the closed list (skein's ui-web.md, 4.4).
#[derive(Debug)]
pub struct File {
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
}

/// The bundle read whole at start (client/browser.md, 2.2).
#[derive(Debug)]
pub struct Bundle {
    pub page: Box<[u8]>,
    pub etag: Box<[u8]>,
    pub digest: Box<[u8]>,
    pub files: Map<Box<[u8]>, File>,
}

/// A path's type by its extension, from the closed list; `None` off it.
#[must_use]
pub fn kind(path: &[u8]) -> Option<&'static [u8]>;

/// The bundle's bytes at its limits.
#[must_use]
pub fn worst_case(limits: &crate::limits::Clients) -> Option<u64>;

/// The answer to a head jig's mapping leaves (client/browser.md, 2.2, routes 2 to 6).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) enum Answer {
    File,
    Page,
    NotModified,
    Root,
    NotFound,
    NotAllowed,
}

pub(crate) fn route(bundle: Option<&Bundle>, head: &http::Call) -> Answer;

// crates/temper-service/src/system.rs

impl Service {
    /// Hold the bundle the binary read at start, served on the clients' listener.
    pub fn adopt_bundle(&mut self, bundle: crate::clients::bundle::Bundle);
}

// crates/temper/src/bundle.rs

/// Read `application.clients.bundle`'s directory whole, within its limits; `Err` is the refusal's line.
pub fn read(directory: &Path, configured: &[u8], limits: &temper_service::limits::Clients) -> Result<Bundle, String>;
```

```rust
// crates/temper-service/src/clients/policy.rs

/// jig's reference policy (jig's protocol/browser.md, 12.1), on every answer of the clients' listener.
pub const FIELDS: [(&[u8], &[u8]); 5] = [
    (b"Content-Security-Policy", b"default-src 'none'; script-src 'self' 'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self'; connect-src 'self'; base-uri 'none'; form-action 'none'; frame-ancestors 'none'"),
    (b"X-Content-Type-Options", b"nosniff"),
    (b"Cross-Origin-Opener-Policy", b"same-origin"),
    (b"Cross-Origin-Resource-Policy", b"same-origin"),
    (b"Referrer-Policy", b"no-referrer"),
];

/// Append each field the response does not hold, its name compared ignoring case.
pub(crate) fn add(response: &mut skein_http::server::Response);
```

01.1 lands `application::Clients.bundle`, `clients`'s argument, `limits::Clients`, `Named`'s two members, `File`,
`Bundle`, `kind`, `worst_case`, `Service::adopt_bundle` and the binary's `read`; 01.2 `Answer` and `route`; 01.3
`FIELDS` and `add`.

## Questions to raise when reached

- If `serve_prepares_once_on_the_owned_example` is refused "the worst case passes memory_bytes" once 01.1 counts the
  bundle, report the bytes it names and ask before raising the owned example's `engine.memory_bytes` (setup.md 8.1
  states its figure).
- If an answer with the policy is refused for its head under `limits::server`'s `response: 1024`, report the answer's
  head bytes and ask: jig's test service answers within the same 1024.

## Done when

Each in `~/src/rust/temper`, after `git fetch`:
- `git show origin/main:docs/examples/owned.json | grep -q -F '"bundle": null'`
- `git show origin/main:tests/one-process/src/fixtures.rs | grep -q -F '"bundle":null'`
- `git show origin/main:crates/temper/tests/support/configuration.rs | grep -q -F '"bundle":null'`
- `git show origin/main:tests/real-loop/src/fixtures.rs | grep -q -F '"bundle":null'`
- `git show origin/main:crates/temper-service/src/tests/fixtures/startup.rs | grep -q -F '"bundle":null'`
- `git show origin/main:crates/temper-service/src/startup/application.rs | grep -q -F 'b"bundle"'`
- `git show origin/main:crates/temper-service/src/limits/names.rs | grep -q -F 'b"clients.bundle_files"'`
- `git show origin/main:crates/temper-service/src/limits/names.rs | grep -q -F 'b"clients.bundle_bytes"'`
- `git show origin/main:crates/temper-service/src/startup/checks.rs | grep -q -F 'a nonempty bundle'`
- `git show origin/main:crates/temper/src/bundle.rs | grep -q -F 'application.clients.bundle'`
- `git show origin/main:crates/temper-service/src/clients/bundle.rs | grep -q -F 'public, max-age=31536000, immutable'`
- `git show origin/main:crates/temper-service/src/clients/policy.rs | grep -q -F "frame-ancestors 'none'"`
- `git show origin/main:crates/temper-service/src/server.rs | grep -q -F 'policy::add'`
- `git show origin/main:crates/temper-client-shell-terminal/src/limits.rs | grep -q -E 'headers: 16, +// Incoming HTTP fields'`
- `git show origin/main:crates/temper-client-shell-cli/src/limits.rs | grep -q -E 'headers: 16, +// Incoming HTTP fields'`
