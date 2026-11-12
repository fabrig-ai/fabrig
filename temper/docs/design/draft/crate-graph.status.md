# crate-graph: status

2026-11-12. Where the design lags or departs from [crate-graph.md](crate-graph.md).

Nothing of this draft is designed or built yet.

- **Design lags:**
  - 3.2, 3.3, 3.4: `cli.md`, 3.2, chooses an external crate's features once, in `[workspace.dependencies]`. It lets a member name one by a bare version, and lets the root name one twice with ungated users. It knows no context, no second entry for another platform and no crate below ours. No unification setting or check exists.
  - 4.1, 4.4: skein's `data/codec/README.md`, section 2, puts a family's schemas, golden bytes and drift test in its codec crate; the draft keeps them, and the kept texts and patches, with the owner, and moves only the generated record code. Its `data/codec/command.md`, section 1 writes every file into the owner's crate and knows no codec crate apart from it. The code departs in section 4.2's crates.
  - 5.2, 5.3: no design says where a crate is cut, or how a split lands.
  - 6.3: `AGENTS.md`, 1, puts shared test harnesses in `testing/*`, but `temper-box`'s binaries include theirs as a module each.
  - 7.1, 7.2, 7.3, 7.4: `tests/boundary/` has none of the checks.
  - 8: no tool reads a build's timings or the landings' chains.
- **Design departs:** none yet.
- **Open:** 11: the sizes of 7.3 and the targets of section 8, the user's.
