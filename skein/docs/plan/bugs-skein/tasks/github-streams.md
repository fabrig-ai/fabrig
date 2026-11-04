+++
id = "github-streams"
part = "fake-forges"
kind = "feature"
needs = []
files = ["skein/crates/skein-fake-forge-github/src/service.rs", "skein/crates/skein-fake-forge-github/src/tests/installations.rs", "skein/crates/skein-fake-forge-github/src/delivery.rs", "skein/crates/skein-fake-forge-github/src/pages.rs", "skein/crates/skein-fake-forge-github/src/installations.rs"]
effort = "medium"
minutes = 35
+++

# The fake GitHub's stores draw apart

## Goal

`skein-fake-forge-github`'s `Service::new` hands one `seed` to four stores beside its own `Rng`:
`logs::Store` (its blob key), `delivery::Store`, `pages::Store`, `Installations`. So their draws
are one sequence: a delivery's GUID bytes and a page state's nibbles are the same draws, and so on.
Each store gets a seed of its own, still from the one scenario seed, so a seed still replays.
(Bug 14, its seen half.) A change of meaning: drawn values move, so its readers' tests run.

Acceptance:

- `Service::new` derives the four stores' seeds by drawing them from `Rng::new(seed)`, in one place,
  with one comment saying why. `Rng::new` starts a counter and `next_u64` hashes it, so a seed
  drawn this way starts an unrelated sequence, where `seed + n` would only shift one.
- `Service`'s own `rng` stays `Rng::new(seed)`, so a fault scenario keeps its draws.
- The store constructors keep their signatures.
- A test shows the five first draws pairwise different, and fails on the old code, where all five
  are equal.

## Inputs

- `skein/docs/plan/bugs-skein/notes/common.md`.
- `skein/docs/design/fake-forge.md`, 10.3 (seeded UUIDs and the replay), and 4 ("a seed replays").

## Owns and may change

- Owns: `src/service.rs` (`Service::new`).
- `src/delivery.rs`, `src/pages.rs`, `src/installations.rs`: at most one `#[cfg(test)] pub(crate)
  fn` each, handing the test the store's next draw. `Rng` is not `Clone`, so a fresh `Service` is
  drawn from directly.
- `src/tests/installations.rs`, or a new test module beside `service_app` (then its `mod` line in
  `src/tests.rs`): the test. The blob key's first word stands for `logs::Store`, read as the crate
  reads it elsewhere; if no crate-private path reaches it, a `#[cfg(test)]` accessor in `src/logs.rs`.
- A test elsewhere that pins a drawn value, a token, a GUID or a key: make it read the value from
  the fake rather than pin it, and list it. None was found by grep.

## Steps

1. Write `each_store_draws_its_own_sequence`, with one `Service::new(config, &limits, seed)` from the
   crate's fixtures; run it on the old code and quote the failure.
2. The derivation.
3. Focused tests: `skein-fake-forge-github`, `skein-fake-forge-tests`, `skein-fake-peers`,
   `skein-fake-peers-tests`, `skein-forge-connection`, `skein-forge-setup`,
   `skein-forge-protocol-world`, `skein-forge-setup-world`, `github-conformance`, `temper-fake`,
   `temper-one-process-world`, each through `heavy cargo nextest run -p`. `cargo fmt`,
   `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-fake-forge-github
   --all-targets -- -D warnings`.
4. Commit, push.

## Report

As the implementer's text says, with what the test showed on the old code.
