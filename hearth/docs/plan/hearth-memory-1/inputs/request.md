# Request: hearth to the dynamic memory model, round 1: no worst case, tests on ceilings, 30 s

From fabrig/hearth, 2026-11-04, on the user's decisions. Every other tree has moved to the model of
`skein/docs/foundation/programming-model.md`, 6.3 and 7.1; hearth is the only one left, with `max_out` 85 and `worst_case` 132
in `hearth/scripts/old-forms.txt`. The user wants it migrated, **low priority and slowly:** one small plan at a time. This
is the first of several. Keep it small, and leave what the later rounds take. Paying `hearth/docs/development/debt/` items
D002 and D003 is this round; D001 (`MAX_OUT`, room checks, capped queues) is the rounds after it. **Don't touch D001's forms.**

What hearth does for a user doesn't change: same specs, same digests, same refusals at the edges.

## 1. D002: no process-wide worst case

`hearth/docs/development/debt/D002-process-worst-case.md`, read whole, and its **Paid when** holds.

- `hearth_service_builder::worst_case` and its tests go, and each sum that fed only it.
- Every formula left is named for what it bounds, `_bound` or `_price` (6.3), and `valid(limits)` stays as startup's check.
- Design first: `domain/render.md`, 8.4 stops saying the builder's worst case is a sum that the defaults keep within 1 GiB, and
  `domain/steward.md`, 5.3's fit check against a share of a ceiling goes the same way; that check isn't built. The user has seen
  and agreed this. Keep section numbers; rename 8.4's heading if it's now wrong. Change each status file with its document.
  `programming-model.md` is skein's, and `skein/docs/foundation/notes.md`'s line that hearth may keep the static model is
  no longer needed: remove it, and only it, from skein.

## 2. D003: memory tests hold fixed scenarios under ceilings

`hearth/docs/development/debt/D003-memory-tests-on-formulas.md`, read whole, and its **Paid when** holds. By
`skein/docs/foundation/testing-strategy.md`, 6: world by world, measure each scenario's peak, name a ceiling beside the test (an
upper bound with headroom, moved by hand, never by a type's size), and show that it bites by lowering it once and seeing the
test fail. The commit gives the peak. The builder world and git's step already work this way: follow them. `parent::checked`
drops its `Memory` argument. A formula left at an edge keeps a test of the edge's value.

This replaces the at-the-limit tests of about 30 `..._no_more_than_...`. That's a change to what they prove, and the user has
agreed it. Elsewhere, no assertion or case goes.

## 3. A large render, under its own ceiling

The 1 GiB figure stood for one real guarantee: the defaults fit a modest machine. The user wants that kept as a measurement. Add a
fixed scenario for a realistic large render through the builder, in hearth's render or builder world, with its peak under a named ceiling
set from what it measures. Say in the commit what "large" means here: entries, documents and targets. `domain/render.md`, 8.4 and
`domain/testing.md` name it. A test about the small-VM claim belongs here, not a new formula.

## 4. The focused budget

The user's ruling: hearth's focused suite stays at 30 CPU s. It measures about 35 to 36 (gate records, 11-04), every other test kept. The
fixed scenarios above should bring most of that; the slowest non-memory tests are `frames::no_cut_of_a_generated_frame_reads_as_a_record` (2.5 s)
and the builder's domain tests (about 0.9 s each). Keep every other assertion and case. Measure as the gate does:
`tools/tiers/cpu.py`, with `NEXTEST_DOUBLE_SPAWN=0`. Add to `tools/tiers/budgets.toml`:

```toml
[gate.hearth.default]
cpu_suite_s = 30
```

If the suite can't reach 30 this round, say what's left and by how much, and leave the figure in anyway: going over is recorded,
not rejected. Don't raise it.

## Rules

- Delete `D002-...md` and `D003-...md` as they're paid. `hearth/scripts/old-forms.txt` lowers with `--write`.
- A test that fails after a ceiling or formula change is a finding first: don't move a ceiling to pass it unless a measurement says the real
  peak moved, and say why it did.
- A skein change hearth seems to need is a question, not a task. skein changes only by the one line section 1 names.
- `tools/citations/citations.py --all` stays at 0, and `tests/boundary/tests/outward.rs` passes.
- All parts must be in the plan when the cut ends.
