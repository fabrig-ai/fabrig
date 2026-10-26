# Session 01: temper on skein-forge

Put temper's vocabulary, dialect, call-plan and start-plan users on the kit,
then delete their temper copies. Policy, configuration, files and the owner
stay temper's; the bytes temper writes keep its word and credential.

**Needs at start:** skein 01.1's `skein_forge::{api,locate}`; the tools
plan's `temper-01.1`, whose Cargo.lock names smith at or after
`b415f737` (README, sessions and Rules). Each capability must be on origin/main.
The coordinator registers `plan await coordinator tools/temper-01.1`;
this session starts only after that cross-machine landing.

## Read first

- skein `docs/design/forge.md` 2.3, 3, 4.2, 5.1, 6, 8;
  temper `docs/design/protocol/forge.md` 2, 4, 10, 12.
- skein `docs/foundation/programming-model.md` 3, 4.2, 9, 10;
  `testing-strategy.md` 2.1, 2.4–2.5, 8, 10; temper workflow 1–2.
- README (sessions, Rules, Done when); code-organization 1–3, 5–8;
  decisions 2–8, 14, 18, 24–25 and Design questions 1, 14, 34;

- `skein/01-forge.md`, and skein 02.2/03.1's “temper's side”;
  findings `survey-{01-forge,02-client,03-connection,07-pins}.md`,
  `trace-forge.md`, `base-runs.md`, `base-temper-01.md`.

## What the design asks

- forge 2.3 and protocol/forge 2: change paths, preserving temper's
  translations, stored records, connector decisions and configuration.
- forge 3: one vocabulary, with client admission still in the client.
- forge 4.2, 6 and protocol/forge 4: pure plans receive the owner's word;
  the component charges requests, and temper's marker overhead stays 21 bytes.
- forge 5.1, 8: use the dialect's starts, Host helper and typed token;
  scheduling, identity comparison, backoff and deadlines remain component duties.

## Choices this plan makes

Counts and Files below describe the base, dd12b8e. Skein copies temper's
origin/main at each upstream step's start, never this base. Sizes count the
users' path-renamed production lines as moved; removed source belongs in
Removes. The 193 crate-name lines are not the whole vocabulary migration:
there are 652 `client::api` lines including qualified paths, at the base.
Imports split the vocabulary from client state; no forwarding module survives.
The answer-check part of api.rs stays private until temper 02.1 deletes the client.

## Rules that bind it

- Every increment's **first commit** is its repin: `cargo update -p
  skein-lib`, then `cargo check --workspace --all-targets`. Repair in
  that commit all upstream interface/test changes since the previous pin,
  including skein 10 and 02.2 when present. A lock conflict takes origin/main's
  Cargo.lock and repeats the update. Do not merge it by hand.
- No rewrite of a moved part, aliases or re-exports. Every affected use names
  skein's crate. Preserve test names, budgets and unoptimized builds.
  New units <20 ms, focused worlds <0.5 s, fuzzy tests about 5 s.
- Shared hotspots: Cargo.toml/Cargo.lock; temper-service's forge.rs/settings.rs
  in 01.3; the listener's hooks.rs. Follow README's tools-plan ordering.
  Gate Proofs are the gate, `scripts/check.sh`; during the run
  do not run a second gate before land. Planning ran each command once.

## Increments

### 01.1 Vocabulary users

Scope: forge 2.3, 3; repin, use the vocabulary, delete its local definitions.
Needs: the session's start capabilities.
Files: Cargo.toml/Cargo.lock; changed Cargo.toml and src/tests Rust users in
  `crates/temper-{engine-domain-forge-client,engine-domain-forge,engine-domain,
  engine-protocol-forge,engine-smith,protocol-store,service}/`;
  `tests/{client,codecs,conformance,system,engine/domain,engine/forge,
  engine/forge/client,engine/protocol-forge}/`; `tools/forgejo-conformance/`.
  All 15 external client dependents are enumerated in base-temper-01.md.
  Repin-only repairs, if the tools plan left them: worker-protocol-git/src/files.rs
  and client-shell-terminal/tests/terminal.rs (both under crates/temper-).
  The jig repin's store-family name repair also includes
  crates/temper-protocol-store/{src/records/application.rs,
  src/records/application/refusal.rs,src/tests/records.rs,schema/store-v2.schema}.
  Client src/{lib,api,bounds,boundary,calls,domain,facts,keep,outbox,identity}.rs and
  src/tests{.rs,/keep.rs,/outbox.rs,/locate.rs} change imports as needed;
  src/locate.rs deleted. Source destination: skein-forge/src/{api,locate}.rs.
Shape: follow README's first-repin jig repair rule when the lock still
  names jig 719bd70d4: take landed tools jig-02.1 at
  b2eebe6ab91af9eca121588f2d44072fcbdf3b9c in the same repin commit, retaining
  the skein update; run the existing all-target check. Keep an already newer
  compatible jig pin, and refresh derived fixtures reached by that check.
  Apply README's mechanical-repair rule: the authorized jig pin includes
  people_v4, so replace all people_v3 references in those four store files
  with people_v4, including the schema comment. Keep temper's store-v2 family,
  fields and behaviour; the existing record tests retain their assertions.
  Add workspace git dependency skein-forge and direct dependencies to
  vocabulary users, keeping client dependencies where they still use state.
  Replace client::api, qualified client vocabulary paths, and local crate::api
  vocabulary references with `skein_forge::api`; locate with
  `skein_forge::locate`. Remove public api exposure; retain Shape/accepts and
  their helpers privately, importing vocabulary types directly. Do not alias
  those types back into the client. Keep crate::api::accepts in calls.rs and
  its six unit assertions; import vocabulary from skein directly and use fully
  qualified skein paths in lib.rs to avoid its private api module's name.
  Existing byte-counter signatures stay
  until skein 02.2's measure, as skein 01.1's Shape and decision 4 say.
Test shape: **0 tests move in this temper increment**; 76 client tests at the
  base stay, including the two tests::locate Harness tests. Skein 01.1 adds
  its own two pure locate tests. All existing callers' assertions and stored
  bytes remain; no new test. Count source tests with `cargo nextest list
  -p temper-engine-domain-forge-client -p temper-forge-forgejo
  -p temper-engine-protocol-forge`: 76/84/117 at the base.
Proof: joint; `cargo nextest run -p temper-engine-domain-forge-client
  -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-smith
  -p temper-protocol-store -p temper-service -p temper-engine-forge-client-world
  -p temper-engine-forge-world -p temper-engine-protocol-forge
  -p temper-engine-protocol-forge-world -p temper-engine-domain-world
  -p temper-conformance-world -p temper-codecs-world -p temper-client-world
  -p temper-system-world -p forgejo-conformance`: origin/main's count at the
  step's start passed, the same identities through the cutover (997 at the
  base, 5.821 s), <15 s.
Proof: joint; `scripts/check.sh`: formatting/clippy clean,
  origin/main's focused and fuzzy counts at the step's start passed, unchanged
  (1879 / 50 at the base, 7.184 / 25.879 s); budgets 15/60 s.
Don't: move EffectPurpose, change durable bytes, remove admission, or delete
  the client crate. No client-module alias for the vocabulary.
Size: about 400 lines moved, 60 new.
Removes: api.rs's vocabulary through Collaborator (about 417 lines) and
  locate.rs (35 lines), their public declarations; no client answer check.

### 01.2 Dialect users

Scope: forge's Forgejo dialect; repin and replace temper's dialect crate.
Needs: skein 01.2's `skein_forge::forgejo`, with its documents, codecs and pin.
Files: Cargo.toml/Cargo.lock; source and Cargo.toml in `crates/{temper,
  temper-forge-setup-shell,temper-engine-protocol-forge,temper-forge-setup}`,
  `testing/{temper-fake-forge-protocol,temper-throwaway-forgejo}`,
  `tests/{codecs,engine/protocol-forge,forge-setup/reconcile}`.
  Also tools/forgejo-conformance/{Cargo.toml,src/runner.rs,src/registration.rs,
  src/tests.rs}, crates/temper/tests/{process.rs,support/owned.rs} and
  testing/temper-throwaway-forgejo/tests/throwaway.rs: indirect pin users.
  tools/{git-transcripts,forgejo-conformance}/README.md: the pin sentence
  names `skein_forge::forgejo::pin` (one line each).
  Delete `crates/temper-forge-forgejo/`; its destination is
  skein-forge/src/forgejo/. The fake is a tools-plan hotspot.
Shape: replace each dialect import/path with `skein_forge::forgejo`, including
  fully qualified paths, setup request/answer types, document limits, delivery
  headers, version constants and tests. Base Rust-line counts in the Files
  order: 2, 7, 48, 30, 38, 2, 40, 1, 12 (180 in all).
  Remove its workspace dependency and all nine dependent manifest entries;
  add skein-forge where absent. Delete the explicit
  `"crates/temper-forge-forgejo"` workspace member. Remove the throwaway's
  re-export of VERSION/RUNNER_VERSION/names_runner; its seven indirect user
  lines import skein_forge::forgejo::pin directly. No substitute crate/module.
Test shape: **84 copied dialect tests now leave temper**, names unchanged in
  skein 01.2 (base list: root 37, accounts 9, delivery 15, hooks 7, orgs 7,
  pin 4, tokens 5; base-runs 0.062 s total). Zero new tests; all caller tests
  stay. Codecs' 55 focused/3 fuzzy dialect tests wait until temper 07.
Proof: joint; `cargo nextest run -p temper -p temper-forge-setup-shell
  -p temper-fake-forge-protocol -p temper-throwaway-forgejo
  -p temper-codecs-world -p temper-forge-setup-reconcile-world
  -p temper-engine-protocol-forge -p temper-forge-setup -p forgejo-conformance`:
  origin/main's count at the step's start passed, unchanged, with its ignored
  throwaway test (471 and 1 at the base, 0.673 s); same caller identities, <15 s.
Proof: joint; `scripts/check.sh`: origin/main's focused count at
  the step's start less 84, the unit tests that leave, and its fuzzy count
  passed (1795 / 50 from the base's 1879 / 50), <15 s / <60 s; no budget
  or ignore changes.
Don't: move reconcile, fakes, transcripts or codec tests yet; change pin
  constants, JSON, request targets, response codes or the setup's values.
Size: about 180 lines moved, 35 new.
Removes: the whole 9,300-line dialect crate (about 7,820 production lines
  and 1,478 test lines), its dependency entries and workspace membership.

### 01.3 Component on the call plans and temper's word

Scope: forge 4.2, 6; repin, call skein's plans, hand temper's word.
Needs: skein 01.3's plans, Bounds, exchange vocabulary, tables and marker.
Files: component src/{lib,component,component/calls,connection,limits,
  settings,conventions,hooks,hooks/connection,tests,tests/fixtures,
  tests/connection,tests/reads,tests/writes,tests/contents,tests/hooks}.rs;
  temper-service src/{forge,settings,tests/forge}.rs; manifests/Cargo.lock;
  docs/examples/owned.json; crates/temper/tests/startup.rs (existing proof).
  Change direct Settings builders in component src/tests/fixtures.rs,
  tests/engine/protocol-forge/src/fixtures.rs and
  tools/forgejo-conformance/src/component/hosted.rs.
  Delete component src/{reads,reads/done,reads/size,reading,writes,contents,
  actions,plan,tables,failure,marker,hooks/hint}.rs after their upstream copy
  to skein-forge/src/forgejo/plans/ (marker to src/marker.rs).
  revision is reading::revision, not another source file at the base.
Shape: import `skein_forge::forgejo::plans::{Body,Fields,Heard,Failure}`
  directly; `plans::plan::{Plan,Step,Bounds}` and `plans::tables` replace
  component copies. Plan::new drops cap; begin/heard get Bounds and word;
  raw_room gets answer_bytes. Derive page_size with existing Listing/page_size:
  PullFor uses Items, Checks Statuses, PullFiles PullFiles; every other listing
  read passes its own Listing (reads.rs 152-213, reads/done.rs 265-272); single
  reads use 1.
  Bounds mirrors page_size:u32, rows:u32, answer_bytes:u32, body_bytes:u32.
  Preserve cost accounting at written && !charged; Done carries only result.
  Add the owned word to Settings; service hands b"temper", as do every fixture
  and conformance builder. Use settings.rs's existing Box and List imports
  in these boundaries. The complete changed boundary (01.4 replaces Forge):
```rust
// temper_engine_protocol_forge::settings
/// Owner inputs held until the component closes.
pub struct Settings {
    pub forges: List<Forge>,
    pub webhooks: Option<crate::hooks::Webhooks>,
    pub word: Box<[u8]>,
}
```
  Hooks::drive and Hooked::pump take &Settings instead of &Webhooks, so they
  borrow the same word; call's secret lookup uses settings.webhooks.
  Hooked::verified(env: &Env<Limits>, word: &[u8]) passes it to
  `plans::hint::hint(forge, word, delivery)`. Keep response-before-hint order.
  Replace marker's 21 in limits::check by checked marker::length(word, op_bytes).
  Price the held word (at most 16 bytes) in worst_case. After the complete
  cutover, recompute docs/examples/owned.json's engine.memory_bytes through
  temper-service/src/system.rs::startup_worst_case at the existing startup
  fixture's bounds. The ready check's word-only sketch needed 32 bytes more
  (16 inline and 16 held); measure the complete cutover rather than adopting
  that sketch's literal. Refresh further copies and derived values found,
  listing them in the commit body. Keep service's shipped
  temper body minimum 2*op_bytes+21. No new word refusal before skein 03.1.
  marker::hex stays private upstream: conventions::host and hook signature
  rendering use the same indexed hexadecimal digit expression locally.
  temper's side of skein 01.3, repaired here: component/calls.rs:238's
  `reads::long` becomes `skein_forge::forgejo::plans::reads::keep::long`;
  limits.rs:424-425's `size_of::<crate::reads::Plan>` and `writes::Plan`, and
  :383's `tables::{RepositoryEntry,UserEntry}`, name skein-forge's plans and
  tables; src/tests/hooks.rs's three `crate::hooks::hint::hint` calls (base
  lines 66, 128, 134) become `plans::hint::hint(1, b"temper", &delivery)`.
  Remove the four upstream-copied marker/table tests; retarget remaining tests.
Test shape: **4 copied tests leave temper** (base list names below);
  113 component tests remain, including the 68 Harness plan tests:
  a_marker_round_trips_its_key_and_keeps_the_text (0.003 s),
  a_body_without_a_well_formed_marker_has_no_key (0.006 s),
  a_full_table_drops_the_entry_used_longest_ago (0.005 s),
  a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place (0.003 s),
  times from base-skein-01; their names survive in skein. Add one owner unit
  in temper-service/src/tests/forge.rs, <20 ms, using existing fixtures:
```rust
// crates/temper-service/src/tests/forge.rs
#[test]
fn temper_word_keeps_the_marker_bytes_and_length() {
    let config = fixtures::engine();
    let mut found = jig_protocol_config::Problems::empty();
    let settings = forge::settings(
        &config.forges, fixtures::forge_runtime(), None, &mut found,
    ).expect("fixture settings");
    assert_eq!(settings.word.as_ref(), b"temper");
    assert_eq!(skein_forge::marker::length(&settings.word, 2), Some(25));
    let encoded = skein_forge::marker::encode(&settings.word, &[0xab, 1], b"", 25);
    assert_eq!(encoded.as_deref(), Some(b"<!-- temper:key ab01 -->\n".as_slice()));
}
```
  No new world. The fixture uses the shipped Small profile. Existing
  creations_carry_the_key_marker_at_the_head_of_their_body proves component
  wiring over HTTP with b"<!-- temper:key abff -->\\nbody" and cost 1.
Proof: joint; `cargo nextest run -p temper-engine-protocol-forge
  -p temper-service -p temper-engine-protocol-forge-world`: origin/main's count
  at the step's start less 3, four deletions and the owner unit (373 from the
  base's 376, 1.600 s); <15 s.
Proof: world; `cargo nextest run -p temper-engine-protocol-forge-world
  --profile fuzzy`: 6 passed in 7.027 s here (7.709 s in base-runs).
  Four fuzzy_machine tests exceed the 5 s aim at the base; preserve them
  and report their streaming-cut cost, with no filtering; suite <60 s.
Proof: joint; `scripts/check.sh`: origin/main's focused count at
  the step's start less 3 (four component tests leave, the owner unit
  enters) and its fuzzy count passed (1792 / 50 after 01.2's 1795 / 50 on
  the base), <15 s / <60 s.
Proof: joint; `cargo nextest run -p temper --test startup
  -E 'test(=serve_prepares_once_on_the_owned_example)'`: the existing
  owned-example startup regression passes once after the complete cutover,
  within the unchanged focused budget. The process proof follows 01.4's
  changed starts.
Don't: change call outcomes, page-size arithmetic, retry policy or user agent;
  duplicate Bounds/Heard/Body, move the component or its 68 Harness tests.
Size: about 90 lines moved, 150 new.
Removes: about 3,500 production lines copied upstream, exchange definitions
  in connection.rs, module declarations and four copied tests; no scheduler.

### 01.4 Component on the start plans, credential and Host

Scope: forge 4.1, 5.1, 8; repin, consume the dialect's two start plans.
Needs: skein 01.4's start::{Plan,Read,Step,Heard}, credential and host.
Files: component src/{component,connection,start,settings,limits,conventions,
  tests,tests/fixtures,tests/connection}.rs; temper-service src/forge.rs;
  crates/temper/src/forge.rs; tests/engine/protocol-forge/src/fixtures.rs;
  tools/forgejo-conformance/src/component/hosted.rs; manifests/Cargo.lock;
  docs/examples/owned.json; crates/temper/tests/startup.rs (existing proof).
Shape: replace start_request and drive_starts's request/document mapping
  with `skein_forge::forgejo::plans::start` Plan::new(Read::User/ApiSettings),
  begin and heard. Keep only the component's scheduling loop: bounded io,
  close/drain, wake, deadlines, identity check and backoff. Skein's plan
  does not drive sockets or time. One optional start plan per configured forge
  is created for each attempt; Done retires it before another attempt.
  Price these holders in worst_case. Recompute the owned example's
  engine.memory_bytes through system::startup_worst_case after these changes,
  as 01.3 does; the existing joint proof below includes
  serve_prepares_once_on_the_owned_example. Refresh further copies and derived
  values found, listing them in the commit body. Delete local start::Heard and import
  skein's; start::heard keeps (Start, Heard, user:u64, now:Time, &Limits)->Start.
  Starts remain outside call costs.
  Replace Forge.token with the upstream Credential; complete boundary:
```rust
// temper_engine_protocol_forge::settings
/// One configured forge; credential held until close.
pub struct Forge {
    pub number: u16,
    pub kind: Kind,
    pub address: skein_io::Addr,
    pub transport: Transport,
    pub host: Box<[u8]>,
    pub credential: skein_forge::credential::Credential,
    pub user: u64,
}
```
  Service constructs Credential::Token(forge.api_token.clone()); fixture and
  conformance builders wrap their existing secrets. connection::start uses
  forgejo::credential::authorization; limits inspect the Token's raw length,
  preserving current request-head arithmetic until skein 03.2.
  Replace conventions::host calls by `skein_forge::host::host(address,name)`:
  service 1, binary 1, component test 1, plus its settings doc comment at the base.
  Omit both 80 and 443, as answer 1/decision 25 says; preserve all other bytes.
  The owner still hands Host until temper 03.1, which removes that field and
  carries plaintext's configured name in Transport. USER_AGENT stays here.
Test shape: **1 copied pure Host test leaves temper** for skein 01.4,
  keeping host_names_include_only_nondefault_ports_and_bracket_ipv6 (0.002 s
  in base-skein-01); skein owns its new TLS-80/plaintext-443 edge cases.
  No new test, 112 component tests remain (skein 03.1 adds Host wiring).
  Retain the_start_reads_user_then_settings_on_one_connection,
  the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise,
  each_start_read_keeps_its_deadline_through_connect_and_starts_afresh:
  user then cap, one connection, same Stopped/Waiting/Ready literals.
  temper-service's a_refused_start_check_stops_the_engine_naming_the_forge
  still reports forge 1 / User / 401.
Proof: joint; `cargo nextest run -p temper -p temper-forge-setup-shell
  -p temper-service -p temper-engine-protocol-forge
  -p temper-engine-protocol-forge-world -p forgejo-conformance`: origin/main's
  count at the step's start less 1, the copied test that leaves (479 from
  01.3's 480; 483 at the base, 2.934 s); <15 s. Retain the fuzzy command of 01.3 when these start paths change.
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh` (README Rules): its count at the step's
  start passed (19 at the base), <5 minutes; 145.694 s at the base, alone
  (findings/base-runs.md). Once here,
  covering the session's starts, authenticated polling, writes and webhooks.
Proof: joint; `scripts/check.sh`: origin/main's focused count at
  the step's start less 1 and its fuzzy count (1791 / 50 after 01.3's 1792 /
  50 on the base) passed, clean clippy, focused <15 s/fuzzy <60 s. Signature sketches compile with
  `cargo clippy -p temper-engine-protocol-forge --all-targets -- -D warnings`;
  scratch source/limits and the base repin failure are recorded in findings.
Don't: delete scheduling with the extracted mapping; hand Host differently
  before temper 03.1; implement renewal or change token/user-agent admission.
Size: about 40 lines moved, 220 new.
Removes: start_request, drive_starts's dialect mapping, start::Heard,
  conventions::{host,put} (about 110 source lines); no clock/backoff machinery.

## Done when

- `cargo check --workspace --all-targets` prints Finished, exits 0.
  The session's gate Proof is `scripts/check.sh`: no warnings
  or failures, focused <15 s and fuzzy <60 s; land owns that run.
- `rg -n 'temper_forge_forgejo|temper-forge-forgejo|temper_engine_domain_forge_client::api|client::api|conventions::host' Cargo.toml crates testing tests tools -g '*.rs' -g Cargo.toml`
  prints nothing (exit 1); it checks current source, including surviving users
  after temper 02/03/07 delete their copies, without naming deleted test files.
- `python3 -c 'from pathlib import Path; print("dialect_copy_absent=" + str(not Path("crates/temper-forge-forgejo").exists()))'`
  prints `dialect_copy_absent=True`. At the base it prints False, as recorded.

