+++
id = "batch-options"
part = "change-batch"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-service/src/tools.rs", "temper/crates/temper-service/src/tests/tools.rs", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 40
+++

## Goal

A `change` call with `changes` refuses, as it is decoded and before any
change is made, a member given beside `changes` at the call's outer
level, naming it and saying it belongs in each change. Today the decoder
reads each change from its own object and drops the call's `landing`,
`producer`, `questions`, `llm` and the rest unheard, so a goal's batch
landed on main with CI required and its questions routed to none (the
request, item 1).

Acceptance:

- `temper_engine_smith::Problem` gains one variant,
  `Beside(&'static [u8])`, the member named, with a doc comment citing
  `domain/agent.md, 4.3.1`.
- `calls.rs`, the `b"change"` arm: with `changes` present, after the
  existing `words` check (which still answers `Range(b"changes")`), the
  first of `repository`, `name`, `landing`, `branch`, `merge`, `after`,
  `producer`, `llm`, `repairs`, `questions` present on the call's object,
  whatever its value, answers `Err(Problem::Beside(member))`, checked in
  that order (the declaration's), before any item is decoded. The single
  form and each change's own members decode as today.
- `temper-service/src/tools.rs`, `invalid`: `Problem::Beside(member)`
  renders as `host call member belongs in each change, not beside
  changes: <member>`, by the existing `(body, Some(member))` path.
  `answers.rs`' recorded-answer match takes the variant in its existing
  "cannot be rendered" arm.
- Failing rows first, in existing tables:
  - `change_refuses_words_with_changes_and_empty_members`
    (`temper-engine-smith/src/tests.rs`): one row per member above, each
    alone beside `"changes":[{"words":"a"}]` and answering
    `Problem::Beside(member)`; the request's call,
    `{"landing":"goal","producer":2,"questions":"foreman","llm":{"producer":{"main":{"model":"m","effort":"high"}}},"changes":[{"name":"a","words":"a"},{"name":"b","words":"b"},{"name":"c","words":"c"},{"name":"d","words":"d"}]}`,
    answering `Beside(b"landing")`; `{"questions":"up","producer":2,"changes":[{"words":"a"}]}`
    answering `Beside(b"producer")` (the declaration's order, not the
    input's); `{"words":"a","landing":"goal","changes":[{"words":"b"}]}`
    still answering `Range(b"changes")`.
  - `a_flat_forge_read_error_names_its_missing_member`
    (`temper-service/src/tests/tools.rs`): the row `(b"change",
    br#"{"landing":"goal","changes":[{"words":"a"}]}"#, b"host call
    member belongs in each change, not beside changes: landing")`.
  - Each new row fails before the fix (the call decodes, or renders
    otherwise) and passes after; break the order check once and see the
    order row fail.
- `change_decodes_several_changes_in_order` and the schema golden
  `change_offers_its_plan_members_only_with_control` pass unchanged: the
  declaration's words and bytes stay as they are (no figure moves).
- Close the entry: remove the **Not yet** 4.3.1 entry this plan added
  to `temper/docs/design/domain/agent.status.md` ("a call with `changes`
  and another member beside it is decoded, not refused"), its lines
  alone. This departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's, and the design
  section it closes is written.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1: the `changes` bullet
  (its last paragraph is the rule this task builds) and the members
  listed before and after it.
- `temper/docs/design/domain/root.md`, 5.2, *An undecodable call decides
  nothing*: the refusal is a decode error, relayed with no commit and no
  record.
- `temper/docs/plan/goal-call-faults/notes/common.md`.

## Owns and may change

- Owns the files in `files`; in `agent.status.md`, the one entry above.
- Compiler-forced arms for the new variant in any temper file that
  matches `Problem` exhaustively; name each. `temper/tests/system/src/composed/review.rs`
  matches `Range` and panics on the rest: leave it.
- Not yours: the `change` declaration's schema text (`PLAN_CHANGE` in
  `calls.rs`) and its golden; `temper/docs/help/change.md`; an unknown
  member's handling (an unknown key stays accepted in both forms, as
  today).
- Pre-approved: if `Value::field` reports a present `null` as absent,
  keep that; a `null` member beside `changes` is then not refused, and
  the report says so. If a test outside `temper-engine-smith` and
  `temper-service` asserts that a batch with outer members decodes, it
  is a reader: change its input to give the member in each change, and
  name it.

## Steps

1. The rows above; run both tests and see them fail
   (`heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(=tests::change_refuses_words_with_changes_and_empty_members)'`;
   `heavy cargo nextest run -p temper-service --lib -E
   'test(=tests::tools::a_flat_forge_read_error_names_its_missing_member)'`;
   read the count, and find the exact path with `--list` if either
   selects nothing). Commit.
2. The variant, the decoder's check, the rendering and the
   `answers.rs` arm; both tests pass; break the order once, see its row
   fail, restore.
3. The status entry closed.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-smith -p temper-service --all-targets -- -D
   warnings`; `heavy cargo nextest run -p temper-engine-smith -p
   temper-service`; `heavy cargo check -p temper-engine-domain-world -p
   temper-system-world -p temper-client-world -p temper -p
   temper-playground --tests`.
5. Commit and push.

## Report

As the implementer's text says, with the failure each new row showed
before the fix.
