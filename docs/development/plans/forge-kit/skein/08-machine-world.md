# Session 08: the machine and listener world

Move the machine/listener world into skein, then prove plaintext/TLS faults, deadlines and closing.
The complete component-memory measurement is deferred to the GitHub draft, section17 (ruling189; execution195). The client/component protocol world is skein 09. Needs at start: temper 03.1's users on the
component, temper 05.1's users on the fakes, and skein 06.2's transcripts/deliveries, all landed on
origin/main (README; decisions 22, 23).

## Read first

- skein `docs/design/forge-connection.md` 2.2, 3.1–3.3, 6–8, 10; `forge-client.md` 8, 10; `fake-forge.md` 4, 8; `forge.md`
  3.7, 5.4.
- skein `docs/foundation/programming-model.md` 4.2–4.4, 5.2–5.4, 6.3, 7, 9, 10.2; `testing-strategy.md`
  2.4–2.5, 3, 4.4, 6–8, 10.
- README, code-organization.md 1, 3, 5–8, decisions.md 11, 18, 22, 23, 25; later.md;
  findings/{survey-03-connection,survey-06-tests,trace-connection,
  trace-fakes,base-skein-03,base-skein-07,base-skein-08}.md. The last is this session's checked inventory;
  earlier drafts' 32-test partition is obsolete.

## What the design asks

- Connection 6, 8: authenticated hints after responses; each phase's deadline; Close drops outstanding work,
  then closed once and last after io settles.
- Connection7: plaintext/TLS byte faults; client/component stories and invariants over the fake's bytes.
  Connection10's complete worst-case measurement is deferred to the GitHub draft, section17,
  after its holders exist (ruling189; execution195).
- Fake 4, 8: its existing HTTP/domain boundary, observations and tally; testing-strategy 8: preserve moved
  coverage and understand slow tests.

## Choices and rules that bind it

Copy temper's origin/main at each move's start, naming the source commit. Files and counts below are **at
the base**; preserve additional tests landed before copying. No rewrite of a moved part; only the
home-forced changes listed in Shape. Count file pairs by code-organization 6. A moved file over 800 lines is
split by its next changing increment, never by the move. No `temper` in skein code, comments, fixtures or
values; use `kit`, `kit/0.0.0`, `b"kit"`; no serde, alias or re-export of a moved type. Each world is a hand-listed
Cargo.toml member. No optimized test builds, raised budgets or filters hiding moved tests. New units <20 ms,
focused world tests <0.5 s; new focused work aims at ≤1 s per world, fuzzy ≤5 s. Decision 11 preserves the
four inherited 7.481–7.709 s sweeps. All Sizes include tests (code-organization 6).

## Increments

### 08.1 The machine and listener world moved
Scope: forge-connection 6, 10's existing world, including the Joint story.
Needs: the session's start capabilities; skein 05.2's delivery is already implied by temper 05.1. No
  dependency on skein 07's client world here.
Files: at the base, temper `tests/engine/protocol-forge/src/{machine,hooks,peer,pipe,joint}.rs` -> skein
  `tests/forge/machine/src/`; all of `tests/{machine,hooks,fuzzy_machine,fuzzy_hooks}.rs` -> its tests/.
  Copy the used fixtures.rs parts and `tests/engine/forge/src/world.rs::fake_config` into its sole
  src/fixtures.rs. New Cargo.toml/lib.rs; shared hotspots: skein Cargo.toml (member/dependency) and
  Cargo.lock.
Shape: set `license.workspace = true` in the new Cargo.toml's `[package]`, beside its other
  workspace-inherited fields (README, Workspace license inheritance).
  Package `skein-forge-machine-world`; retain Machine, Listener, Joint, Peer, Pipe, Cut,
  Answer/Response/Connect, signatures and algorithms. Public modules fixtures, machine, hooks, pipe, joint;
  peer stays private. Keep `Pipe::new(seed: u64, cut: Cut) -> Self`, `fake_limits() ->
  skein_fake_forge_protocol::Limits` and `fake_config() -> skein_fake_forge_domain::Config` available to
  worlds. Joint calls the local fake_config, not 05.2's private test fixture. Copy
  limits/shared/settings/response/branch, protocol_config,
  hook_limits/hook_settings/hooked_protocol_config/push_hook and their constants; leave writer_settings,
  protocol_shared and goal_view to temper's policy. Flatten Shared using skein 03.1; Credential and
  Transport use its landed shape. fixtures::settings sets Settings.word=b"kit" and
  Settings.user_agent=b"kit/0.0.0"; remove Forge.host and use Transport::Plaintext { name: None }. Neutralize the two machine-test marker literals; body
  room follows 2*op_bytes+word.len()+15 (512, kit ->1042). hook_limits' five profile-derived values become
  plain 4/8192/1048576/10 s/128; cite the kit fixture provenance without naming a service. Neutralize fake_limits' profile comment too;
  keep its numbers. Other caps stay.
  case_bytes reads ../conformance/transcripts from CARGO_MANIFEST_DIR; hooks' recorded push includes
  ../../conformance/deliveries/push/request. Drop service, connector/change/issues, jig and client
  dependencies; api comes from skein-forge; keep the fake domain/protocol dependencies. The new crate
  boundary forces public visibility for Joint's existing component/env/above/below/fake_env/answers fields
  and busy/time/replace, plus joint::checks; types stay unchanged, busy/checks gain #[must_use]. Temper's
  side (temper 07): protocol.rs uses those fields/helpers directly from skein's Joint, with its own owner
  settings; no second Joint or alias. Other consumers: setup's exchange/world/fixtures (skein 07.3), and
  one-process/src/forge.rs's fake_limits (temper 05.2/07); keep exports.
Test shape: **38 moved tests: 33 focused, 5 fuzzy**, all names below. Their fixtures retain seeds and
  limits; no new test. machine (14):
```text
a_body_past_body_bytes_is_carried_empty_without_its_key
a_call_silent_past_its_deadline_is_a_timeout_and_aborts
a_call_waiting_past_its_deadline_is_unavailable_at_no_cost
a_connection_marked_close_finishes_its_call_and_the_next_connects
a_path_dropped_on_404_is_resolved_again_by_id
a_redirect_drops_the_path_and_answers_unavailable
a_reset_after_a_written_request_is_timeout
a_well_framed_malformed_document_is_invalid_answer
a_write_body_is_read_before_its_scripted_response
an_idle_connection_closes_and_the_next_call_connects
an_unreachable_forge_is_checked_again_after_its_backoff
another_user_stops_the_start_once_naming_the_forge
settings_answered_404_stops_the_start_with_its_status
the_start_reaches_ready_from_the_recorded_answers
```
  hooks (19), including its Joint story:
```text
a_body_past_its_cap_is_answered_413_and_hints_nothing
a_connection_past_hooks_waits_until_one_closes
a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint
a_failed_listener_is_told_to_the_root_as_lost
a_get_to_a_forges_path_is_answered_405
a_held_hint_stops_the_next_body_until_it_is_offered
a_hint_the_root_refused_is_held_and_offered_again
a_malformed_head_is_answered_4xx_and_closed
a_missing_signature_is_answered_403_without_reading_a_body
a_path_without_a_forge_is_answered_404
a_path_without_a_secret_is_answered_404
a_recorded_forgejo_push_signed_by_the_world_is_hinted
a_request_unfinished_at_its_deadline_is_closed_with_no_answer
a_signed_push_is_answered_204_before_its_hint_goes_up
a_verified_body_the_decoder_calls_malformed_is_answered_400
a_verified_event_of_no_row_is_answered_204_and_hints_nothing
a_wrong_signature_is_answered_403_and_hints_nothing
the_secret_appears_in_no_event_or_debug_text
two_deliveries_on_one_connection_are_both_answered_and_hinted
```
  fuzzy_hooks: `a_delivery_split_at_every_point_is_answered_alike` (1).
  fuzzy_machine (4), with their recorded base seconds:
  `a_response_cut_at_each_byte_ends_its_call_once`: 7.635;
  `a_response_cut_at_each_byte_ends_its_call_once_seed_1`: 7.544;
  `a_response_cut_at_each_byte_ends_its_call_once_seed_2`: 7.709;
  `a_response_cut_at_each_byte_ends_its_call_once_seed_3`: 7.481.
  Each rebuilds the component, two start reads and path resolution per recorded response cut.
  Keep that cost/budget; report it, never shorten its sweep.
Proof:
world; `cargo nextest run -p skein-forge-machine-world`: 33 passed, each <0.5 s, including signed domain push through Joint. Full source base: 47 focused at 0.404 s; separate lighter-load reruns: machine 14/0.034 s, hooks 19/0.024 s. This proves the moved harness; conformance cannot drive it.
world; `cargo nextest run -p skein-forge-machine-world --profile fuzzy`: 5 passed, each fuzzy_machine at most 7.8 s (base 7.481–7.709 s), fuzzy_hooks <0.5 s; full source base 6/7.709 s. Reruns: fuzzy_machine 4/1.746 s, fuzzy_hooks 1/0.189 s; no new ceiling.
Don't: move protocol.rs or its policy tests; copy recordings; drop the Joint story; export fake_config from
  the fake library; alter Pipe's cut schedule.
Size: about 2,000 lines moved, 120 new including tests: 60 fixture/path edits, 40 manifest/module
  lines, 20 visibility/comment edits; no shipped code (findings recount).
Removes: downstream dependencies in the skein copy; nothing in temper.

### 08.2 The machine's deadlines, faults and reuse
Scope: forge-connection 2.2, 3.3, 7.2, 8, 10's missing plaintext machine stories.
Needs: 08.1's Machine and admitted fixtures; TLS repeats these cases in 08.4.
Files: tests/forge/machine/src/{machine,peer,fixtures}.rs; tests/machine.rs and new tests/fuzzy_requests.rs.
Shape: add test-driver pauses at io/HTTP boundaries and refused/returned/offer actions; observe component
  events and requests, never production diagnostics. Keep limits/shared; Machine::with takes a second
  configured forge. Test-local rows and drivers below are new, not production APIs.
Test shape: five new focused tests (<0.5 s each) and one fuzzy (<5 s); seeds/limits from 08.1.
  deadlines_end_each_phase uses this literal table. D is admission + limits.deadline (10 s), S each start
  read's admission +10 s. At D-1 ns/S-1 ns: 0 new terminals/aborts; at the boundary:

| Phase held | Deadline | Failure kind / outcome | Call cost | New call terminals |
|---|---|---|---:|---:|
| Start Checking, Settings; call Waiting | S / D | start retries; call Unavailable | 0 | 1 |
| Call Connecting, Sending before first byte | D | Unseen / Unavailable | 0 | 1 |
| Call Sending after first byte, Reading head, Reading body | D | Seen / Timeout | 1 | 1 |

  Call rows use a first Read::Locate, so no earlier path lookup changes cost; start traffic is excluded.
  Each active exchange aborts once. Rows are separate cases, not one long test body.
```rust
// tests/forge/machine/tests/machine.rs; test-local table/driver implement the rows above.
#[test]
fn deadlines_end_each_phase() {
    for row in DEADLINES {
        let observed = drive_deadline(row);
        assert_eq!(observed, row.expected);
    }
}
```
  a_start_doubles_its_backoff_to_the_cap: refuse successive connects, observe waits [1,2,4,8,8] s,
  no connect one ns before each due time, one at it, then ready after identity/cap; stopped count0.
  an_idle_connection_is_reassigned_to_another_forge: start both forges Ready, then fill both slots idle
  for forge1 and call forge2
  (number2, distinct loopback port, admitted settings); 1 idle close, 1 replacement connect,
  1 successful terminal, cost2 (resolve+branch), never Busy. Hold other idle expiry during this case.
  a_held_answer_is_offered_again: refuse the first answer, Offer twice; deliveries [1,2,2], cost2 on both
  offers of that same answer, accept the second. a_returned_answer_is_never_offered_again: return the
  delivered answer, Offer twice; deliveries [1,1,1], closed count1 after owner Close.
  plaintext_request_cuts_end_once in fuzzy_requests.rs: seed7, EOF/reset before first write, at every
  request byte, and each response-head byte for a minimal first Read::Locate; starts exercised too.
  Request/head lengths come from the generated fixture, not a checkout path. Each case runs to its
  deadline: 1 terminal; before first write Unavailable/cost0, after first write Timeout/cost1; for a
  failed start, waiting call Unavailable/cost0. Complete control exchange succeeds/cost1.
```rust
// tests/forge/machine/tests/fuzzy_requests.rs; driver returns observed events and byte counts.
#[test]
fn plaintext_request_cuts_end_once() {
    for (written, cost, terminals, closed) in plaintext_request_cuts(7) {
        assert_eq!(cost, written);
        assert_eq!((terminals, closed), (1, 1));
    }
}
```
  The driver also checks each row's literal Error above; no expected cost comes from Answered.cost.
Proof:
world; `cargo nextest run -p skein-forge-machine-world`: 38 passed (33+5), each <0.5 s; only this world controls phase boundaries, so no higher proof can observe them.
world; `cargo nextest run -p skein-forge-machine-world --profile fuzzy`: 6 passed (5+1), new sweep ≤5 s, inherited four ≤7.8 s each and fuzzy_hooks <0.5 s; base forms in findings.
Don't: change moved sweeps, import a service profile, or claim 03.3's Harness tests are world coverage.
Size: about 0 lines moved, 300 new including tests: 100 driver/pauses, 100 focused cases/table,
  100 request/head sweep and reference; no shipped code.
Removes: nothing.

Skein08.s1 separately splits the seven existing08.2deadline cases into three uniform tests, adding two focused identities and no scenario. Its count delta applies to later runtime totals:38/6 becomes40/6, then42/7 after08.3 and43/8 after08.4. The landed08.2proof above stays historical. Preserve its driver inputs and W73's pending08.3append when rebasing the shared test file.

### 08.3 Close races and listener faults
Scope: forge-connection 6, 8.4–8.5, 10's close in every state, and the listener's byte faults.
Needs: 08.2's paused driver and refusal/return actions.
Files: tests/forge/machine/src/{machine,hooks,fixtures}.rs; tests/{machine,hooks,fuzzy_hooks}.rs.
Shape: count closed events and retain event order independently; hold io settlement until released.
  hooks::listener_faults(seed: u64) -> Vec<[u64;6]> returns counts per case: unverified hints, duplicate
  responses, duplicate hints, hints before response queueing, unsettled sockets, component Closed events.
Test shape: two new focused tests (<0.5 s each) and one fuzzy (<5 s). Reuse 08.1's Listener,
  hook_settings/hook_limits and signed delivery. deadlines_end_each_hook_phase has these rows:

| Phase | Deadline | New responses | New hints | Sockets after settlement |
|---|---|---:|---:|---:|
| Head, Body | accept/last answer + hook_deadline (10 s) | 0 | 0 | 0 |
| Answering, its hint refused and held | its 204 + hook_deadline (10 s) | 0 | 0 | 0 |
| Closing | close request + shared.io.close_timeout (1 s) | 0 | 0 | 0 |

  At deadline-1 ns, expiry count0; at deadline, expiry count1. Closing has no API cost or call terminal.
  Answering has no public moment before its 204 is queued (forge-connection 8.4): with the entry point's
  room, the step that verifies the body queues the 204 and then offers the hint. Its row first asserts
  that step, in the `send` of a signed push: responses1, hints1 and `hinted_after_answer()` (the 204
  queued before the Hint). Then `refuse()` (hints0, the hint held) and no Offer; its new
  responses and hints count from there, a zero hint baseline. At its deadline the held hint is dropped,
  never offered. Never reach a pre-204 state by starving room, a private poke or a diagnostic control.
  Use 08.2's table/driver reference shape with the literal expected columns above.
  owner_close_wins_or_follows_each_terminal covers connection Closed/Connecting/Sending/Reading/Idle,
  call Waiting/Resolving/running/answered, start Checking/Settings/Waiting/Ready/Stopped, listener
  Head/Body (Close before pending terminal), Answering (the held row: its hint refused, then Close;
  no pre-204 row), Closing and held/refused/returned answers/hints. Repeat Handshaking in 08.4.
  Cross each applicable pending call/hint/stopped terminal with Close before/after its delivery:

| Ordering at owner Close | Terminals before Close | Reoffers after Close | Closed before io settles | Closed after settlement |
|---|---:|---:|---:|---:|
| Close before pending terminal | 0 | 0 | 0 | 1 |
| Terminal delivered then Close (accepted/refused/returned) | 1 | 0 | 0 | 1 |
| Held answer/hint: initial offer refused, then Close | 1 | 0 | 0 | 1 |
| Idle, holding its socket, with no pending terminal (deltas) | 0 | 0 | 0 | 1 |
| Waiting/Stopped with no socket and no pending terminal (deltas) | 0 | 0 | 1 | 1 |

  With no sockets at Close, Closed is immediately 1; do not wait for nonexistent settlement. Every row:
  events after Closed=0, outstanding sockets=0, termination within the existing bounded driver loop.
```rust
// tests/forge/machine/tests/machine.rs; expected rows from the table.
#[test]
fn owner_close_wins_or_follows_each_terminal() {
    for row in CLOSES {
        let observed = drive_close(row);
        assert_eq!(observed, row.expected);
    }
}
// tests/forge/machine/tests/fuzzy_hooks.rs
#[test]
fn listener_faults_at_each_byte() {
    for counts in skein_forge_machine_world::hooks::listener_faults(7) {
        assert_eq!(counts, [0, 0, 0, 0, 0, 1]);
    }
}
```
  Sweep EOF/reset at every signed request head/body and response byte. Valid uncut control: responses1,
  hints1, status204. Before verification: hints0; after verification a queued204 may have produced one
  hint, never two. Response queued before Hint; all sockets settle. Retain the old split sweep.
Proof:
world; `cargo nextest run -p skein-forge-machine-world`: its step-start focused count plus2 (40 from38;42 after08.s1), each <0.5 s; this world alone controls close/terminal ordering and io settlement.
world; `cargo nextest run -p skein-forge-machine-world --profile fuzzy`: 7 passed (6+1), new sweeps ≤5 s together; inherited four ≤7.8 s each and fuzzy_hooks <0.5 s.
Don't: send an answer for an outstanding call after Close; spin waiting for already settled io.
Size: about 0 lines moved, 280 new including tests: 90 close/settlement observation, 90 tables/tests,
  100 listener sweep and assertions; no shipped code.
Removes: nothing.

### 08.4 TLS faults and the machine stories over TLS
Scope: forge-connection 8, 10's TLS forge, including handshake faults/deadlines and Close.
Needs: 08.3's machine/listener stories and landed skein08.s1; skein's TLS fixtures exist at base.
Files: tests/forge/machine/{Cargo.toml,src/{machine,peer,fixtures,lib}.rs,tests/machine.rs}; new src/tls.rs,
  tests/fuzzy_tls.rs and tests/exhaustive_tls.rs; skein's .config/nextest.toml (the exhaustive profile).
  Manifest: skein-tls-world = { path = "../../tls" }.
Shape: wrap scripted HTTP peer bytes in skein_tls_world::server::Server, pki::Server::plain().config(),
  pki::client(&[]) and pki::name(). TLS clock starts at pki::VALID and advances with virtual time,
  never EPOCH. Use 03.1's Transport::Tls, no host field; Host follows decision25. Extend 08.2/08.3's
  machine tables to both transports, plus Handshaking (D, Unseen, Unavailable, cost0, terminal1) and Close.
  tls::tls_cuts(seed: u64, sweep: Sweep) -> Vec<(u64,u64,u64,u64)> returns independently counted written
  requests, reported cost, terminals, Closed events per cut: Sweep::Every at every byte (4,709 rows at
  this pin), Sweep::Boundaries at the readers' boundaries (the measured aim, below). TLS draws kernel
  entropy (testing-strategy 4.4); ECDSA handshake lengths vary. Capture each trial's wire flight,
  derive the cut range from those bytes, and cut that same flight; never use a previous handshake's length or assert an exact TLS byte count.
  HTTP cuts range over that trial's plaintext framing; assertions concern outcomes/counts only.
  Ruling214's single handshake-only follow-up is exhausted. A fresh owner
  gets at most20 wall minutes from start, TWO executable
  absolute-Bash submissions and10 active execution minutes, including
  setup/compilation, for the reached HTTP terminal copies and busy clock.
  First inspect and repair only this world's Peer and Machine driver.
  Submission1 may exercise the existing full-length first-start HTTP
  response cut, checking pending io, independently measured writes/cost
  and last-Closed ordering. Submission2 is ONE complete existing TLS
  sweep, after that narrow check passes and a source-backed cause is
  recorded. Setup failures spend submissions; queue time creates none.
  Generalize the responder-terminal guard across every reached cut path,
  retaining decryption and independent write accounting. A one-time
  awaiting_request clear is insufficient because io rearms it. Establish
  quiescence with all actionable room/grant, ciphertext and fault work
  settled before letting run_for jump to its existing next deadline;
  settle is a same-time loop, not the10us clock loop. No skipped io,
  early peer retirement or shared Wire/TLS API change. Keep Silent,
  held-response and release paths nonterminal. Preserve all byte rows,
  actual-trial flights, controls, assertions and caps. Return on an
  exhausted submission, repeated failure, missing coverage, unproved
  io-liveness premise, timing miss or needed change outside this world.
  A complete passing sweep still beyond5s goes to the designer for its
  measured aim before any further tuning run. Success permits only the
  unspent normal proofs once, at43 focused/8 fuzzy. A narrow diagnostic
  adds no permanent identity/scenario. Refresh further reached terminal
  copies and observations, listing users and changed observed write counts
  in the commit body; the before/after-write error mapping stays.
  The measured aim (the designer's answer of 2026-10-20, forge-connection 10, "Over TLS, every byte
  outside the gate"). Cause, measured per row on this draft: a client-hello cut 1.2 ms (6 ticks); a
  server-flight cut 3.4-3.7 ms (55 ticks: the component's TLS client 1.1 ms, the peer's rustls server
  1.3 ms); an HTTP cut 5-7 ms (106-138 ticks); a Locate-response cut 9-10 ms (1,749 ticks, 1,598 of
  them busy only on `wire.eof && !wire.ended`). The 4,241 rows past the client's hello cost about
  10 s in cryptography alone, so no driver repair brings Every within 5 s. No two rows can share a
  flight: each is bound to its own hello, and seeded entropy, a cloned world or a replayed flight
  changes the contract. So:
  - Quiet: a peer whose only busy cause is its end waiting (`wire.eof && !wire.ended`) and whose Wire
    answered nothing in that tick is not busy: Wire delivers the end once a demand comes, and run_for
    jumps to the next deadline meanwhile. The rest of busy, the same-time settle and every terminal
    path stay. On this draft it took the Locate-response rows from 1,749 ticks to 151, and all 4,709
    rows passed.
  - Boundaries: in each trial's own captured flight, each record's start and its 5-byte header's end,
    and the flight's end; in the control's captured requests and the script's responses, 0, each line's
    end (after its CRLF), the head's end and the body's end. Each edge -1/0/+1, clamped to 0..=length,
    deduplicated, with EOF and reset. A handshake cut names its record, edge and delta, and the peer
    resolves it against that trial's flight at capture, as Every resolves its byte; a trial whose flight
    has no such record ends the phase. At this pin flights hold 3 records (ServerHello 122,
    ChangeCipherSpec 1, one encrypted record of 1,063-1,066) and the hello 1 (228): about 220 rows.
  - Map: `fuzzy_tls.rs::tls_faults_at_each_byte_end_once` moves unchanged to
    `tests/exhaustive_tls.rs` (same name, `tls_cuts(7, Sweep::Every)`): every row's exact result
    (Unavailable/cost0 before the call's write, Timeout/cost1 after, the complete row Ok/cost1),
    cost == written, (terminals, Closed) = (1, 1), last event closed, and the control.
    `fuzzy_tls.rs::tls_faults_at_each_boundary_end_once` runs `tls_cuts(7, Sweep::Boundaries)` through
    the same `outcome` and control. `a_tls_forge_starts_and_answers_a_call` is unchanged. Counts:
    focused 43, fuzzy 8 (7+1), exhaustive 1.
  - Suite: `[profile.exhaustive]`, default-filter `binary(/^exhaustive_/)`, global-timeout 60 s,
    slow-timeout 20 s; the default profile's filter adds `not binary(/^exhaustive_/)`; fuzzy's is
    unchanged. The machine's post-landing lane runs `cargo nextest run --workspace --profile
    exhaustive` after each skein push and reports a failure against the commit; the lane is not this
    step's.
  - Next, bounded: one narrow check of the quiet rule (the Locate response cut at its length minus one,
    EOF: result, ticks, last Closed), then the normal proofs once each. Return a boundary sweep past 5 s
    (with each phase's rows and time), an exhaustive one past 60 s, or an unproved quiet premise; never
    sample, split or thin a sweep to fit.

Test shape: one new focused (<0.5 s), one fuzzy (<5 s), one exhaustive (<60 s); pki is the trust override,
  no application profile.
  a_tls_forge_starts_and_answers_a_call uses the moved fixture's limits and this reference:
```rust
// tests/forge/machine/tests/machine.rs; existing imports plus pki and Transport.
#[test]
fn a_tls_forge_starts_and_answers_a_call() {
    let mut settings = fixtures::settings();
    settings.forges.get_mut(0).expect("forge").transport =
        Transport::Tls { server_name: pki::name(), trust: pki::client(&[]) };
    let script = vec![
        Answer::case("GET /api/v1/user", "user"),
        Answer::case("GET /api/v1/settings/api", "settings-api"),
        Answer::case("GET /api/v1/repositories/1", "repository-by-id"),
        Answer::case("GET /api/v1/repos/fixture/specimen/branches/main", "branch"),
    ];
    let mut world = Machine::with(7, fixtures::limits(), settings, script);
    world.call(1, fixtures::REPO, fixtures::branch());
    world.run_for(SECOND);
    let answer = world.answered(1).expect("one terminal");
    assert_eq!(answer.cost, 2);
    assert!(answer.result.is_ok());
    assert!(world.script_done());
    assert!(world.close());
}
```
```rust
// tests/forge/machine/tests/fuzzy_tls.rs; both files use skein_forge_machine_world::tls::{Sweep, tls_cuts}.
#[test]
fn tls_faults_at_each_boundary_end_once() {
    for (written, cost, terminals, closed) in tls_cuts(7, Sweep::Boundaries) {
        assert_eq!(cost, written);
        assert_eq!((terminals, closed), (1, 1));
    }
}
// tests/forge/machine/tests/exhaustive_tls.rs: the every-byte sweep, moved unchanged.
#[test]
fn tls_faults_at_each_byte_end_once() {
    for (written, cost, terminals, closed) in tls_cuts(7, Sweep::Every) {
        assert_eq!(cost, written);
        assert_eq!((terminals, closed), (1, 1));
    }
}
```
  Sweep minimal handshake and HTTP start/first-call bytes, request and response, EOF/reset, at every
  byte (exhaustive) or each boundary (fuzzy); reuse trust.
  Before any call write (including failed start/handshake): Unavailable/cost0; after first call write:
  Timeout/cost1 (Read::Locate as 08.2); successful control cost1. Each case checks that Error.
Proof:
world; `cargo nextest run -p skein-forge-machine-world`: its step-start focused count plus1 (43 after08.s1/08.3), each <0.5 s, new focused work ≤1 s so far; only this harness controls TLS/HTTP phase and Close boundaries.
world; `cargo nextest run -p skein-forge-machine-world --profile fuzzy`: 8 passed (7+1), additions ≤5 s together, inherited four ≤7.8 s each and fuzzy_hooks <0.5 s; suite <60 s.
world; `cargo nextest run -p skein-forge-machine-world --profile exhaustive`: 1 passed, within the profile's 60 s; run once before landing, then by the post-landing lane after each push.
Don't: TLS listener, renewal, optimized dependencies, fixed ECDSA flight lengths or entropy-seeded assertions;
  a boundary row drawn at random, a cut resolved from another trial's flight, a cloned or snapshotted
  world, a replayed flight, or the every-byte sweep in the gate's fuzzy suite.
Size: about 0 lines moved, 360 new including tests: 140 TLS peer/clock, 160 sweeps and the quiet rule,
  60 focused/table tests; 8 lines of nextest profile.
Removes: nothing.

### 08.5 The counted worst case — dropped

Dropped by the scope cut, ruling 189 (2026-10-20). Moved to the GitHub draft, section 17 (ruling 189).
No implementation or proof of this increment runs in forge-kit; its earlier specification remains
in git history. Every retained move and its inherited coverage still runs.

## Timing and deferred measurement

The retained ten focused identities (eight original plus08.s1’s two separated identities) and three new fuzzy tests keep their original ceilings.
Report additions against decision18's +1.5s default/+6s fuzzy whole-plan aims, retaining the four
inherited sweeps and their measured cost separately. The new protocol tests09.2–09.4 and independent
allocator measurement08.5 move to the GitHub draft, section17 (ruling189); measure the complete
component once after GitHub's holders exist. Keep the existing component formula and runtime admission.

## Done when

From skein; existing base-form command evidence is findings/base-skein-08.md. New crates/tests first run
when dependencies land (development.md 5.4). Record each new test's elapsed time from these commands,
then sum the ten focused identities (including08.s1’s two separated cases) and three new fuzzy tests separately; report those sums against decision18's +1.5 s/+6 s whole-plan aims. Keep moved times separately, not subtracted from
skein's additions. Per-test sums are conservative accounting, not a substitute for README's three gate
runs measuring whole-suite wall time. Report an excess with its three slowest tests and measured sums.

- `cargo nextest run -p skein-forge-machine-world` prints43passed after08.s1 (41 original+2 separated identities), each <0.5 s; new focused sum ≤1 s.
- `cargo nextest run -p skein-forge-machine-world --profile fuzzy` prints 8 passed; inherited four ≤7.8 s each, fuzzy_hooks <0.5 s, new fuzzy sum ≤5 s.
- `cargo nextest run -p skein-forge-machine-world --profile exhaustive` prints 1 passed within 60 s: TLS's every-byte sweep, outside the gate (forge-connection 10).
- `cargo clippy -p skein-forge-machine-world --all-targets -- -D warnings` prints Finished without warnings.
- `rg -n '^    "tests/forge/machine",$' Cargo.toml` prints exactly one member line.
- `rg -n -i temper tests/forge/machine` prints nothing (rg exit1); README Done when5 covers the whole kit.
