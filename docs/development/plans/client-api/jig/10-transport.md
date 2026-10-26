# Session 10: the link's HTTP half becomes `jig-client-protocol-http`

The link (`jig-client-protocol-link`, so named by jig 01) speaks HTTP
today: it builds paths, reads statuses, cuts the server-sent events, and
keeps each exchange's deadline, while the native shell holds the
credential and signs in. This session moves the HTTP half into a new
step crate, `jig-client-protocol-http`, the default mapping's client end
(`protocol/client.md`, 17.4); gives the link the transport vocabulary of
`protocol/client.md`, 4.3 (`Exchange { exchange, credential, message }`,
`Reply`, `Lost`, `Streamed`, `StreamGone`); moves the sign-in and the
credential from the native shell into the link, so that the credential
travels on each message; and puts the native shell
(`jig-client-shell-native`) on the new crate. No wire record changes. It
runs beside jig 11 and before jig 12, which composes the link into
`jig_client::Client`.

## Read first

- jig's `docs/design/protocol/client.md`, sections 2, 4 (4.1 to 4.3), 8,
  12 and 17 (17.1 to 17.4).
- jig's `docs/design/client/domain/client.md`, sections 3, 12 and 16.
- jig's `docs/design/README.md`, sections 10 and 15.
- The plan's `findings/wire-and-client.md`, section 3 (the HTTP half, by
  line), and `findings/renames.md`, section 1 (the crates' names after
  jig 01).
- `code-organization.md`, sections 1, 5, 6 and 7.
- skein's `programming-model.md`, sections 3, 4.3, 4.4 and 10;
  `testing-strategy.md`, sections 2.5 and 2.8.

## What the design asks

- "`jig-client-protocol-http`, the client end, step code for the host and
  for wasm32: an `Exchange` as a method, a path under the prefix, headers
  (the bearer, or the browser's required header) and a body; a response's
  status and body back as a `Reply`, or `Lost`; `OpenStream` as the
  stream's `GET`, its body cut into messages (17.2)" (protocol/client.md,
  17.4).
- The link: "emitted: `Exchange { exchange, credential, message } |
  OpenStream { credential } | CloseStream | Save { records } | Erase {
  records }`; heard: `Reply { exchange, message } | Lost { exchange } |
  Streamed { message } | StreamGone | Stored | NotStored`", and "There is
  no method, path or header here: those are the transport's"
  (protocol/client.md, 4.3).
- "One attempt per exchange, whose terminal is a `Reply`, or `Lost`: the
  exchange failed, or ended without a whole reply within the exchange's
  deadline, which the transport keeps" (4.3); "The transport makes one
  attempt; retrying is the client's" (client.md, 16).
- The credential "is the link's" (client.md, 12) and is put "on each
  `Exchange` and `OpenStream`" (client.md, 3); a sign-in's answer gives
  it; "A host that keeps a sign-in across its own restart keeps the
  credential where its platform keeps secrets, and gives it back with
  `Held`" (client.md, 12).
- Routes under the prefix (17.1): `GET <prefix>/v1/stream`, `POST
  <prefix>/v1/request`, `/read`, `/watch`, `/unwatch`, `/stream-gone`,
  `/sign-in/link`, `/sign-in/deployment`, `/sign-out`; a POST's body
  `application/octet-stream`; the credential as `Authorization: Bearer`
  in base64url outside a browser (17.3); the stream as server-sent
  events whose data is a record in base64 and whose heartbeat is a
  comment line (17.2).

## Choices this plan makes

- **The message names its kind; the transport names the route.** The
  link's `Message` is an enum of jig's messages, each carrying its
  encoded record (sketch below); `jig_client_protocol_http::exchange`
  maps each variant to its route. The engine end keeps its present
  routes (`/jig/v1/...`) until jig 17; the native shell makes its
  mapping with the prefix `/jig`, so the bytes on the wire do not move.
- **The link holds the credential and signs in.** `jig-client-domain`
  never holds it (client.md, 3); the link is the protocol layer of
  `jig-client` that does. This session builds `With::Link` and
  `With::Held`, the two the native shell needs; jig 12.3 adds
  `With::Service` and `With::Transport` and the domain's side of signing
  out.
- **The exchange's deadline moves to the transport.** The link keeps no
  per-exchange deadline and asks for no `Abandon`; the native shell's
  `Connections` answers `Lost` past `Limits::deadline`. A late reply to
  a watch opening the link already closed is dropped by the watch's
  name, as today.
- **The small store's messages stay as they are** (`Write`, `Remove`,
  `Written`, `WriteFailed`): jig 12.1 gives them 4.3's names (`Save`,
  `Erase`, `Stored`, `NotStored`) and `jig_client::Client`'s `Save {
  records }`, and jig 12.2 gives the client client.md, 11's barrier.
  Changing them here would change them twice.
- **No browser header.** `Jig-Client: browser` and the cookie are jig
  19's, when a browser shell uses this crate; this session builds the
  bearer only.

## Rules that bind it

- **No wire record changes** (README, Rules): `git diff origin/main --
  crates/jig-client-protocol-wire/schema` prints nothing at each
  increment's end. The client wire's v8 is jig 11.3's.
- **No HTTP in the link once 10.3 lands** (code-organization.md, 6):
  10.3 adds the boundary test
  `client_step_crates_link_no_http_but_the_mapping` (below), which the
  gate runs from then on.
- **jig never names the repository downstream of jig**: `git -C <the
  session's jig worktree> grep -n -i -w temper -- crates testing
  tests examples scripts` prints nothing at each increment's end
  (code-organization.md, 6).

## Increments

### 10.1 The link signs in and holds the credential
Scope: the printed link's sign-in and signing out move from the native
shell into the link, which holds the credential the answer gives and
puts it on each exchange and on the stream's opening (client.md, 3 and
12; protocol/client.md, 8). A host gives a kept credential back as
`With::Held`.
Needs: jig 01's renames on `main`.
Files: `crates/jig-client-protocol-link/src/{boundary.rs, component.rs,
store.rs, lib.rs}`, a new `src/sign_in.rs`; the link's `src/tests.rs`
(`mod sign_in;`), a new `src/tests/sign_in.rs`, and the exhaustive
matches on `Request` that `Request::Signing` breaks, in
`src/tests/fixtures.rs:96,130,163` and
`src/tests/fixtures/{link_deliveries.rs:117, link_reads.rs:27,61,86,
link_store.rs:191,211, link_stream.rs:99-241}`;
`crates/jig-client-shell-native/src/{connections.rs, shell.rs,
sign_in.rs, files.rs, lib.rs}` (shared hotspot with 10.2 and 10.3).
Users of the shell's sign-in API (found by `git grep -n -E
'set_token|\.signing\(\)|Signing::|connections\.sign_in|\.sign_in\(&'`,
log `search-jig10-jig.txt`): `tests/client-protocol/src/{front.rs,
front/persistence.rs:100, native.rs, native/world.rs}` (persistence.rs
calls `set_token` and `connections.token()`),
`tests/one-process/src/referee.rs:265`,
`testing/jig-test/tests/support/mod.rs:169`; in the repository downstream
of jig, its terminal's `client.rs`, `terminal.rs`, `terminal/native.rs`,
`tests/support/mod.rs:128` (`sign_in -> Result<(), Unlinked>`) and
`src/tests.rs:146`, its binary's `tests/support/person.rs` and its
one-process world's `world.rs:302`, which its own sessions follow.
Shape: in `jig_client_protocol_link::boundary`: `With { Link(Secret),
Held(Secret) }`; `Signing { SignedIn { party }, Again, Refused, Lost,
SignedOut }` (moved from the shell's `sign_in.rs`, without the exchange
tokens, which become the link's); `Event::SignIn { with: With }`,
`Event::SignOut`; `Request::Signing { signing: Signing }`;
`Request::Exchange { exchange, credential: Option<Secret>, path, body }`
and `Request::OpenStream { credential: Option<Secret> }` (the field
added; `path` until 10.2). `Link::credential(&self) -> Option<&Secret>`
for a host that keeps it. In the link, `sign_in.rs`: `LinkSignIn`
encoded on `With::Link`, `SignInAnswer` and `SignOutAnswer` decoded as
the shell's `sign_in::answer` does today (`SignedIn` with a token of 40
bytes holds it; `NotReady` is `Again`; a non-200 or malformed answer is
`Lost`; signed out drops it). The shell: `Connections` loses
`set_token`, `token`, `sign_in`, `sign_out`, `signing` and its
`Owner::{SignIn, SignOut}`; its bearer comes from each request's
`credential`; `Shell::sign_in(printed) -> Result<Secret, Unlinked>`
returns the secret for its host to give the link, and keeps today's
address check and `Host` (connections.rs:82-85, :158): a printed link
whose host is not the configured one, unless that is `localhost`, is
`Unlinked::Address`; otherwise its host becomes the one each exchange's
`Host` header names (protocol/client.md, 4.3: "There is no method, path
or header here: those are the transport's"); `Shell::loaded()`
returns the kept credential beside the records,
`Option<(Option<Secret>, Vec<StoreRead>)>`, `None` until loaded as today
(shell.rs:85; its user `tests/client-protocol/src/native.rs:301`), and
`Shell::keep(credential: Option<Secret>)` writes it to its private file,
which its host calls on `Signing::SignedIn` and `SignedOut`, before it
tells anyone (as `saving_signing` orders it today).
Test shape: `crates/jig-client-protocol-link/src/tests/sign_in.rs`, new,
on `src/tests/fixtures.rs`'s `Harness` (fixtures.rs:84), which gains
`sign_in(&mut self, with: With) -> Option<Token>` (the sign-in's
exchange, if one left) and `credentials(&self) -> Vec<Option<Secret>>`
(each `Exchange`'s credential, in order); `fixtures::signed_in(party:
u64, token: &[u8; 40]) -> Box<[u8]>` encodes `SignInAnswer::SignedIn`.
Limits from `fixtures::limits()` (fixtures.rs:37), the link's own test
limits. Reference test:
```rust
#[test]
fn a_sign_in_answer_puts_its_credential_on_each_exchange() {
    let mut h = Harness::new(&limits());
    let exchange = h.sign_in(With::Link(Secret::new(Box::new([3; 32])))).expect("one sign-in exchange");
    h.answer(exchange, 200, &signed_in(7, &[9; 40]));
    assert!(h.has(&Request::Signing { signing: Signing::SignedIn { party: 7 } }));
    h.send(ATTEMPT, KEY, start_chat());
    assert_eq!(h.credentials().last(), Some(&Some(Secret::new(Box::new([9; 40])))));
}
```
Tests: `a_sign_in_answer_puts_its_credential_on_each_exchange`,
`a_held_credential_is_carried_without_an_exchange`,
`a_not_ready_sign_in_is_again_and_holds_nothing`,
`a_refused_secret_is_refused_and_holds_nothing`,
`a_lost_sign_out_keeps_the_credential`,
`a_signed_out_answer_drops_the_credential`. The shell's
`a_sign_in_keeps_its_token_only_after_a_whole_answer`,
`a_not_ready_signing_exchange_takes_nothing_and_can_be_sent_again`,
`a_whole_sign_out_answer_clears_the_shell_token` and
`a_lost_sign_out_keeps_the_shell_token` (shell `src/tests.rs:148-199`)
move to these; `an_exchange_carries_the_bearer_token` (tests.rs:70)
stays, its credential given on the request. Each under 20 ms.
Proof: process layer, the native shell signing in by the printed link
over real sockets: `sh -c 'cargo build -p jig-test --bin jig-test
&& cargo nextest run -p jig-test --profile process -E
"test(a_killed_jig_test_restarts_and_the_work_is_found)"'`, once; it
passes when the restarted client's held credential reads the chat again
(under a minute, the build included). At the base this test fails
before the client runs ("configuration JSON: missing field `prices`"):
the agent's model at `testing/jig-test/tests/support/mod.rs:30` lacks
the `prices` smith now requires. This increment adds
`"prices":{"input":1,"cached":1,"output":1,"unit":1000}` to that model,
one line under the README's 20-line rule (tried at the base: the test
then passes, in 0.27 s). It passes; nothing else is wanted. Below it,
the gate runs `a_person_signs_in_by_the_printed_link_and_starts_a_chat`
(`jig-client-protocol-world`, real engine over HTTP bytes); nothing else.
Don't: no credential in the link's small-store records or in any
`Request` but `Exchange` and `OpenStream`; no `With::Service` or
`With::Transport` (jig 12.3); no change to the engine end.
Size: at most 220 lines (the link's sign-in about 120; the shell's
changes are mostly removals).
Removes: the shell's token, its sign-in exchanges and `Signing` with
exchange tokens.

### 10.2 `jig-client-protocol-http`: an exchange as a call
Scope: the new crate maps a message and the credential to a call
(method, path under the prefix, headers, body) and a response to `Reply`
or `Lost` (17.1, 17.3, 17.4); the link emits `Exchange { exchange,
credential, message }` and hears `Reply` and `Lost`; the paths, the
status checks and the per-exchange deadline leave the link; the printed
link's parsing moves from the shell into the new crate.
Needs: 10.1's `Request::Exchange { credential }`.
Files: new `crates/jig-client-protocol-http/` (`Cargo.toml`, `src/lib.rs`,
`src/call.rs`, `src/printed.rs`, `src/tests.rs`,
`src/tests/fixtures.rs`); workspace `Cargo.toml` and `Cargo.lock`
(shared); `crates/jig-client-protocol-link/src/{boundary.rs,
component.rs, store.rs, limits.rs}` (the paths at component.rs:144,
178, 386, 453, 641 and store.rs:168-170, the statuses at 84-88, 193,
244, 464; `Limits.deadline`); `crates/jig-client-shell-native/src/
{connections.rs, shell.rs, sign_in.rs (removed)}`;
`tests/boundary/src/tests.rs:244-252` (the native shell's dependency
list gains `jig-client-protocol-http`) and `tests/boundary/src/tests/
graph.rs:138` (`client_step` gains it) (shared). Users of
`Request::Exchange { path, body }`, `link::Event::Answered`,
`Event::Failed`, `Request::Abandon` and `link::Limits::new(..,
deadline)` with its `limits().deadline`:
`tests/client-protocol/src/{front.rs, fixtures.rs:102}`, the link's
`src/tests/{attempts.rs, fixtures.rs, reads.rs:62,83,93, store.rs:172,
pages.rs:174, stream.rs:250,253}` and
`src/tests/fixtures/{documents.rs:322, link_reads.rs:26-87,289,
link_store.rs:186-215, link_stream.rs:95-242, link_deliveries.rs:117,
turns.rs:100}`, 10.1's `src/tests/sign_in.rs` (its `h.answer(.., 200,
..)`), and the shell's
`src/tests.rs:37,84,98,110,258,283-289`; the link's `Harness::answer(..,
status: u16, ..)` (fixtures.rs:120) matches Done when's grep, so it loses
`status`, and with it its callers (about 35). Users of the shell's
`Limits`, which gains `deadline`:
`tests/client-protocol/src/fixtures.rs:94-97` (`connections_limits()`,
which reaches `front.rs:35`, `native/world.rs:183`,
`testing/jig-test/tests/support/mod.rs:87` and
`tests/one-process/src/fixtures.rs:73`) and the shell's `src/tests.rs:13`.
Downstream, its terminal's `route.rs:52-57`, `tests.rs:86-92`,
`limits.rs:71`, `limits.rs:100` (`link::Limits::new(.., DEADLINE)`) and
`client.rs:263`.
Shape: as the sketch: `Mapping::new(prefix) -> Result<Mapping,
Unmapped>`, `Call { method, path, headers, body }`, `exchange(&Mapping,
Option<&Secret>, Message) -> Call`, `answered(exchange, status, body) ->
Heard`, `printed(&Mapping, url) -> Result<Printed, Unlinked>`. The link:
`Message` replaces `path` and `body`; `Event::Reply { exchange, message:
Box<[u8]> }` and `Event::Lost { exchange }` replace `Answered { exchange,
status, body }` and `Failed`; `Request::Abandon` goes; `Limits::new(client,
exchanges)` (no deadline). The shell's `Limits` gains `deadline:
Duration`, past which `Connections` closes the exchange and answers
`Lost`; `Shell::next_deadline` (shell.rs:327) includes each open
exchange's deadline, which wakes `Connections` to answer it.
Dependencies: `skein-lib`, `skein-http` (its `Method` and `Header`),
`jig-client-protocol-link` (its `Message` and `Heard`); the crate is
`no_std` with `alloc`.
Test shape: `crates/jig-client-protocol-http/src/tests.rs` and
`src/tests/fixtures.rs` (the crate's fixtures module,
code-organization.md, 5): `fixtures::mapping()` (`/jig`),
`fixtures::credential()` (40 bytes of 9). Reference test:
```rust
#[test]
fn a_keyed_request_is_a_post_under_the_prefix_with_its_bearer() {
    let call = exchange(&mapping(), Some(&credential()), Message::Request(Box::from(*b"keyed")));
    assert_eq!(call.method, skein_http::Method::Post);
    assert_eq!(call.path.as_ref(), b"/jig/v1/request");
    assert_eq!(call.body.as_ref(), b"keyed");
    let bearer = skein_http::Header {
        name: Box::from(*b"Authorization"),
        value: Box::from(*b"Bearer CQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQkJCQ"),
    };
    assert!(call.headers.as_slice().contains(&bearer));
}
```
Tests: `a_keyed_request_is_a_post_under_the_prefix_with_its_bearer`,
`each_message_maps_to_its_route_under_the_prefix` (a table of the
seven, one behaviour), `a_message_with_no_credential_carries_no_bearer`,
`a_status_other_than_200_is_lost`, `a_printed_link_gives_its_host_and_secret`,
`a_printed_link_under_another_prefix_is_refused`. The link's
`non_success_statuses_and_malformed_outer_bodies_are_lost`
(attempts.rs:96) keeps its malformed-body half; its status half moves
here. `an_attempt_past_its_deadline_is_abandoned_and_lost`
(attempts.rs:73) moves to the shell as
`an_exchange_past_its_deadline_is_answered_lost`. Each under 20 ms.
Proof: joint layer over HTTP bytes:
`cargo nextest run -p jig-client-protocol-world -E
'test(a_request_whose_answer_was_lost_is_made_once)'`, once; it passes
when the request whose answer was cut is made once by the real engine
(under 2 s). It passes; nothing else is wanted. The gate runs the
suite's other `jig-client-protocol-world` stories.
Don't: no browser header and no cookie (jig 19); no change to the engine
end's routes (jig 17); no retry in the shell (retrying is the client's).
Size: at most 250 lines (the crate about 150, the link and shell about
100).
Removes: the link's paths, statuses, `Abandon`, its exchange deadline;
the shell's `sign_in.rs`.

### 10.3 The stream's body cut into messages by the mapping
Scope: the server-sent events reader and the base64 decoder move from
the link's `stream.rs` into `jig_client_protocol_http::Stream` (17.2);
the link hears `Streamed { message }` and `StreamGone` and emits
`OpenStream { credential }`; the shell builds the stream's `GET` with
`stream(&Mapping, credential)` and feeds its body through `Stream`. The
link loses `skein-http`.
Needs: 10.2's crate.
Files: `crates/jig-client-protocol-link/src/{stream.rs (moved to
crates/jig-client-protocol-http/src/stream.rs), component.rs (stream_bytes
508-547, line_piece 648-657), boundary.rs, limits.rs (stream_chunk),
Cargo.toml}`; `crates/jig-client-protocol-http/src/{lib.rs, stream.rs}`;
`crates/jig-client-shell-native/src/{connections.rs, shell.rs}`;
`tests/boundary/src/tests/graph.rs` (the new test below). Users of
`link::Event::{StreamOpened, StreamBytes, StreamEnded}`:
`tests/client-protocol/src/front.rs`, the shell's `connections.rs`
(`StreamOpened` at its response), the link's `src/tests/{stream.rs,
fixtures/link_stream.rs}`.
Shape: as the sketch: `Stream::new(&Limits)`, `opened(status, out)`,
`body(piece, out)`, `ended(out)`, `worst_case(&Limits)`; `Limits {
record, chunk }`. The link: `Event::Streamed { message: Streamed }` and
`Event::StreamGone` replace `StreamOpened { status }`, `StreamBytes {
bytes }` and `StreamEnded`; a `Streamed::Heartbeat` keeps each watch
alive as a comment line does today; the link still drops the stream
past twice the heartbeat with nothing heard (`Stream::deadline` stays
in the link, renamed `silence`).
Test shape: `crates/jig-client-protocol-http/src/tests/stream.rs`. The
link's `the_stream_s_body_cut_at_each_byte_reads_the_same_events`,
`an_events_data_read_as_it_arrives_decodes_the_same_whatever_its_cuts`
and `malformed_base64_drops_the_stream_and_is_reported` (link
`src/tests/stream.rs:13, 24, 55`) move here with their fixtures from
`src/tests/fixtures/link_stream.rs`, renamed
`the_body_cut_at_each_byte_gives_the_same_records`,
`an_event_s_data_read_as_it_arrives_gives_the_same_record` and
`malformed_base64_ends_the_stream`; added:
`a_comment_line_is_a_heartbeat`, `a_status_other_than_200_ends_the_stream`,
`a_piece_after_the_end_is_dropped`. The link keeps
`a_heartbeat_comment_keeps_each_watch_alive` (stream.rs:120) as
`a_heartbeat_keeps_each_watch_alive`. Each under 20 ms.
Proof: process layer: `sh -c 'cargo build -p jig-test --bin
jig-test && cargo nextest run -p jig-test --profile process -E
"test(a_killed_jig_test_restarts_and_the_work_is_found)"'`, once,
passing with the stream's deliveries read through the mapping over real
sockets (about a minute). It passes; nothing else is wanted. In the
suite, the gate runs
`a_slow_reader_gets_fewer_deliveries_and_others_are_answered`
(`jig-client-protocol-world`); nothing else.
Don't: no event names or ids, and no reconnecting by the event stream's
retry (17.2: the client reopens by its own backoffs).
Size: at most 150 lines (the moved `stream.rs` counts only its changed
lines under `git diff -M`).
Removes: the link's `stream.rs`, `line_piece`, its `skein-http`
dependency.

The boundary test 10.3 adds, in `tests/boundary/src/tests/graph.rs`,
with this body:
```rust
#[test]
fn client_step_crates_link_no_http_but_the_mapping() {
    let failures: Vec<_> = manifests()
        .into_iter()
        .filter(|m| m.name.as_deref().is_some_and(client_step) && m.name.as_deref() != Some("jig-client-protocol-http"))
        .filter(|m| m.dependencies.iter().any(|d| d == "skein-http"))
        .map(|m| m.path.display().to_string())
        .collect();
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
```

## Interface sketches

Checked at the base with `cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate (bodies
`unreachable!("sketch")`, unused parameters prefixed `_`), passing.

```rust
// crates/jig-client-protocol-link/src/boundary.rs (jig 10), the transport's side.
// Who owns what: the link encodes, decodes and holds the credential; the
// transport maps, carries, and keeps each exchange's deadline.

/// One of jig's messages (protocol/client.md, 4.1), as the link encoded it.
#[derive(Debug, PartialEq, Eq)]
pub enum Message {
    /// A keyed request's `Keyed` record, answered by an `Answer`.
    Request(Box<[u8]>),
    /// A read's `Owned` record, answered by a `ReadAnswer`.
    Read(Box<[u8]>),
    /// A watch's `Open` record, answered by an `OpenAnswer`.
    Watch(Box<[u8]>),
    /// A watch's `WatchClose` record, answered by a `CloseAnswer`.
    Unwatch(Box<[u8]>),
    /// The stream's `Gone` record; its answer is dropped.
    StreamGone(Box<[u8]>),
    /// The printed link's `LinkSignIn` record, answered by a `SignInAnswer`.
    SignInLink(Box<[u8]>),
    /// Signing out: the message's credential alone, answered by a `SignOutAnswer`.
    SignOut,
}
/// One message of the stream, as the transport cut it (17.2).
#[derive(Debug, PartialEq, Eq)]
pub enum Streamed { Record(Box<[u8]>), Heartbeat }
/// The transport's four events, which the link's `Event` carries as variants of its own.
#[derive(Debug, PartialEq, Eq)]
pub enum Heard {
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
}
/// How the link signs in (client.md, 12); jig 12.3 adds `Service(Secret)` and `Transport`.
#[derive(Debug, PartialEq, Eq)]
pub enum With {
    /// The printed link's 32-byte secret: the link sends the component's sign-in.
    Link(Secret),
    /// A credential its host kept across a restart: held, nothing sent.
    Held(Secret),
}
/// A sign-in's or sign-out's outcome, told to the link's host once per exchange.
#[derive(Debug, PartialEq, Eq)]
pub enum Signing { SignedIn { party: u64 }, Again, Refused, Lost, SignedOut }
// Event gains:   SignIn { with: With }, SignOut,
//                Reply { exchange: Token, message: Box<[u8]> }, Lost { exchange: Token },
//                Streamed { message: Streamed }, StreamGone
// Event loses:   Answered { exchange, status, body }, Failed { exchange },
//                StreamOpened { status }, StreamBytes { bytes }, StreamEnded
// Request:       Exchange { exchange: Token, credential: Option<Secret>, message: Message },
//                OpenStream { credential: Option<Secret> }, CloseStream, Signing { signing: Signing }
// Request loses: Abandon { exchange }
// Unchanged until jig 12.2: Write, Remove, Written, WriteFailed.
```

```rust
// crates/jig-client-protocol-http/src/lib.rs: it maps; it holds no
// connection, no credential and no deadline.
#[derive(Debug, PartialEq, Eq)]
pub struct Mapping { prefix: Box<[u8]> }
#[derive(Debug, PartialEq, Eq)]
pub enum Unmapped { Prefix }
impl Mapping {
    /// A prefix of at most 64 bytes starting with `/`, with no trailing `/`.
    pub fn new(prefix: &[u8]) -> Result<Mapping, Unmapped>;
}
/// One HTTP call, whatever the shell's HTTP stack.
#[derive(Debug, PartialEq, Eq)]
pub struct Call {
    pub method: skein_http::Method,
    /// `<prefix>/v1/<route>`.
    pub path: Box<[u8]>,
    /// `Content-Type: application/octet-stream` on a POST; `Authorization: Bearer <base64url>` with a credential.
    pub headers: List<skein_http::Header>,
    pub body: Box<[u8]>,
}
pub fn exchange(mapping: &Mapping, credential: Option<&Secret>, message: Message) -> Call;
pub fn stream(mapping: &Mapping, credential: Option<&Secret>) -> Call;
pub fn answered(exchange: Token, status: u16, body: Box<[u8]>) -> Heard {
    if status == 200 { Heard::Reply { exchange, message: body } } else { Heard::Lost { exchange } }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Printed { pub host: Box<[u8]>, pub secret: Secret }
#[derive(Debug, PartialEq, Eq)]
pub enum Unlinked { NotHttp, NoSecret, Address }
pub fn printed(mapping: &Mapping, url: &[u8]) -> Result<Printed, Unlinked>;
#[derive(Debug, PartialEq, Eq)]
pub struct Limits { pub record: u32, pub chunk: u32 }
#[derive(Debug)] // alone: the sse reader and the base64 decoder it holds derive no `PartialEq`.
pub struct Stream { /* skein_http::sse::Reader, skein_lib::base64::Decoder, moved from the link */ }
impl Stream {
    pub fn new(limits: &Limits) -> Stream;
    pub fn opened(&mut self, status: u16, out: &mut Queue<Heard>);
    pub fn body(&mut self, piece: &[u8], out: &mut Queue<Heard>);
    pub fn ended(&mut self, out: &mut Queue<Heard>);
}
pub fn worst_case(limits: &Limits) -> Option<u64>;
```

(`Heard`, in the link's sketch, is its four transport events, `Reply`,
`Lost`, `Streamed` and `StreamGone`, which the link's `Event` carries as
variants of its own; the crate pushes them as the link's `Event`.)

## Questions to raise when reached

- None: the routes, headers and stream format are 17.1 to 17.3's.

## Done when

- `git -C ~/src/rust/jig grep -n -E 'v1/|status: u16|skein_http'
  origin/main -- crates/jig-client-protocol-link` prints nothing (the
  clients' component keeps its HTTP until jig 17, whose Done when checks
  it; `crates/jig-client-domain` is jig 11's, which runs beside this
  session, and `crates/jig-client` jig 12's, after it, so neither need
  exist when this session ends).
- `cargo nextest run -p jig-boundary -E
  'test(client_step_crates_link_no_http_but_the_mapping)'` passes one
  test, in a worktree at `origin/main`.
- `git -C ~/src/rust/jig ls-tree -d --name-only origin/main
  crates/jig-client-protocol-http` prints the directory.
