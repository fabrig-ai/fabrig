+++
id = "joint-across"
part = "proof"
kind = "feature"
needs = ["all-home-reads"]
files = ["temper/tests/client/tests/suite/status.rs", "temper/tests/client/tests/suite/inbox.rs", "temper/tests/client/tests/suite/away.rs", "temper/tests/client/src/fixtures.rs"]
effort = "high"
minutes = 160
+++

## Goal

The client's joint world across two projects, through the real engine
and the real client roots: Home on All projects reading both projects'
status, briefs, looks and decisions, and Needs you across both, as
`temper/docs/design/client/README.md`, 3.5 has them. These are the
stories that cross the boundary; the tables of each crate stay where they
are.

Acceptance, stories beside their siblings' fixtures, a row of an existing
table where the behaviour is one already told:

- **Home across two projects** (beside `status.rs`'s `Story`): a person
  in projects 1 and 2, landings on both and the status procedure's brief
  on each; the person goes to `AllHome`; each project's `Looked` reaches
  the engine once, and a second entry within the sitting sends none;
  `all_home::facts` shows both statuses; `all_home::intro` says
  `Landed::Across { count, projects: 2 }` and its brief joins both
  projects' first sentences, newest first; a project's status refused
  leaves that project `Unknown` and the other shown.
- **Decided for you across two projects** (beside `away.rs`'s fixture):
  an Away decision on each project; `AllHome`'s facts hold both rows,
  oldest first, each its project's; Overturn from `AllHome` on project 2's
  row reaches the engine under project 2 and its task is told as on a
  project's Home; Needs you's `all_home::decided` holds both rows, and only
  project 1's at Needs you filtered by project 1.
- **Needs you across two projects** (beside `inbox.rs`'s fixture): a
  question asked in each project; the unfiltered inbox's open cards hold
  both, each with its project; answering project 2's from `AllHome`'s card
  through the held send leaves project 1's open; `parts::needs_facts`
  counts what is open.
- **Ask temper's line:** `Act::AskLine` then `Act::AskWith` on `AllHome`
  opens `Ask { project }` on the open project with the words in its
  composer and no chat started.
- The focused tests of `temper-client-world` pass, and the run's CPU
  seconds for the stories added are in the report.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Home on All projects*,
  *Needs you* (*Decided for you*), *Away* (*Overturn*), *A page's parts*.
- `temper/docs/design/domain/testing.md` (the joint world's place).
- `temper/docs/plan/ui-p2-home/notes/contracts.md`, sections 2 to 4.
- `temper/docs/plan/ui-p2-home/notes/common.md`.

## Owns and may change

- Owns the stories it adds in the files above, and a second project's
  fixture in `src/fixtures.rs` if `to_begin.rs`'s `OTHER_PROJECT` way does
  not serve (reuse it first).
- May change: `tests/suite/main.rs`, a `mod` line if a story needs a file
  of its own; nothing in `tests/suite/fuzzy/` (`density-fuzz`'s).
- Options, each pre-approved: if a fixture `Story` is private to its file,
  add the story in that file rather than share the fixture; if the engine's
  status procedure cannot run on a second project in the fixture, deliver
  that project's brief as `status.rs`'s refused-run story does and say so.

## Steps

1. Read `tests/suite/status.rs` (`Story`, `sign_in_plan`, the look
   stories), `away.rs`, `inbox.rs` and `to_begin.rs`'s second project.
2. The stories, a person's acts through the root as the siblings drive
   them, the engine's state read where the siblings read it.
3. Break each story's rule once by hand (a project's look not sent, a row
   under the wrong project) and see it fail.
4. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-client-world`.
   Commit and push.

## Report

The stories, where each went and why, what each failed with when its rule
was broken, the CPU seconds they add, and your implementation and check
minutes apart.

## Answers

- 2026-11-08. *Does `AllHome` send each project's `Looked` once per
  sitting or once per entry?* Once per entry, as
  `temper/docs/design/client/README.md`, 3.5, *Home on All projects*
  (*Its reads*) has it and `status.rs`'s
  `a_second_entry_within_the_sitting_keeps_its_last_looked` tells for a
  project's Home. The acceptance's "a second entry within the sitting
  sends none" was a misreading: read it as "the settled page sends no
  second look while that entry stays open". The story then leaves
  `AllHome`, comes back within the sitting (under 30 minutes), and checks
  that each project's `Looked` reaches the engine once more, its `latest`
  advances, and its `previous` and `seen` stay as the first entry left
  them. Change no production code for this. The skeleton is unchanged.
