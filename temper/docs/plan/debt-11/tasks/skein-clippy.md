+++
id = "skein-clippy"
part = "skein"
kind = "migration"
needs = ["skein-worlds", "skein-raised", "skein-shells"]
files = ["skein/clippy.toml", "skein/**/clippy.toml", "skein/crates/**/*.rs", "skein/testing/**/*.rs", "skein/tests/**/*.rs", "skein/examples/**/*.rs", "skein/AGENTS.md", "tools/lints/design.status.md"]
effort = "high"
minutes = 60
+++

# skein: every `clippy.toml` a variant

## Goal

skein has 27 `clippy.toml` files in 14 variants, several carrying another crate's header. Take them down to
`step`, `ordinary` and `ring`, per the contracts' section 4, and meet what that turns red.

Acceptance:

- `python3 -I -B tools/lints/lints.py skein` reads 0 findings.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for every skein package a changed `clippy.toml`
  reaches. `skein/clippy.toml` and `skein/tests/clippy.toml` reach most of the tree: run
  `heavy cargo clippy` with a `-p` for each skein package.
- `heavy cargo clippy --target wasm32-unknown-unknown` passes for each skein package in `.config/gate/wasm32.txt`.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 1 and 4
- `tools/lints/design.md`, 4
- `skein/docs/foundation/programming-model.md`, 2.1

## Owns and may change

- **Owns:** every `clippy.toml` under `skein/`.
- **May change:** skein's code, for what a variant turns red, by `notes/common.md`'s list. `skein/AGENTS.md`'s line on
  `skein/clippy.toml` stays true: leave it. `tools/lints/design.status.md`: no change, `lint-gate` closes it.
- **Readers:** `skein/testing/skein-codegen/src/patch/skeleton.rs` and its test write a comment naming "production
  clippy.toml". It stays true: leave those strings.
- **Options:**
  - `skein-world` and `skein-codegen` take `ordinary`, not `ring`, because they run files and processes, and lose
    the `Arc`, `Mutex` and `RwLock` bans. A sweep at the base finds none in either.
  - `skein-shell` gains `Mutex`, `Arc`, `RwLock` and `env::var` under `ring`. A sweep at the base finds `Mutex` twice
    and `env::var` twice, and `Command::new` 4 times and `File` once where `Command` and `File` are banned already.
    What fires takes a scoped `#[expect(clippy::disallowed_*, reason = "...")]`, in its test or its one audited place.
  - An exception a variant makes unneeded goes: skein's `#[expect(clippy::disallowed_methods, ...)]` and
    `#[expect(clippy::disallowed_types, ...)]` in crates whose variant no longer bans the item, and `clippy::panic`'s
    where `ring` now allows panics in tests.
  - `skein/tests/forge/conformance` and `skein/tests/forge/github` inherit `skein/tests/clippy.toml` once their own
    copies go. They gain `env::var`, `thread::scope` and `RandomState`: what fires takes a scoped exception. Their
    `thread::spawn` was banned already.

## Steps

1. Copy the variants by the contracts' section 4 (`cp tools/lints/clippy/NAME.toml DIR/clippy.toml`), and delete
   `skein/tests/forge/conformance/clippy.toml` and `skein/tests/forge/github/clippy.toml`.
2. Clippy, as above. Fix by `notes/common.md`'s order. Cargo tracks a `clippy.toml` a crate read; if a result looks
   stale, `heavy cargo clean -p PKG` in your slot.
3. Fmt, old forms, the check, citations.
4. Tests: `heavy cargo nextest run -p PKG` for each package whose code changed beyond a deleted or added attribute.
5. Commit and push.

## Report

The report of your implementer text, and: each ban you saw move, by crate, against the contracts' section 4; the
exceptions you added, each with its reason; the branch's net lines.
