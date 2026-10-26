# Supplement jig-04.s1: a compound's operations numbered by place

Added by ruling 184, from the design answer of 2026-10-16T23:28Z (jig
`a20cf96c4`: `client/domain/contract.md`, 3.4; `domain/people.md`, 5.1).
jig 14 and jig 15 need its landing (their 14.3 and 15.4 read the
numbering). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan"; `jig/04-grace-and-compound.md`, 04.6.

Scope: `contract.md`, 3.4: "Its operations take consecutive message
numbers, one each, in the request's order, and the answer names the
first: the operation at place *i* took the first plus *i*. ... What
another operation adds to its task's inbox, if anything, has its number
too; one that adds nothing leaves its number unused."
Needs: jig 04.7's landing, earlier in this session (both change
`tests/core/tests/operations.rs` and the people child).
Files, at jig `a20cf96c4`:
- `crates/jig-domain-core/src/compound.rs`: `issue` (:224) takes a fresh
  `Family::Message` number for every operation; the arms that return
  `Ok(None)` today (`Cancel`/`Release` with `message: None`, `Move`,
  `Stop`) take one and leave it unused; `first_message` (:43, :60) is the
  number of the operation at place 0.
- `crates/jig-domain-people/src/domain.rs`: `valid_answer_shape` (:1261),
  its `Outcome::Compound` arm (:1345) admits a made compound only when
  `first_message != 0`, as `Created` checks `first != 0` (:1359).
- Fakes whose `first_message: 0` become 1:
  `crates/jig-domain-people/src/tests/compound.rs:33`,
  `tests/people/tests/compound.rs:27`, `tests/people/src/world.rs:563`.
- `tests/core/tests/operations.rs` (:134-135, :239-240): the chat's
  graced cancel is at place 2, so its grace's `message` is asserted as
  `first_message + 2`; the answer's `first_message` stays 1.
- Refresh any further copy of a made compound's `first_message: 0`
  (04.7's scenarios included), and list it in the commit body.
Test shape: the people child refuses a made compound answered with
`first_message: 0` (beside `tests/compound.rs`, under 20 ms); the two
`operations.rs` tests above, as amended.
Proof: `cargo nextest run -p jig-core-world -E
'test(a_compound_closes_a_conversation_moving_one_delegate_and_cancelling_another)'`,
once. Nothing else is wanted; jig's gate.
Size: aim for at most 30 lines, production and tests; past it, report why.
Don't: change `people::Outcome::Compound`'s fields, its `Copy`, the
people codec or `people_v3`; number a lone operation differently; name
temper.
Done when: `git -C ~/src/rust/jig log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: jig-04.s1$' --format=%H -1`
prints its landing.
