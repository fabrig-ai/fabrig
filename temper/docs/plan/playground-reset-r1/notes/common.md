# Standing rules for every task of playground-reset-r1

Every task of this plan names this file. Where a spec departs from it, the
spec says so and wins.

## The source

- The draft is `temper/docs/design/draft/playground-reset.md`. It is not
  adopted: the designs it amends (its section 10) still say the old
  behaviour, and no task edits any `docs/design/` file. Where the draft and
  an adopted design disagree, the draft wins for what the draft says, and
  the adopted design for everything else. What the code taught goes in your
  report, not into a design.
- `temper/docs/plan/playground-reset-r1/status.md` says where the code
  stands; `plan.md` beside it says how the work is cut and the decisions
  the lead made. Read the decisions that your spec cites.
- `docs/help/` is the lead's, never an implementer's. A help test that
  breaks because a page names something your task removes is reported,
  with the page and line, and left failing only if the spec says so;
  otherwise stop with a `Question:`.

## The code's rules

- skein's `docs/foundation/programming-model.md` and `testing-strategy.md`,
  section 1 of each, hold everywhere; `AGENTS.md` sections 2 and 6.
- Lint exceptions are scoped `#[expect(..., reason = "...")]`.
- Comments say what is true now, citing the draft by path and section
  (`temper/docs/design/draft/playground-reset.md, 4`) where the behaviour
  is the draft's.
- No new dependency between trees against the one-way rule: skein names
  none of smith, jig, temper; jig never names temper.

## Stored and wire formats

- No release has happened: a stored family or a wire record changes in
  place under a new schema version, as the store's and wire's designs say
  for changes before the first release (temper's `protocol/store.md`,
  section 4; jig's `protocol/store.md`). Regenerate generated code and its
  goldens with the generator the crate already uses (look for the crate's
  `schema/` and the drift test in `temper/tests/codecs` or the jig
  equivalent); never hand-edit `generated/`.
- Only the task that owns a schema changes it. A task that needs a field
  another task's schema lacks stops with a `Question:`.

## Fakes and worlds

- Tests use the tree's existing worlds and fakes; a new world only when the
  spec says so.
- Every new behaviour that survives a restart is tested across a crash or
  restart in the world that owns the state (the engine's world for the
  root, the store world for the store, the box's `Harness` for the box).

## What not to touch

- No task touches `agents/`, `fab/`, `tools/tiers/`, another plan's files,
  or this plan's `plan.toml`, `plan.md`, `status.md` and `notes/`.
- `scripts/old-forms.txt`: lower it with `old_forms.py --write` only for
  what the task removes; never raise it.

## Checks before the push

- `cargo fmt --all`; `heavy cargo clippy -p PKG ... --all-targets -- -D
  warnings` for every package changed; the focused tests of the packages
  changed and their direct dependents, through `heavy`, one command at a
  time (implementer text, "From fabrig's agents/implementer.md").
- A change of meaning of a shared item runs the tests of every package
  that reads it; the spec names them.
