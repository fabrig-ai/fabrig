# Session 03: skein-forge-connection

Move the forge component into skein, on the owner's values and plain shared
bounds. Then close the request-head gap and check starts and page sizes at
their boundaries. The machine and listener worlds belong to skein 08
(decision 23); this session ends at 03.3.
Needs at start: temper 01.4's component using skein-forge's call/start plans,
Credential and host, landed on origin/main. Copy that code, never this base.
Counts below preserve inherited test names; the ready check adds prerequisite
tests, without counting skein-forge's tests again.

## Read first

- skein `docs/design/forge-connection.md` 2, 3, 5, 6, 7, 8, 10;
  `forge.md` 3.7, 4, 5.1, 6, 8, 11; `tls.md` 2, 3.4; the designer's answers
  1–6, 13 and 14 (copied into
  decisions.md), and the TLS name answer (skein 1908a365).
- skein `docs/foundation/programming-model.md` 4.2, 5.2–5.4, 6.2–6.3, 7,
  9, 10; `testing-strategy.md` 2.1, 2.4–2.5, 3, 4.4, 6–8, 10.
- README (sessions, Rules, Done when), code-organization.md; decisions.md
  2, 3, 7, 8, 11, 14, 18, 23, 24 and Design questions 1–6, 13–14;
  findings/{survey-03-connection,trace-connection,base-runs,base-skein-03}.md,
  especially base-skein-03's revision evidence, which supersedes its draft.

## What the design asks

- 2–5: owner-routed calls, bounded starts, private credentials and caches;
  the transport's name supplies Host, a TLS name read by `Name::server`;
  no component dependency on the client.
- 6: signed deliveries, response before hint, listener failures and bounds.
- 7: each start check; page sizes use forge.md 3.7's fixed-width measure.
- 8, 10: step coverage here; TLS, faults, deadlines, close and allocator
  verification in skein 08, after its fakes and conformance dependencies.

## Choices this plan makes

Answers 1–6, 13 and 14 are settled, in skein 8938e16's design. Use skein 01.4's
`skein_forge::credential::Credential::Token(skein_lib::Secret)` directly.
Fixtures use `kit` and `kit/0.0.0`; retain their base numbers except the named
word/body adaptation. The five hook limits retain the shipped small profile's
values, recorded in base-skein-03.md; the kit has no service profile.

## Rules that bind it

- Copy temper's origin/main at the step's start and name its commit; only
  adaptations named in Shape accompany the move (code-organization.md 6).
  Split a moved file over 800 lines when next changing it.
- No service name in skein code, tests, values or comments; no serde, alias,
  re-export or dependency on skein-forge-client. README's listener hotspot
  covers hooks.rs; temper-service's forge.rs/settings.rs are temper's edits.
  Each "temper's side" is repaired by whichever temper increment repins next.
- Moved tests keep names, budgets and times. New units <20 ms, focused worlds
  <0.5 s, fuzzy tests about 5 s; unoptimized builds and unchanged 15/60 s budgets.
  Base-runs.md records the component's 117 tests at 0.149 s; 0.082 s is a
  lighter-load rerun. The world baseline (skein 08) is 47 focused at 0.404 s,
  6 fuzzy at 7.709 s, fuzzy_machine 7.481–7.709 s; faster runs are reruns.

## Increments

### 03.1 The component moved
Scope: forge-connection 2–8; owner-value and plain-bound adaptations only;
  the 68 plan tests stay with their Component/HTTP harness.
Needs: temper 01.4's component on skein's plans, Credential and host.
Files: temper `crates/temper-engine-protocol-forge/` -> skein
  `crates/skein-forge-connection/`: Cargo.toml; src/{lib,component,
  component/calls,connection,hooks,hooks/connection,start,limits,grow,
  boundary,settings}.rs; src/tests.rs and src/tests/*.rs. Remove the residual
  conventions.rs. Plans, tables, marker, hooks/hint, Fields/Heard/Body/Failure,
  start's exchange mapping and host already belong to skein 01: import them.
  skein `crates/skein-tls/src/{config,lib}.rs` and `src/tests/config.rs`:
  `Name::server` and `Server` (tls.md 3.4).
  Shared hotspots: skein Cargo.toml's workspace dependency and Cargo.lock;
  crates/* supplies the member. Temper consumer paths are inventoried below.
Shape: retain Component entry points, Request/Event, MAX_OUT (16/16), Limits
  and start scheduling. Dependencies: skein-lib, -io, -http, -tls, -json,
  skein-forge; skein-world for tests. Replace Shared.client's five used fields
  with plain numbers. Changed boundaries (Kind and Webhooks stay as moved):
```rust
// skein_forge_connection::settings
/// Owner inputs held by the component until close.
#[derive(Debug)]
pub struct Settings {
    pub forges: List<Forge>, pub webhooks: Option<Webhooks>,
    pub word: Box<[u8]>, pub user_agent: Box<[u8]>,
}
/// One forge; Host derives from its address and transport name.
#[derive(Debug)]
pub struct Forge {
    pub number: u16, pub kind: Kind, pub address: skein_io::Addr,
    pub transport: Transport,
    pub credential: skein_forge::credential::Credential, pub user: u64,
}
/// A forge's connections, held by the component until close.
#[derive(Debug)]
pub enum Transport {
    /// Loopback plaintext, with the configured name when present.
    Plaintext { name: Option<Box<[u8]>> },
    /// TLS with the configured server name and trust.
    Tls { server_name: skein_tls::Name, trust: skein_tls::Config },
}
// skein_forge_connection::limits
/// Owner's client/domain numbers and io stream caps, copied at start.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Shared {
    pub calls: u32, pub rows: u32, pub answer_bytes: u32, pub op_bytes: u32,
    pub lifetime: Duration, pub name_bytes: u32, pub repositories: u32,
    pub io: skein_io::Limits,
}
// skein_forge_connection::limits::Unusable; added variants only.
/// Owner word outside its alphabet or length: terminal before begin.
Word,
/// Empty or unsafe plaintext name: terminal before begin; no input bytes.
PlaintextName { forge: u16 },
```
  No Forge.host or separate token/kind fields. skein-forge's pure
  host(address, name), from 01.4, unchanged, derives Host for both check and
  send: name, else IP (IPv6 bracketed), :port only when neither 443 nor 80
  (decision 25). Plaintext supplies its optional name. TLS supplies its
  server name, read by skein-tls's new `Name::server(&self) -> Server<'_>`
  (`Server::Dns(&str)`, the text as given; `Server::Ip(core::net::IpAddr)`;
  tls.md 3.4): Dns -> host(address, Some(text)); Ip ->
  host(SocketAddr::new(ip, port), None), with the address's port: the
  name's IP even when it differs from the address's (forge-connection 3.1).
  One private function makes Host for check and send; a TLS name is never
  refused. rustls's ServerName is non_exhaustive: Name keeps which of the
  two it holds, set by new, so that server is total with no unreachable arm;
  build core's IpAddr from pki-types' octets (AsRef<[u8; 4]>,
  AsRef<[u8; 16]>), whose std conversions skein-tls lacks. Never
  ServerName::to_str. conventions::host was deleted by temper 01.4. Remove
  USER_AGENT; send Settings.user_agent on every request.
  Render the owner's Credential through the dialect; pass word to its plans.
  Word and PlaintextName refusals are forced by the new home's owner-value
  boundary: the owner now hands the word and transport, so neither can rely
  on a system's fixed word or host. Decision 25's port rule is likewise forced
  by this boundary: derive Host from the transport's name with 01.4's helper.
  Their checks and wiring tests are part of this move, below.
  Refuse Word outside 1..=16 bytes of [a-z0-9-]; refuse PlaintextName when
  empty or containing a byte <=space, >=DEL or one of /:[], before any io.
  Retain landed skein_forge::marker::length(&settings.word, op_bytes),
  whose checked formula is 2*op_bytes+word.len()+15. In
  tests::limits_that_disagree_with_the_client_name_both_limits, change the
  BodyBytes row from 1044/needed 1045 to 1041/needed 1042; fixtures::limits'
  body_bytes:1045 becomes 1042 for kit. Other fixture limits stay unchanged.
  Reuse skein 01.3's table storage sizing; price changed Settings/Forge holders
  in worst_case and report the complete fixture sum now and after each change.
  Price retained owner bytes explicitly: Settings.word, Settings.user_agent,
  plaintext name boxes and each TLS DNS name's text (Server::Dns's length; an
  IP name holds none; forge-connection 7.5) have no size input in
  worst_case(l,s,forges). In 03.1,
  add checked settings::owned_bytes(&Settings) for the newly retained owner
  byte slices and pass that byte count into
  worst_case; thread it through check and every caller, including temper's
  crates/temper-service/src/system.rs::memory and the conformance's
  tools/forgejo-conformance/src/component/hosted.rs::Hosted::new. Adapt the
  moved src/tests.rs and src/tests/hooks.rs bound callers too. Refresh any
  further caller, copied expected sum or derived startup fixture reached by
  the change, and list it in the commit body. This is pending owner-value code, not a ready-check
  implementation. Keep ready-check sum 2,812,325 as historical baseline;
  the pre-owner-value fixture now measures 2,812,377 after landed holder changes
  (recheck at temper9a2eda75). Neither figure is the final owner-byte sum.
  The independent allocator measurement is to the GitHub draft, section 17 (ruling 189),
  once its complete holders exist; the existing formula and runtime admission remain.
  Temper's side: crates/temper-service/src/forge.rs::{settings,shared,problems,
  indexed} must construct Forge/Credential/Transport, flatten Shared, handle
  Word/PlaintextName and map forge numbers to indices. crates/temper/src/forge.rs
  setup construction uses skein-forge's host and the owner's user agent.
  Repair runtime fixtures in crates/temper-service/src/tests/fixtures.rs::forge_runtime,
  tests/one-process/src/fixtures.rs::forges and tools/forgejo-conformance/src/component/hosted.rs::new;
  adapt this crate's src/tests/fixtures.rs::settings before copying.
  Also inventory tests/engine/protocol-forge/src/fixtures.rs::{settings,shared,
  protocol_shared}: hand the component word and agent, flatten the five client
  numbers in shared and protocol_shared. Keep the world's temper word and its
  1045-byte body bound until its skein 08 move adapts the word to kit.
  crates/temper/src/lib.rs::forges also imports the runtime transport; its
  plaintext-name construction belongs to temper03.1's next repin (temper
  01.4 retained the unit Plaintext variant); its component crate path changes
  at that repin. Temper hands TLS's server_name as now and reads no name back;
  a TLS forge's Host now follows that name, not the API address's host name
  that temper-service's forge.rs passes to host today (forge.md 5.1). Verify
  tests/system/src/composed/link.rs::assignment too: its settings::Transport
  is temper's configuration enum, not the component enum. Preserve that seam.
Test shape: 112 inherited component test identities: root 12, connection 20, hooks 12,
  reads 27, writes 18, contents 23; four root marker/table tests went in 01.3.
  host_names_include_only_nondefault_ports_and_bracket_ipv6 and its pure
  vectors live only in skein-forge's src/tests.rs from 01.4; drop this copy.
  New each_request_carries_the_host_its_transport_derives is the component
  wiring check: capture each sent Host for TLS/named plaintext, IPv4/v6 and
  default/custom ports; include TLS on 80 and plaintext on 443, both omitted.
  Assert literal forge.example, forge.example:3000, 127.0.0.1 and
  [0:0:0:0:0:0:0:1] as applicable. Add TLS IP names that differ from the
  address: name 192.0.2.7 on 127.0.0.1:3000 -> 192.0.2.7:3000; name
  2001:db8::7 on 127.0.0.1:443 -> [2001:db8:0:0:0:0:0:7]; and name ::1 on
  [::1]:443 -> [0:0:0:0:0:0:0:1], never ::1.
  Extend skein-tls's a_name_is_a_dns_name_or_an_ip_address: skein.test ->
  Server::Dns("skein.test"), Skein.Test keeps its case; 127.0.0.1 and ::1
  -> Server::Ip of core's IpAddr. No new test name.
  Extend unusable_forge_settings_name_the_forge: None is Ok; empty, NUL, tab,
  0x1f, space, DEL, 0x80, 0xff and each /:[] -> PlaintextName{forge:1};
  !, ~ and adjacent . 0 9 ; Z \\ ^ -> Ok. Check and construction agree.
  Define fixtures::WORD and USER_AGENT from kit and kit/0.0.0 once. Define
  OP_BYTES=512 from fixtures::shared, BODY_ROOM=1042 from 2*OP_BYTES+WORD.len()+15,
  BODY_ROOM_SHORT=BODY_ROOM-1 and MAX_WORD_BODY_ROOM=1055 from 2*512+16+15.
  Use the named bounds in the new body/word units per testing-strategy.md §10.
  Three new tests in src/tests.rs, reusing the sole src/tests/fixtures.rs:
  the wiring check above, plus two units:
  owner_words_are_checked_before_io: 0/1/16/17 bytes and K -> Word/Ok/Ok/Word/Word
  (give longer words sufficient body room); body_room_includes_the_owner_word:
  op_bytes=512, word=kit, 1041 -> BodyBytes{body_bytes:1041,needed:1042}, 1042 -> Ok.
```rust
// crates/skein-forge-connection/src/tests.rs; existing imports and fixtures.
#[test]
fn body_room_includes_the_owner_word() {
    let shared = fixtures::shared();
    let settings = fixtures::settings();
    for (body_bytes, expected) in [
        (fixtures::BODY_ROOM_SHORT, Err(Unusable::BodyBytes {
            body_bytes: fixtures::BODY_ROOM_SHORT, needed: u64::from(fixtures::BODY_ROOM),
        })),
        (fixtures::BODY_ROOM, Ok(())),
    ] {
        let limits = Limits { body_bytes, ..fixtures::limits() };
        assert_eq!(limits::check(&limits, &shared, &settings), expected);
    }
}
```
Proof: world, `cargo nextest run -p skein-forge-connection`: 115 passed,
  <0.5 s total, new units <20 ms; `cargo nextest run -p skein-tls`: all
  pass, count unchanged; includes
  creations_carry_the_key_marker_at_the_head_of_their_body through Component
  and HTTP. This is the highest caller until temper 03.1 connects the crate.
Don't: fix head size, signature or start-policy gaps; move tables twice;
  edit temper's consumers here (its next repin repairs them).
Size: about 2,850 production lines moved, 270 new; about 3,100 test lines moved.
Removes: residual conventions/USER_AGENT, Forge.host/token, Shared.client in
  the copy; nothing from temper.

### 03.2 Head admission, signature shape and the token ranges
Scope: forge-connection 2, 3.2–3.3, 6.2–6.3, 7.4; answers 2–6.
Needs: skein 03.1's component on owner values and transport names.
Files: crates/skein-forge-connection/src/{limits,component,start,
  hooks,hooks/connection,tests,tests/hooks,tests/connection,tests/fixtures}.rs;
  new src/component/tests.rs, a private child of component; make
  tests.rs's fixtures module pub(crate) for it and 03.3's limits tests.
Shape: three changes against the base, each tested below. Head admission:
  head = target_bytes + derived Host length + credential's token length
  + user_agent length + 192, checked as u64 against http(...).request for
  both transports. Preserve refusal order and 03.1's body fixture/bound edits.
  Reuse Unusable, adding no variant: credential empty/oversize or containing
  a byte <0x20 except tab, or 0x7f -> Token{forge}; user 0 -> User{forge};
  empty/unsafe user agent -> Request{forge} at that forge's head check.
  The base already refuses empty/oversize tokens and user 0; controls and
  the owner's agent are new checks. Update Token/Request's descriptions.
  Signature: replace signature's is_ascii_hexdigit with lowercase hex only.
  The base accepts uppercase at the head and refuses it after the body;
  now anything but 64 lowercase hex digits is 403 at the head, body unread.
  Make hooks.rs's mod connection pub(crate), so fixtures can name Hooked;
  keep signature private and test it through Hooked::{stream,pump}.
  Tokens: API owners start at 1; open_start's base guard is <2^60, overlapping
  hooks. Check <2^59 before issuing an API owner; keep checked increment and
  exhaustion assertion, allowing 2^59-1 but never the listener's 2^59.
  hooks::OWNER=2^59, hooks start at 2^59+1 and remain <2^60: no hook change.
  No change for answer 4: Hooked::pump forwards the server's answer then
  closes on Refused; skein-http already emits 4xx, 501 and 505. Its existing
  every_rejection_s_answer_fits_the_room_set_aside_and_names_its_status covers
  all in skein-http's src/tests/server/head.rs, outside this crate's count
  and Proof command; a_malformed_head_is_answered_4xx_and_closed is skein 08's.
  No change for answer 6: start::heard already takes max(next, Retry-After),
  saturates now+wait and caps only the doubled next backoff; Hooked::call
  with no EVENT_HEADER already verifies the body, answers 204 and hints none.
  Temper's side: crates/temper-service/src/forge.rs::{problems,indexed}
  retain Token/User/Request, with no new answer-2 match arms; broaden Request's
  explanation to head validity and size, and index its forge number.
  Its prepare/settings and small profile, and the 03.1 runtime fixtures,
  must pass the longer head check at the next repin; keep their owner bytes.
Test shape: seven new tests, using fixtures::{limits,shared,settings,hook_settings}.
  Extend the inherited unusable_forge_settings_name_the_forge in src/tests.rs:
  forges 2/3, token 0/64/65, user 0/1, duplicate, loopback v4/v6 versus
  nonloopback/InProcess listener, off-loopback plaintext, secret 0/128/129
  and unknown forge -> Forges/Token/User/Duplicate/Listener/Address/Secret
  as applicable, Ok at valid bounds; all other constraints valid.
  Extend zero_documents_and_stacked_demands_are_refused_in_order: numeric
  zero/first valid; connections=calls and ±1, deadline=lifetime and +1 ns,
  string=name and -1, repositories=owner and +1, hook_secret_bytes=31/32;
  documents and all six Stacked paths at demand=cap/cap-1; overflow -> Zero;
  refusals name the literal changed pair, valid bounds -> Ok.
  Keep both names; these extensions replace the draft's two new limit tables.
  New src/tests.rs tests:
  request_head_counts_user_agent: pad only agent to REQUEST_ROOM minus
  target/derived Host/token/192; bound -> Ok, +1 -> Request{forge:1}; TLS DNS,
  TLS IP (2001:db8::7, bracketed and uncompressed) and named plaintext, with
  nondefault ports included in Host's measured length.
  header_controls_are_refused_before_io: for credential and agent separately,
  empty, NUL/CR/LF/0x1f/0x7f -> Token{forge:1}/Request{forge:1};
  tab/space/0x7e/0x80/0xff -> Ok; user 0/1 -> User{forge:1}/Ok.
  limits::check and Component::new agree, before begin/io; no secret in errors.
  New tests/hooks.rs tests: add fixtures::hook, fixtures::signed_head and
  their small Hooked/Wire driver in fixtures.rs; all three are new. The
  driver retains full hook_settings() and passes &Settings to Hooked::pump,
  including the owner word. signed_head uses a local lowercase digit table
  for signature text; the old crate::marker::hex path is gone and skein
  marker::hex is private. Model
  pumping on fixtures::ConnectionHarness::{pump,answer} and
  skein_world::stream::Wire (no copied machine world/fake dependency):
  signature_shape_is_refused_at_the_head: known HMAC, valid pull payload;
  63/65 digits, 64 uppercase (including A–F), or 64 with nonhex g
  ->403, no body read/hint; correct 64 ->204 then one hint;
  wrong lowercase 64 ->403 after body, zero hints. Supply head separately to
  prove early refusals without body demand; assert closed for every 403.
  a_verified_delivery_without_an_event_is_answered_204_without_a_hint:
  omit EVENT_HEADER completely; valid signature ->204, zero hints, kept alive.
  Reference for this new harness kind; signed_head signs the supplied body
  with hook_settings's secret and omits the event header for None:
```rust
// crates/skein-forge-connection/src/tests/hooks.rs
#[test]
fn a_verified_delivery_without_an_event_is_answered_204_without_a_hint() {
    let mut h = fixtures::hook();
    h.write(&fixtures::signed_head(b"{}", None));
    assert_eq!(h.status(), None);
    h.write(b"{}");
    assert_eq!((h.status(), h.hints(), h.closed()), (Some(204), 0, false));
}
```
  New tests/connection.rs test start_retry_after_wins_over_the_backoff_cap:
  fixtures::Harness receives 429 on the identity start read only, at now=1 s;
  Retry-After=20 s, next=1 s, max=8 s -> Waiting{until:21 s,next:2 s}.
  Admit a call at 1 s: no answer at 11 s-1 ns, exactly one Unavailable,
  cost 0 at 11 s; no reconnect before 21 s, one at 21 s. Separately call
  start::heard using skein_forge::forgejo::plans::start::Heard for
  Start::Checking and Start::Settings, each with next=1 s and
  retry=20 s: at now=1 s both give Waiting{until:21 s,next:2 s}; at
  Time::from_nanos(u64::MAX-1), both give until=Time::from_nanos(u64::MAX),
  next=2 s. This covers the gaps in moved
  the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise (5 s only) and
  a_call_waiting_for_its_forge_is_unavailable_at_its_deadline (no 429).
  Split the new token boundary coverage into two tests so should_panic cannot
  hide rejection of the valid predecessor. api_tokens_stop_before_the_listener_range
  starts next_token at LISTENER_OWNER and makes one call; a separate
  the_last_api_token_is_still_issued starts at LAST_API_OWNER and asserts the
  returned token. Define LISTENER_OWNER=1_u64<<59_u32 from hooks' reserved prefix,
  LAST_API_OWNER=LISTENER_OWNER-1, and use MAX_OUT.below for the queue capacity.
  Keep the production guard before token issuance and checked increment.
  This adds one new test, changing 03.2 to seven new/122 total and 03.3 to 128.
  The corrected private-file reference follows:
```rust
// crates/skein-forge-connection/src/component/tests.rs
use super::*;
use crate::tests::fixtures;
#[test]
#[should_panic(expected = "connection token prefix")]
fn api_tokens_stop_before_the_listener_range() {
    let limits = fixtures::limits();
    let mut c = Component::new(fixtures::settings(), &limits, &fixtures::shared()).expect("valid");
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut io = Queue::with_capacity(MAX_OUT.below);
    c.next_token = LISTENER_OWNER;
    c.open_start(0, &env, &mut io);
}

// The listener owns 2^59; an API owner may still use its predecessor.
const LISTENER_OWNER: u64 = 1_u64 << 59_u32;
const LAST_API_OWNER: u64 = LISTENER_OWNER - 1;
#[test]
fn the_last_api_token_is_still_issued() {
    let limits = fixtures::limits();
    let mut c = Component::new(fixtures::settings(), &limits, &fixtures::shared()).expect("valid");
    let env = Env { now: Time::ZERO, wall: skein_lib::Wall::EPOCH, limits };
    let mut io = Queue::with_capacity(MAX_OUT.below);
    c.next_token = LAST_API_OWNER;
    assert_eq!(c.open_start(0, &env, &mut io).raw(), LAST_API_OWNER);
}
```
Proof: world, `cargo nextest run -p skein-forge-connection`: 122 passed,
  <0.5 s total, new units <20 ms; harnesses observe HTTP requests/refusals.
  The component harness is the highest local caller; temper's next repin
  (temper 03.2 if next) repairs and proves its configuration mapping.
Don't: add answer-2 refusal variants, cap Retry-After, change HTTP's answers,
  reorder existing refusals or duplicate the dialect's header/credential vocabulary.
Size: about 0 lines moved, 110 new production lines.
Removes: head calculation omitting the agent, uppercase signature admission,
  API token allocation into the listener's range.

### 03.3 Page sizes, two forges and start progression
Scope: forge-connection 3.3, 4.1, 7.3–7.4, 10; forge.md 3.7 and 5.1.
Needs: skein 03.2's admitted settings; skein 02.2's skein_forge::measure::answer
  and fixed widths (decision 24, designer's answers 13/14).
Files: crates/skein-forge-connection/src/{limits,tests,tests/connection,
  tests/hooks,tests/fixtures}.rs; new src/limits/tests.rs for private largest_row tests.
Shape: keep page_size(Listing,u32,&Limits,&Shared)->u32 and private
  largest_row(Listing,&Limits)->Option<u64>. Price every listing by
  skein_forge::measure::answer, using its fixed widths in checked bound
  arithmetic, without allocating maximal answers in production or adding a
  client dependency. The independent test oracle constructs maximal answers
  from string_bytes/body_bytes/inner, then calls measure::answer; no rows
  policy in the measure. Keep the conservative key allowance body_bytes/2.
  At the adapted fixture, named ROW_ITEMS/REVIEWS/STATUSES/REMARKS/BRANCHES/
  COLLABORATORS/PULLFILES are 2971/1691/544/1234/144/16/2276 bytes.
  Keep a_page_is_the_smallest_of_rows_cap_and_answer_room's name: its answer
  room row uses ROW_ITEMS (was 4096), still expects 1; cap/rows expect 2/8.
  In limits_that_disagree_with_the_client_name_both_limits, the ("answer", ...)
  row uses ROW_ITEMS-1 (was 64), still PageSize{listing:Items}. Share named
  sizes in tests/fixtures.rs; limits/tests.rs reuses 03.2's pub(crate) fixtures.
  Existing start/two-forge behavior stays unchanged.
  Temper's side: crates/temper-service/src/forge.rs::{small,shared,problems,
  prepare} and the 03.1 fixtures consume answer_bytes/rows and PageSize;
  the next repin repairs affected page-size expectations to this same measure,
  including the copied conformance's component/hosted.rs, without raising limits.
Test shape: six new tests. In src/limits/tests.rs reuse crate::tests::fixtures;
  each_listing_uses_its_largest_answer_row: all seven listings; independently
  built maximal answer's measure equals ROW_<LISTING> and largest_row;
  page_size at ROW-1/ROW/2*ROW ->0/1/2, cap=1 ->1, rows=1 ->1.
  limits_check_admits_one_largest_row_of_every_listing: public check covers
  all seven together, so MAX_ROW=max(the seven ROW values)=2971; MAX_ROW-1
  ->PageSize{Items}, MAX_ROW ->Ok. A smaller listing's individual exact row
  cannot make public check accept while another listing needs more room.
  Reference for this new file, with the sizes shared from tests/fixtures.rs:
```rust
// crates/skein-forge-connection/src/limits/tests.rs
use super::{Listing, Shared, Unusable, check};
use crate::tests::fixtures;
#[test]
fn limits_check_admits_one_largest_row_of_every_listing() {
    let limits = fixtures::limits();
    let settings = fixtures::settings();
    for (answer_bytes, expected) in [
        (fixtures::MAX_ROW - 1, Err(Unusable::PageSize { listing: Listing::Items })),
        (fixtures::MAX_ROW, Ok(())),
    ] {
        let shared = Shared { answer_bytes, ..fixtures::shared() };
        assert_eq!(check(&limits, &shared, &settings), expected);
    }
}
```
  In src/tests/connection.rs, reuse fixtures::Harness with its admitted limits.
  First repair tests/fixtures.rs::Harness::{with,drain}: keep a bounded mapping
  from configured endpoint address to forge number, and route Request::Connect
  using addr instead of assigning every Peer forge:FORGE. Give the two forges
  distinct loopback ports in a shared fixture. Keep existing moved identities.
  For an unknown-forge assertion of zero total connects, use empty configured
  forges; otherwise assert the connect count did not change around the call.
  an_unconfigured_forge_is_unavailable_at_zero_cost: forge 99 ->one
  Answered{cost:0,result:Err(Unavailable)}, zero connection requests.
  a_stopped_forge_does_not_stop_another: forge 1 wrong user ->Stopped once;
  forge 2 reaches Ready and answers its branch, cost=2 including path.
  an_idle_connection_is_reassigned_to_another_forge: calls=connections=1,
  ready forges 1/2; idle forge 1 closes, forge 2 connects and answers once.
  start_backoff_doubles_to_its_maximum: failed start waits 1,2,4,8,8 s;
  before each deadline no connect, at it one. The original terminal-state matrix
  is separated by03.s1, preserving its inputs and assertions.
Proof: world, `cargo nextest run -p skein-forge-connection`: 128 passed,
  <0.5 s total, new units <20 ms; Component/HTTP harness proves calls and
  eviction over bytes. skein 08.1 connects the separate machine world after
  temper 03.1, temper 05.1, skein 05.2 and skein 06.2 supply its dependencies.
Don't: move/rewrite plans, retry a call, add Busy or invent a second measure.
Size: about 0 lines moved, 60 new production lines.
Removes: largest_row's target-layout-dependent size_of row pricing.

## Done when

Run once from skein's root; base forms and outputs are in base-skein-03.md.

- `cargo nextest run -p skein-forge-connection` prints the03.3 count
  plus03.s1’s relative+1: 129 at the checked source, 112 inherited identities
  +16 original additions (3+7+6) +1 separated terminal test, within the bounds
  above. Recount unrelated additions at start; historical128 proofs stay historical. Pure host vectors belong to skein 01.4; world tests to skein 08.
- `cargo clippy -p skein-forge-connection --all-targets -- -D warnings`
  exits 0 without warnings.
- `rg -n -i temper crates/skein-forge-connection` prints nothing
  (rg exits 1 for no matches).
