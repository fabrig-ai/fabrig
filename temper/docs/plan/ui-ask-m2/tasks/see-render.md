+++
id = "see-render"
part = "render"
kind = "feature"
needs = ["see-surface"]
files = ["temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/see/**", "temper/crates/temper-engine-smith/src/tests/see.rs", "temper/crates/temper-engine-smith/src/tests/see_person.rs", "temper/crates/temper-engine-smith/src/tests.rs"]
effort = "high"
minutes = 200
+++

## Goal

Render what `see`'s person reads answered, for `me`, `project`, `goals`,
`goals ended`, `tasks`, `task N`, `task N tree` and `inbox`, by
`temper/docs/design/domain/agent.md`, 8.4.1, *How an answer reads*: an
answer a person's explainer can read, names beside numbers, UTC times,
money in dollars, a list a table of lines, and no act, key, digest,
cursor or command. `see-render-runs` renders the rest (`task N activity`,
`task N turn A.T`, `change N`, `changes`) on the same frame.

Acceptance:

- `see_answer` renders `SeeAnswer::Person` for the `SeeRead` variants
  `Me`, `Project`, `Goals`, `Tasks`, `Task`, `Tree` and `Inbox` into a
  whole text that `page` cuts as today (6,144-byte parts, `more: TOKEN`),
  then its `see:` line, the row's of 8.4.1's table (`goals ended`'s its
  own). `Activity`, `Turn`, `Change` and `Changes` still answer `see
  ADDRESS: not there yet.` until `see-render-runs`.
- **First lines,** exactly (N, NAME and counts filled):
  - `me`: `you: PERSON · ROLE in project N, PROJECT`
  - `project`: `project N, PROJECT`
  - `goals`: `goals of project N, PROJECT: K, by priority`; `goals ended`:
    `ended goals of project N, PROJECT: K, the latest ended first`
  - `tasks`: `live tasks of project N, PROJECT: K, the latest active first`
  - `task N`: `task N, TITLE · KIND · STATE`
  - `task N tree`: `delegates of task N: K live, J ended`
  - `inbox`: `waiting on you in project N, PROJECT: K, oldest first; each is answered on Needs you`
  where a count whose page has more is `K or more`.
- **Rows,** a line each, fields joined by ` · ` in 8.4.1's order for the
  address, for example (the fields bind, the words between are yours):
  - a task line: `task 41, Add CSV export · a change · an agent, charter 2, the producer's, on gpt-6 · working · 2026-11-09 07:40`;
  - a goal: `goal 12, Add CSV export · priority 1 · waiting on Ada · report: The export lands this week.` (`no report yet` where it has none);
  - an inbox entry: `question · task 41, Add CSV export · Which delimiter should the export use?`;
  - `project`'s sections as headed lists: `repositories: K`, `people: K`
    (each `NAME, ROLE`, the reader's `(you)`), `this period: N, from TIME
    to TIME · budget $B · reserved $R · spent $S · left $L`, `held tasks:
    K`;
  - `me`'s `your pool this period: $B, $L left` and `waiting on you here:
    K · 1 question · 2 proposals` (kinds with none left out) before the
    entries.
- **Names:** a person by `SeeNames`' name, else `person N`; a charter
  `charter N, NAME` (the name as kept), else `charter N`; a model by the
  catalogue's name for its index; a task by number and title; a label 1
  `a chat`, 2 `a goal`, a change's `a change`, any other `label N`.
- **Formats:** times `YYYY-MM-DD HH:MM` in UTC; money from cents as
  `$D.CC`; words cut to their first line and at most 160 bytes, at a
  character, ending `…` where cut; no digest, key, cursor, command or act
  name in any answer.
- **A part not read:** a later read answered `None(_)` renders its section
  as one line saying so (`report: not read`, `held tasks: not read`), and
  the answer goes on.
- `temper/crates/temper-engine-smith/src/tests/see_person.rs`: one table,
  a row an address (and `goals ended`), each a `SeeAnswer::Person` built
  from hand-made documents, checking its first line, lines it must hold,
  that it holds no `digest`, `cursor` or `temper ctl`, and its `see:` line;
  edge rows: an empty list (`: 0`), a page with more (`or more`), a later
  read not read, a person with no name kept, a long title cut.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 2.
- `temper/docs/design/domain/agent.md`, 8.4.1: the rows `me` to `inbox`
  (their *Answers* and `see:` columns), *How an answer reads*, *Bounded*,
  *Then what to read next*; `temper/docs/design/draft/ask-temper.md`, 5.1
  (*How an answer reads*).
- `temper/crates/temper-engine-smith/src/see.rs` (`see_answer`,
  `following`, `page`, `proposal_text` for translating a core answer to the
  client's contract with `jig-protocol-engine-clients`).
- The words ctl uses for the same documents, for reference only (this
  text is not ctl's): `temper/crates/temper-client-tools/src/jig/render/`
  (`lists.rs`, `document.rs`'s `amount` and `utc`,
  `document/projects.rs`); fixtures to copy for documents:
  `temper/crates/temper-client-tools/src/tests/fixtures.rs` (`me`, `goal`),
  `temper/crates/temper-client-tools/src/tests/jig/task_words.rs`.

## Owns and may change

- **Owns:** `see.rs`'s dispatch for `Person` and the new module
  `see/person.rs` (or `see_person.rs`, beside it) that renders these seven;
  the new test file and its `mod` line in `tests.rs`.
- **Not yours:** the `see:` constants and `ADDRESSES` beyond adding the
  new answers' lines to `following` (`see-page` moves them into a table);
  `temper-engine-domain`.
- **Options, pre-approved:** render from the client's contract types
  (translated with `jig-protocol-engine-clients`, as `proposal_text`
  does) or from the core's documents directly; pick one for all seven and
  say which. If a field 8.4.1 names is in no document the read gives
  (a goal's priority, a task's charter), leave it out of the row and list
  it in the report; never read anything more.

## Steps

1. The dispatch and one address end to end (`tasks`), with its row.
2. The other six, each with its row; the edge rows.
3. For one row, break a rule once (print the digest, or the cents
   undivided) to see it fail, and restore.
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-smith --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-engine-smith --lib`;
   `heavy cargo check -p temper-service --tests`. Commit and push.

## Report

As the implementer's text asks; paste one rendered answer per address.
