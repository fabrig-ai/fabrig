+++
id = "team-joint"
part = "proof"
kind = "feature"
needs = ["team-reads", "team-page"]
files = [
  "temper/tests/client/tests/suite/team.rs",
  "temper/tests/client/tests/suite/main.rs",
]
effort = "high"
minutes = 120
+++

# The client's joint world: Team with two people

## Goal

One story in the client's joint world, the real client and engine roots:
two people in one project each open Team and read the project's people,
themselves first, and what waits on each, the owner's question counted on
both sides: on the owner's own line from Needs you, and on Sam's Team as
what waits on the owner on the owner's own task.

Acceptance, by `temper/docs/design/client/README.md`, 3.5, *Team*:

1. A new `temper/tests/client/tests/suite/team.rs`, `mod team;` in
   `main.rs` beside its siblings, holding one test,
   `two_people_read_each_other_on_team`:
   - the owner (`fixtures::PARTY`) in the project, with a question
     committed by their running chat for them, as `fixtures::question`
     commits one;
   - a second person bound and given a member's role in that project
     through the owner's real request, as `suite/to_begin.rs`'s
     `skip_leaves_the_others_to_begin` does (`bind_dismissal_person`,
     `add_dismissal_member`);
   - the owner's client at `Team { project }`: the heading "Who owns what
     on PROJECT", "Two of you.", the first box "You" with "Owner of
     PROJECT" and "1 question waits on you.", the second the member's
     name, "Member" and "Nothing waits on NAME.";
   - the member's client at `Team { project }`: the first box "You",
     "Member", "Nothing waits on you."; the second the owner's name,
     "Owner of PROJECT", "1 question waits on NAME.";
   - the owner answers the question in Needs you as `suite/inbox.rs`'s
     stories answer one; after settling, the owner's line is "Nothing
     waits on you." and the member's view of the owner "Nothing waits on
     NAME.".
2. The story reads the view's tree by role and name, as its siblings do;
   it asserts no request's order beyond what its words need.
3. Its CPU time under 0.6 s, in the report (skein's
   `docs/foundation/testing-strategy.md`, 8: a joint world's story).

## Inputs

- `temper/docs/plan/ui-p4a-team/notes/common.md`;
  `temper/docs/plan/ui-p4a-team/notes/contracts.md`, T3.
- `temper/docs/design/client/README.md`, 3.5, *Team*, *A person's line*
  and *Its reads*.
- The world: `temper/tests/client/src/fixtures.rs` (`world`, `question`,
  `PARTY`), `src/front/dismissed.rs` (`bind_dismissal_person`,
  `add_dismissal_member`), `tests/suite/to_begin.rs` (two people in one
  project, its `client` helper), `tests/suite/inbox.rs` and
  `question.rs` (a question answered).

## Owns and may change

Owns the two files in the header; in `main.rs` the one `mod team;` line
alone. Neighbour: `temper/tests/client/src/fixtures.rs`, where one `pub fn`
beside `question` may be added if the story can't make the owner's
question and the second person meet in one world otherwise, named in the
report (ui-p2-home's `joint-across` also writes that file: add, never
change, a function).

Options, each decided:

- **The question's world and the two-person world can't meet** without a
  change of the front or the engine (beyond a fixture function): keep the
  story's two people, their names, roles and "Nothing waits on" lines,
  drop the question's rows, and list it under `## Left` with why.
- **The member can't see the owner's chat in a `tasks{query}`** (a
  refusal of the watch): the member's line of the owner is absent, which
  the story asserts instead, and the report names it as a finding for
  temper/ui, since the design expects it visible (jig's
  `domain/people.md`, 4).
- **A sibling story's helper is private to its file:** copy the few lines
  into `team.rs` rather than move it.

## Steps

1. Read the Inputs. The story (1, 2); bite: in a scratch copy of
   `temper-client-domain/src/pages/team.rs`, not committed, count lines
   waiting on any person, see the member's view of the owner fail, restore.
2. Runs through `heavy`: `cargo nextest run -p temper-client-world --test
   suite -E 'test(=team::two_people_read_each_other_on_team)'` with its
   CPU time; then `cargo nextest run -p temper-client-world` whole,
   counts in the report; clippy on `temper-client-world`; then as
   `notes/common.md` says before the push. Commit, saying that two people
   read each other on Team in the joint world, and push.

## Report

The implementer's report: the story's steps and what each asserts, the
fixtures used or added, the bite and its failure, the runs' counts and the
story's CPU time, and what is left.
