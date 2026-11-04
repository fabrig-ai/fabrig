+++
id = "github-fixture-readers"
part = "fake-github"
kind = "migration"
needs = ["github-fake-surface"]
files = ["skein/crates/skein-fake-forge-github/src/tests.rs", "skein/crates/skein-fake-forge-github/src/tests/fixtures.rs", "skein/tests/fake-forge/tests/github/fixtures.rs", "skein/tests/fake-peers/tests/forge/github.rs", "skein/tests/forge/github/src/fake.rs"]
effort = "medium"
minutes = 50
+++

# fake-github: the other GitHub fixtures build on the fake's

## Goal

Four more fixtures in skein write a whole GitHub `Config` by hand. The fake's own step tests copy what is now
`fixture`, and go to it outright. The other three are scenarios of their own, so they keep their tables and take the
rest of `Config` from `fixture::config`. After this the fixture App's defaults are written once in skein. No value
changes and no test changes.

Acceptance:

- `rg -n 'token_life:' skein/crates/skein-fake-forge-github/src/tests/fixtures.rs
  skein/tests/fake-forge/tests/github/fixtures.rs skein/tests/fake-peers/tests/forge/github.rs
  skein/tests/forge/github/src/fake.rs` prints nothing.
- `skein-fake-forge-github/src/tests/fixtures.rs` has no `Row {`, `Account {` or `fake::Config {`, and no
  `fn seed_installations`.
- Every test of `skein-fake-forge-github`, `skein-fake-peers-tests`, `skein-fake-forge-tests` and
  `github-conformance` passes with the count it has at base, and no test's body changes.
- Net lines at most -130.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`; `temper/docs/plan/debt-7/notes/fake-github.md`, 1 and 5.
- `skein/docs/design/fake-forge.md`, 10.6: "A scenario's own accounts and repositories".

## Owns and may change

- Owns the files above, in their fixture functions only.
- **`skein-fake-forge-github/src/tests/fixtures.rs`:** `config()` is `fixture::config(fixture::repositories())`. Its
  `limits()` is `fixture::limits()` with what differs named: `github.inner` 8, `installations.tokens` 4.
  `domain_config()` and `seed_installations` go to `fixture::domain_config` and `fixture::install_three`, with their
  callers, or as a one-line body if a caller would otherwise change more than its call. `src/tests.rs`, `app()`: its body
  is `crate::fixture::app()`.
- **The three scenarios** keep every value that differs from `fixture::config`, by name, and drop the rest with
  `..fixture::config(repositories)`:
  - `skein/tests/fake-forge/tests/github/fixtures.rs`: `blob`, `app`, `shown`, `required_signatures`, `hook`,
    `accounts`, `tokens`.
  - `skein/tests/fake-peers/tests/forge/github.rs`: `base`, `app`, `hook`, `accounts`.
  - `skein/tests/forge/github/src/fake.rs`: `base`, `app`, `ruleset_id`, `tokens`, `hook`, `accounts`.

  A field whose value this list has wrong keeps its value: compare each line before you drop it, and say so in the
  report. `limits()` in those three stay: they are bounds a memory ceiling or a page size reads.
- **Not here:** temper's fixtures. Its builders stay in temper, whose scenarios they are.

## Steps

1. The fake's own tests, one commit. A test that reads `f::config()`'s value through a struct update needs no edit.
2. The three scenarios, one commit each.
3. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-fake-forge-github -p
   skein-fake-peers-tests -p skein-fake-forge-tests -p github-conformance --all-targets -- -D warnings`; `heavy cargo
   nextest run` naming the same packages. The memory tests of `tests/fake-forge` and `tests/fake-peers` are among them:
   a ceiling passed is a stop. Citations. Push.

## Report

The common note's items; the search's output; any value that differed from the list; net lines. No new test, so no
break.
