+++
id = "skein-shells"
part = "skein"
kind = "migration"
needs = ["lint-check"]
files = ["skein/crates/skein-shell/Cargo.toml", "skein/crates/skein-ui-web/**", "skein/crates/skein-ui-bundle/**", "skein/examples/ui/shell-web/**", "skein/examples/ui/server/**", "skein/examples/echo/shell/**"]
effort = "high"
minutes = 75
+++

# skein: the shells, on `binary` and `ring`

## Goal

Move skein's six shipping crates with a block of their own onto their profiles. `skein-shell` moves in form alone.
The echo's shell and the UI example's server take its stricter block and keep `forbid` on `unsafe_code` at their
crate roots. The browser parts rise to `binary`: above all `wildcard_enum_match_arm`, the review's finding that
`skein-ui-web` has `_` arms on its own enums.

Acceptance:

- For each of the six, `python3 -I -B tools/lints/lints.py PATH | rg Cargo.toml` finds nothing.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for the six, and
  `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui-web -p skein-ui-example-shell-web -- -D warnings`.
- `rg -n '#!\[forbid\(unsafe_code\)\]' skein/examples/echo/shell/src skein/examples/ui/server/src` finds one in each
  crate's library or main.
- Their tests pass as before: `heavy cargo nextest run -p PKG`, for each package whose code changed.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2 and 3
- `tools/lints/design.md`, 2.3, 2.4, 3 and 5
- `skein/docs/foundation/programming-model.md`, 2.1 and 10.2

## Owns and may change

- **Owns:** the manifests of `skein-shell`, `skein-ui-web`, `skein-ui-bundle`, `skein-ui-example-shell-web`,
  `skein-echo-shell`, `skein-ui-example-server`, and the code of all but `skein-shell`. Not their `clippy.toml`:
  `skein-clippy`'s.
- **May change:** `skein-shell`'s code, only if its new block asks, which the contracts' section 3 doesn't foresee.
- **Options:**
  - A `_` arm on an own enum in `skein-ui-web` lists the variants, which changes no behaviour. A `_` on a foreign
    enum or a non-exhaustive one doesn't fire. Where listing would run past a dozen variants and what the arm says
    is that the others are all alike, a scoped exception with that reason.
  - `skein-ui-web`'s test modules are under `binary` too: list variants there the same way, or a scoped exception
    on the helper.
  - `skein-ui-example-server`'s `#![forbid(unsafe_code)]` is already in `src/lib.rs`. `skein-echo-shell` has a
    `src/lib.rs` and a `src/main.rs`: both get it.
  - If `--set` deleted the comments above `skein-shell`'s block, leave them gone: `tools/lints/design.md`, 2.4 says
    what they said.

## Steps

1. ```sh
   python3 -I -B tools/lints/lints.py --set ring skein/crates/skein-shell/Cargo.toml \
       skein/examples/echo/shell/Cargo.toml skein/examples/ui/server/Cargo.toml
   python3 -I -B tools/lints/lints.py --set binary skein/crates/skein-ui-web/Cargo.toml \
       skein/crates/skein-ui-bundle/Cargo.toml skein/examples/ui/shell-web/Cargo.toml
   ```
2. Clippy for the first three, the `forbid` lines. Then `skein-ui-bundle`, `skein-ui-example-shell-web`,
   `skein-ui-web`, one commit each, by `notes/common.md`'s order. They gain `allow_attributes`,
   `allow_attributes_without_reason`, `unwrap_used`, `wildcard_enum_match_arm` and `elided_lifetimes_in_paths`.
3. Fmt, old forms, the check, citations; the tests above; commit and push.

## Report

The report of your implementer text, and: for each crate, the levels that moved and the fixes they took, by lint;
each `_` arm you listed out or excepted; the branch's net lines.
