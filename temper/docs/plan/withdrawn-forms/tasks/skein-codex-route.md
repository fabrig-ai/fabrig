+++
id = "skein-codex-route"
part = "skein-routes"
kind = "migration"
needs = ["skein-anthropic-bearer", "smith-channel-v6"]
files = [
  "skein/crates/skein-llm/**",
  "skein/crates/skein-llm-connection/src/tests.rs",
  "skein/crates/skein-fake-llm-protocol/src/**",
  "skein/crates/skein-fake-peers/src/llm.rs",
  "skein/tests/llm/**",
  "skein/tests/llm-connection/**",
  "skein/tests/fake-peers/tests/suite/llm.rs",
  "skein/tests/tls/tests/suite/fake_peer.rs",
  ".config/fixture-readers.toml",
  "skein/scripts/old-forms.txt",
  "skein/docs/design/llm.md",
  "skein/docs/design/llm.status.md",
  "skein/docs/design/llm-connection.md",
  "skein/docs/design/fake-llm.md",
  "skein/docs/design/STATUS.md",
  "smith/crates/smith-protocol-llm/src/framing.rs",
  "smith/crates/smith-protocol-llm/src/component.rs",
  "smith/crates/smith-protocol-llm/src/failure.rs",
  "smith/crates/smith-protocol-llm/src/tests.rs",
  "smith/crates/smith-agent-shell/src/config/endpoints.rs",
  "smith/crates/smith-agent-service/src/tests/line.rs",
  "smith/crates/smith-agent-service/src/profile/effective/tests.rs",
  "smith/crates/smith-agent-service/src/profile/words.rs",
  "smith/crates/smith-local-shell/src/credentials/product.rs",
  "smith/tests/protocol-llm/**",
  "smith/tests/agent-process/src/lib.rs",
  "jig/tests/channel/src/program_runs.rs",
]
effort = "high"
minutes = 240
+++

# skein-llm: the Codex route, the account id and the Codex recordings removed

## Goal

skein-llm keeps two routes, OpenAI's public Responses route (`skein/docs/design/llm.md`, 4.8) and
Anthropic's Messages with a key (4.7). The withdrawn Codex route leaves: `Endpoint::codex()`,
`Provider::OpenAiCodex`, the `codex` limits set, its replay tag 1, its affinity headers, its
classes and request id, `openai::identity`. The account id leaves with it:
`Credential.account_id`, `Credential::codex`, `CredentialLimits.account_id`, `Cap::AccountId`.
The Codex recordings (`skein-llm/src/openai/fixtures/`) are deleted. They are never captured
again (no probe of a real service): what they covered becomes synthetic cases, labelled so, as
4.8's public route already is. smith's arms and fields that only this removal can drop go in the
same branch. Section 4.6 is deleted and its number left unused. This is the plan's last task.

Acceptance:

- `rg -n 'OpenAiCodex|Endpoint::codex|Credential::codex|account_id|AccountId|chatgpt\.com|backend-api/codex|usage_limit_reached|x-oai-request-id|openai::identity|limits\.codex' skein smith jig temper --glob '!**/docs/plan/**' --glob '!**/docs/development/**'`
  finds only names that are not the route: the forge's GitHub `account_id`
  (`temper/tests/engine/protocol-forge`), skein-oauth's generic claim test data, smith's own
  sign-in record fields (`smith-local-shell/src/credentials/sign_in.rs`, `login/provider/driver/tests.rs`)
  and smith's benchmarks' Codex CLI arm. The report lists what is left, by file and why.
- `skein/crates/skein-llm/src/openai/fixtures/` is gone, and `.config/fixture-readers.toml` names
  none of it: every entry left matches a tracked file and has a reader that selects a test, as
  `tools/tiers/design.md`, 9 requires (the queue validates the map).
- A replay envelope with tag 1 is refused as a provider the client does not know. Tags 2
  (Anthropic) and 3 (OpenAI) keep their values. A test shows it.
- `skein/docs/design/llm.md` has no section 4.6 (the number unused, 4.7 and 4.8 unmoved), and
  `python3 -I -B tools/citations/citations.py --base origin/main` reports nothing new.
- The tests in Steps pass, each count not zero, and clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `skein/AGENTS.md`; `smith/AGENTS.md`, `jig/AGENTS.md` for their lines.
- Design: `skein/docs/design/llm.md`, all sections the edits below name; `llm-connection.md` (~92,
  ~177-193); `fake-llm.md` (its Codex lines); `skein/docs/foundation/testing-strategy.md`, 8 (fixture
  readers); `tools/tiers/design.md`, 9.
- Status (the spec): `skein/docs/design/llm.status.md` (4.6, as `skein-anthropic-bearer` left it),
  `skein/docs/design/STATUS.md` ("the withdrawn Codex route still stands").

## The lead's calls (no question needed)

- **`Credential` and its limits:** `pub struct Credential { pub value: Box<[u8]>, pub scheme:
  Scheme }`; `Scheme { Bearer, Key }` stays; `Credential::bearer` and `Credential::key` stay.
  `CredentialLimits { value: u32 }`. `Cap::AccountId` goes. Admission keeps refusing what a route
  does not take (2.6).
- **The limits:** `Limits` holds `openai` and `anthropic`. `derive.rs`'s `OPENAI` is written out in
  full where it inherited from `CODEX` (`{ response_fields: 128, ..CODEX }`, ~111-153). Its values
  stay as they are, with a comment saying that they were measured from captures of a Responses
  route and that 7's floor holds the head. A synthetic test pins each one.
- **The replay tag:** `Provider::OpenAiCodex => 1` and `Some(1)` (`replay.rs`, ~52-56, ~75-80) go.
  An envelope tagged 1 is refused like any unknown provider's. Never renumber 2 or 3: stored
  envelopes hold them (the decision: stored tag-1 replays start afresh, no migration).
- **Reserved names stay reserved:** `session-id`, `thread-id` (4.8 says so) and
  `chatgpt-account-id` stay in `reserved()` (`client.rs`, ~1842-1844), so a configured endpoint
  cannot rebuild the route. `affinity::rendered`'s thread XOR and its test go. `Affinity`'s
  thread field (2.1) and `uuid` stay.
- **gzip stays as a mechanism:** `Encoding::Gzip`, the encoder use and the fake's inflater stay.
  No shipped endpoint compresses. The two Codex gzip tests in `client/tests.rs` and the fake's
  gzip tests move to a hand-built `Endpoint { encoding: Encoding::Gzip, ..Endpoint::openai() }`.
- **`response.done`** (`openai/response.rs`, ~773; `dialect.rs`, ~73-80) was the Codex route's
  terminal: it goes, with `openai/tests.rs`'s case (~420). `public_route.rs` (~178) keeps
  asserting it is `Protocol`.
- **`openai::identity` goes,** and `VERBOSITY`, which the public route sends (`translate.rs`
  ~109, `openai/request.rs` ~141), moves into the Responses request code.
- **The recordings' coverage becomes synthetic, labelled so:** the selective-decoding golden
  (`openai/fixtures/provider-completed.json`, read by `openai/tests.rs` ~1370 and
  `skein/tests/llm/tests/suite/selective.rs` ~17) becomes an echo-heavy `response.completed`
  built by the fake's echo. The archived request and answer tests
  (`archived_real_provider_requests_and_answers_match_known_completions`,
  `captured_completion_retains_status_and_usage_without_echoes`,
  `borrowed_encoder_matches_every_archived_codex_request`) go. Drawn tests cover borrowed against
  owned encoding (`translate.rs`, ~869). `every_archived_head_fits_within_a_quarter_of_its_derived_bounds`
  and `archived_heads_fit_with_headroom_and_complete_with_derived_limits` keep their Anthropic
  halves.
- **The shared worlds move to the public route:** `skein_llm_world::call()` and `limits()`
  (`skein/tests/llm/src/lib.rs`, ~31-63) take `Endpoint::openai()` and `Provider::OpenAi`, with
  a bearer and no account id. The suite's tests that rode on them follow. Codex-named tests are
  deleted where they tested only the route (affinity's thread headers, `usage_limit_reached`,
  `x-oai-request-id`, account-id admission, the Codex echo, Codex limits rows), and move to the
  public route where they test the client (fragmentation, reasoning drops, memory, prepared raw,
  failures). `skein-llm-connection` (`src/tests.rs`) and `skein-llm-connection-world` (its default
  endpoint, `simulated.rs`, the suites, `fuzzy/close.rs`) do the same, as do
  `skein-fake-llm-protocol` (the route by account id, `chatgpt-account-id`, the Codex affinity
  check, `usage_limit_reached` and `x-oai-request-id` rows), `skein-fake-peers/src/llm.rs`,
  `skein-fake-peers-tests` and `skein-tls-world`'s `fake_peer.rs`. jig's and temper's worlds read
  `skein_llm_world::limits()` and `fake::config()`, which now change from Codex to public values
  (128 head fields, no gzip, no session headers): run them (Steps).
- **The live suite** (`skein/tests/llm/tests/live.rs`): its `openai_*` tests call
  `Endpoint::openai()` with the lent ChatGPT login's access token as `Credential::bearer`. The
  login is Codex's `auth.json`, as today; the harness no longer reads its account id. That is the
  plan's question `live-logins` default, and nothing waits for it. The README's live section says
  so.

## Owns and may change

skein (owned): the files list, for the edits above. Also `skein/crates/skein-llm/README.md`'s
Codex lines and module docs naming Codex (`lib.rs` ~3, `openai.rs` ~1, `anthropic.rs` ~1,
`translate.rs` ~1, ~42 `validate_codex`, `openai/filter.rs` ~1, `client.rs` ~62, ~123, ~464,
`types.rs` ~49-123), detail strings "ChatGPT stream" (`openai/response.rs`, ~234, ~239, ~316) and
code comments citing text that goes (`affinity.rs` ~2, `openai/common.rs` ~248, `dialect.rs` ~73,
`client.rs` ~1786, `derive.rs` ~111-118, `skein/tests/llm/tests/suite/classes.rs` ~1, `replay.rs`
~97).

smith (the arms and fields only this removal drops; delete only them):

- `Provider::OpenAiCodex` arms: `smith-protocol-llm/src/framing.rs` (~18, ~22, ~40),
  `component.rs` (~264-268), `smith-agent-shell/src/config/endpoints.rs` (~70, ~228),
  `smith-local-shell/src/credentials/product.rs` (~37, or `product::codex` whole if nothing else
  is left in it), `smith-agent-service/src/tests/line.rs` (~166, ~338), smith-protocol-llm-world
  (`src/component_backend.rs` ~114, `src/pool_peer.rs` ~80, ~95; `tests/suite/component_peer.rs`
  ~214, ~231, ~615, `codec.rs` ~588, `content_observations.rs` ~129, `endpoints_grants.rs` ~17,
  `guard.rs` ~50, `history_window.rs` ~79), smith-agent-process-world `src/lib.rs` (~223).
- `CredentialLimits { .. account_id .. }` fields: `smith-agent-shell/src/config/endpoints.rs`
  (~296-300), `smith-agent-service/src/tests/line.rs` (~334), `profile/effective/tests.rs` (~107),
  smith-agent-process-world `src/lib.rs` (~214), smith-protocol-llm-world (`src/component_backend.rs`
  ~131, `src/pool_peer.rs` ~75, `tests/suite/component_peer.rs` ~272, `component_boundary.rs` ~38,
  `endpoints_grants.rs` ~26).
- `skein_llm::Cap::AccountId` arms: `smith-protocol-llm/src/failure.rs` (~232),
  `smith-agent-service/src/profile/words.rs` (~177), and the variant list in
  `smith-protocol-llm/src/tests.rs` (~894).

jig: `jig/tests/channel/src/program_runs.rs` (~92), the line
`destination.credential.account_id = …` goes.

Design and status (as `notes/common.md` says; delete what describes the route, the account id or
its recordings; rewrite only where a sentence about what stays named them):

- `skein/docs/design/llm.md`:
  - The intro's "**A withdrawn route.**" paragraph goes. 1's bullets lose "on its public route and
    on the withdrawn Codex route" (two dialects: OpenAI's Responses and Anthropic's Messages) and
    "A credential carries no account id, the withdrawn Codex route's aside" (it becomes "A
    credential carries no account id").
  - "(sections 4.6 to 4.8)" in 2.5 and elsewhere (~169, ~285) becomes "(sections 4.7 and 4.8)".
  - 2.6: the struct loses its `account_id` line. "the field `account_id` is the withdrawn Codex
    route's alone (below)" goes. "The withdrawn Codex route refuses a call without one at
    admission, as `Invalid`." goes. The "**No account id.**" bullet goes. "**Its bounds.**" loses
    "and the account id" and "either". "**Reserved names.**" becomes "`Authorization` and
    `x-api-key` are credential fields, and `chatgpt-account-id` stays reserved: an endpoint's
    configured headers may not use them, as they may not use the framing fields."
  - 4: "Limits are per route: `Limits` holds a set for OpenAI's public route (`openai`) and for
    Anthropic's (`anthropic`), as their heads, events, replay and output caps differ."
  - 4.6: deleted whole, heading included. The number stays unused; 4.7 and 4.8 keep theirs.
  - 4.8: "Both OpenAI Responses routes accept a missing `Content-Type`" becomes "The public route
    accepts a missing `Content-Type`". "`Endpoint::codex()` compresses: ... slow uplink." goes,
    leaving "`Endpoint::openai()` does not, until a capture shows the public endpoint takes it. A
    local model's server never does." The failure paragraph's "neither this section nor, on the
    Codex route, section 4.6 names" becomes "this section does not name".
  - 5: Affinity's Codex clause goes ("the public route carries the key in the body alone, with
    neither header; no affinity sends none"). Failure classes lose "the Codex route's
    `usage_limit_reached` in both" and "nor, on the Codex route, section 4.6". Credentials lose
    "and required on the Codex route", `chatgpt-account-id` from the refused headers (it stays
    refused, as a reserved name), "or account id", and "and is refused on the Codex route".
    Selective decoding: "A real-structure Codex `response.completed` event is kept as a golden
    event." becomes "A synthetic echo-heavy `response.completed` event, labelled so, is kept as
    a golden event."
  - 6, "Routing state": "The Codex route's private turn-state token ... and its
    `reasoning.context`" becomes "A Responses route's turn-state token ... and its
    `reasoning.context`, which ChatGPT's backend offers,". Keep the rest.
  - 7: "Codex's thread rendering" goes whole. The head bound's "16 KiB and 168 fields for the
    Codex route," goes. "Until then both OpenAI routes render `Auto`" becomes "Until then the
    public route renders `Auto`". Any other Codex clause goes.
- `skein/docs/design/llm-connection.md` (~92 "and account id"; ~177-178; ~190-193): the account
  id and the Codex route's refusal go. A credential the route does not take is "a bearer on
  Anthropic".
- `skein/docs/design/fake-llm.md`: every Codex line (~125 the citation, ~132-144, ~263-267 the
  paragraph citing `llm.md, section 4.6`, ~271, ~307, ~315-317, ~326, ~357-358, ~481-482) goes or
  names the public route.
- `skein/docs/design/llm.status.md`: `No gap.` Re-date it.
- `skein/docs/design/STATUS.md`: the clause "the withdrawn Codex route still stands (`llm.md`)"
  goes, the sentence mended. Re-date it.

Pre-approved options:

- If `citations.py` lists a citation of `llm.md` 4.6 in a file outside this list, change it to the
  section that now holds the matter (4.8, or 2.6), or delete the clause that cited it. List each.
  smith's four were removed by `smith-agent-llm`.
- If a smith, jig or temper test changes behaviour because a shared world moved from Codex to
  public limits (a header-field bound, gzip, session headers), fix the test's expectation in that
  test only and list it. A memory ceiling only falls; if one must rise, stop and ask.
- If a decoder constant's synthetic pin fails at the value `OPENAI` inherited, keep the value and
  make the pin match the code. Say so: the constants are not to move in this task.
- `skein-oauth`'s test data naming `chatgpt_account_id` (`src/tests.rs`, ~40) is a generic claim
  selector's example: leave it.

## Steps

1. skein-llm: types, client, derive, replay, dialect, translate, affinity, the OpenAI module, the
   recordings' deletion and their synthetic replacements:
   `heavy cargo nextest run -p skein-llm --lib`.
2. The fake peer and connection: `heavy cargo nextest run -p skein-fake-llm-protocol --lib`,
   `heavy cargo nextest run -p skein-fake-peers --lib`, `heavy cargo nextest run -p skein-llm-connection --lib`.
3. The worlds, one at a time, each count not zero:
   - `heavy cargo nextest run -p skein-llm-world --test suite`
   - `heavy cargo check -p skein-llm-world --tests`
   - `heavy cargo nextest run -p skein-llm-connection-world --test suite`
   - `heavy cargo nextest run -p skein-fake-peers-tests --test suite -E 'test(/llm/)'`
   - `heavy cargo nextest run -p skein-tls-world --test suite -E 'test(/fake_peer/)'`
   - `heavy cargo nextest run --profile fuzzy -p skein-llm-world` and
     `heavy cargo nextest run --profile fuzzy -p skein-llm-connection-world`
4. Cross-tree readers: `heavy cargo check --tests` on `smith`, `smith-bench`, `smith-agent-service`,
   `smith-agent-shell`, `smith-local-shell`, `smith-protocol-llm`, `smith-protocol-program`,
   `smith-agent-world`, `smith-agent-process-world`, `smith-local-process-world`,
   `smith-protocol-llm-world`, `smith-session-world`, `jig-test`, `jig-channel-world`,
   `jig-one-process-world`, `temper`, `temper-fake`, `temper-one-process-world`,
   `temper-real-loop-world` (one `-p` each, in one command). Then, a change of meaning (the shared
   worlds' defaults), one at a time:
   `heavy cargo nextest run -p smith-protocol-llm --lib`, `-p smith-protocol-llm-world --test suite`,
   `-p smith-agent-service --lib`, `-p smith-agent-shell --lib`, `-p smith-agent-process-world --test suite`,
   `-p jig-channel-world --test suite`, `-p jig-one-process-world --test suite`,
   `-p temper-one-process-world --test suite`.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py --write` in `skein/` if a count fell, and the
   check without `--write` in `smith/` and `jig/`; `heavy cargo clippy` with `--all-targets -- -D warnings`
   on `skein-llm`, `skein-fake-llm-protocol`, `skein-fake-peers`, `skein-llm-connection`,
   `skein-llm-world`, `skein-llm-connection-world`, `skein-fake-peers-tests`, `skein-tls-world`,
   `smith-protocol-llm`, `smith-agent-shell`, `smith-agent-service`, `smith-local-shell`,
   `smith-protocol-llm-world`, `smith-agent-process-world`, `jig-channel-world`.
6. The design and status edits; check by hand that each remaining `fixture-readers.toml` reader
   of the files you touched still selects a test (`--list` its filter);
   `python3 -I -B tools/citations/citations.py --base origin/main`; the acceptance's `rg`.
7. Commits: the recordings' deletion with their readers' replacement; the route and account id in
   skein-llm; the fake peer, connection and worlds; smith's and jig's lines; the design and status.
   Push your branch.

## Report

As the implementer's text says, and also: each test deleted and each moved to the public route,
by file; each synthetic case added and what capture it stands for; the constants written out in
`OPENAI`; every cross-tree expectation changed, with old and new values; what the acceptance's
`rg` leaves.
