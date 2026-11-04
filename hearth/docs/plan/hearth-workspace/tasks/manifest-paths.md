+++
id = "manifest-paths"
part = "layout"
kind = "feature"
needs = ["join-worlds"]
files = [
  "hearth/testing/hearth-recordings/src/lib.rs",
  "hearth/testing/hearth-test-mirror/**",
  "hearth/tests/conformance/src/fixtures.rs",
  "hearth/tests/conformance/src/fake_mmdebstrap.rs",
  "hearth/tests/conformance/tests/conformance_resolve.rs",
  "hearth/tests/mmdebstrap/src/recordings.rs",
  "hearth/tests/mmdebstrap/Cargo.toml",
  "hearth/tests/render/src/golden.rs",
  "hearth/tests/render/src/fixtures/decoders.rs",
  "hearth/tests/render/src/fixtures/decoders/golden.rs",
  "hearth/tests/render/Cargo.toml",
  "hearth/tests/config/Cargo.toml",
  "hearth/tests/template/Cargo.toml",
  "hearth/tests/builder/src/fixtures/goldens.rs",
  "hearth/tests/builder/tests/goldens.rs",
]
effort = "medium"
minutes = 30
+++

# hearth's tests find their files when they run: `env!("CARGO_MANIFEST_DIR")` goes

## Goal

`env!("CARGO_MANIFEST_DIR")` compiles a worktree's path into a test, and the build cache then shares
no test executable between worktrees or machines (`skein/docs/foundation/testing-strategy.md`, 8, and
`skein_scratch::paths`' module doc). hearth is a member now, so its 11 sites take `skein_scratch::paths`.

Acceptance:

- `rg -n 'CARGO_MANIFEST_DIR' hearth --glob '!docs/**'` finds nothing.
- `heavy cargo nextest run -p 'hearth*'` passes in `default`, and with `--profile fuzzy` in `fuzzy`,
  with the same counts as at your base.
- `heavy cargo clippy -p 'hearth*' --all-targets -- -D warnings` passes.
- `heavy cargo nextest run -p hearth-conformance-world --ignore-default-filter -E 'binary(/^recorded_/)'`
  passes: those read `testing/mirror/universe` and the recordings, and `default` leaves them out.

## Inputs

- `notes/common.md`, in this plan's directory (`hearth/docs/plan/hearth-workspace/`).
- `skein/testing/skein-scratch/src/paths.rs`: `tree()` for a file named from the tree, which is also the
  form for a library other packages' tests call, since `package()` is theirs.
- This cut wrote no design: the rule stands in those two sources.

## Owns and may change

**Owns:** the sites below. **May change, only for this:** the `Cargo.toml` of a package that gains
`skein-scratch` (`skein-scratch.workspace = true`, in `[dev-dependencies]` where only its tests read the
path, and `[dependencies]` where its library does).

Pre-approved: a site that runs outside a test, where `CARGO_MANIFEST_DIR` isn't set at run time, stays,
and the report names it. On this cut's reading none does.

## Steps

The sites, under `hearth/`, with the form each takes (`paths` is `skein_scratch::paths`):

| Site | Form |
|---|---|
| `testing/hearth-recordings/src/lib.rs`, 21 | `paths::tree().join("testing/recordings").join(tool)` |
| `testing/hearth-test-mirror/src/pin.rs`, 105 | `paths::tree().join("testing/mirror/pin")`; `skein-scratch` moves to its `[dependencies]` |
| `testing/hearth-test-mirror/src/tests/fixtures.rs`, 42 | `paths::tree().join("testing/repos").join(name)` |
| `tests/conformance/src/fixtures.rs`, 282 | `const UNIVERSE` becomes `pub fn universe() -> PathBuf`, `paths::tree().join("testing/mirror/universe")`; its 3 uses in `src/fake_mmdebstrap.rs` and `tests/conformance_resolve.rs` call it |
| `tests/mmdebstrap/src/recordings.rs`, 16 | `paths::tree().join("tests/mmdebstrap/expect")` |
| `tests/render/src/fixtures/decoders.rs`, 8 | `paths::tree()` |
| `tests/render/src/fixtures/decoders/golden.rs`, 13 | `paths::tree().join("tests/render")`; it's `#[path]`-included by the config and template worlds' tests, so both gain the dev-dependency |
| `tests/render/src/golden.rs`, 55 and 62 | `paths::tree().join("tests/render").join(name)` |
| `tests/builder/src/fixtures/goldens.rs`, 144 | `paths::tree().join("testing/repos/test")` |
| `tests/builder/tests/goldens.rs`, 191 | `paths::tree().join("tests/render/errors")` |

Unused imports go with them. Then the acceptance's commands, fmt, commit and push.

## Report

The implementer's report, and the counts at your base and after.
