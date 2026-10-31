+++
id = "fr-qa"
part = "forge-records"
kind = "migration"
needs = ["fr-retire"]
files = [
  "temper/crates/temper/tests/qa_all/**",
  "temper/crates/temper/tests/support/configuration.rs",
  "temper/crates/temper/tests/support/mod.rs",
]
effort = "medium"
minutes = 75
+++

## Goal

The shipped binary's QA tier (`qa_all`, the `qa-temper` tier) runs on
documents that name no forge (`temper/docs/design/draft/playground-reset.md,
3`): a test that needs a connected Forgejo gives it as a seed file
(`<data>/seed/forges/N.json`, fr-document), its credential files as
today; one that needs an owned or fake setup asks it through the
welcome's acts (`temper ctl` SetUp/Connect) or a seed whose setup entry
the box group files carry (bx-serve writes `groups/N`). No test changes
meaning, but those whose meaning was the configured forge slots
(`qa_first_run.rs`'s three default forges): they assert the empty
first run (no forge rows) instead.

Acceptance: `tools/tiers/run qa-temper` passes (or, per test,
`heavy cargo nextest run -p temper --test qa_all --profile qa-temper
-E 'test(=NAME)'` for each file touched), the support's `documents()`
(`support/configuration.rs:6`), `old_form_check_document` (`:117`),
`check_document` (`:132`) and `demo_document` (`:146`) naming no
`application.forges`, `identities`, `setup` or `demo`.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, sections 2
  and 4.
- `temper/docs/design/draft/playground-reset.md`, sections 3 and 6.3.
- `tasks/fr-document.md` (seed files), `tasks/fr-retire.md`.
- `tools/tiers/run` and `tools/tiers/packages.toml` for the tier.

## Owns and may change

Owns `temper/crates/temper/tests/qa_all/**`,
`tests/support/configuration.rs` and `tests/support/mod.rs` (the
document and secret-file writers). The rest of `tests/support/` and
`tests/process.rs` are `fr-process`'s (it runs after this task). May
change anything `notes/scope.md` lists as forced.

## Steps

1. `support/configuration.rs`: each document writer moves its forge,
   identity and setup entries into a seed file, written 0600 in a 0700
   `<data>/seed/forges/` before the binary starts; `demo_document`
   writes `application.playground` and `ports`.
2. Each `qa_all` file the compiler or a failure names: the same; a case
   that read `application.setup[i]` reads the seed's or `ports`.
3. Run the tier's tests touched, one command at a time; read each
   failure's log before rerunning. `cargo fmt`; `heavy cargo clippy -p
   temper --all-targets -- -D warnings`; old forms.
4. Commit and push.

## Report

The standard report, plus: each qa file whose assertions changed and
why; the tier's wall time against its budget.
