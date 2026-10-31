+++
id = "fr-skein-setup"
part = "forge-records"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge-setup/src/host.rs",
  "skein/crates/skein-forge-setup/src/host/**",
  "skein/crates/skein-forge-setup/src/tests/**",
  "skein/tests/forge/setup/**",
]
effort = "high"
minutes = 50
+++

## Goal

skein's setup host component is built with every forge it serves
(`skein/docs/design/forge-setup.md, 6.3`, its `Settings.forges`) and
knows no other. temper now makes forges at runtime
(`temper/docs/design/draft/playground-reset.md, 3`): add a request
that adds one forge to the running host, checked as `Host::new` checks
one, so its plans can run.

Acceptance, new tests in `skein-forge-setup` (beside the host's own,
`src/host/tests.rs`, or a new `src/tests/host_add.rs`):

- `an_added_forge_runs_its_plans`: a host built with one Forgejo is
  asked `Add` for a second; it answers `Added { refused: None }`; a
  reconcile `Run` naming the second forge runs as the first's does (its
  needs go out to the second's address).
- `an_add_past_the_limit_or_of_a_held_forge_is_refused`: past the
  host's forge bound, or naming a forge it holds, `Add` is refused and
  the host is unchanged.
- The host's existing tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `skein/AGENTS.md`; `skein/docs/foundation/programming-model.md`,
  sections 1 and 10.
- `skein/docs/design/forge-setup.md`, 6.3 (the host component) and 6.5
  (its limits).

## Owns and may change

Owns the host's files (`src/host.rs`, `src/host/**`) and the crate's
tests. May change: the setup world (`skein/tests/forge/setup`) and any
owner's exhaustive match on `skein_forge_setup::host::Event` or
`Request` in any tree (temper-service's `src/setup.rs` and
`src/setup/lifecycle.rs`: an `Added` arm that does nothing, commented
"fr-setup adds forges at runtime"), as `notes/scope.md` says. The
design document is not yours.

## Steps

1. In `src/host/boundary.rs`:

   ```rust
   // Request
   /// Adds one forge to the running host, checked as `Host::new` checks
   /// each of its settings' forges.
   Add { forge: settings::Forge },
   // Event
   /// `Add`'s one answer; `refused` names what `Host::new` would refuse.
   Added { forge: u16, refused: Option<Unusable> },
   ```

   `forge` in `Added` is the forge's number as the host keys its forges
   (read how a `Run`'s plan names its forge: by its given's number or
   by its position; key `Added` the same way and say which). `Unusable`
   is `Host::new`'s refusal; if it is not per forge, add the per-forge
   variant it needs (`notes/scope.md`, companion items).
2. Factor `Host::new`'s per-forge check into one function `new` and
   the `Add` arm both call. The added forge joins every table the host
   sizes per forge; after `Close`/`Abort`, an `Add` is ignored.
3. Tests (Goal); `cargo fmt`; `heavy cargo clippy -p skein-forge-setup
   --all-targets -- -D warnings`; one at a time: `heavy cargo nextest
   run -p skein-forge-setup`, then `-p skein-forge-setup-world`, then
   `heavy cargo check --tests -p temper-service -p temper-forge-setup`;
   `python3 -I -B scripts/old_forms.py` from `skein/`.
4. Commit and push.

## Report

The standard report, plus: how `Added` keys its forge; what
forge-setup.md 6.3 should say of a forge added at runtime.
