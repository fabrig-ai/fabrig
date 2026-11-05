# debt-21: hearth's worlds stop being kits (hearth D009)

Binding on the `hearth-*` tasks. `hearth/docs/design/domain/testing.md`, 2.9 holds the layout. Line numbers are at the plan's base.

## The rule of each move

- **Moved code moves:** `git mv` where a file moves whole, and no assertion, event or behaviour changes. The only edits beyond paths are those this note names; anything further is a question.
- **A world may use a kit. Nothing may depend on a world.** The world keeps `pub use` lines at its old paths, so its own suites change nothing.
- **Files stay where tree-relative reads find them.** No golden or fixture file moves: `golden::load` resolves `tests/render/<name>` through `paths::tree()` at run time.
- **Off limits:** `hearth/crates/hearth-service-builder/src/lib.rs` and the machines' `MAX_OUT` forms. Nothing moved holds a `MAX_OUT`. Its neighbours stay put (`hearth/tests/builder/src/protocol.rs`, `tests/suite/bounds.rs`, `hearth/tests/config/src/world.rs`, `hearth/tests/tree/src/helper.rs`).
- **No kit module is called `fixtures`:** that glob would need an entry in `.config/fixture-readers.toml`.
- **Lints:** a kit copies a sibling's manifest, takes `python3 -I -B tools/lints/lints.py --set harness <its Cargo.toml>` and copies `tools/lints/clippy/ordinary.toml` as its `clippy.toml`. A kit's dependencies are the ones below: if a move needs another, add its line and say so.
- **`hearth-test-fixtures` takes nothing needing `hearth-lib`, `hearth-domain-render` or `hearth-protocol-tree`:** they dev-depend on it, which would make a cycle.

## What goes where

| Edge at base | Kit | Takes | Its dependencies | The world keeps |
|---|---|---|---|---|
| `hearth-render-world` on `hearth-config-world` | `hearth-test-fixtures`, new `src/hostile.rs` | `hearth/tests/config/src/generate.rs`'s `mutate`, `HOSTILE` | unchanged | `generate` without them |
| `hearth-builder-world` on `hearth-render-world` | `hearth-test-render` | render world's `golden.rs`, `golden/repository.rs`, `json.rs`, `repositories.rs`, `parent.rs`, `fixtures/properties.rs` (as `properties.rs`), `fixtures.rs:8-18` (`Repository`, `oid`), and `fixtures/goldens.rs:26-31`'s `JSON_*` | `hearth-domain-render`, `hearth-lib`, `hearth-protocol-builder`, `hearth-protocol-config`, `hearth-protocol-template`, `hearth-test-fixtures`, `skein-lib`, `skein-scratch`, `skein-world` | `pub use` at the old paths; the rest |
| `hearth` on `hearth-builder-world` | `hearth-test-repository`, a flat root with modules `reserved`, `binary`, `resolution` | builder world's `fixtures.rs` lines 18-34, 35-136, 157, 192-202, 232-253, 419-473, 746-751; `fixtures/binary.rs`; `fixtures/resolution.rs` lines 5-80, 92-100, 229-237; from `fixtures/reserved.rs`: `EMPTY_HEARTH`, `FLEET_JSON`, `PROD_ENV`, `LAB_ENV`, `METAL_EFI`, `git_path`, `MOTD_KIND`, `MOTD_DIFF_TEXT` | `hearth-protocol-tree`, `hearth-lib`, `skein-world`, `skein-shell`, `skein-io`, `skein-scratch` | `pub use`; all code that names the `hearth` lib (`render`, `Holding`, `Scenario`) |
| `hearth-conformance-world` on `hearth-git-world` | `hearth-test-git`, modules `expect`, `replay`, `hunks`, `world` | git world's `expect.rs`, `replay.rs`, `hunks.rs`, `world.rs:25-206` and `:384-477` | `hearth-protocol-git`, `hearth-recordings`, `skein-lib`, `skein-world` | `Stepped`, `Action` (`world.rs:207-383`), `tail` (`:11-24`), `pub use hearth_test_git::world::*` |

## Why render's parent moves with the goldens

`properties::run`, the builder's digest oracle, runs the scripted parent. A kit without it would change the test's oracle.

## The last search

The item's **Measure** catches all four edges. This one is more complete, and also lists exactly those four at base:

```sh
git grep -nE '^\s*(\[[a-z.-]*dependencies\.)?hearth-[a-z-]+-world\b|package *= *"hearth-[a-z-]+-world"|path *= *"[^"]*(tests/|\.\./(accounts|builder|config|conformance|git|mmdebstrap|render|template|tree)")' -- 'hearth/*Cargo.toml'
```

Data edges stay, and are no dependency: `hearth/tests/builder/tests/suite/goldens.rs` reads `../../../render/…`, and `hearth-test-fixtures`' corpus reads the goldens.
