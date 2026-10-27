# Agent guidance for temper

- **main moves only through agents' `land`,** which runs the gate,
  `scripts/check.sh`, and pushes. Do not run the gate yourself before
  `land`. Markdown-only changes land with `land --docs`, except paths in
  `scripts/gated-docs`: `docs/help/` (including `reference/`) is compiled
  into temper-docs and checked by its tests, so those changes use `land`.
  Changes to the declaration itself use `land` too. Push only your own branch.
- The default test suite (unit tests and the worlds' focused tests under
  `tests/`) takes at most 30 seconds (15 seconds before the user's doubling
  of 10-22, provisional); the fuzzy suite (the worlds' `tests/fuzzy_*.rs`,
  randomized tests) at most 2 minutes (1 minute before the user's doubling
  of 10-22, provisional). Keep new tests within these budgets.
- temper is built on skein, the io and generic-protocol kit, and follows
  the foundation documents in the `docs/foundation` directory of skein's
  repository. temper's code and documents cite them by file name:
  - `programming-model.md`: how code is written; every crate follows it,
    and a departure from it or the next needs its reason, stated in the
    design or the commit;
  - `testing-strategy.md`: how that code is tested;
  - `notes.md`: the questions still open, and why the memory strategy is
    what it is.
- temper's own design documents are in `docs/design/`, whose `README.md`
  comes first and says where each part lives; `domain/testing.md` is how the
  testing strategy applies to temper. Code cites a design document by its
  path from `docs/design/`: `domain/forge.md, 8.2`, `protocol/forge.md, 3.2`.
- jig, the kit temper is built on, is its own repository
  (`~/src/rust/jig`), pinned as skein and smith are
  (`docs/development/local-jig.md`). temper cites its documents as
  jig's `<path>`, the path in jig's `docs/design/` (jig's
  `domain/connectors.md`, jig's `client/domain/client.md`), never by a
  path on disk.