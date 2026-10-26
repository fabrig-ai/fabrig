# Session 01: skein-forge

Build the neutral vocabulary, dialect, plans, marker, starts, Host and credentials.
Needs at start: nothing (README, sessions).

## Read first

- skein `docs/design/forge.md` 2–9, 11; `forge-connection.md` 3.1–3.2;
  `docs/foundation/programming-model.md` 3, 10; `testing-strategy.md` 2.1, 8, 10.
- README (sessions, Rules, Done when); `code-organization.md` 1–3, 5–8;
  `decisions.md` 2–4, 6–8, 14, 18–19, 24–25, Design questions 1, 7–14, 34–35;
  designer answers 1–3; findings `survey-01-forge.md`, `survey-03-connection.md`,
  `trace-forge.md`, `base-runs.md`, `base-skein-01.md`.

## What the design asks

- 2–3: lib-only no_std vocabulary on skein-lib/skein-json; admission stays client-side.
- 4–5: pure begin/heard plans, plain bounds, tables, starts, Host, exchanges and failures.
- 6–8: the owner's marker word, deliveries to hints, redacted typed credentials.
- 9, 11: preserve pin facts; the plans' canned tests and the settled differences
  are skein 10's, the document/transcript sweeps skein 06.3's.

## Choices this plan makes

The entry is forgejo::plans::plan::Plan; Bounds carries page_size, rows, answer_bytes, body_bytes.
The component retains limits::{Listing,page_size,largest_row}.
PullFor uses Items, Checks Statuses, PullFiles PullFiles; one-row lookups keep limit 1.
The word is borrowed, never stored. revision is reading::revision, not a file.
No plan calls answer_bytes_unbounded/client bounds helpers (rg-checked): skein 02.2 owns them.
Harness-based locate tests stay for skein 02.1; the 68 reads/writes/contents tests for skein 03.1.

## Rules that bind it

- Copy temper origin/main at each step's start, name its commit; rename paths,
  change only what Shape names. Measure pairs as code-organization.md 6 says.
- No system name, alias, re-export or serde in skein; tests use kit (organization 5).
- Keep moved test names, budgets and times; new units <20 ms unoptimized, no new world.
  No gate proof. Split reads.rs in 01.3, the first increment changing it.

## Increments

### 01.1 The crate and vocabulary
Scope: forge.md 2–3; make vocabulary available without a client dependency.
Needs: nothing.
Files: new `crates/skein-forge/{Cargo.toml,src/lib.rs,src/api.rs,src/locate.rs,
  src/tests.rs}`; root `Cargo.toml`, `Cargo.lock`. From temper-engine-domain-forge-client/src/: api.rs through
  Collaborator -> api.rs; locate.rs -> locate.rs. No sizing helper moves.
Shape: retain every vocabulary field and variant; `#![cfg_attr(not(test),
  no_std)]`, `#![forbid(unsafe_code)]`, alloc; only skein-lib and skein-json.
  `locate::bytes(owner: &[u8], name: &[u8]) -> Option<u64>` and
  `locate::accepts(answer: &Answer) -> bool` become
  public with #[must_use]. Private Shape/accepts/answer checks stay client-side; citations -> forge.md 3.
Test shape: src/tests.rs, two new pure tests, no fixtures or world needed:
  `locate_bytes_counts_owner_and_name`: b"ab", b"cde" -> Some(5);
  `locate_accepts_only_nonzero_repository`: Located 7 -> true, 0 and Done ->
  false. Zero tests move; existing tests::locate stays client-side. Each <20 ms. Reference:
```rust
// crates/skein-forge/src/tests.rs
#[test]
fn locate_bytes_counts_owner_and_name() {
    assert_eq!(crate::locate::bytes(b"ab", b"cde"), Some(5));
}
```
Proof: unit; `cargo nextest run -p skein-forge` passes 2 tests, <1 s;
  base form `cargo nextest run -p temper-engine-domain-forge-client`
  passes 76 in 0.084 s. No caller yet; temper 01.1 connects it, skein 02.1 brings the client.
Don't: move api::accepts, copy the client Harness, change the byte measure,
  or add an alias at the old path. Question 14's measure belongs to skein 02.2.
Size: about 450 lines moved, 30 new.
Removes: nothing; temper 01.1 removes the vocabulary copy after repinning.

### 01.2 Forgejo's dialect
Scope: forge.md 4.1, 4.3, 5–9; documents, both sides, deliveries and the pin.
Needs: skein 01.1's vocabulary and crate.
Files: `crates/temper-forge-forgejo/src/` -> `crates/skein-forge/src/forgejo/`,
  lib.rs -> src/forgejo.rs; all subdirectories and tests, including tests/fixtures.rs;
  skein-forge/src/lib.rs changed. Its two dependencies are already 01.1's; no second package.
Shape: change crate-root paths to crate::forgejo; preserve module/file names,
  types, encoders, decoders and pin constants. Replace service-specific
  fixture bytes/citations with organization 5's values/lengths; document.rs's
  “repository units temper uses” becomes
  “repository units the dialect uses”. No request, decoder or status change. Make the
  cfg(test) tests module and its fixtures module pub(crate) so later kit tests
  reuse and extend `forgejo::tests::fixtures::{limits,small,user,decode,decode_cut}`.
Test shape: all 84 tests move, names unchanged: tests root 37, accounts 9,
  delivery 15, hooks 7, orgs 7, pin 4, tokens 5, at the base. No new tests.
  Their current 0.001–0.008 s per-test times keep the <20 ms ceiling.
Proof: unit; `cargo nextest run -p skein-forge` passes 86 tests, <1 s;
  base form `cargo nextest run -p temper-forge-forgejo` passes 84 in 0.048 s.
  No caller yet; temper 01.2 connects its component; skein 06.3 adds transcripts.
Don't: rewrite a moved decoder, split files for neatness, change goldens
  except owner bytes, or bring the HTTP host into the dialect.
Size: about 7,820 lines moved, 60 new.
Removes: nothing; temper 01.2 deletes its dialect crate after repinning.

### 01.3 The call plans, tables and marker
Scope: forge.md 4.2, 5–7; all mutually calling plans move together.
Needs: skein 01.2's dialect; temper 01.1's vocabulary users and temper
  01.2's dialect users on skein, so the copy already names both kit paths.
Files: from `temper-engine-protocol-forge/src/`, reads.rs, reads/{done,size}.rs,
  reading.rs, writes.rs, contents.rs, actions.rs, plan.rs, tables.rs,
  failure.rs -> `skein-forge/src/forgejo/plans/` with the same names;
  hooks/hint.rs -> plans/hint.rs; marker.rs -> src/marker.rs;
  connection.rs's Fields, Body, Failure, Heard -> forgejo/plans.rs. Changed
  lib.rs, forgejo.rs; four tests from protocol src/tests.rs -> src/tests.rs;
  its fixtures::{REPO,specimen} -> forgejo/tests/fixtures.rs; new plans/reads/keep.rs.
Shape: expose Plan (its reads/writes payload types too), Step, Exchange, Range and events; tables'
  RepositoryEntry { row: Repository, used: u64 }, UserEntry { login: Box<[u8]>,
  used: u64 } become public for the component's memory accounting; Tables
  retains its private maps and clock, its methods and LRU algorithm.
  Repository keeps owner/name/html_url: Box<[u8]>, object_format: ObjectFormat,
  merge_style: MergeStyle. Fields keeps date: Option<Wall>, retry_after:
  Option<Duration>, has_more: Option<bool>, content_range: Option<ContentRange>,
  close: bool. Body keeps Document(Document), Raw { bytes: Box<[u8]>, more:
  bool }, Empty, Malformed(Malformed); Failure keeps Unseen/Seen; Heard keeps
  Head { status: u16, fields: Fields }, Row(Row), Done(Body), Failed(Failure).
  Exchange keeps request: Request, range: Option<Range>; Range Head/Tail(u32);
  Step Send(Exchange)/Done(Result<api::Answer,api::Error>), no cost field:
  the component counts cost, one per request whose first byte it wrote, as
  the base and forge.md 4.1 and 4.2 do (decisions.md, Design questions 34).
  The forced boundary change, compiled in findings/base-skein-01.md:
```rust
// skein_forge::forgejo::plans::plan
/// The owner's checked values for this call; no component/client Limits.
pub struct Bounds {
    pub page_size: u32, pub rows: u32,
    pub answer_bytes: u32, pub body_bytes: u32,
}
impl Plan {
    pub fn new(repository: api::Repository, op: api::Op) -> Self;
    pub fn begin(&mut self, bounds: Bounds, word: &[u8], tables: &mut Tables)
        -> Result<Step, api::Error>;
    pub fn heard(&mut self, heard: Heard, bounds: Bounds, word: &[u8], tables: &mut Tables)
        -> Result<Option<Step>, api::Error>;
    pub fn raw_room(&self, answer_bytes: u32) -> u32;
}
// skein_forge::marker
pub fn valid_word(word: &[u8]) -> bool;
pub fn length(word: &[u8], key: u32) -> Option<u32>;
pub fn encode(word: &[u8], key: &[u8], text: &[u8], limit: u32) -> Option<Box<[u8]>>;
pub fn decode(word: &[u8], body: &[u8]) -> Option<Box<[u8]>>;
// skein_forge::forgejo::plans::reading; d is forgejo::document
pub fn body(word: &[u8], body: d::Body) -> (Option<Box<[u8]>>, Box<[u8]>);
pub fn summary(word: &[u8], issue: d::Issue) -> api::Summary;
pub fn comment(word: &[u8], value: d::Comment) -> api::Comment;
pub fn remark(word: &[u8], value: d::ReviewComment) -> api::Remark;
pub fn review(word: &[u8], value: d::Review, format: ObjectFormat)
    -> Result<Option<api::Review>, api::Error>;
// skein_forge::forgejo::plans::hint
pub fn hint(forge: u16, word: &[u8], delivery: &Delivery) -> Option<api::Hint>;
```
  Preserve pending/set_pending/after/take_cut/error signatures. Thread Bounds
  and word through reads/writes/contents/actions, replacing l/shared accesses;
  remove cap from read-plan constructors. Split reads.rs (793 base lines)
  before this change can cross 800: keep_user/keep_pull/keep_row/keep_document/
  page_bound/long/long_body -> reads/keep.rs, about 190 lines, paired with reads.rs
  for the moved count. Only paths/visibility change; expose reads::keep::long
  for the component's diagnostics. No page-size arithmetic moves here.
  Add must_use to newly public pure value-returning functions; no lint exemption.
  The sketched word arguments are prefixed; other arguments/results stay.
  valid_word checks 1..=16 bytes of [a-z0-9-], checked by the owner at start. Marker
  functions use checked `2*key + word.len() + 15`, removing HEAD's fixed word;
  hex stays pub(crate); host joins it in 01.4. The second hard-coded 21
  in the component's limits.rs is temper 01.3/skein 03.1's.
  Replace skein_io::digest with skein_lib::sha256::digest and read its first
  eight bytes big-endian; retain Long's length:u64 BE then updated nanos:u64
  BE, missing updated = 0. No streaming-decoder rewrite in this move.
  temper's side: temper 01.3 repairs component/calls.rs's begin/heard/raw_room
  and reads::long -> reads::keep::long, hooks/connection.rs's hint call and
  limits.rs's marker length. conventions::host and hooks/connection.rs's two
  marker::hex calls use local hex-digit expressions; the former ends at temper
  01.4's host::host switch, the latter at temper 03.1's component-crate deletion.
Test shape: 4 moved tests, unchanged names: a_marker_round_trips_its_key_and_keeps_the_text
  (0.003 s), a_body_without_a_well_formed_marker_has_no_key (0.006 s),
  a_full_table_drops_the_entry_used_longest_ago (0.005 s),
  a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place (0.003 s).
  Adapt word arguments and literal marker lengths only; no new tests here.
  The 68 tests::reads/writes/contents and the page-size test remain for 03.1.
Proof: unit; `cargo nextest run -p skein-forge` passes 90 tests, <1 s;
  base form `cargo nextest run -p temper-engine-protocol-forge` passes 117 in 0.131 s.
  No connected component yet; temper 01.3 connects it, skein 03.1 brings its tests.
Don't: move conventions, Limits/Shared, cost counters or Component, or fix
  skein 10's behaviours here. Keep all plan size_of uses unchanged: reads.rs's
  Branches room and row widths, reads/size.rs (including rows()), contents.rs's
  File/Commit sizes, actions.rs's Status room, reads/done.rs's Comment room;
  skein 02.2 switches them to the measure (decision 24).
Size: about 3,210 lines moved, 300 new; measured 3,206/300, tests included
  (findings/base-skein-01.md, Re-check revision, scratch pairs).
Removes: nothing; temper 01.3 removes these copies and hands its word.

### 01.4 The start reads, Host and credential kinds
Scope: forge.md 4.1, 5.1, 8; forge-connection.md 3.1–3.2.
Needs: skein 01.3's exchange vocabulary; temper 01.3's component on those
  plans, requests and exchange events, so start's fragments name skein.
Files: new src/{credential,host}.rs, forgejo/credential.rs, forgejo/plans/start.rs,
  forgejo/plans/tests.rs; module declarations changed; shared fixtures extended.
  Move component::{start_request,drive_starts}'s mapping and start::Heard;
  conventions::{host,put} -> src/host.rs, its test -> src/tests.rs; no scheduling.
Shape: the owner runs a separate Plan for User then ApiSettings. A non-200
  head ends with Status, preserving retry; a matching decoded document ends
  with User or a positive ApiSettings; zero cap, wrong document, malformed,
  empty/raw body or transport failure ends Unanswered. Head 200/rows wait.
  The component decides identity/stop/backoff/deadline/cost; starts count in no call.
  Emit no second terminal. authorization (#[must_use]) returns Secret:
  `token ` then the secret, checked for overflow.
  The component names Authorization; header-control checks are skein 03.2's.
  host::host(address, name) takes TLS's server name or plaintext's configured
  name, else the IP (IPv6 bracketed). Preserve name validation. Change the
  base's server_name.is_some() default-port choice: omit 80 AND 443 whatever
  transport (answer 1, decision 25); append every other port.
  temper's side: temper 01.4 switches temper-service/src/forge.rs and
  temper/src/forge.rs's conventions::host calls to host::host, removing the
  helper and start mapping. skein 03.1/temper 03.1 removes owner-built Host
  and hands plaintext's name through Transport.
```rust
// skein_forge::host
/// The transport's name, else its IP; neither port 80 nor 443 is appended.
pub fn host(address: core::net::SocketAddr, name: Option<&[u8]>) -> Option<Box<[u8]>>;
// skein_forge::credential
/// Owner-supplied credential; its Debug delegates to Secret's redaction.
pub enum Credential {
    /// Permanent token, until revoked; no renewal plan.
    Token(skein_lib::Secret),
}
// skein_forge::forgejo::credential
pub fn authorization(credential: &Credential) -> Option<skein_lib::Secret>;
// skein_forge::forgejo::plans::start
/// One owner-selected start exchange.
pub enum Read { User, ApiSettings }
/// Send is answered with exchange Heard; Done is terminal for this read.
pub enum Step { Send(Exchange), Done(Heard) }
/// A terminal value for the component's start machine, moved from start.rs.
pub enum Heard {
    /// The credential's user id; owner checks identity.
    User { id: u64 },
    /// A positive cap; owner may begin calls.
    ApiSettings { max_response_items: u32 },
    /// Refusal; owner stops or backs off.
    Status { status: u16, retry: Option<Duration> },
    /// No usable answer; owner backs off.
    Unanswered,
}
impl Plan {
    pub fn new(read: Read) -> Self;
    pub fn begin(&mut self) -> Step;
    pub fn heard(&mut self, heard: super::Heard) -> Option<Step>;
}
```
Test shape: 1 moved, 8 new. src/tests.rs keeps
  host_names_include_only_nondefault_ports_and_bracket_ipv6 (0.002 s). Add
  host_uses_names_and_omits_both_default_ports: IPv4/IPv6, TLS/named plaintext,
  names/None, ports 80/443/3000 -> b"forge.example" or b"127.0.0.1" or
  b"[0:0:0:0:0:0:0:1]", with b":3000" only; include TLS on 80 and plaintext
  on 443 explicitly. Seven new tests in plans/tests.rs reuse fixtures::user and
  shared bounds; only a canned exchange, no world. Each <20 ms:
  `identity_start_sends_user_and_returns_id` -> Request::User, User { id: 7 };
  `page_cap_start_returns_positive_cap` -> ApiSettings request, cap 50;
  `zero_page_cap_is_unanswered` -> Unanswered;
  `start_status_preserves_status_and_retry` -> Status {429, Some(5 s)};
  `bad_start_answer_is_unanswered` -> Unanswered for wrong/malformed bodies
  and Unseen/Seen; feed again -> None;
  `token_authorization_has_the_dialect_prefix` -> b"token secret";
  `credential_debug_contains_no_secret` -> false for search of b"secret" in
  both credential and rendered Secret Debug. Reference:
```rust
// crates/skein-forge/src/forgejo/plans/tests.rs
#[test]
fn zero_page_cap_is_unanswered() {
    use super::{Body, Heard, start};
    use crate::forgejo::document::{ApiSettings, Document};
    let mut plan = start::Plan::new(start::Read::ApiSettings);
    let _sent = plan.begin();
    let ended = plan.heard(Heard::Done(Body::Document(Document::ApiSettings(
        ApiSettings { max_response_items: 0, default_paging_num: 30 },
    ))));
    assert_eq!(ended, Some(start::Step::Done(start::Heard::Unanswered)));
}
```
Proof: unit; `cargo nextest run -p skein-forge` passes 99 tests, <1 s;
  base protocol suite as in 01.3; `cargo clippy -p skein-forge --all-targets -- -D warnings`
  compiles the sketch. No connected start yet: temper 01.4 is its first user.
Don't: implement App credentials, renewal, a clock, a timer or TLS;
  change another user's start verdict; log a rendered header.
Size: about 110 lines moved, 195 new production lines, plus 125 of tests and fixtures (estimated).
Removes: nothing; temper 01.4 removes its start mapping and Host helper.

## Done when

- `cargo nextest run -p skein-forge` prints 99 passed, 0 failed after
  01.4; skein 06.3 and 10 add theirs later; <1 s here.
- `cargo clippy -p skein-forge --all-targets -- -D warnings` exits 0,
  prints Finished and no warning (base equivalent: the three source crates).
- `rg -n -i temper crates/skein-forge` prints nothing (exit 1); the
  source-form run and owner-literal locations are in findings/base-skein-01.md.
