+++
id = "judge-parses"
part = "judge"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-tls/src/rs256.rs",
  "skein/crates/skein-tls/src/lib.rs",
  "skein/crates/skein-tls/src/tests/rs256.rs",
  "skein/tests/tls/fixtures/make.sh",
  "skein/tests/tls/fixtures/rsa-2050.der",
  "skein/crates/skein-forge-connection/src/limits.rs",
  "skein/crates/skein-forge-connection/src/limits/tests.rs",
  "skein/crates/skein-forge-connection/src/component/tests.rs",
  "skein/crates/skein-fake-forge-github/src/app.rs",
  "skein/crates/skein-fake-forge-github/src/tests/app.rs",
  "skein/docs/design/tls.status.md",
  "skein/docs/design/forge-connection.status.md",
  "skein/docs/design/fake-forge.status.md",
  "skein/docs/development/skein-limits-heavy.md",
  "skein/docs/plan/skein-limits-rows/plan.md",
]
effort = "medium"
minutes = 75
+++

# The access judge parses, and the fake verifies: neither signs to check

## Goal

A GitHub App's two checks each pay for an RS256 signing they don't need. `limits::access` (`skein-forge-connection`) signs `b"skein"` to judge a key, and `verified_claims` (`skein-fake-forge-github`) signs a token's input again to compare it. One new function in `skein-tls`, `rs256_public_pkcs1`, serves both: the judge parses, and the fake verifies by the key's public half. `Jwt::bearer`'s signing is the product's and stays, as does parsing at each signing.

Acceptance:

- `skein_tls::rs256_public_pkcs1` exists as `skein/docs/design/tls.md`, 4 has it. Its refusals share code with `sign_rs256`, so the two can't drift.
- `limits::access` and `verified_claims` no longer call `sign_rs256`. Under callgrind, `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded` calls `sign_rs256` once.
- A wrong key is still judged wrong, in each of the three places below, and each test bites: remove the check, see it fail for that reason, restore.
- The refusals of `verified_claims` stay exact: `github_fake_jwt_refused` and `github_fake_cached_signature_checks_claims_and_changed_bytes` pass unchanged.
- `plan.md`, *Before and after*, *judge-parses*, gives the counts below and who pays.
- The focused tests of the packages in step 7 pass.

## Inputs

- `skein/docs/plan/skein-limits-rows/inputs/request.md`, item 1.
- `skein/docs/plan/skein-limits-rows/notes/common.md`: how to measure, and the rules.
- `skein/docs/design/tls.md`, 4: *An App's signature* and *An App key's public half*.
- `skein/docs/design/forge-connection.md`, 3.5 and 5.2.
- `skein/docs/design/fake-forge.md`, 10.1, *Credentials*.
- `skein/docs/development/skein-limits-heavy.md`, *Found and not fixed*, *Forge hand RSA*.

The design above is written. The task builds it, and takes out its **Not yet** lines.

## Owns and may change

Owns the files in the header. Beyond them:

- `skein/docs/design/tls.status.md`, `forge-connection.status.md` and `fake-forge.status.md`: this task removes the one **Not yet** entry for this work in each, which a departure from the implementer's text allows. In `forge-connection.status.md`, that leaves no **Not yet** line, so the line goes. Other tasks may be editing these files: touch your entry alone.
- `skein/docs/development/skein-limits-heavy.md`, *Forge hand RSA*: one sentence, saying `limits::access` no longer signs (`skein/docs/design/tls.md`, 4), so that what's left is `Jwt::bearer`'s.
- `skein/crates/skein-forge-connection/src/app.rs` and `Verified` stay as they are. `Verified`'s cache stays too.

Doesn't change: `sign_rs256`'s signature or what it accepts, `sign_rs256_pkcs8`, `rs256_public`, `verify_rs256`, `Jwt::bearer`, `skein/tests/forge/github/src/live.rs` (a live tier, whose own key check signs).

Options, each decided:

- **A heap ceiling reads a smaller peak now:** the ceiling stays. If a test asserts a tight fit and fails for that, lower its bound to what the measure shows, and say so in your report.
- **The test key with a changed `e` is refused at parse** (ring may have changed from what the design says), so `rs256_public_pkcs1` and `sign_rs256` agree on it: make the row say so, drop the sentence *ring checks a key's private values…* from `tls.md`, 4 and the clause in `forge-connection.md`, 5.2, keep only that a key which won't sign stops the forge, and say so in your report.
- **ring refuses the 2050-bit key at parse:** keep the size check, keep the row, and say in your report that nothing can reach the check.
- **The 2050-bit DER is past `RS256_KEY_BYTES`:** generate again until it fits. The arithmetic says 1194 bytes at most.

## Steps

1. **Measure before,** each alone by the common note's command, on your branch's base:
   - `-p skein-forge-machine-world --test suite`, `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded`, which read 30,917,000 Ir with two signings at 22,621,385 Ir;
   - `-p skein-fake-forge-github --lib`, `tests::app::github_fake_jwt_accepted`;
   - `-p github-conformance --lib`, `tests::github_conformance_names_a_token_expiring_within_30_days`.

   If a name has moved, take the test holding it now and say so. Quote the `Collected` count and the inclusive count of `sign_rs256` and its calls.

2. **`skein-tls`.** In `rs256.rs`:

   ```rust
   /// The public half of a key [`sign_rs256`] takes: PKCS#1 DER of a 2048-bit
   /// key, within [`RS256_KEY_BYTES`], that ring accepts (tls.md, 4). It parses
   /// as a signing does and never signs, so a key ring refuses only when it
   /// signs has a public half.
   #[must_use]
   pub fn rs256_public_pkcs1(key: &[u8]) -> Option<Rs256Public>
   ```

   - A private `fn app_key(key: &[u8]) -> Option<ring::sign::RsaSigningKey>` holds the portable target's refusal, the `RS256_KEY_BYTES` bound and ring's parse. `sign_rs256` and `rs256_public_pkcs1` both start from it.
   - `issuer_components` splits at its `OCTET STRING`: a private `pkcs1_components(&[u8])` reads version 0, `n` and `e` from an `RSAPrivateKey`, and both public functions use it. `rs256_public` behaves as it does now.
   - `rs256_public_pkcs1` refuses a modulus that isn't 256 bytes once unsigned. `sign_rs256` keeps its `try_from`.
   - Export it from `lib.rs`, in the existing `pub use rs256::{...}`.

3. **A 2050-bit key.** `skein/tests/tls/fixtures/make.sh`: the loop becomes `for bits in 1024 2050 3072`, and its header comment says what `rsa-2050.der` is: within `RS256_KEY_BYTES`, yet too large to sign. Make that file alone, by the loop's two `openssl` lines, leaving the other fixtures as they are. It's a public test key like its neighbours.

4. **Tests in `skein-tls`,** in `src/tests/rs256.rs`:
   - `rs256_signs_with_a_2048_bit_pkcs1_key_alone` gains the row `("2050-bit PKCS#1", fixture("rsa-2050.der"))`. Move its rows into a function, and read it from the new test below too.
   - `the_app_public_half_refuses_every_key_signing_refuses`: for each of those rows, `rs256_public_pkcs1` is `None`. Bite by removing the modulus check, and see the 2050-bit row fail.
   - `the_app_public_half_of_the_rfc_key_verifies_its_signature`: the RFC key gives `raw(N)` and `raw(E)`, and `verify_rs256` with them holds for `SIGNATURE`.
   - `a_key_with_another_e_has_a_public_half_and_will_not_sign`: the RFC key built by `integers` with `e` = `0x010003` in place of `0x010001`. `rs256_public_pkcs1` is `Some`, and `sign_rs256` is `None`.

5. **The judge.** In `limits.rs`, `access`:

   ```rust
   if key.len() > usize_of(l.key_bytes) || skein_tls::rs256_public_pkcs1(key.expose()).is_none() {
       return Err(Value::Key);
   }
   ```

   Tests, in `limits/tests.rs`:
   - The row `"key will not sign"` becomes `"key ring refuses"`, and `"key before user"` stays.
   - `a_github_key_ring_refuses_is_judged_wrong`, whose rows are fixture `KEYS[0]` one byte short and with its last byte changed, each `Err(Value::Key)`. Bite by removing the `rs256_public_pkcs1` check, and see both fail.

   In `component/tests.rs`, beside `renewal_local_signing_refusal_stops_invalid_key` and on the same `StepFixture`: `a_key_that_parses_and_will_not_sign_stops_before_any_request`. Fixture `KEYS[0]` with its `e` changed: find `[2, 3, 1, 0, 1]` with `skein_lib::bytes::find`, and set the byte after `1, 0` to `3`. `access` takes it, then the first signing stops the forge `Stopped::Invalid { value: Value::Key }`, and nothing was written to the peer. This holds what `skein/docs/design/forge-connection.md`, 5.2 now says. Bite by making `bearer`'s failure not stop the forge.

6. **The fake.** In `app.rs`, `verified_claims`:

   ```rust
   let public = skein_tls::rs256_public_pkcs1(app.key).ok_or(Refused::Signature)?;
   if !skein_tls::verify_rs256(&public.modulus, &public.exponent, input, &signature) {
       return Err(Refused::Signature);
   }
   ```

   `check`'s comment stops saying *deterministic*. No test changes: the rows `"another key"` and `"changed claims"` in `github_fake_jwt_refused`, and the key change in `github_fake_cached_signature_checks_claims_and_changed_bytes`, hold it. Bite by ignoring `verify_rs256`'s answer, and see those rows fail. Also run `skein-forge-machine-world`'s `a_jwt_signed_by_another_key_is_401`.

7. **Measure again, and fill `plan.md`.** The same commands as step 1. *Before and after*, *judge-parses*, gets the three tests' counts before and after, with their `sign_rs256` calls and inclusive counts, and which packages pay for each of the two checks (`rg` the callers; don't estimate a total).

8. **The end of a run.** `cargo fmt`, old forms, citations, and `heavy cargo clippy -p skein-tls -p skein-forge-connection -p skein-fake-forge-github --all-targets -- -D warnings`. Then `heavy cargo check -p skein-tls`, since it's also checked alone, and the focused tests of `skein-tls`, `skein-forge-connection`, `skein-fake-forge-github`, `skein-forge-machine-world`, `skein-forge-setup-world`, `github-conformance` and `skein-fake-peers-tests`. What `limits::access` means changed for a key ring refuses only when it signs, so its readers' tests run. Commit and push.

## Report

The counts, what each bite broke and how it failed, which options you took, and any package whose test you changed. Your implementation and check minutes apart.
