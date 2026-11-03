+++
id = "questions-call"
part = "call"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs"]
effort = "medium"
minutes = 45
+++

# questions-call: `change`'s `questions` member, decoded

## Goal

`change` decodes each change's `questions` member (contract C6) into `ChangeCall.questions`,
and its plan schema offers it. The root counts it as a plan member, so a run without jig's
control family naming it is refused as for `landing`; and, until task `relay-up` builds the
relay, it refuses a call naming `"foreman"` or `"up"`, saying so. A call naming `"none"`, or
naming none, goes on as today.

Acceptance:

- `temper_engine_domain::ChangeQuestions { None, Foreman, Up }` (derives as
  `ChangeLanding`'s) and `ChangeCall.questions: Option<ChangeQuestions>` after `after`, in
  `boundary.rs`, exported as `ChangeCall` is; `ChangeCall::words` sets `None`.
- `temper-engine-smith`'s `fn change` decodes `questions` in a change's object, the single
  one or each of `changes`: `"none"`, `"foreman"`, `"up"`; any other string
  `Problem::Range(b"questions")`, any other type `Problem::Type(b"questions")`.
- `PLAN_CHANGE`'s schema gains, after `after`, `"questions":{"type":"string","enum":["none","foreman","up"],"description":"How its producer's questions go: foreman, settled first by a foreman from the words and the design; up, straight to you; none, it asks no one. Left out, the project's setting."}`;
  the plain `change` spec (a run without control) stays byte for byte.
- The root, `route/forge_route.rs`'s `change_batch`: the `plan` test counts
  `questions.is_some()` beside `name`, `landing` and `after`, and its refusal's words read
  "change's landing, name, after, questions and changes are a plan's: ..." (the rest as
  today). Then, before any delegate is built, a change whose `questions` is `Foreman` or
  `Up` refuses the call with `ChangeRefused::Words(b"change's questions foreman and up are
  not offered yet.")`. (A stated departure from the header's files: this one hunk of
  `change_batch` is this task's; task `relay-surface` changes other functions of the same
  file in the same wave.)

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 ("What a plan decides", the `questions`
  bullet and "Who may set them").
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C6.
- `temper/docs/design/domain/agent.status.md`'s entry for 4.3.1's `questions` stays (task
  `relay-up` removes it); this task edits no design file.

## Owns and may change

Owns the files in the header, and the `change_batch` hunk above in
`temper/crates/temper-engine-domain/src/route/forge_route.rs`.

- Readers that build a `ChangeCall` by its fields, edited only to add `questions: None`:
  `rg 'ChangeCall \{' temper` lists them (the domain world's `forge_routes` suites, the
  root's tests); name each.
- If the plan schema's byte count is measured by a test (`temper-service`'s
  `stock_brief`), update the measured figure there and say so; it is the declaration's
  bytes, not a budget.
- If smith's schema check refuses `enum` in a property, drop `enum` (keeping the
  description) and say so.

## Steps

1. Type, decode, schema, the root's two hunks, the literals.
2. Tests:
   - in `temper-engine-smith/src/tests.rs`, beside `change_decodes_a_plan_member_each`:
     `change_decodes_its_questions` (`"foreman"`, `"up"`, `"none"` each in a single change
     and in one of `changes`); `change_refuses_unknown_questions` (`"ask"` is
     `Range(b"questions")`, `true` is `Type(b"questions")`); the plan declaration's test
     (`change_offers_its_plan_members_only_with_control`) still passes, the plain schema
     unchanged.
   - in `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs`:
     `questions_are_a_plans_member` (a run without control naming `"none"` is refused with
     the words above); `questions_foreman_is_not_offered_yet` (a run with control naming
     `"foreman"` is refused with those words, and nothing is delegated);
     `questions_none_makes_the_change_as_today`.
3. Narrow: `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/change_/)'`
   (check the count), then `heavy cargo nextest run -p temper-engine-domain-world --test
   suite -E 'test(/forge_routes::change_tool/)'`, `heavy cargo nextest run -p
   temper-engine-domain --lib`, and `heavy cargo nextest run -p temper-service --lib -E
   'test(/stock_brief/)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-smith -p temper-engine-domain -p temper-engine-domain-world --all-targets --
   -D warnings`; `heavy cargo check -p temper-service --tests`. Commit ("Decode change's
   questions member") and push.

## Report

As the implementer's text says, with the plan schema's new byte count.
