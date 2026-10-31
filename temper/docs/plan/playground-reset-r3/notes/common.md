# Standing rules for every task of playground-reset-r3

Every spec of this plan cites this note; it holds unless the spec says
otherwise, and says why.

## The change

- The contract is the draft `temper/docs/design/draft/playground-reset.md`
  (cited by section: `playground-reset.md, 4`), read with the adopted
  designs it amends (its section 10). Where the draft and an adopted
  design disagree, the draft wins: it is what this run builds. The
  draft's section 9 defaults are decided: take each default as said.
- What the code stood at when the plan was cut is
  `temper/docs/plan/playground-reset-r3/status.md`.
- Code cites the draft at module or type level as
  `temper/docs/design/draft/playground-reset.md, N`; never on every
  field. The draft is not adopted by this plan: no task amends the
  designs its section 10 lists.

## Boundaries

- jig names no forge, repository, git or temper (jig's `AGENTS.md`).
  A jig act for a project is generic: its connectors are told through
  jig's connector interface, never by naming temper.
- One writer per crate: the files a spec owns are its own for the
  wave. A change another task's crate needs is a question, unless the
  spec lists it under "may change".
- A required field crossing trees: the producer's branch edits the
  other tree's lines that name it, each only in the field's name, type
  path or value (a literal's new field, a match arm's new variant made
  to do what the spec says).
- Store families are in place until the first release (temper's
  `docs/design/protocol/store.md`, 2 and 4; jig's
  `docs/design/protocol/store.md`): a new family takes the next free
  byte and bumps `STORE_VERSION` once per branch, no migration.

## Not yours

- `docs/design/` of any tree, drafts and `docs/help/`: what the code
  taught goes in your report. A doc read by tests (jig's
  `docs/design/protocol/store.md`) is the exception only where a spec
  says so.
- Another task's owned files, `scripts/old-forms.txt` raises, any
  `Cargo.lock` change beyond the path dependencies your new edges need.

## Tests

- Fakes, never real services: skein's fake forge and fake LLM, temper's
  `temper-fake`, `temper box --fake-backends`. No probes of real
  forges or repositories.
- The narrowest test after each change (`AGENTS.md`, 6). Before the
  push, once: `cargo fmt`; `heavy cargo clippy -p ... --all-targets --
  -D warnings` over every package you changed; the focused tests of
  every package you changed and of its direct dependents in every
  tree (`heavy cargo nextest run -p A -p B ...`).
- A change of meaning (a value a shared item returns, an order it
  keeps, a refusal it gives) runs the tests of every package that
  reads the item; each spec names them.
- Old forms: from each tree you touched, `python3 -I -B
  scripts/old_forms.py`; a count your change lowers is lowered with
  `--write` in your branch; one it would raise is a redesign, never a
  raise.

## Machines

- Your slot is on a builder (hetzner6, whose hostname
  `debian-16gb-hel1-6` contains "hel1" but which is a builder, not
  hel8, or hetzner9). Every build and test is `heavy cargo ...`, one at a
  time; never `heavy --gate`.
