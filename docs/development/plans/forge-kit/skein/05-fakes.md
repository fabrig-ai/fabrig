# Session 05: the fake forge and its peer

Move the domain and protocol with their tests; host them on loopback,
sharing one forge with another fake and writing webhooks (fake-forge.md 8).

**Needs at start:** temper 01.2, the fakes using skein-forge's dialect on
temper's origin/main. Each copy is from origin/main at its step's start,
after that migration; its commit body names the source commit.
Paths and counts below are **at the base**, not instructions to copy the base.

## Read first

- skein: `docs/design/fake-forge.md` 1, 2, 4, 5, 8; `forge.md` 7, 9;
  `docs/foundation/programming-model.md` 3, 4.4, 6.3, 9, 10;
  `docs/foundation/testing-strategy.md` 4.1, 6, 8, 10.
- This plan's README: sessions, Rules and Done when; code-organization.md
  1, 3, 5–8; decisions.md 2, 3, 9, 14, 15, 18, 20, 26 and Design questions 32, 33.
- findings/survey-05-fakes.md (a–e), trace-fakes.md (1–4),
  base-runs.md and base-skein-05.md (commands, users, times and sketches).

## What the design asks

- 1, 2: independent domain, bounded store, clock and cued CI; evicting
  a commit's statuses removes its run.
- 4: Forgejo protocol, bounded setup store, quirks, byte faults, signed delivery.
- 5: shared forge, loopback, frozen observations, settlement and checked bound.
- 8: step coverage, the three counting-allocator tests, replay and peer stories.

## Choices this plan makes

Memory goes to the domain's `tests/memory.rs` (decision 9); the protocol's
one fixture goes to `tests/fixtures.rs`, included by path from `src/tests.rs`.
The peer owns `Forge`; downstream fake git keeps `Transport` and `Remote`.
No application world moves; `tests/forge/machine` is the component's.

## Rules that bind it

- **No rewrite of a moved part.** Paths, neutral names, forced dependency
  and lint adaptations only, each listed below. The oversized domain
  `src/tests.rs` moves intact; the new regression goes in its existing
  `src/tests/actions.rs`, not a split of the large file.
- README's fake-crate hotspot is held from 05.1 until temper 05.1 lands.
  The forge-host hotspot is held from 05.4 until temper 05.2 lands: 05.4
  copies the base host, 05.5 adds its delivery (decision 26).
- No system name in skein's code, tests or comments; fixtures use
  `kit`, `kit-admin`, `kit:bot`, `kit-api`, `kit-git`.
  No alias or re-export of a moved type. Preserve moved test names,
  budgets and times; no optimized tests or filters hiding expensive cases.
  New units <20 ms, worlds <0.5 s each/about 1 s total; report main's added tests.

## Increments

### 05.1 The domain and its memory tests

Scope: fake-forge.md 2, 8; copy the domain and prove the existing CI eviction.
Needs: nothing beyond the session's start.
Files: temper `testing/temper-fake-forge-domain/` ->
skein `testing/skein-fake-forge-domain/`; temper
`tests/fake-forge/tests/memory.rs` -> that crate's `tests/memory.rs`.
Change its `src/tests/actions.rs`; root `Cargo.toml` (explicit member,
workspace dependency), `Cargo.lock`; the crate's manifest adds dev-only
`skein-world`. Shared hotspot: both root Cargo files.
Shape: retain `Domain`, `Config`, `Limits`, `Event::Call`,
`Request::{Reply,Hook}`, `step`, `fire`, `MAX_OUT = 1`,
`next_deadline`, `worst_case` and the scenario API, field for field.
Rename crate/package/dependency names and imports, and `src/{lib,api}.rs`
docs to fake-forge.md. In `src/tests.rs`, `ai/temper` -> `ai/kit`
and all 15 label lines `b"temper"` -> `b"kit"`; change the Actions link
in `src/tests/actions.rs` too. Memory's `name`/`text` replace `vec!`
with bounded `Writer` fills; `name` replaces slice/copy_from_slice with
checked lengths and `Writer::put`. `names`/`tree` replace map/collect
with bounded `List` loops; `fill`'s collected dependencies use capacity 2,
and the filled-forge test's `Vec::new` uses `List` of LIMITS.repositories.
Add `#![cfg(test)]` to the moved `tests/memory.rs`; explicitly type the
numeric ranges in `Measured::call` and `fill` as `u32` to satisfy numeric-fallback
lints. Use checked arithmetic/access; keep algorithms, allocator, limits, assertions.
No downstream production dependency, test lint exception or lint relaxation.
Test shape: 96 unit tests plus 3 memory tests move with their names:
`a_forge_filled_to_its_limits_stays_within_its_worst_case`,
`full_merge_history_and_both_parent_traversals_stay_bounded`,
`many_check_contexts_and_small_trees_keep_the_temporary_status_array_bounded`.
Retain `ci_follows_content_where_a_cue_is_configured` (answer 33), and
memory's `LIMITS`, `CONFIG`, `heap::Counting`, `Meter`
and its worst-case checks. Their bounds are deliberately saturated fixture
limits, not an application's profile. Add one unit in `src/tests/actions.rs`,
reusing `src/tests.rs::{Harness,CALM,LIMITS,FIRST,ENGINE,PERSON,read}`:
```rust
#[test]
fn forgotten_statuses_remove_the_commits_run() {
    let mut h = Harness::new(Config { limits: Limits { statuses: 1, ..LIMITS }, ..CALM });
    let one = h.commit(FIRST, &[(b"src", b"one")]);
    let two = h.commit(one, &[(b"src", b"two")]);
    h.push(ENGINE, b"work", one).expect("first head");
    h.push(ENGINE, b"work", two).expect("replacement head");
    assert_eq!(
        h.ok(PERSON, read(Read::Runs { commit: one, number: 1 })),
        Answer::Runs(Box::new([])),
        "the displaced commit's run was forgotten"
    );
}
```
One status row instead of LIMITS.statuses = 8 forces eviction; ci::room stays.
The moved `every_call_is_answered_once_under_every_fault`: 0.051 s, 32 seeded
sweeps, each up to 64 sending rounds and 4096 draining rounds. Retain
it with its cost recorded, not hidden by a filter.
Proof: world `cargo nextest run -p skein-fake-forge-domain`: 100 pass
(97 units, 3 memory); base forms are the domain's 96 in 0.081 s and the
memory package's 3 in 0.005 s; after the move under 0.5 s overall.
Memory is the highest caller until 05.2 connects protocol; 05.4 adds the peer.
Don't: move the component world; change CI behaviour or split `src/tests.rs`.
Size: about 5,500 production lines moved, 150 new; 3,244 test lines excluded.
Removes: nothing in temper; temper 05.1 removes its copies.

### 05.2 The protocol layer and its own fixtures

Scope: fake-forge.md 4, 8; move the protocol, test its existing bound refusal.
Needs: 05.1's domain on skein's origin/main.
Files: temper `testing/temper-fake-forge-protocol/` ->
skein `testing/skein-fake-forge-protocol/`; its
`src/tests/fixtures.rs` -> `tests/fixtures.rs` (not a second fixture).
Copy `tests/engine/forge/src/world.rs::fake_config` into that fixture;
add the refusal regression in `src/tests/deliveries.rs`.
Change `src/tests.rs`'s fixture path; root `Cargo.toml`, `Cargo.lock`.
Shape: `Server`, `Service`, `Limits`, `Config`, `Repository`,
`Faults`, `setup::Store`, `delivery::{Endpoint,Post,sign}`,
`worst_case` and their fields/signatures move unchanged.
Rename domain imports; the dialect already names skein-forge after temper
01.2. Drop the dev-dependency on `temper-engine-forge-world`; copy
`fake_config() -> domain::Config`'s numbers and call it locally.
Use `#[path = "../tests/fixtures.rs"] mod fixtures;` in `src/tests.rs`;
set `autotests = false` so Cargo does not build the helper as another crate.
Neutralize setup logins, markers, tokens, paths, JSON expectations and docs.
No decoder, table, status or wire-order change.
Test shape: all 55 tests from nextest list move, including the three setup
tests and all twelve delivery tests. Keep `Harness::{new,hooked,ask,ask_as}`,
`config`, `hooked_config`, `limits`, `boxed`, `FIXTURE_SECRET`
and `TOKEN` in that one fixture; all its capacities remain the base's.
Add `service_refuses_limits_without_a_worst_case` (<20 ms): fixture
`config()` and `limits()` with `calls = 0` instead of 4 yield
`worst_case(&limits, &config) == None` and
`Service::new(config, &limits, 3).err() == Some(crate::Error::Limits)`.
The base already refuses this (answer 33); no production change.
Proof: joint `cargo nextest run -p skein-fake-forge-protocol`: 56 pass;
base 0.033 s (earlier baseline 0.078 s), under 0.5 s overall. It connects
HTTP, dialect and domain; a socket host is not available until 05.4.
Don't: repair a protocol difference while copying; import an application world.
Size: about 3,750 production lines moved, 80 new; 1,585 test lines move.
Removes: the copied crate's application-world dev-dependency; no temper file.

### 05.3 Setup, quirks and signatures through the layer

Scope: fake-forge.md 4, 8's missing protocol coverage.
Needs: 05.2's protocol and local fixture.
Files: `testing/skein-fake-forge-protocol/src/tests/{setup,http,deliveries}.rs`
and `tests/fixtures.rs` (extend the one fixture).
Shape: no production changes. Copy `Pump` and `deliver` from temper's
`crates/temper-forge-forgejo/src/tests/fixtures.rs` at the base (now
skein 01.2's `crates/skein-forge/src/forgejo/tests/fixtures.rs`).
Destination APIs from 01.2: `skein_forge::forgejo::client::delivery::
{decoder,first,step}`, and `forgejo::client::{decoder,first,step}` for setup.
Reuse `Harness`, `setup_harness`, `make_setup_token`, `limits`.
Test shape: eleven new units, each <20 ms; setup rows go through HTTP and
decode with the dialect, not a new JSON reader:
- `setup_routes_keep_orgs_teams_members_and_hooks`: admin makes org
  `work`, team `kit`, member id 2, hook id 1; patch/list/delete round trip
  leaves member `[2]` and hooks `[]`; active/events/url agree with input.
  Before deletion, a domain push with the hook active and Config.hooks None
  yields 0 deliveries: stored hooks route nothing.
- `user_scopes_admit_the_three_documented_spellings`: `read:user`,
  `write:user`, `all` each read user id 2 with status 200.
- `a_token_without_user_scope_is_refused`: no user scope, status 403.
- `missing_setup_objects_answer_not_found`: absent user/org/team/hook
  targets each answer 404; existing non-admin test keeps its literal 403.
- `a_full_setup_table_refuses_one_more_row`: per accounts/tokens/orgs/
  teams/hooks table, fill `limits().documents.inner = 32` through the
  fixture's scenario setters (no 32-request setup loop); the next
  creation answers 503, message `setup store is full`, count stays 32.
  The last admitted row exercises the bound; the API attempt is one past.
- `collaborators_are_listed_without_permissions`: grant two users;
  decoded ids `[1,2]`, response contains no permission field.
- `pull_by_branches_answers_the_oldest_pull`: close pull 1, open pull 2
  on the same branches; the literal branch-pair route answers number 1.
- `unrouted_paths_increment_only_the_unrouted_count`: unknown route
  answers 404; total 1, unrouted 1, domain calls 0.
- `unavailable_calls_answer_503`: domain unavailable chance 1000,
  no mutation, status 503.
- `retry_after_rounds_up_to_whole_seconds`: remaining 1501 ms yields
  `Retry-After: 2` on a 429; use the fixture clock, not wall time.
- `a_signature_holds_for_its_body_only`, in `deliveries.rs`:
```rust
#[test]
fn a_signature_holds_for_its_body_only() {
    let post = delivered(Change::Push);
    let secret = Secret::new(boxed(fixtures::FIXTURE_SECRET));
    let signature = *b"59d76cdeaea4277b5fe9055ba0c6f03f8c3ce07dfc2e298df9d624cce4c99df3";
    assert_eq!(delivery::sign(&secret, &post.body), signature);
    assert_ne!(delivery::sign(&secret, b"{}"), signature);
}
```
Extend the existing delivery tests to decode their posts to literal
`Delivery` values too (repository 2, item 3, commit 7 as their hooks
specify); keep the moved names and the independent RFC 4231 vector.
Proof: joint `cargo nextest run -p skein-fake-forge-protocol`: 67 pass,
under 0.5 s overall; base 55 in 0.033 s. Highest available before 05.4.
Don't: change setup permissions, subscribe stored hooks, or add a production codec.
Size: about 60 lines moved (`Pump` and `deliver`, the base fixtures.rs
84-143), 230 new, all tests (code-organization.md 6).
Removes: nothing.

### 05.4 The forge peer and the shared handle

Scope: fake-forge.md 5, 8; hosted admission, sharing, capture and settlement.
Needs: 05.3's tested protocol; **temper 05.1**, its users on skein's fakes.
Files: copy temper's host, `tests/one-process/src/forge.rs` (268 lines at the base; decision
26), and `testing/temper-fake-git/src/transport.rs::Forge` into
`testing/skein-fake-peers/src/forge.rs`, `src/forge/{shared,observe}.rs`. Change peer
`src/lib.rs`, `Cargo.toml`, root `Cargo.lock`. New `tests/forge.rs`, `tests/forge/fixtures.rs`;
reuse `tests/support/mod.rs`. Change protocol `src/server.rs`, `src/service.rs`
(`worst_case`, snapshot accounting), `src/tests/http.rs`, `tests/fixtures.rs`.
Temper repin compatibility files: `testing/temper-fake-git/{Cargo.toml,
src/lib.rs,src/transport.rs,src/fixtures.rs}`, `tests/one-process/src/forge.rs`,
`src/stories.rs::{forge,prepare_merge_branch}` and `src/referee/forge.rs::green`
(the latter paths under `tests/one-process`).
Shape: copy the host's admission and the shared handle; the base host delivers nothing, and 05.5
adds delivery. `Forge` becomes the shared type; pending becomes bounded Queue, reserved before
stepping; `next_reply` owns the disjoint external token range. `Forge.hooks` starts None, and
05.5 makes it owner-settable. Peer constructs Service with Config.hooks None; direct users keep
that constructor. Follow fake-forge.md 5's clock/fire/reclaim/borrow rules; Service::target
routes pending. Late replies consume snapshots after close; unmatched replies stay. Reuse
Face/plaintext Wire for admission/inbound traffic only; both are server-side. Shutdown ends
admission, settles accepted work then closes; deadline=min(io,domain). Capture in
server::entrance before 401; keep method/target/version/framing, empty all case-insensitive
Authorization values in the snapshot only; authenticate the original, drain take_head after each
server entry point before reuse. Add per connection HEAD_BYTES = size_of::<Option<http::Call>>()
+ http.head + http.headers * size_of::<Header>(), checked in worst_case; Event/MAX_UP=3 stay.
Before effects validate face/protocol/domain/config, queue>=64, pending capacity calls+hooks,
connections within Limits.connections and io.sockets. First checked sum:
face+protocol+domain+pending payloads+connections/Wire buffers+call snapshots+frozen
records/bytes; recompute each owner update, report it. Retain response::follow and its HTTP
count tests: a document-returning write reads back on the same request.
temper's side: `temper_fake_git::Forge` becomes `skein_fake_peers::forge::Forge`;
`testing/temper-fake-git/src/lib.rs` drops its export;
`testing/temper-fake-git/src/fixtures.rs`'s `Forge::new(domain, CONFIG)` imports the new type.
In `src/transport.rs`, Forge's field/new and Transport::call's serial/token/pending operations
use Queue reservation/push and next_reply; the host's Shared/pending drain use Forge/Queue. Next
temper repin repairs all; temper 05.2 deletes the copies. Add `skein-fake-peers`
to `testing/temper-fake-git/Cargo.toml`. The one-process `stories::{forge,
prepare_merge_branch}` and `referee::forge::green` follow the new Forge type and
constructor; `prepare_merge_branch` uses `Forge::next_reply` and reserves pending
Queue room before stepping/firing. Keep story inputs and assertions. Refresh
further copies and derived values reached by these repairs and list further
users in the repin commit body.
Test shape: keep 8 peer tests (0.046 s); the base host has no test of its own. New
`tests/forge/fixtures.rs` copies `tests/llm.rs::limits`' values (2 connections, queue 64), not
its private helper; seed 23, skein_sim::Config::calm, 05.2's config, checked by Peer::new; Sim
clock, no sleep.
Protocol unit `unauthorized_heads_are_drained_and_counted_in_the_bound` in `src/tests/http.rs`:
Harness/config/limits; 401 -> take_head Some(sanitized b"/api/v1/user"), then None; next
captures b"/api/v1/repos/org/repo/issues". BASE_BYTES=4_645_153, HEAD_BYTES=4_664; new
sum=4_663_809; http.head+1 -> 4_663_949; <20 ms.
Four peer tests, on Sim driving peer and client, at most 1000 passes; new helpers are
destinations:
- `forge_peer_refuses_unusable_configuration_before_effects`: non-loopback listen, queue 63,
  None worst_case/incompatible capacities -> Error::Limits.
- `forge_peer_publishes_its_bound_loopback_address`: None before Listening, loopback/nonzero
  port after port-0 admission.
- `a_second_fake_shares_names_replies_and_time`: tiny git caller's pushed commit equals HTTP
  branch's; pending empty, one reply each, same last now.
- `shutdown_keeps_a_mid_request_client_until_its_terminal`: call ends once, admission closes,
  then all hosts empty and 0 open descriptors.
Proof: joint `cargo nextest run -p skein-fake-peers`: 12 pass (8 at the base, 0.046 s);
the new forge tests <0.5 s together, each <0.5 s; moved tests keep their times.
Proof: joint `cargo nextest run -p skein-fake-forge-protocol`: 68 pass, <0.5 s overall;
covers capture below the peer.
Don't: move fake git's parsing/Remote, thread-local factory, application stories or credentials;
deliver a hook; enable TLS, subscribe setup hooks or change protocol constants.
Size: about 280 lines moved (the host and the handle), 220 new, forced adaptations and tests
counted (estimated: the base host's 520 new of findings/base-skein-05.md, split with 05.5).
Removes: no temper file; temper 05.2 deletes host/shared Forge and repins first.

### 05.5 The peer's webhook delivery

Scope: fake-forge.md 4, 5, 8; the peer writes the protocol's signed deliveries (answers 32, 33).
Needs: 05.4.
Files: new `testing/skein-fake-peers/src/forge/delivery.rs`; change `src/forge.rs`,
`src/forge/{shared,observe}.rs`, `tests/forge.rs`, `tests/forge/fixtures.rs`; protocol
`src/service.rs`.
Shape: new, on the protocol's rendered `delivery::Post` (05.2's `src/delivery.rs`; decision 26).
`Forge.hooks` is owner-settable between passes; each pass swaps hooks in then back, never
cloning. `set_hooks` (sketch A) refuses empty host/secret with Error::Limits, leaving its
current endpoint unchanged; a rejected update leaves the peer's service without an endpoint for
that pass, so its hooks end NoAddress. Err hooks reach delivery. Outbound connect, bounded
client-write buffers and io.sockets growth are in `src/forge/delivery.rs`. Endpoint.host
(Box<[u8]>, as at the base) holds a loopback ip:port, no DNS. Absent/malformed/non-loopback ->
NoAddress, dropped and observed, 0 connects (answer 32); only listen is refused. No
room/refusal/reset drops and observes, never retries. Validation adds outbound slots within
Limits.connections and io.sockets; the checked sum adds client-write buffers and the endpoint
once. Answer 33: 05.2 tests Service's None worst_case refusal. Every review renders
pull_request_approved, verdict unchanged; others: issues, pull_request, issue_comment, push,
delete, workflow_job_success, workflow_job_failure.
temper's side: none; temper takes the peer in temper 05.2.
Test shape: fixtures add io.sockets covering outbound slots and LATE_MIN=fake_config's one
second; Sim also drives a receiving listener. Six peer tests:
- `an_endpoint_set_after_binding_delivers_the_next_hook`: None, malformed host, 192.0.2.1:7200
  and empty secret yield NoAddress, 0 connects/posts; empty host/secret fail set_hooks with
  Error::Limits. Bind port 0 after peer start, assign valid Forge.hooks: next hook Written, 1
  POST, 0 retries.
- `loopback_calls_and_deliveries_leave_frozen_observations`: issue create 201, 2 domain calls
  (write/readback), 1 POST, all 4 record kinds frozen after mutation; every Head Authorization
  value b"", no fixture TOKEN bytes.
- `late_deliveries_are_written_once`: one Written at/after LATE_MIN, 1 POST.
- `lost_hooks_make_no_delivery_attempt`: hooks_lost=1000, 0 connects/posts.
- `unwritable_deliveries_are_dropped_once`: no room/refusal/reset, one named terminal per hook,
  0 retry connects.
- `a_seed_replays_names_replies_observations_and_deliveries`: two seed-23 worlds have equal
  frozen traces, including names and wire bytes.
Their form, in `tests/forge.rs`: `fixtures::World::late_delivery(23)`, `create_issue_and_settle()`,
then `delivery_outcomes()` is `[forge::Delivery::Written]`, `received_posts()` 1 and
`first_delivery_at() >= fixtures::LATE_MIN`.
Proof: joint `cargo nextest run -p skein-fake-peers`: 18 pass; the new forge tests <1 s
together, each <0.5 s.
Proof: joint `cargo nextest run -p skein-fake-forge-protocol`: 68 pass, unchanged, <0.5 s.
Don't: retry or resend a delivery; resolve a name; enable TLS; subscribe setup hooks.
Size: about 0 lines moved, 300 new (estimated: delivery about 220, six tests and fixtures about
80; decision 26).
Removes: nothing.

## Interface sketch A: the peer boundary

Public types are defined in `forge.rs`; its submodules hold helpers/impls.
Compiled in detached skein scratches (base-skein-05.md); peer bodies are placeholders.

```rust
// skein_fake_peers::forge; use domain/protocol for skein's two fake crates.
use std::{cell::RefCell, net::SocketAddr, rc::Rc};
use skein_lib::{Queue, Time, Token};
use crate::{Error, Limits};
/// The owner's one forge; peer and second fake borrow for separate passes.
pub struct Forge {
    pub domain: domain::Domain, pub config: domain::Config, pub now: Time,
    pub pending: Queue<domain::Request>, pub hooks: Option<protocol::delivery::Endpoint>,
    serial: u64,
}
impl Forge {
    pub fn new(domain: domain::Domain, config: domain::Config) -> Self;
    pub fn next_reply(&mut self) -> Token;
}
/// Loopback host; `Host::is_empty` is its shutdown terminal.
pub struct Peer { /* private face, service, connections and observations */ }
impl Peer {
    pub fn new(address: SocketAddr, limits: Limits, config: protocol::Config,
        protocol: protocol::Limits, forge: Rc<RefCell<Forge>>, seed: u64) -> Result<Self, Error>;
    pub fn address(&self) -> Option<SocketAddr>;
    pub fn observations(&self) -> &[Observation];
    pub fn shutdown(&mut self);
}
pub fn worst_case(limits: &Limits, protocol: &protocol::Limits,
    config: &protocol::Config, forge: &Forge) -> Option<u64>;
/// Frozen records for the referee; Call holds values, never a `ReplyTo`.
pub enum Observation {
    /// Accepted or fully closed connection.
    Connection { connection: Token, open: bool },
    /// HTTP Call snapshot; every Authorization value is empty, never a token.
    Head { connection: Token, call: skein_http::server::Call },
    /// One domain call and its terminal, even if its socket has closed.
    Call { connection: Option<Token>, user: u64, repository: Box<[u8]>,
        op: domain::api::Op, result: Result<domain::api::Answer, domain::api::Error> },
    /// Rendered bytes when available, and exactly one delivery terminal.
    Delivery { post: Option<protocol::delivery::Post>, outcome: Delivery },
}
/// Best-effort write terminal, not the recipient's application verdict.
pub enum Delivery {
    /// Whole request written.
    Written,
    /// No usable endpoint, or host is not a loopback ip:port.
    NoAddress,
    /// No connection or byte room.
    NoRoom,
    /// io refused connection.
    Refused,
    /// io reset the write.
    Reset,
}
// skein_fake_forge_protocol::Server
impl Server { pub fn take_head(&mut self) -> Option<skein_http::server::Call>; }
// skein_fake_forge_protocol::Service; admitted ownership swap, no clone.
impl Service {
    pub fn set_hooks(&mut self, hooks: Option<delivery::Endpoint>)
        -> Result<Option<delivery::Endpoint>, Error>;
}
```

## Done when

Run from skein; no full gate duplicate.
- `cargo nextest run -p skein-fake-forge-domain` prints 100 passed;
  `cargo nextest run -p skein-fake-forge-domain --test memory`
  prints 3 passed, under 0.5 s overall.
- `cargo nextest run -p skein-fake-forge-protocol` prints 68 passed, <0.5 s.
- `cargo nextest run -p skein-fake-peers` prints 18 passed; new forge
  tests <1 s together, each <0.5 s; moved times retained.
- `cargo clippy -p skein-fake-forge-domain -p skein-fake-forge-protocol
  --all-targets -- -D warnings` and `cargo clippy -p skein-fake-peers
  --all-targets -- -D warnings` print Finished without warnings.
- `rg -n -i temper testing/skein-fake-forge-domain
  testing/skein-fake-forge-protocol testing/skein-fake-peers` prints
  nothing (rg exit 1 means no matches). Base forms and the expected
  source-name matches are recorded in findings/base-skein-05.md.
