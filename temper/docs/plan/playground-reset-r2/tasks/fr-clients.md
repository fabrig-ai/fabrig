+++
id = "fr-clients"
part = "forge-records"
kind = "feature"
needs = ["fr-registry"]
files = [
  "temper/crates/temper-service/src/clients.rs",
  "temper/crates/temper-service/src/clients/**",
  "temper/crates/temper-service/src/family.rs",
  "temper/crates/temper-service/src/family/**",
  "temper/crates/temper-service/src/tests/registry_clients.rs",
  "temper/crates/temper-service/src/tests/family.rs",
  "temper/crates/temper-service/src/tests/clients*.rs",
]
effort = "medium"
minutes = 50
+++

## Goal

A forge built from its record gets what a configured forge gets on the
clients' side (`temper/docs/design/draft/playground-reset.md, 3`, "Its
client"): its browser sign-in metadata (`clients/bundle.rs`, an owned
Forgejo's OAuth origin), and its pull, landing and document browser
URLs (`family.rs`), which today read the static forge list.
`crate::clients::add` (fr-registry's empty hook) does it.

Acceptance, new tests in `temper-service`
(`src/tests/registry_clients.rs`):

- `a_made_forges_links_name_its_git_base`: after `registry::made` for
  a Forgejo record, a pull row of one of its repositories carries the
  pull URL built from the record's `git`, as a configured forge's does.
- `a_made_owned_forge_signs_in`: an owned Forgejo record with a public
  origin gets its sign-in metadata in the bundle the clients read, as
  a configured one's (`clients/bundle.rs:341,388`).
- The existing clients and family tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/design/draft/playground-reset.md`, section 3.
- `temper/docs/design/client/forge.md` (the forges' rows and links),
  the sections `family.rs` cites.
- `tasks/fr-registry.md` (`Built`, `install`, the `add` hooks).

## Owns and may change

Owns, after `fr-registry` (`fr-service` owns `family.rs`'s request
translation next, D4 after it): `src/clients.rs`, `src/clients/**`,
`src/family.rs`, `src/family/**`. The other fr- consumer tasks run at
once and own `forge*`, `engine.rs`, `workers.rs`, `values.rs`,
`setup*`. `src/route.rs` is fr-registry's: a call site there only.

May change: client test fixtures; anything `notes/scope.md` lists as
forced.

## Steps

1. Where `family.rs` and `clients.rs` read a forge's kind, git base or
   web base from a forge list passed in (`family.rs:197,305,443,451,951,
   1031`; `clients.rs:453,545,677`), read it from the list the workers
   hold (`engine.workers.forges`, which `fr-workers` grows) or from
   `engine.registry` for a forge not seeded, through one lookup
   function (`fn forge_of(engine, number) -> Option<&settings::Forge>`)
   so both are seen. If `fr-workers` has not merged, the registry
   lookup alone serves forges made at runtime.
2. `crate::clients::add(engine, built)`: the forge's sign-in metadata
   (`bundle.rs`'s `forge_metas`/`fill_forge_sign_in` for one forge) put
   into the bundle's forge sign-in rows, if its setup entry gives an
   owned public origin.
3. Tests; `cargo fmt`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-service`, then `-p temper-client-world`; old forms.
4. Commit and push.

## Report

The standard report, plus: each reader moved to the lookup; whether
the bundle's sign-in rows can change after the bundle is served.
