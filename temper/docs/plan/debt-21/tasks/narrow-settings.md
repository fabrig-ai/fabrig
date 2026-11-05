+++
id = "narrow-settings"
part = "narrowing"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/settings/templates.rs", "jig/crates/jig-domain-core/src/documents/settings.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch/project_policy.rs", "jig/crates/jig-domain-core/src/acts/bounds.rs", "jig/crates/jig-domain-core/src/admission/policy.rs", "jig/crates/jig-domain-core/src/admission/authority_facts.rs", "jig/crates/jig-domain-core/src/reads/run_task.rs"]
effort = "high"
minutes = 90
+++

# jig-domain-core: a jig-only lookup stops answering `Application`

## Goal

Part of `jig/docs/development/debt/D017-unreachable-re-narrowing.md`. Jig's own settings lookups match `Value::Application` or `Setting::Application` only to abort, at 22 sites here, most saying "an application's setting: apps-core", a name that means nothing now. A `Value` accessor and two lists of offered numbers carry that once. Five more sites aren't re-narrows at all, and get the small fix each needs.

Acceptance:

- `rg -c 'unreachable!'` reads, under `jig/crates/jig-domain-core/src`: `settings.rs` 1 (`rules`'s "checked settings always offer rules"), `settings/check.rs` 0, `documents/settings.rs` 0, `routing/settings/dispatch/project_policy.rs` 0, `settings/charters.rs` 3, `settings/templates.rs` 0, `acts/bounds.rs` 2, `admission/policy.rs` 3, `routing/settings/dispatch.rs` 0, `reads/run_task.rs` 1, `admission/authority_facts.rs` 0.
- `rg -n 'apps-core' jig/crates/jig-domain-core/src -g '!reads/settings.rs' -g '!settings/tests.rs'` finds nothing. Those two files keep theirs: one is `reads/settings.rs:496`, below, and the other is test code this task needn't touch.
- `heavy cargo nextest run -p jig-domain-core --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass with no assertion changed. `heavy cargo check -p jig-ops-domain -p temper-engine-domain --tests` passes.
- The branch's net production lines fall.

## Inputs

- `temper/docs/plan/debt-21/notes/common.md`
- `temper/docs/plan/debt-21/notes/narrowing.md`, binding
- `jig/docs/development/debt/D017-unreachable-re-narrowing.md`
- `jig/docs/design/domain/engine.md`, 17.1, 17.5 and 17.7

## The sites

- **A jig-only lookup:** `settings.rs:270, 278, 286, 296, 486`; `settings/check.rs:133, 180, 466, 578, 716, 726, 727`; `documents/settings.rs:77`; `dispatch/project_policy.rs:40, 46, 176`; `settings/charters.rs:89`; `settings/templates.rs:124, 204`; `acts/bounds.rs:96`; `admission/policy.rs:243`; `routing/settings/dispatch.rs:232`.
  - `pub(crate) Value::{as_charter, as_template, as_policy, as_rules} -> Option<&T>`, each an exhaustive match.
  - `Current::offered_charters()` and `offered_templates() -> Box<[u32]>`.
  - `project_policy::value(item) -> Option<&ConfiguredPolicy>`.
  - A dead arm folds into its neighbour. `Value::Application` under a `Setting::Charter` key reads as absent, as `Value::Template` there already does. `shape()` refuses the pair, and the store's `decode_value` has no `Application` arm, so no run reaches it. `Current::rules` keeps its one `unreachable!`.
- **One match:** `routing/settings/dispatch.rs:513`, on the proposer.
- **Test code the Measure counts:** `settings.rs:422, 423, 432` (`test_charter_mut`, `test_templates`) and `reads/run_task.rs:741`, each `panic!` in its `#[cfg(test)]` item.
- **The last `else` of a bool chain:** `admission/authority_facts.rs:55` returns `Scope`.

`reads/settings.rs:496` stays: other plans are changing that file.

## Owns and may change

- **Owns:** the files in `files`.
- **`settings::Value` keeps its shape:** `jig/tests/*`, `temper/tests/*` and `jig-protocol-store/src/codec/settings.rs` read it, so the maps don't split by family.
- **Reuse** `application::is_application`, `kind_of`, `settings::is_project_policy` and `check::edit`, where they fit.
- **Don't** touch `settings/application.rs`, `routing/settings/restore.rs`, `effects.rs`, `routing/party/*` or `routing/task.rs`: other tasks of this plan hold them.

## Steps

1. The accessors and offered lists; the sites, file by file.
2. The rest of the list.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py --write)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; the counts; any site that stayed, and why.
