+++
id = "smith-product"
part = "smith-product"
kind = "migration"
needs = []
files = [
  "smith/crates/smith-local-shell/**",
  "smith/crates/smith-local-service/**",
  "smith/crates/smith-local-protocol/src/borrowed.rs",
  "smith/tests/local-process/**",
  "smith/tests/end-to-end/**",
  "smith/tests/agent-process/src/fake.rs",
  "smith/benchmarks/src/result.rs",
  "smith/benchmarks/src/agent/smith/prepare.rs",
  "smith/benchmarks/tests/suite/smith_adapter.rs",
  "smith/benchmarks/tests/suite/codex_adapter.rs",
  "smith/benchmarks/tests/recorded/smith-product-check.json",
  "smith/docs/design/shell.status.md",
  "smith/docs/design/STATUS.md",
  "smith/docs/design/protocol/channel.status.md",
  "smith/docs/design/protocol/limits.md",
]
effort = "high"
minutes = 180
+++

# smith's product off the withdrawn forms: no codex preset, no identity, no account id

## Goal

smith's product (the local shell and the local service) stops using every withdrawn form, as
`smith/docs/design/shell.md` already says. The built-in `codex` preset, its endpoint, its account
and its `codex/<name>` models go. The `anthropic` preset loses `identity = 'claude-code'`.
Settings refuse an endpoint's `identity`, `dialect = "codex"` and the `codex` endpoint with "use
`openai/<model>`" (5.3). The agent's configuration the shell writes carries no dialect 1 and no
`identity` (5.4). No account reads a token's `chatgpt_account_id` claim. The local service lends
a sign-in's token as a bearer and an `env` value as a key, so that on `anthropic` an `env` key
goes in `x-api-key` (6.2, 7.1), and it never sends the withdrawn id-and-bearer grant. smith's
product then runs on the `openai` and `anthropic` presets alone, and smith's agent, channel and
skein can remove the forms after it.

The variants `GrantValue::Withdrawn` (`smith-protocol-translate`) and `Form::Withdrawn`
(`smith-host-domain`) and skein's `Provider::OpenAiCodex` still exist when this task runs. Their
arms in this task's exhaustive matches stay, unreachable, for smith's channel task and skein's
task to delete.

Acceptance:

- `rg -n -i "codex" smith/crates/smith-local-shell smith/crates/smith-local-service smith/crates/smith-local-protocol --glob '!**/tests*'`
  finds only the refusal of 5.3 (its words and the keys it names) and the exhaustive arms kept
  above (`credentials/product.rs`'s `Provider::OpenAiCodex` arm, if `product::codex` is not
  deleted whole). No preset, model, dialect, selector or flag is left. The same search with
  `chatgpt_account_id`, `account_id`, `claude-code` and `identity` finds no product reader.
- Settings naming `endpoints.<name>.identity`, `dialect = "codex"`, `preset = "codex"` or an
  endpoint named `codex` (an `[endpoints.codex]` table or a `codex/<model>`) are refused at
  their key with exit 2 and "use `openai/<model>`". A test shows each.
- The local service lends `GrantValue::Bearer` for a sign-in and `GrantValue::Key` with
  `Form::Credential(Scheme::Key)` for an `env` account, and never `Withdrawn`. A test shows
  both.
- The status entries below are updated. The tests in Steps pass, each count not zero, and
  clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `smith/AGENTS.md` (its module-doc rules, its `clippy.toml`).
- `smith/docs/design/shell.md`: 5.2 (the presets; "Neither names an account" is a separate gap,
  below), 5.3 ("Settings of an earlier version are refused": an endpoint's `identity`,
  `dialect = "codex"`, and the `codex` endpoint, "use `openai/<model>`"), 5.4 (the generated
  agent configuration), 6.2 (per provider: `openai` sends a sign-in's token or a key alike as a
  bearer, `anthropic` takes a key in `x-api-key`), 7.1 (sources: `env` is the variable's
  value).
- Status (the spec): `smith/docs/design/shell.status.md`, the bullet "5.2, 5.3, 5.4, 6.2, 7.1:
  the product still runs on the withdrawn forms ...".
- `smith/docs/design/protocol/agent.md`, 4 (dialects: `openai` 3, `anthropic` 2), and
  `protocol/channel.md`, 6 (the grant forms).
- `skein/docs/design/llm.md`, 2.6 and 4.8: a key on the public route is sent as `Authorization:
  Bearer`, as a token is (`skein-llm/src/client.rs`, ~1670-1674), so lending an `env` value on
  `openai` as a key changes nothing on that wire.

## The lead's calls (no question needed)

- **The `codex` preset's replacement is the `openai` preset,** which already runs the public
  route with smith's ChatGPT sign-in (`credentials/registration.rs`, ~85, `metadata_claim:
  None`) and lends it as `GrantValue::Bearer`. Tests that used `codex/<model>`, `[endpoints.codex]`
  or `preset='codex'` move to `openai/<model>` with an account whose `provider = 'openai'`.
  Where `codex` is only an account's name in a test (`credentials/tests.rs`'s `resolved`,
  `account_report/tests.rs`, `numbers/tests.rs`, `local_host/lend/tests.rs`) it may stay.
- **The refusal is 5.3's, at the strict layer:** exit 2, naming file, line and key, as
  `withdrawn_source_settings_name_the_replacement` does (`settings/sources_tests/validation.rs`,
  ~93; `settings/resolve/providers.rs`, ~18-27). `identity` stays in `settings/schema.rs` so that
  the refusal fires before the unknown-key one, as the withdrawn `file` and `format` keys stay.
  `account` is a setting that stays (5.2).
- **An `env` account lends a key.** The service chooses the grant by the account's origin: a
  sign-in lends `GrantValue::Bearer` with `Form::Credential(Scheme::Bearer)`; an `env` value lends
  `GrantValue::Key` with `Form::Credential(Scheme::Key)`, not renewable. `Accounts::account_id`
  (`smith-local-service/src/credentials.rs`, ~486-492) becomes a per-account scheme lookup that
  also serves `service.rs` (~957, "is this a driver account"). `Source::SignIn.account_id`,
  `Source::account_id_bytes`, `Account.identifier`, `Origin::SignIn.claim`, `claim_identifier`
  and `Borrowed.account_id` go. An `env` value's expiry came only from the Codex claim
  (`credentials.rs`, ~201-204): with the claim gone, an `env` grant announces no expiry, and a
  test of that expiry goes with it.
- **No stored format changes.** `tokens/<account>` records (skein-oauth `SavedToken`, written
  with `metadata: None`), key files and `host-id` are untouched; skein's
  `Registration.metadata_claim` stays, and the shell always sets `None`. A user's settings that
  name the `codex` preset are refused (above); a chat started on `codex/<model>` is not
  migrated.
- **The shared fake** in `smith/tests/agent-process/src/fake.rs` gains a way to serve the public
  route, `/v1/responses` (as `smith/tests/local-process/src/world/key.rs` already does), so that
  the product's worlds move off `skein_llm_world::call(1)` (`Endpoint::codex()`). Keep its other
  behaviour; smith's channel task changes it next.

## Owns and may change

smith, the product (owned):

- `smith-local-shell`:
  - `src/presets.rs` (`Preset::Codex`, its settings text, the `codex/*` models in `MODELS`;
    `list_prices` keyed on `openai`, not mapped to `codex`; its test at ~179);
  - `src/settings/layers.rs` (~69);
  - `src/settings/schema.rs` (~78, `identity` kept for the refusal);
  - `src/settings/resolve/{inherit.rs (~22), document.rs (~87 the `codex` dialect, ~100-106 the
    copied `identity`), models.rs (~74), providers.rs (the refusal)}`;
  - `src/settings/built-in.golden`: the `endpoints.codex.*`, `models."codex/*"` and
    `endpoints.anthropic.identity` lines go. It is asserted whole by
    `the_built_in_layer_matches_its_golden`, with no write mode: rewrite it to the test's
    output;
  - `src/credentials.rs` (`Source::Env { codex }`, `codex_selector`, the `codex` parameters of
    `source`, `inspect`, `env_state`, `environment`, `env_record`);
  - `src/credentials/{product.rs, sign_in.rs (~43), account_report.rs (~93), login.rs, logout.rs,
    login/provider.rs}` (the `codex` flag and its callers);
  - `src/settings/resolve.rs` (~124);
  - `src/local_host/lend.rs` (~42: the `Withdrawn` arm stays);
  - their tests: `settings/tests.rs`, `settings/providers_tests.rs` (~18), `settings/sources_tests*`,
    `check/{catalog_tests.rs, tests.rs, program_tests.rs}`, `compose/{tests.rs, spawned.rs (~71),
    placement.rs}`, `credentials/{tests.rs (~45: the Codex claim case goes), login.rs (~156)}`.
    Moved to `openai`, as above; `refresh_profile` (`settings/tests.rs`, ~615) takes
    `Provider::OpenAi`. Add the 5.3 refusal test beside `withdrawn_source_settings_name_the_replacement`.
- `smith-local-service`: `src/service/accounts.rs` (~56-65), `src/credentials.rs` (as above),
  `src/credentials/{terminals.rs, adopt.rs}`, `src/service.rs` (~479-480 sizes, ~957; the
  `Withdrawn` arm at ~479 stays), and their tests: `credentials/tests.rs` (~124, ~141, ~163, the
  `borrowed()` helper), `credentials/tests/private.rs` (~160-169; ~381
  `a_loaded_private_grant_supplies_its_account_id_from_the_claims` is deleted),
  `service/tests.rs` (~20: the `Withdrawn` row goes), `tests.rs` (~735 `colocated_grant` lends a
  bearer; its charter's `dialect: 1`, ~109, and `service/tests/output.rs`, ~45, become 3).
- `smith-local-protocol/src/borrowed.rs` (`Borrowed.account_id`).

smith, the product's worlds and readers (owned for this task):

- `smith/tests/local-process/**` (`smith-local-process-world`; `smith-real-world` reuses it):
  `src/process.rs` (~830-856), `src/process/configuration.rs` (~366-374 the endpoint becomes
  dialect 3, provider `openai`, with no `identity` and no `account`; ~445, ~457),
  `src/world.rs` (~1131, ~1144-1148), `src/world/lending.rs` (~26), `tests/suite/simulation.rs`
  (~176, words only). Leave `tests/suite/fuzzy/store.rs`'s dialect-1 charter (~243) to smith's
  channel task, unless your change turns it red.
- `smith/tests/agent-process/src/fake.rs` (~93, ~193-197), as above.
- `smith/tests/end-to-end/**` (the root package `smith`, `--features end-to-end`):
  `end_to_end.rs`, `credentials.rs`, `chat_crossing.rs`, `support/{mod.rs, exec.rs, window.rs,
  sample_host.rs, live_run.rs}`, `live_tests.rs`, `live_report.rs`: `codex/*` becomes `openai/*`;
  each skein-llm `Credential { .. account_id .. }` literal becomes `Credential::bearer` or
  `Credential::key`; the live tests' allowances of the built-in `codex` and `anthropic` warnings
  go. A hand-written agent document that checks only a refusal (`end_to_end.rs`, ~203) may stay
  at dialect 1.
- smith's benchmarks (`smith-bench`), the product's pins only: `src/result.rs` (~320) and
  `tests/suite/codex_adapter.rs` (~821) price with `"openai"`; `tests/suite/smith_adapter.rs`
  (~449) drops `"codex"`; `src/agent/smith/prepare.rs` (~342, ~367, ~387) `codex/gpt-6-luna`
  becomes `openai/<the same model>`; `tests/recorded/smith-product-check.json`: the keys this task
  removes (`endpoints.codex.*`, `models.codex/*`, `accounts.codex.*`,
  `endpoints.anthropic.identity`) are deleted by hand, and the commit says so. It is smith's own
  check output, not a provider's recording. The benchmarks' own Codex arm (`Provider::Codex`,
  `src/credential.rs`, the Codex CLI's credentials) is a different program and stays.

Design and status (as `notes/common.md` says):

- `smith/docs/design/shell.status.md`: delete the bullet "5.2, 5.3, 5.4, 6.2, 7.1: the product
  still runs on the withdrawn forms ..."; the `--account` bullet and the bullet "5.2: the
  `anthropic` preset names an account ..." stay; the date line today's.
- `smith/docs/design/STATUS.md`: the sentence on the withdrawn forms loses "and smith's product
  still runs on them (its `codex` preset, the `anthropic` preset's identity and the ChatGPT
  account id)" and `shell.md` from its list. It then reads "... are still accepted, since jig's
  worlds still write dialect 1, and their removal is not yet (`protocol/agent.md`,
  `protocol/channel.md`, `protocol/hosts.md`, `protocol/llm.md`)." Leave the rest of the
  paragraph alone.
- `smith/docs/design/protocol/channel.status.md`: delete ", smith's local service still sends it
  for a Codex endpoint's account (shell.status.md)", so that the entry says the agent's half
  still installs it and "no host sends it, but its dialect goes with the `codex` one".
- `smith/docs/design/protocol/limits.md` (~856): an example naming `endpoint=codex` takes
  `endpoint=openai` if the product's check output now says so. Change no heading.

Pre-approved options:

- If another smith file outside the product names a product item this task removes (a preset, the
  `codex` flag, `Borrowed.account_id`, `Source::account_id_bytes`), edit it only to drop or rename
  that item and list it.
- If the anthropic preset's own account (`account = 'anthropic'`, `[accounts.anthropic]
  variable = 'ANTHROPIC_API_KEY'`) blocks a test of this task, leave it as it is: it is
  shell.status.md's separate 5.2 gap, outside this plan. Do not remove it.
- If moving a world to the public route needs a fake-peer option that `fake.rs` cannot give
  without a new public item in a non-test crate, stop and ask.
- Leave to smith's channel task: `smith/tests/hosts/src/lib.rs` (~471-495, it builds
  `GrantValue::Withdrawn` itself), the agent-process, channel, codecs and protocol-llm worlds,
  `smith/tests/local/src/world.rs` (~104, ~504) and every `smith-agent-*`, `smith-protocol-*`,
  `smith-host-*`, `smith-channel` file.

## Steps

1. The presets, schema and resolution, with the 5.3 refusal and its test; the golden:
   `heavy cargo nextest run -p smith-local-shell --lib`.
2. The credentials: the `codex` flag, the selector and the sign-in's claim; then the service's
   grants by origin and its account id's removal:
   `heavy cargo nextest run -p smith-local-service --lib`, `heavy cargo nextest run -p smith-local-protocol --lib`.
3. The worlds and readers, then, one at a time, each count not zero:
   - `heavy cargo nextest run -p smith-local-shell --test suite`
   - `heavy cargo nextest run -p smith-local-process-world --test suite`
   - `heavy cargo nextest run -p smith-real-world --test suite`
   - `heavy cargo nextest run -p smith --features end-to-end --test end_to_end`
   - `heavy cargo nextest run -p smith --test live_configuration`
   - `heavy cargo check -p smith --tests` (the `live` binary builds; it is the owner's to run)
   - `heavy cargo nextest run -p smith-bench --lib` and `heavy cargo nextest run -p smith-bench --test suite`
   - `heavy cargo nextest run -p smith-agent-process-world --test suite` (it shares `fake.rs`)
4. A change of meaning (an `env` account lends a key): also
   `heavy cargo nextest run -p smith-local-domain --lib` (its `parent_tests.rs` reads `shell.md`).
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `smith/`;
   `heavy cargo clippy -p smith-local-shell -p smith-local-service -p smith-local-protocol -p smith-local-process-world -p smith-agent-process-world -p smith-real-world -p smith-bench --all-targets -- -D warnings`
   and `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings`.
6. The status edits; the acceptance's `rg`.
7. Commits: the presets and settings; the credentials and the service's grants; the worlds,
   end-to-end and benchmarks; the status. Push your branch.

## Report

As the implementer's text says, and also: each test moved from `codex` to `openai` and each one
deleted, by file; how the service now chooses a grant's form; the keys removed from the
recorded check; what the acceptance's `rg` leaves, by file; any live allowance removed.
