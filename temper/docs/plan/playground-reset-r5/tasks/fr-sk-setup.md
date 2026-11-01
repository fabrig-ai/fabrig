+++
id = "fr-sk-setup"
part = "forge-records"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/**", "skein/crates/skein-forge-setup/src/tests/host_forges.rs", "skein/crates/skein-forge-setup/src/tests.rs"]
effort = "high"
minutes = 60
+++

## Goal

skein's setup host takes forges at runtime: a forge added after
construction and a forge removed, so temper can set up or connect a forge
it made from a record (temper's draft
`temper/docs/design/draft/playground-reset.md`, sections 3 and 4). Today
`Host::new` takes `Settings { forges }` once and a plan for an
unconfigured forge is an owner's error (`configured`).

Acceptance:

- `skein_forge_setup::host::Request::Add { forge: Forge }` and
  `Request::Remove { forge: u16 }` (in `host/boundary.rs`, beside `Run`).
  `Add` of a number not configured, while not closing, makes it
  configured exactly as one in `Settings` (its plans admitted by its kind;
  `limits::check`'s per-forge checks run on it, an unfit forge refused by
  an `Event` naming it rather than a panic); of a configured number it is
  an owner's error (`assert!`), as a second live run per token is.
- `Remove` ends each live run and Reachable connection of the forge as a
  close ends them (their terminal `Event`s, `Why::Unexpected` or the
  closing answer the host already gives), drops its states, and makes the
  number unconfigured: a later plan naming it is the owner's error, and a
  later `Add` of the same number is allowed.
- The existing `Limits::forges` ("the most configured forges") bounds
  the forges configured at once, adds counted, removals freeing room; the
  host's tables take no other size from the configured count. An empty
  `Settings.forges` is valid.
- Tests (a new `src/tests/host_forges.rs`, registered as the crate's other
  host tests are): `an_added_forgejo_runs_its_reconcile_as_a_configured_one`,
  `an_added_github_forge_runs_its_manifest_as_a_configured_one`,
  `a_removed_forge_ends_its_live_run_and_takes_no_plan`,
  `a_removed_number_can_be_added_again`,
  `adds_past_the_forge_bound_are_refused_by_an_event`; every existing
  test passes unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r5/notes/common.md`,
  `temper/docs/plan/playground-reset-r5/notes/contracts.md`.
- `skein/docs/design/forge-setup.md`, section 6.3 (the host).
- `skein/docs/foundation/programming-model.md`, section 1.

## Owns and may change

- Owns: `skein/crates/skein-forge-setup/src/host.rs`,
  `src/host/**`, the new `src/tests/host_forges.rs` and its `mod` line.
- Nothing outside the crate: `Limits` keeps its fields.
- Options: if `Request` is not the host's input type for configuration
  (it is today: `Host::down(env, request, up, io)`), add the two as
  methods `Host::add` / `Host::remove` with the same meaning, and say so.

## Steps

1. `Add`, `Remove` under `Limits::forges`; their events.
2. Tests.
3. Narrow: `heavy cargo nextest run -p skein-forge-setup --lib -E
   'test(/host_forges/)'`.
4. `cargo fmt`; `heavy cargo clippy -p skein-forge-setup --all-targets --
   -D warnings`; focused tests of `skein-forge-setup`; `heavy cargo check
   -p temper-service --tests`. Old-forms from `skein/`. Commit and push.

## Report

As `notes/common.md` says; under `## Design`, what
`skein/docs/design/forge-setup.md`, 6.3 should gain.
