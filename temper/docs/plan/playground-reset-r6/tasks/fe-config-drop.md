+++
id = "fe-config-drop"
part = "fs"
kind = "migration"
needs = ["fe-world-reset", "sv-startup-refuse"]
files = [
  "temper/crates/temper-engine-domain-forge/**",
  "temper/crates/temper-engine-domain/**",
  "temper/tests/engine/**",
]
effort = "medium"
minutes = 70
+++

## Goal

The forges' document path leaves the engine (`notes/contracts.md` T1, T4;
`notes/sv-progress.md`, contract request 1, step (d)): `root::Config` loses `forges` and
`identities`, the connector knows forges only as records, every forge's git identity is
its number, and a number with no record is a removed forge's. This is the end of the
transition `fe-records` began.

The lead adds to `needs` sv's task that refuses `application.forges`, `identities` and
`setup` (named `sv-startup-refuse` in `notes/sv-progress.md`): after it, serve's root
`Config` literal has no document forges to pass.

Acceptance:
- `forge::Domain::new(l, seed, config, machine, under_box)`; `root::Config` has no
  `forges` and no `identities`.
- The configured list is the records' projections alone; `GitIdentity { forge, credential:
  forge as u32 }` for every record; `forge_ended(n)` is true for a `Removing` record, for
  a number with no record, and for nothing else; `MakeForge`'s document refusal goes;
  the playground's numbering past document forges goes.
- Worlds: the walking world's stores seed forge 1 as a record (`forges::record(1, Forgejo,
  Connected)` through `forges::rows`), the forge its writer and repositories name; every
  site that set `config.forges`/`identities` seeds records instead
  (`forges::from_configured`).
- Every test of the owned packages, of `temper-client-world`, `temper-system-world` and
  `temper-service` passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T1, T4;
  `notes/fe-surface.md` section 1; `notes/sv-progress.md`, contract request 1.
- `temper/tests/engine/domain/src/forges.rs` (`fe-records`' world helper).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain-forge/`, `temper/crates/temper-engine-domain/`,
  `temper/tests/engine/*`.
- May change, compiler-forced or mechanical only (list each file):
  - `temper/crates/temper-service/`: the root `Config` literals lose `forges` and
    `identities` (`src/system.rs`, `src/startup/checks.rs`, `src/tests/fixtures.rs`,
    `src/tests/welcome_policy.rs`); a function left unused by that (`crate::forge::config`'s
    identities part, `startup/application.rs::configured`) is deleted with its own tests.
  - `temper/tests/client/`, `temper/tests/system/`, `temper/tests/conformance/`: sites that
    set `config.forges` or `config.identities` seed the same forges as records through
    `temper_engine_domain_world::forges`, nothing else.
- Pre-approved options: a world story that relied on a forge number configured nowhere
  (neither document nor record) now meets an ended forge: seed a record for it and say so.

## Steps

1. Forge and root: the parameters and fields go; the transition's rules go.
2. Worlds and the listed sites: seeding.
3. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain-forge --lib`; `-p
   temper-engine-domain --lib`; `-p temper-engine-domain-world --test suite`; `-p
   temper-engine-forge-world`; `-p temper-engine-protocol-forge-world`; `-p
   temper-client-world --test suite`; `-p temper-system-world`; `-p temper-service --lib`;
   `heavy cargo check --workspace --all-targets`; `cargo fmt --all`; `heavy cargo clippy` on
   every changed package `--all-targets -- -D warnings`; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
4. Commit, push your branch.

## Report

- Every file outside the owned crates edited.

## Answers

2026-11-01. May this task handle the seeded forge's `Register` request in
`temper/tests/system/src/composed.rs`? Yes (option A): the task's own acceptance names
`temper-system-world`, and the request arrives there because of this task's seeding (the
walking fixture's writer names forge 1, whose record now registers on restore), so the
repair is compiler-forced in kind and the task's. Add to "May change", mechanical only:
`temper/tests/system/src/` engine-output matches (`composed.rs`, and `forge.rs` if it meets
the same request) consume `engine::Request::Register(row)` as the fake service's registry
update, asserting the row equals the seeded record
(`temper_engine_domain_world::forges::record`), and drop `Register` from the panic
alternatives; no event is answered for it. Nothing else in `tests/system` changes; a
system-world failure past this one that is not the same mechanical kind is reported under
`## Left`. Then finish the checks the run left: the focused reruns, `temper-service --lib`,
the workspace check, clippy on changed packages.
