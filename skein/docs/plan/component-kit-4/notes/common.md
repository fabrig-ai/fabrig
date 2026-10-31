# component-kit-4: rules every task keeps

The run: skein's component kit, phase 1
(`skein/docs/design/draft/component-kit.md`). Shared mechanism written
privately in many crates moves once into lib, io, the shell and the world
harness, and the copies move onto it. These rules hold in every task; a
task's spec departs from them only where it says so.

## Read first

- fabrig's `AGENTS.md`, then `skein/AGENTS.md`.
- `skein/docs/foundation/programming-model.md`, 1 and 10 (the step subset)
  for any step crate you touch (`skein/crates/*`); 5.2 for lifecycle.
- `skein/docs/foundation/testing-strategy.md`, 1 and 6 for any test crate.
- The draft's section your spec names. The draft is the user's: never edit
  it, nor any `docs/design/` file. What the code taught you goes in the
  report.

## The kit is the contract

The kit's public items, once merged, are fixed for the run:

- lib (task `lib-kit`): `skein_lib::earliest`, `Queue::clear`,
  `Stack::clear`, `skein_lib::Lifecycle`, `Slab::iter`, `Slab::iter_mut`
  (`skein_lib::slab::{Iter, IterMut}`), `skein_lib::hex::{digit, value,
  lower_value}`, `Decimal::parse`.
- io (task `io-stage`): `skein_io::stage::{up, down}`.
- shell and world (task `world-kit`): `Host::next_mechanism_deadline`, the
  provided `Host::next_deadline`, `skein_shell::Forward` (re-exported by
  skein-world), and `Referee`'s defaults for `act`, `next_deadline` and
  `overdue`.

A migration uses these as they are. If a site needs a kit item to change or
a new kit item, do not add one in your crate and do not edit the kit's
crate: keep that site's old form, say so in the report (the site, what it
needs), and finish the rest. Never write a new private copy of a kit item
under another name: a renamed copy is a copy.

## One writer per crate

Your spec lists the crates you own. Outside them you change only what the
spec allows (call sites of an item you changed; a literal's new field).
Nothing in `smith/`, `jig/` or `temper/` changes in this run, unless your
spec says so for a compile repair.

## Behaviour stays

A migration changes form, not behaviour. Where a site's old code differs
from the kit item's contract (an order, a bound, a case rule, an edge such
as an empty input), keep the site's behaviour: call the kit item and keep
the site's extra check inline beside the call, or keep the site's old form
and report it. A changed order of events, of submissions or of slab
iteration is a change of behaviour.

## The counts

`skein/scripts/old-forms.txt` holds the counts that may only fall
(`skein/scripts/old_forms.py`; the forms of this run are added by task
`counts`). After your changes, from `skein/`:

```
python3 -I -B scripts/old_forms.py --write
```

and commit the lowered file with your change. If your base does not yet
hold a form, skip it: fab recounts where merges meet. A raised count is
red: never add an old form, not even in a test.

## Tests

- Each change's narrowest test first (`AGENTS.md`, 6), then, once, the
  checks of the implementer's text: `cargo fmt`, clippy on every package
  you changed, the focused tests of those packages and of their direct
  dependents within skein.
- A step crate's tests are its own (`src/tests/`), a world's are its
  crate's (`tests/<name>/`); no new `#[path]` include and no new
  `include_bytes!` across crates.
- A test you delete because its subject moved to the kit is named in the
  report, with the kit test that covers it.

## Report

Besides the implementer's text: the counts before and after
(`old_forms.py`'s figures for each form you touched), the production and
test lines your change added and removed (`git diff --stat` against your
base, split into `src/` and tests), every site you left in its old form and
why.
