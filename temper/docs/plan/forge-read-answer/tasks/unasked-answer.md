+++
id = "unasked-answer"
part = "render"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/lib.rs",
  "temper/crates/temper-engine-smith/src/tests.rs",
  "temper/crates/temper-service/src/route.rs",
  "temper/docs/design/domain/agent.status.md",
]
effort = "high"
minutes = 45
+++

# No forge answer takes down `temper serve`

## Goal

`temper serve` panics in `forge_answer` (`temper/crates/temper-engine-smith/src/answers.rs`) on a forge read answered `Runs`, `Run`, `Commits`, `Tree` or `Queued`: two `unreachable!` arms. Such an answer becomes a refused read: words for the run naming the tool and the kind, a `Problem` in the rendering, and a read refusal note on stderr. The cause of such answers is `read-owners`' task; this one makes the rendering safe whatever the cause, including an answer already recorded in a store.

This task departs from the implementer's text in one way: one status entry is yours to remove (acceptance 4), because the design already says what this task builds.

Acceptance:

1. `temper_engine_smith::unasked_kind` exists as `temper/docs/plan/forge-read-answer/notes/contracts.md`, section 3 sketches it, and `Problem::Unasked(&'static str)` is the rendering's problem for those kinds. No `unreachable!` is left in `forge_answer`.
2. `answer`, `answer_bounded`, `answer_for_tool` and `answer_for_task` render each of the five kinds as an error whose text is exactly the contract's words, with `read_compare` and `read_file` taking the same words as any tool (no bounded path reaches `forge_answer` with such a kind, so no `expect` there can fire).
3. `render_host` (`temper/crates/temper-service/src/route.rs`) pushes the contract's `Note::ReadRefused` for such an answer, beside the rendered event it already pushes.
4. The **Not yet** entry for section 8 that begins *a forge read answered with `Runs`, `Run`, `Commits`* is gone from `temper/docs/design/domain/agent.status.md`, the rest of the file unchanged.
5. The table rows below pass and bite; clippy is clean for `temper-engine-smith` and `temper-service`; `heavy cargo nextest run -p temper-engine-smith --lib` passes.

## Inputs

- `temper/docs/plan/forge-read-answer/notes/common.md`.
- `temper/docs/plan/forge-read-answer/notes/contracts.md`, sections 1 and 3: they bind (names, words, the note's fields).
- `temper/docs/design/domain/agent.md`, section 8, the paragraph beginning *A refused forge read keeps its refusing layer and reason* to *A reviewer fails*: the design this builds.

## Owns and may change

**Owned:** the `files` above.

**Readers and neighbours, edits allowed:**

- `temper/crates/temper-service/src/tools.rs`: the arm `Problem::Unasked(_)` the compiler asks for in its match on `Problem` (a decode problem never is one; give it a plain body such as `b"host call answer of an unasked kind"`).
- Any other exhaustive `match` on `temper_engine_smith::Problem` the compiler names: its arm only.
- `temper/crates/temper-service/src/route.rs`: `render_host`'s signature and its one call site, to pass the `CallKey` the `HostDelivery::Render` arm already binds (`key.task`, `key.attempt`).

**Options, each with its default:**

- The words are built by `Text` and fit the bound by `clipped_refusal(text, cap)`, as other refusals in `answers.rs` do. Default: check `unasked_kind` first in `bounded` and in `answer_text`'s path, before `file_read_bounded`, `compare_bounded`, `job_bounded` and the task read.
- `forge_answer`'s five arms return `Err(Problem::Unasked(..))` and `answer_text` turns that into the words with the tool. If threading the tool into `answer_text`'s `Problem` arm is awkward, build the words in `bounded` from `unasked_kind` and keep `forge_answer`'s arms returning the `Problem`; either way the words are the contract's.
- The note in `render_host`: `if let Some(reason) = temper_engine_smith::unasked_kind(answer)`. If `engine.notes` isn't reachable there, push it where the `HostDelivery::Render` arm is handled, and say so.
- No new test drives `render_host`: the note's line is the existing `Note::ReadRefused` format (`temper/crates/temper/src/host.rs`, `note_line`, and its test). If a test in `temper/crates/temper-service/src/tests/` already renders a `HostDelivery::Render` of a forge read and reads `engine.notes`, add one row there; otherwise say so in the report.

## Steps

1. In `answers.rs`, beside `answer_for_task`:

   ```rust
   /// The kind of a forge read's answer no agent read asks for, as the log's
   /// reason (temper/docs/design/domain/agent.md, 8).
   #[must_use]
   pub fn unasked_kind(answer: &CallAnswer) -> Option<&'static str> {
       // CallAnswer::ForgeRead(Ok(Runs { .. })) => Some("answer of kind Runs"), and Run,
       // Commits, Tree, Queued the same; every other answer None. List every
       // `api::Answer` variant (the strict subset: no `_`, no `matches!`).
   }
   ```

   Export it from `lib.rs` beside `answer_for_tool`. Add `Unasked(&'static str)` to `Problem` in `lib.rs` with a one-line doc comment.
2. The words, with `tool` the call's tool or `the read` when it is empty:
   `Forge read refused: <tool> got an <phrase>, which it never asks for. Nothing was read; read again.`
   — so `Forge read refused: read_file got an answer of kind Commits, which it never asks for. Nothing was read; read again.` Marked an error.
3. The rows, in the existing `root_call_answers_become_bounded_smith_text_or_error` (`src/tests.rs`), as a table over the five kinds (`Runs { runs: [], more: false }`, `Run(..)`, `Commits { commits: [], more: false }`, `Tree { entries: [], more: false }`, `Queued`) and three ways in: `answer(..)` (tool `the read`), `answer_for_tool(.., b"read_file", 4096)`, `answer_for_tool(.., b"read_compare", 4096)`. Each asserts `error()`, the exact text and `unasked_kind(..) == Some(phrase)`; one more row asserts `unasked_kind` of `ForgeRead(Ok(Answer::Commit(..)))` is `None`. Bite: restore the `unreachable!` for `Commits` once and see the row panic with *agent forge reads do not request Actions or trees*.
4. `render_host`'s note, as the contract has it.
5. The status entry (acceptance 4).
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-engine-smith -p temper-service --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-engine-smith --lib`; `heavy cargo nextest run -p temper-service --lib`; `heavy cargo check -p temper --tests` and `heavy cargo check -p temper-engine-domain-world --tests` for the dependents. Commit ("temper-engine-smith: refuse a forge answer of a kind no agent read asks for, so no answer stops serve"), push.

## Report

As the implementer's text says, with the rule broken for the bite and the failure seen, and which option you took for the words and the note.
