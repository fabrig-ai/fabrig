# Session 03: temper on `skein-forge-connection`

Put temper on skein's forge component and delete temper's copy. 03.1 repins, moves the component's
users onto `skein_forge_connection`, hands it temper's values (the word `temper`, the user agent
`temper/0.0.0`, a plaintext forge's name, the limits as plain numbers), maps its two new refusals and
deletes `crates/temper-engine-protocol-forge`. 03.2 repins onto skein 03.2 and 03.3 (the head check,
the page sizes).

**Needs at start:** **skein 03.1** and **temper 02.1** on `origin/main` (README), for 03.1;
**03.2 needs skein 03.3**, which needs skein 03.2 (as the README row says). Temper
01.3 and 01.4 have landed (they took the marker, the start plans and `host` from the crate).

## Read first

- skein `docs/design/forge-connection.md` 2, 3, 7 and 9; temper `docs/design/protocol/forge.md`; skein
  `docs/foundation/testing-strategy.md` 8; `development.md` 5.2 to 5.7; README (Rules, Done when 5 to
  7); `code-organization.md` 1 to 3 and 5 to 8; `decisions.md` 25; Design questions 1 to 6,
  13, 14, 17, 34 and 36 (all answered).
- `skein/03-connection.md` (03.1's shapes and its "Temper's side", and 03.2's and 03.3's); findings
  `survey-03-connection.md`, `trace-connection.md`, `base-runs.md`, `base-temper-03.md`.

## What the design asks

- forge-connection 2 and 3: the owner hands the word, the user agent, the credential and the
  transport (plaintext's optional name, TLS's server name); the kit derives Host (decision 25: the
  name, else the address, `:port` only when neither 80 nor 443) and checks the word and the name
  before any io. The client's numbers arrive as plain numbers: no dependency between the client and
  the component.
- 7 and 9: the owner maps each `Unusable` to the configuration's members, and a refusal naming a
  forge is indexed to the configured forge.

## Choices this plan makes

- Counts are **at the base** (temper dd12b8e, skein 064c55f: skein 03.1 is not on `origin/main`,
  so nothing here compiled against `skein_forge_connection`; sketches ran with stand-ins; the runs are
  in `findings/base-temper-03.md`). A moved line is copied from temper's `origin/main` at the step's
  start, never from the base.
- The mapping of `Word` and `PlaintextName` lands in 03.1: `problems` is an exhaustive match, so the
  repin does not compile without the arms (skein 03.1's "Temper's side"). What is left for later is
  the repin onto skein 03.2 and 03.3 (03.2). If both are on `main` at 03.1's repin, 03.1's second
  commit makes their repairs and 03.2 keeps its tests and its arm.
- temper-service owns `pub const USER_AGENT: &str = "temper/0.0.0"` and a private
  `WORD: &[u8] = b"temper"`. The world, the codecs and the conformance hand fixture literals.
- `ForgeRuntime.transport` is the kit's `Transport`; the shell hands `Plaintext { name: None }` and
  `settings()` fills the name from the forge's `api`. A TLS forge's Host now comes from its server
  name (the base chose by `server_name.is_some()`): equal for every fixture; skein 01.4's `host`
  tests pin the server name's case, and the start test below the plaintext one.
- The setup shell keeps its own `reconcile::Settings` with `host` and `user_agent`: `reconcile_settings`
  keeps `skein_forge::host::host(address, None)` (01.4), takes `temper_service::forge::USER_AGENT` and
  `skein_forge_connection::limits::{http, documents}`; of 03.1 and 04.1 the second keeps the
  first's lines.

## Rules that bind it

- The repin is the first commit of 03.1 and of 03.2: `cargo update -p skein-lib`, then
  `cargo check --workspace --all-targets`; it repairs what skein's steps since temper's last
  repin broke. A `Cargo.lock` conflict takes `origin/main`'s and runs the update again, never a hand
  merge. temper-service's `forge.rs` and `settings.rs` are not named hotspots (README): 03.1 and
  03.2 edit them minimally and rebase onto the tools plan's changes.
- No behaviour change but a TLS forge's Host, no raised bound, no loosened assertion; moved tests keep
  names. Units under 20 ms, focused worlds under 0.5 s, unoptimized builds.
- Gate counts do not chain with the sibling files', so Proofs state deltas; the base is 1879 focused
  (6.668 s) and 50 fuzzy. The process tier (19 tests, 145.694 s at the base) failed twice at the base,
  in different tests, each passing alone: rerun a failure once alone, never count it as a pass. The
  gate lane takes a committed tree only, so recordings run committed.
- Don't (code-organization 7): rewrite a moved part; name temper in skein; re-export or alias; depend
  client-on-connection; use serde in skein; change a hotspot outside README Rules; raise a budget,
  filter, ignore or split a test, or optimize a test build; merge `Cargo.lock` by hand; edit a
  transcript, a delivery or `facts.txt`.

## Increments

### 03.1 The repin, the component's users, the deletion

Scope: forge-connection 2, 3 and 7. What the crate's tests proved of temper's values (the Host, the
  agent, the small profile's body room, the shared numbers) and of the two mappings becomes five tests.
Needs: skein 03.1; temper 02.1 (it repaired the measure callers and `bind_deployment`: the repin
  repairs none of them).
Files (from -> to): `crates/temper-engine-protocol-forge/` -> `skein-forge-connection`, deleted here.
  Root `Cargo.toml` (the member and the `path` entry out, a git entry for `skein-forge-connection`
  in) and `Cargo.lock`. Manifests, each `temper-engine-protocol-forge.workspace = true` becoming
  `skein-forge-connection.workspace = true`: `crates/{temper-service,temper,temper-forge-setup-shell}`,
  `tools/forgejo-conformance`, `tests/{codecs,one-process,engine/protocol-forge}` (the world, which
  stays until temper 07.1). Changed Rust at the base: temper-service
  `src/{forge,settings,lib,stop,route,limits,engine,system}.rs`,
  `src/startup.rs::startup`, `src/startup/application.rs::{Resolved,resolve}`
  and `src/tests/{forge,fixtures}.rs`,
  `src/tests/fixtures/forge.rs`; `crates/temper/src/{forge,lib,host}.rs`; the shell's
  `tests/support/mod.rs`; `tests/codecs/{src/forgejo.rs,tests/{fuzzy_forgejo,forgejo_deliveries}.rs}`;
  the conformance's `src/{component.rs,component/hosted.rs}`; `tests/one-process/src/fixtures.rs`;
  the world's `src/{protocol,machine,joint,hooks,fixtures}.rs` and `tests/{machine,hooks}.rs`.
  `tests/forge-setup/reconcile` names only the world: no changed line.
Shape: three commits, each builds and passes its crates' tests.
  1. **The repin** (README Rules). The crate still builds beside the kit.
  2. **The users**: every path becomes `skein_forge_connection::` (88 alias-using lines in six files
     and 25 direct paths at the base; the 6 `as` aliases, `kit` and `forge`, go). Temper hands:
     `Settings { forges, webhooks, word: Box::from(WORD), user_agent: Box::from(USER_AGENT.as_bytes()) }`;
     `Forge { number, kind, address, transport, credential: Credential::Token(forge.api_token.clone()),
     user }` with no `host`; `Shared` flat
     (`calls`, `rows`, `answer_bytes`, `op_bytes`, `lifetime` from `limits.forge.client`, then
     `name_bytes`, `repositories`, `io: *io`). `settings()` takes the plaintext name from `forge.api`:
     `Address::Host { name }` is `Some(name)`, `Addr::Ip` is `None`, `InProcess` stays `What::Plaintext`;
     the base's `What::Bound` push for a missing host is the `PlaintextName` arm now. `small()`'s
     `body_bytes` is `16_384.max(skein_forge::marker::length(WORD, op_bytes).unwrap_or(u32::MAX))`,
     the base's saturation (`2 * op_bytes + 21` for `temper`: the same numbers). `Webhooks` is named
     `skein_forge_connection::hooks::Webhooks`: the `pub use` in `settings.rs` goes; the three
     occurrences in `startup/application.rs::{Resolved,resolve}` and `forge.rs::settings` name the path.
     Before consuming the component Settings, compute checked `settings::owned_bytes(&settings)`
     and thread the retained-owner byte count through its landed `limits::worst_case` signature:
     temper-service `system::{Service::new,startup_worst_case,memory}`, the separate
     `startup.rs::startup` pre-admission caller, and the conformance's
     `component/hosted.rs::Hosted::new`. Follow skein 03.1's landed signature; retain checked
     overflow/refusal handling and separate TLS-root accounting. The pre-admission sum uses
     the same actual retained DNS/plaintext/owner bytes as Settings construction, counted once;
     borrowing or factoring construction must not allocate or price a second retained copy.
     Refresh each reached startup or
     memory fixture and further derived copy, report the complete sum and list changed users in the
     commit body. Historical sums 2,812,325 and 2,812,377 are not the new owner-byte total.
     `problems` gains
     `Unusable::Zero | Unusable::Word => &[b"forge_component"]` and the `PlaintextName { forge }`
     arm (pushes `forges[i].api` as `What::Bound`, returns); `indexed` gains `PlaintextName`.
     Fixture builders: temper-service `tests/{forge,fixtures,fixtures/forge}.rs`,
     `crates/temper/src/lib.rs::forges`, one-process `fixtures::forges`, the conformance's `hosted::new`, the world's
     `settings()`: `Plaintext { name: None }`, no `host`, `word: temper`, `user_agent: temper/0.0.0`.
     The setup-shell `tests/support/mod.rs::settings` retains its reconcile Settings, including
     `host` and the fixture agent `test`; only its component-limits paths change.
  3. **The deletion**: the crate's directory, its member, its `path` entry, its manifest lines.
Test shape: **no test of the crate stays** (112 at the step's start, 117 at the base, 0.099 s; skein
  03.1 owns them). Five new tests in temper-service `tests/forge.rs`, each <20 ms:
  - `the_component_s_new_refusals_are_problems_naming_their_members`: rows `(Unusable::Word,
    b"application.limits.overrides.forge_component: past its bound\n")` and
    `(Unusable::PlaintextName { forge: 1 }, b"application.forges[1].api: past its bound\n")`.
  - `a_refusal_naming_a_forge_is_indexed_to_the_configured_forge`: `PlaintextName { forge:
    config.forges[0].number }` through `indexed` names `application.forges[0].api` (the number differs
    from the index;03.2 gives that fixture contract its own test).
  - `the_start_request_carries_the_user_agent_and_the_host` (reference; world: `fixtures::started()`
    with the `ForgeFace`, whose accessor `sent(&self) -> &[u8]` is three new lines):
    ```rust
    let mut service = fixtures::started();
    let mut face = fixtures::ForgeFace::new();
    face.connected(&mut service);
    assert_eq!(face.sent(), b"GET /api/v1/user HTTP/1.1\r\nHost: forge.example:3000\r\n\
        Authorization: token api-token\r\nUser-Agent: temper/0.0.0\r\n\r\n");
    ```
  - `the_small_profile_keeps_the_body_room_the_word_derives`: `op_bytes` 100 gives `body_bytes`
    16_384 and 10_000 gives 20_021 (`2 * 10_000 + 6 + 15`), through `forge::small`.
  - `the_shared_numbers_are_the_clients_plain_numbers`: the eight flat fields equal `limits.forge.client`'s
    five numbers, `name_bytes`, `repositories` and the io limits.
  The owner unit `temper_word_keeps_the_marker_bytes_and_length` (01.3) stays; existing tests change
  `use`s and construction only. A profile, limit or size test that fails after the repin is a question.
Proof: unit; `cargo nextest run -p temper-service tests::forge`: 16 passed, origin/main's 11
  (the base's 10 and 01.3's owner unit; 0.028 s for the 10) plus 5.
Proof: joint; `cargo nextest run -p temper-service -p temper -p temper-forge-setup-shell
  -p temper-codecs-world -p temper-one-process-world -p temper-engine-protocol-forge-world
  -p forgejo-conformance`: the set's count at the step's start plus 5 (435 at the base, 2.325 s), <15 s.
Proof: joint; the same command with `--profile fuzzy` on `-p temper-engine-protocol-forge-world`:
  origin/main's count at the step's start, unchanged (6 at the base, 6.185 s), <60 s.
Proof: world; `cargo nextest run -p temper-forge-setup-reconcile-world -p temper-system-world`:
  origin/main's count at the step's start, unchanged (37 at the base, 0.248 s).
Proof: process; `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh`: its count at the step's start passed (19 at the base),
  <5 minutes (145.694 s at the base). Once.
Proof: process; the component recording (03.1 changes `component/hosted.rs`), with `T` a fresh
  directory under `$WS/tmp` and the two forge variables above: `cargo run -p
  forgejo-conformance -- --component --scratch $T/component --out $T/out`, exit 0, about 30 s (28 s
  alone at the base). Once.
Proof: joint; `scripts/check.sh`: origin/main's focused count at the step's start less
  107 (the crate's 112 out, 5 in; the base's 1879 still holds the crate's 117), fuzzy count unchanged
  (50), clippy and format clean, focused <15 s (6.668 s at the base), fuzzy <60 s.
Don't: keep `USER_AGENT` or `conventions` but in temper-service; hand a Host; alias or re-export
  (not `Webhooks`: every `use` names `skein_forge_connection`); touch the world's tests beyond `use`s
  and construction; change a bound or an expectation to pass; write `temper` into skein.
Size: about 170 lines moved (changed callers counted as moved: 88 alias-using lines, 25 direct
  paths, 14 in the world, about 30 of `Settings` and `Forge` construction), about 150 new (the five
  tests about 110, `settings`, `small`, `shared` and the arms about 30, `sent` 3, manifest 3).
Removes: `crates/temper-engine-protocol-forge/`: 6,365 source, 3,178 test and 23 manifest lines in 32
  files at the base (less what temper 01.3 and 01.4 took); its 112 tests live in skein.

### 03.2 The repin onto skein 03.2 and 03.3

Scope: forge-connection 3.1, 7.3 and 7.4: the head check counts the user agent, page sizes are priced
  by the measure; temper explains the refusal.
Needs: skein 03.3 (and so 03.2); 03.1.
Files: `Cargo.lock`; temper-service `src/forge.rs` (`problems`' `Request` arm, `indexed`) and
  `src/tests/forge.rs`; what the repin's run shows broken among `forge::{small, shared, prepare}`, the
  fixtures of 03.1 and the conformance's `component/hosted.rs` (skein 03.2 and 03.3 name them).
Compatibility at this repin: rebase browser-1's landed `qa-401.3`
  (`eca69f1f36cb2f28eefa54c7b936cac53e8a2607`), which takes Jig's `qa-401.2`
  (`f407ba4501064b06fc03041f20173bc89696bca9`). Keep its web host's status
  forwarding in `crates/temper-client-shell-web/src/host.rs`, its charter
  translation in `crates/temper-engine-domain/src/route/asks.rs`, the
  `tests/engine/domain/src/walking.rs` authority fixture, browser regression
  and derived `docs/examples/owned.json`. Retain the newer compatible Skein
  source this step needs; resolve Cargo source conflicts from main and rerun
  this step's repin, keeping one revision per source. Recompute reached complete
  memory sums at unchanged inputs, never copy a historical delta. Refresh
  further reached copies and list users in the commit body; all proofs remain.
Shape: two commits. 1. **The repin**: update and check, then the page-size and head expectations
  repaired to skein's measure, no limit raised. 2. **The arm**: `Unusable::Request { forge }` pushes
  `forges[i].api_token` as `What::Bound` and names `forge_component.target_bytes`, `.token_bytes`,
  `.tls.send` and `io.output` (the base named the first three, without the forge); `indexed` gains
  `Request { forge }`. Answer 2 adds no variant; answer 3 changes no temper line (no temper test pins
  a malformed or uppercase signature).
Test shape: three new units, each <20 ms (the two original units took0.007 and0.010s in the scratch):
  - `the_forge_fixture_number_differs_from_its_index`: the same `fixtures::engine()`
    setup and the existing `assert_ne!(config.forges[0].number, 0)`, moved from
    `a_refusal_naming_a_forge_is_indexed_to_the_configured_forge`. Keep the latter
    test's input and indexed-problem assertion unchanged. This separates the
    fixture contract under testing-strategy.md10, The setup the assertion reads;
    both assertions remain, with no replacement value or setup change.
  - `a_head_past_the_request_room_is_a_problem_naming_its_members_and_the_forge`: `Request { forge:
    config.forges[0].number }` through `indexed` and `problems` gives `HEAD_PROBLEMS` (number not 0).
  - `the_longest_head_of_the_shipped_forge_fits_its_room_to_the_byte`: `forge::prepare` with
    `forge_component.target_bytes` 1_048_345 is `Ok`, with 1_048_346 gives `HEAD_PROBLEMS`.
  ```rust
  const HEAD_PROBLEMS: &[u8] = b"application.forges[0].api_token: past its bound\n\
      application.limits.overrides.forge_component.target_bytes: past its bound\n\
      application.limits.overrides.forge_component.token_bytes: past its bound\n\
      application.limits.overrides.forge_component.tls.send: past its bound\n\
      application.limits.overrides.io.output: past its bound\n";
  ```
  The edge is `io.output` less the head's fixed 192, Host `forge.example:3000` (18), token `api-token`
  (9) and the agent (12): 1_048_576 - 231 = 1_048_345. The scratch ran the base head, without the
  agent, at 1_048_357 and 1_048_358; if skein 03.2's head differs, return the question. The shipped
  profile's maximum Host is 253 name bytes + one colon + five port digits = 259 bytes. Its worst
  head is 4096 + 259 + 512 + 12 + 192 = 5071, under `tls.send` 16_384 and
  `io.output` 1_048_576 (the landed Host helper and arithmetic passed a scratch oracle; future
  component admission remains unproved).
Proof: unit; `cargo nextest run -p temper-service tests::forge`: 19 passed (16 at03.1, plus3; the added fixture-contract unit is included).
Proof: joint; the users command of 03.1: its count at the step's start plus3, <15 s.
Proof: process; 03.1's component recording: exit 0, about 30 s. Once.
Proof: process; 03.1's command: its count at the step's start passed (19 at the base), <5 minutes. Once.
Proof: joint; `scripts/check.sh`: focused plus3, fuzzy unchanged, clean.
Don't: raise a bound, drop a row or loosen an assertion to pass; add an arm for answer 2.
Size: about 20 lines moved (the repin's repairs, as many as the run shows), about 55 new (the arm and
  `indexed` about 12, the tests and the literal about 43).
Removes: nothing.

## Interface sketches

Compiled in the scratch (clippy `-D warnings`, format clean): `problems` and `indexed` with stand-ins
for the two new variants. The rest cannot compile before skein 03.1 lands.

```rust
// crates/temper-service/src/forge.rs
/// The user agent every forge request carries; a test pins it.
pub const USER_AGENT: &str = "temper/0.0.0";
/// A forge's resolved endpoint and its transport, supplied by the shell.
pub struct ForgeRuntime {
    pub number: u16, pub address: core::net::SocketAddr,
    pub transport: skein_forge_connection::settings::Transport,
    pub trust_roots: u32, pub trust_root_bytes: u32,
}
pub fn settings(forges: &[settings::Forge], runtime: List<ForgeRuntime>,
    webhooks: Option<skein_forge_connection::hooks::Webhooks>, found: &mut Problems)
    -> Option<skein_forge_connection::settings::Settings>;
pub fn shared(limits: &limits::Limits, io: &skein_io::Limits) -> skein_forge_connection::Shared;
pub fn small(forge: &forge::Limits) -> skein_forge_connection::Limits;
pub fn problems(unusable: skein_forge_connection::Unusable, found: &mut Problems);
pub(crate) fn indexed(unusable: Unusable, forges: &[settings::Forge]) -> Unusable;
```

temper-service owns the word, the agent, the mapping and the indexing; the kit owns Host, the checks
and the head's size.

## Questions to raise

- A `Request` or `PageSize` refusal of the shipped profile at 03.2's repin, or a head that differs
  from 1_048_345: change no bound; return it.
- A process-tier failure that persists on a rerun alone (03.1, 03.2).

## Done when

- `cargo check --workspace --all-targets` prints `Finished`, exit 0.
- `rg -n 'temper_engine_protocol_forge\b|temper-engine-protocol-forge($|[^-])' Cargo.toml
  Cargo.lock crates testing tests tools -g '*.rs' -g Cargo.toml -g Cargo.lock` prints nothing; at the
  base 63 lines in 38 files.
- `rg -n '(temper_engine_protocol_forge|skein_forge_connection)( as |::\{.* as )' crates testing
  tests tools -g '*.rs'` prints nothing; at the base 6 lines.
- `rg -n 'conventions::(host|USER_AGENT)' crates tests tools -g '*.rs'` prints nothing (6 lines
  at the base), and `rg -c --sort path 'temper/0\.0\.0' crates -g '*.rs'` prints
  `crates/temper-service/src/forge.rs:1` and `crates/temper-service/src/tests/forge.rs:1`.
- `python3 -I -c 'from pathlib import Path; print("component_copy_absent=" +
  str(not Path("crates/temper-engine-protocol-forge").exists()))'` prints `component_copy_absent=True`.
- `cargo nextest run -p temper-service tests::forge` shows19passed (the step-start count plus3).
- The process tier (03.1's command) exits 0 with its count at the step's start (19 at the base).
