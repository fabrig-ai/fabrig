+++
id = "room-table"
part = "room"
kind = "interface"
needs = ["tools-surface", "agent-kinds"]
files = ["temper/crates/temper-engine-domain/src/room.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/tests/room.rs", "temper/crates/temper-engine-smith/src/room.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests/room.rs", "temper/crates/temper-engine-smith/src/tests.rs"]
effort = "high"
minutes = 90
+++

# room-table: what a charter's host part costs, where the domain reads it

## Goal

The table temper's start and a change's entrances check a charter's brief room by (temper's
`domain/agent.md`, 4.3, "Checked before a run meets it"; contract C2): each part of a run's
non-brief host part in bytes, held by `temper-engine-domain`, built by `temper-engine-smith`
from its own declarations and smith's measures, and held equal to smith's measure by a step
test. Nothing checks by it yet (`room-checks`).

Acceptance:

- `temper_engine_domain::room` exists with C2's `HostPart`, `BRIEF_ROOM`, `Short` and their
  methods, its module doc citing `temper/docs/design/domain/agent.md`, 4.3.
- `temper_engine_smith::room::host_part(limits: &smith_domain_run::Limits) -> HostPart`
  builds it: `declared` from the full registry (`calls::tools`, the plan's `change` and
  `message` included) less the settings family's two tools, `settings` from those two, each
  tool's name, description and schema bytes as smith counts a host tool; `fixed` and smith's
  own families from smith's measure (`smith_domain_run`'s `brief_room` /
  `non_brief_host_bytes`, `smith_protocol_llm::schema_costs()`), never a figure typed in.
- Step tests in `temper-engine-smith/src/tests/room.rs`:
  `the_table_is_smiths_measure`: for charter policies that turn each smith family on and off,
  with `tools` `EVERY_FAMILY` and with `512`, and instructions of 0 and 3,000 bytes,
  `HostPart::non_brief` equals smith's non-brief measure of the run whose offer is largest (an
  authority of every family, a Change contract, delivering) and is at least smith's for an
  Open and a Report run of the same charter; `a_charter_naming_settings_pays_for_its_tools`.
  In `temper-engine-domain/src/tests/room.rs`: `check` refuses a policy whose room is 2,047
  and admits 2,048, and `Short`'s words name the charter and both figures.
- The focused tests of `temper-engine-domain` and `temper-engine-smith` pass.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3 ("The host part's budget", "The brief fits the
  room its charter leaves", "Checked before a run meets it").
- `temper/docs/design/protocol/configuration.md`, 13.4 ("Each charter's host families").
- smith's `smith/docs/design/domain/run.md` where it defines the host part (read its section
  on the charter's host part and `brief_room`; `smith/crates/smith-domain-run/src/charter.rs`,
  `brief_room`, `non_brief_host_bytes`).
- `temper/docs/plan/configuring-p3/notes/contracts.md`, C1 and C2.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files` (in `lib.rs` and `tests.rs`, the `mod` lines alone).
- Options, pre-approved:
  - If smith's measure needs a whole `run::Charter` to count, build one in the test from a
    stock-like assignment (`temper-engine-smith/src/tests.rs`' `assignment()` and
    `crate::charter`) rather than adding API to smith; if `host_part` needs a smith function
    smith keeps private, stop and ask.
  - If a charter's model names or other policy text enter smith's count, add them to
    `non_brief` as C2's `instructions` row does and name them in the report.
  - If smith's own families cost the same whatever the policy, keep their fields and fill
    them 0, saying so.

## Steps

1. `temper-engine-domain/src/room.rs`: C2's types, `non_brief`, `room`, `check`.
2. `temper-engine-smith/src/room.rs`: `host_part`.
3. The tests.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-smith --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-engine-domain -p temper-engine-smith`.
5. Commit ("temper: a charter's host part, by its parts, where the domain reads it
   (agent.md, 4.3)") and push.

## Report

As the implementer's text says, and the table's figures in the small profile (`fixed`,
`declared`, `settings`, smith's families), with stock charter 1's and a 1,536-tools charter's
`non_brief`.
