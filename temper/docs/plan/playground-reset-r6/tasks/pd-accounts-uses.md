+++
id = "pd-accounts-uses"
part = "pd"
kind = "feature"
needs = ["pd-vocabulary"]
files = [
  "jig/crates/jig-domain-accounts/**",
  "jig/tests/accounts/**",
]
effort = "medium"
minutes = 50
+++

## Goal

The accounts child's two changes for this run: an account made by `AddKey` takes the uses the
change carries (contracts J8), and `ForgetProject { project }` drops `Use::Project(project)`
from every account (contracts J4).

Acceptance:
- `Change::AddKey { uses: Some(list), join: None, .. }` makes the new account with
  `uses: Some(list)` when `records::valid_uses(Some(&list))`; otherwise it refuses
  `Refusal::Uses` and makes nothing. `uses: None` behaves as today. With `join: Some(_)` and
  `uses: Some(_)` it refuses `Refusal::Uses` (a joined account keeps its own uses).
- `Event::ForgetProject { project }`: for every account whose uses are `Some(list)` and name
  `Use::Project(project)`, the list without that entry (an emptied list stays `Some([])`), one
  `Request::Save { record: ordering::account_record(..) }` per changed account; accounts with
  `None` or without the entry are untouched; no answer. It is accepted whenever the child is
  restored (gated as `Limited` is), and repeating it changes nothing.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, J4 (account uses), J8.
- `jig/docs/design/domain/accounts.md`, 4.2 (uses).
- The code: `src/changes.rs` (`Change::SetUses` ~110-121), `src/keys.rs` `add`,
  `src/records.rs` `valid_uses`, `src/ordering.rs` `account_record`.

## Owns and may change

Owned: `jig/crates/jig-domain-accounts/**` and `jig/tests/accounts/**`. Nothing outside; the
shapes are `pd-vocabulary`'s.

## Steps

1. `keys.rs` `add`: use its `uses` parameter as above.
2. `domain.rs`/`changes.rs`: `forget_project(domain, project, out)` for the event.
3. Tests in `jig/tests/accounts/tests/suite/changes.rs`:
   `a_key_added_with_uses_makes_an_account_serving_them`,
   `a_key_added_with_invalid_uses_makes_nothing`,
   `forgetting_a_project_drops_its_use_and_keeps_an_emptied_list` (three accounts: one with
   `[Project(2)]` becomes `Some([])`, one with `[Party(5), Project(2)]` becomes
   `[Party(5)]`, one with `None` is untouched; after a restart the saved uses restore).
   Commands: `heavy cargo nextest run -p jig-accounts-world --test suite -E 'test(/uses/)'`
   and `-E 'test(/forgetting_a_project/)'`, then the whole
   `heavy cargo nextest run -p jig-accounts-world --test suite`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-accounts -p jig-accounts-world --all-targets`;
   old forms; commit and push.
