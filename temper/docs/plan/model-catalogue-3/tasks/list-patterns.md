+++
id = "list-patterns"
part = "reads"
kind = "feature"
needs = []
files = ["jig/crates/jig-protocol-config/src/llm/providers.rs", "jig/crates/jig-protocol-config/src/members.rs", "jig/crates/jig-protocol-config/src/core.rs", "jig/crates/jig-protocol-config/src/tests/providers.rs", "jig/crates/jig-protocol-config/src/tests/members.rs", "jig/crates/jig-domain-accounts/src/providers.rs", "jig/crates/jig-domain-accounts/src/lib.rs", "jig/crates/jig-domain-core/src/settings/catalogue.rs", "jig/crates/jig-domain-core/src/tests/settings/**", "jig/docs/design/protocol/configuration.status.md"]
effort = "medium"
minutes = 80
+++

# A provider says which names are models

## Goal

`keep` and `drop` read from the document and carried to the accounts child's provider, and the matcher that judges a name by them. It is inert: nothing reads a list yet.

Acceptance:

- `engine.llm.providers[].keep` and `.drop` parse as `jig/docs/design/protocol/configuration.md`, section 3 says, with `llm-provider` rows in `members.rs`, and reach `accounts::Provider.list: Option<ProviderList>` (`contracts.md`, 6) through `core.rs:43`.
- Refused at their paths, each a row of the existing table in `tests/providers.rs`: a pattern empty or past 64 bytes; one with a byte a name can't hold (`valid_name`'s, and `*` and `?`); more than 16 of either; `keep` with no `endpoint`; `drop` with no `keep`. The error names `llm.providers[i].keep[j]` or `.drop[j]`, as the file's others do.
- Absent `keep` is `list: None`: no document that says neither changes in meaning, and no existing fixture's document changes.
- `settings::catalogue::matches` and `kept` (`contracts.md`, 7), with a table test: `gpt-*` on `gpt-6-luna` and not `xgpt-6`; `*audio*` anywhere in a name; `*-20??-??-??` on `gpt-4o-2024-08-06` and not `gpt-4o-audio`; `?` on exactly one byte; `*` on an empty run; a drop winning over a keep; several `*` with backtracking, in step code with a bounded loop and no recursion.
- `configuration.status.md`'s `keep` and `drop` line goes. The help's generated reference regenerates (`temper/crates/temper/src/tests/configuration.rs:43`, `SKEIN_CODEGEN_WRITE=1`).

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/notes/contracts.md`, 6 (`ProviderList`) and 7 (`matches`, `kept`).
- `jig/docs/design/protocol/configuration.md`, section 3, its paragraph on `keep` and `drop`.

## Owns and may change

Owns the header's files.

- **Every `accounts::Provider { .. }` literal** gains `list: None`, by the required-field exception, 23 sites in 21 files, among them `jig/crates/jig-domain-core/src/tests.rs:462`, `jig/testing/jig-test-profile/src/lib.rs:374`, `jig/testing/jig-test-profile/src/charters.rs:43`, `jig/tests/accounts/src/lib.rs:121`, `temper/tests/engine/domain/src/walking.rs:347`, `temper/crates/temper-engine-domain/src/tests/resume_fixture.rs:334`. `rg 'accounts::Provider \{|Provider \{' jig temper` finds them.
- `jig_protocol_config::Provider`'s second literal, `temper/crates/temper-service/src/tests/accounts_host.rs:720`, likewise.
- `temper/docs/help/reference/configuration-engine.md`, regenerated.

Options, each pre-approved:

- If a provider's `list` on `accounts::Provider` needs `#[derive]`s `Provider` has, give them.
- If a `jig-domain-core` test directory for `settings` doesn't exist as `tests/settings/**`, put the table beside `settings::catalogue`'s nearest test and name it in the report.

## Steps

```rust
// jig-domain-accounts/src/providers.rs
/// Which names of a provider's list are models for this work (accounts.md, 3.7).
pub struct ProviderList { pub keep: Box<[Box<[u8]>]>, pub drop: Box<[Box<[u8]>]> }

// jig-domain-core/src/settings/catalogue.rs
#[must_use] pub fn matches(pattern: &[u8], name: &[u8]) -> bool { .. }
#[must_use] pub fn kept(name: &[u8], list: &accounts::ProviderList) -> bool { .. }
```

Then `cargo fmt`; `heavy cargo clippy -p jig-protocol-config -p jig-domain-accounts -p jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-config -p jig-domain-accounts`; `heavy cargo nextest run -p jig-domain-core --lib -E 'test(catalogue)'`; `heavy cargo check -p temper-service -p temper-engine-domain --tests`; `heavy cargo nextest run -p temper --lib -E 'test(configuration)'`. Commit and push.

## Report

As the implementer's text says.
