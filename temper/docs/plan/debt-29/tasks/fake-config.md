+++
id = "fake-config"
part = "fake-config"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-domain/src/domain.rs", "skein/crates/skein-fake-forge-domain/src/limits.rs", "skein/crates/skein-fake-forge-domain/src/tests.rs", "skein/crates/skein-fake-forge-forgejo/src/tests/fixtures.rs", "skein/crates/skein-fake-forge-github/src/fixture.rs", "skein/testing/skein-forge-kit/src/fixtures.rs", "skein/testing/skein-forge-kit/src/fake_github.rs", "skein/tests/fake-peers/tests/forge/fixtures.rs", "skein/tests/forge/client/src/world.rs", "skein/tests/forge/github/src/fake.rs", "skein/tests/fake-forge/tests/suite/memory.rs", "skein/docs/development/debt/D033-limits-literals-in-tests.md"]
effort = "medium"
minutes = 60
+++

# The fake forge's `Config` starts from one constructor

## Goal

Pay skein's D033. Nine whole literals of `skein_fake_forge_domain::Config`, 429 lines, each list 26 fields, so no reader can see what a test wants of the forge. `Config::TEST` and `Limits::TEST` become the base, as `io::Limits::TEST` is, and each site names only what differs from them. **Every site keeps every value.**

Acceptance:

- The measure reads 0: `rg -U --pcre2 -o 'Config \{\s*limits:(?<fields>(?:(?!\.\.)[^{}]|\{(?&fields)\})*)\}' skein -g '*.rs' | rg -c 'settle_min:'` (9 at the cut; `rg -c` prints nothing and exits 1 at 0).
- `TEST`'s definitions write `Self {`, which the measure doesn't count. `Limits::TEST` is the one whole `Limits` literal this task adds, and the literals it replaces lose the fields that match it.
- No site's value changes; no test's outcome changes.
- `skein/docs/development/debt/D033-limits-literals-in-tests.md` is deleted.
- The net is at least 150 lines fewer.

## Inputs

- `skein/docs/development/debt/D033-limits-literals-in-tests.md`, whole.
- `skein/docs/foundation/testing-strategy.md`, 10 ("The setup the assertion reads").
- `skein/crates/skein-io/src/limits.rs`, `Limits::TEST`: the form to follow.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `Config` and `Limits` are `Copy`, and `TEST` is an addition.

- **The base is the literal most sites nearly hold:** `skein/tests/fake-peers/tests/forge/fixtures.rs`'s `fake_config`. As the cut read them, `skein-fake-forge-forgejo/src/tests/fixtures.rs`'s `fake_config` holds the same values, `skein/tests/forge/client/src/world.rs`'s `config` differs in its `if faults` fields, and `skein-forge-kit/src/fixtures.rs`'s `fake_config` in `name_bytes: 96` and its comment.
- **The other five are far from any base, and stay honest about it:** `skein-fake-forge-domain/src/tests.rs`'s `CALM`, `skein/tests/fake-forge/tests/suite/memory.rs`'s `CONFIG`, `skein-fake-forge-github/src/fixture.rs`'s `domain_config`, `skein-forge-kit/src/fake_github.rs`'s `domain_config` and `skein/tests/forge/github/src/fake.rs`'s `config`. Each becomes `Config { .., ..Config::TEST }`, naming what differs, however many fields that is. `CALM` and `CONFIG` keep their `limits: LIMITS`: those `Limits` literals are constants of their own and stay.
- **A comment on a value stays with it,** such as `skein-forge-kit/src/fixtures.rs`'s on 68 bytes.
- **`..CALM` updates and `Limits { facts: .., ..LIMITS }` stay:** they already change only what they test.
- **`Config::TEST` isn't `fake_config`:** the helpers keep their names and callers, and a helper that holds nothing beyond `Config::TEST` still stands. Merging them is outside the item.
- **Temper's own literals stay:** `temper/tests/client-browser/src/github.rs`, `temper/tests/worker/checkout/src/forge.rs`, `temper/tests/engine/forge/src/world.rs`, `temper/crates/temper-fake/src/forge.rs`, `temper/crates/temper-playground/src/documents/hosted_git.rs`. The item names skein's tests, and a branch stays in its tree. Name them in your report.
- **`CI`** in `skein-fake-forge-domain/src/tests.rs` is a test's constant: `TEST` holds the literal `3`, and `CALM` keeps `ci: CI`.

## Steps

```rust
// skein-fake-forge-domain/src/domain.rs, beside `Config`
impl Config {
    /// A forge with small latencies and no faults, for test worlds, each of
    /// which names what it wants otherwise (testing-strategy.md, 10).
    pub const TEST: Self = Self { limits: Limits::TEST, /* ...fake-peers' values... */ };
}

// skein-fake-forge-domain/src/limits.rs
impl Limits {
    /// Small limits for test worlds.
    pub const TEST: Self = Self { /* ...fake-peers' values... */ };
}
```

`Config`'s `Debug` holds the proof. Before you start, in a throwaway test or `dbg!`, print `{:?}` of each site that a test can reach, and print them again after: the two must be equal. Delete it before you push. The worlds' traces read these values too, so their tests go red on a slip.

No new test: `TEST` is data, and a table repeating it would only pin the refactor.

`heavy cargo nextest run -p skein-fake-forge-domain -p skein-fake-forge-forgejo -p skein-fake-forge-github -p skein-fake-peers-tests -p skein-forge-client-world -p skein-fake-forge-tests -p github-conformance`, and `heavy cargo check --tests -p skein-forge-kit`, with the kit's dependents that `cargo metadata` names. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming the packages you changed, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the measure's reading, the net lines, and temper's five literals, which could start from `TEST` later.
