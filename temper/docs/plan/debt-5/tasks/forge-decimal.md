+++
id = "forge-decimal"
part = "skein-forge-root"
kind = "migration"
needs = ["forge-neutral-modules"]
files = ["skein/crates/skein-forge/src/github/**"]
effort = "medium"
minutes = 15
+++

# skein D010: GitHub spells its integers with `bytes::decimal`

## Goal

GitHub's 20 sites of `forgejo::id::unsigned` call `skein_lib::bytes::decimal`, which is what it wraps, and which GitHub already calls at `redirect.rs:108` and `plans/reads/checks.rs:165,168`. The measure reaches 0. D010 stays open.

Acceptance:

- The measure in `temper/docs/plan/debt-5/notes/forge-root.md` prints 0.
- `forgejo::id::unsigned` still exists: Forgejo's readers and the fakes call it.
- What the note calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.

## Owns and may change

- Owns: the Rust files of `skein-forge/src/github`, at the 20 sites and their imports.
- The sites: `decode.rs` 450, 531, 538, 543, 546; one each in `header.rs`, `server.rs`, `server/{app,queries}.rs`, `decode/{log,delivery,reviews,walk,app,checks,files,issues,pull_graphql,pulls,protection}.rs`.
- The local wrappers at `decode/delivery.rs:266` and `decode/app.rs:413` may stay, calling `bytes::decimal`.

## Steps

1. Replace each site with `bytes::decimal` (or a `use skein_lib::bytes;` where the file has none).
2. Run the measure and `What runs`; commit (`D010: GitHub spells its integers with bytes::decimal`, saying why) and push.

## Report

The measure, and any non-test line under `src/github` that still names `forgejo`.
