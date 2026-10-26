# Session 18: the test application's server owns the listener, mounts `/jig`, serves its bundle

jig's test application's server becomes the example of an
application's server (`protocol/service.md`, 7; `protocol/browser.md`,
12): its listener is a member of `jig-test`'s own configuration, and
`engine.clients` in jig's part names none; it mounts the clients'
component at `/jig` through jig 17's engine end and answers everything
else itself: the bundle under `/bundle/`, the page at each other `GET`
and at the printed link's route, one root file, and the reference page
policy on each answer. The bundle is data `jig-test` reads from the
directory its configuration names, or none, which serves no browser. The
bundle's building is jig 19's; this session serves a bundle a test
makes.

## Read first

- jig's `docs/design/protocol/service.md`: 1, 2 (the bullet "The
  clients' traffic, through the application's server"), 3 (step 4), 7
  (the bullets "Its server" and "The reference binary"), 10 (the bullet
  "A client on loopback").
- jig's `docs/design/protocol/configuration.md`: 3 (the row `clients`).
- jig's `docs/design/protocol/browser.md`: 4.2 (the bullet "jig's test
  application's bundle"), 4.3, 9 (the second bullet), 12, 12.1, 12.2,
  19 (the table's rows on the referrer policy and the link's route).
- skein's `docs/design/ui-web.md`: 4.4 (serving a bundle); skein's
  `examples/ui/server/src/{routes.rs, bundle.rs}`, the serving contract
  in code (89 and 117 lines).
- `findings/server-and-browser.md`: 2.
- jig 17's session file, its sketches (`jig_test_service::server`, the
  mount's `claims` and `link_page`).
- skein's `programming-model.md`: 4.2, 8, 10 (by number only).

## What the design asks

- **The listener is the application's** (`configuration.md`, 3,
  `clients`): "The listener, its certificate and key, and what serves a
  browser are the application's service's own members". jig's
  `engine.clients` keeps the heartbeat and whether the link is given.
- **The startup's check** (`service.md`, 3, step 4): "Where configuration
  enables the printed link, the startup first checks that the clients'
  listener, a member of the application's own, is on a loopback address,
  and refuses the start, naming both".
- **Its server** (`service.md`, 7): "it mounts the clients' component at
  `/jig` with jig's default HTTP mapping and its example guards whole,
  its `Host` allow-list the address its printed link names; and it
  serves jig's test application's bundle itself, as skein's `ui-web.md`,
  4.4 says, under `/bundle/`, its page at every other GET and at the
  printed link's route, with the reference page policy (`browser.md`,
  12.1). The bundle is data its startup gives it, or none, which serves
  no browser."
- **Serving a bundle** (skein's `ui-web.md`, 4.4): the digest directory
  under the prefix with `Cache-Control: public, max-age=31536000,
  immutable`, any other path under the prefix not found; the page at
  each `GET` of a path the server does not reserve, with an `ETag` and
  `Cache-Control: no-cache`, a matching `If-None-Match` answered `304`;
  "a root file the application names, one of the bundle's `files/`,
  cached as the page is"; each file's type from the closed list; nosniff
  on each answer.
- **The reference policy** (`browser.md`, 12.1), on each answer of the
  origin: `Content-Security-Policy: default-src 'none'; script-src 'self'
  'wasm-unsafe-eval'; style-src 'self'; img-src 'self'; font-src 'self';
  connect-src 'self'; base-uri 'none'; form-action 'none';
  frame-ancestors 'none'`, `X-Content-Type-Options: nosniff`,
  `Cross-Origin-Opener-Policy: same-origin`,
  `Cross-Origin-Resource-Policy: same-origin`, `Referrer-Policy:
  no-referrer`.

## Choices this plan makes

- **The server's members** are `application.server` in `jig-test`'s
  configuration: `listener` (an address, as jig's `check::address`
  reads one) and `bundle` (a directory, optional). A listener off
  loopback is refused at the start, naming `application.server.listener`:
  TLS for the clients' listener waits for skein's TLS server side (jig's
  `STATUS.md`, "Between hosts", not planned), so no certificate or key
  member is read yet.
- **The root file** is `robots.txt`, the bundle's `files/robots.txt`,
  served at `/robots.txt` when the bundle has it (the test shell's
  bundle has it from jig 19.4).
- **The mount's answers carry the reference policy too:** the server
  adds the policy's fields the mount's answer lacks (the mount sends
  `Cache-Control: no-store` and nosniff, jig 17.5), as "a header added to
  its answers" (`browser.md`, 3.5, step 10).
- **The link's route's page** carries `Referrer-Policy: no-referrer`
  through the reference policy, which sends it on each answer; the
  server adds the mount's `link_page()` only to an answer that lacks a
  `Referrer-Policy`, which in jig's test application is none.
- **A method other than `GET` outside the mount is `405`,** as skein's
  example answers it.
- **The bundle is read by `jig-test`** (ordinary Rust), after skein's
  `examples/ui/server/src/bundle.rs:46-117`, and handed to the service
  after its startup (`Service::serve(bundle)`), as `adopt_store` and
  `adopt_signals` are, so the fifteen callers of `startup` at the base (`logs/w8/startup-callers.txt`)
  stay as they are.

## Rules that bind it

- **jig's configuration names no listener for the clients:** after
  18.1, `git -C <worktree> grep -n -E 'clients\.listener|"clients"\]\["listener'
  -- crates testing tests` prints nothing (README, "Done when", 6 keeps
  the component free of a listener; this keeps the configuration free).
  18.1's `an_engine_clients_member_naming_a_listener_is_refused` names
  the path by its segments (`fixtures::path(&[b"engine", b"clients",
  b"listener"])`) and writes its JSON as `"clients": {"listener": ...}`,
  which neither pattern matches.
- **jig never names the repository downstream of it:** the check of
  code-organization.md, 6, quoted: "`git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end". The repository downstream of jig moves its own `clients.listener`
  (10 lines in 5 files, `logs/w8/users-18.1.txt`) at its repin after 18.1.
- **Limits come from the shipped profile's builder:** the server's limits
  are `jig_test_service::limits::for_profile`'s; a test that needs
  another value names it beside the profile's, with the reason.

## Corrections after the ready check of jig-18.1 (ruling 308)

Checked at skein `a0c3a66`, jig `6605e0564`, temper `90a3f7048`; where
this section and the increments below differ, this section holds. jig 17
lands before 18.1 and moves lines again: re-read each line named here at
the session's start.

18.1:
- Lines that moved: `crates/jig-protocol-config/src/tests/fixtures.rs:63`
  (not 59), `testing/jig-test-service/src/tests/fixtures.rs:54` (not 50),
  `tests/channel/src/fixtures.rs:84` (not 79),
  `tests/client-protocol/src/native/config.rs:48` (not 44); `hosts.rs:53`,
  `:73` and `testing/jig-test/tests/support/mod.rs:60` are right.
- Files gain `testing/jig-test-service/src/tests/clients.rs:258`, which
  calls `Clients::new` with the listener.
- `jig_protocol_config::check::address` is `pub(crate)` in a private
  module: the startup reads the listener with the public
  `jig_protocol_config::address::read`. The loopback check has no public
  form: write it in `jig-test-service`'s startup, beside its one user.
- The three JSON fixtures (`tests/channel/src/fixtures.rs:84`,
  `tests/client-protocol/src/native/config.rs:48`,
  `testing/jig-test/tests/support/mod.rs:60`) each gain
  `"server":{"listener":{"socket":"127.0.0.1:7100"}}` in `application`
  as they lose `engine.clients.listener`: without it 68 default and 2
  fuzzy tests fail; with it, all pass (the check ran it).
- "Rules that bind it": temper's code names `clients.listener` in 7 lines
  of 5 files, beside its configurations' JSON; temper 05.2 moves them
  all (its Files), not jig. Drop the count "10 lines in 5 files".

18.2:
- `fixtures::head` exists nowhere: 18.2 adds it to
  `testing/jig-test-service/src/tests/fixtures.rs`.
- Files gain `testing/jig-test-service/Cargo.toml` (the `skein-http`
  dependency, if jig 17 has not added it).
- The sketch's `Server::new(limits: &Limits)` and `fn answer(&self, call:
  &Call) -> Answer` are not jig 17.2's, which lands first and which temper
  05.1 copies: keep 17.2's `fn new(listener: skein_io::Addr, limits:
  &Limits)`, `fn answer(&mut self, call: Token, request:
  skein_http::server::Request, out: &mut Queue<Routed>)`, `fn up`,
  `fn done` and `fn listening`.
  18.2's decision is `fn route(&self, head: &Call) -> Answer`, which
  17.2's `answer` calls; `serve(&mut self, bundle: Bundle)` is 18.2's.
- `Answer` derives `Clone, Copy, Debug, PartialEq, Eq`; the routing
  table's rows have a `type Row` alias (clippy's `type_complexity`).

18.3:
- skein's bundle refusals name no file: "each naming the file" is this
  increment's own work, not a copy.
- The refusal of a type off the closed list is skein's private `of()`:
  write its like in `bundle.rs`.
- The test scratch directory is `tests.rs:4`'s `scratch(case)`, on
  `std::env::temp_dir`, not `tempfile`.
- Files gain `testing/jig-test/src/lib.rs`'s `mod bundle;`.

## The mount's stream end (ruling 327)

- **18.2's Files gain** `crates/jig-protocol-engine-clients-http/src/stream.rs`
  and `crates/jig-protocol-engine-clients-http/src/tests.rs` (with its
  `tests/fixtures.rs`). `Call::end` (`stream.rs:67-70`) sends
  `sse::Request::Finish` whenever a writer exists, and skein's SSE writer
  refuses it while a record is being written (`writer.rs:248` at skein
  `0f093bb`, "a Finish while an event is being written"), as it refuses
  a second Finish and one after the stream failed (:242-250). 18.2 sends
  the Finish only while `stream.writer.waiting()` is `sse::Waiting::Above`,
  holds it while it is `Room` and sends it once the writer is idle again,
  and sends none at `Close` or `Nothing`; the shape is 18.2's, within
  these Files.
- **Its regression test** is 18.2's
  `ending_a_mount_stream_while_its_record_waits_for_room_does_not_panic`,
  in the mount crate's `tests.rs` where its fixtures reach a `Call`, or
  else where 18.2 wrote it (`testing/jig-test-service/src/tests/server.rs`).
  It fails before the repair and passes after it.
- **Size:** the repair and its test, at most 40 lines together, count
  apart from 18.2's 250.

## Increments

### 18.1 The server's listener in `jig-test`'s own configuration

Scope: `application.server.listener` read by the test service's startup
(`configuration.md`, 3; `service.md`, 7); `engine.clients` without a
listener (`jig_protocol_config::Clients { heartbeat, link }`); the
startup's loopback check on the server's listener (`service.md`, 3,
step 4).
Needs: nothing beyond the session's start (jig 17's
`jig_test_service::server`, which owns the listener, is on main).
Files: `crates/jig-protocol-config/src/{clients.rs, lib.rs}`,
`crates/jig-protocol-config/src/tests/{fixtures.rs:59, hosts.rs:53, :73}`
(the case `a_clients_listener_on_a_public_address_is_refused` of the table
test `hosts_configuration_problems_name_the_invalid_fields` leaves that
table for the test service's startup tests, as a test of its own, its
name kept);
`testing/jig-test-service/src/{startup.rs, clients.rs:69, server.rs}`,
`testing/jig-test-service/src/tests/{fixtures.rs:50, startup.rs}`; the
JSON fixtures that name `engine.clients.listener`:
`testing/jig-test/tests/support/mod.rs:60`, `tests/channel/src/fixtures.rs:79`,
`tests/client-protocol/src/native/config.rs:44` (seven jig places with
`crates/jig-protocol-config/src/clients.rs:26` and the four JSON
fixtures, `logs/w8/users-18.1.txt`). Shared: `crates/jig-protocol-config/src/`
(README, "Shared files").
Shape:
- `jig_protocol_config::Clients { pub heartbeat: Duration, pub link:
  bool }`; `clients::parse` reads no `listener`, and an `engine.clients`
  that names one is refused as an unknown member, as the object's
  `finish` refuses any.
- `jig_test_service::startup` reads `application.server { listener,
  bundle }`: `listener` by `jig_protocol_config::check::address`, `bundle`
  as a text of at most 4096 bytes that the service keeps unread (the
  binary reads it, 18.3). A link enabled (`engine.clients.link`) with a
  listener off loopback is refused, its `Problems` naming
  `application.server.listener` and `engine.clients.link`; a listener
  off loopback is refused in any case, naming
  `application.server.listener` (no TLS yet).
Test shape: unit tests in `testing/jig-test-service/src/tests/startup.rs`
on `fixtures::engine_document()` (`tests/fixtures.rs`, whose `clients`
member loses `listener` and whose `application` gains `"server":
{"listener": {"socket": "127.0.0.1:7100"}}`). The reference test is jig
17.2's, moved there from the engine-clients world, its expected paths
now the server's member:
```rust
#[test]
fn a_link_on_a_non_loopback_listener_refuses_the_start() {
    let document = fixtures::with_server_listener(br#"{"socket": "192.0.2.1:7100"}"#);
    let found = crate::startup(&document, &fixtures::files(&document), None, &fixtures::seeds(), None, None)
        .err()
        .expect("startup refused");
    assert_eq!(
        fixtures::paths(&found),
        [fixtures::path(&[b"application", b"server", b"listener"]), fixtures::path(&[b"engine", b"clients", b"link"])]
    );
}
```
Tests: `a_link_on_a_non_loopback_listener_refuses_the_start` (its paths
changed, its name kept), `a_clients_listener_on_a_public_address_is_refused`
(the case moved from `jig-protocol-config`'s table, now a test; it names
`application.server.listener` now, its one changed literal; the table
test it leaves keeps its three other cases, and passes at the base,
`logs/w8/base-config-test.txt`),
`an_engine_clients_member_naming_a_listener_is_refused`,
`the_server_listens_where_the_application_member_says`. Each under 20 ms.
Proof: the protocol world reads the new member: `cargo nextest run -p jig-client-protocol-world -E
'test(a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat)'`
(or the name jig 16 gave it), once (0.14 s at the base): the native
client reaches the service at the listener `application.server` names.
Don't: read a certificate or a key; keep `listener` in jig's `Clients`;
change the heartbeat's or the link's members.
Size: at most 100 lines.
Removes: `Clients.listener`.

### 18.2 The server's own answers: the bundle, the page, a root file, the policy

Scope: the server answers each call the mount does not claim, as skein's
`ui-web.md`, 4.4 says, with the bundle it was given or none: the digest
directory under `/bundle/`, immutable; the page at each other `GET`, at
the link's route among them, with its `ETag`, `no-cache` and `304`; the
root file `/robots.txt`; `404` with no bundle, `405` for another method;
the reference policy's fields on each answer, the mount's included
(`browser.md`, 12 and 12.1).
Needs: 18.1.
Files: `testing/jig-test-service/src/server.rs`, new
`testing/jig-test-service/src/server/{bundle.rs, policy.rs}` (after
skein's `examples/ui/server/src/routes.rs` and the `Bundle` half of
`bundle.rs`, in step code), `testing/jig-test-service/src/lib.rs`
(`Service::serve`), `testing/jig-test-service/src/tests/{server.rs,
fixtures.rs}` (the crate's one fixtures module, code-organization.md, 5).
Shape (sketch below): `server::Bundle { page, etag, digest, files:
Map<Box<[u8]>, File> }`, `server::File { kind: &'static [u8], bytes:
Box<[u8]> }`; `pub fn kind(path: &[u8]) -> Option<&'static [u8]>`, the
closed list of skein's `ui-web.md`, 4.4; `Server::serve(&mut self,
bundle: Bundle)`; `Server::answer(&self, call: &Call) -> Answer` with
`enum Answer { Mount, File, Page, NotModified, Root, NotFound,
NotAllowed }`, the decision written as data so it is testable alone;
`policy::FIELDS: [(&[u8], &[u8]); 5]`, the reference policy's five
fields, added to each answer that lacks one; `Service::serve(&mut self,
bundle: server::Bundle) -> Result<(), Refused>`, refused once the loop
has started.
Test shape: unit tests in `testing/jig-test-service/src/tests/server.rs`
on a fixture `fixtures::bundle()` (a page `<!doctype html><title>t</title>`,
digest `0123456789abcdef`, files `boot.js` and `files/robots.txt`) and
17.2's `fixtures::server()`; the answers' heads read by
`fixtures::head(&mut out, call)`. A table test for the decision, its rows
in a `const`:
```rust
const ANSWERS: &[(Method, &[u8], &[(&[u8], &[u8])], Answer)] = &[
    (Method::Get, b"/", &[], Answer::Page),
    (Method::Get, b"/tasks/7", &[], Answer::Page),
    (Method::Get, b"/jig/v1/sign-in/link?secret=abc", &[], Answer::Page),
    (Method::Post, b"/jig/v1/read", &[], Answer::Mount),
    (Method::Get, b"/jig/v1/stream", &[], Answer::Mount),
    (Method::Get, b"/bundle/0123456789abcdef/boot.js", &[], Answer::File),
    (Method::Get, b"/bundle/fedcba9876543210/boot.js", &[], Answer::NotFound),
    (Method::Get, b"/robots.txt", &[], Answer::Root),
    (Method::Get, b"/", &[(b"if-none-match", b"\"0011223344556677\"")], Answer::NotModified),
    (Method::Post, b"/", &[], Answer::NotAllowed),
];
#[test]
fn each_path_is_answered_by_its_part_of_the_server() {
    let server = fixtures::serving(fixtures::bundle());
    for (method, target, headers, expected) in ANSWERS {
        assert_eq!(server.answer(&fixtures::head(*method, target, headers)), *expected, "{method:?} {target:?}");
    }
}
```
(the fixture's page `ETag` is the literal `"0011223344556677"`, set in
`fixtures::bundle()`, not computed by the code under test).
Tests, each under 20 ms: `each_path_is_answered_by_its_part_of_the_server`,
`with_no_bundle_a_page_get_is_404`,
`a_bundle_file_is_immutable_and_typed_by_its_extension`,
`the_page_is_revalidated_by_its_etag`,
`each_answer_carries_the_reference_policy_s_five_fields`,
`the_mount_s_answer_gains_the_policy_but_keeps_its_own_fields`,
`the_link_route_s_page_carries_no_referrer`.
Proof: unit layer here; the process tier serves a real bundle in 18.3,
which connects this increment's answers to a running `jig-test`. The
protocol world's mapping answers gain the policy's fields, which its
native client ignores: `cargo nextest run -p
jig-client-protocol-world -E 'test(a_request_whose_answer_was_lost_is_made_once)'`,
once (0.55 s at the base).
Don't: serve a file whose extension is off the closed list (the bundle
refuses it at read, 18.3); answer the mount's paths; add a field the
mount's answer already has.
Size: at most 250 lines.
Removes: nothing.

### 18.3 `jig-test` reads its bundle and serves it

Scope: `jig-test` reads the directory `application.server.bundle` names,
after skein's `bundle.rs` (the page, the one digest directory, each file
under it, bounded), and hands the service the bundle before its loop
(`service.md`, 7: "Its startup reads the bundle there and gives it, its
page included, to its server as data"); none named serves no browser.
Needs: 18.2.
Files: new `testing/jig-test/src/bundle.rs` (ordinary Rust); `testing/jig-test/src/lib.rs`
(`prepare` reads `application.server.bundle` and calls
`service.serve`); `testing/jig-test/tests/process.rs` and
`testing/jig-test/tests/support/mod.rs` (a configuration with no workers
and a fixture bundle, for the process test below).
Shape: `jig_test::bundle::read(directory: &Path) -> Result<server::Bundle,
String>`, with skein's bounds (`FILE_BYTES` 4 MiB, `BUNDLE_BYTES` 16
MiB, 128 files, 256 entries) and skein's refusals (no `index.html`, more
than one digest directory, a digest not 16 lowercase hex digits, a type
off the closed list, a path that is not plain), each naming the file;
the page's `ETag` is the quoted FNV-1a of its bytes, as skein's example
computes it.
Test shape: unit tests in `testing/jig-test/src/tests.rs` (beside its
46 lines at the base) on a scratch directory (`tempfile`, already a
dev-dependency of `jig-test`'s tests), each under 20 ms:
`a_bundle_directory_is_read_with_its_one_digest_directory`,
`a_file_off_the_closed_list_refuses_the_bundle_naming_it`,
`a_second_digest_directory_refuses_the_bundle`. And one process test, in
the `process` profile (outside the gate, `.config/nextest.toml`), the
reference:
```rust
#[test]
fn jig_test_serves_its_bundle_s_page_under_the_reference_policy() {
    let directory = support::scratch();
    support::browser_configuration(directory.path(), &support::fixture_bundle(directory.path()));
    let mut binary = support::start(directory.path());
    let link = support::printed_link_line(&mut binary);
    let (status, headers, body) = support::get(&link, b"/tasks/7");
    assert_eq!(status, 200);
    assert_eq!(support::field(&headers, "content-security-policy"), Some(support::REFERENCE_POLICY));
    assert_eq!(body, support::FIXTURE_PAGE);
}
```
(`support::get` is a GET over `std::net::TcpStream` to the link's
`host:port`; `REFERENCE_POLICY` is the literal of `browser.md`, 12.1).
Proof: the process layer, once, by the author, through the build
lane: `cargo nextest run -p jig-test --profile process -E
'test(jig_test_serves_its_bundle_s_page_under_the_reference_policy)'`
(about 5 s): the running binary answers a path of the reference UI with
the bundle's page under the reference policy. Nothing else is run.
Don't: link `skein-ui-bundle` into `jig-test`; read the bundle in the
service (step code reads no file); serve a page with no bundle.
Size: at most 150 lines.
Removes: nothing.

## Interface sketches

Compiled at the base in the scratch crate of jig 17's sketches, `cargo clippy -p jig-sketch-w8 --all-targets -- -D warnings`, which
passed (`logs/w8/sketch-17-18.txt`), with trivial bodies.

```rust
// testing/jig-test-service/src/server.rs
/// The server's own bounds, the application's (jig 17.2 moves them from jig's `CLIENTS`).
pub struct Limits {
    /// Connections open at once.
    pub connections: u32,
    /// Each connection's skein-http server.
    pub http: skein_http::server::Limits,
    /// A connection silent past this is closed.
    pub stall: Duration,
}
/// What the server hands on: io, or a call for the mount.
pub enum Routed {
    /// For io: the listener, the connections.
    Io(skein_io::Request),
    /// A call `claims` took, by the server's name for it.
    Call { call: Token, head: Call },
    /// What that call's connection said next, until `Done`.
    Http { call: Token, event: skein_http::server::Event },
}
/// How the server answers a call's head (browser.md, 12; skein's ui-web.md, 4.4).
pub enum Answer {
    /// The mount's: under its prefix, but a `GET` of the link's route.
    Mount,
    /// A file of the digest directory, immutable.
    File,
    /// The page, revalidated by its `ETag`; at the link's route too.
    Page,
    /// `304` for a matching `If-None-Match`.
    NotModified,
    /// The root file, from the bundle's `files/`.
    Root,
    /// No such path, or no bundle.
    NotFound,
    /// A method other than `GET` outside the mount.
    NotAllowed,
}
impl Server {
    pub fn new(limits: &Limits) -> Self;
    /// The bundle the startup read; none serves no browser.
    pub fn serve(&mut self, bundle: Bundle);
    pub fn answer(&self, call: &Call) -> Answer;
    pub fn up(&mut self, event: skein_io::Event, out: &mut Queue<Routed>);
}
// testing/jig-test-service/src/server/bundle.rs
/// One file of the digest directory, its type from the closed list.
pub struct File {
    /// `Content-Type`, by its extension (skein's ui-web.md, 4.4).
    pub kind: &'static [u8],
    pub bytes: Box<[u8]>,
}
/// The bundle the startup read, as data (protocol/service.md, 7).
pub struct Bundle {
    /// `index.html`, answered at each GET the server does not reserve.
    pub page: Box<[u8]>,
    /// The page's `ETag`, quoted.
    pub etag: Box<[u8]>,
    /// The digest directory's name, 16 lowercase hex digits.
    pub digest: Box<[u8]>,
    /// The digest directory's files by their paths under it, `files/robots.txt` among them.
    pub files: Map<Box<[u8]>, File>,
}
```

## Questions to raise when reached

- None the design leaves open that these increments need.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig log origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-18\.' | sort -u | wc -l` prints 3.
- `git -C ~/src/rust/jig grep -n -E 'clients\.listener|"clients"\]\["listener'
  origin/main -- crates testing tests` prints nothing.
- In a worktree of jig at `origin/main`, `cargo nextest run -p
  jig-test --profile process -E
  'test(jig_test_serves_its_bundle_s_page_under_the_reference_policy)'`
  passes one test.
