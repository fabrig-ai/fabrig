# Session 12: `jig-client`, the client the application holds

This session makes `jig_client::Client` (`crates/jig-client`): jig 11's
`jig-client-domain` and jig 10's link composed into one step machine,
with client.md, 4's surface (`step`, `fire`, `next_deadline`;
`document`, `pending`, `link`). It gives the client the link's five
states, judged from the stream and its own watch of `me` (12.1); the
small store's barrier and kept work on its surface (12.2); `SignIn {
with }` and `SignOut` (12.3); versions per owner, with families as bytes
under their owner (12.4); and a typed transport for the client's world,
whose engine is scripted in the contract's types (12.5). It is the
meeting point (README, Rules): its first increment runs a thin story of
the new client against the real engine in `tests/client-protocol`, and
each increment after it grows that story; from here each client
increment proves against the real engine.

## Read first

- jig's `docs/design/client/domain/client.md`, sections 1 to 4, 6, 8
  (8.1 and 8.2), 11, 12, 14 and 16.
- jig's `docs/design/client/domain/root.md`, sections 3, 5, 7 and 15.
- jig's `docs/design/client/domain/testing.md`, sections 2, 4, 5 and 6.1.
- jig's `docs/design/protocol/client.md`, sections 2, 4, 8, 9, 10 and 11.
- jig's `docs/design/README.md`, section 10.
- The plan's `findings/wire-and-client.md`, sections 5 (c, e, j, k) and
  6, and `code-organization.md`, sections 1, 3, 5, 6 and 7.
- Sessions 10 and 11's files: their sketches are this session's types.
- skein's `programming-model.md`, sections 3, 4.4, 4.5, 6.3 and 9;
  `testing-strategy.md`, sections 2.5, 6 and 7.

## What the design asks

- "`jig-client` composes the two into `jig_client::Client`, one step
  machine" (client.md, 3); its surface is client.md, 4's sketch: `step(now,
  Event, &mut Queue<Request>), fire(now, &mut Queue<Request>),
  next_deadline()`, `client.document(handle), client.pending(key),
  client.link()`.
- The link's states (8.2): "signed out: no sign-in is live; starting:
  signed in, and `me`'s first delivery has not come; live: the stream is
  live, and `me` has been delivered on it; offline: the stream dropped,
  or went silent past twice its heartbeat ...; out of date: the engine
  serves no version of jig's that the client speaks. It sends nothing,
  and keeps every request." "Beside its state, the link names the party
  confirmed, each owner whose version the engine does not serve, and a
  sign-out that did not reach the engine."
- The small store (11): "A request's record leaves in a `Save`, and the
  client gives the request's first `Exchange` only once the transport
  says `Stored`. `NotStored` ... retires the requests the write carried,
  unsent, each answered *not kept*, by its key (skein's `lib.md`, 12,
  not kept): parked, or with its `Exchange` waiting for the write ... A
  record is whole in each write, so a request whose record a later write
  carries again is decided by that later write".
- `SignIn { with }` (12): `Link(secret)`, `Service(credential)`,
  `Held(credential)`, `Transport`; signed out "drops the credential
  held, ends every handle (`Ended`, signed out), parks every request,
  and tells `Link` signed out"; "A sign-out that does not reach the
  engine ... leaves the party signed in"; "A sign-out's answer is its
  sign-in's".
- Versions per owner (8.2; protocol/client.md, 9): "each family owner's,
  which the application declares as it makes the client, with its
  limits (section 14) ... Each family's request, read and subject names
  a declared owner, in its declared version; any other is the
  application's mistake, and loud"; "Judged at each hello, whole. A
  declared owner is not served when the hello names no range of it, or
  one that leaves out its version. The judgement holds until the next
  hello ... The client keeps, for each declared owner, its judgement and
  the range the hello named of it; of an owner it does not declare,
  nothing"; "Outside a range of a family's owner, that owner's requests
  stay kept and unsent, held for its version (section 6); its reads are
  answered at once, unsent (section 7); its watches end, none opened
  again at that hello; each as version not served. Outside jig's, the
  link is out of date, and its reads are answered so too."
- A read of an owner not served (client.md, 7): "answered at once and
  never sent: version not served, naming the owner and the range the
  last hello named of it, or none where it named none".
- A kept family request (client.md, 11): "one whose owner the build does
  not declare, or whose version is not the one declared (8.2), is such
  a record", dropped at start, and the application told.
- Families as bytes (client.md, 4; root.md, 7): "The client never reads
  a family's bytes. It reads their owner, their version and their size".
- The client's world: "the engine, scripted; the transport, played by
  the world" (testing.md, 2 and 4).

## Choices this plan makes

- **The domain and the link keep their vocabularies;** `jig-client`
  maps the domain's `Down` onto the link's events and the link's
  requests onto the domain's `Up` (jig 11's choice). The map is
  exhaustive both ways: a variant added on either side breaks the build
  (development.md, 5.3).
- **The link's limits derive from the client's.** The link's
  `Limits.client` becomes `jig_client_domain::Limits`, which gains the
  decoding bounds the link reads today from the page's limits (`rows`,
  `text`, `projects`, `charters`, `fields`, `heartbeat`), and those it
  reads today from `Limits.client` (`delegates`,
  `crates/jig-client-protocol-link/src/limits.rs:94`; `records`, `kept`
  and `parts`, `src/store.rs:36`); its users build them from the same
  values.
- **The small store's barrier is the client's, for keyed requests
  alone** (findings/wire-and-client.md, 5j): reads and watches leave at
  once; a step's records go in one `Save` or one `Erase`; `NotStored`
  of a `Save` answers each keyed request whose record it was the last
  to carry `NotKept`, by its key, parked or not (skein-01.s3's
  `RequestTable::not_kept`).
- **The thin story** lives in `tests/client-protocol` (findings 6:
  `src/engine.rs` runs the real core under the clients' component over
  HTTP bytes): a scripted application in front of `jig_client::Client`,
  over the native shell's `Connections` and the world's `Net`. Its
  request is `Stop` of task 9 in project 1, which does not exist: the
  engine refuses it, durably, and no shape of work is named (findings
  6), so jig 14 does not rewrite it.
- **`me` is still `Subject::Frame`** (jig 14.1 and 22.1 reshape it).
- **`jig-client` re-exports `jig-client-domain`'s types,** the client's
  surface (client.md, 3), so an application, and the reference UI, depend
  on `jig-client` alone (view.md, 2); code-organization.md, 2's "no
  re-export" forbids only aliases kept for old paths.
- **The typed transport** (`Client::unlinked`) carries the domain's `Down`
  and `Up` instead of the link's bytes, for the client's world, whose
  scripted engine speaks the contract's types (testing.md, 4); no shell
  uses it (Questions).

## Rules that bind it

- **The meeting point** (README, Rules): each increment's Proof runs
  against the real engine in `tests/client-protocol`, never only against
  `jig-client-scripted-engine`; the story grows with each increment.
- **No HTTP and no page in `jig-client`** (code-organization.md, 6):
  12.1 adds the boundary test
  `client_crate_links_only_its_domain_link_and_foundation` (below).
- **The credential is never in the small store** (client.md, 12): the
  link's `Kept` records hold none; 12.3's test
  `a_held_credential_never_reaches_the_small_store` checks it.

## Increments

### 12.1 `jig_client::Client`: the domain and the link composed; the link's states; the story begins
Scope: the new crate, its surface (client.md, 4; sketch), the
composition, and the link's states signed out, starting, live, offline
and out of date (8.2), with `Told::Link` and `link()`; the client's own
`me` watch opened when a sign-in is live (moved from jig 11.2's
`Start`), its party and retention given to the table; the thin story's
first part against the real engine: signed in by the printed link, `me`
live, the project `me` names read.
Needs: jig 10's link vocabulary and jig 11's domain on `main` (the
session's start).
Files: new `crates/jig-client/{Cargo.toml, src/lib.rs, src/client.rs,
src/limits.rs, src/tests.rs, src/tests/fixtures.rs}`; workspace
`Cargo.toml` and `Cargo.lock` (shared);
`crates/jig-client-domain/src/{boundary.rs, domain.rs, link.rs (new),
watches.rs, limits.rs}`; `crates/jig-client-protocol-link/src/limits.rs`
(`Limits.client`) and its users: `tests/client-protocol/src/fixtures.rs:103-104`,
the link's `src/tests/fixtures.rs:7-35` (`CLIENT`) and `:38` (`limits()`),
`crates/jig-protocol-engine-clients/src/tests/fixtures.rs:740-770`
(`client_limits()`, a literal of the old domain's `Limits` for the
link), and downstream its terminal's `limits.rs` and `client.rs:88-100`
(it copies the link's `store.rs:36` budget, `records + kept + parts`
and `document_bytes`, from `bounds.client.jig`: it moves with
`Limits.client`); `crates/jig-client-protocol-link/src/{boundary.rs, store.rs}`
(the small store's names, below) and their users: the native shell's
`shell.rs:78` (`loaded`), `shell.rs:106-107, 239`,
`connections.rs:112-113` and `files.rs` (`loaded`); the link's
`src/component.rs`, `src/tests/fixtures.rs:105, 139, 171`,
`src/tests/fixtures/link_{deliveries,reads,store,stream}.rs` and
`src/tests/store.rs:61, 71, 112`;
`tests/client-protocol/src/{front.rs, front/persistence.rs:37, 43, 87,
native.rs}`, the native shell's `src/tests.rs:174, 188, 198-199`; downstream, its
terminal's `client.rs:328`, `route.rs:52-53` and `tests.rs:53-82`; `tests/boundary/src/tests/graph.rs` (`client_step` gains
`jig-client`; the new test below) (shared);
`tests/client-protocol/{Cargo.toml, src/lib.rs, src/client.rs (new),
tests/client.rs (new)}`.
Shape: `jig_client::{Client, Event, Request, LinkView, Limits, Unusable,
max_out, worst_case}` as the sketch (`Event::Up` and
`Request::Down` come with 12.5); `jig_client` re-exports the client's
surface it names: `jig_client_domain`'s `contract`, `Asker`, `Handle`,
`Key`, `Held`, `Ask`, `Family`, `Readable`, `Watchable`, `Readout`,
`Answered`, `Outcome`, `Pending`, `Progress`, `State`, `Latest`,
`Document`, `Why`, `Told`, `Link`, `Down`, `Up`, `Retry`, `Backoff`, its
`Limits` as `DomainLimits` (the type of `jig_client::Limits.domain`), and
the link's `With`,
`Message`, `Streamed` and `Record`; `jig_client_domain::Link { SignedOut,
Starting, Live, Offline, OutOfDate }`, `Told::Link { link }`,
`Up::{SignedIn, Served { served: bool }}`. Starting on the link's
`Signing::SignedIn` or a `Held` credential; live at `me`'s delivery;
offline when `me`'s watch drops (the link drops the stream when it ends
or is silent past twice its heartbeat, protocol/client.md, 4); out of
date at a hello whose jig range the link judges not served (jig 11.1's
`serves`), after which nothing is sent and the table stays parked. The
table is up while live, confirmed to `me`'s party, with `me`'s
retention. The link's small-store names become 4.3's: `Write { key,
record }`, `Remove { key }`, `Written`, `WriteFailed` and `StoreRead {
key, record }` are `Save { record: Record }`, `Erase { key }`, `Stored`,
`NotStored` and `Record { key, bytes }` (one type, both ways); the
client gathers a step's `Save`s and `Erase`s into one `Request::Save {
records }` and one `Request::Erase { records }`. The barrier is the
link's, unchanged, until 12.2.
Test shape: unit tests in `crates/jig-client/src/tests.rs` on
`src/tests/fixtures.rs` (made here; limits copied, as literals, from
the client world's profile, as jig 11's fixtures copy them:
`jig_client_world::test::limits()`, tests/client/src/test.rs:19, each
with a comment naming that line; no dev-dependency on
`jig-client-world`, which depends on `jig-client` from 12.5: a cycle),
for the composition: `step` routes each
`Event` once and each request once; reference test:
```rust
#[test]
fn a_watch_asked_of_the_client_leaves_as_one_opening_on_the_stream() {
    let mut c = fixtures::Client::signed_in(PARTY);
    // `signed_in` has opened `me`, and with it the stream (the link's component.rs:318-322, and `send_open` at :330-355):
    // a later watch leaves on the named stream as one exchange.
    c.step(Event::Watch { handle: TASK, subject: task(9) });
    let [Request::Exchange { credential: Some(_), message: Message::Watch(_), .. }] = c.requests().as_slice() else {
        panic!("one watch exchange on the named stream: {:?}", c.requests());
    };
}
```
The story: `tests/client-protocol/src/client.rs`, `pub struct Story`
holding a `jig_client::Client`, a `jig_client_shell_native::Connections`
(fixtures.rs:94's limits), a `jig_client_protocol_http::Mapping` (`/jig`)
and `Stream`, the world's `Engine` (src/engine.rs:27, `Engine::bounded`
with `fixtures::owner_limits()`, fixtures.rs:17) and `Net` (src/net.rs:52),
the told requests in order, and the small store as the host keeps it;
its loop follows `Protocol::advance` (src/front.rs) with one client.
Methods: `Story::new(seed)`, `sign_in()` (the engine's printed link, read
with `jig_client_protocol_http::printed`), `watch(handle, subject)`,
`read(handle, read)`, `send(key, request)`, `run_until(done, within) ->
bool`, `told_links() -> Vec<Link>`, `answered(key) -> Vec<Outcome>`,
`made(party) -> usize` (the engine's `Observed::Made` for that party),
`reload()` (12.2), `cut_answer()` (`Net::cut_answer`, net.rs:92).
Reference test, `tests/client-protocol/tests/client.rs`:
```rust
#[test]
fn a_client_signed_in_by_the_printed_link_sees_me_live() {
    let mut story = Story::new(fixtures::SEED);
    story.sign_in();
    assert!(story.run_until(|s| s.client.link().state == Link::Live, Duration::from_secs(1)));
    assert_eq!(story.told_links(), vec![Link::Starting, Link::Live]);
    assert_eq!(story.client.link().party, Some(fixtures::FIRST_OWNER));
}
```
Tests: `a_client_signed_in_by_the_printed_link_sees_me_live`,
`a_client_reads_the_project_me_names_by_its_handle`,
`a_dropped_stream_makes_each_handle_behind_then_live_with_me` (story:
the world's `Net::cut_stream`, net.rs:104; each handle's state behind
and the link offline, then each live once `me` returns),
`each_delivery_the_client_took_reaches_the_core_as_taken` (testing.md,
6.1, "A delivery taken is told": at quiescence the engine's core holds
as taken each delivery the client took, whatever ended its watch); unit:
`a_watch_asked_of_the_client_leaves_as_one_opening_on_the_stream`,
`a_reply_reaches_the_domain_as_its_decoded_answer`,
`an_application_watch_on_me_shares_the_client_s_own`,
`an_engine_not_serving_jig_s_version_leaves_the_link_out_of_date`,
`a_request_parked_offline_is_sent_once_me_returns` (client.md, 8.2),
`a_read_whose_exchange_is_lost_is_answered_unreachable` (client.md, 7).
The story keeps a referee (`tests/client-protocol/src/client.rs`'s
`Referee`), checked at each step, which this increment starts with one
rule of testing.md, 6.1: a document told `Changed` for a handle never
carries a lower revision than one told before for its subject; its
referee test `a_document_older_than_one_told_is_caught`, in
`tests/client-protocol/tests/client.rs` beside the stories, feeds it two
documents out of order. Each story under 0.5 s, each unit test under
20 ms.
Proof: joint layer, the real engine over HTTP bytes: `cargo
nextest run -p jig-client-protocol-world -E
'test(a_client_signed_in_by_the_printed_link_sees_me_live)'`, once; it
passes when the client's link goes starting then live and names party 1
(under 0.5 s). It passes; nothing else is wanted.
Don't: no HTTP, no page, no family's bytes read; no change to the
reference UI or the roots (jig 13 and jig 21).
Size: at most 300 lines (the crate about 180, the link's states about
80, the limits about 40).
Removes: nothing.

The boundary test 12.1 adds, in `tests/boundary/src/tests/graph.rs`:
```rust
#[test]
fn client_crate_links_only_its_domain_link_and_foundation() {
    let clients: Vec<_> = manifests().into_iter().filter(|m| m.name.as_deref() == Some("jig-client")).collect();
    assert_eq!(clients.len(), 1, "one client");
    let mut dependencies = clients[0].dependencies.clone();
    dependencies.sort();
    assert_eq!(dependencies, ["jig-client-domain", "jig-client-protocol-link", "skein-lib"]);
}
```

### 12.2 Keyed requests end to end: kept before sent, answered once durable, kept across a reload
Scope: the client's barrier (client.md, 11): a step's records in one
`Save`; a keyed request's first `Exchange` only after `Stored`; reads
and watches never wait; `NotStored` of a `Save` answers each keyed
request whose record it was the last to carry `Outcome::NotKept`, by its
key, parked or not (client.md, 11; skein's `lib.md`, 12, not kept);
`Start { kept }` restores the table's
records, each told `Kept { key, request }`. The story grows: a `Stop`
answered once after its commit is durable, and kept across a reload.
Needs: 12.1's client; skein-01.s3's landing (`RequestTable::not_kept`).
Files: `Cargo.lock` and `tests/client-protocol/src/native/world.rs`
(the skein move, a commit of its own, first; README, "Pins");
`crates/jig-client/src/{client.rs, store.rs (new)}`;
`crates/jig-client-domain/src/{boundary.rs, requests.rs}` (`NotKept`;
`Up::NotKept { key }`);
`crates/jig-client-protocol-link/src/{boundary.rs, store.rs}` (the
link's barrier holds `Message::Request` exchanges alone, findings 5j;
`Request::NotKept { attempt }` for each held keyed attempt on
`NotStored`) and the users of its `Request::NotKept` and `Answer.why`:
the link's `src/component.rs:197, 211, 268` and the exhaustive matches
of `src/tests/fixtures.rs:96-107, 130-141, 161-173`,
the native shell's exhaustive matches at `connections.rs:112-124` and
`shell.rs:113-125`, and downstream its terminal's `route.rs:50-134`
(`Answer` destructured at :84), `src/tests/family.rs:174` and
`src/tests/fixtures.rs:77`; `tests/client-protocol/src/front.rs` (the old roots' map
of `NotKept`); `tests/client-protocol/src/client.rs` (the story's
`reload()`, `restart_engine(offline)` as front/faults.rs:35-42 restarts
it, `refuse_next_save()`, a second client of the same party, and the
referee's rules below).
Shape: `jig_client_domain::Outcome::NotKept`; the link's
`Request::NotKept { attempt }`; the link's `Request::Answer` gains `why:
Option<Retry>` (jig 11.3's), telling busy from not ready, and a final
answer that does not decode is `Lost` with `Retry::Undecodable`, beside
its fact (protocol/client.md, 5: "lost, and reported through the
request's progress as retrying because its answer did not decode"), so
the application is told `Pending` and reads `Retrying { why:
Undecodable }`; the link's `Request::Dropped` reaches
the application as `Told::Dropped` (jig 11.3). The old
roots' path, through the link outside them until jig 13.4 (the test
root) and jig 21.2 (`ops`'s), maps the
link's `NotKept { attempt }` to its table's final answer with none (its
`Fact::InvalidAnswer`), as the link's `WriteFailed` has no other user.
A request not kept (the design answer to jig 12.2's question of
2026-10-16T19:46:42Z; client.md, 11; skein's `lib.md`, 12):
- The skein move is the first commit, on its own (README, "Pins"):
  `cargo update -p skein-lib --precise <skein-01.s3's landing>`,
  with its one repair, `lines: 0, line_bytes: 0` in the
  `skein_ui_text::Settings` literal at
  `tests/client-protocol/src/native/world.rs:232` (jig `bb977c909`).
- `jig-client`'s `store.rs` numbers the writes it gives, `Save` and
  `Erase` alike, and counts their answers: each answer is for the
  oldest write not yet answered (client.md, 16). It keeps a map from
  each table key whose record a `Save` carried (`Down::Save`'s
  `record.key`) to the number of the last such write. A key leaves the
  map when that write is answered, or when its `Down::Erase` passes, so
  the map holds at most `requests.requests` keys, priced in
  `worst_case`. A request's record is whole in each `Save`, so its last
  write alone decides whether it is kept.
- On `NotStored` of write `w`, before the link takes it, the client
  gives the domain `Up::NotKept { key }` for each key whose last write
  is `w`, and drops them from the map; on `Stored` of `w`, it drops
  them only. The domain calls `table.not_kept(key)`: for `Some(owner)`
  it removes `owner` from its requests and tells `Answered { key:
  owner, outcome: NotKept }`; for `None` (answered, unknown or not
  kept already), nothing. An `Erase` is no key's last write, so its
  `NotStored` answers nothing.
- Then the link takes `NotStored`. Each `Request::NotKept { attempt }`
  it gives reaches the domain as `Up::Answer { attempt, envelope:
  RequestEnvelope::Lost, answer: None, why: None }`. That attempt is
  stale for a request retired just before. It is a retry under its key
  for a request whose record a later write carries, which leaves once
  that write is stored.
- Don't: answer a request not kept by its attempt, or by a write that
  is not its last.
Test shape: unit tests in `crates/jig-client/src/tests/store.rs`
(fixtures from 12.1); story tests in `tests/client-protocol/tests/client.rs`.
Tests: story `a_stop_is_answered_once_after_its_commit_is_durable`
(written out below), `a_stop_kept_across_a_reload_is_answered_once`
(the answer cut with `cut_answer`, the client reloaded from the story's
small store before the retry: `Kept` told for its key, the engine's
`made(1)` 1 at the end),
`a_stop_whose_record_is_refused_is_answered_not_kept_and_never_made`
(testing.md, 7: the story's store refuses the save),
`the_engine_restarting_under_a_pending_stop_answers_it_once`,
`two_clients_stopping_one_task_are_answered_once_each` (a fake party of
the engine's world, party 1, starts a chat, as stories.rs:67's
`pressure()` scripts a party, and two clients of party 1 each send
`Stop` of its task under their own keys: each key is made once, and the
second is answered as the first decided, contract.md, 4); unit
`a_keyed_request_waits_for_its_record_to_be_stored`,
`a_read_leaves_while_a_record_is_being_stored`,
`a_record_not_stored_answers_its_request_not_kept`,
`a_parked_request_whose_record_is_not_stored_is_answered_not_kept`
(`me` delivered, then `StreamGone`: a `Send` leaves one `Save`, answered
`NotStored`; `Answered { outcome: NotKept }` is told once, the next
`Erase` carries its record's key, and once live again no `Exchange`
carries it),
`a_failed_write_answers_only_the_requests_it_carried` (two requests
whose records leave in two writes, the first answered `NotStored` and
the second `Stored`: the first is answered `NotKept`, the second is not,
and its `Exchange` leaves once, at most `requests.first` after its
`Stored`),
`a_step_s_records_leave_in_one_save`,
`a_request_lost_while_live_is_sent_again_and_the_link_stays_live`
(client.md, 8.2), `a_kept_record_of_another_version_is_dropped_and_told`
(client.md, 11),
`a_final_answer_that_does_not_decode_is_told_retrying_as_undecodable`. The story's referee gains client.md, 17's rules: no
keyed exchange leaves before its record is stored, none for a party
`me` has not confirmed, and no queue past its limit; its referee test
`a_keyed_exchange_before_its_store_is_caught`, in
`tests/client-protocol/tests/client.rs`. Reference story:
```rust
#[test]
fn a_stop_is_answered_once_after_its_commit_is_durable() {
    let mut story = Story::live(fixtures::SEED);
    story.send(STOP, Ask::Jig(Box::new(contract::Ask::Stop { project: 1, task: 9 })));
    assert!(story.run_until(|s| !s.answered(STOP).is_empty(), Duration::from_secs(1)));
    let [Outcome::Answered(Answered::Jig(contract::Answer::Refused(_)))] = story.answered(STOP).as_slice() else {
        panic!("one refusal: {:?}", story.answered(STOP));
    };
    assert_eq!(story.made(fixtures::FIRST_OWNER), 1);
    assert!(story.told_at(STOP) >= story.durable_at(fixtures::FIRST_OWNER));
}
```
(`Story::live` is `new`, `sign_in` and `run_until` live; `STOP` is
`Key { asker: Asker(1), number: 1 }`; `told_at(key)` is the time the
story took `Answered` for the key, and `durable_at(party)` the time of
the `Observed::Durable` whose commit is that party's `Made`.) Each story
under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_stop_kept_across_a_reload_is_answered_once)'`, once; it
passes when the reloaded client is told `Kept` for its key and the
engine makes the `Stop` once (under 0.5 s). It passes; nothing else is
wanted.
Don't: no answer kept in the small store; no request sent before its
`Stored`.
Size: at most 250 lines (35 for a request not kept: the writes' numbers
and the map, `Up::NotKept`).
Removes: the link's barrier on reads and watches.

### 12.3 `SignIn { with }` and `SignOut`
Scope: `With::{Link, Service, Held, Transport}` (client.md, 12); a
service signs in with its credential as the message's credential and
holds the sign-in's credential the answer gives; under `Transport` the
client holds none and exchanges carry none; signed out (an answer's,
a read's or a watch's end, the stream ending signed out, or `SignOut`
answered) drops the credential, ends each handle `Ended { why:
SignedOut }`, parks each request, abandons each read, and tells
`Link(SignedOut)`; a sign-out that does not reach the engine keeps the
credential and the handles, `link().sign_out_lost` true; a sign-out's
answer is its sign-in's (an earlier sign-in's is dropped once a newer is
live); `me` opens on each sign-in. The story grows: signed out with a
`Stop` parked, signed in again, the `Stop` sent once.
Needs: 12.2's story.
Files: `crates/jig-client-protocol-link/src/{boundary.rs, sign_in.rs}`
(`With::{Service, Transport}`, `Message::SignInService`, a sign-in's
number); `crates/jig-client-protocol-http/src/call.rs`
(`SignInService` to `<prefix>/v1/sign-in/deployment`);
`crates/jig-client-domain/src/{boundary.rs, link.rs, domain.rs}`
(`Up::SignedOut`, the ends and parking); the link's `src/store.rs:170-172`
(an exhaustive match on `Message`) and `jig-client-protocol-http`'s
`src/tests/fixtures.rs:15-21` (every `Message`); `crates/jig-client/src/client.rs`;
`tests/client-protocol/src/client.rs`, `tests/client.rs`.
Shape: `With` gains `Service(Secret)` and `Transport`; `Message` gains
`SignInService`; `Up::SignedOut`; `LinkView.sign_out_lost`. A service's
sign-in answered not ready is sent again after a backoff drawn from the
seed ("Not ready is *again*: a service's client signs in again after
its backoff", protocol/client.md, 8); a person's sign-out answered not
ready shows as one that did not reach the engine (8).
Test shape: unit tests in `crates/jig-client/src/tests/sign_in.rs`;
story in `tests/client-protocol/tests/client.rs`. Tests: story
`a_client_signed_out_with_a_stop_parked_sends_it_once_signed_in_again`
(the engine ends the sign-in as the protocol world's
`FrontFault::SignedOut` does, src/front/faults.rs:13; the client signs
in again by a fresh printed link; `made(1)` is 1); unit
`signed_out_ends_each_handle_once_and_parks_each_request`,
`a_lost_sign_out_keeps_the_credential_and_says_so`,
`an_earlier_sign_in_s_sign_out_answer_is_dropped`,
`a_service_signs_in_with_its_credential_as_the_message_s`,
`under_transport_exchanges_carry_no_credential`,
`a_held_credential_never_reaches_the_small_store`,
`signing_out_again_after_a_lost_sign_out_is_answered_signed_out`,
`a_sign_out_answered_not_ready_did_not_reach_the_engine`,
`a_service_s_sign_in_answered_not_ready_is_sent_again_after_its_backoff`.
Each story under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_client_signed_out_with_a_stop_parked_sends_it_once_signed_in_again)'`,
once; it passes when the parked `Stop` is made once after the second
sign-in names the same party (under 0.5 s). It passes; nothing else is
wanted.
Don't: no sign-in flow beyond jig's two (client.md, 12: the
application's own end in `Held`); no `Mint` (jig 17).
Size: at most 280 lines.
Removes: nothing.

### 12.4 Versions per owner, and families as bytes under their owner
Scope: the family owners the build declares, and each hello judged
whole against them (client.md, 7, 8.2, 11 and 14; protocol/client.md,
9 and 10; the design answer to jig 12.4's question of
2026-10-16T22:51:29Z). The application declares, in
`jig_client_domain::Limits.owners`, each owner whose families it sends,
with its wire's version. Outside jig's range, out of date (12.1). A
declared owner the hello names no range of, or one without its
version, is not served until the next hello: its requests stay kept and
unsent (each attempt answered `Again` by the client without leaving,
its progress `NotServed`), its reads are answered at once
`Readout::NotServed`, its watches end `Why::NotServed` and the link
reopens none of them at that hello, and `link().not_served` lists it
from that hello on; a hello in range again sends what was kept, under
its keys. While the link is out of date, every read is answered at once
`Readout::NotServed { family: None, .. }`. A family's `Send`, `Read` or
`Watch` naming an owner not declared, or another version, `assert!`s,
as a key conflict does (`crates/jig-client-domain/src/requests.rs:37`);
a kept family request of an owner not declared, or of another version,
is dropped at `Start` and told `Told::Dropped`. Families ride the same
table, watches and reads as bytes under their owner (client.md, 4 and
6); the story grows: a family's request answered as its owner's bytes.
Needs: 12.3's client.
Files: `crates/jig-client-domain/src/{boundary.rs, limits.rs,
domain.rs, requests.rs, watches.rs, reads.rs}`;
`crates/jig-client-protocol-link/src/{boundary.rs, component.rs (the
hello, :536-562), limits.rs (`Limits::new`, :16)}`;
`crates/jig-client/src/client.rs` (the link's `Hello`s to `Up::Served`
and `Up::FamilyServed`, :322-332; `link()`, :192); the fixtures that
build `jig_client_domain::Limits`: `crates/{jig-client-domain,
jig-client-protocol-link, jig-client,
jig-protocol-engine-clients}/src/tests/fixtures.rs` and
`tests/client-protocol/src/fixtures.rs` (`owners: &[]` where no family
is sent; the family each one's tests send, at its version, where one
is); `tests/client-protocol/src/client.rs`, `tests/client.rs` (the
family's request with `src/family.rs:15`'s `request` and `:32`'s
`answer`).
Shape:
- `jig_client_domain::Limits` gains `owners: &'static [Spoken]`, with
  `pub struct Spoken { pub family: u16, pub version: u32 }` in
  `boundary.rs`. The link's `Limits::new` gives `Unusable::Limits`
  (so `Client::new` does) for more than `w::CEILINGS.hello_served - 1`
  owners (15: the wire's `Hello.served` holds 16 ranges, jig's among
  them), or one family twice. The domain's `worst_case` counts its one
  judgement per declared owner.
- The link judges each hello against `self.limits.client.owners`: its
  `Request::Hello` becomes `Hello { family: Option<u16>, versions:
  Option<VersionRange> }`, one for jig's and one per declared owner,
  `versions: None` where the hello names no range of it, none for a
  range of an owner not declared. At the hello it reopens only the
  watches whose owner's range holds their version; one it does not
  reopen keeps its subject, so the domain's `Close` ends it with no
  exchange (`close`, :377-383).
- The domain: `Up::Served { served }` (12.1) gains `versions:
  Option<VersionRange>`, jig's range from the hello, `served: false`
  when it is `None`; `Up::FamilyServed { family: u16,
  versions: Option<VersionRange> }`, served when `versions` holds the
  declared version, an owner not declared changing nothing; `Why::NotServed`;
  `Progress::NotServed`; `Readout::NotServed { family: Option<u16>,
  served: Option<VersionRange> }`, `family: None` for jig's, `served`
  the range the last hello named of that owner.
- `LinkView.not_served: &'a [u16]`, the declared owners the last hello
  did not serve, in declaration order; `&[]` before the first hello.
Test shape: unit tests in `crates/jig-client-domain/src/tests/versions.rs`
and the link's `src/tests/stream.rs`; story in
`tests/client-protocol/tests/client.rs`. Tests: story
`a_family_request_is_answered_as_its_owner_s_bytes`,
`an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing`
(the protocol world's `FrontFault::Versions` serving, front/faults.rs:43-48);
unit `a_family_owner_not_served_keeps_its_requests_unsent`,
`a_family_owner_not_served_ends_its_watches_not_served`,
`a_family_owner_served_again_sends_what_was_kept`,
`a_family_document_past_the_bytes_limit_ends_its_handle`,
`a_declared_owner_the_hello_does_not_name_is_not_served_before_any_send`
(`link().not_served` is `[19]` after a hello with jig's range alone,
nothing sent),
`a_read_of_an_owner_not_served_is_answered_at_once_naming_its_range`
(no `Down::Read`; the page names family 19 and the hello's range),
`a_read_while_out_of_date_is_answered_not_served`,
`a_kept_family_request_of_another_version_is_dropped_and_told`; the
link's `the_hello_s_jig_range_reaches_the_domain` and
`the_hello_s_family_range_reaches_its_owner` take `Some(range)`, and
`a_hello_reopens_no_watch_of_an_owner_it_does_not_serve` is added. The fuzzy
suite gains `tests/client-protocol/tests/fuzzy_client.rs`'s
`random_scripted_applications_keep_the_client_s_rules`: a scripted
application drawing its events (watches, reads, sends, sign-outs and
sign-ins, reloads) from the seed against the real engine, the story's
referee in the loop (testing.md, 4: "A random one, for the fuzzy suite,
draws its events from the seed"), about 5 s over its seeds. Each story
under 0.5 s, each unit test under 20 ms.
Proof: joint layer: `cargo nextest run -p jig-client-protocol-world
-E 'test(a_family_request_is_answered_as_its_owner_s_bytes)'`, once; it
passes when the test connector's outcome comes back as the family's
bytes (under 0.5 s). It passes; nothing else is wanted.
Don't: no family's bytes decoded in the client (the story decodes them
with the test application's wire); no typed version refusal per message
(jig 14's); no range kept of an owner not declared; no declaration read
from the messages (each message's version is checked against it, never
learnt from it); no change to `Client::new`'s arguments.
Size: at most 210 lines (150 before the design answer of
2026-10-16T22:51:29Z; the declaration, its bound, the link's judgement
and reopen, the reads at once and the kept records' drop add 60).
Removes: nothing.

### 12.5 A typed transport for the client's world
Scope: `Client::unlinked(limits, seed)`: the domain with no link, its
`Down` given out as `Request::Down` and its `Up` taken as `Event::Up`,
for a transport that carries them to an engine scripted in the
contract's types (testing.md, 4; the client's world's scripted engine).
Sign-in under it is `With::Transport`; the small store's records are the
domain's (`RequestRecord<Held>`), kept by the world. The world's
transport plays the hello as the link gives it (12.4): `Up::Served`,
then one `Up::FamilyServed` per owner its client's limits declare;
`owners: &[]` where the world sends no family.
Needs: 12.4's client.
Files: `crates/jig-client/src/{client.rs, lib.rs}`,
`src/tests/unlinked.rs`; `tests/client/Cargo.toml` (`jig-client-world`
gains `jig-client`, and `"jig-client"` joins the allow list of
`tests/boundary/src/tests/graph.rs:297-341`,
`client_world_joins_only_the_client_and_test_roles`); `tests/client/src/joint/front.rs` (its front
carries an unlinked client's `Down` and `Up` through its translation to the
real core, `tests/client/src/joint/transport.rs`), `tests/client/tests/`
(the joint story below).
Shape: `Client::unlinked`, `Event::Up { up }`, `Request::Down { down }`,
and `Event::StartUnlinked { kept: List<RequestRecord<Held>> }` (ruling
187): the unlinked host's start, before any other event, as `Start` is
the linked host's; the client gives the domain `Event::Start { kept }`
as the host gave it; its records leave in `Down::{Save, Erase}`, as
on main.
Under `unlinked`, `Event::{Start, Reply, Lost, Streamed, StreamGone}`
are dropped, and under `new`, `Event::{StartUnlinked, Up}` are; each is
a fact the client's host made a mistake of, and `assert!`s it
(`debug_assert!` is banned in step crates, `clippy.toml:87`).
Test shape: `crates/jig-client/src/tests/unlinked.rs`: tests
`an_unlinked_client_gives_its_watch_as_the_domain_s_opening`,
`an_unlinked_client_takes_an_answer_as_the_domain_s`,
`an_unlinked_client_s_records_are_the_table_s`. Each under 20 ms. The joint
story `an_unlinked_client_sees_me_live_against_the_real_core` (tests/client,
`jig-client-world`, the joint world: a scripted application in front of
`Client::unlinked`, its `Down` and `Up` carried by the joint front to the
real core), under 0.5 s.
Proof: joint layer, the real core: `cargo nextest run -p
jig-client-world -E 'test(an_unlinked_client_sees_me_live_against_the_real_core)'`,
once; it passes when the unlinked client's link goes live with party 1;
nothing else is wanted. jig 13.4's test root then runs the client's
world on it (`signing_in_and_starting_a_chat`, tests/client).
Don't: no typed path in any shell; no `Down` or `Up` under `new`.
Size: at most 110 lines (ruling 187).
Removes: nothing.

## Interface sketches

Checked at the base with `cargo clippy -p jig-w6-sketch
--all-targets -- -D warnings` in a scratch crate (session 10's link
types and session 11's domain types stubbed beside it), passing. The
client owns the domain, the link and their map; the application owns
handles' and keys' numbers and the host's store; the transport owns
deadlines.

```rust
// crates/jig-client/src/lib.rs. The client's surface, re-exported (client.md, 3):
// pub use jig_client_domain::{contract, Answered, Ask, Asker, Backoff, Document, Down, Family,
//     Handle, Held, Key, Latest, Link, Outcome, Pending, Progress, Readable, Readout, Retry,
//     State, Told, Up, Watchable, Why};
// pub use jig_client_domain::Limits as DomainLimits;
// pub use jig_client_protocol_link::{Message, Record, Streamed, With};
// `Record` is the link's (12.1: `StoreRead` renamed):
// pub struct Record { pub key: Box<[u8]>, pub bytes: Box<[u8]> }, the keeper-tagged
// storage key and the `Kept` record's bytes, as the host keeps them.
/// What the client hears (client.md, 4.1).
pub enum Event {
    /// The small store's records, as the host loaded them; before any other event.
    Start { kept: List<Record> },
    SignIn { with: With },
    SignOut,
    Watch { handle: Handle, subject: Watchable },
    Unwatch { handle: Handle },
    Read { handle: Handle, read: Readable },
    Send { key: Key, request: Ask },
    /// The transport's (protocol/client.md, 4.3).
    Reply { exchange: Token, message: Box<[u8]> },
    Lost { exchange: Token },
    Streamed { message: Streamed },
    StreamGone,
    /// The last `Save` or `Erase` written, or not.
    Stored,
    NotStored,
    /// From a typed transport only (12.5).
    Up { up: Up },
}
/// What the client asks (client.md, 4.2).
pub enum Request {
    Exchange { exchange: Token, credential: Option<Secret>, message: Message },
    OpenStream { credential: Option<Secret> },
    CloseStream,
    /// A step's records, written before the keyed requests they keep are sent.
    Save { records: List<Record> },
    /// A step's records' keys.
    Erase { records: List<Box<[u8]>> },
    /// For a typed transport only (12.5).
    Down { down: Down },
    /// For the application, routed by the asker it names; `Link` to each child.
    Told { told: Told },
}
/// What `link()` lends (client.md, 8.2 and 12).
pub struct LinkView<'a> {
    pub state: Link,
    pub party: Option<u64>,
    pub not_served: &'a [u16],
    pub sign_out_lost: bool,
    pub credential: Option<&'a Secret>,
}
pub enum Unusable { Limits }
pub struct Limits { pub domain: DomainLimits, pub exchanges: u32 }
pub struct Client { /* domain, link (none under `unlinked`), the map's state */ }
impl Client {
    /// # Errors
    /// `Unusable::Limits` when the limits admit no client.
    pub fn new(limits: &Limits, seed: u64) -> Result<Client, Unusable>;
    /// # Errors
    /// `Unusable::Limits` when the limits admit no client.
    pub fn unlinked(limits: &Limits, seed: u64) -> Result<Client, Unusable>;
    pub fn step(&mut self, now: Time, wall: Wall, event: Event, out: &mut Queue<Request>);
    pub fn fire(&mut self, now: Time, wall: Wall, out: &mut Queue<Request>);
    pub fn next_deadline(&self) -> Option<Time>;
    pub fn reclaim(&mut self);
    pub fn document(&self, handle: Handle) -> Option<Document<'_>>;
    pub fn text(&self, handle: Handle) -> Option<&RunText>;
    pub fn pending(&self, key: Key) -> Option<Pending>;
    pub fn link(&self) -> LinkView<'_>;
}
pub fn max_out(limits: &Limits) -> u32;
pub fn worst_case(limits: &Limits) -> Option<u64>;

// crates/jig-client-domain/src/boundary.rs, what this session adds
pub enum Link { SignedOut, Starting, Live, Offline, OutOfDate }   // Told::Link { link } (12.1)
// Up gains: SignedIn, Served { served: bool } (12.1; 12.4 adds versions: Option<VersionRange>);
//           SignedOut (12.3); FamilyServed { family: u16, versions: Option<VersionRange> } (12.4)
// Outcome gains NotKept (12.2); Why and Progress gain NotServed (12.4);
// Readout gains NotServed { family: Option<u16>, served: Option<VersionRange> } (12.4)
/// A family owner the build sends, with its wire's version (client.md, 8.2) (12.4).
pub struct Spoken { pub family: u16, pub version: u32 }
// crates/jig-client-domain/src/limits.rs: Limits gains owners: &'static [Spoken] (12.4)

// crates/jig-client-protocol-link/src/boundary.rs, what this session adds
// With gains Service(Secret) and Transport (12.3); Message gains SignInService (12.3);
// Write, Remove, Written, WriteFailed and StoreRead become Save { record }, Erase { key },
// Stored, NotStored and Record { key, bytes } (12.1); Request gains NotKept { attempt } (12.2);
// Request::Hello's versions become Option<VersionRange>, one per declared owner (12.4).
```

The map in `jig-client`, written once (an algorithm easy to get wrong):
```rust
// One step: application events to the domain; transport events to the link;
// then, until both are quiet within the step's bound:
//   the link's requests: Exchange/OpenStream/CloseStream/Save/Erase -> out (keyed
//   exchanges held until Stored); Answer/Watch/FamilyDelivery/Ended/Read/Hello/Kept
//   -> domain as Up; Signing -> domain as SignedIn/SignedOut;
//   the domain's requests: Down -> link events; Told -> out.
// The bound is max_out: each pass consumes what the other produced, so the loop
// runs at most twice per message the step admitted.
```

## Questions to raise when reached

- **The client's world and the typed transport** (testing.md, 4): the
  design has the world play the transport to a scripted engine, which
  speaks the contract's types, while the client gives the transport
  bytes. The plan adds `Client::unlinked` (12.5); if the designer wants the
  scripted engine to read and write wire records instead, 12.5 goes and
  jig 13.4's and jig 21.2's client worlds change.

## Done when

- `cargo nextest run -p jig-client-protocol-world -E
  'binary(client)'` passes the fourteen tests this session names for that binary, in a worktree at
  `origin/main`.
- `cargo nextest run -p jig-boundary -E
  'test(client_crate_links_only_its_domain_link_and_foundation)'`
  passes one test.
