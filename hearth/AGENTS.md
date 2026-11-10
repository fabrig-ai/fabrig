# hearth

Immutable machines, built from a revision: the daemon that renders, builds and runs a machine's outputs. Start from `hearth/docs/design/README.md`: its section 2 is the reading order, its section 3 the milestones. `hearth/docs/design/STATUS.md` says how far the code is from the design.

- **hearth builds on skein alone:** it depends on no crate of smith, jig or temper, and none of theirs depends on hearth.
- **Every crate is step code,** except the fakes and test kits under `testing/`, the worlds under `tests/` and the binary's shell, which are ordinary Rust and take a lint profile (`tools/lints/design.md`, 3).
- **Step code depends on skein's crates and hearth's own, and nothing else:** no serde and no template engine; hearth parses its own templates (`hearth/docs/design/domain/render/README.md, 5.3`). MiniJinja 2 is a dev-dependency only, pinned to an exact version, as the template world's oracle (`hearth/docs/design/domain/render/README.md, 6.5`).
- **Conformance, real builds and the VM tier** are outside the gate (`hearth/docs/design/testing/README.md, 2.8`).
