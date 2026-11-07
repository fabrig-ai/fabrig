+++
id = "foreman"
part = "foreman"
kind = "feature"
needs = ["size-refusal"]
files = ["jig/crates/jig-domain-core/src/settings/charters.rs", "jig/crates/jig-domain-core/src/tests/runs/default_llm.rs"]
effort = "high"
minutes = 70
+++

# wake-followups: a task given no LLM choice takes the effort its charter selects

## Goal

A charter declaring Sol at medium for a change's foremen runs them at Sol's default, high.
The foreman's delegate names no choice (`temper-engine-domain/src/route/forge_route/foreman.rs`,
`llm: None`), as temper's design has it: a change's foremen run at their charter's default
(`temper/docs/design/domain/agent.md`, its Later on a change's foremen's LLM choice). The
fault is jig's: a charter's model may select one of the catalogue row's efforts (temper's
settings form: "every member but `effort` must match its configured row"), and a task given
no choice "takes the effort its charter selects, or else its model's default" (jig's
`jig/docs/design/domain/tasks.md`, section 3), but the charter's resolution
(`jig-domain-core/src/settings/charters.rs`, `resolution`) installs each catalogue row whole
and drops the selected effort before `runs/llm.rs`'s `defaults` reads it. So the fix is the
resolution's, and the foreman's delegate, the status agent's and a producer given no choice
all take their charter's effort; the request's "make the foreman's delegate take the choice
its charter declares" is met without temper naming a choice, a stated departure from the
request's suggested site, since an application's root choosing an LLM would decide what jig
promises.

Acceptance:

- `notes/contracts.md`, section 4, holds in `resolution`.
- `an_omitted_choice_takes_each_models_own_default`
  (`jig-domain-core/src/tests/runs/default_llm.rs`) becomes a table over the charter's
  selection, its fixture (`task_with_default`) taking the charter's main and sub-agent
  efforts: the catalogue's main declares `low`, `medium`, `high` with `medium` its default,
  the sub-agent `high`; rows `(None, None)` expecting main effort 1 and sub-agent 2 (each
  row's own default, as today); `(Some("low"), Some("medium"))` expecting 0 and 1, the
  policy's efforts `low` and `medium`; and `(Some("high"), None)` expecting 2 and 2. The
  middle row is the foreman's case, Sol's medium under a high default. It fails before the
  fix (expecting 0, finding 1) and passes after. `a_given_choice_keeps_low_despite_the_models_default`
  keeps holding: a given choice wins over the charter's selection.
- It bites: install the row whole again and see the middle row fail; restore.
- This changes what every task given no choice runs at where its charter selects an effort,
  so the readers' tests run (Steps): jig's core and its world, and temper's packages whose
  charters name models.
- Delete the bullets `**Not yet:** 3: a charter's selected effort ...` from
  `jig/docs/design/domain/tasks.status.md` and `**Not yet:** 17.11: a charter's selected
  effort; see ...` from `jig/docs/design/domain/engine.status.md`, a stated departure from
  the implementer's text (`notes/common.md`), with no other change there.

## Inputs

- `temper/docs/plan/wake-followups/notes/common.md`; `notes/contracts.md`, section 4.
- jig's `jig/docs/design/domain/tasks.md`, section 3 (a charter's model and its `effort`;
  "A task given none keeps none"); jig's `jig/docs/design/domain/engine.md`, 17.11 (a
  charter's resolution, the sentence this plan changes) and 17.1 (a model setting's
  efforts).
- `temper/docs/design/domain/forge.md`, 8.11 (a foreman per question, under the forge's
  foreman charter).

## Owns and may change

- Owns the files in the header. `size-refusal` wrote this crate before you and has merged.
- Pre-approved: if a test of jig's core or world asserts a resolved charter's model whole
  where its charter selects another effort than the row's, it follows the selection; name
  each. If a temper test builds a charter whose model's `effort` differs from its catalogue
  row's and asserts the row's effort, follow the selection the same way, a reader file
  edited only there. If the resolution's `run_bytes` fit moves for a charter because the
  selected effort's name is longer than the row's, keep the fit measured on the record the
  resolution keeps, and say so.
- Not yours: temper's foreman delegate (`foreman.rs` stays `llm: None`), `default_llm_at`'s
  first-effort fallback in `jig-domain-tasks`, any stored format.

## Steps

1. The table; see the middle row fail.
2. The resolution keeps the charter's selected effort; the table passes.
3. Bite as above; delete the two status bullets.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-core
   -p jig-core-world`; `heavy cargo nextest run -p temper-service -p temper-engine-domain -p
   temper-engine-domain-world`; `heavy cargo check --tests` of `jig-domain-core`'s other
   direct dependents in every tree.
5. Commit (why: a charter's selected effort was dropped by its resolution, so a foreman
   declared at medium ran at its model's default); push your branch.

## Report

As the implementer's text says, and also: every reader whose expectation moved.
