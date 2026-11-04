+++
id = "skein-anthropic-bearer"
part = "skein-routes"
kind = "migration"
needs = ["smith-agent-llm"]
files = [
  "skein/crates/skein-llm/src/anthropic.rs",
  "skein/crates/skein-llm/src/anthropic/**",
  "skein/crates/skein-llm/src/openai/common.rs",
  "skein/crates/skein-llm/src/types.rs",
  "skein/crates/skein-llm/src/client.rs",
  "skein/crates/skein-llm/src/client/tests.rs",
  "skein/crates/skein-llm/src/lib.rs",
  "skein/crates/skein-llm/README.md",
  "skein/crates/skein-fake-llm-protocol/src/provider/credentials.rs",
  "skein/crates/skein-fake-llm-protocol/src/provider/failure.rs",
  "skein/tests/llm/**",
  "skein/tests/fake-peers/tests/suite/llm.rs",
  "skein/docs/design/llm.md",
  "skein/docs/design/llm.status.md",
  "skein/docs/design/fake-llm.md",
  "skein/docs/design/STATUS.md",
]
effort = "high"
minutes = 120
+++

# skein-llm: Anthropic's bearer route and its Claude Code identity profile removed

## Goal

skein-llm's Anthropic dialect takes a key alone (`skein/docs/design/llm.md`, 4.7: "a key replaces
them, and they are removed once their last caller has moved"; smith's LLM component, after
`smith-agent-llm`, installs a key alone on Anthropic). A bearer on Anthropic is refused at
admission as `Invalid`. The OAuth beta flag goes, and with it `Credential::anthropic`, the
archived Claude Code identity profile (`anthropic::identity`, with the encoder's split of an
identity block) and the plan-limit rule read from `anthropic-ratelimit-unified-status`. The
`Credential` struct keeps its shape here. The Codex route and the account id are skein's next
task, `skein-codex-route`.

Acceptance:

- `Endpoint::anthropic()` with `Credential::bearer(..)` is refused at admission as `Invalid`, and
  with `Credential::key(..)` sends `x-api-key` with neither `Authorization` nor `anthropic-beta`.
  A call with no credential still sends neither, with `anthropic-version`. Focused tests show
  each.
- `rg -n 'Credential::anthropic|anthropic::identity|oauth-2025-04-20|ratelimit-unified' skein smith jig temper --glob '!**/docs/plan/**' --glob '!**/docs/development/**' --glob '!**/fixtures/**'`
  finds nothing. The Anthropic recordings under `anthropic/fixtures/` stay, and their README says
  which route recorded them.
- The design and status edits below are made. The tests in Steps pass, each count not zero, and
  clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `skein/AGENTS.md`.
- Design: `skein/docs/design/llm.md`: the intro's "Withdrawn routes", 2.6 (credentials), 4.7
  (the Anthropic dialect: the bearer bullet, "Breakpoints", "Failures"), 5 ("Credentials",
  "Failure classes"); `skein/docs/design/fake-llm.md` (the peer's Anthropic bearer lines, ~132-144
  and ~249-256).
- Status (the spec): `skein/docs/design/llm.status.md` (4.6, 4.7), `skein/docs/design/STATUS.md`
  ("the withdrawn LLM routes still stand while smith uses them").

## The lead's calls (no question needed)

- **A bearer on Anthropic is `Invalid` at admission,** by the rule 2.6 already gives an account id a
  route does not take. `request_head` (`client.rs`, ~137, ~167) measures Anthropic with its key's
  `x-api-key`, not a bearer. The Anthropic arms of `client.rs` (~1674, ~1748) render a key alone,
  and the OAuth beta (~1786-1789) goes.
- **The identity profile goes whole:** `anthropic/identity.rs`, its module line, the encoder's
  identity split (`anthropic/request.rs`, ~4, ~377-392), and its tests
  (`anthropic/request_tests.rs`: `historical_identity_is_explicit_and_is_the_first_separate_system_block`,
  `explicit_identity_blocks_remain_subject_to_instruction_and_wire_limits`, the identity uses
  ~271-325, ~443). Instructions that happen to begin with Claude Code's words are ordinary
  instructions.
- **The unified plan limit goes** (`openai/common.rs`, ~217-220, ~327-339, ~359): 4.7 ties it to
  the withdrawn route. `rate_limit_error` is `RateLimited`, and `billing_error` stays `Exhausted`
  with no reset.
- **The worlds take a key:** `skein_llm_world::anthropic_call()` (`skein/tests/llm/src/lib.rs`,
  ~76-83) grants `Credential::key`. In `skein/tests/llm/tests/suite/`, `anthropic.rs`'s
  `oauth_request_text_usage_and_http_fragmentation` keeps its text, usage and fragmentation
  checks on a key and drops its OAuth assertions. `anthropic_admission.rs`'s identity headers
  (~58, ~68, ~74), `credentials.rs`, `archives.rs` (~85, `Credential::anthropic`), `fake.rs` and
  `fake_failures.rs` follow, with a new case: a bearer is refused. The fake peer drops its
  Anthropic bearer reset (`provider/failure.rs`, ~60-70, ~120-131, the test
  `an_anthropic_reset_is_refused_off_a_first_bearer_line`, ~302-309) and its bearer acceptance on
  Anthropic (`provider/credentials.rs`).
- **The live suite** (`skein/tests/llm/tests/live.rs`): its `anthropic_*` tests take a key from a
  file the owner names in `SKEIN_TEST_LIVE_ANTHROPIC_KEY` (the file 0600, read once, never
  printed; the same `lent` reading the login files use), sent as `Credential::key` on
  `Endpoint::anthropic()`. The Claude Code login reader (`CLAUDE_CODE`, its `.credentials.json`
  fields) goes. `skein-llm/README.md`'s live section says so. Which login the owner lends is the
  plan's question `live-logins`: this is its default, and nothing here waits for it.
- **The recordings stay:** `anthropic/fixtures/*` are read as Messages grammar
  (`.config/fixture-readers.toml`, ~152-187). Their README gains one line: they were recorded on
  the subscription route with Claude Code's betas and identity, and are read for their response
  grammar and as historical requests.

## Owns and may change

skein (owned): the files list, for the edits above, and `skein/tests/fake-peers/tests/suite/llm.rs`
for its Anthropic bearer cases.

Design and status (as `notes/common.md` says):

- `skein/docs/design/llm.md`:
  - The intro's "**Withdrawn routes.**" paragraph becomes "**A withdrawn route.** OpenAI's Codex
    route (section 4.6) is withdrawn; its section says what replaces it."
  - 2.6: "Anthropic takes a key in `x-api-key`, with no authorization field; a bearer there is the
    withdrawn subscription route (section 4.7)." becomes "Anthropic takes a key in `x-api-key`,
    with no authorization field, and refuses a bearer at admission, as `Invalid`."
  - 4.7: the bullet "**The bearer route, withdrawn.** ..." goes whole. "Breakpoints": "`system` is
    a block array, the withdrawn identity profile's block first when a caller opts in, and its
    last block carries ..." becomes "`system` is a block array, and its last block carries ...".
    "Failures": "`rate_limit_error` is `RateLimited`, or `Exhausted` where the head's
    `anthropic-ratelimit-unified-status` is `rejected` (the withdrawn route's plan limit), its
    reset from `anthropic-ratelimit-unified-reset`;" becomes "`rate_limit_error` is
    `RateLimited`;".
  - 5, "Credentials": "a key on Anthropic goes as `x-api-key` with neither an authorization field
    nor a beta, and a bearer with both;" becomes "a key on Anthropic goes as `x-api-key` with
    neither an authorization field nor a beta, and a bearer is refused at admission;".
- `skein/docs/design/fake-llm.md`: the sentences on Anthropic's withdrawn bearer route (~132-144:
  "On Anthropic only the withdrawn bearer route carries one" and its reset; ~249-256: the bearer
  "(the withdrawn route, llm.md, section 4.7)") are deleted or say that the peer refuses a
  bearer on Anthropic, as the client does. Leave its Codex sentences to the next task.
- `skein/docs/design/llm.status.md`: the entry becomes "- **Not yet:** 4.6: removal of the
  withdrawn Codex route, with its account id (2.6) and its recordings." Re-date it.
- `skein/docs/design/STATUS.md`: "the withdrawn LLM routes still stand while smith uses them
  (`llm.md`)" becomes "the withdrawn Codex route still stands (`llm.md`)". Re-date it.

Pre-approved options:

- If a test elsewhere in skein, smith, jig or temper grants a bearer on Anthropic, or names
  `Credential::anthropic`, switch it to `Credential::key` and list it (smith's worlds should
  have none after `smith-agent-llm`).
- If a measured head bound or memory ceiling moves because Anthropic's head now holds `x-api-key`
  rather than a bearer and a beta, recompute it from its test and list the old and new values. A
  ceiling only falls; if one must rise, stop and ask.
- Leave every Codex, account-id and `openai::identity` item to `skein-codex-route`.

## Steps

1. The client: admission, head measure, rendering, the identity profile, the unified rule:
   `heavy cargo nextest run -p skein-llm --lib`.
2. The fake peer: `heavy cargo nextest run -p skein-fake-llm-protocol --lib`.
3. The worlds, one at a time, each count not zero:
   - `heavy cargo nextest run -p skein-llm-world --test suite`
   - `heavy cargo check -p skein-llm-world --tests` (the `live` binary builds; it is the owner's)
   - `heavy cargo nextest run -p skein-fake-peers-tests --test suite -E 'test(/llm/)'`
   - `heavy cargo nextest run --profile fuzzy -p skein-llm-world`
4. A change of meaning (a bearer on Anthropic is refused): cross-tree readers,
   `heavy cargo check -p smith-protocol-llm -p smith-protocol-llm-world -p smith-agent-service -p smith-agent-process-world -p jig-channel-world -p jig-one-process-world -p temper-one-process-world --tests`,
   then `heavy cargo nextest run -p smith-protocol-llm-world --test suite` and
   `heavy cargo nextest run -p jig-channel-world --test suite`.
5. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/` (`--write` if a count fell);
   `heavy cargo clippy -p skein-llm -p skein-fake-llm-protocol -p skein-llm-world -p skein-fake-peers-tests --all-targets -- -D warnings`.
6. The design and status edits; `python3 -I -B tools/citations/citations.py --base origin/main`;
   the acceptance's `rg`.
7. Commits: the client and its tests; the fake peer and worlds; the live suite and README; the
   design and status. Push your branch.

## Report

As the implementer's text says, and also: each test changed or deleted, by file; any bound or
ceiling that moved; the live suite's new variable and what the README says of it.
