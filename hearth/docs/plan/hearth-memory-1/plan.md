# hearth-memory-1: the plan

Round 1 of hearth's move to the dynamic memory model: it pays D002 and D003, adds the large render and holds the focused budget. D001's forms
are later rounds. The design this plan builds on is `hearth/docs/design/domain/render.md`, 8.4 and `hearth/docs/design/domain/testing.md`, 8.4
and 9.1, which this cut wrote.

## Parts and tasks

| Part | Tasks | Needs |
|---|---|---|
| `sums` | `sums` | nothing |
| `large` | `large-render` | nothing |
| `ceilings` | `git`, `accounts`, `mmdebstrap`, `config`, `template`, `tree`, `render` | `sums` |
| `startup` | `valid` | every `ceilings` task |
| `budget` | `gate-table`; `focused-budget` | nothing; `gate-table`, `large-render`, `valid` |
| `close` | `close` | `focused-budget` |

The critical path is `sums`, `render` (or `tree`), `valid`, `focused-budget`, `close`: about 240 implementer-minutes.

## What was decided

- **`sums` comes first, and is small.** It deletes the top of the sum, so that each family then renames and deletes in its own crate alone, and the
  seven families run side by side, one writer per crate.
- **A world and its crate are one task.** Each family's world stops reading a formula in the same commit that renames or removes it.
- **Which formulas stay.** No production code reads a `worst_case` today. A formula stays if it sizes an edge, as 6.3 names them (a decoder's
  retained bound, a decoded record), or if a test that isn't a peak test reads it. Render's and the store's and the build's other sums go.
  `notes/contracts.md`, 1 lists every one.
- **`valid(limits)` is new.** The request says it stays, but hearth has none: startup asserts only `limits.io.is_usable() && limits.queue >= 3`. The
  end state the request and 6.3 name has one, so `valid` adds it, reading what's left of the formulas. `Service::new` asserts it. That assertion
  stays beside it, as a D001 form.
- **`[gate.hearth.default]` needs its reader.** `budgets()` in `tools/tiers/cpu.py` refuses a top-level `gate`, so the table alone would have broken
  `tools/tiers/run --verify-all` and every `--cpu` run. `gate-table` teaches it the table, and `tools/tiers/design.md` says so.
- **steward's fit check is render's sentence.** `hearth/docs/design/domain/steward.md`, 5.3 holds no fit check: it was `render.md`, 8.4's sentence citing
  it. That sentence is gone, and `steward.md` and its status don't change.
- **skein's line is the whole bullet.** `skein/docs/foundation/notes.md`'s "Narrow services may keep their caps" is one bullet, and all of it goes.
- **"Large" is 4,096 entries, 512 of them documents, declarations and templates, and 64 targets,** a quarter of the shipped listing limit and a
  sixteenth of its targets, at the shipped limits. `large-render` may halve entries and documents if it can't reach 1.5 CPU s, and `close` then
  carries the figures into the design.
- **`hearth-domain-tree` and `hearth-protocol-accounts` aren't in the builder's `Limits`,** so `valid` doesn't read them. Their formulas stay, with
  the tests they already have.
- **The `max_out` count falls by what `sums` deletes,** one line that read `MAX_OUT` inside `hearth_domain_build::limits::worst_case`. No D001 form
  is otherwise touched.
- **The budget's measurement has no tool.** `cpu.py` is a caller's library, so `focused-budget` drives it from throwaway code, as `AGENTS.md`, 4 allows.

## Overlaps

No other draft is changing these files. `tools/tiers/` and `skein/docs/foundation/notes.md` were the two outside hearth, and neither is in the list
of drafts beside this one.
