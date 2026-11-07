# ui-live-cheap: the plan

The request (`inputs/request.md`) is one task, and this plan makes it one:
the live Claude Code owned-forge landing story runs on `claude-haiku-5-5`
at `low` effort again, where `model-catalogue-2`'s fixture change left it
on stock row 3, `claude-fable-5-1`, at `high`.

## Parts and tasks

- **live-haiku** (`temper/docs/design/domain/testing.md`, 2.7), slice 1.
  - `haiku-by-name` (needs none): a fresh sitting adds `claude-haiku-5-5`
    by name in the `settings apply` it already makes, at the highest free
    model number, from stock row 3's figures with `low` its default
    effort, and charters 1 and 2 name it; a focused table test of the new
    `live_settings` holds both stories' selections. The live story is
    compiled, not run.

Critical path: `haiku-by-name`, alone.

## What was decided

- **By name through ctl, so request item 2's fallback is not taken.**
  temper ctl has no verb for `ChangeModel`, the Models page's *Add a
  model by name*, but `settings apply` sets a model at a number no default
  has (jig's `domain/engine.md`, 17.1 and 17.11), and jig's core holds a
  test that adds one so. The story already makes one `settings apply` for
  its charters; the model goes in the same file, checked whole with the
  charters that name it. No stock row is added; Sonnet is not used.
- **The added model copies stock row 3,** the Anthropic provider's dearest
  stock model, as `ChangeModel` does: endpoint, window, output and prices,
  marked estimated, `source` person, on the picker, at the highest free
  number (64 on a fresh deployment). The story's spend figures therefore
  stay Fable's, as they are now.
- **It keeps the row's efforts, with `low` its default,** where
  `ChangeModel` gives an added model none. A charter's model resolves to
  its catalogue record whole, and Claude Code takes `--effort` only when
  the run's model has one, so a model with no efforts would drop
  `--effort` from the launch, against the request's item 3.
- **One focused test, new,** since no focused test of the sitting's
  selection exists to take a row: a pure table over `live_settings`, two
  rows and a refusal, no process.
- **Design:** `temper/docs/design/domain/testing.md`, 2.7 already named
  Haiku; this cut adds how the sitting takes it and its effort, and a
  **Not yet** entry in `testing.status.md` that the task removes.

## Overlaps with other drafts

- `live-change-call` (its `referee-words` merged into its integration,
  not yet on main) edits `temper/crates/temper/tests/support/forge_land.rs`
  and `temper/crates/temper/tests/suite/forge_land.rs`, the two files this
  task changes. It can't be avoided: `LiveModel` and the settings phase
  live there. The spec keeps this task's hunks off theirs (`replies_after`,
  `find_delegates`, `check_landing`, and their two test insertion points),
  so the merges should meet without conflict.
