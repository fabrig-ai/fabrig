# crate-graph: status

2026-11-12. Where the design lags or departs from [crate-graph.md](crate-graph.md).

Sections 7.3 and 8 are designed in `tools/crates/design.md` (milestone G7). Sections 4.1, 4.3, 4.4 and 7.2 are designed in skein's `data/codec/README.md`, sections 2 and 6, and `data/codec/command.md`, section 1; jig's `protocol/README.md`, section 5; temper's `protocol/store.md`, section 4; and `tools/crates/design.md`, 3.2 (milestone G2), and section 4.2's other crates are that check's exceptions until G3. Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 2: skein's `ui/web/bundle.md`, 1, and jig's `protocol/browser/README.md`, 4.1, build the browser's module on stable Rust; the workspace moves to nightly (G9).
  - 3.2, 3.3, 3.4: `cli.md`, 3.2, chooses an external crate's features once, in `[workspace.dependencies]`. It lets a member name one by a bare version, and lets the root name one twice with ungated users. It knows no context, no second entry for another platform and no crate below ours. No unification setting or check exists.
  - 5.2, 5.3: no design says where a crate is cut, or how a split lands.
  - 6.3: `AGENTS.md`, 1, puts shared test harnesses in `testing/*`, but `temper-box`'s binaries include theirs as a module each.
  - 7.1, 7.4: `tests/boundary/` has neither of these checks.
- **Design departs:**
  - 4.5, 7.3: each of the two client wires, `jig-client-protocol-wire` and `temper-client-protocol-wire`, holds a second, small family beside its own: the public wire's (`public_v1.rs`). So its own family is a shared crate's family, over 20,000 lines. The size check keeps the rule as written and excepts both, until each public family has a crate of its own (`tools/crates/design.md`, 3.1 and 5). G7's designer's reading, 2026-11-12.
  - 7.3: the 20,000-line bound on a family measures every package that holds generated record code, the owners of 4.2 among them, and not only codec crates (`tools/crates/design.md`, 2.2 and 3.1). G7's designer's reading, 2026-11-12.
  - 4.4: the codec command that learns to write a family's generated code into its codec crate is the owner's drift test. skein-codegen's `patch` command keeps the owner's schemas, kept texts and patches and never wrote generated code; each owner's drift test already names each family's paths and writes its code when asked, and now names its codec crate there (skein's `data/codec/command.md`, section 1, and `data/codec/README.md`, section 6). skein-codegen doesn't change. G2's designer's reading, 2026-11-12.
  - 4.4: `temper-protocol-store`'s older versions' generated code, `v12.rs`, `v13.rs` and `v14.rs`, is dropped rather than moved. skein's `data/codec/README.md`, section 5, keeps one version's code, an older record being read as a generic value and patched. Of the three, nothing compiled `v13.rs`, no test used `v12`, and one test used `v14` to read a patched record, which it now reads otherwise. Moved, `v14`'s 50,000 lines would compile in every build of the codec crate (temper's `protocol/store.md`, section 4). G2's designer's reading, 2026-11-12.
  - 4.4: callers keep the owners' paths. Each store re-exports its codec crates' modules at its root under their names, with no end set, and a caller may name either path. Every caller outside the two stores depends on its store for other items too (its `codec` module, `sizes`, `keys`), so naming the codec crate would save it no build (jig's `protocol/README.md`, section 5; temper's `protocol/store.md`, section 4). G2's designer's reading, 2026-11-12.
