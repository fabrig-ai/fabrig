+++
id = "client-limits"
part = "test-limits"
kind = "migration"
needs = ["io-limits"]
files = ["skein/crates/skein-oauth/src/machine.rs", "skein/tests/oauth/src/lib.rs", "skein/tests/forge/machine/src/memory.rs", "skein/tests/forge/machine/tests/suite/memory.rs", "skein/docs/development/debt/D033-limits-literals-in-tests.md"]
effort = "medium"
minutes = 45
+++

# `ClientLimits::TEST`, labelled memory rows, and D033 said honestly

## Goal

The rest of what skein's D033 can pay here. `ClientLimits` is written whole 8 times (215 lines) and gets a `TEST` as `io::Limits` has. The forge machine world's memory rows of 15 bare numbers get labels. The item is then rewritten to what's left, with what the cut found: it stays open, for one type.

The whole-literal measure, which read 8 when the cut ran it:

```
rg -U --pcre2 --count-matches '(?<!struct )(?<!-> )(?<!-> oauth::)ClientLimits \{(?:(?!\.\.)[^{}]|\{(?:(?!\.\.)[^{}])*\})*\}' skein -g '*.rs' | awk -F: '{s+=$2} END{print s+0}'
```

Acceptance:

- That measure reads 0, or 1 if `TEST`'s own definition counts: say which.
- **Every field of every converted site has the value it had,** as in `io-limits`.
- `skein/tests/forge/machine/tests/suite/memory.rs`'s three rows read by field name, and `check` compares a named value. The assertions hold the same 15 numbers.
- `skein/docs/development/debt/D033-limits-literals-in-tests.md` says what's below and `python3 -I -B tools/debt/debt.py` exits 0.
- The task's net is at least 50 lines fewer, the rows' labels included.

## Inputs

- `skein/docs/development/debt/D033-limits-literals-in-tests.md`, whole.
- `skein/docs/foundation/testing-strategy.md`, 10.
- `skein/crates/skein-io/src/limits.rs`'s `TEST`, the pattern.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header, and in `skein/tests/**` and the unit tests of `skein/crates/**`, only the literal of a `ClientLimits`. The 18 updates already written from `..limits()` or `..limits().client` are the target shape: they stay.

`skein_oauth_world::client_limits()` stays, as what its readers call, and its body becomes an update from `TEST` holding its own values. `skein_oauth_world::documents()` likewise.

## Steps

```rust
// skein-oauth/src/machine.rs
impl ClientLimits {
    /// Small client limits for test worlds, overridable at each entrance.
    pub const TEST: Self = Self { /* below */ };
}
```

Each field's most common value across the literals: `uri 256, scope 64, state 64, code 64, url 1024, request 1024, parameter 0, resource 0, nonce 0, metadata 0, key_set 0, keys 0, modulus 2048, skew 0, sign_in 120 s, request_time 10 s, backoff 2 s to 8 s, attempts 3`, and `document`: `skein_oauth_world::documents()`'s values. If a validity function refuses these, change only what it refuses, and say so.

**The rows.** `Full::saturation` returns a named struct rather than `[u32; 15]`, deriving what `assert_eq!` needs, with the fields the array lists today in `skein/tests/forge/machine/src/memory.rs`. The second use of `saturation()` in the suite, at about line 36, follows. Each row is a struct update from one base holding what the rows share, so a row names only what differs:

```rust
check(Forges { forgejo: 2, github: 0 }, Saturation { repositories: 2, ..FULL }, CEILING);
```

**The item.** Update `skein/docs/development/debt/D033-limits-literals-in-tests.md`:

- **Size:** S. **Where:** `skein_fake_forge_domain::Config` in skein's tests. **Checked:** the date alone.
- **Paid when:** "`skein_fake_forge_domain::Config`'s test literals start from one constructor, and change only the fields they test (`skein/docs/foundation/testing-strategy.md`, 10)."
- **Measure:** a narrow command you write that counts that type's whole literals, with its reading now. The cut read 9, at 429 lines, all distinct, with `fake_config` in `skein-forge-kit/src/fixtures.rs` and a private `CALM` in the crate's tests.
- **What:** that `io::Limits` and `ClientLimits` start from a `TEST`, each with its whole-literal count; and what the item no longer asks, with why:
  - **`tokenizer::Limits`:** 46 of 59 literals are one line, and a mean of 3.9 of 5 fields would differ from any base, so a constructor saves nothing and says less.
  - **One- and two-field types** (`json::writer`, `json::document`, `skein_shell::Config`): a constructor can't shorten them.
  - **Local `fn limits()`:** what `testing-strategy.md`, 10, asks for, "a setup written once". 62 of 67 return other or composite types.
- Its old measure and counts go, since they counted signatures and updates and could never fall to 0.

No new test: the rows' assertions keep the same numbers, and the types hold the labels.

`heavy cargo nextest run -p skein-oauth -p skein-oauth-world -p skein-forge-machine-world`, and each package whose literal you changed. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the measure's reading, the net lines, and how you checked the values.
