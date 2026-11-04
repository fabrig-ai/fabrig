# D003: Memory tests hold peaks to formulas, not to ceilings

- **Kind:** tests
- **Where:** `tests/render/tests/suite/memory.rs`, `tests/render/src/parent.rs`, `tests/tree/tests/suite/memory.rs`, `tests/template/tests/suite/memory.rs`, `tests/config/tests/suite/memory.rs`, `tests/accounts/tests/suite/memory.rs`, `tests/mmdebstrap/tests/suite/memory.rs`, `tests/git/tests/suite/memory.rs`
- **Seen:** 2026-11-04 at `66968a30ef`, in `hearth/docs/design/STATUS.md` (Departs, `domain/testing.md`, 8.4)
- **Seen:** 2026-11-04 at `66968a30ef`, in the report of plan hearth-workspace's join-worlds task (the builder's and git's checks moved to measured ceilings; the rest stayed)
- **Seen:** 2026-10-18 at `56096145d0`, in render's retrospective (its list of debts, item 7: the at-the-limit memory tests are slow)
- **Checked:** 2026-11-04 at `66968a30ef`
- **Value:** medium (tests stop moving with a type's size; one way to read a memory test)
- **Size:** M
- **Status:** open
- **Paid when:** every memory test of hearth's worlds holds a fixed scenario under a ceiling named beside it, none calls a `worst_case`, and `parent::checked` takes no `Memory`.
- **Measure:** `git grep -E 'fn \w*worst_case\w*' -- 'hearth/tests/*.rs' 'hearth/crates/*/src/tests/*.rs' | wc -l`: 31

## What

`skein/docs/foundation/testing-strategy.md`, 6, wants each fixed scenario's
peak under a ceiling: an upper bound with headroom, moved by hand, never by
a type's size. In hearth only the builder world's four scenarios and git's
step peak do. The other worlds still compare a peak to a formula at limits:

- 30 tests are named `..._no_more_than_...`, most ending in `the_worst_case`;
- `tests/render/tests/suite/memory.rs` passes
  `compose::worst_case(..)` and the like as the bound;
- `hearth_render_world::parent::checked` takes a `skein_world::Memory`
  it can only assert to be `Checked`, a flag left beside the `Meter` that
  does the check.

The port showed why this bites: lazy growth changed step peaks, and git's
formula failed although nothing was wrong, and it now holds a ceiling.

## Why it matters

Formulas priced for an edge now also carry the tests' weight, which keeps
them from being retired (D002). The at-limit scenarios are also the slow
ones: fixed small scenarios with ceilings would be cheaper.

## What pays it back

World by world, measure each scenario's peak, name a ceiling beside the
test by the rule of the testing strategy, and show it bites by lowering it
once. Drop the `memory` argument of `parent::checked`. Formulas that remain
at the edges keep a test of the edge's value. Goes with D002.
