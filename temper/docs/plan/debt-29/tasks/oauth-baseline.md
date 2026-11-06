+++
id = "oauth-baseline"
part = "oauth-owner"
kind = "feature"
needs = []
files = ["skein/tests/oauth-accounts/tests/suite/baseline.rs", "skein/tests/oauth-accounts/tests/suite/main.rs", "skein/tests/oauth-accounts/src/sign_in/cuts.rs"]
effort = "high"
minutes = 60
+++

# A recorded digest of what each story of the oauth-accounts world does

## Goal

Skein's D048 says a refactor of this world "needs a baseline of trace and facts per seed and story first", since most sign-in tests are `drop(run(..))`. This task adds it, and changes nothing else. `oauth-one-owner`, `oauth-one-driver` and `oauth-one-restart` lean on it, and `oauth-close` deletes it.

Acceptance:

- `tests/suite/baseline.rs` holds one `#[ignore]`d test per entry point below, under `baseline::`, and `sign_in/cuts.rs` holds its own rows.
- All pass, recorded, in two separate runs: a digest that differs between two processes on the same code is a question, since no refactor could lean on it.
- **It bites:** break `PasteThenListener`'s hold by hand (`src/sign_in/owner.rs`, `self.story == Story::PasteThenListener && self.callback != ..`), run it, see a row go red, and restore. Your report names what you broke and which rows failed.
- No line of `src/` changes but the rows in `sign_in/cuts.rs`.
- `heavy cargo nextest run -p skein-oauth-accounts-world` runs what it ran before: the new tests are ignored there.

## Inputs

- `temper/docs/plan/debt-29/notes/oauth-world.md`, whole.
- `skein/docs/development/debt/D048-oauth-accounts-world-copies.md`.
- `skein/docs/foundation/testing-strategy.md`, 10.
- `temper/docs/plan/debt-29/notes/common.md`.

## Owns and may change

Owns the files in the header. `src/sign_in/cuts.rs` is `#[cfg(test)]` inside the library, so `tests/suite` can't reach it: it takes its two rows inside its own test, with the same `#[ignore]`. This task can't end with fewer lines: it adds about 150 of code and about 270 rows, all deleted by `oauth-close`.

- **A row that panics at base,** such as a faulted sign-in story outside the set the tests run faulted, is dropped, and named in your report. Hold no row a test doesn't already hold true.
- **For hardness, add seeds, not stories.** Don't widen past the table.
- **If nextest's global timeout of 150 s stops a test,** split that entry point's rows by story group, keeping the names under `baseline::`.

## Steps

```rust
// tests/suite/baseline.rs
//! What each story does, as digests, while D048's owners merge. Deleted by
//! the plan's `oauth-close`.

/// The first 8 bytes of a SHA-256, in hex.
fn short(text: &str) -> String { /* skein_lib::sha256::digest */ }

/// One row: the story, how it ran, what the sim saw, what the owner saw.
fn row(name: &str, trace: &str, facts: &str) -> String { /* "name trace facts\n" */ }

const SIGN_IN_RUN: &str = "\
Omit:plain 0123456789abcdef 0123456789abcdef
...
";

#[test]
#[ignore = "D048 baseline, deleted by oauth-close"]
fn sign_in_run() {
    let mut lines = String::new();
    // each story of the entry point, by name
    assert_eq!(lines, SIGN_IN_RUN, "a story's digest moved; every row is printed for a paste");
}
```

A trace digest is over `skein_sim::render(seed, &outcome.trace)` for a run, and over the joined `Vec<String>` for a protocol run. A facts digest is over `format!("{:?}", facts)`, and `private`'s rows add `machine.reads`, `writes`, `syncs`, `renames` and `faulted`. Don't record `iterations`, `end` or `peaks()`: client layout and an idle wakeup can move them legitimately. Skip issuer observations: pid 0's trace covers them. The inside is yours within this shape.

| Entry point | Rows, as the survey counted |
|---|---|
| `sign_in::run`: 40 story leaves unfaulted, and about 30 faulted where tests already run them faulted | 70 |
| `sign_in::protocol`: 33 leaves, two seeds (7 stories are barred at the top of `protocol.rs`) | 66 |
| `sign_in::run_private`, `run_private_return` | 11 |
| `world::run`: 12, faulted or not | 24 |
| `world::protocol`: 12, two seeds, and `tls_close`, two | 26 |
| `private::run` | 22 |
| `private_cuts::world`: uncut, and two cuts | 3 |
| `sign_in/cuts.rs` | 2 |
| `accounts::run`, `protocol` | 18 |
| `shared::run`, `protocol` | 16 |
| `sign_out::run`, `protocol` | 15 |

Don't name a test under `fuzzy::`: `.config/nextest.toml` gives that its own rules.

Then the note's **Common to the tasks**. Commit, push.

## Report

As your text says, with the rows recorded per entry point, each row dropped and why, how long the baseline command took, what the break showed, and that the two runs agreed.
