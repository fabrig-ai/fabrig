# component-kit-3: the rules every task keeps

The run moves shared mechanism that skein's components write privately
into lib and io, and moves the call sites onto it
(`skein/docs/design/draft/component-kit.md`, sections 1 to 4). Every task
names this note; where a task departs from it, the task says so.

## Read first

- `AGENTS.md`, then `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, sections 1 and 10 (the
  Rust subset step crates keep); `skein/docs/foundation/testing-strategy.md`,
  section 1 and section 10 (a test's shape).
- The task's own design sections, never a whole design.

## Scope

- **skein only.** jig, smith and temper keep their own copies of these
  helpers this run. A task touches another tree only where a public change
  of skein's breaks a build there, and then at call sites only, named in
  its report.
- **One writer per crate.** A task changes only the crates its spec owns.
  A reader or neighbour file outside them changes only as its spec allows.
- **No new public item** beyond what the spec sketches. lib's and io's new
  items are those of `kit-contract`; a migration that needs another is a
  `Question:`, never a private helper beside the call site.
- **Not yours:** any `docs/design/` (the draft and the designs), `docs/help/`,
  and this plan's directory. What the code taught goes in the report.

## A migration changes no behaviour

- A migration task replaces a private copy with lib's or io's item and
  changes nothing a test or a peer can observe. Existing tests stay as they
  are, except a test of a private helper the task removes: that test goes
  with it when lib's own tests cover the same contract, and the report
  names it.
- **Drift.** Where a copy differs from the shared item (a case it accepts,
  a bound, an order), keep the copy's observable behaviour unless no test,
  no design section and no peer depends on the difference. Keeping it means
  a check beside the call (for instance, refusing an uppercase digit before
  calling `hex::value`), never a second copy of the helper. Name each such
  case in the report.
- Remove what the move leaves dead (imports, fields, constants, a module
  that only held the copy).

## The counts

The counts of `notes/counts.md` sit in `skein/scripts/old-forms.txt` and
may only fall. After removing copies, from `skein/`:
`python3 -I -B scripts/old_forms.py --write`, and commit the lowered file
with the change. Never raise a count; a change that would is a `Question:`.

## Tests and checks

- After each change, the narrowest test that checks it (`AGENTS.md`, 6).
- When done, once: `cargo fmt`; `heavy cargo clippy -p PKG ... --all-targets
  -- -D warnings` for every package changed; `heavy cargo nextest run -p PKG
  ...` for the packages changed and their direct dependents inside skein
  (`cargo tree -i PKG -e normal,dev --depth 1` lists them).
- Fixtures: reuse the crate's or the world's existing fixtures and limits
  constants; never retype a `Limits` literal a fixture already gives.
- No `#[path]` and no `include_bytes!` across a crate boundary.

## Code

- Comments say what is true now, in the design's words; a module or type
  cites its design section.
- A lint exception is a scoped `#[expect(..., reason = "...")]`.
- Commits are small and say why.
