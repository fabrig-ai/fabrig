# Session 17: `Mint`, the link's secret, `jig-protocol-engine-clients-http` and its guards

The clients' component stops speaking HTTP. It takes `Mint` from the
application's server code in place of the provider route, gives the
printed link's secret as `Link { secret }` and lets the application print
the URL, and owns no listener and no connection: jig's test service's
server owns the listener and its connections, and a new crate,
`jig-protocol-engine-clients-http`, the default mapping's engine end,
turns each call under its prefix into one of the component's messages and
writes the component's answers back as HTTP, with the example guards
(`protocol/client.md`, 4.2, 8 and 17; `protocol/browser.md`, 11 and 12.2).
The session ends with each guard of `browser.md`, 12.2 in place and the
worlds' clients sending a `Host` the allow-list takes.

## Read first

- jig's `docs/design/protocol/client.md`: 4 (4.1, 4.2), 8, 17 (17.1 to
  17.4).
- jig's `docs/design/protocol/browser.md`: 11, 12.2, 15 (the bullet "the
  mapping's engine end"), 19 (the table's first four rows).
- jig's `docs/design/protocol/service.md`: 2 (the bullet "The clients'
  traffic, through the application's server"), 3 (step 4), 7 (the bullets
  "Its server" and "The reference binary").
- jig's `docs/design/protocol/configuration.md`: 3 (the rows `providers`
  and `clients`).
- `findings/server-and-browser.md`: 1, 2, 6.
- skein's `programming-model.md`: 2, 4.2, 5.2, 9, 10 (by number only);
  skein's `testing-strategy.md`: 2.7, 8 (by number only).

## What the design asks

- **The component's messages** (`client.md`, 4.2): in from the transport
  `Exchange { exchange, credential, message }`, `StreamOpen { stream,
  credential }`, `StreamTaken { stream }`, `StreamGone { stream }`; in
  from the application's server code `Mint { exchange, identity }`; out
  `Reply`, `Streamed`, `StreamEnded`, `Minted | MintRefused`, and `Link {
  secret }` to the application's startup. "The component knows no HTTP"
  (`service.md`, 1).
- **`Mint`** (`client.md`, 8): "a provider's number, as its configuration
  numbers it ..., the subject there, and a login and a name for display";
  refused for "a provider the configuration does not give as one of the
  application's (of kind application) ...; a login or a name not UTF-8 or
  past its bound; not ready, while the engine stops; busy, at the people
  child's limits". "No provider route" (`client.md`, 17.1).
- **The printed link** (`client.md`, 8): "the component draws a one-time
  secret ... and gives it to the application's startup, `Link { secret
  }`, which prints a URL carrying it. The URL is the application's".
- **The engine end** (`client.md`, 17.4): "each call under its prefix
  routed to one of the component's messages, but a `GET` of the link's
  route, left to the application's page with the referrer policy (17.1);
  its credential read from the cookie or the bearer; each `Reply` written
  as `200` with its record; the stream written as events, `StreamTaken`
  said as their bytes are written."
- **The credential** (`client.md`, 17.3): "As a cookie, for a browser:
  set by the engine end on a sign-in's answer, which then carries no
  credential in its body, or by the application's server after a
  `Minted`, through the engine end's function that sets the same cookie
  ... cleared by signing out." "An exchange carrying [`Jig-Client:
  browser`] is a browser's, its sign-in answered with the cookie; any
  other, with the credential in its answer's body."
- **The guards** (`browser.md`, 12.2): a `Host` off the list `421`; "a
  request the cookie authenticates without `Jig-Client: browser` answered
  `403`"; a POST of another content type `415`; "no `OPTIONS` answered
  and no `Access-Control-` header sent"; the cookie's attributes; and the
  referrer policy on the link's route. Each "with nothing done".

## Choices this plan makes

- **The server is the test service's, from 17.2.** The listener and its
  connections move once, into `jig_test_service::server`, which hands
  each call to what it mounts; jig 18 then reads its listener from
  `jig-test`'s own configuration and serves the bundle beside the mount.
  This keeps the connection code from moving twice (into the new crate,
  then out again). Until jig 18.1 the server's listener is still read
  from jig's `engine.clients.listener`.
- **The engine end works per call.** The server owns each connection's
  skein-http server and hands the engine end the calls it claims
  (`claims`), with the events that follow on that connection until
  `Done`; the engine end answers with skein-http requests for that call.
  A stream's call stays with the engine end until it ends.
- **The component's replies keep HTTP's three outcomes as data:** a
  record (`200`), not a record of its route (`400`), a conflict (`409`,
  a second stream for one sign-in, or a watch closed twice), the statuses
  the base answers today; `413` stays the engine end's own (a body past
  its bound).
- **The sign-in's credential and the cookie** (the audit's open point:
  "`SignedIn.token` is always in the answer's body today; a credential
  held by a cookie needs it absent"): settled by the design's words
  above. The component's reply keeps the credential in its record, as
  `client.md`, 4.2 lists it ("a sign-in's credential"); the engine end,
  for an exchange carrying the browser header on a sign-in route, decodes
  the answer, sets the cookie from it, and writes the record again with
  no credential. The wire's `SignedIn.token` becomes optional, edited in
  place in the client wire's current version (README, "Format
  versions"), and a client that gets none holds none (`client/domain/
  client.md`, 12: "holds the credential the answer gives, unless the
  transport keeps it").
- **`Minted` carries the sign-in's expiry** beside the credential, so the
  server can set the cookie's `Max-Age` ("its life the sign-in's",
  `client.md`, 17.3), which `Minted { exchange, credential }` alone does
  not give.
- **The component draws the link's secret at its start**, and on each
  refusal, as today; the startup prints the URL once its listener is
  bound.
- **The link's loopback check moves to the startup** (`service.md`, 3,
  step 4: "the startup first checks that the clients' listener ... is on
  a loopback address, and refuses the start, naming both").

## Rules that bind it

- **No HTTP in `jig-protocol-engine-clients` once 17.3 lands:** `git -C
  <worktree> grep -n -w -e Listen -e skein_http -- crates/jig-protocol-engine-clients/src`
  prints nothing at 17.3's end (README, "Done when", 6; code-organization.md, 6).
- **jig never names the repository downstream of it:** the check of
  code-organization.md, 6, quoted: "`git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end". The repository downstream of jig follows
  17.1 to 17.5 at its repin (the plan's README, "Repin").
- **No switch on a guard** (`browser.md`, 12.2: "the mapping takes the
  allow-list and nothing that turns a guard off"): `Settings` holds
  `prefix` and `loopback` only, and 17.5's guards' fixture builds it with
  both fields named, so a field added fails that fixture's build.

## Corrections at jig `5b4e287f0` (ruling 302, the ready check of jig 17, 2026-10-18)

These take precedence over the increments' Files, Shape, Test shape and
sketches where they differ. Refresh any further copy or derived value,
and list it in the commit body.

- **17.1, the protocol world's engine:** the link is kept at
  `tests/client-protocol/src/engine.rs:78` (`Request::Link { link } =>
  self.link = link`) and the provider arm panics at :107; :69 and :98
  are the wall and `to_root`. Read "`engine.rs:69` and `:98`" as :78 and
  :107.
- **17.1, users its Files miss** (without them 17.1 does not build):
  `crates/jig-protocol-engine-clients/src/watches.rs:107` (`|
  Route::Provider(_)`); `tests/engine-clients/src/peer/closing.rs:3,
  :44` (`ProviderCall`, `take_provider`); `tests/engine-clients/src/peer.rs:178`
  (`providers: VecDeque<…>`) and :285-286.
- **17.1, Test shape:** the unit file `src/tests/closing.rs` holds only
  `stopping_keeps_the_printed_link_live_and_unspent` (:4; its
  `Request::Link { .. }` at :9 becomes `secret`). The five provider
  tests are the world's, `tests/engine-clients/tests/closing.rs:107,
  :130, :139, :307, :317`: they move or go there.
- **17.1, the sketch's fixtures are new:** `Peer::mint`, `Peer::minted`
  and `fixtures::application_person` exist nowhere at the base; 17.1
  adds them, in `tests/engine-clients/src/{peer.rs, fixtures.rs}`.
- **17.2, the world's front:** `tests/client-protocol/src/front.rs` has
  331 lines. It signs in with the printed link at :157-161 and :318-322
  (`side.connections.printed(&self.engine.link)`), and `Event::Io` is at
  :150, :192 and :199: read these for "`front.rs:441`" and
  "`front.rs:312-379`".
- **17.2, the component's `Limits`** are `small.rs`'s `CLIENTS`
  (`crates/jig-protocol-config/src/profile/small.rs:257-266`) and
  `standard.rs:147-148` (`{ connections: 16, streams: 8, exchanges: 32,
  ..small::CLIENTS }`); `small.rs:250` (`clients: CLIENTS,`) and
  `standard.rs:137` (the driver's `exchanges: 8`) are not them. A new
  field such as `mints` is named in small's `CLIENTS`.
- **17.4, who builds `SignedIn`:** `component.rs:484-486` builds it
  (`w::SignedInParts { token, … }`), and so does the unit fixture
  `src/tests/fixtures/services.rs:77-79`; `services.rs:25` only reads it.
  Its users the Files miss: `crates/jig-client-protocol-link/src/sign_in.rs:97`,
  `crates/jig-client/src/tests/fixtures.rs:239-242`,
  `crates/jig-client-protocol-link/src/tests/fixtures.rs:269-272`,
  `tests/client-protocol/src/native/wire.rs:176`; the schema's field is
  `client-v8.schema:13` (`token: secret 48`). `jig-client-shell-native`
  reads no token: drop it from 17.4's Files.
- **17.5, the native shell:** the crate is `crates/jig-client-shell-native`
  (there is no `jig-shell-native`): `connections.rs:65` (`host:
  "localhost".to_owned()`) and `tests.rs:77` (`host: localhost\r\n`).
  `connections.rs:81` (`if self.host != "localhost" && self.host !=
  host`) reads `"localhost"` as "no link yet": change that marker with
  the default (an `Option`, or the configured address), not the default
  alone.
- **17.5, the host guard:** `Settings` holds `prefix` and `loopback` only
  (the sketches' `pub struct Settings`), and the sketch calls `guards::host(&self.hosts, &head)` (the sketch of the mount):
  the guard is `fn host(hosts: &List<Box<[u8]>>, call: &Call) -> bool`.
- **The sketches under clippy `-D warnings`:** `#[derive(Debug)]` on
  each public type; `#[must_use]` on `claims`, `link_path`, `cookie`,
  `cleared`, `link_page` and `Mount::{new, up, next_deadline}`;
  `PartialEq` on `FromTransport` and `Message`; a `type Fields =
  &'static [(&'static [u8], &'static [u8])];` for 17.5's `REFUSED`; and
  the reference tests (17.3's and 17.5's, which call `fixtures::component(&mut out)`) compare `*fixtures::component(..)`
  with the array, or return a slice. With these the sketches pass
  clippy. jig pins skein at `ca5b75ef7`, where `skein_lib::sha256` is
  not yet: a sketch that needs it takes the repin first.

## Corrections at jig `6605e0564` (ruling 307, the re-check after jig 16.5)

These take precedence over the section above and the increments where
they differ.

- **Lines moved by 16.5:** `small.rs`'s `CLIENTS` is :256-270 (its
  `exchanges: 8` :264) and `clients: CLIENTS` :249; `standard.rs`'s
  `CLIENTS` :146-147, its driver's line :136 (`clients: CLIENTS` :140);
  the world's provider tests `tests/engine-clients/tests/closing.rs:108,
  :131, :140, :308, :318`; `crates/jig-client-protocol-link/src/tests/fixtures.rs:275-278`;
  `tests/client-protocol/src/native/wire.rs:177`.
- **17.4, who builds `SignedIn`:** `component.rs:484-486`,
  `crates/jig-client-protocol-link/src/tests/fixtures.rs:275-278` and
  `crates/jig-client/src/tests/fixtures.rs:239-242`;
  `src/tests/fixtures/services.rs:77` (`signed_in_answer`) decodes one,
  and builds none.
- **17.4, the limit's name:** `token: option secret 48` makes the
  generated limit `signed_in_token_some`, in place of `signed_in_token`.
  Its users: `crates/jig-protocol-engine-clients/src/limits.rs:25` and
  `:52` (the worst case gains the option's one-byte tag), and
  `crates/jig-client-protocol-wire/src/tests.rs:43`, in 17.4's Files.
- **17.4, the readers of `.token()`** take the option
  (`.as_ref().expect(..)` in tests):
  `crates/jig-protocol-engine-clients/src/tests/services.rs:70, 71`;
  `tests/engine-clients/src/{peer.rs:349, fixtures.rs:92,
  peer/closing.rs:103}`; `tests/engine-clients/tests/{sign_in.rs:45,
  requests.rs:17, closing.rs:128}`; `tests/client-protocol/src/native/wire.rs:177`
  (`credential` takes the option). The link's guard at
  `crates/jig-client-protocol-link/src/sign_in.rs:97, 100` admits a
  token of 40 bytes or none (a `match`: clippy bans `is_some_and`); a
  `SignedIn` with none signs the link in as `With::Transport` does,
  holding none, whatever it held before (`end_sign_in` clears it), and
  `a_sign_in_answer_with_no_token_leaves_none_held` starts with a
  credential held (ruling 318).
- **17.4, the regenerated files:** `generated/v8.rs` and the goldens
  `golden/v8/{record_signed_in_*, enum_sign_in_answer_signed_in_*}.bin`
  (`SKEIN_CODEGEN_WRITE=1`, then the drift test). So edited, the
  workspace's `cargo check --all-targets` passed and the drift test 3 of
  3; no other suite was run.
- **17.1 and 17.2, a user the Files miss:** `tests/ops/src/joint/bytes.rs`
  (jig-ops-world) uses the component's `Limits`, `Io`, `Link` and
  `Provider`.

Refresh any further copy of these lines or names, and list it in the
commit body.

## Increments

### 17.1 `Mint` in place of the provider route, and the link's secret

Scope: `Mint { exchange, identity }` in, `Minted { exchange, credential,
expires }` or `MintRefused { exchange, why }` out (`client.md`, 4.2, 8);
the provider route, its verdict and its call removed (`client.md`,
17.1); `Link { secret }` out in place of the printed URL, the test
service building the URL from the secret and its listener's bound
address (`client.md`, 8; `service.md`, 7).
Needs: nothing beyond jig 16 on main.
Files: `crates/jig-protocol-engine-clients/src/{boundary.rs, component.rs,
closing.rs, sign_in.rs, routes.rs}`; `testing/jig-test-service/src/clients.rs`
(the `Provider` arm of `to_root`, the `links` queue now filled from
`Link { secret }`), `testing/jig-test-service/src/lib.rs` (`links()`
unchanged in type); `tests/engine-clients/src/peer.rs` (`printed_link`,
from the secret), `tests/client-protocol/src/engine.rs:69` and `:98` (the protocol world's
engine keeps the link and panics on a provider request; it keeps the
secret now, and its provider arm goes); `tests/engine-clients/tests/closing.rs`
(the five provider tests, moved to `Mint`: `a_provider_identity_signs_in_as_the_link_does`,
`a_provider_refusal_signs_nobody_in`, `an_unconfigured_provider_is_not_routed`,
`after_stop_an_application_provider_is_answered_not_ready`,
`after_stop_a_pending_provider_identity_commits_nothing`); the unit
tests' fixtures `crates/jig-protocol-engine-clients/src/tests/fixtures.rs`
and `tests/closing.rs`; `crates/jig-protocol-engine-clients/src/limits.rs`
and `crates/jig-protocol-config/src/profile/{small.rs, standard.rs}`
(`Limits.mints`), and the other literals of the component's `Limits`,
`tests/engine-clients/src/fixtures.rs:34` and
`tests/client-protocol/src/fixtures.rs:69` (the repository downstream of
jig builds none; it reads jig's profile). Users found by `git grep -n -E
'Request::Provider|Event::Provider|Verdict|ProviderCall|Route::Provider|Request::Link'`
in jig (`logs/w8/users-17.1-component.txt`: 29 lines in 11 files name the
component's types; the other hits of `logs/w8/users-17.1.txt` are the
client's own `Request::Link` in `jig-domain-client`, its test domain and
`ops`, and the worker's `Request::Link`, which this does not touch); the
repository downstream of jig has its own copy of the `Provider` refusal
arm and of the `links` queue (3 lines in one file), which it follows at
its repin.
Shape:
- `jig_protocol_engine_clients::Event::Mint { exchange: Token, identity:
  jig_domain_people::Identity }` replaces `Event::Provider { exchange,
  verdict }`; `Request::Minted { exchange: Token, credential: Secret,
  expires: Wall }` and `Request::MintRefused { exchange: Token, why:
  MintRefusal }` replace `Request::Provider { exchange, provider, call }`;
  `enum MintRefusal { NotApplication, Bound, NotReady, Busy }`.
  `Verdict`, `ProviderCall`, `Route::Provider` and `routes::provider` go.
- `Settings.providers: List<u16>` keeps its name and type and now lists
  the providers of kind application, the only ones `Mint` takes (its doc
  line says so).
- A mint runs the link's path once checked: `Waiting::Core` for the
  core's `SignIn`, then `Waiting::Keep`, then `Minted` in place of a
  reply (`Waiting::Keep`'s `answer` gains `sign_in::Answer::Minted {
  expires }`). The core's `Again` for it (the people child at its
  sign-ins' limit) is `MintRefused { why: Busy }`.
- Mints in flight are bounded (`client.md`, 4.2: "Bounded: ... mints in
  flight"): `jig_protocol_engine_clients::Limits` gains `mints: u32`,
  4 in the small profile and 16 in the standard one
  (`crates/jig-protocol-config/src/profile/{small.rs, standard.rs}`,
  `CLIENTS`); a `Mint` past it is `MintRefused { why: Busy }`, and
  `worst_case` counts it.
- `Request::Link { link: Box<[u8]> }` becomes `Request::Link { secret:
  Secret }`; `sign_in::printed` and `address_text` leave the component.
- `jig_test_service::clients`: `fn printed(listener: SocketAddr, secret:
  &Secret) -> Box<[u8]>` builds `http://<listener>/jig/v1/sign-in/link?secret=<base64url>`
  (the component's `printed` and `address_text`, moved); `Clients::up`
  records the address of `io::Event::Listening`; a secret waits in
  `Clients` until that address is known.
Test shape: unit tests in `crates/jig-protocol-engine-clients/src/tests/closing.rs`
(the provider tests at the base live there and in the world), on the
fixtures `Exchange::with_env` and `env()` of `src/tests/fixtures.rs`
(limits from `env()`, the unit fixture's, which the base sets beside the
small profile's: `identity_bytes: 64`, `exchanges: 4`, named there);
world tests in `tests/engine-clients/tests/sign_in.rs` on `Peer::started`
and `fixtures::settings()` (`tests/engine-clients/src/fixtures.rs`, whose
`settings().providers` gains provider `5`, of kind application). The
reference test, new in `tests/sign_in.rs`:
```rust
#[test]
fn a_mint_for_an_application_provider_signs_in_as_the_link_does() {
    let mut peer = Peer::started(&fixtures::settings());
    let exchange = peer.mint(fixtures::application_person());
    peer.answer_signed_in(fixtures::FIRST_SIGN_IN, fixtures::FIRST_PERSON);
    peer.answer_kept(fixtures::FIRST_SIGN_IN);
    let (credential, expires) = peer.minted(exchange).expect("minted");
    assert_eq!(u64::from_be_bytes(credential.expose()[..8].try_into().expect("number")), fixtures::FIRST_SIGN_IN);
    assert_eq!(expires, fixtures::expires());
}
```
The five provider tests at the base become `Mint`'s, each a test of the
world unless it says otherwise, each under 0.5 s (world) or 20 ms
(unit):

| Old name, at the base (`tests/engine-clients/tests/closing.rs`) | New name | File |
|---|---|---|
| `a_provider_identity_signs_in_as_the_link_does` | `a_mint_for_an_application_provider_signs_in_as_the_link_does` | `tests/engine-clients/tests/sign_in.rs` |
| `a_provider_refusal_signs_nobody_in` | `a_mint_with_a_login_that_is_not_utf8_is_refused_as_bound` | `tests/engine-clients/tests/sign_in.rs` |
| `an_unconfigured_provider_is_not_routed` | `a_mint_for_the_link_s_provider_is_refused_as_not_application` | `tests/engine-clients/tests/sign_in.rs` |
| `after_stop_an_application_provider_is_answered_not_ready` | `after_stop_a_mint_is_refused_not_ready` | `tests/engine-clients/tests/closing.rs` |
| `after_stop_a_pending_provider_identity_commits_nothing` | `after_stop_a_pending_mint_commits_nothing` | `tests/engine-clients/tests/closing.rs` |

New: `a_mint_with_a_name_past_its_bound_is_refused_as_bound` and
`a_mint_the_people_child_answers_busy_is_refused_busy` (world,
`tests/sign_in.rs`); `a_mint_past_the_mints_in_flight_is_refused_busy`
(unit, `crates/jig-protocol-engine-clients/src/tests/closing.rs`);
`the_printed_url_carries_the_secret_the_component_gave` (unit,
`testing/jig-test-service/src/tests/clients.rs`).
Proof: the protocol world: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_request_whose_answer_was_lost_is_made_once)'`, once: its
person signs in by the URL the test service printed from `Link { secret
}` (the world's front signs in with the engine's printed link,
`tests/client-protocol/src/front.rs:441` at the base) and its request is
made once (0.55 s at the base). It shows the URL the service
builds is the one the component's secret signs in with; `Mint` has no
caller above its unit and world tests until an application's sign-in
(none in jig; `later.md`, "the forge ... through `Mint`" for the
repository downstream of jig).
Don't: keep a route, a verdict or a call for providers; make a `Mint`
from anything a client sends (`client.md`, 8: "a transport never makes
one from a client's bytes"); change what a link sign-in answers.
Size: at most 150 lines.
Removes: `Event::Provider`, `Request::Provider`, `Verdict`,
`ProviderCall`, `Route::Provider`, the component's `printed`.

### 17.2 The test service's server owns the listener and the connections

Scope: the clients' listener and each connection's skein-http server
leave the component for `jig_test_service::server`, which hands each
call to the component with the events that follow it (`service.md`, 1
and 2: "The clients' listener is the application's"); the link's
loopback check moves to the startup (`service.md`, 3, step 4).
Needs: 17.1.
Files: new `testing/jig-test-service/src/server.rs`, and its tests in a
new `testing/jig-test-service/src/tests/server.rs` on the crate's one
fixtures module, `src/tests/fixtures.rs` (code-organization.md, 5);
`testing/jig-test-service/src/{clients.rs, lib.rs,
engine.rs (listeners_open), stop.rs, startup.rs, limits.rs}`;
`crates/jig-protocol-engine-clients/src/{component.rs, connection.rs,
closing.rs, limits.rs, boundary.rs}`; `crates/jig-protocol-config/src/profile/{small.rs,
standard.rs}` (`connections` and `stall` leave `CLIENTS` for the
server's limits); `tests/engine-clients/src/{peer.rs, peer/closing.rs}` and
`tests/client-protocol/src/{engine.rs, front.rs:312-379, fixtures.rs:48}`
(both worlds put `jig_test_service::server::Server` between their
simulated sockets and the component, in place of `Event::Io`; the
engine-clients world takes `jig-test-service` as a dependency),
`tests/engine-clients/tests/sign_in.rs`
(`a_link_on_a_non_loopback_listener_refuses_the_start` moves to the
startup's tests, `testing/jig-test-service/src/tests/startup.rs`).
`tests/engine-clients/tests/closing.rs` (`a_close_waits_for_its_listener_to_open`
and `abort_upgrades_sockets_waiting_for_their_graceful_close` test the
listener and the sockets: they move to the server's tests, their names
and assertions kept). The users: `git grep -n -E 'listener|Event::Io|Request::Io|Unusable::Listener'
-- crates/jig-protocol-engine-clients testing/jig-test-service tests/engine-clients tests/client-protocol`,
137 lines at the base (`logs/w8/users-17.2.txt`), of which the workers'
listener's (`testing/jig-test-service/src/workers.rs`, `tests/worker.rs`,
21 lines) is not touched. The component's `Limits` are built in
`crates/jig-protocol-config/src/profile/{small.rs:250, standard.rs:137}`,
`tests/engine-clients/src/fixtures.rs:34`, `tests/client-protocol/src/fixtures.rs:69`
and read in `testing/jig-test-service/src/{clients.rs, limits.rs}`
(`logs/w8/users-17.3-limits.txt`, whose `tools.rs` and `workers.rs` hits
are other components' `kit::Limits`); the repository downstream of jig
reads `limits.clients.connections` in its startup check and sets it in a
fixture, which it follows at its repin.
Shared: `crates/jig-protocol-config/src/` (README, "Shared files").
Shape:
- `jig_protocol_engine_clients::Settings` loses `listener`;
  `Limits` loses `connections` and `stall` (the server's now);
  `Unusable::Listener` goes.
- The component takes calls, not sockets: `Event::Call { call: Token,
  head: skein_http::server::Call }` and `Event::Http { call: Token,
  event: skein_http::server::Event }` replace `Event::Io`;
  `Request::Http { call: Token, request: skein_http::server::Request }`
  and `Request::Done { call: Token }` (the call is over; its connection
  is the server's again) replace `Request::Io`. The component keeps each
  call's body intake and response (`connection.rs`'s per-call half) until
  17.3.
- `jig_test_service::server::Server`: `fn new(listener: skein_io::Addr,
  limits: &Limits) -> Self`; `fn up(&mut self, event: io::Event, out:
  &mut Queue<Routed>)`; `fn answer(&mut self, call: Token, request:
  skein_http::server::Request, out: &mut Queue<Routed>)`; `fn done(&mut
  self, call: Token, out: ...)`; `fn listening(&self) -> Option<SocketAddr>`;
  `enum Routed { Io(io::Request), Call { call, head }, Http { call,
  event } }`. Each call goes to the component; nothing else is served
  until jig 18. `server::Limits { connections: u32, http:
  skein_http::server::Limits, stall: Duration }`, its numbers from the
  small and standard profiles' `CLIENTS` at the base (moved, unchanged).
- The startup refuses a link enabled on a listener off loopback,
  `Problems` naming `engine.clients.link` and `engine.clients.listener`.
Test shape: `testing/jig-test-service/src/tests/server.rs`, new, on a
fixture `fixtures::server()`, added to `src/tests/fixtures.rs`, built from
`limits::for_profile(Profile::Small, ..)`'s server limits. The reference
test:
```rust
#[test]
fn an_accepted_connection_s_call_goes_to_the_mount() {
    let mut server = fixtures::server();
    let mut out = Queue::with_capacity(16);
    server.up(fixtures::listening(), &mut out);
    server.up(fixtures::accepted(SOCKET), &mut out);
    server.up(fixtures::bytes(SOCKET, b"POST /jig/v1/read HTTP/1.1\r\nHost: 127.0.0.1:7100\r\nContent-Length: 0\r\n\r\n"), &mut out);
    assert_eq!(*fixtures::calls(&mut out), [(Box::from(b"/jig/v1/read".as_slice()), skein_http::Method::Post)]);
}
```
Tests: `an_accepted_connection_s_call_goes_to_the_mount`,
`a_connection_past_the_limit_is_rejected`,
`a_kept_connection_carries_a_second_call_after_done`,
`a_stalled_connection_is_closed_at_its_deadline`,
`closing_the_server_closes_its_listener_then_its_connections`,
`a_link_on_a_non_loopback_listener_refuses_the_start` (moved, its name
and assertion kept, now on the startup). Unit tests under 20 ms.
Proof: the protocol world, which runs the real service over simulated io:
`cargo nextest run -p jig-client-protocol-world -E
'test(a_request_whose_answer_was_lost_is_made_once)'`, once: a request
whose answer is lost on the wire is made once, through the server's
connections (0.55 s at the base). And the one-process world: `cargo nextest
run -p jig-one-process-world -E 'test(a_person_s_chat_is_answered_by_the_in_process_worker)'`,
once (0.32 s at the base; or the name jig 16 gave it, if it renamed a
chat's test). Nothing else is run for the proof.
Don't: serve anything but the component's calls; change a status the
component answers; change the limits' numbers.
Size: at most 250 lines; stop 560 (ruling 312: the listener and the sockets leave `component.rs`, 255 lines removed).
Removes: the component's listener, its sockets, `Event::Io`,
`Request::Io`, `Unusable::Listener`.

### 17.3 `jig-protocol-engine-clients-http`: the component on its messages

Scope: the default mapping's engine end (`client.md`, 17.1, 17.2, 17.4):
each call under its prefix dispatched to one of the component's
messages, bodies read up to the route's bound, the bearer read as the
credential, each reply written as `200` with its record and `Cache-Control:
no-store`, the stream written as server-sent events with `StreamTaken`
said once an event's bytes are written; the component on its messages
only (`client.md`, 4.2); the prefix a parameter, `/jig` in the test
service.
Needs: 17.2.
Files: new crate `crates/jig-protocol-engine-clients-http` (`Cargo.toml`,
`src/{lib.rs, dispatch.rs, call.rs, stream.rs, credential.rs}`,
`src/tests.rs`, `src/tests/fixtures.rs`); moved as one file each from the component
where they stay one file: `connection.rs`'s per-call half to `call.rs`,
`stream.rs`'s writer half to `stream.rs`, `routes::route` to
`dispatch.rs`, `sign_in::bearer` to `credential.rs`;
`crates/jig-protocol-engine-clients/src/{boundary.rs, component.rs,
watches.rs, reads.rs, services.rs, closing.rs, stream.rs, routes.rs,
limits.rs, lib.rs}` and its unit tests' `src/tests/fixtures.rs` (the
`Exchange::post_to` helpers become `Exchange::send(Message)`);
`tests/client-protocol/src/{engine.rs, front.rs}` (the mount between the
server and the component);
`crates/jig-protocol-config/src/{profile.rs, profile/small.rs,
profile/standard.rs}` (`Limits.clients_http`); `testing/jig-test-service/src/{clients.rs,
server.rs, limits.rs, Cargo.toml}`; `tests/engine-clients/src/peer.rs`
(the peer mounts the engine end in front of the component);
`tests/boundary/src/tests.rs` (`engine_clients_link_only_their_boundary_wire_and_foundation`
loses `skein-http` and `skein-io`; a sibling test for the new crate) and
`tests/boundary/src/tests/graph.rs`
(`configuration_links_only_its_vocabulary_and_foundation` takes the new
crate); workspace `Cargo.toml` and `Cargo.lock`. Shared hotspots:
workspace `Cargo.toml`, `Cargo.lock`, `tests/boundary/src/tests/graph.rs`,
`crates/jig-protocol-engine-clients/src/translate/` (README, "Shared
files").
Shape: the sketches below, `jig_protocol_engine_clients::{FromTransport,
ToTransport, Message, Reply, Streamed, Ended}` and
`jig_protocol_engine_clients_http::{Mount, Settings, Limits, Event,
Request, claims, link_path}`. The component's `Event::Transport(FromTransport)`
and `Request::Transport(ToTransport)` replace `Event::Call`,
`Event::Http`, `Request::Http`, `Request::Done`. Heartbeats leave the
component as `Streamed::Heartbeat`, which the engine end writes as a
comment line. A stream ended before its head is written as `401` when
signed out and `409` otherwise, the statuses the base answers.
`jig_test_service::clients::printed` takes the path from `link_path`.
`jig_protocol_config::Limits` gains `clients_http:
jig_protocol_engine_clients_http::Limits`, its numbers moved from
`CLIENTS.http`, `CLIENTS.queued_bytes`' framing and the base's
`connections` bound on calls.
`jig_protocol_engine_clients::Limits` gains `message_bytes: u32`, the
component's bound on a message's bytes (jig's `protocol/client.md`, 4.2
and 13; ruling 314), from the old `http.body`'s numbers, the profiles'
`clients_http.body` equal to it.
Test shape: step tests in `crates/jig-protocol-engine-clients-http/src/tests.rs`
on new fixtures in `src/tests/fixtures.rs` (step code's tests: no `Vec`,
no closure-taking combinator, as jig's `clippy.toml` bans them): `mount()`
(settings with prefix `/jig` and `loopback: true`, limits from
`jig_protocol_config::profile(Profile::Small).clients_http`), `call(method,
target, headers, body)` driving `Event::Call` then `Event::Http` body
events, `answered(call) -> (u16, Box<[u8]>, Box<[Header]>)` reading the
`Request::Http` responses. The reference test:
```rust
#[test]
fn a_read_post_becomes_the_component_s_read_exchange() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/read", &fixtures::bearer(), b"\x01\x02");
    assert_eq!(
        fixtures::component(&mut out),
        [FromTransport::Exchange {
            exchange: fixtures::EXCHANGE,
            credential: Some(fixtures::credential()),
            message: Message::Read(Box::from(b"\x01\x02".as_slice())),
        }]
    );
}
```
Tests, each under 20 ms: `a_read_post_becomes_the_component_s_read_exchange`,
`a_reply_is_written_as_200_with_its_record_and_no_store`,
`a_record_the_component_cannot_read_is_written_as_400`,
`a_body_past_the_route_s_bound_is_413_and_reaches_nothing`,
`a_path_under_the_prefix_with_no_route_is_404`,
`a_streamed_record_is_one_event_and_taken_once_written`,
`a_heartbeat_is_written_as_a_comment_line`,
`the_same_routes_answer_under_another_prefix`. The component's unit tests
keep their names and expected values, moved onto `Exchange::send`.
Proof: the protocol world: `cargo nextest run -p
jig-client-protocol-world -E 'test(a_slow_reader_gets_fewer_deliveries_and_others_are_answered)'`,
once: a slow reader's stream is paced by the engine end's `StreamTaken`
and the others are answered (0.31 s at the base). Nothing else is run.
Don't: give the engine end anything of jig's model (no record decoded
but the sign-in answer in 17.4); answer `OPTIONS` or send an
`Access-Control-` header; keep `skein-http` or `skein-io` in the
component's manifest.
Size: at most 300 lines, counted as code-organization.md, 7 counts its
moves and renames, among which jig 17.3 is: the command's number, and
beside it the added lines that name a moved path or a new name
(`jig_protocol_engine_clients_http`, `call.rs`, `dispatch.rs`,
`credential.rs`, the per-call half of `connection.rs`, which `-M` does not
follow), the Size checked against the difference. Landed at 743 under
the README's "A move's Size" (ruling 314: 861 added, 452 removed).
Removes: `Event::Call`, `Event::Http`, `Request::Http`, `Request::Done`,
the component's `connection.rs`, `routes::route`, `sign_in::bearer`,
`skein-http` and `skein-io` from the component's dependencies.

### 17.4 The browser's credential: the cookie and the required header

Scope: the cookie (`client.md`, 17.3; `browser.md`, 11): set on a
browser's sign-in answer, whose body then carries no credential; read
as each exchange's and the stream's credential; cleared (`Max-Age=0`)
on a sign-out's answer; `cookie()` for the server after a `Minted`; the
required header's guard (`browser.md`, 12.2): a request the cookie
authenticates without `Jig-Client: browser` answered `403`, with nothing
done.
Needs: 17.3.
Files: `crates/jig-protocol-engine-clients-http/src/{credential.rs,
lib.rs, dispatch.rs}` and its tests; the client wire's schema and
generated code, `crates/jig-client-protocol-wire/schema/*.schema`
(`SignedIn.token` optional) and `crates/jig-client-protocol-wire/src/generated/`;
the client end that decodes a sign-in answer (the link's sign-in answer
in `crates/jig-client-protocol-link` or `crates/jig-client`, whichever
jig 12 put it in; found by `git grep -n 'SignInAnswer::SignedIn'`), and
`crates/jig-client-shell-native` where it reads the token; the
component's `component.rs` and `services.rs` (they build `SignedIn` with
`token: Some(..)`); the engine-clients world's `peer.rs` (`sign_in_answer`);
`crates/jig-client-protocol-wire/src/tests.rs`. The repository downstream
of jig reads `SignedIn` in two test files, which follow at its repin.
Shape:
- Schema: `record SignedIn { token: option secret 48 party: u64 expires:
  u64 }`, edited in place in the wire's current version.
- `jig_protocol_engine_clients_http::cookie(settings: &Settings,
  credential: &Secret, life: Duration) -> skein_http::Header`: `Set-Cookie:
  jig-sign-in=<base64url>; Path=<prefix>; HttpOnly; SameSite=Strict;
  Max-Age=<seconds>`, with `; Secure` unless `settings.loopback`;
  `fn cleared(settings: &Settings) -> skein_http::Header` with `Max-Age=0`.
- In `credential.rs`: `fn read(call: &Call) -> Read` with `enum Read {
  Bearer(Secret), Cookie(Secret), None }`; the bearer wins when both come.
- In `dispatch.rs`: a call carrying the cookie and no `Jig-Client:
  browser` is answered `403` before any message leaves; a browser's
  exchange on `sign-in/link` or `sign-in/deployment` whose reply decodes
  as `SignInAnswer::SignedIn` with a token is written with `cookie(..)`
  and the record again with `token: None`; a browser's `sign-out` reply
  is written with `cleared(..)`.
Test shape: step tests in `crates/jig-protocol-engine-clients-http/src/tests.rs`
on 17.3's fixtures, plus `fixtures::browser()` (the header and a cookie)
and `fixtures::signed_in_reply(token, expires)`. The reference test:
```rust
#[test]
fn a_browser_s_link_sign_in_sets_the_cookie_and_answers_no_token() {
    let mut mount = fixtures::mount();
    let mut out = Queue::with_capacity(16);
    let call = fixtures::call(&mut mount, &mut out, Method::Post, b"/jig/v1/sign-in/link", &fixtures::browser_header(), &fixtures::link_record());
    fixtures::reply(&mut mount, &mut out, call, fixtures::signed_in_reply(fixtures::TOKEN, fixtures::IN_A_DAY));
    let (status, body, headers) = fixtures::answered(&mut out, call);
    assert_eq!(status, 200);
    assert_eq!(fixtures::header(&headers, b"set-cookie"), Some(fixtures::COOKIE_ON_LOOPBACK));
    assert_eq!(fixtures::signed_in(&body).token(), None);
}
```
where `COOKIE_ON_LOOPBACK` is the literal
`b"jig-sign-in=<the token's base64url>; Path=/jig; HttpOnly; SameSite=Strict; Max-Age=86400"`.
Tests, each under 20 ms:
`a_browser_s_link_sign_in_sets_the_cookie_and_answers_no_token`,
`a_native_sign_in_answers_its_token_in_the_body`,
`the_cookie_is_secure_off_loopback`,
`a_browser_exchange_s_cookie_is_its_credential`,
`a_cookie_without_the_browser_header_is_403_and_reaches_nothing`,
`a_browser_s_sign_out_clears_the_cookie`; and in the client end that
decodes the answer (`jig-client-protocol-link` or `jig-client`, as jig 12
placed it), `a_sign_in_answer_with_no_token_leaves_none_held`.
Proof: unit layer only: no browser exists in jig until jig 19; the
native path's sign-in through the engine end runs in the protocol world,
`cargo nextest run -p jig-client-protocol-world -E
'test(a_request_whose_answer_was_lost_is_made_once)'`, once (0.55 s at
the base), showing the bearer path unchanged; jig 19.4's first browser
story connects the cookie.
Don't: let the component see the browser header or the cookie; keep a
credential in a browser's sign-in body; add a header or a cookie to a
native exchange's answer.
Size: at most 200 lines (the schema's generated code not counted).
Removes: nothing.

### 17.5 The other example guards, and the worlds' `Host`

Scope: the `Host` allow-list (`421`), a POST whose `Content-Type` is not
`application/octet-stream` (`415`), no `OPTIONS` answered (`404`) and
no `Access-Control-` header, `X-Content-Type-Options: nosniff` beside
`Cache-Control: no-store` on each of the mapping's answers
(`browser.md`, 12), and a `GET` of the link's route left to the
application's page (`claims` says no) with the referrer policy's header
for the server to add (`browser.md`, 12.2, 19); the worlds' clients send
the `Host` the list takes.
Needs: 17.4.
Files: `crates/jig-protocol-engine-clients-http/src/{guards.rs (new),
dispatch.rs, lib.rs}` and its tests; `testing/jig-test-service/src/{server.rs,
clients.rs}` (the mount's hosts: the listener's bound address as the
printed link names it); the native shell's `Host`
(`crates/jig-client-shell-native`, which sends `localhost` until a link
sets it at the base, `crates/jig-shell-native/src/connections.rs:58`):
its configured engine address; `tests/engine-clients/src/peer.rs:47`
(`localhost`, at the base) and its fixtures;
`testing/jig-test-service/src/tests/clients.rs:83` (a raw request with
`Host: localhost`); the native shell's test of its head
(`crates/jig-shell-native/src/tests.rs:82` at the base asserts `host:
localhost`, an expected literal this changes). Found by `git grep -n
localhost -- crates testing tests examples` (`logs/w8/users-17.5.txt`;
its other hits are OAuth redirects, untouched). The repository
downstream of jig sends `Host: localhost` from its own test peer (2
lines), which it follows at its repin.
Shape:
- `jig_protocol_engine_clients_http::Event::Listening { hosts:
  List<Box<[u8]>> }`, which the server sends once its listener is bound;
  until then the list is empty and each call is `421`.
- `guards.rs`: `fn host(settings: &Settings, call: &Call) -> bool`, the
  `Host` value equal, byte for byte, to an entry of the list; `fn content(call:
  &Call) -> bool`; their order in `dispatch`: host, method and route
  (`404`), required header (17.4), content type, then the body.
- `pub fn claims(settings: &Settings, call: &Call) -> bool`: false for a
  `GET` of `<prefix>/v1/sign-in/link` and for paths outside the prefix.
- `pub fn link_page() -> Header`: `Referrer-Policy: no-referrer`, the
  header the server adds to the page it answers at the link's route.
Test shape: step tests in `src/tests.rs` on 17.3's fixtures, `mount()`
now sending `Event::Listening { hosts: ["127.0.0.1:7100"] }` and
`call(..)` adding `Host: 127.0.0.1:7100` unless the test names one, so
17.3's and 17.4's tests keep their names and expected values; plus
`fresh_mount()`, one not yet listening. A table test for the refusals,
its rows in a `const`:
```rust
const REFUSED: &[(Method, &[u8], &[(&[u8], &[u8])], u16)] = &[
    (Method::Post, b"/jig/v1/read", &[(b"host", b"evil.test:7100"), (b"content-type", b"application/octet-stream")], 421),
    (Method::Post, b"/jig/v1/read", &[(b"host", b"127.0.0.1:7100"), (b"content-type", b"text/plain")], 415),
    (Method::Options, b"/jig/v1/read", &[(b"host", b"127.0.0.1:7100")], 404),
];
#[test]
fn each_guard_refuses_with_nothing_reaching_the_component() {
    for (method, target, headers, status) in REFUSED {
        let mut mount = fixtures::mount();
        let mut out = Queue::with_capacity(16);
        let call = fixtures::call(&mut mount, &mut out, *method, target, headers, b"");
        let (answered, _, sent) = fixtures::answered(&mut out, call);
        assert_eq!((answered, fixtures::component(&mut out).len()), (*status, 0), "{target:?} {headers:?}");
        assert!(!fixtures::names_a_cors_field(&sent), "no CORS header for {target:?}");
    }
}
```
Tests: `each_guard_refuses_with_nothing_reaching_the_component`,
`a_call_before_listening_is_421`,
`a_get_of_the_link_route_is_not_claimed_and_spends_nothing`,
`the_mapping_s_answers_carry_nosniff_and_no_store`; and in
`jig-client-shell-native`, `the_native_shell_sends_its_engine_s_address_as_host`.
Each under 20 ms.
Proof: the protocol world: `cargo nextest run -p
jig-client-protocol-world -E 'test(a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat)'`
(or the name jig 16 gave it), once (0.14 s at the base): the native
client's `Host` passes the service's allow-list and its chat starts. And
the engine-clients world, `cargo nextest run -p
jig-engine-clients-world`, once, its tests passing with the peer's `Host`
changed (37 at the base, `logs/w8/base-jig-worlds.txt`; 17.1 moves five
to `Mint`, and 17.2 moves one to the startup's tests and two to the
server's).
Don't: add a field that turns a guard off; answer the link route's `GET`
from the engine end; send `Access-Control-` headers on a refusal.
Size: at most 200 lines.
Removes: the `localhost` defaults named in Files.

## Interface sketches

Compiled at the base in a scratch crate of jig's workspace with `cargo clippy -p jig-sketch-w8 --all-targets -- -D warnings`, which
passed (`logs/w8/sketch-17-18.txt`; the component's messages again after the review, `logs/w8/sketch-review.txt`), with bodies that return a trivial
value; the component's enums there hold only the new variants, the
base's others elided as below. Names an earlier session of this plan
renames (the wire crate's) are checked at the ready check.

```rust
// crates/jig-protocol-engine-clients/src/boundary.rs
/// What the transport hands the clients' component (protocol/client.md, 4.2).
pub enum FromTransport {
    /// One of jig's messages, its credential as opaque bytes or none; answered by one `Reply`.
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    /// A client opened its stream; ended by `StreamEnded` or `StreamGone`.
    StreamOpen { stream: Token, credential: Option<Secret> },
    /// The transport wrote the last `Streamed` of this stream.
    StreamTaken { stream: Token },
    /// The stream's connection ended; terminal for the stream.
    StreamGone { stream: Token },
}
/// One of jig's messages by its route, its record undecoded.
pub enum Message {
    /// A keyed request.
    Request(Box<[u8]>),
    /// A read.
    Read(Box<[u8]>),
    /// Opening a watch.
    Watch(Box<[u8]>),
    /// Closing a watch.
    Unwatch(Box<[u8]>),
    /// A stream gone, by its name.
    Gone(Box<[u8]>),
    /// Signing in with the printed link's secret.
    SignInLink(Box<[u8]>),
    /// Signing in as a service; the service's credential is the exchange's.
    SignInService(Box<[u8]>),
    /// Signing out the credential's sign-in.
    SignOut(Box<[u8]>),
}
/// What the component hands its transport.
pub enum ToTransport {
    /// The exchange's one answer.
    Reply { exchange: Token, reply: Reply },
    /// One message on the stream; the next waits for `StreamTaken`.
    Streamed { stream: Token, message: Streamed },
    /// The stream is over; terminal.
    StreamEnded { stream: Token, why: Ended },
}
/// An exchange's answer.
pub enum Reply {
    /// One record: an answer, a page, a watch admitted or refused, a sign-in's answer.
    Record(Box<[u8]>),
    /// The message's bytes are not a record of its route.
    NotARecord,
    /// A second stream for one sign-in, or a watch already closing.
    Conflict,
}
/// One streamed message.
pub enum Streamed {
    /// The hello, a delivery, a run's text, a watch ended: one record.
    Record(Box<[u8]>),
    /// Nothing sent for the heartbeat's span.
    Heartbeat,
}
/// Why a stream ended.
pub enum Ended {
    /// Its credential is unknown, or its sign-in ended.
    SignedOut,
    /// The engine stops.
    Stopping,
    /// Its sign-in has a stream already, or the streams are full.
    Busy,
}
/// Why a `Mint` made no credential (protocol/client.md, 8).
pub enum MintRefusal {
    /// The provider is not one of kind application.
    NotApplication,
    /// The login or the name is not UTF-8 or past its bound.
    Bound,
    /// The engine stops.
    NotReady,
    /// At the exchange limit, the mints in flight, or the people child's sign-ins.
    Busy,
}
pub enum Event {
    /// From the transport.
    Transport(FromTransport),
    /// From the application's server code only (protocol/client.md, 8).
    Mint { exchange: Token, identity: jig_domain_people::Identity },
    // ...the core's, the application's and the secrets' events, Stop, Close, Abort, as at the base
}
pub enum Request {
    /// To the transport.
    Transport(ToTransport),
    /// To the application's server code, for its `Mint`.
    Minted { exchange: Token, credential: Secret, expires: Wall },
    /// To the application's server code, for its `Mint`.
    MintRefused { exchange: Token, why: MintRefusal },
    /// To the application's startup: the printed link's one-time secret.
    Link { secret: Secret },
    // ...Core, Application, Secrets, Closed, as at the base
}
```

```rust
// crates/jig-protocol-engine-clients-http/src/lib.rs
//! The default HTTP mapping's engine end (protocol/client.md, 17.4; browser.md, 12.2).
#![cfg_attr(not(test), no_std)]
#![forbid(unsafe_code)]
/// What the application gives the mount.
pub struct Settings {
    /// The path the server mounts the mapping at, with no trailing `/`: `/jig` in jig's test service.
    pub prefix: Box<[u8]>,
    /// The listener is on loopback: the cookie is set without `Secure` (browser.md, 11).
    pub loopback: bool,
}
/// The engine end's bounds, from jig-protocol-config's profiles (browser.md, 14).
pub struct Limits {
    /// Calls in flight, streams among them.
    pub calls: u32,
    /// Streams at once, across clients.
    pub streams: u32,
    /// A request body's bytes, per route the most.
    pub body: u32,
    /// The most read of a body at once.
    pub read: u32,
    /// The most written of an answer at once.
    pub send: u32,
    /// One event's bytes, framing included.
    pub event: u32,
}
/// The engine end of one mount; the server keeps one per prefix.
pub struct Mount { /* settings, hosts, calls by the server's name, streams */ }
/// To the mount.
pub enum Event {
    /// The server's listener is bound: the `Host` values it accepts (browser.md, 12.2).
    Listening { hosts: List<Box<[u8]>> },
    /// A call the server routed here, which `claims` took, by the server's name for it.
    Call { call: Token, head: skein_http::server::Call },
    /// What the call's connection said after its head, until `Done`.
    Http { call: Token, event: skein_http::server::Event },
    /// The clients' component's message to its transport.
    Component(jig_protocol_engine_clients::ToTransport),
}
/// From the mount.
pub enum Request {
    /// For the call's connection's skein-http server.
    Http { call: Token, request: skein_http::server::Request },
    /// For the clients' component.
    Component(jig_protocol_engine_clients::FromTransport),
}
impl Mount {
    pub fn new(settings: Settings, limits: &Limits) -> Self;
    pub fn down(&mut self, env: &Env<Limits>, event: Event, out: &mut Queue<Request>);
    pub fn up(&mut self, env: &Env<Limits>, out: &mut Queue<Request>) -> bool;
    pub fn next_deadline(&self) -> Option<Time>;
    pub fn fire(&mut self, env: &Env<Limits>, out: &mut Queue<Request>);
}
/// Whether a call is the mount's: under its prefix, but a `GET` of the link's route (client.md, 17.1).
pub fn claims(settings: &Settings, call: &skein_http::server::Call) -> bool;
/// `<prefix>/v1/sign-in/link?secret=<base64url>`, the path of the printed link's URL (client.md, 8).
pub fn link_path(settings: &Settings, secret: &Secret) -> Box<[u8]>;
/// The cookie a sign-in's answer, or the server after a `Minted`, sets (client.md, 17.3).
pub fn cookie(settings: &Settings, credential: &Secret, life: Duration) -> skein_http::Header;
/// The cookie cleared, on a sign-out's answer.
pub fn cleared(settings: &Settings) -> skein_http::Header;
/// `Referrer-Policy: no-referrer`, for the page the server answers at the link's route (browser.md, 12.2).
pub fn link_page() -> skein_http::Header;
```

The dispatch, which is easy to get wrong in its order (17.4, 17.5):

```rust
// crates/jig-protocol-engine-clients-http/src/dispatch.rs, on Event::Call
if !guards::host(&self.hosts, &head) { return respond(421) }
let Some(route) = route(&self.settings.prefix, &head) else { return respond(404) }; // OPTIONS, other paths
let credential = credential::read(&head);
let browser = head.header(b"jig-client") == Some(b"browser");
if matches!(credential, Read::Cookie(_)) && !browser { return respond(403) }
if route.is_post() && head.header(b"content-type") != Some(b"application/octet-stream") { return respond(415) }
read the body up to the route's bound (413 past it), then hand the component Exchange or StreamOpen
```

## Questions to raise when reached

- None the design leaves open that an increment needs; the token's
  absence for a browser's sign-in is settled above from `client.md`,
  8 and 17.3.

## Done when

Each command runs once, after the session's last increment lands.

- `git -C ~/src/rust/jig log origin/main --grep='^Plan: client-api$'
  --format='%(trailers:key=Step,valueonly,separator=)' | grep
  '^jig-17\.' | sort -u | wc -l` prints 5.
- `git -C ~/src/rust/jig grep -n -w -e Listen -e skein_http origin/main
  -- crates/jig-protocol-engine-clients/src` prints nothing.
- `git -C ~/src/rust/jig grep -n -e 'sign-in/provider' -e
  'Request::Provider' origin/main -- crates testing tests` prints nothing.
- In a worktree of jig at `origin/main`, `cargo nextest run -p
  jig-protocol-engine-clients-http` passes 18 tests (17.3's 8, 17.4's 6,
  17.5's 4), `each_guard_refuses_with_nothing_reaching_the_component`
  among them; 17.4's `a_sign_in_answer_with_no_token_leaves_none_held` is
  in the client end that decodes the answer, and 17.5's
  `the_native_shell_sends_its_engine_s_address_as_host` in
  `jig-client-shell-native`.
