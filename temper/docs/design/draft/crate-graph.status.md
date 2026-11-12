# crate-graph: status

2026-11-12. Where the design lags or departs from [crate-graph.md](crate-graph.md).

Sections 7.3 and 8 are designed in `tools/crates/design.md` (milestone G7), and sections 6.3 and 7.4 in its 2.4 and 3.2 (milestone G8). Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 2: skein's `ui/web/bundle.md`, 1, and jig's `protocol/browser/README.md`, 4.1, build the browser's module on stable Rust; the workspace moves to nightly (G9).
  - 3.2, 3.3, 3.4: `cli.md`, 3.2, chooses an external crate's features once, in `[workspace.dependencies]`. It lets a member name one by a bare version, and lets the root name one twice with ungated users. It knows no context, no second entry for another platform and no crate below ours. No unification setting or check exists.
  - 4.1, 4.4: skein's `data/codec/README.md`, section 2, puts a family's schemas, golden bytes and drift test in its codec crate; the draft keeps them, and the kept texts and patches, with the owner, and moves only the generated record code. Its `data/codec/command.md`, section 1 writes every file into the owner's crate and knows no codec crate apart from it. The code departs in section 4.2's crates.
  - 5.2, 5.3: no design says where a crate is cut, or how a split lands.
  - 7.1, 7.2: `tests/boundary/` has none of these checks.
- **Design departs:**
  - 4.5, 7.3: each of the two client wires, `jig-client-protocol-wire` and `temper-client-protocol-wire`, holds a second, small family beside its own: the public wire's (`public_v1.rs`). So its own family is a shared crate's family, over 20,000 lines. The size check keeps the rule as written and excepts both, until each public family has a crate of its own (`tools/crates/design.md`, 3.1 and 5). G7's designer's reading, 2026-11-12.
  - 7.3: the 20,000-line bound on a family measures every package that holds generated record code, the owners of 4.2 among them, and not only codec crates. `jig-protocol-store`'s `people` and `tasks` are exceptions until G2, so G2's cut reads the figure from the check (`tools/crates/design.md`, 2.2 and 3.1). G7's designer's reading, 2026-11-12.
  - 6.3: shared test code goes to a library its binaries depend on, and not always to a crate in `testing/`: a world's shared code that drives that world alone goes to the world's own library, as skein's layout has it for a world's binaries, so `temper-real-loop-world`'s 11 lines need no crate (`tools/crates/design.md`, 3.2). G8's designer, 2026-11-12.
  - 6.3: the binary's path reaches a kit from the running test's environment, which the test's runner sets, through `skein-scratch`'s `paths::binary`, as `temper-box`'s support already reads it; no argument carries it (`tools/crates/design.md`, 3.2). G8's designer's reading, 2026-11-12.
  - 7.4: two files that a library's own tests and its integration suite both compile, in `temper-client-shell-terminal` and `temper-engine-domain-ways`, which the draft's survey didn't name, stay the check's exceptions: a kit would depend on the library, and the library's own tests would meet two copies of its types (`tools/crates/design.md`, 3.2). G8's designer, 2026-11-12.
