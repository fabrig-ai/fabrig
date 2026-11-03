+++
id = "smith-channel-v6"
part = "smith-agent-channel"
kind = "migration"
needs = ["smith-agent-llm"]
files = [
  "smith/crates/smith-channel/**",
  "smith/crates/smith-protocol-translate/src/**",
  "smith/crates/smith-protocol-channel/src/**",
  "smith/crates/smith-host-protocol/src/**",
  "smith/crates/smith-host-domain/src/**",
  "smith/crates/smith-domain/src/**",
  "smith/crates/smith-domain-run/src/**",
  "smith/crates/smith-domain-session/src/**",
  "smith/crates/smith-protocol-events/src/**",
  "smith/crates/smith-local-protocol/src/**",
  "smith/crates/smith-inline-agent/src/**",
  "smith/crates/smith-protocol-llm/src/failure.rs",
  "smith/crates/smith-agent-service/src/service.rs",
  "smith/crates/smith-agent-service/src/service/**",
  "smith/crates/smith-agent-service/src/profile.rs",
  "smith/crates/smith-agent-service/src/profile/effective/channel.rs",
  "smith/crates/smith-agent-shell/src/agent_shell.rs",
  "smith/crates/smith-agent-shell/src/config/endpoints.rs",
  "smith/crates/smith-local-service/src/**",
  "smith/crates/smith-local-shell/src/startup.rs",
  "smith/crates/smith-local-shell/src/local_host/lend.rs",
  "smith/tests/codecs/**",
  "smith/tests/channel/**",
  "smith/tests/hosts/**",
  "smith/tests/agent-process/**",
  "smith/tests/end-to-end/end_to_end.rs",
  "jig/testing/jig-test-service/src/worker.rs",
  "jig/tests/channel/src/worker_orders/probe.rs",
  "temper/crates/temper-service/src/worker.rs",
  "temper/crates/temper-service/src/tests/worker.rs",
  "temper/crates/temper-worker-domain/src/translate.rs",
  "temper/crates/temper-worker-domain/src/tests/translate.rs",
  "temper/docs/design/domain/worker.md",
  "temper/docs/design/protocol/credentials.md",
  "temper/docs/design/protocol/channel.status.md",
  "smith/docs/design/protocol/channel.md",
  "smith/docs/design/protocol/hosts.md",
  "smith/docs/design/protocol/README.md",
  "smith/docs/design/STATUS.md",
  "smith/docs/design/protocol/channel.status.md",
  "smith/docs/design/protocol/hosts.status.md",
]
effort = "high"
minutes = 150
+++

# smith's channel at v6: the withdrawn id-and-bearer grant and `Cap::AccountId` leave

## Goal

smith's channel becomes version 6 (`smith/docs/design/protocol/README.md`, 8: a change a reader
notices is a new version, nothing is translated, and the two halves move together). v6 is v5
without the withdrawn grant value, `GrantValue`'s `withdrawn: WithdrawnValue` arm (tag 2, the
last arm), and without `Cap`'s `account_id` (tag 20, the last arm). Nothing renumbers. v5 is
renamed to v6, not kept, as v4 became v5 (smith's `d9a67acabd`: schema, generated module and
goldens renamed; `kinds.rs` and `lib.rs` follow). Every half, translation and reader in smith,
jig and temper drops the two forms: `GrantValue::Withdrawn`, `Form::Withdrawn`, `WithdrawnValue`
and the `Cap::AccountId` chain. This is the second half of the part. After it, no smith, jig or
temper code sends or reads an account id on the channel.

Acceptance:

- `smith/crates/smith-channel/schema/channel-v6.schema` exists, beginning
  `family smith-channel 6`. It has no `WithdrawnValue` record, no `withdrawn` arm and no `Cap`
  `account_id`. `channel-v5.schema`, `src/generated/v5.rs` and `golden/v5/` are gone.
  `KindRule::FORMAT_VERSION` is 6. The drift test passes without the write mode.
- `rg -n 'WithdrawnValue|GrantValue::Withdrawn|Form::Withdrawn|withdrawn_value|smith_channel::v5|Cap::AccountId|channel-v5' smith jig temper --glob '!**/docs/plan/**' --glob '!**/docs/development/**'`
  finds only skein-llm's own `skein_llm::Cap::AccountId` arms. `smith-protocol-llm/src/failure.rs`
  (~232) and `smith-agent-service/src/profile/words.rs` (~177) keep them for skein's task.
  `smith/benchmarks/tests/recorded/smith-product-check.json` stays a recording.
- The design and status edits below are made. The tests in Steps pass, each count not zero, and
  clippy is clean.

## Inputs

- `temper/docs/plan/withdrawn-forms/notes/common.md`.
- `smith/AGENTS.md`; `jig/AGENTS.md` and `temper/AGENTS.md` for the readers.
- Design: `smith/docs/design/protocol/README.md`, 8 (versions); `protocol/channel.md`, 2 (the
  version) and 6 (grants and their values); `protocol/hosts.md`, 1 and 2 (the half);
  `protocol/channel.md`, the world's list (~527-532).
- Status (the spec): `smith/docs/design/protocol/channel.status.md` (6),
  `protocol/hosts.status.md` (1, 2: the withdrawn grant form), and `smith/docs/design/STATUS.md`'s
  withdrawn-grant sentence, as `smith-agent-llm` left it.

## The lead's calls (no question needed)

- **The rename first, then the removal,** so the diff reads: `git mv` the schema to
  `channel-v6.schema`, `src/generated/v5.rs` to `v6.rs` and `golden/v5` to `golden/v6`. Change
  the family line, `pub mod v6`, the root re-exports (`V6` and the names; `WithdrawnValue` and
  `WithdrawnValueParts` leave them), `kinds.rs` (`pub const V6`, `Version { version: 6 .. }`,
  `FORMAT_VERSION: u64 = 6`), and delete the two schema items. Then regenerate:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p smith-codecs-world --test suite -E 'test(=drift::channel_schema_matches_generated_code_and_all_goldens)'`
  (check the name with `--list`). The write mode never deletes, so `git rm` the withdrawn
  goldens it leaves: `enum_cap_account_id_*`, `enum_grant_value_withdrawn_*` and
  `record_withdrawn_value_*`. Then run it again without the variable.
- **Version literals take the constant** where code can name it: `tests/agent-process/src/lib.rs`
  (~314, `ScriptedPeer`), `tests/channel/src/lib.rs` (~499),
  `smith-host-protocol/src/process/tests.rs` (~84), `smith-local-service/src/tests.rs` (~907),
  `smith-agent-service/src/service/tests/setup.rs` (~142). A literal in text stays a literal and
  becomes 6: `smith-local-shell/src/startup.rs` (~105 `channel: 5`), `smith-agent-shell/src/agent_shell.rs`
  (the header), `tests/end-to-end/end_to_end.rs` (~371 `{"channel":5,…}`). `smith_channel::v5::Problem`
  and `v5::Start` (`smith-protocol-translate/src/error.rs` ~36, `smith-host-protocol/src/limits.rs`
  ~35, ~46-47, `src/fixture.rs` ~14) become `v6::`. `smith_channel::V5.len()`
  (`smith-agent-service/src/profile.rs` ~422) becomes `V6`.
- **The withdrawn grant leaves every half:** `smith-host-protocol/src/translate.rs` (~304,
  ~321-327) and `component/tests.rs` (~307-311);
  `smith-host-domain/src/channel.rs` (~122-123); `smith-protocol-translate/src/credentials.rs`
  (~20-21, ~55-57, ~69); `smith-protocol-channel/src/limits.rs` (~100-101) and `tests.rs`
  (~751-760); `smith-channel/src/fixture.rs` (~13-14); `smith-agent-service/src/service.rs`
  (~449-466 with its `Credential::codex`), `service/program.rs` (~404, ~444), `service/tests.rs`
  (~44, ~65, ~70), `profile/effective/channel.rs` (~91-92, ~134-135: the `smith check`
  inventory loses `channel.withdrawn_value_id` and `channel.withdrawn_value_bearer`);
  `smith-inline-agent/src/translate.rs` (~24, ~75) and `tests.rs` (~673); the product's
  unreachable arms `smith-local-service/src/service.rs` (~479), `service/accounts.rs`,
  `credentials.rs`, `profile/effective/channel.rs` (~113-114, ~156-157) and their tests, and
  `smith-local-shell/src/local_host/lend.rs` (~42). A host answer's `Reply::Withdrawn`
  (`translate.rs`, ~286) is a request's reply, not the grant: it stays.
- **`Cap::AccountId` leaves smith's chain:** `smith-domain-session/src/llm.rs` (~383),
  `smith-domain/src/translate.rs` (~408), `smith-domain-run/src/boundary.rs` (~776, re-exported by
  `smith-host-domain`), `smith-protocol-translate/src/answer.rs` (~394),
  `smith-host-protocol/src/answer.rs` (~257), `smith-protocol-events/src/completion.rs` (~131,
  ~199), `smith-local-protocol/src/terminal/failure.rs` (~464). skein's own
  `skein_llm::Cap::AccountId` stays until skein's task. smith's mapping of it
  (`smith-protocol-llm/src/failure.rs` ~232) goes to `llm::Cap::AccessToken`, the cap that
  stays.
- **The agent's credential limits:** `smith-agent-shell/src/config/endpoints.rs` (~296-300) sets
  skein's `CredentialLimits.account_id` from `CEILINGS.withdrawn_value_id`. With v6 it is `0`: no
  account id is admitted. Skein's task deletes the field.
- **No other format moves:** the charter (v4), the transcript (v6), the events (v4) and the local
  store keep their versions. The event stream never wrote `account_id` for a value that is never
  sent, so it needs no new version. `smith check`'s format version stays.

## Owns and may change

smith (owned): the files list, for the edits above, with the tests in
`smith/tests/codecs/tests/suite/channel.rs` (~7-162: the v5 names, the table version, the
`include_bytes!` paths, the withdrawn rows), `fuzzy/decode.rs` (~60-79: the 20 `v5` seeds become
`v6`), `tests/agent-process/src/grants.rs` (~29-35), `world/grants.rs` (~26, ~37-38),
`tests/agent-process/tests/suite/grants.rs` (~21, ~39: pick another value the agent cannot
install for the refused row), `tests/hosts/src/lib.rs` (~165, ~471-495) and
`tests/channel/tests/suite/stories.rs` (~1344, ~1360, ~1381).

Readers in jig and temper (the arms leave an exhaustive match; delete only them):

- `jig/testing/jig-test-service/src/worker.rs` (~723): the `Withdrawn` arm.
- `jig/tests/channel/src/worker_orders/probe.rs` (~12): "version-five kind table" becomes
  "version-six kind table". Check that `PROCESS_CALL = 0x0107` (~13) is v6's kind too.
- `temper/crates/temper-service/src/worker.rs` (~908) and `src/tests/worker.rs` (~373): the
  `Withdrawn` arm and its test row.
- `temper/crates/temper-worker-domain/src/translate.rs` (~264): drop `| agent::Cap::AccountId`.
  Drop its row in `src/tests/translate.rs` (~35).
- temper's design text naming them: `temper/docs/design/domain/worker.md` (~324), "`Cap::AccessToken`
  and `Cap::AccountId` depend on credentials" becomes "`Cap::AccessToken` depends on
  credentials"; `temper/docs/design/protocol/credentials.md` (~352), "Bearer, Key and Withdrawn
  forms are checked" becomes "Bearer and Key forms are checked";
  `temper/docs/design/protocol/channel.status.md` (~6), the clause "and temper sends no withdrawn
  value" goes, the sentence ending at "capped by the client wire's keyed value
  (jig's `jig-protocol-config/src/core.rs`)". Re-date it.

Design and status (as `notes/common.md` says):

- `smith/docs/design/protocol/channel.md`, 2 (~75): "The current version is 6
  (`channel-v6.schema`)." 6 (~360-377): "told apart by a tag in the record, so no form is read as
  another and the withdrawn one needs no guess" ends at "so no form is read as another". The
  ***withdrawn*** sub-bullet goes. In the world's list (~530-531), "and the withdrawn id and bearer,
  each installed" becomes "each installed".
- `protocol/hosts.md`, 1 (~20-22): ", the withdrawn one included for a host that still writes it"
  goes, the item ending at "(channel.md, section 6);".
- `protocol/README.md`, 8 (~347): "The channel speaks version 6 (channel.md, section 2)".
- `protocol/channel.status.md`: the entry goes; the file says `No gap.`. `protocol/hosts.status.md`:
  the item "1, 2: the withdrawn grant form's removal ..." goes. The item at 1, 3 stays, and with
  one item left the list returns to one line.
- `smith/docs/design/STATUS.md`: the sentence "The withdrawn id-and-bearer grant is still accepted
  on the channel, and its removal is not yet (`protocol/channel.md`, `protocol/hosts.md`)." goes.
  Mend the paragraph's flow ("Beside them:" becomes "Beside it:" or the next sentence starts
  plainly). Re-date it.

Pre-approved options:

- If another file in any tree names a v5 item, `Form::Withdrawn` or `Cap::AccountId` (the
  acceptance's `rg`), edit it only to rename or drop that item and list it.
- If a golden outside `golden/v6/` (a check's or a world's) holds `channel":5` or the withdrawn
  inventory lines, regenerate it with its test's write mode and list it. A recorded check in
  `smith/benchmarks/tests/recorded/` stays as recorded.
- If the generator renumbers any tag (it should not: both arms are last), stop and ask: that would
  be a wire change beyond the decision.

## Steps

1. The rename (`git mv`) in one commit, then the schema's removal and the regeneration, as above;
   `git diff --stat` on `golden/` shows only renames and the deleted withdrawn files.
2. The compile repair across the trees:
   `heavy cargo check -p smith-channel -p smith-protocol-translate -p smith-protocol-channel -p smith-host-protocol -p smith-host-domain -p smith-agent-service -p smith-agent-shell -p smith-local-service -p smith-local-shell -p smith-inline-agent -p jig-test-service -p temper-service -p temper-worker-domain --tests`
   while errors last.
3. Narrow tests, one at a time, each count not zero:
   - `heavy cargo nextest run -p smith-codecs-world --test suite`, then
     `heavy cargo nextest run --profile fuzzy -p smith-codecs-world -E 'test(/fuzzy::decode/)'`
   - `heavy cargo nextest run -p smith-channel --lib`, `-p smith-protocol-translate --lib`,
     `-p smith-protocol-channel --lib`, `-p smith-host-protocol --lib`, `-p smith-host-domain --lib`,
     `-p smith-domain --lib`, `-p smith-domain-run --lib`, `-p smith-domain-session --lib`,
     `-p smith-protocol-events --lib`, `-p smith-local-protocol --lib`, `-p smith-inline-agent --lib`
   - `heavy cargo nextest run -p smith-agent-service --lib`, `-p smith-agent-shell --lib`,
     `-p smith-local-service --lib`, `-p smith-local-shell --lib`
   - `heavy cargo nextest run -p smith-channel-world --test suite`, `-p smith-hosts-world --test suite`,
     `-p smith-agent-process-world --test suite`
   - `heavy cargo nextest run -p smith --features end-to-end --test end_to_end`
   - `heavy cargo nextest run -p jig-test-service --lib`, `-p jig-channel-world --test suite`,
     `-p jig-core-world --test suite`, `-p jig-ops-world --test suite`
   - `heavy cargo nextest run -p temper-service --lib`, `-p temper-worker-domain --lib`
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` in `smith/`, `jig/` and `temper/`;
   `heavy cargo clippy` with `--all-targets -- -D warnings` on every package in step 3, and
   `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings`.
5. The design and status edits, then:
   `heavy cargo nextest run -p smith-bench --test suite -E 'test(/offline|probe|tool_output/)'`;
   `python3 -I -B tools/citations/citations.py --base origin/main`; the acceptance's `rg`.
6. Commits: the rename; the removal with the regenerated code and goldens; the halves and
   readers (all three trees); the design and status. Push your branch.

## Report

As the implementer's text says, and also: the goldens renamed and deleted (by `git diff
--stat`); each version literal changed, by file; every reader edited outside smith, by file;
what the acceptance's `rg` leaves.
