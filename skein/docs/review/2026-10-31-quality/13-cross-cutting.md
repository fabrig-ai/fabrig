**Code-quality review of skein, the whole tree**

All paths below are relative to `skein (at 3371027)`.

## 1. Scope read

**Read closely:**
- `docs/design/README.md` §3 and `testing.md` §6.
- `programming-model.md` §1, §2, §5.2, §6.3 and §10, and `testing-strategy.md` §4, §7 and §10.
- Every `Cargo.toml`, compared against the documented crate graph.
- The connection modules side by side:
  - `skein-forge-connection/src/connection.rs`
  - `skein-forge-setup/src/host/connection.rs`
  - `skein-llm-connection/src/call.rs`
  - `skein-oauth-accounts/src/exchange.rs`
  - the settings, limits, boundary and `next_deadline` code of all four components.
- The `Host` trait and `Referee` trait, plus 12 of the 45 `Host` impls.
- `skein-fake-peers/src/face.rs`, `skein-world/src/{stream,referee}.rs`, and the top-level layout of 12 test crates.

**Skimmed:** two or three test files each in forge/machine, oauth-accounts and io, and what the fake crates import.

## 2. Scores (tree as a whole)

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 4 | Little dead code. Noise is local: about 160 copies of one doc line, and constants retyped per file. |
| Organization | 3 | The layout rules are documented, but each is applied 2–4 ways. Shared harness parts live inside test crates and are reached through `#[path]` and `include_bytes!` across crates. |
| Taste | 4 | Careful and consistent idiom. The step machines read like the design. |
| Naming | 3 | The same concept has several names across components (has-work, validity, lifecycle, errors, queues). |
| Primitives | 3 | lib, io, http and tls are strong. The component layer is missing its shared pieces. |
| Elegance | 4 | The machines follow the problem's shape, and the component boundaries match the docs. |
| Economy | 3 | About 2.5–3.5k lines of cross-crate near-copies (findings 1–4), small next to 481k but concentrated. |
| Complexity | 4 | Mostly just enough. Close/abort is encoded four ways, including an `Option<bool>` used as a three-state value. |

## 3. Strengths

- The component boundaries are uniform where it matters: `new → Result`, then `down`/`up`/`fire`/`next_deadline`/`reclaim`, with `Close` answered by `Closed`.
- The step crates hold the subset everywhere sampled: exhaustive matches, the `Env` threading, `expect` messages that name the invariant.
- The world harness is real infrastructure that tests reuse instead of mocking: `skein-world`'s `Expectations` and `stream::Wire`, and the `Host` trait.
- The test names are behaviour sentences, e.g. `a_refused_github_key_stops_the_forge`.

## 4. Findings, ranked by payoff

### 1. Four hand-rolled "socket + optional TLS + HTTP" stacks (Primitives, Economy)

Each component carries its own copy of the same machinery: socket, `Option<tls::Client>`, the TLS and plaintext queues, routing down to io or through TLS, handshake, chunked upload, and close/abort with a `close_sent` flag.

**Evidence:**
- **forge-connection** (`skein-forge-connection/src/connection.rs`): struct at 116–148, `connect` 352, `stream` 370, `http_down`/`json_down` 385–396, `pump` 397–449, `send` 500, `tls_event` 509, `upload` 779–811, `close` 920–936, `ready` 943.
- **forge-setup** (`skein-forge-setup/src/host/connection.rs`): struct 28–50, `connect` 207, `stream` 239, `pump` 286–355, `upload` 477–505.
- **oauth-accounts** (`skein-oauth-accounts/src/exchange.rs`): `Web` at 171–556, with `up` 260, `progress` 349, and `close`/`close_progress`/`close_socket` 475–536.
- **llm-connection** (`skein-llm-connection/src/call.rs`): `connected` 177, `cipher_up` 250, `close_socket` 305, `close_transport` 322.

**The copies have already drifted:**
- Only oauth-accounts (`exchange.rs:494`) and llm-connection (`call.rs:326`) send TLS `close_notify`. The two forge components close the socket directly (possibly intended; unsure).
- The queues have three naming schemes: `tls_up/tls_down/http_up/http_down`, `tls_events/cipher_down/llm_events/plain_down`, and `events/requests/plain/tls_events/cipher`.
- Close/abort is encoded four ways: `closing: bool` + `Phase`; `closing: Option<bool>` + `close_sent`; a `Control` enum; and `abort: bool` + `Phase` + `close_sent`.

**`Transport` is defined four times:**
- `skein-llm-connection/src/endpoint.rs:11` is identical to `skein-oauth-accounts/src/accounts.rs:38`.
- `skein-forge-connection/src/settings.rs:66` is identical to `skein-forge-setup/src/host/settings.rs:77`, and `host()` is byte-identical in both (`settings.rs:75` and `host/settings.rs:87`).

**Forge-specific copies:**
- Method mapping appears four times (`connection.rs:239`, `:272`; `host/connection.rs:99`, `:170`).
- JWT minting appears twice (`skein-forge-connection/src/app.rs:24` and `host/connection.rs:734`).
- forge-setup repeats its 22-field struct literal (`host/connection.rs:106` and `:177`).

**Fix:**
- Add one concrete transport struct in skein-tls, or a small crate over io and tls. It holds the socket, the optional TLS client, the cipher queues and the close state, with `connected`, `up`, `down`, `close(abort)`, `pump(io)` and `has_work`.
- Add a shared `Endpoint { address, transport }` with `host()`.
- Add a `skein_forge::github::jwt::window(wall)` helper.

**Size:** about 750 lines of plumbing become about 200 shared; net about −550. Abort-after-close would then have one implementation.

### 2. The hosted-process boilerplate is reinvented in test crates (Economy)

**The io pass:** `Face::up`/`down`/`work_pending`/`is_empty` in `skein-fake-peers/src/face.rs:64–133` is `pub(crate)`. It is retyped in:
- `tests/oauth-accounts/src/world.rs:667–745`
- `tests/llm-connection/src/simulated.rs:149–220`
- `tests/tls/tests/suite/fake_peer.rs:94–200`
- `tests/ui-web/src/harness_peer.rs:128–170`

`tests/fake-peers/tests/suite/stages.rs:4` even compiles `face.rs` a second time through `#[path]`. `const ROOM: u32 = 64` appears in 8 files.

**Delegating `enum Process` Host impls:**
- Five go through `host()`/`host_mut() -> &dyn Host` (tls `fake_peer.rs:225`, `echo/tests/end_to_end.rs:20`, and three in `tests/world/tests/suite`).
- At least five write an explicit match in every method, 44–68 lines each: oauth-accounts `world.rs:815`, `sign_in.rs:548`, `sign_in/cuts.rs:69`, `private_cuts.rs:37`, and llm-connection `simulated.rs:285`.

**Trivial referees:** about 21 have an empty `act`, 22 have `overdue → None`, and 17 have `next_deadline → None` (e.g. `tests/io/src/append.rs:278–292`).

**Fix:**
- Make `Face` a public io-pass struct in skein-world.
- Add `impl<H: Host + ?Sized> Host for Box<H>` in skein-shell, so every delegating enum shrinks to a single `host_mut`.
- Give `Referee::act`, `next_deadline` and `overdue` defaults, as `starved` and `last_word` already have.

**Size:** about −900 to −1,200 test lines.

### 3. Small missing primitives, each written privately 4–9 times (Primitives)

- **Earliest-deadline fold:**
  - Four private copies: `skein-lib/src/requests.rs:718`, `skein-oauth-accounts/src/component.rs:1239`, `skein-llm-connection/src/deadlines.rs:159`, `skein-forge-setup/src/host.rs:376`.
  - Seven inline `earliest.is_none() || Some(t) < earliest` checks in `skein-forge-connection/src/component.rs:545–587` and `hooks.rs:116`.
  - Nested-match versions in `skein-llm-connection/src/component.rs:584–619`, even though that crate has its own `earlier`.
- **Header construction:** `fn header/field(name, value) -> Header` has 9 definitions (`skein-llm/src/{openai,anthropic}/identity.rs`, forge-connection `connection.rs:981`, forge-setup `connection.rs:655`, and 5 test crates). There are also about 35 inline `Header { name: bytes::copy_of(..), value: bytes::copy_of(..) }` literals.
- **Forge `Method`:** two identical enums (`skein-forge/src/forgejo/request.rs:5`, `github/request.rs:8`), mapped to `skein_http::Method` four times in production and back three times in tests.

**Fix:** `skein_lib::time::earliest(&mut Option<Time>, Time)`, `skein_http::Header::new`, and one `skein_forge::Method`.

**Size:** about −200 lines, and one name for each concept.

### 4. Shared harness parts live in `tests/` and spread by path tricks (Organization)

**Test crates used as libraries by other test crates** (all depended on by relative path):
- `skein-tls-world::pki` is used by forge/machine, forge/setup, llm-connection and oauth-accounts.
- `skein-llm-world` is used by 6 crates.
- `skein-forge-machine-world` is used by forge/setup and forge/protocol.
- The step crate `skein-llm` takes `skein-json-world` as a dev-dependency.

**Includes that cross crate boundaries:**
- `skein-fake-peers/src/transport.rs:16` and `transport/native.rs:277–279` use `include_bytes!` on `tests/tls/fixtures`.
- `#[path]` reaches across crates in:
  - `tests/fake-peers/tests/suite/github_memory.rs:2`: another test crate's fixtures, behind `#[expect(dead_code)]`.
  - `tests/echo/tests/suite/scenarios.rs:249`: the example's `io_stage.rs`.
  - `skein-codec/src/lib.rs:16`: `tests/codec/src/generated/v1.rs`.

**Why it matters:** `testing-strategy.md` §7 says a harness moves into skein once a second user needs it. `testing.md` §6 says `testing/` holds what more than one crate shares.

**Stream endpoints are reimplemented too:** `skein-world::stream::Wire` is 120 lines with 8 users. Yet `tests/forge/machine/src/pipe.rs` (`End`, 247 lines), `tests/protocol/src/wire.rs`, `tests/tls/src/drive.rs`, `tests/http/src/client_world.rs:479` and `tests/llm/src/fake.rs:206` each build their own. About 20 files hold an `Option<(Read, u32)>` demand. Some of them model withdrawn demands, so they may not all merge (unsure).

**Fix:**
- Move the PKI to `testing/skein-test-pki`.
- Move the LLM fixtures to a `testing/` kit.
- Extend `Wire` with cuts, and retire `pipe.rs`.

### 5. The layout rules are written once and applied four ways (Organization)

**In-crate tests take four shapes:**
- `src/tests.rs`
- a `src/tests/` directory
- about 25 `src/<mod>/tests.rs` files
- 13 sibling `*_tests.rs` files mounted by `#[path]` (8 in oauth-accounts)

**`tests/` inside `crates/`:** three crates have one, against `testing.md` §6 ("No crate under `crates/` has a `tests/` directory"): `skein-forge` (6,654 lines, with its own `common/`), `skein-fake-forge-protocol` and `skein-fail-stop`.

**Test-crate support code** sits in `src/` (the rule), in `tests/support/` (codec, browser, ui-web, oauth-issuer, conformance/real), or in `tests/suite/support.rs`.

**Test-crate paths and names:**
- `tests/forge/*` nests by tier (machine, protocol, client, setup, github, conformance), while `tests/oauth*` and `tests/llm*` are flat.
- Two package names lack the `skein-` prefix: `forgejo-conformance` (in `tests/forge/conformance`) and `github-conformance` (in `tests/forge/github`).

**Test-crate lint policy** is hand-copied into about 36 manifests, with about 20 distinct variants:
- `tests/kv` denies neither `wildcard_enum_match_arm` nor `unwrap_used`.
- `tests/forge/*` deny 9 lints.
- 9 crates take the full step-crate workspace lints.

**Fix:**
- Pick one form for each kind.
- Move `crates/skein-forge/tests` under `tests/forge/`.
- Rename the `*_tests.rs` files to `<mod>/tests.rs`.
- Generate the test-crate lint blocks from one template; the assembly already generates the root manifest.

### 6. The documented crate graph and parts table have drifted (Organization)

Compared with the manifests, `README.md` §3 is wrong in these ways:

**Missing dependencies:**
- skein-oauth also depends on codec and tls.
- skein-fake-oauth also depends on tls.
- skein-forge-setup also depends on tls.
- skein-throwaway-forgejo also depends on fail-stop and world.
- skein-fake-person-browser also depends on http, shell, scratch, ui, fake-peers and world.

**Wrong "nothing from skein":** ui-bundle actually depends on fail-stop and lib, and codegen on fail-stop and codec.

**Absent crates:** 14 crates are not in the graph at all (fake-forge-github, fake-llm-*, fake-peers, fake-checkout, fail-stop, heap, scratch, world, conformance, echo-client, fake-channel, forge-conformance, lab-browser).

**Stale entries:**
- `skein-playground` is listed, but no such crate exists and the entry is not marked unbuilt.
- §2 says "Forgejo's built, GitHub's later", but `skein-forge/src/github` is 14.5k lines.

The root `Cargo.toml` also lists 9 fake crates twice.

**Fix:** generate §3 from `cargo metadata` in the gate.

### 7. Same concept, different names (Naming)

- **"Has a step to take":** `has_work` (7 crates), `ready` (forge-connection, forge-setup, tls and others), `is_ready` (io, kv, forge-client), and `work_pending` (`Host`).
- **Limits validity:** `programming-model.md` §6.3 prescribes `valid(limits) -> bool`. The forge family uses `check -> Result<(), Problem | Unusable | Refused>` (`skein-forge` ×2, `forge-connection/src/limits.rs:205`, `forge-setup/src/limits.rs:20`).
- **Constructor errors:** `Unusable` in three components, `EndpointError` in llm-connection.
- **Component lifecycle:**
  - llm-connection (`component.rs:24`) and oauth-accounts (`component.rs:122`) both define an identical `enum Lifecycle {Live, Closing, Aborting, Closed}`.
  - forge-connection and forge-setup use bools instead.
  - Neither forge component has `Request::Abort` (`forge-connection/src/boundary.rs:81`, `forge-setup/src/host/boundary.rs:114`), although §5.2 says a component "takes an abort as well".
- **`Host` means two things:** forge-setup's component is called `Host` (`host.rs:24`), the same name as the `skein_shell::Host` trait.
- **Limits fields:** `idle_keep` vs `idle`; `start_backoff`/`start_backoff_max` vs `backoff`/`backoff_max`.

**Fix:**
- Use `has_work` everywhere.
- Choose `valid` or `check`, and amend §6.3 to match.
- Add a `skein_lib::Lifecycle` with `close()` and `abort()` transitions.
- Rename forge-setup's `Host`.

### 8. The fakes reuse the real codecs (Primitives; breadth only)

**What the fakes import from the crates they stand in for:**
- fake-forge-protocol: 20 of 29 production files import `skein_forge`, including `forgejo::client::encode`.
- fake-forge-github: 24 of 28 (`api::Answer` ×12, `github::request::encode` ×7).
- fake-llm-protocol decodes requests with the client's own parsers (`documents.rs:145–147`, `:208`) and then translates.
- fake-oauth answers with `skein_oauth::encode_response` and `encode_error` (`lib.rs:9–12`, `:234`, `:445`). `oauth.md` documents this choice.

**Clean:** fake-llm-domain, fake-forge-domain, fake-checkout and fake-machine.

**The pattern:** the domains are independent, but the protocol faces share the client's codec, so a codec bug cancels out on both sides of the wire.

**Fix:** either amend `testing-strategy.md` §4 to allow faces to share codecs, or give the faces their own decoders.

### 9. Noise (Cleanliness)

- `/// Contract: docs/design/fake-llm.md, sections 2–5; programming-model.md, section 4.4.` appears about 160 times, mostly across `skein-fake-llm-domain/src/{api,domain}.rs` and `skein-fake-llm-protocol/src/{provider,documents}.rs`. With the blank `///` line before each, that is about 320 lines that could be one module doc.
- `const SECOND`/`SEED = 7` is retyped in 12 files of `tests/forge/machine/tests/suite`, although `src/fixtures.rs` exists.

## 5. Cross-area suspicions (unverified beyond this pass)

- **Forge dialects:** the GitHub dialect imports neutral helpers from the Forgejo one: `forgejo::document::Body` ×14, `id::unsigned` ×13, `time`, `path`, `decode::Malformed`. These belong at the root of `skein_forge`.
- **Label-driven harnesses (oauth-accounts, io, world):** there are 13 `Story` enums, and the harness branches on the label, e.g. `tests/oauth-accounts/src/world.rs:512` and `:729`, with 63 `Story::` references in `sign_in.rs`. Calls use positional bools, e.g. `sign_in::run(43, Story::Public, false, false, …)`. This goes against §10 "Rows carry data".
- **forge-setup:** `Speech` reimplements forge-connection's per-dialect `Request`/`Decoder`/`Body` enums in a different shape.

## 6. Rule costs

- **Exhaustive matches in tests:** test crates deny `wildcard_enum_match_arm`, so pulling one event out of a list costs about 12 lines of `| Variant { .. } => panic!` (`tests/forge/machine/tests/suite/github_start.rs:43–57`). A `let … else` would do, and the lint does not forbid it. Low priority.