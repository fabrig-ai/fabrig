+++
id = "ways-delegation"
part = "acts"
kind = "feature"
needs = ["client-ways"]
files = ["temper/crates/temper-client-domain/src/forge/stock_ways.rs", "temper/crates/temper-client-domain/src/forge/stock_ways/**", "temper/crates/temper-client-domain/src/tests/forge_stock_ways.rs", "temper/docs/design/client/README.status.md"]
effort = "medium"
minutes = 90
+++

## Goal

Where a way's `delegation` holds, add the project's policy to its act:
read the policy whole and send it, with charter 5, role 0 and the merge
job added to its ceiling's and role 0's delegation, in the same
`ChangeSettings` as the way's items, so that Draft first runs on a project
made before the stock's delegation reached charter 5
(`temper/docs/design/client/README.md`, 5.13, *A way's acts*;
`temper/docs/design/protocol/configuration.md`, 13.10).

Acceptance:

- `UseWay` or `KeepWay` on a way whose `delegation` is true first reads
  jig's setting `ProjectPolicy { project }` at the newest revision
  (`c::Read::Setting`), then sends one `ChangeSettings` of the way's
  items and the item `Setting::ProjectPolicy { project }` set to the
  policy read, its ceiling's delegation and role 0's authority's
  delegation each holding the executors `Charter(5)`, `Role(0)` and
  `Job(3)` (jig's executor codes, as jig's client contract encodes a
  delegation's allowed executors), none twice, nothing else changed. A
  policy that already holds them sends no policy item. The read refused
  or failed is the act's refusal, kept as `client-ways` keeps one.
- Tests in `src/tests/forge_stock_ways.rs`: a way with `delegation` reads
  the policy, then sends its items and the policy with the three
  executors added; a policy holding them sends the items alone; the read
  refused keeps the refusal and sends nothing; a way without `delegation`
  reads nothing.
- `temper/docs/design/client/README.status.md`: "5.13, *A way's acts*: a
  way's `delegation` adds no policy item." leaves its **Not yet**.

## Inputs

- `temper/docs/plan/ui-ask-m3a/notes/common.md`; `notes/contracts.md`,
  section 6.
- `temper/docs/design/client/README.md`, 5.13; `temper/docs/design/protocol/configuration.md`,
  13.4 (*the rules*: the stock's delegation to charter 5, role 0 and the
  merge job) and 13.10.
- jig's client contract: `jig/crates/jig-client-domain/src/contract/settings.rs`
  (`SettingPolicy`, `SettingItem`), `contract/authority.rs`
  (`Delegation`, its allowed executors), `contract/reads.rs`
  (`Read::Setting`, `ReadSetting`).

## Owns and may change

- **Owns:** the policy's read and item in `forge/stock_ways.rs`; its
  tests.
- **Options, pre-approved:** if jig's client contract gives no way to
  name an executor's code from the client, stop: that is a `Question:`
  naming the contract's types.

## Steps

1. Read jig's delegation encoding; the read, then the send.
2. The tests; break one rule (send the policy unchanged) to see its row
   fail, and restore.
3. The status sentence.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-domain --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-domain`.
   Commit and push.

## Report

As the implementer's text asks.
