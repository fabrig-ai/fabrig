# Agent guidance for smith

- Land on main with `land`: it rebases on `origin/main`, runs
  `scripts/check.sh`, and pushes. A
  Markdown-only change lands with `land --docs`, except paths in
  `scripts/gated-docs`: the design, benchmark README and benchmark fixture
  trees are read by tests, so those changes use `land`.
  Changes to the declaration itself use `land` too.
  Rust doc comments do not skip the checks.
- The focused suite takes at most 30 seconds (15 seconds before the user's
  doubling of 10-22, provisional) and the fuzzy suite at most 120 seconds
  (60 seconds before the user's doubling of 10-22, provisional). Measure
  worlds on an idle machine; keep
  additions within those budgets.
- smith follows skein's `docs/foundation/programming-model.md`,
  `testing-strategy.md` and `notes.md`. Read the first two's section 1
  before writing code: they are rules, and a departure needs its reason, stated in
  the design or the commit. Source citations use those file names and their sections.
- smith's contracts are in `docs/design/domain/`. Code cites them as
  `domain/<file>.md` with a specific section, at module or type level,
  never stamped on every field. Module docs say what state is kept, what
  is never known, and the entry points and contracts. Each public type
  and variant has one line of prose: what it is, who sends it and, where
  it has one, its terminal. A field is documented only when its name and
  type do not say it: units, bounds, invariants.
- Production domains use the strict Rust subset: exhaustive own-enum
  matches, bounded skein containers and checked arithmetic. Review the
  rules the compiler cannot check, including no own-enum if-let or
  let-else, tuple scrutinees, closures, `loop`, `while`, application traits
  or generics. Test harnesses are ordinary Rust with deterministic clocks
  and seeds, following programming-model.md, section 10.2.
- Re-export public names explicitly, use descriptive parameter names,
  separate items with blank lines, and keep the standard `Result` name
  for fallible operations.
- Reuse skein's generic test kit. Do not make smith a second owner of
  generic schedules, replay, referee plumbing or counting allocators.
  A missing shared mechanism belongs in skein before its smith consumer.
- Work in a branch of your own worktree, never in the primary checkout.
  Push only your own branch: `land` is the only way to `main`.