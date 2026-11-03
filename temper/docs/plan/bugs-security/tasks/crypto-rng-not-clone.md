+++
id = "crypto-rng-not-clone"
part = "crypto-rng"
kind = "feature"
needs = ["jig-accounts-check", "temper-accounts-check"]
files = [
  "skein/crates/skein-lib/src/crypto_rng.rs",
  "skein/crates/skein-lib/src/tests/crypto_rng.rs",
  "skein/crates/skein-forge-setup/src/tests/github_host.rs",
  "skein/crates/skein-forge-setup/src/tests/github_states.rs",
  "skein/docs/design/lib.status.md",
]
effort = "high"
minutes = 50
+++

# CryptoRng is neither Copy nor Clone

## Goal

`skein/crates/skein-lib/src/crypto_rng.rs`'s `CryptoRng` derives `Clone`, so a clone repeats the
random stream, against `skein/docs/design/lib.md`, 8 ("It is neither `Copy` nor `Clone`, since a
copy would yield its bytes again"). Remove the derive. Its two owners' copies are gone
(`jig-accounts-check`, `temper-accounts-check`); two skein-forge-setup tests clone the host's
generator as an oracle for its next bytes, and replay a fresh one from the same fixture instead.

Acceptance:

- `CryptoRng` derives nothing; its hand-written `Debug` and `Hash` stay as they are.
- A trait check in skein-lib's step tests holds `CryptoRng: !Clone` (and so `!Copy`), and
  fails on the old code; a control in the same test shows the check sees a `Clone` type.
- The two skein-forge-setup tests keep every assertion and expected byte, their oracle a fresh
  generator from the fixture the world's host was made with, advanced by what the host drew
  before the clone point.
- Nothing in any tree clones a `CryptoRng`: `heavy cargo check` of the packages below passes.
- `lib.status.md` drops the entry this closes. clippy clean.

## Inputs

- `temper/docs/plan/bugs-security/notes/common.md`.
- `skein/AGENTS.md`.
- `temper/docs/plan/bugs-security/inputs/request.md`, item 3.
- `skein/docs/design/lib.md`, 8, the `CryptoRng` bullet (its last sentence written by this
  cut), and 4 (`ReplyTo`, the other type lib promises is neither `Copy` nor `Clone`, untested).
- The surveyed clone sites, all four: `jig/testing/jig-test-service/src/accounts.rs` and
  `temper/crates/temper-service/src/accounts.rs` (gone through this task's `needs`);
  `skein/crates/skein-forge-setup/src/tests/github_host.rs` ~506 (module
  `host::install_tests`, test
  `stateless_accept_keeps_the_checked_installation_without_drawing_or_taking_state`) and
  `src/tests/github_states.rs` ~303 (module `host::state_tests`, test
  `a_manifest_uses_the_normal_write_slot_and_admission_precedes_issuance`). No type holding a
  `CryptoRng` derives `Clone`, and no `T: Clone` bound takes one.

## Owns and may change

- `skein/crates/skein-lib/src/crypto_rng.rs`: the derive line only.
- `skein/crates/skein-lib/src/tests/crypto_rng.rs`: the trait check below.
- `skein/crates/skein-forge-setup/src/tests/github_host.rs` and `src/tests/github_states.rs`:
  in the two tests named above, the oracle's line (and a line advancing it), nothing else.
- `skein/docs/design/lib.status.md`: delete "8: `CryptoRng` without `Clone` (it derives it; two
  owners and two tests clone one)." from its **Not yet** line, keeping "3: `Slab::get2_mut`.";
  the date line today's.

Nothing else. If another clone of a `CryptoRng` shows up in a package below that this list does
not name, in skein fix it the same way (a replay from its fixture, or a generator made on a
stream of its own) and say so; in jig, smith or temper stop and ask, naming the file.

Pre-approved options:

- The trait check is the inherent-constant probe (no new dependency; nextest runs no doctests,
  so a `compile_fail` doctest would not run in the gate):

  ```rust
  /// Whether `T` is `Clone`, decided at compile time: the inherent constant applies only where
  /// `T: Clone`, the trait's default everywhere else.
  struct Probe<T>(core::marker::PhantomData<T>);
  trait Fallback {
      const CLONE: bool = false;
  }
  impl<T> Fallback for Probe<T> {}
  impl<T: Clone> Probe<T> {
      const CLONE: bool = true;
  }

  #[test]
  fn a_generator_is_neither_copied_nor_cloned() {
      assert!(Probe::<crate::Token>::CLONE, "the probe sees a Clone type");
      assert!(!Probe::<CryptoRng>::CLONE, "a copy would yield its bytes again (lib.md, 8)");
  }
  ```

  A lint the probe trips takes a scoped `#[expect(.., reason = "...")]` on the item. If the
  probe does not compile on the workspace's toolchain, write a `compile_fail` doctest on
  `CryptoRng` instead, run it with `heavy cargo test -p skein-lib --doc`, and say in the report
  that the gate's nextest does not run it.
- In `github_host.rs`, the host has drawn nothing at the clone point (the world's `state()`
  issues with fixed bytes): the oracle is `fixtures::crypto()` (`src/tests/host.rs` ~101, which
  both files import as `fixtures`), the generator `ConvertHost::new`
  (~119) makes the host with. In `github_states.rs`, the
  host drew the admitted run's issued state before the clone point (`host/run.rs` ~359,
  `issue_state`: 16 bytes): the oracle is `fixtures::crypto()`, which the world's `with` (~52)
  makes the host with, those bytes drawn and dropped.
  If what the host drew before cannot be read from the code, stop and ask.

## Steps

1. The trait check first; `heavy cargo nextest run -p skein-lib --lib -E
   'test(=tests::crypto_rng::a_generator_is_neither_copied_nor_cloned)'` on the old code fails
   on the second assertion; keep the output. (Check the module path in nextest's list.)
2. Drop the derive; fix the two forge-setup oracles.
3. Narrow tests, one at a time:
   - the trait check again: passes;
   - `heavy cargo nextest run -p skein-lib --lib -E 'test(/crypto_rng|digest/)'`;
   - `heavy cargo nextest run -p skein-forge-setup --lib -E 'test(/stateless_accept_keeps_the_checked_installation|a_manifest_uses_the_normal_write_slot/)'`: 2 passed;
   - `heavy cargo nextest run -p skein-forge-setup --lib`;
   - every crate that names `CryptoRng`, checked (each a separate command):
     `heavy cargo check -p skein-oauth-accounts --tests`, `-p skein-oauth-accounts-world`,
     `-p skein-forge-setup-world`, `-p skein-oauth-issuer-world`, `-p smith-local-service`,
     `-p jig-protocol-accounts`, `-p jig-protocol-engine-clients`, `-p jig-protocol-accounts-world`,
     `-p jig-test-service`, `-p temper-service`, each with `--tests`.
4. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `skein/`;
   `heavy cargo clippy -p skein-lib -p skein-forge-setup --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root.
5. One commit for the code and its tests, one for `lib.status.md`, each saying why; push.

## Report

As the implementer's text says, and: the failing run on the old code, the passing runs, each
check's result, and the bytes each forge-setup oracle skips. `ReplyTo`'s promise (lib.md, 4) has
no test: name it under `## Left`.
