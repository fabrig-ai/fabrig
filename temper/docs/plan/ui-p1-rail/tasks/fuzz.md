+++
id = "fuzz"
part = "checks"
kind = "feature"
needs = ["rail-parts", "work-list", "work-page"]
files = ["temper/tests/client/tests/suite/fuzzy/rail_and_work.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 90
+++

# A fuzz test of the rail's and Work's parts against surfaces.md 12

## Goal

Seeded random facts and random moves of a project's settings, through the rail's
and Work's pure functions, checked against `temper/docs/design/draft/surfaces.md`,
section 12's rules as the client's design gives them: a part shows exactly when
its rule holds; no part is drawn empty; every live item is placed exactly once;
and a move of the settings never hides a live item.

Acceptance:

1. `temper-client-world`'s fuzzy suite holds `fuzzy::rail_and_work`, selected by
   `--profile fuzzy` and by no focused profile.
2. For each seed, random `rail::Facts` give `rail::parts` exactly what an oracle in
   the test, written from `temper/docs/design/client/README.md`, 5.3's table *The
   rail's parts* and *While To begin shows*, says: every part, with its count.
3. For each seed, random `work::Facts` and a random `wall` give `work::place` a
   `Work` in which: every live goal is on exactly one `Goal` card; every queued
   change is on exactly one card, its goal's if that goal is live, else its chat's
   if that chat is listed, else its own; every chat with a queued change has exactly
   one `Chat` card and no other chat has one; every draft has exactly one `Draft`
   card; In flight and Drafts are newest activity first with ties by task; Landed
   holds exactly the landed changes of the week before `wall`; each of
   `Filters`' counts is what it counts (contracts.md, C4).
4. For each seed, a random kind 7 record and a random move to another: `work::shown`
   follows C7 before and after; the Work placed is the same before and after (no
   settings enter `place`); the rail's parts that kind 7 feeds (`drafts` into the new
   box, `away`) follow the rule after the move, and no part 5.3 shows always is
   hidden by it.
5. A failure names its seed and replays from it alone. The test's CPU time is under
   one second in the fuzzy profile, measured and said.

## Inputs

- `temper/docs/plan/ui-p1-rail/notes/common.md`; `contracts.md`, C3, C4 and C7.
- `temper/docs/design/draft/surfaces.md`, section 12, and section 14's last item.
- `temper/docs/design/client/README.md`, 5.3, *The rail's parts*; 3.5, *Work*, *What
  goes where* and *Its filters*.
- `skein/docs/foundation/testing-strategy.md`, section 1 and the fuzzy profile's
  rules; the sibling `temper/tests/client/tests/suite/fuzzy/public_address.rs`, its
  seeds and its independent oracle, as the model.

## Owns and may change

Owns: the new `fuzzy/rail_and_work.rs`; in `tests/suite/main.rs`, one line `mod
rail_and_work;` after `mod public_address;` in the `fuzzy` block.

Not this task's: any product code. A rule the functions break is a question, with
its seed: never change a function to pass.

Pre-approved options:

- **If the facts' space is small enough to cover whole** (the rail's booleans and
  small counts): cover it whole beside the seeds, and say so.
- **If the oracle needs a fact `place` doesn't return** (a card's activity): read it
  from the generated facts by the card's task number.

## Steps

1. Generators, from `skein_lib::Rng` and a seed: rail facts; Work facts with goals
   live and ended, changes queued and landed with each `Belongs`, chats with and
   without drafts, repeats and ties; kind 7 records and their moves.
2. The oracles and invariants of acceptance 2 to 4, independent of the functions'
   code.
3. Seeds as `public_address.rs` runs them; a failure message with the seed.
4. Break two rules by hand in a scratch copy of `rail.rs` or `work.rs`, not
   committed (Team at one member; a queued change of a live goal also on its own
   card), see the test fail with its seed, and restore.
5. **Check** as `notes/common.md` says, `--profile fuzzy -E 'test(/rail_and_work/)'`
   with its count. Commit and push: the message says section 12's rules now hold
   under random settings and their moves.

## Report

Follow the implementer's text. Also give the seeds' count, the CPU time, and the
rules you broke and the failures you saw.
