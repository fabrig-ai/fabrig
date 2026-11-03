+++
id = "smith-agent-llm"
part = "smith-agent-channel"
kind = "migration"
needs = ["temper-configs", "jig-dialects", "smith-product"]
files = [
  "smith/crates/smith-agent-shell/src/config/**",
  "smith/crates/smith-agent-shell/tests/**",
  "smith/crates/smith-protocol-llm/src/**",
  "smith/crates/smith-agent-service/src/fixture.rs",
  "smith/crates/smith-agent-service/src/profile/path.rs",
  "smith/crates/smith-agent-service/src/profile/**/tests.rs",
  "smith/crates/smith-agent-service/src/tests.rs",
  "smith/crates/smith-agent-service/src/tests/**",
  "smith/crates/smith-agent-service/tests/**",
  "smith/tests/protocol-llm/**",
  "smith/tests/agent-process/**",
  "smith/tests/agent/**",
  "smith/tests/session/**",
  "jig/tests/channel/src/fixtures.rs",
  "jig/tests/channel/src/program_runs.rs",
  "temper/tests/one-process/src/fixtures.rs",
  "smith/docs/design/protocol/agent.md",
  "smith/docs/design/protocol/llm.md",
  "smith/docs/design/protocol/transcript.md",
  "smith/docs/design/protocol/README.md",
  "smith/docs/design/domain/session.md",
  "smith/docs/design/STATUS.md",
  "smith/docs/design/protocol/agent.status.md",
  "smith/docs/design/protocol/llm.status.md",
]
effort = "high"
minutes = 180
+++

# smith's agent and LLM component: no `codex` dialect, no endpoint account, no identity profile

## Goal

smith's agent refuses the withdrawn forms of its configuration. An endpoint's `account` and
`identity` are unknown fields. The `codex` provider, dialect 1, is refused. smith's LLM component
serves only `openai` (3) and `anthropic` (2) and sends no identity blocks. Its grants install a
bearer or a key on `openai` and a key alone on `anthropic`, the scheme Anthropic's dialect takes
once skein's bearer route goes. smith's tests and worlds move off the Codex route
(`Endpoint::codex()`, `Provider::OpenAiCodex`, `skein_llm_world::call`, `limits.codex`) and off
`Credential::anthropic`, to the public route and to keys. This is the first half of the part.
The channel at v6, which removes the withdrawn id-and-bearer grant and `Cap::AccountId`, comes
next (`smith-channel-v6`). After both, smith names no skein item the skein tasks remove, except
the arms and fields listed under "What stays for skein's task".

Acceptance:

- An agent configuration with an endpoint's `"account"` or `"identity"` is refused, naming the
  field (serde's unknown field, as `a_credential_field_is_not_part_of_configuration` asserts).
  `"provider": "codex"` or dialect 1 is refused with "endpoint provider must be openai or
  anthropic". A test shows each.
- `rg -n 'IdentityProfile|anthropic::identity|openai::identity|Credential::anthropic|Endpoint::codex\(\)|limits\.codex|skein_llm_world::call\(' smith jig temper --glob '!**/docs/**'`
  finds nothing but the arms of "What stays for skein's task", and the channel's withdrawn arm
  (`smith-agent-service/src/service.rs`, ~449-466, `Credential::codex`), which `smith-channel-v6`
  deletes.
- The component refuses a bearer on `anthropic` as a scheme the dialect does not take. It installs
  a bearer or a key on `openai` with no account id. A test shows each.
- The design and status edits below are made. The tests in Steps pass, each count not zero, and
  clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `smith/AGENTS.md` (module-doc rules; smith's `clippy.toml` bans more types).
- Design: `smith/docs/design/protocol/agent.md`, 4 (endpoints, dialects and the Withdrawn
  bullet); `protocol/llm.md`, 2, 2.1, 2.2, 2.3 and 7 (prompts, affinity, caching, credentials);
  `protocol/transcript.md`, 6; skein's `skein/docs/design/llm.md`, 2.6, 4.7 and 4.8.
- Status (the spec): `smith/docs/design/protocol/agent.status.md`, item 4;
  `protocol/llm.status.md` (2, 7: the component serves `codex` and an endpoint's identity
  profile); `smith/docs/design/STATUS.md`'s withdrawn-forms sentence.

## The lead's calls (no question needed)

- **Refusals are the configuration's own.** `Endpoint` in `smith-agent-shell/src/config/document.rs`
  is `deny_unknown_fields` (~145), so deleting `_account` (~155-157) and `identity` (~176) is
  the whole refusal. `build_models` (`config/endpoints.rs`, ~226-234) drops its `"codex"` arm
  and says "endpoint provider must be openai or anthropic". `endpoint_identity` and the
  `ClaudeCode` check (~98-101, ~207-212, ~235-240) go.
- **The component's grants by dialect** (`smith-protocol-llm/src/component.rs`, ~262-272):
  `OpenAi` installs a bearer or a key; `Anthropic` installs a key and refuses a bearer, which is
  `protocol/llm.md` 7's "a scheme the dialect does not take is a value the agent cannot
  install". The `OpenAiCodex` arm installs nothing: configuration can no longer reach it.
  `grants.rs` (~60, ~107-108) and `framing.rs` (~16-44) build credentials with
  `Credential::bearer`/`Credential::key` by scheme. `framing.rs` measures Anthropic with a key's
  `x-api-key`, not a bearer.
- **Identity goes whole:** `IdentityProfile` and its re-export (`endpoints.rs`, ~15-22;
  `lib.rs`, ~54), `endpoint_own`'s identity, the `identity` fields of `ConfiguredEndpoint` and
  `EndpointOptions`, `EndpointError::Identity`, and the prepended blocks (`component.rs`, ~19,
  ~436-446). jig's and temper's literals drop their `identity:` line (below).
- **Limits:** `smith-agent-service/src/profile/path.rs` (~58) reads `limits.openai`, not
  `limits.codex`. Effective limits, `guard.rs`'s measured bytes and memory ceilings may move. Re-measure them
  and say each number that moved in the report. A memory ceiling may only fall or stay. If one must
  rise, stop and ask (a budget is the user's).
- **Worlds move to the public route:** `smith-agent-process-world`'s default configuration becomes
  `Provider::OpenAi` on `Endpoint::openai()` with a key or bearer grant (~72, ~195, ~205, ~214-224,
  ~274, ~288, ~408, ~422-432 `scenario_credential` without `account_id`; `world/setup.rs`,
  `world/grants.rs` ~26, ~37, `grants.rs` ~29-35, `world.rs` ~1265, `world/coding.rs`,
  `program.rs`, `fake.rs`). An Anthropic scenario grants a key. `smith-protocol-llm-world`'s
  `Dialect::Codex` becomes the public OpenAI route (`src/fixture.rs`, ~56-82). Its suites move
  with it: `component_peer.rs`, `guard.rs`, `component_boundary.rs`, `content_observations.rs`,
  `history_window.rs`, `codec.rs`, `continuation.rs` (`CODEX_*`; the stored replay tag
  `\x00\x01\x01` becomes the public route's `\x00\x01\x03` with its fields), `replay.rs`
  (`CODEX_ENVELOPE`). In `endpoints_grants.rs`,
  `configured_claude_identity_precedes_domain_instructions_only_for_anthropic` and
  `a_withdrawn_pair_keeps_the_bearer_in_value` are deleted. `skein_llm_world::call()` becomes
  `public_call()` (`component_peer.rs` ~211, agent-process `memory.rs` ~95, ~123, `lib.rs` ~223),
  and `skein_llm_world::anthropic_call()` is replaced by an Anthropic call with a key. A
  Codex-only story either moves to the public route, labelled synthetic as it already runs on
  skein's fake peer, or is deleted when it tested only the route.
- **Stored dialect-1 transcripts:** no new code. The transcript's carry-over (`transcript.md`, 4)
  compacts a window whose endpoint and dialect the charter no longer has
  (`compaction.rs`, ~514-530; `route.rs`, ~1061). That is no migration. Only the "such as" clause in
  transcript.md 6 goes.
- **Dialect 1 as an opaque domain number** in smith's domain tests (`smith-domain-run/src/tests/support.rs`,
  `smith/tests/run`, `smith/tests/local`) stays: it names no route.

## Owns and may change

smith (owned): the files list. Within it, the edits above, and each skein-llm
`Credential { .. account_id .. }` literal becomes `Credential::bearer`/`Credential::key`; each
`Provider::OpenAiCodex` value becomes `Provider::OpenAi`; each `Endpoint::codex()` value becomes
`Endpoint::openai()` (`smith-agent-service`: `fixture.rs` ~14, `profile/room/tests.rs` ~18,
`profile/tests.rs` ~28, `profile/effective/tests.rs` ~110, `tests.rs` ~481-487, `tests/line.rs`
~171 (`Credential::anthropic` to a key), `tests/suite/sink_memory.rs` ~39; `smith-agent-shell`:
`config/tests/mod.rs` and `tests/support/product.rs`; smith-agent-world `memory_returns.rs` ~16;
smith-session-world `window_room.rs` ~27, `startup_receiving.rs` ~26; `smith-protocol-llm/src/tests.rs`
~21, ~929). The agent-shell tests: `an_endpoints_account_is_read_and_ignored` (~45-59) becomes a
refusal naming `account`; `a_dialect_name_and_number_must_match` (~130-145) refuses
`("codex", 1)`; `shaped_document` (~261-268) writes `openai`, 3, no account; `all_product_models`
(~408-432) sets no identity.

Readers in jig and temper (the field leaves smith's struct; delete only the line):

- `jig/tests/channel/src/fixtures.rs` (~260) and `jig/tests/channel/src/program_runs.rs` (~99):
  `identity: smith_protocol_llm::IdentityProfile::Plain,`.
- `temper/tests/one-process/src/fixtures.rs` (~291): the same line.

What stays for skein's task (exhaustive matches and a struct's fields, which cannot leave while
skein has them; `wildcard_enum_match_arm` is denied): every `Provider::OpenAiCodex` match arm
(`smith-protocol-llm/src/{framing.rs, component.rs}`, `smith-agent-shell/src/config/endpoints.rs`
~70 and ~228, `smith-agent-service/src/tests/line.rs`, smith-protocol-llm-world's
`component_backend.rs`, `pool_peer.rs`, `component_peer.rs`, `codec.rs`,
`content_observations.rs`, `endpoints_grants.rs`, `guard.rs`, `history_window.rs`,
smith-agent-process-world `lib.rs` ~223), with bodies that install, measure or render nothing
for the route; every `skein_llm::CredentialLimits { .. account_id .. }` literal (its value left
as it is); `skein_llm::Cap::AccountId` arms (`smith-protocol-llm/src/failure.rs` ~232,
`smith-agent-service/src/profile/words.rs` ~177). Leave to `smith-channel-v6` every
`GrantValue::Withdrawn`, `Form::Withdrawn`, `WithdrawnValue`, `smith_channel` path or version,
and `CEILINGS.withdrawn_value_id` (`config/endpoints.rs` ~296-300).

Design and status (as `notes/common.md` says):

- `smith/docs/design/protocol/agent.md`, 4: the dialects' sentence ends "...; `anthropic`, 2,
  Anthropic's Messages dialect (skein's `llm.md`, 4.7). A transcript's turns and opaque blocks
  carry the number (transcript.md, section 2), so a number is never given to another dialect,
  and 1, once the withdrawn Codex route's, is given to none; a name and number that do not match
  are refused, naming the field;". Delete the whole ***Withdrawn*** bullet (~188-196). In 10
  (~404), "each withdrawn field accepted" becomes "each withdrawn field, `account` and `identity`,
  refused, naming it".
- `protocol/llm.md`: 2 (~57-59), "after any identity blocks ... (agent.md, section 4)" goes, so
  the item reads "the system text, which the domain wrote (domain/run.md, 3.3)". 2 (~64-67): the
  output cap's ", and the withdrawn Codex route takes none, so there it stays a local bound on
  the answer" goes, the sentence ending at `max_tokens` with the skein citation kept. 2.1
  (~83-86): the Codex sub-bullet becomes the public route's, "OpenAI: the key as the body's
  `prompt_cache_key`, with no header". Keep the sentence on the reserved names if it is there.
  2.2 (~113-114): the Codex sub-bullet becomes "on the OpenAI dialect, which keeps the tool choice
  outside the cached prefix, the request reads the cached prefix". 2.3 (~131): "**Codex**" becomes
  "**OpenAI**". 7: the sentence on schemes gains "(a bearer on `anthropic`, which takes a key
  alone)" after "a value the agent cannot install". The ***Withdrawn*** bullet (~487-492) goes.
  10 (~598): "the withdrawn `codex` dialect with its id-and-bearer grant;" goes. 12 (~646-648):
  "Codex's turn-state token" becomes "OpenAI's turn-state token". "a patch as a free-form tool
  needs skein's Codex dialect to offer one" becomes "needs skein's Responses dialect to offer
  one".
- `protocol/transcript.md`, 6 (~232-234): ", such as a conversation on the withdrawn `codex`
  dialect keeps," goes.
- `protocol/README.md`, 8 (~423-424): "The Codex dialect and its account header stay, withdrawn
  (4.6);" goes. The item ends at "(2.5);". ~461: "Codex's routing state" becomes "OpenAI's routing
  state".
- `domain/session.md` (~543-544): "on the Codex dialect" becomes "on the OpenAI dialect".
- `protocol/agent.status.md`: item 4 goes. The item at 1, 3, 6, 8 stays.
- `protocol/llm.status.md`: its entry goes, and the file says `No gap.`.
- `smith/docs/design/STATUS.md`: the withdrawn-forms sentence becomes "The withdrawn id-and-bearer
  grant is still accepted on the channel, and its removal is not yet (`protocol/channel.md`,
  `protocol/hosts.md`)." Re-date it.

Pre-approved options:

- If a jig or temper world grants a bearer on Anthropic through smith's component, switch it to a
  key (value only) and list it. jig's channel world already uses a key there
  (`jig/tests/channel/src/processes.rs`, ~292-294).
- If moving a Codex story to the public route changes a measured byte count, a golden or a
  ceiling, regenerate it with its test's write mode. If it has none, recompute it from the test's
  output and say which. A ceiling only falls.
- If `smith/tests/local-process/**` still names a form this task removes after `smith-product`,
  edit only that line and list it.

## Steps

1. The configuration's refusals and their tests: `heavy cargo nextest run -p smith-agent-shell --lib`.
2. The component: identity, grants by dialect, framing:
   `heavy cargo nextest run -p smith-protocol-llm --lib`.
3. The service's limits and fixtures: `heavy cargo nextest run -p smith-agent-service --lib` and
   `heavy cargo nextest run -p smith-agent-service --test suite` (check the binary's name with
   `--list`).
4. The worlds, one at a time, each count not zero:
   - `heavy cargo nextest run -p smith-protocol-llm-world --test suite`
   - `heavy cargo nextest run -p smith-agent-process-world --test suite`
   - `heavy cargo nextest run -p smith-agent-world --test suite`
   - `heavy cargo nextest run -p smith-session-world --test suite`
   - `heavy cargo nextest run -p smith-agent-shell --test suite`
   - `heavy cargo nextest run -p smith-local-process-world --test suite`
5. A change of meaning (smith-agent-process-world's default is the public route, and a bearer on
   Anthropic is refused). These cross-tree readers use it as a library:
   - `heavy cargo nextest run -p jig-channel-world --test suite`
   - `heavy cargo nextest run -p jig-one-process-world --test suite`
   - `heavy cargo nextest run -p temper-one-process-world --test suite`
   - `heavy cargo check -p temper-real-loop-world --tests`
6. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `smith/`;
   `heavy cargo clippy -p smith-agent-shell -p smith-protocol-llm -p smith-agent-service -p smith-protocol-llm-world -p smith-agent-process-world -p smith-agent-world -p smith-session-world -p jig-channel-world -p temper-one-process-world --all-targets -- -D warnings`.
7. The design and status edits, then run the tests that read smith's design:
   `heavy cargo nextest run -p smith-bench --test suite -E 'test(/offline|probe|tool_output/)'`
   and `heavy cargo nextest run -p smith-local-domain --lib -E 'test(/older/)'`;
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root; the acceptance's
   `rg`.
8. Commits: the configuration; the component; the service and worlds (with jig's and temper's
   lines); the design and status. Push your branch.

## Report

As the implementer's text says, and also: every limit, measured byte count and ceiling that
moved, with old and new values; each Codex story moved (and to what) or deleted, by file; what
the acceptance's `rg` leaves, by file and kind (arm, field, withdrawn grant).
