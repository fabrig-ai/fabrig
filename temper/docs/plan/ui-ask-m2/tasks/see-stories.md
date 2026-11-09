+++
id = "see-stories"
part = "stories"
kind = "feature"
needs = ["person-reads", "see-route", "see-render-runs"]
files = ["temper/tests/engine/domain/tests/suite/configuring/asking_person.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/fixture.rs", "temper/tests/engine/domain/src/**", "temper/docs/design/domain/agent.status.md", "temper/crates/temper-engine-domain/src/route/see.rs", "temper/crates/temper-engine-domain/src/route/see/**", "temper/crates/temper-engine-domain/src/tests/see.rs", "temper/crates/temper-engine-smith/src/see/**", "temper/crates/temper-engine-smith/src/tests/see_person.rs"]
effort = "high"
minutes = 210
+++

## Goal

The joined story of `see`'s M2 addresses
(`temper/docs/design/domain/agent.md`, 8.4.1, *Its tests*): in the
engine's domain world, real jig, the real root, the real forge and the
real renderer, an Ask temper chat reads each address as its person, of
its project alone, within its bound, as tables; and is refused another
project's work, a number with nothing there, and what its person may not
read.

Acceptance:

- One world builder, `asking_person::world() -> PersonWorld`, beside the
  asking stories' (`configuring/fixture.rs`'s `World`): project 1, the
  chat's, with a tracked goal whose report note says `report: The export
  lands this week.`, a live task and an ended one under it, a change on an
  adopted fake repository (as `forge_routes.rs`'s `change_world_funded`
  makes one) with its activity and at least one committed turn, and a
  question to the person waiting; project 2, where the same person holds a
  role, with tasks whose titles hold the word `ELSEWHERE`; an owner's Ask
  temper chat in project 1 (template 5) and a member's.
- One table test, `an_asking_chat_reads_its_project_as_its_person`, a row
  an address: `me`, `project`, `goals`, `goals ended`, `tasks`, `task N`
  (the goal, then the change), `task N tree` (the goal), `task N
  activity` (the change), `task N turn A.T` (its turn), `change N`,
  `changes`, `inbox`. Each row checks the answer's first line (as
  `see-render` and `see-render-runs` fix them), a line it must hold (the
  goal's headline, the person's role, the question's words, the change's
  gate, ...), its body within 6,144 bytes and the call's allowance, its
  `see:` line, and that no answer holds `ELSEWHERE`; the store's rows
  unchanged by every row; the turn read whole through `more`, its parts
  joined equal to the turn's text.
- Refusal rows in the same table: `task N` and `task N activity` of a
  project 2 task, `change N` of a number with no change, `task 999999`:
  each `see ADDRESS: not read: nothing you may read in this project is
  there.`, an error, holding no `ELSEWHERE`. The member's chat reads `me`
  with the member's role and `tasks` as the owner's does.
- `temper/docs/design/domain/agent.status.md`: the **Not yet** entry
  *Its tests*, "no row of the engine's domain world reads an address as
  the person", is deleted.
- The focused suite's time: the new test's CPU seconds measured and
  reported; if it passes 1.5 s, make its world smaller before anything
  else (testing-strategy, 8.1).

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 2.
- `temper/docs/design/domain/agent.md`, 8.4.1 (the rows `me` to `inbox`,
  *How an answer reads*, the person's *A refusal*, *Its tests*); jig's
  `jig/docs/design/domain/engine.md`, 7.3.
- The asking stories and their world:
  `temper/tests/engine/domain/tests/suite/configuring/fixture.rs`
  (`config`, `World::start`, `World::chat`, `see_whole`),
  `configuring/asking_forge.rs` (a repository and a long file read
  through `more`), `configuring/asking_proposal.rs`,
  `temper/tests/engine/domain/src/host_tool.rs`.
- Work to seed: `temper/tests/engine/domain/src/walking.rs` (the project
  seed, about line 358), `src/walking/creating.rs` (a tracked goal),
  `src/direct.rs` (a chat or task), `tests/suite/forge_routes.rs`
  (`change_world_funded`), and how a goal writes its report note in the
  engine world (`temper/docs/design/domain/agent.md`, 8.3, *Its stories*).

## Owns and may change

- **Owns:** the new story file and its `mod` line in `configuring.rs`;
  the world's helpers it needs in `temper/tests/engine/domain/src/` (new
  functions only, or a second seed project added to a config builder that
  no other story reads unchanged); the status entry.
- **Reader files:** `configuring/fixture.rs`, new helpers only.
- **Options, pre-approved:** if the fake forge's change takes too long to
  bring to a gate verdict, a change waiting on CI is enough for `change N`
  and `changes`; say so. If a goal's report note cannot be written without
  a scripted run, write it as the goal's run would, through its `note`
  call in the world's driver; say how. If an answer's text differs from
  `see-render`'s first lines where those are fixed, it is a red: report it,
  don't bend the row.

## Steps

1. The world, then one row (`tasks`) end to end.
2. The other rows, then the refusal rows. Break the project check once
   (read project 2's `tasks`) to see the `ELSEWHERE` assert fail, and
   restore.
3. The status entry; measure the test's time
   (`heavy cargo nextest run ... --no-capture` prints it, or the run's
   summary).
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E
   'test(/asking|configuring/)'`. Commit and push.

## Report

As the implementer's text asks; the rows' first lines as rendered, and
the test's CPU time.

## Answers

2026-11-09. **Should `see` supply and render an inbox question's words, or
should acceptance accept its kind and task alone?** Supply and render them:
`temper/docs/design/domain/agent.md`, 8.4.1's `inbox` row gives each entry
"its words' first line", and `see-render`'s acceptance gave the same row
(`question · task 41, Add CSV export · Which delimiter should the export
use?`). The row stays as written. The route and the renderer have landed, so
this task now owns the repair. Its files take the route's `see.rs`,
`see/**` and `tests/see.rs`, and the renderer's `see/**` and
`tests/see_person.rs`:

- **Route** (`temper-engine-domain`, `route/see.rs` and `see/person.rs`):
  after the needs page of `inbox` (and of `me`), read
  `Read::Question { task, message }` once for each `InboxObject::Question`
  row of that page, in the page's order, one read at a time, then deliver.
  The person may make that read (`notes/contracts.md`, section 1, admitted
  reads). `Gathered` keeps these answers in a list beside its first and
  second parts. A refusal still comes only from the first part. A word read
  answered `None(_)` leaves its row without words and the answer goes on.
  The contract grows by one field on each variant:
  `SeeRead::Me { person, project, needs, words: Box<[SeeWords]> }` and
  `SeeRead::Inbox { needs, words: Box<[SeeWords]> }`, with
  `pub struct SeeWords { pub task: u64, pub message: u64, pub words: Answer }`
  exported beside `SeeRead`. Update its doc comments to name the reads.
- **Render** (`temper-engine-smith`, `see/person.rs`'s `inbox_rows`, used
  by both `me` and `inbox`): a question row whose words were read is
  `question · task N, TITLE · FIRST LINE`, the words cut by see-render's
  *Formats* (first line, at most 160 bytes, `…` where cut). Without words
  it stays as today.
- **Other kinds** (proposal, escalation, person task) keep their kind and
  task for now. Add a **Not yet** entry under 8.4.1 in
  `temper/docs/design/domain/agent.status.md`: an inbox row's words are
  read for questions alone.
- **Tests:** one row in `tests/see_person.rs`'s table (a question with
  words and one whose word read answered `None`), and the route's existing
  table in `temper-engine-domain`'s `tests/see.rs` covering one
  `Read::Question` after the needs page. This story's `inbox` row then
  passes as written. Run the narrow tests of both crates, then step 4's
  commands. In the report, name `SeeRead`'s change as an interface change
  of `temper-engine-domain`.

