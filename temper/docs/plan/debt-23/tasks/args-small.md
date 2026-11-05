+++
id = "args-small"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-authority/src/check.rs", "jig/crates/jig-domain-accounts/src/keys.rs", "jig/crates/jig-domain-accounts/src/changes.rs", "jig/crates/jig-domain-accounts/src/selection.rs", "jig/crates/jig-protocol-worker-agents/src/sources.rs"]
effort = "medium"
minutes = 25
+++

# jig: authority and accounts pay their sites, and the two seams say they're kept whole

## Goal

Part of jig D016, the smallest of it:

- `jig-domain-authority/src/check.rs`'s `batch` and `price` take `Drawn { numbers, source }`, both `Copy`.
- `jig-domain-accounts/src/keys.rs`'s `add` takes the `Keeping` it builds first thing today, built at `changes.rs`'s call.
- The two seams keep their exception, and a comment beside each names it as kept whole on purpose, which is D016's **Paid when**: `jig-domain-accounts/src/selection.rs`'s `Domain::select` and `jig-protocol-worker-agents/src/sources.rs`'s `Component::start`. Each comment says why in a line: a public signature read outside its crate, and for `select` borrowed slices no fold may hold. Each `#[expect]`'s `reason` says the same, as the comment or in its place, whichever is shorter.

Acceptance:

- `rg -o 'too_many_arguments' jig/crates/jig-domain-authority/src jig/crates/jig-domain-accounts/src jig/crates/jig-protocol-worker-agents/src -g '!**/tests/**' -g '!**/tests.rs' | wc -l` reads 2.
- `heavy cargo nextest run -p jig-domain-authority --lib`, `-p jig-core-world`, `-p jig-domain-accounts --lib`, `-p jig-accounts-world` and `-p jig-protocol-worker-agents --lib` pass, with no assertion changed.
- About +8 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header.
- **`select` and `Component::start` don't change,** not by an argument: that's the point of keeping them.

## Steps

1. `Drawn`; `Keeping`; the two comments.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-authority -p jig-domain-accounts -p jig-protocol-worker-agents --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items.
