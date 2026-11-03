+++
id = "change-jig-check"
part = "change-jig"
kind = "interface"
needs = []
files = ["jig/crates/jig-domain-tasks/src/llm.rs", "jig/tests/tasks/tests/suite/llm.rs", "jig/tests/core/tests/suite/charters.rs", "jig/tests/core/tests/suite/procedure_steps.rs", "jig/docs/design/domain/tasks.status.md"]
effort = "medium"
minutes = 70
+++

# change-jig-check: a batch's LLM choice against its charter's models, from every creator, and alone

## Goal

jig's batch check refuses an agent task whose LLM choice names a model, main or sub-agent, that
its charter's models at the newest revision do not allow, with `LlmFault::Charter`, whoever
makes the batch (contract C5). The tasks child already does so in `batch.rs` through
`llm::check`; this task proves it for every creator through the core, fixes any creator path
that does not reach it, and offers the same check alone, for a charter and an authority's
models, to a root that checks a choice before the batch carrying it is asked for (temper's
`change`, task `change-root`).

Acceptance:

- In `jig/crates/jig-domain-tasks/src/llm.rs`:

  ```rust
  impl Domain {
      /// Check `choice` for an agent task of `charter` within `models`, at the newest
      /// revision, as a batch checks it (domain/tasks.md, section 4): each model configured,
      /// named once among the sub-agents, each effort one its model declares, each model one
      /// the charter allows and `models` allows. A refusal names the first model that fails.
      pub fn check_llm(&self, charter: u32, models: Models, choice: &LlmChoice) -> Result<(), Refusal>
  }
  ```

  `Domain`, `Models`, `LlmChoice` and `Refusal` are the crate's, already exported. It shares
  its body with `check` (the batch's and the amendment's): `check` keeps its signature and
  behaviour and calls a common private function for an agent's charter, so the two can never
  disagree. A charter the revision does not offer refuses the main model with
  `LlmFault::Charter`, as `check` does today. It reads no account (the core checks a
  preference, `jig-domain-core/src/runs/account_preference.rs`).
- Stories, each one thread and one loop, a seed replaying:
  - tasks world: a task's batch (`Party::Task`, as the delegate in
    `a_choice_is_kept_through_a_restart_amended_and_given_to_a_delegate`) naming, for a charter-1
    task, a sub-agent of model 3 (charter 2's alone) is refused `Llm { model: 3, why: Charter
    }` and writes nothing; `check_llm` gives, for each agent row of the table in
    `a_choice_is_refused_naming_its_first_failing_model_and_why`, the same `Refusal::Llm` the
    batch gives, and `Ok(())` for a fitting choice.
  - core world, on `jig_test_profile::two_charters` (charter 1 allows `scripted`, model 0;
    charter 2 `research`, model 1): a run's delegate of charter 1 whose choice names model 1 is
    refused naming model 1 and `Charter`; a party's `Create` of a charter-1 task naming model 1
    is refused with `people::LlmWhy::Charter`; a procedure's step whose agent member of charter
    1 names model 1 is refused (the step's refusal carrying `Refusal::Llm { model: 1, why:
    Charter }`), the procedure told so as a refused step is, and nothing of the batch made.
- Remove the **Not yet** entry for section 4 from `jig/docs/design/domain/tasks.status.md`.

## Inputs

- `jig/docs/design/domain/tasks.md`, section 3 ("Its LLM choice") and section 4 (the bullet
  "Checked whole", its first item, written for this task).
- `jig/docs/design/protocol/configuration.md`, 3.1 and `jig/docs/design/domain/engine.md`,
  17.4: how a template's and a move's choice are checked against the charter (unchanged).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C5.

## Owns and may change

Owns the files in the header.

- `llm.rs`: the method, the shared private function; no change to `LlmFault`, `Refusal` or
  any stored or wire type.
- The tests: new stories beside the existing ones in `jig/tests/tasks/tests/suite/llm.rs` and
  `jig/tests/core/tests/suite/charters.rs` (which already builds `two_charters` worlds, drives
  a run's delegate with `jig_core_world::fixtures::delegate` and a party's create with
  `fixtures::ask_as`). The procedure's story goes in `procedure_steps.rs` if its world can take
  `two_charters`' configuration through `Setup`; if not (default when unsure), in
  `charters.rs`, holding procedures and deciding the step as `procedure_steps.rs`'s `decide`
  does.
- Options: if a creator path (a run's delegation, a party's create, a procedure's step, an
  accepted proposal's batch, a recurrence's occurrence) does not reach the tasks child's
  `llm::check` with the newest revision, fix that path in `jig-domain-core` with the narrowest
  hunk, add it to your files, and say so in the report. If every path already reaches it, the
  stories are the proof; change no core code.
- Not yours: `jig-domain-core`'s `resolve_llm` beyond such a fix, the store, the wire,
  `jig-protocol-tools`' rendering of `LlmFault::Charter` (it exists).

## Steps

1. The method and the shared function in `llm.rs`, with a module doc line naming the check
   alone (domain/tasks.md, section 4).
2. The stories above.
3. Narrow: `heavy cargo nextest run -p jig-tasks-world --test suite -E 'test(/llm::/)'`,
   `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/charters::/) |
   test(/procedure_steps::/)'` (check each count includes the new stories); then
   `heavy cargo nextest run -p jig-domain-tasks --lib`.
4. `cargo fmt`; `heavy cargo clippy -p jig-domain-tasks -p jig-tasks-world -p jig-core-world
   --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit ("Check an LLM choice against its charter alone, from every creator")
   and push.

## Report

As the implementer's text says, naming each creator path and the story that covers it, and any
core path you had to fix.
