+++
id = "override-relations"
part = "refusal"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/domain.rs", "jig/crates/jig-protocol-config/src/tests/overrides.rs", "jig/crates/jig-protocol-config/src/tests/domain.rs", "jig/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 90
+++

# A broken relation between two limits names itself, not "invalid override" on every one

## Goal

An override that breaks one relation between two of the core's configured limits is refused
with `What::Capacity` naming its source, the configured count, the dependency to raise and the
least it must be, as `jig/docs/design/protocol/configuration.md`, 6 promises, and no other
override is marked `What::Override` for it. Today only three relations are named; any other
(for one, `tasks.message_bytes` past `tasks.inbox_bytes`, which the draft's raise trips) makes
`overrides.rs`'s `validate_members` mark every override of the core's families invalid, naming
nothing (contracts K7).

Acceptance:

- `jig_protocol_config::domain::capacity` (`jig/crates/jig-protocol-config/src/domain.rs`)
  checks, after the three it has and in configuration.md 6's order, each relation that
  section lists, and returns the first failed one as today:
  `Some((member, What::Capacity { member, configured, requires, required }))`, `member` the
  source's override name, `requires` the dependency's, `required` the least the dependency must
  be:

  | member (source) | requires (dependency) | required |
  |---|---|---|
  | `people.operations` | `tasks.tasks` | operations |
  | `authority.grants` | `notes.scopes` | grants + 3 |
  | `tasks.message_bytes` | `tasks.inbox_bytes` | message_bytes |
  | `tasks.run_inbox` | `tasks.inbox_messages` | run_inbox |
  | `people.inbox_entries` | `tasks.inbox_messages` | inbox_entries |
  | `tasks.exemptions` | `tasks.tasks` | exemptions |
  | `fleet.configured_workers` | `people.program_hosts` | workers + 1 |
  | `fleet.configured_workers` | `accounts.program_hosts` | workers + 1 |
  | `fleet.worker_name_bytes` | `people.program_host_name_bytes` | worker_name_bytes |
  | `accounts.program_names` | `fleet.program_sources` | program_names |
  | `authority.roles` | `documents.roles` | roles |
  | `tasks.inbox_messages` | `people.inbox_recipients` | 2 × inbox_messages + authority.roles + 4 (`jig-domain-core`'s `inbox::waiting_capacity`) |
  | `accounts.credentials` | `fleet.grants` | credentials + min(accounts.accounts, `jig_domain_tasks::MODELS`) |
  | `accounts.credentials` | `authority.accounts` | max(credentials, min(accounts.accounts, `MODELS`)) |

  each read from `jig-domain-core/src/limits.rs`'s `checked_valid` and
  `jig-domain-tasks/src/limits.rs`'s `valid`, which stay as they are. A relation with a zero on
  either side is skipped (a zero is a bound, refused as today); arithmetic past `u64` is
  skipped too (the bound refuses it).
- Both paths take it: the override path (`overrides.rs`, `validate_members`, which already pushes
  the first and returns) and the start's (`domain.rs`, `whole`), as the three are taken today.
- The settings' equalities (`tasks.revisions`, `tasks.versions`, `fleet.versions` with
  `core.settings`'s) and every zero or overflow stay as they are: `What::Override` on the
  family's overrides.
- Tests (below) pass; `jig/docs/design/protocol/configuration.status.md`'s **Not yet** entry
  "6: a failed relation other than live tasks' inbox and operations' two is not named: ..." is
  removed, the rest of the line kept.

## Inputs

- `temper/docs/plan/fabrig-scale/notes/common.md`; `notes/contracts.md`, K7.
- `jig/docs/design/protocol/configuration.md`, 6: the paragraphs from "Capacity dependencies are
  checked after all overrides" to "refused as an override past its family's bound, as are the
  settings' equalities".

## Owns and may change

- Owns the files of the header for what acceptance names; `capacity` may become a short table
  walk with a private helper (`fn relation(...) -> Option<(…)>`), its signature unchanged.
- **Reader tests whose expectations move:** in `src/tests/overrides.rs` and `src/tests/domain.rs`,
  a test whose input breaks one of the new relations and today expects `What::Override` on
  several members (for one, `overrides_past_a_familys_bound_are_each_reported_once`, and the
  tests near lines 37-43, 509-522, 565-577, 590-617, 640-660): if its input breaks a relation
  only incidentally (the test is about a bound), change its input so it breaks no relation and
  keeps testing the bound; if it is about the relation, change its expectation to the one
  `What::Capacity`. Say which in the report.
- `jig/testing/jig-test-service/src/tests/startup.rs` asserts a `What::Capacity`: run it; edit
  only an expectation a new relation changes, as above.
- `problem.rs`'s `capacity_message` words stay; `overrides.rs` changes only if `validate_members`
  must (it should not).
- `jig-domain-core` is not yours: if `waiting_capacity` or `MODELS` is not reachable from
  `jig-protocol-config`, compute the same figure inline with a comment naming the function.

## Steps

1. The table in `capacity`, in configuration.md 6's order, the three existing first.
2. Tests in `src/tests/overrides.rs`, rows added to `capacity_dependencies_survive_override_parsing`
   (each a small-profile document with `"documents.lines": 3` as its neighbours have):
   - `{"tasks.message_bytes": 8192}` → `Capacity { member: b"tasks.message_bytes", configured: 8192, requires: b"tasks.inbox_bytes", required: 8192 }`;
   - `{"authority.grants": 6}` → `Capacity { member: b"authority.grants", configured: 6, requires: b"notes.scopes", required: 9 }`;
   - `{"tasks.tasks": 4}` → `Capacity { member: b"people.operations", configured: 8, requires: b"tasks.tasks", required: 8 }`;
   - `{"tasks.run_inbox": 9}` → `Capacity { member: b"tasks.run_inbox", configured: 9, requires: b"tasks.inbox_messages", required: 9 }`.
   And a new test `a_broken_relation_is_the_only_problem`: small with `"documents.lines": 3`,
   `"tasks.charters": 8`, `"authority.grants": 6` and `"tasks.message_bytes": 8192` gives exactly
   one problem, the first relation in order (`authority.grants`'s), and no `What::Override`.
   In `src/tests/domain.rs`, beside the `refuses(..)` cases near lines 136-151, the start path's
   refusal for `tasks.message_bytes` past `tasks.inbox_bytes`, at its `codec_path`.
3. `heavy cargo nextest run -p jig-protocol-config --lib -E 'test(/tests::overrides::|tests::domain::/)'`;
   then `heavy cargo nextest run -p jig-protocol-config` and `-p jig-test-service`; `heavy cargo
   check -p temper-service --tests`.
4. The status line; `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-protocol-config --all-targets -- -D warnings`; the citations check; commit and push.

## Report

As the implementer's text asks, with each test whose input or expectation you changed and why.
