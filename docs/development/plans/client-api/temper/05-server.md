# Session 05: `temper-service` owns the clients' listener and mounts jig's component

jig's clients' component stops owning a listener (jig 17): the
application's server owns it, and mounts the component with jig's default
HTTP mapping's engine end, `jig-protocol-engine-clients-http`, under a
prefix it chooses. temper follows: `temper-service` gets a server child
that owns the clients' listener and its connections, mounts the mapping
under `/jig` with its example guards, answers the printed link's `GET`
and each path outside `/jig` not found, and prints the link from the
component's secret (temper's `client.md`, 8; `protocol.md`, 6). Then,
when jig's configuration drops `engine.clients.listener` (jig 18), the
listener's address moves to temper's own member, `application.clients`
(`configuration.md`, 3). The one-process world and the process tier
reach the engine through the mapping from the first increment on.

## Read first

- temper's `docs/design/client.md`: 8 (the bullets "temper's server owns
  the clients' listener", "The printed link opened in a browser", "jig's
  example guards, as they are", "jig's clients' component").
- temper's `docs/design/protocol.md`: 3 (the row "the clients'
  listener"), 5 (the rows `jig-protocol-engine-clients` and
  `jig-protocol-engine-clients-http`), 6 (the tree, and the bullet "The
  clients' listener").
- temper's `docs/design/configuration.md`: 2 (the `engine` bullet: "Its
  `clients` member is what jig's clients' component takes; the clients'
  listener is temper's"), 3 (`clients`, and the last paragraph: "the
  clients' listener takes `clients.listener`").
- temper's `docs/design/domain/testing.md`: 2.5 ("the same stories through
  jig's default HTTP mapping"), 2.6 (the bullet "The server's guards":
  its rebound-Host story is `later.md`'s, not this session's).
- jig's `docs/design/protocol/client.md`: 17 (17.1, 17.4).
- This plan's `jig/17-engine-http.md` (the Shapes of 17.1 to 17.5 and
  its interface sketches) and `jig/18-server.md` (18.1's move of the
  listener's member); `decisions.md`, 1c and 6; `findings/temper.md`, 4
  (the clients' listener) and 6.
- skein's `programming-model.md`: 2, 4.1, 4.2, 5.2, 7, 9 (by number
  only); skein's `testing-strategy.md`: 2.7, 2.9, 7 (by number only).

## What the design asks

- **temper's server owns the clients' listener** (`client.md`, 8):
  "`temper-service`'s engine owns it: TCP on the loopback address
  temper's configuration names ..., and the stream's and exchanges'
  connections on it. On it, it mounts jig's clients' component
  (`jig-protocol-engine-clients`) with jig's default HTTP mapping's
  engine end (`jig-protocol-engine-clients-http`) under the prefix
  `/jig`. Every other path is answered not found".
- **The printed link opened in a browser** (`client.md`, 8): "a `GET` of
  `/jig/v1/sign-in/link?secret=…` spends nothing and draws nothing ...
  With no page in standalone, temper's server answers it not found, with
  `Referrer-Policy: no-referrer`".
- **jig's example guards, as they are** (`client.md`, 8): "a Host
  allow-list holding the listener's own address; the required header,
  with no CORS, and a `POST` whose type is not `application/octet-stream`
  answered `415`; the cookie's attributes; a referrer policy at the
  printed link's route."
- **No `Mint` in standalone** (`client.md`, 8): "temper gives it no
  `Mint` in standalone".
- **`application.clients`** (`configuration.md`, 3): `listener`, "a
  socket address; on loopback ... in standalone", which feeds "the
  listener on which jig's clients' component is mounted ...; the
  mapping's Host allow-list; and the address of the sign-in links
  `temper serve` prints".

## Choices this plan makes

- **temper's server copies jig's test service's,** as decision 6 has
  temper follow jig's reference code: `temper_service::server` follows
  `jig_test_service::server` as jig 17.2 to 17.5 leave it (the listener,
  one skein-http server per connection, calls handed to the mount that
  `claims` them), without the bundle, the page and the page's policy
  that jig 18 adds to jig's. A call the mount does not claim is answered
  `404`; at the link's route, with `LINK_PAGE` (jig 17.5's
  `Referrer-Policy: no-referrer`).
- **Two steps, following jig's two:** 05.1 needs jig 17 (the mount, the
  guards, the component without a listener) and keeps reading the
  listener from `engine.clients.listener`, as jig 17.2 to 17.5 keep it;
  05.2 needs jig 18.1, which drops that member from jig's configuration,
  and reads `application.clients` instead.
- **The printed link** is built by temper's server from the component's
  `Link { secret }` (jig 17.1) and `link_path` (jig 17.3), on the
  listener's bound address, as `jig_test_service::clients::printed`
  builds jig's. The binary prints it as it prints it today
  (`crates/temper/src/host.rs`).
- **The guards' `Host` list** is the listener's bound address as the
  printed link names it (`127.0.0.1:PORT`), given to the mount once the
  listener is bound (`Event::Listening`, jig 17.5), as jig's test service
  gives it.
- **No `Mint`:** temper's server never sends `Event::Mint`; the
  `Provider` refusal arm at `crates/temper-service/src/clients.rs:231-232`
  goes with jig's provider route (jig 17.1), replaced by nothing.

## Rules that bind it

- **Each increment starts with a repin commit of its own** onto the
  landing of the newest jig step its Needs name (README, Rules, "Pins":
  05.1 jig 17.5's, 05.2 jig 18.1's), found by `git -C ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log
  origin/main --all-match --grep='^Plan: client-api$' --grep='^Step:
  jig-NN.k$' --format=%H -1`
  with the step's number, never jig's `origin/main`; when `git -C
  ~/src/rust/jig merge-base --is-ancestor <that landing> <temper's
  present jig pin>` succeeds, no repin commit is made. After a repin,
  `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*'
  Cargo.lock | sort -u` prints three lines: `jig.git#<the landing>`,
  smith's line equal to that of `git -C ~/src/rust/jig show <the
  landing>:Cargo.lock`, and skein's a commit that holds it or, while it
  does not hold temper's skein floor, the floor (`git -C
  ~/src/rust/skein merge-base --is-ancestor <that commit> <skein's
  line>` succeeds), which the repin's `cargo update -p skein-lib
  --precise` takes only when temper's skein does not already hold it
  (README, "Pins", "The terminal's room", "The print's room" and
  "temper's skein ahead"); a second revision of a
  source still there after a second `cargo update` is a question.
- **A repin's repairs outside its Shape** follow jig's own diffs of
  `jig-test-service`, counted with `git diff --numstat
  origin/main...HEAD -- <the changed paths outside the increment's
  Files> ':!Cargo.lock'`, added lines, at most 200 (README, "Pins");
  past it, a question.
- **jig's guards as they are:** no setting of the mount's that turns a
  guard off, and no answer of temper's for a path the mount claims; a
  review of the diff checks it (jig 17's Rules, "No switch on a guard").
- **Limits from the shipped profile:** the server's and the mount's
  limits come from `jig_protocol_config::profile(Profile::Small)` as jig
  17.2 and 17.3 move them (`server::Limits`, `Limits.clients_http`),
  never set by hand in a test; a test that needs another value names it
  beside the profile's, with the reason.

## Corrections after the ready check of temper-05.1 (ruling 306)

Checked at skein `a0c3a66`, jig `6605e0564`, temper `90a3f7048`; these
take precedence over the increments below where they differ.

- **05.1, lines:** `Stop::Listener` is `lib.rs:51`; the binary's line
  `host.rs:58`; `fixtures::started()` `tests/fixtures.rs:399`; the
  token's reader `tests/clients.rs:168` (not :155).
- **05.1, the listeners' readiness** is `system.rs:171, 412` and
  `workers.rs:94`, not `engine.rs`; `tests/engine.rs:160` is the workers'
  listener's test, kept. The clients' listener's failure is tested at
  `tests/clients.rs:193` and `tests/system.rs:66-68`: both take the
  server's io tag.
- **05.1, users the Files miss**, in `crates/temper-service/src/`:
  `stop.rs:17, 29, 53-54`; `tests.rs` (`mod server;`);
  `tests/limits.rs:5-16` (the tokens' list); `tests/system.rs:66-68`;
  `tests/clients/peer.rs:42` (a second `Host: localhost`, changed as :13
  is); and `crates/temper-client-shell-terminal/src/client.rs:259-264`
  (the `SignedIn` arms, beside the token's readers).
- **05.1, the routing sketch:** skein's `Call` has no `path()`; its
  `target: Box<[u8]>` holds the query too. The route is the target up to
  its first `?`.
- **05.1, the `Host` of temper's restored clients**
  (`tests/one-process/src/world.rs:585-600`, `Start::Token`): jig 17.5
  makes the native shell send its configured engine address
  (`127.0.0.1:7100` there), the address the list holds; nothing to do
  here unless that world's listener moves off 7100 (05.2).
- **05.2, lines:** `startup/application.rs` is 610 lines; `parse` reads
  `charter` at :383, and `forge`'s `api` is read at :222. The binary
  reads `application` at `crates/temper/src/lib.rs:280` and
  `setup.rs:42-44`, not :268, and neither reads `clients`; `setup.rs:38`
  reads `named.clients`, unchanged while its type is kept.
- **05.2, users the Files miss:** `checks::standalone`
  (`startup/checks.rs:24`) takes no application: it gains the
  `Clients` parameter its loopback check reads;
  `tests/one-process/src/world.rs:265, 592` hard-code `7100`, which
  follows the one-process world's `application.clients`; and
  `crates/temper/tests/support/configuration.rs` gains
  `"clients":{"listener":{"socket":"127.0.0.1:0"}}` (the process tier
  binds port 0). `skein-http` is a workspace dependency already
  (`Cargo.toml:86`).
- **05.2, the reference test:** `listener` is a `skein_io::Addr`, so
  `parse()` does not compile (E0277): bind it with `let
  config::Address::Addr(skein_io::Addr::Ip(listener)) =
  members.unwrap().clients.listener else { panic!("a socket address") };`.
  So corrected, it passed with 05.2's reader applied, under clippy, the
  default and the fuzzy profiles (`temper-service`, `temper`,
  `temper-one-process-world`: 234 and 3 passed).

Refresh any further copy of these lines or names, and list it in the
commit body.

## The startup clock (ruling 325)

- **05.1 repairs the clients' startup clock,** in its Files:
  `clients.rs:112` builds the component with `Env { now: Time::ZERO,
  wall: Wall::EPOCH, .. }`, and jig's `Component::new` mints the startup
  link then, until `env.now + link_span`; on a machine up longer than
  `link_span`, the first sign-in finds it expired and is refused. The
  component gets the service's first `now` and `wall` before it mints
  that link; the shape is 05.1's, within its Files, jig unchanged.
- **Its regression test:** in `tests/clients.rs`, the startup link signs
  in at a named uptime of 600 s (temper 06's reproduction,
  `logs/temper-06/process324-focused.patch` in the run's directory). It
  fails before the repair and passes after it.
- **Size:** the repair and its test, at most 40 lines together, count
  apart from 05.1's 250.

## Increments

### 05.1 The clients' listener in `temper-service`, and `/jig` mounted

Scope: `temper_service::server`, the server child owning the clients'
listener and each connection's skein-http server; jig's component
mounted under `/jig` with `jig-protocol-engine-clients-http` and its
example guards; calls the mount does not claim answered `404`, the link
route's `GET` with `LINK_PAGE`; the printed link built from `Link {
secret }`; the listener's failure stopping the engine as
`Stop::Listener` (temper's `client.md`, 8; `protocol.md`, 6).
Needs: temper 04 (the session's start), and jig 17's last increment,
17.5 (the component on its messages, `Mint` and `Link { secret }`, the mount, its
guards and `claims`).
Files:
- new `crates/temper-service/src/server.rs` and
  `crates/temper-service/src/tests/server.rs`;
- `crates/temper-service/src/clients.rs` (498 lines: `kit::Settings`
  without `listener` at :96; the component's `Request::Io` and
  `Event::Io` and the io queue at :19, :174-198, :218, :262-300 move to
  the server; `kit::Request::Link { link }` at :219 becomes `Link {
  secret }`; the `Provider` arm at :231-232 goes; the worst case's
  `limits.http.body` at :314, a copy of `jig-test-service`'s
  `clients.rs`, follows that file as jig 17.3 lands it, ruling 314), `system.rs` (620
  lines: the clients' io at :289-293 and the listener's failure at
  :508 and :517 come from the server), `engine.rs` (the listeners'
  readiness), `lib.rs` (`pub mod server`; `Stop::Listener` kept at :49),
  `limits.rs` (the server's and the mount's limits from jig's profile),
  `tokens.rs` (the server's io tag beside `CLIENTS`);
- `crates/temper-service/Cargo.toml` (`jig-protocol-engine-clients-http`,
  `skein-http`), workspace `Cargo.toml` and `Cargo.lock`;
- tests: `crates/temper-service/src/tests/clients.rs` (its 12 tests keep
  their names and expected values; `the_component_prints_one_sign_in_link_at_start`
  and `the_engine_prints_one_sign_in_link_at_start` read the URL the
  server builds), `crates/temper-service/src/tests/clients/peer.rs` (the
  peer sends `Host: 127.0.0.1:7100`, the fixture listener's address, in
  place of `localhost` at :13, as jig 17.5 changes jig's peer), and
  `crates/temper-service/src/tests/engine.rs:160`
  (`an_io_listener_failure_stops_the_engine`, kept);
- the binary: `crates/temper/src/host.rs` (the links it prints, :57's
  `Stop::Listener` line kept);
- users of the component's `Settings`, `Request::Io`, `Event::Io`,
  `Request::Link` and `Request::Provider`, found by `git grep -n -E
  'kit::(Settings|Request::Io|Event::Io|Request::Link|Request::Provider|Event::Provider|Verdict)'
  -- crates tests` in temper: at the base, `clients.rs` and
  `tests/clients.rs` only; and the readers of `SignedIn`'s token, which
  jig 17.4 makes optional: `crates/temper-service/src/tests/clients.rs:155`
  and `tests/one-process/src/referee/client.rs:132`.
Shape:
- `temper_service::server::Server`, as `jig_test_service::server::Server`
  (jig 17.2): `fn new(listener: skein_io::Addr, limits: &Limits) -> Self`;
  `fn up(&mut self, event: io::Event, out: &mut Queue<Routed>)`; `fn
  answer(&mut self, call: Token, request: skein_http::server::Request,
  out: &mut Queue<Routed>)`; `fn done(&mut self, call: Token, out: &mut
  Queue<Routed>)`; `fn listening(&self) -> Option<SocketAddr>`; `enum
  Routed { Io(io::Request), Call { call: Token, head:
  skein_http::server::Call }, Http { call: Token, event:
  skein_http::server::Event } }`.
- The engine's routing of a `Routed::Call`: `claims(&mount_settings,
  &head)` true hands it to the mount (`Event::Call`, then `Event::Http`
  until `Done`); false answers `404` with no body, adding `LINK_PAGE`
  when the head is a `GET` of `/jig/v1/sign-in/link` (algorithm in the
  sketch below). The mount's `Request::Component` goes to the component
  as `Event::Transport`; the component's `Request::Transport` goes to the
  mount as `Event::Component`; the mount's `Request::Http` goes to the
  server's `answer`.
- The mount's settings: `prefix: b"/jig"`, `loopback: true`
  (standalone's listener is on loopback, checked at start); its hosts,
  `Event::Listening { hosts }` with the one value `<bound address>` as
  the printed link names it, once the server says it is listening.
- The printed link: `http://<bound address>` followed by
  `link_path(&mount_settings, &secret)`, pushed to the service's
  `links()` queue (unchanged in type) once both the secret and the bound
  address are known, as jig 17.1's test service waits for both.
- `Stop::Listener`: the server's listener failing, or closing before the
  engine drains, stops the engine as the component's did.
Test shape: unit tests in `crates/temper-service/src/tests/server.rs`,
on a fixture `server()` in `crates/temper-service/src/tests/fixtures.rs`
built from `jig_protocol_config::profile(Profile::Small)`'s server
limits, and the existing `fixtures::started()`
(`crates/temper-service/src/tests/fixtures.rs:393`) for the engine-level
tests. The reference test:
```rust
#[test]
fn a_get_of_the_printed_links_route_is_not_found_with_no_referrer() {
    let mut service = fixtures::started();
    let call = fixtures::request(&mut service, b"GET /jig/v1/sign-in/link?secret=AAAA HTTP/1.1\r\nHost: 127.0.0.1:7100\r\n\r\n");
    let (status, headers) = fixtures::answered(&mut service, call);
    assert_eq!(status, 404);
    assert_eq!(fixtures::header(&headers, b"referrer-policy"), Some(b"no-referrer".as_slice()));
    assert_eq!(fixtures::to_component(&mut service).len(), 0, "nothing reached the component");
}
```
Tests, each under 20 ms:
`a_post_under_jig_reaches_the_component_as_its_message`,
`a_path_outside_jig_is_answered_not_found`,
`a_get_of_the_printed_links_route_is_not_found_with_no_referrer`,
`a_host_off_the_list_is_answered_421_with_nothing_done`,
`a_post_not_octet_stream_is_answered_415_with_nothing_done`,
`a_cookie_without_the_browser_header_is_answered_403_with_nothing_done`
(the guards as temper's listener wires them, `client.md`, 8: "the
required header, with no CORS, and a `POST` whose type is not
`application/octet-stream` answered `415`"),
`the_printed_link_names_the_bound_address_and_the_secret`,
`a_listener_failure_stops_the_engine_as_listener` (the base's
`an_io_listener_failure_stops_the_engine` keeps its name if it covers
it; then this one is not added). `fixtures::request` (one call's bytes
on a fresh connection to the started service's listener), `answered`
(the status and headers the server wrote for it), `header` and
`to_component` (the mount's messages to the component since the last
call) are new in the fixtures module, written once, used by the six.
Proof: the process tier, through `scripts/proof-one-process.sh`'s two
commands narrowed to one test, once: `sh -c 'cargo build -p temper
--bin temper && NEXTEST_DOUBLE_SPAWN=0 cargo nextest run -p temper
--profile process -E "test(temper_open_signs_in_prints_the_inbox_and_exits_on_q)"'`:
`temper open` signs in by the link `temper serve` printed, through
`/jig` on temper's own listener, prints `== Inbox == Live` and exits 0
(about 1 minute). And the one-process world, once: `cargo nextest
run -p temper-one-process-world -E
'test(a_person_signs_in_by_the_printed_link_and_a_chat_is_answered)'`:
the person's chat is answered through the mapping (about 1 s). Nothing
else is run for the proof.
Don't: serve a page, a bundle or a policy (the browser is `later.md`'s);
send `Event::Mint`; answer a path the mount claims; set a guard's
setting by hand in a test; change `application`'s members (05.2 does).
Size: at most 250 lines.
Removes: the `Provider` refusal arm (`clients.rs:231-232`), the
component's io routing in `clients.rs` and `system.rs` (moved to the
server), `kit::Request::Link { link }`'s use.

### 05.2 `application.clients` in place of `engine.clients.listener`

Scope: temper's member `clients`, `{"listener": address}`, read and
checked at start (a socket address, on loopback in standalone), and the
server's listener taken from it (`configuration.md`, 3), once jig's
configuration no longer has `engine.clients.listener` (jig 18.1).
Needs: jig 18.1 (`jig_protocol_config`'s
`clients` with no `listener`).
Files:
- `crates/temper-service/src/startup/application.rs` (538 lines:
  `Members` gains `clients`, `parse` reads it beside `charter` at
  :340), `startup/checks.rs` (the loopback check at :39-42 moves from
  `engine.clients.listener` to `application.clients.listener`),
  `startup.rs` (the server built from `members.clients`), `settings.rs`;
- the configurations that write `engine.clients.listener` or the
  `application` member: `crates/temper-service/src/tests/fixtures.rs:47-51`
  (`ENGINE`'s `clients.listener`) and
  `crates/temper-service/src/tests/fixtures/startup.rs:10` (`APPLICATION`),
  `crates/temper-service/src/tests/checks.rs:35-37, 204-218`
  (`Shape::ClientProcess`, `Shape::ClientRemote`: their path becomes
  `application.clients.listener`, their `What` kept),
  `crates/temper/tests/support/configuration.rs:27, 38`,
  `tests/one-process/src/fixtures.rs:98` (its `application` gains
  `clients`; its engine part comes from jig's `engine_document()`, which
  jig 18.1 changes), and the binary's reading of `application`
  (`crates/temper/src/lib.rs:268`); found by `git grep -n -e
  'clients\.listener' -e '"clients"' -e '\["clients"\]' -e '"forges"' --
  '*.rs'` in temper.
Shape:
- `temper_service::startup::application::Clients { listener:
  jig_protocol_config::Address }` (the sketch below), `Members.clients:
  Clients`.
- `application::clients(document, at, path, problems) -> Option<Clients>`,
  reading the address with `config::address::read` as `forge`'s `api` is
  read at `application.rs:182`; an in-process name refused as a form
  problem; a missing member named `application.clients: missing`.
- The loopback check: `application.clients.listener` off `127.0.0.0/8`
  and `::1` refused with `What::Loopback`, as the base refuses
  `engine.clients.listener` today.
Test shape: `crates/temper-service/src/tests/startup.rs` (on
`fixtures::APPLICATION` and the local `application()` helper at :6) and
`tests/checks.rs` (on its `Shape` table). The reference test:
```rust
#[test]
fn the_clients_listener_is_read_from_the_application_member() {
    let jig = fixtures::engine().jig;
    let (members, problems) = application(fixtures::APPLICATION, &jig);
    assert_eq!(&*lines(&problems), b"", "example accepted");
    let config::Address::Addr(listener) = members.unwrap().clients.listener else { panic!("a socket address") };
    assert_eq!(listener, "127.0.0.1:7100".parse().unwrap());
}
```
Tests, each under 20 ms:
`the_clients_listener_is_read_from_the_application_member`,
`a_missing_clients_member_is_named_by_its_path`,
`an_in_process_clients_listener_is_refused_as_a_form`,
`a_clients_listener_off_loopback_is_refused_naming_its_path` (the
`checks.rs` table's two rows, their path changed, their `What` kept).
Proof: the process tier, once, as 05.1's command with the filter
`test(temper_serve_prints_its_link_and_drains_on_a_signal)`: `temper
serve`, configured with `application.clients`, prints its link on the
listener that member names and drains on a signal (about 1 minute). And
the one-process world, once, as 05.1's: the person signs in through the
listener the one-process world's `application.clients` names. Nothing
else is run.
Don't: keep a fallback to `engine.clients.listener`; add a member to
`application` beyond `clients.listener`.
Size: at most 80 lines.
Removes: the check of `engine.clients.listener` at `startup/checks.rs:39-42`.

## Interface sketches

`temper_service::startup::application::Clients` and its reader compile
at the base (`logs/w9/clippy-sketches-3.txt`, as
`temper_service::sketch_clients`, its body `unreachable!`). The server's
types name jig 17's and are checked at the ready check.

```rust
// crates/temper-service/src/startup/application.rs (05.2)
/// temper's clients' listener, which `temper-service` owns (client.md, section 8).
#[derive(Debug)]
pub struct Clients {
    /// A socket address on loopback in standalone: the listener, the mapping's Host
    /// allow-list, and the address of the sign-in links `temper serve` prints.
    pub listener: config::Address,
}
/// Read `application.clients`, naming each problem by its path, as the other members are read.
#[must_use]
pub fn clients(document: &Document, at: u32, path: &Path, problems: &mut Problems) -> Option<Clients>;
```

```rust
// crates/temper-service/src/server.rs (05.1), after jig_test_service::server (jig 17.2)
/// The clients' listener and its connections, each with its skein-http server.
pub struct Server { /* listener, connections by token, calls by token, limits */ }
/// What the server hands the engine's routing.
pub enum Routed {
    /// An io request of the server's own entities, tagged as the server's.
    Io(io::Request),
    /// A call's head, before its body: claimed by the mount, or answered here.
    Call { call: Token, head: skein_http::server::Call },
    /// What the call's connection said after its head, until `done`.
    Http { call: Token, event: skein_http::server::Event },
}
impl Server {
    pub fn new(listener: skein_io::Addr, limits: &Limits) -> Self;
    pub fn up(&mut self, event: io::Event, out: &mut Queue<Routed>);
    pub fn answer(&mut self, call: Token, request: skein_http::server::Request, out: &mut Queue<Routed>);
    pub fn done(&mut self, call: Token, out: &mut Queue<Routed>);
    pub fn listening(&self) -> Option<SocketAddr>;
}
```

The engine's routing of a call, which is easy to get wrong in its order:

```rust
// crates/temper-service/src/engine.rs (05.1), on Routed::Call { call, head }
if jig_protocol_engine_clients_http::claims(&self.mount_settings, &head) {
    self.mount.down(&env, mount::Event::Call { call, head }, &mut mount_out);  // the guards run there
} else {
    let link = head.method == Method::Get && head.path() == b"/jig/v1/sign-in/link";
    let headers = if link { [LINK_PAGE].as_slice() } else { [].as_slice() };
    self.server.answer(call, not_found(headers), &mut routed);                // spends nothing
}
```

## Questions to raise when reached

- If jig 18 moves more than the listener's member out of
  `jig_protocol_config`'s `clients` (its 18.1 Shape), the members it
  moves are temper's to place: raise them, with the design's line for
  each (`configuration.md`, 3, names only `listener`).

## Done when

- 05.1 and 05.2 have landed on temper's `origin/main`.
- `git -C <worktree> grep -n -e 'clients\.listener' -- crates/temper-service/src`
  prints only `application`'s reader and check, no `engine.clients`.
- `git -C <worktree> grep -n -w -e 'Request::Provider' -e 'Event::Provider' -- crates tests`
  prints nothing.
- The process tier's two filters of 05.1 and 05.2 each passed once, as
  their landings' reports say.
