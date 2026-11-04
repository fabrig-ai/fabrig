+++
id = "client-capacities"
part = "clients"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-terminal/src/limits.rs", "temper/crates/temper-client-shell-terminal/src/tests.rs", "temper/crates/temper-client-shell-web/src/limits.rs", "temper/crates/temper-client-tools/src/limits.rs", "temper/crates/temper-client-tools/src/jig/settings_adapter.rs", "temper/crates/temper-client-tools/src/tests/limits.rs", "temper/crates/temper-client-tools/src/tests/jig/settings.rs", "temper/crates/temper-settings-form/src/lib.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 75
+++

# temper's clients receive what a fabrig-scale box sends

## Goal

Each of temper's clients receives the lists a box on the `standard` profiles sends (contracts K2;
`temper/docs/design/client/README.md`, section 2, the receiving capacities): sixteen entries in
a document's short lists, sixteen grants and delegation kinds in an authority, eighty live
delegates in a chat's close; and the settings form, which `temper ctl settings` and the
configuring agent's `settings` tool decode through, sixteen grants and kinds. Today the form
holds twelve, below the stock's fourteen grants, so the configuring agent's read of policy 1
fails ("settings: document limit"), and every shell refuses a fifteenth grant.

Acceptance:

- `jig_client::DomainLimits` in each shell's limits, `temper-client-shell-terminal/src/limits.rs`,
  `temper-client-shell-web/src/limits.rs` and `temper-client-tools/src/limits.rs`: `authority`
  16 (was 14); `rows` 16 where it is 8 (the terminal's and the browser's; the CLI's stays 32);
  `delegates` 80 (was 8). Each with a comment naming what it holds (README.md, section 2).
- `temper-settings-form/src/lib.rs`, `Limits::standard()`: `authority` 16 and
  `settings.delegation_kinds` 16 (each was 12); `temper-client-tools/src/jig/settings_adapter.rs`'s
  `limits` `authority` 16 (was 12). Nothing else of the form.
- The tests that pin the old figures name the new ones, each edited only there:
  `temper-client-shell-terminal/src/tests.rs` (near 63), `temper-client-tools/src/tests/limits.rs`
  (near 12-41: the shells' equality and `close --move`'s count within `delegates`),
  `temper-engine-smith/src/tests/settings.rs` (near 257, the form's kinds equal ctl's),
  `temper-client-tools/src/tests/jig/settings.rs` (near 725 and 753, the bound table's
  `delegation_kinds` rows).
- A new test in `temper-engine-smith/src/tests/settings.rs`,
  `the_settings_tool_restores_the_stocks_policy_and_rules`: the stock's policy 1 and rules
  (`temper_service` is not a dependency of this crate: build a policy of fourteen grants and
  seven delegation kinds as the stock's are, with the crate's own fixtures) restore through
  `restore_setting_document(.., Limits::standard())` without `document limit`; and one of
  sixteen of each restores too.
- `temper/docs/design/client/README.status.md`: its entry "2: the shells receive fourteen grants
  ... (`temper-settings-form/src/lib.rs`)." removed.
- `heavy cargo nextest run -p temper-settings-form`, `-p temper-client-tools`,
  `-p temper-client-shell-terminal`, `-p temper-client-shell-web --lib` and `-p temper-engine-smith`
  pass; `heavy cargo nextest run -p temper --test suite -E 'test(/configuration::/)'` passes
  (it holds the engine's documents within the terminal's capacities).

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K2.
- `temper/docs/design/client/README.md`, section 2 (the paragraph "The native terminal, the
  browser and `temper ctl` receive sixteen entries").
- jig's `client/domain/client.md`, 14 and 16 (a client's receiving bounds; refused whole past
  them).

## Owns and may change

- The files of the header, for what acceptance names.
- **Another plan's crate:** `temper-settings-form/src/**` is also `configuring-p3/tools-client`'s
  (in flight on its own trunk): change only the two figures named, no other line, so the merge
  meets one hunk.
- jig's `jig-client-view` sizes its close confirmation from `delegates` (`sizes/sizes.rs`): no
  edit; if a jig test pins a node envelope that the shells' larger `delegates` breaks, it is
  read only by temper's shells' own tests: say so in the report.
- If the browser's measured heap test (`temper-client-shell-web/tests/suite/memory.rs`) moves,
  it is not this task's figure to raise: report it with the figure (a question).

## Steps

1. The figures; their pins; each crate's tests one at a time.
2. The new settings test.
3. The status line; `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy
   -p temper-settings-form -p temper-client-tools -p temper-client-shell-terminal -p
   temper-client-shell-web -p temper-engine-smith --all-targets -- -D warnings`; the citations
   check; commit and push.

## Report

As the implementer's text asks.
