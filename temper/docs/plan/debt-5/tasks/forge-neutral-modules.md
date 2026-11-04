+++
id = "forge-neutral-modules"
part = "skein-forge-root"
kind = "feature"
needs = ["forge-limits"]
files = [
  "skein/crates/skein-forge/src/lib.rs",
  "skein/crates/skein-forge/src/time.rs",
  "skein/crates/skein-forge/src/path.rs",
  "skein/crates/skein-forge/src/text.rs",
  "skein/crates/skein-forge/src/decode.rs",
  "skein/crates/skein-forge/src/forgejo.rs",
  "skein/crates/skein-forge/src/forgejo/**",
  "skein/crates/skein-forge/src/github/**",
]
effort = "medium"
minutes = 35
+++

# skein D010: `time`, `path`, `Body` and `Malformed` move to skein-forge's root

## Goal

The four neutral pieces sit at the root, GitHub names them there, and Forgejo keeps its public paths by re-export. The measure falls from 44 to 20, what is left being `id::unsigned`, the next task's. D010 stays open.

Acceptance:

- The note's measure prints 20, and every line left names `id::unsigned`.
- `rg -n 'mod (time|path)\b' skein/crates/skein-forge/src/forgejo.rs` shows re-exports only, and `forgejo/time.rs` and `forgejo/path.rs` are gone, by `git mv`.
- `skein_forge::forgejo::{time, path}`, `forgejo::document::Body` and `forgejo::decode::Malformed` still name the same items.
- `rg -n 'forgejo::' skein/crates/skein-forge/src/github/tests skein/crates/skein-forge/src/github/tests.rs` names no moved item.
- What `temper/docs/plan/debt-5/notes/forge-root.md` calls `What runs` passes; `heavy cargo nextest run -p github-conformance -E 'test(=github_conformance_names_every_fact_of_forge_github_13)'` passes.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 4.1 and 4.4.

## Owns and may change

- Owns: skein-forge's `src/lib.rs`, `src/forgejo.rs`, `src/forgejo/**`, `src/github/**`, and the new root files.
- `src/github/tests/{reads,items,delivery,fixtures}.rs`: repoint the 12 `forgejo::` lines naming moved items, that path only.
- Forgejo's tests of time and path stay where they are and compile through the re-exports. Moving them is not this task's.
- GitHub's re-measuring of percent-encoding at `request.rs:445-451` may use `segment_length`, if that is a like-for-like swap. Otherwise leave it.

## Steps

1. `git mv src/forgejo/time.rs src/time.rs`; its one tie, `super::id::unsigned`, becomes `skein_lib::bytes::decimal`. `git mv src/forgejo/path.rs src/path.rs`; it takes `crate::limits::Limits`.
2. `src/decode.rs` holds `Malformed` with its doc and derives. `src/text.rs` holds `Body` and `impl Body`, naming `crate::decode::Malformed`.
3. `pub mod decode; pub mod path; pub mod text; pub mod time;` in `src/lib.rs`, each with the module doc the note asks for.
4. Forgejo: `pub use crate::time;` and `pub use crate::path;` in `src/forgejo.rs`; `pub use crate::decode::Malformed;` in `forgejo/decode.rs`; `pub use crate::text::Body;` in `forgejo/document.rs`. Forgejo's own code may keep its short paths through them.
5. GitHub: the 33 sites of `time`, `path`, `document::Body` and `decode::Malformed` name `crate::time`, `crate::path`, `crate::text::Body` and `crate::decode::Malformed`.
6. Run the measure and `What runs`; commit (`D010: time, path, Body and Malformed sit at skein-forge's root`, saying why) and push.

## Report

The measure, the lines added and removed, and any item that held a Forgejo word.
