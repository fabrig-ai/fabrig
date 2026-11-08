+++
id = "copy-ready"
part = "copies"
kind = "feature"
needs = ["copy-ctl"]
files = ["temper/tests/one-process/src/ready_copies.rs", "temper/tests/one-process/tests/suite/ready_copies.rs", "temper/tests/one-process/src/lib.rs", "temper/tests/one-process/tests/suite/main.rs", "temper/tests/one-process/src/copy.rs"]
effort = "high"
minutes = 150
+++

# Ready's two copies made in order and adopted, the first as home, in the one-process world

## Goal

The process walk `copy::readys_two_copies_are_adopted_in_order_with_the_first_as_home`
(`temper/crates/temper/tests/fake_backends/walk/copy.rs`) sends Ready's
burst of two copy requests in one client pass and sees them made one at
a time, in that order, and adopted, the first as the project's home. The
joint world's `first_repository::ready_copies_reach_the_engines_making_rows_in_order`
holds the burst and the rows up to Waiting; it has no setup driver, so
the making and the adoption can't run there without injecting
`MakingMade`, a stub of what the walk proves. The one-process world
(`temper/tests/one-process`) runs the service's copies and a native
person in front of temper's terminal client: the story goes there. The
walk itself is `walks-go`'s; you never touch it.

Acceptance:
**`readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home`**,
new, in `temper/tests/one-process/tests/suite/ready_copies.rs`, its world
in a new `temper/tests/one-process/src/ready_copies.rs`:

1. The catalogue offers two public sources on another host, as the
   walk's does: the first's landing branch `main`, the second's `trunk`,
   and destination names that **sort against the request's order** (the
   walk's `widget` then `gadget`), so that request order and name order
   disagree (`next_making` breaks a tie of `asked` by key).
2. The forge is connected (through ctl, as `copy.rs` does, or the
   welcome's Forge step), with no first repository.
3. The native person (`world::Person::pages` with its steps, as
   `m15::pages` drives the welcome's forms) takes the welcome's Copies
   way, enters both addresses in order, reaches Ready and presses
   "Create your project": the client's `copy_all` sends the project and
   both copies in one pass.
4. Both copy requests are kept; on every store change the referee sees
   at most one row `Making`, none failed, each row naming project 1 and
   its source, `home` only on the first; and at least once the first is
   `Making` while the second is `Waiting`.
5. Both are adopted: no making row is left, project 1 holds both
   repositories, owned, the first home and the second not, each with its
   source's default branch.

- **Red first:** swap the two rows' order of choice by hand (for
  instance make `next_making` prefer the larger `asked`, in
  `temper/crates/temper-engine-domain-forge/src/making.rs`, about 94),
  run the story alone, see step 4's order fail, restore. If the story
  fails before any change because the burst's two rows share an `asked`
  and are chosen by name, that is a product finding: stop, report it
  under `## Left` with the trace, and leave the product as it is.
- Clippy clean, fmt clean, `temper-one-process-world`'s focused suite
  green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the
  `copy::readys_two_copies…` ruling (its keeps: the burst in one client
  pass, the order, the first as home).
- `temper/docs/design/domain/testing.md`, 2.5 and 2.6, the bullet **The
  fake box keeps two walks**, its **Copies made and adopted** item.
- `temper/docs/design/domain/forge.md`, 4.5;
  `temper/docs/design/client/README.md`, 3.5 (Ready's Copies way);
  `temper/docs/design/draft/repository-copies.md`, 7.
- `temper/docs/plan/walks-lower/tasks/copy-ctl.md`: the copy world it
  left on the trunk, whose source, peer and referee helpers you reuse.
- The walk (read only), for its checks: `copy.rs`'s `Copies` host and
  `ready_copies` referee.

What the survey found, for your bearings (verify before relying):

- The client's burst: `pages::default_told` → `copy_all`
  (`temper/crates/temper-client-domain/src/pages.rs`, about 2191 to
  2313), ordinal keys `welcome-copy-N`, home only on the first.
- `world::Person::pages(root, steps)` and `m15::pages::steps` drive a
  fake person through the welcome's forms on the native client;
  `copy.rs`'s `run` hosts the service, ctl, and the copy-capable
  catalogue peer.
- The joint world's `first_repository.rs` (about 307 to 399) shows
  Ready's Copies way and its act in the joint world, for the steps'
  shape.

## Owns and may change

Owns:

- `temper/tests/one-process/src/ready_copies.rs` (new) and its `mod`
  line in `src/lib.rs`.
- `temper/tests/one-process/tests/suite/ready_copies.rs` (new) and its
  `mod` line in `tests/suite/main.rs`.
- `temper/tests/one-process/src/copy.rs`: helpers made `pub(crate)` or
  generalised (a source with several repositories) for reuse, the
  existing three stories unchanged in what they check.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/one-process/src/world.rs`: a `pub(crate)` widening, or
  one small function appended, if the person needs it; named in the
  report.

Pre-approved options:

- **The welcome's form:** if the person can't reach Ready's Copies way
  through this world's pages, let the person's client send Ready's own
  act (`pages::Act::MakeDefault` after the Copies way's typed addresses,
  as the joint world's `copy_act` does): the burst is still the client's
  `copy_all`. Say so.
- **A check the world can't make without a stub:** stop that keep, write
  the rest, report it under `## Left` with what the world lacks; the walk
  then stays for it.

## Steps

1. Read the inputs, `src/copy.rs` as `copy-ctl` left it, `src/m15.rs`
   and `src/m15/pages.rs`.
2. Write the world and the story; run it alone:
   `heavy cargo nextest run -p temper-one-process-world --test suite -E
   'test(=ready_copies::readys_two_copies_are_made_in_order_and_adopted_with_the_first_as_home)'`.
3. The red as above; restore; green.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-one-process-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-one-process-world` (check the count). Commit
   (`walks-lower: Ready's two copies made in order in the one-process
   world`, the body saying why: the joint world can't make a copy
   without a stub) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the red and its failure; whether the person pressed
Ready through the form or sent its act; each keep held, or under `##
Left`; the story's CPU seconds; the minutes.
