+++
id = "change-call"
part = "change"
kind = "interface"
needs = []
files = ["temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/route/host_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs", "temper/tests/engine/domain/tests/suite/forge_routes/goal_tool.rs", "temper/tests/engine/domain/tests/suite/forge_routes/b2_tree.rs", "temper/tests/engine/domain/tests/suite/settings/b2.rs"]
effort = "high"
minutes = 75
+++

# change-call: `change`'s parsed shape, with a plan's members

## Goal

`change` decodes what a plan decides of each change (contract C4): `landing`, `name`,
`after`, and several changes in one call as `changes`, into one typed shape the root builds
on. Its declaration offers those members only to a run whose authority holds jig's control
family; every other run keeps today's schema, byte for byte. The root's handling stays as
today for a call of one change naming none of the new members; any other call is refused,
until task `change-batch` builds them.

Acceptance:

- `temper_engine_domain::Tool::Change { changes: Box<[ChangeCall]> }`, with, in
  `boundary.rs` (exported as `Tool` is):

  ```rust
  /// One change of a `change` call (domain/agent.md, 4.3.1).
  pub struct ChangeCall {
      pub words: Box<[u8]>,
      pub repository: Option<Box<[u8]>>,
      pub name: Option<Box<[u8]>>,
      pub landing: Option<ChangeLanding>,
      pub after: Box<[ChangeAfter]>,
  }
  pub enum ChangeLanding { Goal, Branch(Box<[u8]>) }
  pub enum ChangeAfter { Task(u64), Name(Box<[u8]>) }
  ```

  with the derives `Tool`'s members have (`Clone, PartialEq, Eq, Debug` or as the file has
  them), and `impl ChangeCall { pub fn words(words: Box<[u8]>, repository: Option<Box<[u8]>>)
  -> Self }`, the one-change call with no plan member, for the root and the tests.
- `temper-engine-smith`'s `call_configured`, `b"change"`:
  - `words` (and optionally `repository`, `name`, `landing`, `after`) decodes to one
    `ChangeCall`; `changes`, an array of 1 or more objects each of those members (never a
    nested `changes`), decodes to one `ChangeCall` per item in order. Both `words` and
    `changes` is `Problem::Range(b"changes")`; neither is `Problem::Missing(b"words")`, as
    today; an empty `changes` is `Problem::Range(b"changes")`; more items than fit a `List`
    is `Problem::TooLarge`.
  - Each member as today's `words` and `repository` are checked (non-empty `words`,
    `owner/name`), and: `name` a non-empty string of at most 64 bytes
    (`Problem::Range(b"name")`); `landing` a non-empty string, `"goal"` decoding to
    `ChangeLanding::Goal` and any other to `Branch` (`Problem::Range(b"landing")` when empty);
    `after` an array whose items are each an integer from 1 (`ChangeAfter::Task`) or a
    non-empty string (`ChangeAfter::Name`), else `Problem::Type(b"after")` or
    `Problem::Range(b"after")`. Uniqueness of names and what they name are the root's.
- Declaration: `tools_for(timeout, authority)` offers the plan schema below for `change`
  where `authority.0 & ToolFamilies::JIG.control.0 != 0`, and today's schema otherwise;
  `tools(timeout)`, the complete registry, offers the plan schema. Today's `change`
  description and schema stay byte for byte for a run without control.
- The plan schema, the description unchanged:

  ```json
  {"type":"object","properties":{
   "words":{"type":"string","description":"What to change: the files, and the exact text where it matters. It is all the change's coder reads."},
   "repository":{"type":"string","description":"The repository, as owner/name; leave it out when the project has one you may write."},
   "name":{"type":"string","description":"Its name in this call, for a later change's after."},
   "landing":{"type":"string","description":"goal: your own goal branch; or a branch's name; left out, the default branch."},
   "after":{"type":"array","description":"What it lands after: your live delegates' task numbers, or names given earlier in this call.","items":{"oneOf":[{"type":"integer","minimum":1},{"type":"string"}]}},
   "changes":{"type":"array","description":"Instead of words: several changes as one batch, each with the members above.","items":{"type":"object"}}}}
  ```

  written on one line in `calls.rs` as the other schemas are.
- The root (`route/tools.rs`): a call of one `ChangeCall` whose `name`, `landing` are `None`
  and `after` is empty goes to `forge_route::chat_change` exactly as today; any other call is
  answered `CallPart::DelegationRefused(tasks::Problem::new(None, tasks::Refusal::Spec))`,
  as the refused expansions there are. `route/host_route.rs`'s flight render takes the first
  change's `repository`.
- Every story that names `Tool::Change` passes, edited only to the new shape.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (the paragraph "What a plan decides" and its
  bullets) and 4.3 (the host part's budget: a tool's bytes offered only where a family gates
  it).
- `temper/docs/design/domain/agent.status.md`: its **Not yet** entry for 4.3.1 stays (task
  `change-batch` removes it); this task edits no design file.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C4 and its line
  under "Names the families write here".

## Owns and may change

Owns the files in the header.

- `calls.rs`: the decode, the second `change` spec (a `PLAN_CHANGE` constant beside `SPECS`,
  swapped in by `offered_tools`, which gains a `plan: bool`), and `tools_for`'s family test.
- `boundary.rs`: the types above and the `Tool::Change` member; keep the hunk to them.
- `route/tools.rs` and `route/host_route.rs`: the destructuring sites, as above, no other hunk.
- Tests: `temper-engine-smith/src/tests.rs` (its `change` decode tests, line 1379 and on, and
  the declaration table near line 398), and the stories' literals in the header's four test
  files, each `engine::Tool::Change { words, repository }` becoming `engine::Tool::Change {
  changes: Box::new([engine::ChangeCall::words(words, repository)]) }` and nothing else.
- Options: if a compiler-forced arm names `Tool::Change` elsewhere (`translate.rs`'s arms use
  `{ .. }` and need none), edit it only for the shape. If `rg 'Tool::Change'
  temper/crates temper/tests` finds a reader not listed, edit it the same way and name it.
  If smith's schema check refuses `oneOf` in `items`, drop `items` from `after` (keeping its
  description) and say so.

## Steps

1. Types, decode, declaration, the root's sites, the literals.
2. Tests in `temper/crates/temper-engine-smith/src/tests.rs`, beside the existing `change`
   ones:
   - `change_decodes_a_plan_member_each`: `{"words":"a","name":"x","landing":"goal","after":[4,"w"]}`
     gives one `ChangeCall` with `name` `x`, `Goal`, `[Task(4), Name("w")]`; `landing`
     `"main"` gives `Branch("main")`.
   - `change_decodes_several_changes_in_order`: `{"changes":[{"words":"a","name":"a"},{"words":"b","after":["a"]}]}`
     gives two, in order.
   - `change_refuses_words_with_changes_and_empty_members`: both `words` and `changes`,
     `changes: []`, `name: ""`, a 65-byte name, `landing: ""`, `after: [0]`, `after: [true]`,
     each with its `Problem` above.
   - `change_offers_its_plan_members_only_with_control`: `tools_for` with the stock ceiling's
     tools (1,539) declares today's `change` schema byte for byte; with `1_539 | 4` the plan
     schema.
3. Narrow: `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/change_/)'` (check
   the count includes the four). Then `heavy cargo nextest run -p temper-engine-smith`,
   `heavy cargo nextest run -p temper-engine-domain --lib`, `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/) | test(/settings::b2/)'`,
   and, since the declaration is measured there, `heavy cargo nextest run -p temper-service
   --lib -E 'test(/stock_brief/)'`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-smith -p temper-engine-domain -p temper-engine-domain-world --all-targets --
   -D warnings`; `heavy cargo check -p temper-service --tests`. Commit ("Decode a plan's
   members of change") and push.

## Report

As the implementer's text says, with the plan schema's byte count.
