+++
id = "see-render-runs"
part = "render"
kind = "feature"
needs = ["see-render"]
files = ["temper/crates/temper-engine-smith/src/see.rs", "temper/crates/temper-engine-smith/src/see/**", "temper/crates/temper-engine-smith/src/tests/see_person.rs", "temper/docs/design/domain/agent.status.md"]
effort = "high"
minutes = 180
+++

## Goal

Render what `see`'s person reads answered for how work ran: `task N
activity`, `task N turn A.T`, `change N` and `changes`, on `see-render`'s
frame (`temper/docs/design/domain/agent.md`, 8.4.1, *How an answer
reads*): a call as one line, its tool and what it did, the turn holding
the rest; a change in temper's words with no digest; a queue in order.

Acceptance:

- `see_answer` renders `SeeRead::Activity`, `Turn`, `Change` and
  `Changes`, each with its row's `see:` line; nothing answers `not there
  yet.` for a `SeeAnswer::Person` any more.
- **First lines,** exactly:
  - `task N activity`: `activity of task N: K events, oldest first`
    (`K or more` where the page has earlier lines, and a last line `earlier
    events not read` then);
  - `task N turn A.T`: `turn A.T of task N` then ` · ` and its spend where
    the turn has one;
  - `change N`: `change N, TITLE · OWNER/NAME BRANCH into LANDING · STEP`;
  - `changes`: `changes of project N, PROJECT: K queued, J landing by turn`.
- **Rows:**
  - an activity line: `2026-11-09 07:40 · its run · turn 3.2 · 2 calls`,
    then each call indented by two spaces, `see: read settings charter 4`
    or `propose_settings: proposed, waiting for an owner`, a call a line;
    words by their first line; a run begun or ended; a turn too large for
    the page by its size and `see: task N turn A.T`;
  - a turn: its brief's parts, each by its heading and its first line (its
    whole text stays in the turn; a part a later milestone hides from other
    readers is none of this task's); its words whole; each call with its
    input (JSON as the run wrote it, cut at 512 bytes) and its result's
    text; a turn past the read's bound, `turn A.T of task N: N bytes, past
    what one read holds`;
  - a change: its pull request `pull request N`, its checks and gates each
    `NAME: VERDICT` (passed, failed, running, waiting), approvals `K of M
    approvals by ROLE`, its repairs counted, its questions and holds a line
    each, its place `first in the queue into BRANCH`, `Kth in the queue`
    or `landing by turn`; no commit hash, key or receipt;
  - `changes`: a headed list per landing branch, `into OWNER/NAME BRANCH:
    K`, its changes in order, `change N, TITLE · STEP · waiting on WHAT`;
    then `landing by turn: K`; then `landed lately: K`, each `change N,
    TITLE · into BRANCH · TIME`.
- `temper/crates/temper-engine-smith/src/tests/see_person.rs` gains a row
  each for the four, from hand-made documents (an activity page with a
  turn of two calls and one too large; a turn with a brief, words and a
  call; a change panel with a gate passed and one failed, one approval of
  two; a changes view of two queues and a landed change), and an edge row
  each: an empty activity, a change with no pull request yet, `changes`
  with the forge's landings `None`.
- `temper/docs/design/domain/agent.status.md`: the **Not yet** entry
  *How an answer reads* is deleted.

## Inputs

- `temper/docs/plan/ui-ask-m2/notes/common.md` and
  `temper/docs/plan/ui-ask-m2/notes/contracts.md`, section 2.
- `temper/docs/design/domain/agent.md`, 8.4.1: the rows `task N activity`,
  `task N turn A.T`, `change N`, `changes`; *How an answer reads*.
- `see-render`'s module and test table; how ctl draws the same, for
  reference only: `temper/crates/temper-client-tools/src/jig/render/activity.rs`,
  `render/turn.rs`, `temper/crates/temper-client-tools/src/change.rs`
  (`text`), the browser's `temper/crates/temper-client-view/src/pages/changes.rs`
  (`queues`); the forge's documents,
  `temper/crates/temper-engine-domain-forge/src/panel.rs` (`Panel`) and
  `src/views/changes.rs` (`ChangesView`), `src/views.rs` (`LandingsView`).
- A turn's bytes are smith's: decode them as the client's contract does
  (`jig-protocol-engine-clients`' translation of a turn read), not by hand.

## Owns and may change

- **Owns:** the same module and test file as `see-render`, for these four.
- **Options, pre-approved:** where the client's translation of a turn
  needs a bound or context `see_answer` lacks, pass it through
  `see_answer`'s existing parameters or the catalogue; a new parameter of
  `see_answer` is a question. If a field 8.4.1 names is in no document,
  leave it out and list it in the report.

## Steps

1. Activity and a turn, with their rows.
2. A change and the changes, with their rows; the edge rows.
3. Break one rule once (print a head's hash in a change) to see its row
   fail, and restore.
4. The status entry.
5. `cargo fmt`; `heavy cargo clippy -p temper-engine-smith --all-targets
   -- -D warnings`; `heavy cargo nextest run -p temper-engine-smith --lib`;
   `heavy cargo check -p temper-service --tests`. Commit and push.

## Report

As the implementer's text asks; paste one rendered answer per address.
