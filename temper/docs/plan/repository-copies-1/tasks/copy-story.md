+++
id = "copy-story"
part = "copy-story"
kind = "feature"
needs = ["copy-machine", "copy-words", "fake-migrate"]
files = ["skein/tests/forge/setup/tests/suite/copy.rs", "skein/tests/forge/setup/tests/suite/main.rs", "skein/tests/forge/setup/src/world.rs", "skein/tests/forge/setup/src/host.rs", "skein/tests/forge/setup/src/fixtures.rs"]
effort = "high"
minutes = 50
+++

# `Copy` on the fake: the two real sides meet

## Goal

The plan runs on the fake forge's route, in the setup world, through HTTP bytes: the story of
`skein/docs/design/forge-setup.md`, 4.10 from both sides.

Acceptance, each a test in the new `tests/suite/copy.rs`:

- `a_copy_lands_private_with_the_source_the_bots_access_and_the_starter`.
- `a_copy_run_again_writes_nothing`.
- `a_copy_cut_short_run_again_finishes`: a `Cut` source, then the fill made `Whole`: the run stops,
  the mark is kept, the second run deletes and copies.
- `a_name_taken_stops_and_writes_nothing`, with no mark.
- `a_refused_source_says_the_forges_words`.
- `a_slow_copy_past_its_deadline_stops_timed_out_and_the_next_run_finds_it`.
- `a_second_writing_run_beside_a_copy_is_busy`, and `reachable` beside it is not.
- The world's item store holds `Copying` per copy, as the owner keeps it: `World` answers `Read` and
  `Write` of it from its own field.
- `heavy cargo nextest run -p skein-forge-setup-world` passes.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S2 to S4.
- `skein/docs/design/forge-setup.md`, 4.10, 6.3; `skein/docs/design/fake-forge.md`, 2.6 and 4.

## Owns and may change

- The files of the header. D031 records two worlds in this crate
  (`skein/docs/development/debt/D031-forge-setup-two-worlds.md`): these stories go through the one that
  plays the host, `HostWorld`, since words and deadlines are the host's.
- `fixtures.rs`: `copy_deadline` and a `Source` builder, reachable to the plan tests only.
- Remove from `skein/docs/design/forge-setup.status.md` this plan's **Not yet** text on 4.3, 4.10 and
  the host, leaving section 8's to `copy-facts`, and from `skein/docs/design/fake-forge.status.md`
  what has landed of 2.6 and 4. (A stated departure from the implementer's text, `notes/common.md`.)
- Pre-approved: if a fact of `copy-facts` makes a step of 4.10 wrong, the test states what the design
  says, and the report names it; the fix is a revision, not yours.

## Steps

1. The world's `Copying` and fixtures. 2. The stories. 3. Each test biting: drop the mark's keeping, and
answer `Down` for the deadline, each once. 4. The status lines. 5. The implementer's checks; commit and
push.

## Report

As the implementer's text asks.
