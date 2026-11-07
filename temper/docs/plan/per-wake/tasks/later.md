+++
id = "later"
part = "later"
kind = "feature"
needs = ["offering", "parts-core"]
files = ["jig/crates/jig-domain-tasks/src/llm.rs", "jig/crates/jig-domain-tasks/src/stored/fit.rs", "jig/crates/jig-domain-tasks/src/tests/startup/detail.rs", "jig/tests/tasks/tests/suite/llm.rs", "jig/crates/jig-domain-core/src/runs/llm.rs", "jig/crates/jig-domain-core/src/settings/check.rs", "jig/crates/jig-domain-core/src/tests/runs/later.rs", "jig/crates/jig-domain-core/src/tests/runs.rs", "jig/crates/jig-domain-core/src/tests/runs/run_choice.rs", "jig/crates/jig-domain-core/src/tests/settings/templates.rs", "jig/crates/jig-domain-people/src/llm.rs", "jig/crates/jig-domain-people/src/domain/start/fit.rs", "jig/crates/jig-domain-people/src/tests/start.rs", "jig/docs/design/domain/tasks.status.md", "jig/docs/design/domain/engine.status.md", "jig/docs/design/client/domain/contract.status.md"]
effort = "high"
minutes = 140
+++

# per-wake: a task's later choice, checked as the first is and taken by every run after its first activation

## Goal

`jig/docs/design/domain/tasks.md`, section 3's `later`: checked as the first choice is at a
batch, an amendment, a template's check and a restore, and refused under a charter whose
runs resume; taken by every run whose task's reasons hold no `first` (`contracts.md`,
section 5); and shown on the task's document as its next run's choice. Its account follows:
a `later` on another LLM provider chooses the kept account again at its first later run.

Acceptance:

- **Checks, tasks child** (`src/llm.rs`, `check`/`check_agent`, which a batch and an
  amendment's `control::check_llm` call): a `later`'s main and sub-agents are checked as the
  first choice's are, its sub-agents' repeats among themselves refused as `Repeated`; a
  `later` under a charter version whose `CharterModels.resumes` is true is refused as
  `Refusal::Llm { model: later.main.model, why: LlmFault::Charter }`; `valid_shape` holds a
  `later`'s sub-agents within `MODELS`. A restore (`src/stored/fit.rs`, `choice`) fits a
  `later`'s models and efforts as the first choice's, with the same `RestoreCause`.
- **Checks, people** (`jig-domain-people`): `llm_bytes` counts a `later`'s sub-agents;
  `valid` holds their bound; the start's fit (`src/domain/start/fit.rs`, `choice`) fits a
  `later` as the first choice.
- **Checks, core**: a template's retained choice (`src/settings/check.rs`, `template_fault`
  and `choice_valid`) fits its `later` as its first choice, and a `later` under a charter
  that resumes is refused there as a template whose choice doesn't fit is today; at
  configuration, a policy change and a restore alike. The translation of a party's choice
  into the task's (`crate::runs::llm::choice`, which `routing/party/create.rs` calls)
  carries `later`.
- **The run's choice** (`src/runs/llm.rs`, `task_llm`): a task given a choice with a `later`
  whose `reasons` hold no `FIRST`, under a charter at the run's revision that doesn't
  resume, takes `LlmChoice { account: <the choice's>, main: later.main, subagents:
  later.subagents, later: None }`; otherwise its choice as it is. `next_llm`, and so
  `chosen_policy` and the run's check, follow it; `document_llm` gives the same, so a task's
  document shows its next run's choice, with the `later` beside the first while the task's
  reasons hold `first`. A task given none takes its charter's models as today, with no
  `later`.
- **An amendment replaces the choice whole** (`jig/docs/design/domain/tasks.md`, section 7):
  confirm it does, and say so in the report; if an amendment giving a choice without `later`
  would leave an older `later` on the task, make it replace the whole choice and say so.
- **Tests, failing first:**
  - rows in `jig/tests/tasks/tests/suite/llm.rs`'s refusal table `(Executor, u64,
    LlmChoice, u8, LlmFault)`: a `later` whose main is unconfigured, has an effort its model
    doesn't declare, isn't the charter's, is beyond the authority; a `later` whose
    sub-agents repeat; a `later` under a charter that resumes (`Charter`, naming its main);
    and, beside the amendment assertions there, an amendment giving a choice without `later`
    leaves the task none;
  - rows in `src/tests/startup/detail.rs`'s table: a restored `later` naming a model gone,
    and an effort gone, each refused with the first choice's cause;
  - `jig-domain-core/src/tests/runs/later.rs`, one table `(name, given choice, reasons,
    charter resumes, expected next-run choice, expected document choice)`: `FIRST` with a
    `later` (the first, the whole choice); `EMPTY` with a `later` (the `later`, flattened);
    `EMPTY` without (the first); `EMPTY` with a `later` under a charter now resuming (the
    first); a task given none (its charter's default, both);
  - one row or test beside `src/tests/runs/run_choice.rs`'s: a task whose kept account is
    of the first choice's provider, its `later` on another provider, at its first later run
    chooses an account of the later's provider and the claim keeps it;
  - rows in `src/tests/settings/templates.rs`: a template whose `later` names a model its
    charter doesn't allow, and one with a `later` under a charter that resumes, each refused
    as a template whose choice doesn't fit is; and one in `jig-domain-people`'s start tests
    for a `later` that no longer fits.
  Break each rule by hand (take the first choice always, skip the `later`'s check), see its
  rows fail, restore.
- Delete these sentences, and nothing else (a departure from the implementer's text, which
  closes `docs/design/`: each status must say what is built when this lands):
  `jig/docs/design/domain/tasks.status.md`, "3, 7: a task's LLM choice has no `later`
  (`jig-domain-tasks/src/llm.rs`)."; `jig/docs/design/domain/engine.status.md`, "7.1: a
  run's check takes its task's first choice whatever its reasons (...)."; and
  `jig/docs/design/client/domain/contract.status.md`, "5.1: a task's document shows its
  first choice whatever its reasons (...)."; each bullet's lead with it where the bullet is
  left empty.
- This changes what a shared item means (what a run and a task's document take): run the
  focused tests of `jig-domain-tasks`, `jig-domain-core`, `jig-domain-people`, the tasks,
  people and core worlds, `jig-protocol-engine-clients` and `jig-client-protocol-link`, and
  `heavy cargo check -p temper-engine-domain -p temper-engine-smith --tests`.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1, 2 and 5.
- `jig/docs/design/domain/tasks.md`, section 3 ("Its LLM choice", "Its account"), section 7
  ("A new LLM choice is for the next run") and 8.6 (a run's reasons);
  `jig/docs/design/domain/engine.md`, 7.1, step 1 and 7.3 (a run's choice names no `later`);
  `jig/docs/design/protocol/configuration.md`, 3.1 (a template's `later`);
  `jig/docs/design/client/domain/contract.md`, 5.1 (a task's LLM choice).

## Owns and may change

- Owns the files in the header; the mod line for `src/tests/runs/later.rs`.
- Readers: `jig/crates/jig-domain-tasks/src/control.rs` and `batch.rs`, only if their call of
  `llm::check` must pass something more; temper's change `llm` (`temper/crates/temper-engine-domain/src/route/forge_route/change_llm.rs`)
  stays as the surface left it, its `later` `None`.
- Pre-approved: if `CharterModels.resumes` isn't kept for every charter version a check can
  name, fill it where the core builds the version list and say where; if the run's check
  takes its choice somewhere `task_llm` doesn't reach, route that through `next_llm` and say
  where.
- Not yours: the tasks child's offering and reasons (the offering task's, merged), the
  core's parts (merged), the formats, the run's `delegate` and `amend` tools, which give no
  `later` (`jig/docs/design/domain/engine.md`, 7.3).

## Steps

1. The rows above, failing.
2. The checks; the people's; the template's; the run's choice and the document's.
3. The rows pass; the bites above.
4. The status sentences.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy
   -p jig-domain-tasks -p jig-domain-core -p jig-domain-people --all-targets -- -D
   warnings`; the focused tests named above; the citations check.
6. Commit (why: a task's runs after its first activation take a cheaper choice with no
   amendment by hand); push your branch.

## Report

As the implementer's text says, and also: whether an amendment already replaced the choice
whole; where the run's check takes its choice.
