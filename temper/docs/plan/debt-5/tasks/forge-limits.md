+++
id = "forge-limits"
part = "skein-forge-root"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-forge/src/limits.rs",
  "skein/crates/skein-forge/src/lib.rs",
  "skein/crates/skein-forge/src/forgejo/limits.rs",
  "skein/crates/skein-forge/src/github/limits.rs",
]
effort = "medium"
minutes = 25
+++

# skein D010: one `Limits` and one check at skein-forge's root

## Goal

`skein_forge::limits` holds `Limits`, `Problem` and `check(limits, depth)`. Each dialect keeps its public names and its depth. D010 stays open: `forge-root-close` deletes it.

Acceptance:

- `skein-forge/src/forgejo/limits.rs` and `src/github/limits.rs` define no struct or enum; each holds its re-export, its depth and a one-argument `check`.
- `rg -n 'TokenizerString' skein --type rust` reads nothing.
- What `temper/docs/plan/debt-5/notes/forge-root.md` calls `What runs` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 4.4; `skein/docs/design/forge-github.md`, 2.
- `skein/docs/development/debt/D010-github-on-forgejo-modules.md`, whole.

## Owns and may change

- Owns: the files in the header.
- Reader: `skein-forge/tests/suite/forgejo_bounds.rs:48`: `Problem::TokenizerString` becomes `Problem::String`, that one word.
- `github/tests.rs`'s readers of `Problem::String` do not change.
- The dialects' one-argument `check` must stay: `skein-fake-forge-protocol/src/server.rs:76` and `service.rs:225` call it, and the fakes are not this plan's.

## Steps

1. `src/limits.rs`:

   ```rust
   //! Bounds both dialects share (forge.md, 4.4).
   use skein_json::{tokenizer, writer};
   /// Bounds on tokens, documents, retained fields and targets.
   pub struct Limits { /* the six fields */ }
   /// A limit that cannot hold a dialect's documents.
   pub enum Problem { Zero, String, Depth }
   /// Checks that both sides can honour the limits at a dialect's JSON depth.
   pub fn check(limits: &Limits, depth: u32) -> Result<(), Problem>
   ```

   The body is `github/limits.rs`'s, with `depth` for `DEPTH`. Derives as they are.

2. Each dialect:

   ```rust
   pub use crate::limits::{Limits, Problem};
   /// (what this depth holds)
   pub const DEPTH: u32 = 8; // 10 for GitHub, with its doc as it stands
   pub fn check(limits: &Limits) -> Result<(), Problem> { crate::limits::check(limits, DEPTH) }
   ```

   `forgejo/limits.rs` gains `DEPTH` only so that the two read alike; a test that names a literal 8 stays.

3. `pub mod limits;` in `src/lib.rs`. Run `What runs`; commit (`D010: one Limits and one check at skein-forge's root`, saying why) and push.

## Report

The lines added and removed, and any reader the survey missed.
