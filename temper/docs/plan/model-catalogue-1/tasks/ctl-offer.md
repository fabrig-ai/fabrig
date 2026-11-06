+++
id = "ctl-offer"
part = "ctl-offer"
kind = "feature"
needs = ["offer-surface"]
files = [
  "temper/crates/temper-client-tools/src/jig/runner/acts/choice.rs",
  "temper/crates/temper-client-tools/src/jig/runner/acts.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/accounts.rs",
  "temper/crates/temper-client-tools/src/jig/render/document/account_lines.rs",
  "temper/crates/temper-client-tools/src/tests/jig/choices.rs",
  "temper/crates/temper-client-tools/src/tests/jig/accounts.rs",
  "temper/crates/temper-client-tools/src/tests/jig/amend_choice.rs",
  "temper/docs/help/account.md",
]
effort = "high"
minutes = 70
+++

# `temper ctl` lists the offer and takes a model's default effort

## Goal

Stock models are about to declare efforts, and `--model X` with none then fails `effort required: X`: every walk
that starts a chat on a named model would go red. Take the model's default instead. `model list` prints the offer
with its new facts, and every provider's name, so a provider with no account shows no model of its.

Acceptance:

1. `chat start --model X` with no `--effort`: the task's choice names X at the row's `effort`. With `--effort E`, E,
   checked as now. `--sub-agent X` with no `:EFFORT` the same. A model declaring none: effort none.
2. `--model X` for a name the offer lacks: nothing is sent, exit 1, `model not offered: X`; the same for `--sub-agent`,
   `task amend` and `goal start`.
3. `model list [--project P]`: a `providers` row for every provider in `me.providers`, in order, each with `models`
   of the offer alone: `name`, `shown`, `window`, `output`, `description`, `prices`, `efforts`, `effort`, `accounts`,
   `picker`. A provider with no model: its name and `"models":[]`. Its text form says the same in lines.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*The client*).
- `temper/docs/design/client/tools.md`, the tools' table (`model list`) and *The task's model*.

## Owns and may change

- Owns its `files`.
- **Departure from the implementer's text:** `temper/docs/help/account.md`'s bullet **A model** may change, keeping the
  page's bound: "`model list` prints each project's offer, what an account can run now, by provider." Nothing else
  of `docs/help/`.
- Readers: `temper/crates/temper-client-tools/src/tests/jig/choices.rs`'s `chat_requires_an_effort_for_a_model_with_efforts`
  becomes the default's story; `tests/jig/choices.rs`'s `unknown model` pin becomes `model not offered`;
  `tests/help.rs`'s absences hold. `temper/tests/client/tests/suite/account_tools.rs` and `amend_tools.rs` run as they are;
  fix their expected words only.
- `model list`'s digest moves with its bytes: nothing pins the value but its own test.
- Removes `temper/docs/design/client/tools.status.md`'s **Not yet** 5.5 about `model list`.
- Options, pre-approved:
  - `Failed::EffortRequired` falls unreachable, the wire giving a default wherever there are efforts: remove it and its arm.
  - The scripted provider stays a row with no models outside the playground: ctl doesn't read the playground. Say so
    in the report, and add nothing.

## Steps

1. `acts/choice.rs`: `llm` and `sub_agents`, `model.effort.clone()` where none is named. `Failed::NotOffered { name }`
   beside `Unknown`, printed in `acts.rs`.
2. `accounts.rs`, `models`: walk `me.providers`, not the models' first sightings. A row whose `provider` is none, as
   world fixtures have it, keeps today's row with a null provider, last. `account_lines.rs`: its lines.
3. The stories. Break each once to see it bite.
4. Narrow tests, then the implementer's text's closing checks, with `-p temper-client-world`. Commit and push.

## Report

As the implementer's text says.

## Answers

2026-11-06. May this task update the two obsolete assertions in `tests/jig/amend_choice.rs`, outside its files? Yes:
acceptance 1 and 2 reach `task amend` through the shared choice path, and `temper/docs/design/client/tools.md`, 5.5
fixes both behaviours, so the task's own suite can't pass without them. The file is now in `files`, for these two tests
alone:

- **`amend_refuses_an_unknown_model`:** the expected words become `model not offered: unknown\n`; the rest stands.
- **`amend_requires_an_effort_for_a_model_with_efforts`:** remove it. `tests/jig/choices.rs`'s
  `amend_takes_the_default_instead_of_the_previous_effort`, which the run wrote, is the story that replaces it.

Resume from the pushed `5ba6e91bef` on `model-catalogue-1/ctl-offer`. Merge `origin/model-catalogue-1/offer-surface`
first: its `c::MeModel` and `MeProject` shapes and literals are that task's, and `fixtures/choices.rs` takes `effort`,
the first of `efforts`, from it. If the merge still stops compiling at that task's work, edit none of it: stop and
report that the task still waits on `offer-surface`, naming the error. Then the report's *Unobserved work on resume*
stands as the remaining work, each break-it-once check included.
