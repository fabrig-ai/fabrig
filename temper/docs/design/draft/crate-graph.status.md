# crate-graph: status

2026-11-12. Where the design lags or departs from [crate-graph.md](crate-graph.md).

Sections 7.3 and 8 are designed in `tools/crates/design.md` (milestone G7). Nothing else of this draft is designed or built yet.

- **Design lags:**
  - 3.2, 3.3, 3.4: `cli.md`, 3.2, chooses an external crate's features once, in `[workspace.dependencies]`. It lets a member name one by a bare version, and lets the root name one twice with ungated users. It knows no context, no second entry for another platform and no crate below ours. No unification setting or check exists.
  - 4.1, 4.4: skein's `data/codec/README.md`, section 2, puts a family's schemas, golden bytes and drift test in its codec crate; the draft keeps them, and the kept texts and patches, with the owner, and moves only the generated record code. Its `data/codec/command.md`, section 1 writes every file into the owner's crate and knows no codec crate apart from it. The code departs in section 4.2's crates.
  - 5.2, 5.3: no design says where a crate is cut, or how a split lands.
  - 6.3: `AGENTS.md`, 1, puts shared test harnesses in `testing/*`, but `temper-box`'s binaries include theirs as a module each.
  - 7.1, 7.2, 7.4: `tests/boundary/` has none of these checks.
- **Design departs:**
  - 4.5, 7.3: each of the two client wires, `jig-client-protocol-wire` and `temper-client-protocol-wire`, holds a second, small family beside its own: the public wire's (`public_v1.rs`). So its own family is a shared crate's family, over 20,000 lines. The size check keeps the rule as written and excepts both, until each public family has a crate of its own (`tools/crates/design.md`, 3.1 and 5). G7's designer's reading, 2026-11-12.
  - 7.3: the 20,000-line bound on a family measures every package that holds generated record code, the owners of 4.2 among them, and not only codec crates. `jig-protocol-store`'s `people` and `tasks` are exceptions until G2, so G2's cut reads the figure from the check (`tools/crates/design.md`, 2.2 and 3.1). G7's designer's reading, 2026-11-12.
- **Open:** 11: the sizes of 7.3 and the targets of section 8, the user's. The size check is built on the proposals, 60,000 and 20,000 lines, until the user sets them (`tools/crates/design.md`, 5).
