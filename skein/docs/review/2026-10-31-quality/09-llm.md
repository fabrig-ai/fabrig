I read the main types, the client and both dialect paths, llm-connection's component and connection, the fake crates' state machines and the test harnesses closely, and sampled the test suites. The two largest costs are two OpenAI request encoders kept in lockstep, and output and outcome types copied once per dialect and again at crate level. About 1,000 lines of production code and 800–1,200 lines of tests could go without breaking any house rule.

All paths below are under `skein (at 3371027)`.

## 1. Scope read

**Read closely.**
- skein-llm: `types.rs`, `lib.rs`, `dialect.rs`, the production part of `translate.rs`, `openai/common.rs`, `openai/request.rs`, `client.rs` (state, fields, entry points), `anthropic/request.rs` (validate and charge), the receive/fail/ready parts of both `response.rs` files, `derive.rs`, `document.rs`, `replay.rs`, `README.md`.
- skein-llm-connection: `call.rs` (state and `route`), `component.rs` (state, `down`, `has_work`, `next_deadline`), `deadlines.rs`, `endpoint.rs`, `boundary.rs`.
- skein-fake-llm-domain: `domain.rs`, `lib.rs`, parts of `api.rs`.
- skein-fake-llm-protocol: the start of `documents.rs`, `Service`/`Server` in `provider.rs`, `credentials.rs`.
- tests/llm: `src/lib.rs`, the `Exchange` in `src/fake.rs`; samples of `suite/client.rs`, `suite/archives.rs`, `openai/tests.rs`, `anthropic/request_tests.rs`, `client/tests.rs`.
- tests/llm-connection: `world.rs`, `limits` in `simulated.rs`, the start of `suite/failures.rs`.

**Skimmed.** The bodies of the decoders and encoders, `peer.rs`, `json.rs`, `filter.rs`, `respond.rs`, `cache.rs`, `pieces.rs`, `inflate.rs`, most suites, the fixtures.

## 2. Scores

| Criterion | Score | Why |
|---|---|---|
| Cleanliness | 3 | The code is tidy. Against that: about 340 lines of "Contract:" boilerplate in the fake crates, stale route docs, doubled doc summaries, unused parameters. |
| Organization | 3 | Shared dialect code lives under `openai::`. `translate.rs` sits at the crate root but only serves OpenAI. Code that only the fake uses ships in the production crate. |
| Taste | 3 | The bounded arithmetic and derived limits are careful. The main state machines are flags plus fields, and several types have chains of forwarding constructors. |
| Naming | 3 | `Cap`, `Phase`, `Evidence` and `Report` are precise. But "codex"/"ChatGPT" names sit on code that serves the public route, and `openai::Limits` is the neutral limits type. |
| Primitives | 3 | `Cap`/bound, the `Replay` envelope and `Declared → Limits` are good. Against that: three `Output` enums, two of `Stop`/`Usage`/`Failure`, two OpenAI request types, and two error types mapped onto each other by hand 32 times. |
| Elegance | 3 | The derivation and selective decoding follow the problem. The client's and the connection's state does not follow the shape in 5.4. |
| Economy | 2 | About 1,000 removable lines in production; test setups typed out again and again. |
| Complexity | 3 | The withdrawn Codex route is still threaded through everything (261 mentions in the area), there's a test-only second encoder, and constructor chains. |

## 3. Strengths

- `crates/skein-llm/src/derive.rs:111-148`: per-dialect constants, each with a comment deriving it from the archived fixtures, plus a `Violation` that names the relationship that broke. This is the model for "named sizes".
- The failure vocabulary is typed: `Cap` with its `bound`, `Evidence`, `Report`, `Phase`. Tests assert limits by name (for example `tests/llm/tests/suite/client.rs:868`).
- Transcripts are kept as records, with provenance in `README.md` and `meta.json` files. They are driven through the codec (`openai/tests.rs:515`) and through the whole client (`suite/archives.rs:35`).
- skein-llm-connection reuses skein-llm's vocabulary instead of wrapping it. `EndpointError` (`endpoint.rs:49-78`) names each capacity relationship. `deadlines.rs` is a small table with total functions over it.

## 4. Findings, ranked by payoff

**F1. Two OpenAI request encoders and an owned request type, kept for the fake and as a test oracle.** *Economy, Primitives.*
- `translate.rs:49-199` writes a `Prompt`. `openai/request.rs:98-212` writes an `openai::Request` with the same fields in the same order.
- `translate.rs:249-302`, `571-628` and `685-712` are a `#[cfg(test)]` converter from `Prompt` to `openai::Request`, used only to compare the two encoders (`client/tests.rs:572-575`, `translate.rs:858`, `900`, `941`).
- Anthropic already decodes into the neutral `Prompt` (`anthropic/peer.rs:13-17`). OpenAI decodes into the owned type, and the fake's `documents.rs` consumes `openai::Input`.
- **Fix:** have `openai::decode_request` return `Prompt` plus the peer's extras, as Anthropic does. Delete `openai::{Request, Input, Tool, Role, encode_request, measure_request}` and the test-only converter. The archived bodies already check the encoder (`translate.rs:930`).
- **Size:** about 350 production lines and about 100 test lines.

**F2. The output vocabulary is copied three times, and `Stop`/`Usage`/`Failure` twice.** *Primitives, Economy.*
- `openai::Output` (`openai/response.rs:123`) and `anthropic::Output` (`anthropic/response.rs:74`) are identical apart from `Part`; `dialect::Output` (`dialect.rs:15`) is a third copy.
- `openai::{Stop, Usage, Failure}` (`common.rs:146-193`) duplicate `types.rs:137-158` and `304-331`, doc comments included. The identity mappings are at `translate.rs:795-827`.
- `codex_outputs` and `anthropic_outputs` (`dialect.rs:176-232`) are near copies, as are the decoders' `received` and `fail`.
- Anthropic re-exports OpenAI's `Stop`/`Usage`/`Failure` (`anthropic.rs:8`), so these are not separate per-dialect vocabularies anyway.
- **Fix:** have the codecs push `dialect::Output` with `Block` already built, emit the crate's `Stop` and `Usage`, and have `classify` return the crate's `Failure`. Share one detail helper for failures.
- **Size:** about 200 lines.

**F3. Request admission is written once per dialect, with hand-written error remapping.** *Economy.*
- `validate` at `translate.rs:304-353` and at `anthropic/request.rs:37-99` share the same opening checks: empty model, tool count, history count, `validate_choice`, charging the names, model, instructions and effort.
- `charge`/`charge_text`/`charge_json` (`translate.rs:438-484`) are mirrored by `charge`/`text`/`json` (`anthropic/request.rs:215-253`).
- The mapping from `writer::Refusal` to a cap is written 7 times: `translate.rs:446`, `481`, `678`, `756`; `common.rs:414`; `json.rs:124`; `anthropic/request.rs:259`.
- `append` exists twice (`common.rs:386`, `anthropic/response.rs:542`).
- The four-line `Err(error) => return Err(decode(error))` appears at 32 sites.
- **Fix:** one admission module with a per-dialect hook for block checks. Write the inner helpers in `DecodeError` so `?` works, and convert once at `measure_request`. No `From` impl is needed.
- **Size:** about 200 lines.

**F4. The client and the connection hold per-state data in fields next to a state enum that carries none.** *Primitives, Elegance; breaks programming-model 5.4.* This is the largest structural change.
- `client.rs:245` `State` carries no data. The `Client` (`client.rs:275-313`) holds `call: Option<…>`, `upload`/`upload_offset`/`upload_end`, `decoder: Option`, `content: Option`, `error`, `status`, `asked`, `close_sent`, `sse_closed`, plus `Outcome` and `EventData`.
- There are 16 `state == State::…` checks (for example `:682`, `:693`, `:752`, `:768`), no `mem::replace` on `state`, and `expect` calls standing in for invariants the type could hold (`:683`, `:769`).
- The same shape appears in `skein-llm-connection/src/call.rs:28-70` (`Phase`, two `Option<Token>`s, three flags, `Control`) and in `skein-fake-llm-protocol/src/provider.rs:272-300`.
- By contrast, `skein-fake-llm-domain/src/domain.rs:288` and `:432` follow the idiom.
- **Fix:** move `error` into `ErrorBody`, `call` into `Prepared`, `upload*` into `Head`, and `decoder`/`content`/`asked` into `Streaming`. Apply one `follow_state` after each transition.
- **Size:** roughly neutral, but much clearer.

**F5. Every harness re-wires the fake peer by hand.** *Organization, Economy (tests); testing-strategy §4, "one fake, every tier".*
- `provider::Service`, `Server` and the fake `Domain`, plus the routing from domain to protocol, are assembled by hand in:
  - `tests/llm/src/fake.rs:181-216` and `546-760`
  - `tests/llm-connection/src/world.rs:110-278`
  - `tests/llm-connection/tests/suite/fake_protocol.rs:66-78`
  - `tests/llm/tests/suite/fake.rs:993-1008`
  - `crates/skein-fake-peers/src/llm.rs:49-110` and `402`
- **Fix:** add a composed `Peer` step machine to skein-fake-llm-protocol. Harnesses keep only their byte pump and their observations.
- **Size:** about 300–400 test lines.

**F6. "Contract:" boilerplate in the fake crates.** *Cleanliness.*
- There are 171 `/// Contract: …` lines, 156 of them the identical "fake-llm.md, sections 2–5; programming-model.md, section 4.4", each with a blank `///` line before it (168 of those).
- They sit on every enum variant and trivial accessor: `api.rs:220-240`, `domain.rs:380-400`. `api.rs` alone has 65.
- **Fix:** keep the module-level contract, which `lib.rs:22-24` already has, and delete the rest.
- **Size:** about 340 lines.

**F7. Test fixtures default to the withdrawn Codex route.** *Complexity.*
- `tests/llm/src/lib.rs:32-65`: `limits()` and `call()` use `Endpoint::codex()` with an account id, and `anthropic_call` uses the withdrawn bearer.
- `call(N)` has 131 uses against 23 for `public_call`. The area mentions Codex 261 times, while `types.rs:15`, `:391` and `:418` say "removed by the second part".
- **Fix:** make the public route and an Anthropic key the defaults now, and gather the Codex stories into one file, so removing Codex later is mostly deleting that file.

**F8. Dialect-neutral code is placed and named as OpenAI's.** *Naming, Organization.*
- Anthropic's code takes `&openai::Limits` everywhere.
- `openai::classify` handles Anthropic (`common.rs:262`, `327-357`), and `RateLimit::observe` reads `anthropic-ratelimit-*` headers (`common.rs:217-221`).
- `lib.rs:27-28` exports `openai::DecodeError as DocumentError`.
- `translate.rs` sits at the crate root ("Bridge … Codex documents") but serves only OpenAI.
- `codex_outputs` (`dialect.rs:176`) and `validate_codex` (`translate.rs:42`) also serve the public route, and the decoder's failure details say "ChatGPT stream" (`openai/response.rs:234-316`).
- **Fix:** move `common.rs` and `json.rs` into a `native` module, move `translate.rs` to `openai/prompt.rs`, and rename `codex_*` to `responses_*`.

**F9. Stale and doubled docs.** *Cleanliness.*
- `lib.rs:3` says the crate "Supports ChatGPT's Codex … subscription routes"; both are withdrawn.
- `README.md:19-25` and `50-56` present Codex as the main route and Anthropic's default as Bearer with `oauth-2025-04-20`.
- A "`/// Bounds X.`" line is stacked on top of an existing summary at `common.rs:58`, `78`; `anthropic.rs:22`; `client.rs:1571`; `document.rs:27`; `replay.rs:20`; `openai/response.rs:1241`; `anthropic/response.rs:756`.
- `skein-fake-llm-domain/src/lib.rs:16-18` has two crate-doc blocks with a blank line between them.
- **Size:** about 40 lines.

**F10. Tests break the test-shape rules (testing-strategy §10).**
- **Rows branch on labels:**
  - `suite/client.rs:285-296` uses `match field {0 => …}` inside the loop.
  - `suite/client.rs:842-872` uses `match which` and `if bound == edge … else …`, two behaviours in one body.
  - `openai/tests.rs:569`, `573`, `586` branch on `if scenario == "…"`.
  - `client/tests.rs:572` branches on the provider.
- **Compound names:** 75 of 145 skein-llm tests and 94 of 216 tests/llm tests contain "_and_" (for example `openai/tests.rs:661`).
- **Retyped setups:**
  - A roughly 30-line llm-connection `Limits` literal appears in 7 files: `crates/skein-llm-connection/src/tests.rs:8`, `tests/llm-connection/src/world.rs:85`, `src/simulated.rs:21`, `suite/failures.rs:15`, `fake_protocol.rs`, `memory.rs`, `protocol.rs`. Some copies use 19_000 and others 32_768, with no derivation.
  - Five separate fixture loaders: `openai/tests.rs:100`, `archives.rs:7`, `derive.rs:69`, `translate.rs:932`, `selective.rs:17`.
- **Bare sizes:**
  - `TINY_DECL` (`lib.rs:9-18`): 171, 342, 174, 684, with no derivation.
  - `(Cap::Answer, 101)` and `document + 8` (`client.rs:840-846`).
- **JSON edited as text with no check that it changed:** `anthropic/request_tests.rs:498-507`.
- **Fix:**
  - Rows carry a setter closure.
  - Split the bound and one-past-it cases into separate tests.
  - One fixture loader in skein-llm-world.
  - Reuse `world::limits(calls)` with struct update.
- **Size:** about 250 lines.

**F11. Chains of forwarding constructors.** *Complexity.*
- The fake `Domain` has new → scripted → try_scripted → configured → played → played_by_input (`domain.rs:304-372`), each adding an empty default.
- `Exchange` has six constructors (`tests/llm/src/fake.rs:263-405`).
- The same pattern, on a smaller scale: `Client::prepare_with_reasoning_drop` (`client.rs:467`) and `Component::admit_with_reasoning_drop` (`component.rs:182`).
- **Fix:** one constructor that takes a setup struct with an `EMPTY` constant.
- **Size:** about 100 lines.

**F12. Small rule breaks and dead bits.** About 60 lines in total.
- `impl From<Error> for Failure` in a step crate (`skein-fake-llm-domain/src/api.rs:478`). Hand-written trait impls are out; use `Failure::of(error)`.
- Lifetime-parameterised structs in a step crate: `credentials.rs:52` (`Fields<'a>`) and `inflate.rs:134` (`Input<'a>`).
- Unused parameters: `respond.rs:507` (`_query`) and `call.rs:361` (`_owner`).
- No-op stubs `has_ready`/`ready` in `anthropic/response.rs:166-173`.
- `component.rs:584-619` reimplements `deadlines.rs:159` `earlier` twice.
- `call.rs:376-420` repeats the same terminal tail three times.

## Area questions

- **Per-provider shape.** The core is shared: one `Client`, an enum dispatch in `dialect::Decoder`, shared JSON, limits and `classify`. The two stream decoders differ for good reason: Responses reorders items by index, while Anthropic emits one block at a time. The edges are copy-and-adapt: admission (F3), outputs and outcomes (F2), failure details, and the duplicated OpenAI request (F1).
- **Fake coupling.** Importing `Credential` and `Provider` is minor, but it costs something:
  - The fake keeps two provider enums, its own `documents::Provider` and `skein_llm::Provider` as its route, and `route`/`authorize` match on `OpenAiCodex`, so withdrawing Codex will ripple into the fake.
  - The real coupling is the codec. The fake's request decoders and event encoders are skein-llm's. That is roughly 1,100 lines that only the fake uses (`anthropic/peer.rs`, `openai/response.rs:971-1346`, OpenAI request decoding and the owned encoder), and they ship in every service.
  - fake-llm.md §1 and §4 sanction this, and the archived transcripts reduce the risk of client and fake sharing one misreading. Still, it conflicts with testing-strategy §4 ("no shared types"). Moving the peer-only codecs into skein-fake-llm-protocol, with the transcripts as the shared referee, would fit the rules better.

## 5. Cross-area

- **What makes llm-connection small.**
  - The client already owns HTTP, SSE and JSON, measurement, failure classes, evidence and connection reuse (`next_call`, `Reusable`). skein-tls owns TLS and skein-io owns sockets.
  - Deadlines are a flat table of `Option<Time>` per phase.
  - There is no retry, no DNS and no credential logic; the credential passes straight through.
  - Its boundary reuses skein-llm's `Prompt`, `Block`, `Failure` and `Evidence` unchanged.
- **Its shape.**
  - `Component` holds the pool: a FIFO for memory admission, a waiting list, a slot list beside the slab, and a round-robin cursor.
  - Each `Connection` holds a socket token, an optional TLS client, the LLM client, a deadline table and four routing queues.
  - A bounded `route` pass moves work between the client, TLS and io.
  - Weak points: the flag-and-field state (F4), and 12 nested `for slot in &self.slots` / `match` loops that `if let` chains could flatten.
- **Unverified suspicions:**
  - skein-forge-connection (`connection.rs:9` `Phase`) and skein-oauth-accounts (`component.rs:100` `State`) may share F4's flag-and-field state shape.
  - Fake peers are hosted by hand for the forge too (`crates/skein-fake-peers/src/forge.rs`, `tests/forge/machine/src/joint.rs:334`), which would be the F5 pattern again.
  - skein-oauth and skein-fake-oauth may share codecs the same way.
  - The "Contract:" boilerplate does not appear in the forge or oauth fakes.

## 6. Rule costs

- `?` without `From` produces the 32 hand-written remaps; F3's fix avoids them within the rules.
- Having no `Option::map` (no closures) forces `match` blocks, marked with `#[expect(clippy::manual_map)]` (for example `deadlines.rs:151`, `translate.rs:250`). This costs about 5 lines per site, about 20 sites.