+++
id = "jig-profiles"
part = "jig"
kind = "migration"
needs = ["lint-check"]
files = ["jig/clippy.toml", "jig/tests/**", "jig/testing/**", "jig/crates/jig-client-shell-native/**", "jig/crates/jig-client-shell-web/**"]
effort = "medium"
minutes = 60
+++

# jig: every package on a profile, every `clippy.toml` a variant

## Goal

Move jig's 41 packages with a block of their own or none onto `harness` and `binary`, and its 22 `clippy.toml`
files onto their variants. jig's item stays until `lint-gate` deletes it, with the others'.

Acceptance:

- `python3 -I -B tools/lints/lints.py jig` reads 0 findings.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for every jig package you changed, and for
  `--target wasm32-unknown-unknown -p jig-client-shell-web`.
- `rg -n '^disallowed_\w+ = "(allow|warn)"' -g Cargo.toml jig` finds nothing.
- Each jig `clippy.toml` but `jig/clippy.toml` and `jig/tests/clippy.toml` states why it differs: the variant's
  first lines do.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2, 3 and 4
- `tools/lints/design.md`, 3, 4 and 5
- `jig/docs/development/debt/D019-lint-config-copied.md`

## Owns and may change

- **Owns:** the 41 manifests of the contracts' section 2 and every `clippy.toml` under `jig/`.
- **May change:** code in those packages, as far as `notes/common.md`'s list says.
- **Readers:** `jig/tests/boundary/src/tests.rs` asserts that `jig-test-store-files`'s manifest holds
  `[lints.clippy]` and that its `clippy.toml` exists. Both stay true: run that test, don't change it.
- **Options:**
  - `jig-test-profile` has no block at the base, so all of `harness` is new to it (pedantic, `unsafe_code`,
    `unwrap_used`). Fix what is mechanical; the rest is a scoped exception.
  - `jig/tests/client-protocol` loses its `clippy.toml` and its `Cell` ban: its `#[expect]`s of that ban go,
    and the reasons with them.
  - `std::env::var` in `jig-test`'s `tests/support/mod.rs` and `jig/tests/core/tests/suite/fuzzy/settings.rs`
    reads a seed or a profile for the test run: each takes a scoped
    `#[expect(clippy::disallowed_methods, reason = "...")]`.
  - If `--set` deleted a comment that spoke of something other than lints, put it back.

## Steps

1. ```sh
   python3 -I -B tools/lints/lints.py --set binary jig/crates/jig-client-shell-native/Cargo.toml \
       jig/crates/jig-client-shell-web/Cargo.toml
   python3 -I -B tools/lints/lints.py --set harness PATH...   # the contracts' section 2, jig, `harness`
   ```
2. Copy the variants by the contracts' section 4, and delete `jig/tests/client-protocol/clippy.toml`.
3. Clippy, then what it turns red, by `notes/common.md`: 10 `#[expect(clippy::wildcard_enum_match_arm, ...)]`
   sit in harness crates at the base, and go. Those in step crates stay: they are still fulfilled. Cargo tracks
   a `clippy.toml` a crate read; if a result looks stale, `heavy cargo clean -p PKG` in your slot.
4. Fmt, old forms, the check, citations.
5. Tests: `heavy cargo nextest run -p jig-boundary`, and `-p PKG` for each package whose code changed beyond a
   deleted or added attribute. A lint block changes no behaviour.
6. Commit and push.

## Report

The report of your implementer text, and: each level or ban you saw move, against the contracts' sections 3 and 4;
the exceptions you added, by crate; the branch's net lines.
