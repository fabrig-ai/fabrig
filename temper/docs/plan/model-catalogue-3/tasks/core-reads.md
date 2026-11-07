+++
id = "core-reads"
part = "reads"
kind = "feature"
needs = ["list-change", "fleet-reads", "account-reads"]
files = ["jig/crates/jig-domain-core/src/routing/lists.rs", "jig/crates/jig-domain-core/src/routing/mod.rs", "jig/crates/jig-domain-core/src/routing/children.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/routing/view/marks.rs", "jig/crates/jig-domain-core/src/documents.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/limits.rs", "jig/tests/core/tests/suite/lists.rs", "jig/tests/core/tests/suite/main.rs", "jig/testing/jig-core-kit/src/**", "jig/docs/design/domain/engine.status.md", "jig/docs/design/domain/accounts.status.md", "jig/docs/design/protocol/store.status.md"]
effort = "high"
minutes = 170
+++

# An answer becomes one decision

## Goal

The core's half of `jig/docs/design/domain/engine.md`, 17.12: it gives the fleet what the child asks, and turns an answer into one decision by the provider's list and the account's record, in one commit. Marks go by `contracts.md`, 9. This is the meeting point: the child, the fleet and `from_list` run together, in the core's world.

Acceptance, stories in `jig/tests/core/tests/suite/lists.rs`, beside `models.rs`, its fixture's provider given a `list` and a key account, the worker answering by its hub's event:

- **A new name:** a key added places a read on the worker; `Listed` with a new name makes one revision, `by: Decider::List { provider }`, `Running::Stays`, off the picker; the account's record is saved in that commit, a row checking the commit's writes hold both.
- **Nothing new:** the same list again makes no revision, saves the record, and delivers no `me` to a watcher, a row as `model_act.rs:143`'s.
- **Marks,** one table with `me` and `accounts` watched: a read that changes settings marks `me`; one whose `names` narrow with no setting's change marks `me`; a repeat, a failure and a day's alarm mark no `me`; every answer marks `accounts`; `Timer::Account` for a list alone marks nothing and places the read.
- **Running work stays:** a live task keeps its charter's version and its model through a list's revision; a new task takes the new resolution; `moves` isn't taken.
- **Failures:** each `ListFailure` saves `tried` and `failure`, no setting, no name.
- **Refused:** a list's revision that would hold versions past their limit makes none, saves `failure: Room`, and the core stands; a row with `settings.revisions` at its edge.
- **A name gone:** withdrawn, out of the offer and the charter's resolution, and a held task keeps it; it comes back on a later read with its `first_read`.
- **`left_out`:** a full catalogue counts it on the record.
- **No worker:** a read waits, nothing fails, and a worker connecting places it. A worker lost in flight re-sends.
- **A restart:** after a commit the record and the revision read back; a crash before it leaves neither, the core's world's restart oracle holding this story as its siblings'; and a read in flight is simply due again.
- **Stale:** a pending settings proposal is withdrawn by a list's revision, as any revision withdraws it, and its proposer hears so.
- **A removed account:** an answer to its read saves nothing and changes no setting.
- `settings::writes_bound` prices the record's one write, and `jig-protocol-config/src/domain.rs:265`'s edge check holds at a profile's bound.
- `engine.status.md`'s 17.7 and 17.12 go from its line, leaving 11.1 for `offer-narrows`; `store.status.md`'s 4.4 line goes.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 5, 6, 7 and 9.
- `jig/docs/design/domain/engine.md`, 17.2, 17.6, 17.7, 17.8, 17.11 and 17.12; `jig/docs/design/domain/accounts.md`, 3.7.

## Owns and may change

Owns the header's files.

- `routing/lists.rs`, new, is the answer's route. `routing/children.rs:494,556,661,746` give `accounts::Request::ReadList` to the fleet, finding the endpoint by `settings.endpoints`' `providers`, and `Request::ReadList` and `Request::ListRead` from the fleet their places: `Held::ReadList`, and `lists::answered`.
- `routing/settings/dispatch.rs:240`'s `decide` is the entrance: `decide(core, env, routed, s::Decider::List { provider }, None, s::Change { against: core.current.newest(), items, running: s::Running::Stays }, room)`, whose `match by` at `:274` takes `List` beside `FirstStart | Release`. An empty `items` calls nothing, since `SettingsUnchanged` would refuse it. `room` is what a decision of this kind reserves: look at the release's at `settings/restart.rs:494` for a decision with no party.
- `documents.rs:227,268-276`: `Touch::AccountLines`. `routing/view/marks.rs:31`: `touch_account_lines`, beside `touch_accounts`.
- `routing/mod.rs:744`: `Timer::Account` as `contracts.md`, 9 says. Keep its `is_due` guard and `documents/offer.rs:389`'s test passing.
- `routing/settings/dispatch.rs:328`'s `models = true`, and `marks.rs:543`'s `settings_revised` mark `me` as they do; a decision's `models` is what "moved something" means. Read `jig/tests/core/tests/suite/model_act.rs:143` and `temper/docs/plan/model-catalogue-2/tasks/models-switch.md`, its `Mark Frames` line.
- `settings.rs:613`'s `writes_bound` gains one. `limits.rs:90,107,238` if the commit's priced there.
- `jig-core-kit`: a fake worker's answer to a read, and a helper for `set_list`-like rows.
- `jig-protocol-accounts/src/component.rs:234,307,340`, `jig-test-service/src/route.rs:292`, `jig-test-domain/src/routes.rs:95`: arms where a request reaches them.
- A `Request::Save` from the child and the decision's writes sit in one `Routed`, as `account_requests.rs` and `dispatch.rs` each have: if they can't, by `routed.out`'s order, take the narrowest change that commits them together and say so.

Options, each pre-approved:

- `others` for `from_list` is `core.accounts.listed_names(provider, account)`.
- `at` is `env.wall`.
- When a read's account has gone, or its read number is unknown, drop it before `from_list`, so no setting changes.
- On `Refused`, the record takes `Room`, and nothing retries until a day.

## Steps

```rust
// routing/lists.rs
//! A read of an account's list, into settings and the account's record (domain/engine.md, 17.12).
pub(crate) fn answered(core: &mut Core, env: &Env<Limits>, routed: &mut Routed, read: u64, outcome: link::ListOutcome) { .. }
```

This changes what a shared item means for readers of `Timer::Account`'s marks: run the core's readers' tests. Then `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-core -p jig-core-world -p jig-core-kit --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core -p jig-core-world -p jig-protocol-accounts -p jig-accounts-world -p jig-fleet-world -p jig-protocol-config`; `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world`; `heavy cargo nextest run --profile fuzzy -p jig-core-world -p jig-accounts-world`, watching the count and the budget. Commit and push.

## Report

As the implementer's text says.
