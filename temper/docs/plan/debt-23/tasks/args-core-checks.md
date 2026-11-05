+++
id = "args-core-checks"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/acts/bounds.rs", "jig/crates/jig-domain-core/src/tests/acts/**", "jig/crates/jig-domain-core/src/tests.rs", "jig/crates/jig-domain-core/src/tests/admission/escalation_holder.rs"]
effort = "medium"
minutes = 30
+++

# jig-domain-core: settings' checks split, and `create_start` takes its `Creator`

## Goal

Part of jig D016: three sites that share nothing with core's routes.

- `settings/check.rs`: paid with no struct, as its `#[expect]`'s reason wants to avoid a stored reference. `connectors` is read only in `template_fault`'s first match: that match moves to a small `contract_fault`, called first by both of `template_fault`'s callers. `rules` is read only by `policy_fault`'s first statement: that check moves to its one caller. **The order of the checks stays,** so each fault still answers first where it does.
- `acts/bounds.rs`'s `create_start` takes `Creator { project, person, role }`, and its 20 test callers take it too, through their helper where one exists.

Acceptance:

- `rg -n 'too_many_arguments' jig/crates/jig-domain-core/src/settings/check.rs jig/crates/jig-domain-core/src/acts/bounds.rs` reads nothing.
- `heavy cargo nextest run -p jig-domain-core --lib` and `-p jig-core-world` pass, with no assertion changed.
- About +15 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header; the tests only in `create_start`'s call. If a file there calls it many times, one local helper instead of a literal at each.
- **If a split would move a check past another,** keep the order, by a fold if that's what it takes, and say so.

## Steps

1. `check.rs`, one function at a time.
2. `Creator`.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
