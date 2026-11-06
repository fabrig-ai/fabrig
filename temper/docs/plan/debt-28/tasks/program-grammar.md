+++
id = "program-grammar"
part = "program-grammar"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-people/src/lib.rs", "jig/crates/jig-domain-people/src/domain/asks.rs", "jig/crates/jig-domain-people/Cargo.toml", "jig/crates/jig-client-domain/src/contract/deployment.rs", "jig/crates/jig-domain-accounts/src/records.rs", "jig/crates/jig-domain-accounts/src/changes.rs", "jig/crates/jig-protocol-engine-clients/src/translate/asks.rs", "jig/crates/jig-protocol-engine-clients/Cargo.toml", "jig/crates/jig-protocol-store/src/codec/people/asks/account_changes.rs", "jig/crates/jig-protocol-store/Cargo.toml", "jig/docs/development/debt/D006-link-values-two-paths.md"]
effort = "medium"
minutes = 25
+++

# The program-name grammar is written once

## Goal

Pay the grammar half of jig's D006. The grammar is written byte for byte in `jig_domain_people::program_name` and `jig_client_domain`'s `valid_program_name`, beside `jig_domain_link::SourceName::new`. All three spell one rule: `skein_lib::Name::<32>::new(name, skein_lib::Grammar::LowercaseDigitsHyphen)`, which `SourceName` already calls.

**The other half stays open, and the item stays with it.** The item counts 59 uses of its re-exports. The cut counted 916 in 180 files across jig and temper, far past its Size S, so the request's "stop and ask fabrig/debt" applies, and the cut has asked. Don't touch a `pub use jig_domain_link`: the item's measure stays at 6.

Acceptance:

- `jig_domain_people::program_name` is gone, and people's admission and its two outside readers ask `jig_domain_link::SourceName::new(name).is_some()`.
- `jig_client_domain`'s `valid_program_name` keeps its name and signature, since `temper-client-view` reads it and that crate is closed, and its body is one call: `skein_lib::Name::<32>::new(name, skein_lib::Grammar::LowercaseDigitsHyphen).is_some()`. Its loop has gone.
- `jig_domain_accounts`'s `records::program_name` wrapper is gone, and its one caller asks `crate::SourceName::new` directly.
- This search finds `valid_program_name`'s definition, and the two `program_name` helpers in `temper-client-view/src/choice.rs` and `temper-client-domain/src/pages/connect_model.rs`, which build a suggested name and check nothing, so they're exempted: `rg -n 'fn \w*program_name\b' jig/crates temper/crates -g '!**/tests/**' -g '!**/generated/**'`.
- The item reads true afterwards: see Steps.

The task ends with about 25 fewer lines.

## Inputs

- `jig/docs/development/debt/D006-link-values-two-paths.md`, whole.
- `jig/docs/design/domain/root.md`, 2: `jig-domain-link` is a shared leaf value crate, which the core's children may depend on.
- `jig/docs/design/client/domain/README.md`: `jig-client-domain` depends on skein-lib alone, so it can't reach the link. The rule it calls is skein-lib's, so it needs no checked copy.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. May add `jig-domain-link.workspace = true` to the three `Cargo.toml`s in the header, in their lists' order, and take `Cargo.lock`'s lines that follow.

`jig-domain-people` gains that dependency, as `jig/docs/design/domain/README.md` allows. If `jig/tests/boundary` holds a list that refuses it, add it there: a one-line edit, and your report says so.

A test that called `program_name` by name may reach `SourceName::new` instead, keeping what it asserts. The cut saw none.

If any of the three spellings turns out to differ from the others on some input, the merge would change what a refusal says: stop and ask.

## Steps

Before you remove them, compare the three once with a throwaway untracked test or by reading: empty, 32 bytes, 33 bytes, and every byte value at one position. They should agree, and `skein-lib/src/name.rs`'s `new` refuses empty and over-long as both loops do.

Then the item: `jig/docs/development/debt/D006-link-values-two-paths.md` keeps its status and **Measure** (6). Update, to what you find:

- **Size:** M, and **Value:** low (one name per value).
- **Paid when:** "`SourceName`, `Worker`, `Form`, `Grant` and the link's other values are reached only through `jig-domain-link`."
- **What:** the grammar's copies are gone, each now asking `skein_lib::Name` or `SourceName`. What is left is the six `pub use` lines and their readers, about 916 uses in 180 files by the cut's search (`jig_domain_host` 536, `jig_domain_fleet` 227, `jig_domain_accounts` 153), most through a `use jig_domain_host as host` alias, which the item's first count of 59 missed.
- **What pays it back:** dropping the re-exports, crate by crate, each with a dependency line in about 25 manifests; a private `use` at each crate's root keeps `crate::SourceName` inside it. It adds a line per file more than it removes.

`heavy cargo nextest run -p jig-domain-people -p jig-domain-accounts -p jig-client-domain`; `heavy cargo nextest run -p jig-protocol-engine-clients -p jig-protocol-store -E 'test(program)'`, checking the count isn't 0; `heavy cargo nextest run -p jig-boundary`. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-people -p jig-domain-accounts -p jig-client-domain -p jig-protocol-engine-clients -p jig-protocol-store --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the search's reading, the net lines, and what your comparison showed.
