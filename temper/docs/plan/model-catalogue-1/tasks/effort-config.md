+++
id = "effort-config"
part = "effort-config"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-protocol-config/src/llm.rs",
  "jig/crates/jig-protocol-config/src/tests/llm.rs",
  "jig/crates/jig-protocol-config/src/members.rs",
  "jig/crates/jig-protocol-config/src/tests/members.rs",
  "temper/docs/help/reference/configuration-engine.md",
  "jig/docs/design/protocol/configuration.status.md",
  "jig/docs/design/domain/tasks.status.md",
  "jig/crates/jig-domain-core/src/runs/state.rs",
  "jig/crates/jig-domain-core/src/runs/llm.rs",
  "jig/crates/jig-domain-core/src/tests/runs/default_llm.rs",
  "jig/crates/jig-domain-tasks/src/llm.rs",
]
effort = "high"
minutes = 50
+++

# A model names its default effort apart from its list

## Goal

Today a model's first effort is the one a task given no LLM choice takes, which would draw the picker's
scale out of order. Let `engine.llm.models[].effort` name it, so that `"efforts"` runs from low to high
whichever is the default. A document that leaves the member out keeps today's meaning, the first.

Acceptance:

1. `"efforts": ["low", "medium", "high"], "effort": "medium"` parses to `Model.effort` `Some("medium")`.
2. An `effort` not among `efforts`, or given on a model declaring none, is refused at
   `engine.llm.models[N].effort` with `What::Refers`; an empty one with `What::Form`.
3. With the member left out, `Model.effort` stays `None` and every existing document parses to what it
   did, byte for byte in its model values.
4. `Model::default_effort(&self) -> Option<&[u8]>`, in `jig-domain-core`, is its `effort`, else the first of its
   `efforts`, else none.
5. A task given no choice, on a model with `efforts` `[low, medium, high]` and `effort` `medium`: its
   `Core::next_llm` main names index 1, and a run's `chosen_policy` has `model.effort` `Some("medium")`. Its
   sub-agent models each take their own default the same way.
6. A task given a choice keeps its own effort (index 0 stays `low`).

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`.
- `jig/docs/design/protocol/configuration.md`, 3 (the `llm` row).
- `jig/docs/design/domain/tasks.md`, 3 (*Its LLM choice*).

## Owns and may change

- Owns `jig/crates/jig-protocol-config/src/llm.rs` (its `model` and `price` functions) and
  `src/tests/llm.rs`; `jig/crates/jig-domain-core/src/runs/state.rs` (the `Model` type's `effort` comment
  and the new method); `runs/llm.rs`'s `default_effort`; `src/tests/runs/default_llm.rs`.
- May change `jig/crates/jig-protocol-config/src/members.rs` (the `model` shape's `effort` row, beside
  `efforts`), `src/tests/members.rs` (its `EXTRAS` example) and
  `temper/docs/help/reference/configuration-engine.md` (that row's rendered line, nothing else).
- May change `jig/crates/jig-domain-tasks/src/llm.rs`: the comment above `first_effort`'s call only. tasks
  keeps a count of each model's efforts, and `first_effort` stays the fallback that `default_effort` then
  overrides: don't teach tasks the names.
- Removes the two **Not yet** entries this closes: `jig/docs/design/protocol/configuration.status.md`, 3, "a
  model's `effort`", and `jig/docs/design/domain/tasks.status.md`, 3, "a model's default effort".
- Options, each pre-approved:
  - `Model.effort` already exists, is stored in a charter's value and is held to `efforts` by
    `settings/check.rs`'s `model_valid` and `settings/charters.rs`'s `declares`: use it, and change no store
    or wire codec. If a store codec would need changing, stop and say so in the report as a question.
  - `RunPolicy` models clone the configured `Model`, so a charter carries the default with no new code in
    `jig-protocol-config/src/charters.rs` or temper's stock. If a reader's test pins a policy's
    `effort: None` for a model that now names one, edit that test's literal.

## Steps

1. `llm.rs`, `model`: read the optional `effort` with `read::text`, bounded as `model_efforts` bounds a name;
   empty is `What::Form`; after `efforts` is parsed, a name not among them is `What::Refers` at
   `path.member(b"effort")`. Set `model.effort`.
2. `state.rs`: the `effort` comment says what it now holds: on a configured model and a charter's, the
   default a task given no choice takes, or `None` for the first of `efforts`; on a run's charter, its task's
   choice's. Add `default_effort`, in the strict subset.
3. `runs/llm.rs`, `default_effort`: its comment names the model's default. Its body stands: it reads the
   charter's model's `effort`.
4. Tests in `tests/llm.rs` beside `a_model_takes_the_efforts_it_names_in_order`, and a story in
   `tests/runs/default_llm.rs` on a charter whose model declares `[low, medium, high]` with `effort` `medium`.
   Break each rule once to see it bite.
5. Narrow tests, then the implementer's text's closing checks over `jig-protocol-config`, `jig-domain-core`
   and `jig-domain-tasks`, with `heavy cargo check -p temper-service --tests`. Commit and push.

## Report

As the implementer's text says; each rule broken and the failure seen.

## Answers

2026-11-06. May this task add the model's `effort` row to `jig-protocol-config/src/members.rs`, outside its
files? Yes: `read::Object::member` asserts every member it reads has a row, and acceptance 1 can't hold
without one, so the row is this task's. Resume from the pushed `a17ebcb083` on `model-catalogue-1/effort-config`.

- **The row:** `shape: "model"`, `name: "effort"`, `kind: "text"`, `bound: "32 bytes"`, `required: false`,
  `gone: None`, right after `efforts`, with a line in `each_line_is_within_its_bound`'s form, one sentence
  under 160 bytes: "Names the effort a task given no choice takes, the first of the efforts when absent."
- **Its example:** `every_row_is_a_member_its_decoder_takes` needs one. In `tests/members.rs`'s `EXTRAS`, add
  `("llm.models.0", "effort", br#""medium""#)` right after the `efforts` entry, which declares `medium`.
- **The rendered page:** a row changes `temper/docs/help/reference/configuration-engine.md`, which
  `temper`'s `the_pages_configuration_are_what_the_tables_render` holds equal to its table. Take the new line
  from that test's output (or `SKEIN_CODEGEN_WRITE=1`), and run that test as well as the closing checks.
  This task's init writes no `effort`, so the line reads "init leaves it out".
- **The two status entries** named under `Owns and may change` are now in `files`.
- Anything else outside `files`, stop and ask, as before. No store or wire codec changes with this.
