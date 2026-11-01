+++
id = "fr-limits"
part = "forge-records"
kind = "feature"
needs = ["store-families"]
files = [
  "temper/crates/temper-engine-domain-forge/src/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/limits.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/limits/**",
  "temper/crates/temper-service/src/forge.rs",
  "temper/crates/temper-service/src/engine.rs",
  "temper/crates/temper-service/src/setup/settings.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/records.rs",
]
effort = "medium"
minutes = 35
+++

## Goal

`forge.forges`, the forges the deployment keeps at most (default 4),
is a limit of the forge connector (contracts, section 6; draft 3,
"Limits are counts"), and it sizes skein's connection component and the
setup host in place of `forge_component.forges`, which goes as a name.
The connector's own per-forge tables move to it in `fr-seed`, which
rewrites their constructor.

Acceptance:

- `temper_engine_domain_forge::Limits` has `pub forges: u32`; `valid`
  refuses 0; `kept_bound` counts forges by it (in place of
  `client.repositories`).
- The service's limits read `forge.forges` (`limits/names.rs`), 4 in
  every profile; `forge_component.forges` is no longer a name (an
  override naming it is refused as an unknown name, as any unknown name
  is); the connection component's and the setup host's `forges` bound
  is `forge.forges` (`limits.rs`'s `small`, `engine.rs`'s
  `connection_limits` call, `setup/settings.rs`'s capacity and
  `host::Limits.forges`); `forge.rs`'s refusal naming for
  `Unusable::Forges` names `forge.forges`, and so does
  `setup/settings.rs`'s.
- The tests below pass; the service's and the connector's focused tests
  pass.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 6,
  "`forge.forges`").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  section 5 ("Couplings the new `forge.forges` count would replace").
- `temper/docs/design/protocol/configuration.md`, the limits section
  (find `forge_component.forges`), for the names' form.

## Owns and may change

Owned: the files in the header.

Neighbours: every `temper_engine_domain_forge::Limits` literal in
temper (tests, worlds, `temper/tests/**`, `temper/tools/**`): the new
field, compiler-forced, with the value 4 unless the literal's forges
need more; a test or document fixture overriding
`forge_component.forges` (`temper-service/src/tests/forge.rs` sets the
field directly: keep it, set `forge.forges`'s field instead if the
component's field is now derived).

Not here: the connector's tables (`fr-seed`), `application.forges[]`'s
bound (`fr-seed`), `docs/`.

Pre-approved options:

- If the service keeps `forge_component.forges` as a field of
  `skein_forge_connection::Limits` (it is skein's), set it from
  `forge.forges` where the service assembles the component's limits;
  default.
- If a profile or fixture needs more than 4 forges today (a world with
  five configured forges), give that literal what it needs.

## Steps

1. `temper-engine-domain-forge/src/limits.rs`: the field, doc line
   "Forges kept at most (`forge.forges`; draft 3)", `valid`, and
   `kept_bound`'s count.
2. `temper-service`: the name in `limits/names.rs` beside
   `forge.repositories`; the profiles' value; the component's and the
   host's bound from it; the refusal names.
3. Tests: `temper-engine-domain-forge/src/tests/limits.rs`:
   `zero_forges_are_invalid`, `kept_bound_counts_forges`;
   `temper-service/src/tests/limits.rs`:
   `forge_forges_overrides_the_component_and_host_bounds`,
   `forge_component_forges_is_no_longer_a_name`. Narrow runs by name.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-service --all-targets -- -D warnings`; their focused tests;
   `heavy cargo check --tests` for `temper`, `temper-engine-domain`,
   `temper-engine-domain-world`, `temper-engine-forge-world`,
   `temper-engine-protocol-forge-world`, `temper-protocol-store`,
   `temper-codecs-world`, `temper-system-world`,
   `temper-one-process-world`, `temper-fake`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `Limits.forges`, the name, and
where the component's and host's bounds now come from.

## Answers

2026-11-01. May fr-limits update the stale `kept_bound` assertions in
`temper-engine-domain-forge/src/tests/records.rs`
(`fits_refuses_a_field_past_its_bound`)? Yes (option A): the file is
now in this task's header. They test this task's acceptance
(`kept_bound` counts forges), and `store-families`, which wrote them, is
done. In that test only: set `l.forges = u32::MAX` in place of
`l.client.repositories = u32::MAX` (still `kept_bound` None and
`!valid`), and expect `LIMITS.tasks + 1 + 2 * LIMITS.forges + 2` (or
the formula `kept_bound` now has) in place of `CLIENT.repositories`.
Nothing else in the file changes.
