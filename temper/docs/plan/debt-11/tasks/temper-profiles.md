+++
id = "temper-profiles"
part = "temper"
kind = "migration"
needs = ["lint-check"]
files = ["temper/tests/**/Cargo.toml", "temper/tests/**/*.rs", "temper/testing/**", "temper/tools/**/Cargo.toml", "temper/tools/**/*.rs", "temper/crates/temper/Cargo.toml", "temper/crates/temper-forge-setup-shell/Cargo.toml", "temper/crates/temper-client-shell-cli/Cargo.toml", "temper/crates/temper-client-shell-terminal/Cargo.toml", "temper/crates/temper-client-shell-web/Cargo.toml", "temper/crates/temper-client-bundle/Cargo.toml", "temper/crates/temper-playground/Cargo.toml", "temper/crates/temper-fake-git/**", "temper/crates/temper-fake-git-hosted/**"]
effort = "medium"
minutes = 75
+++

# temper: every package on a profile

## Goal

Move temper's 31 packages with a block of their own onto `harness` and `binary`. Most only lose
`wildcard_enum_match_arm`, with 40 `#[expect]`s of it going; `temper-fixtures` and `temper-github-app` rise. Every
`clippy.toml` is `temper-clippy`'s.

Acceptance:

- `python3 -I -B tools/lints/lints.py temper | rg Cargo.toml` finds nothing.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for each package you changed, and
  `heavy cargo clippy --target wasm32-unknown-unknown -p temper-client-shell-web -p temper-playground -- -D warnings`.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2 and 3
- `tools/lints/design.md`, 2.2, 2.3, 3 and 5

## Owns and may change

- **Owns:** the 31 manifests of the contracts' section 2, and the code of those on `harness`, as far as
  `notes/common.md`'s list says.
- **May change:** a line of a `binary` crate's code, only for what its block asks. The two `#[expect]`s of the wildcard
  lint in `temper` and `temper-client-shell-web` stay: `binary` denies it.
- **Parallel:** `map-quiet` rewrites the `manual_map` matches in `temper/tests/engine/forge/src/translate.rs` and
  `temper/testing/temper-fixtures/src/first_run.rs`. Leave those attributes and matches to it.
- **Overlap:** `macos-cheap` is changing `temper/crates/temper/tests/support/`, `serve-without-box` and `look-round-3`
  the client crates. A manifest's block touches none of those files.
- **Options:**
  - `temper-fixtures` and `temper-github-app` gain `allow_attributes`, `unwrap_used` and `elided_lifetimes_in_paths`:
    fix by `notes/common.md`'s order. They are small.
  - `temper-playground`'s block carries comments on its lines. They go with `--set`: the design says what each lint
    is for. If one says something a reader needs of this crate, it moves to the crate's root doc.
  - A `harness` or `binary` block gains `missing_errors_doc = "allow"` and `module_name_repetitions = "allow"`: if an
    `#[expect]` of either sits in a crate, it goes.
  - If `--set` deleted a comment that spoke of something other than lints, put it back.

## Steps

1. ```sh
   python3 -I -B tools/lints/lints.py --set binary PATH...    # the contracts' section 2, temper, `binary`
   python3 -I -B tools/lints/lints.py --set harness PATH...   # the contracts' section 2, temper, `harness`
   ```
2. Clippy, then what it turns red, by `notes/common.md`. The `#[expect(clippy::wildcard_enum_match_arm, ...)]` at the
   base by crate: `temper-conformance-world` 14, `temper-engine-domain-world` 13, `temper-system-world` 5,
   `temper-engine-protocol-forge-world` 4, `temper-one-process-world` 3, `temper-engine-forge-world` 1.
3. Fmt, old forms, citations.
4. Tests: `heavy cargo nextest run -p PKG` for each package whose code changed beyond a deleted or added attribute.
   A lint block changes no behaviour.
5. Commit and push.

## Report

The report of your implementer text, and: each level you saw move, against the contracts' section 3; the exceptions
you added, by crate; the branch's net lines.
