+++
id = "change-root"
part = "change-temper"
kind = "feature"
needs = ["change-jig-check", "change-call-llm", "change-row-llm"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/forge_route/change_llm.rs", "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs", "temper/docs/help/change.md", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 110
+++

# change-root: the root checks a change's producer and LLM choices and makes the change under them

## Goal

A `change` naming `producer` or `llm` (decoded by task `change-call-llm`) is checked by the root
as it decides the change's delegate, and the change it makes runs its producer under that
charter and its delegates at those choices (contract C4): the producer's charter offered by the
newest settings and one the run's authority may delegate to; each LLM choice checked by jig's
check (task `change-jig-check`) against the charter its role runs under and the run's models.
The procedure's authority grants the producer's charter and the review charter; the procedure
carries them as its text parameter 11; `start_change` keeps them on the row (task
`change-row-llm`), whose delegates already take them.

Acceptance:

- `forge_route.rs`, `change_batch`: `change-call-llm`'s refusal of `producer` and `llm` goes.
  For each change, with `producer_charter = change.producer.unwrap_or(<the change charter
  chat_change grants today, domain.config.forge_charter>)` and the review charter as
  `chat_change` has it (4):
  - a named producer that `domain.core.charter(n)` does not offer:
    `ChangeRefused::Refusal(tasks::Refusal::Executor)`, as a delegate naming it is refused;
  - `llm.producer`, if any: `domain.core.tasks.check_llm(producer_charter,
    context.authority.models, choice)`; `llm.review` against the review charter; an `Err(why)`
    is `ChangeRefused::Refusal(why)` (it names the model and `LlmFault`).
  A producer's charter the run's authority may not delegate to is refused by jig's admission of
  the procedure's authority, whose kinds now name it (below), as beyond authority. If a story
  shows jig admits it (the procedure's kinds not checked against the run's), refuse it here
  instead when `context.authority.delegation.kinds` does not hold
  `tasks::AuthorityExecutor::Charter(producer_charter)` (or a kind covering every charter, as
  jig's authority reads it), with `tasks::Refusal::Authority` or the variant jig gives a
  delegate beyond its kinds, and say so.
- `chat_change`: the procedure's delegation kinds are `Charter(producer_charter)` and, where it
  differs, `Charter(review charter)`; when the change names `producer` or any `llm` role, its
  spec's parameters gain, after today's, `Bytes { name: 11, value: change_llm::spell(producer,
  &llm) }`; a change naming neither gains none (its spec as today).
- A new module `route/forge_route/change_llm.rs` (`mod change_llm;` in `forge_route.rs`):
  `pub(super) fn spell(producer: Option<u32>, llm: &crate::ChangeLlm) -> Box<[u8]>` and
  `pub(super) fn read(text: &[u8]) -> Option<(Option<u32>, crate::ChangeLlm)>`, the text
  `C;P;R` of `temper/docs/design/domain/forge.md`, 8.1: `C` the charter in decimal, `P` and
  `R` the producer's and the review's choices, each part empty when none; a choice its main,
  then each sub-agent, separated by one space, each `M` or `M.E` in decimal (`6;3.1 0.0 2;3.0`),
  `account` `None`. `read` refuses (`None`) a text without exactly two `;`, a charter past
  `u32` or 0, an empty entry within a choice, a number past `u8`, a non-digit, a choice of more
  than `1 + tasks::MODELS` entries; `read(&spell(p, l)) == Some((p, l))` for every input with
  no account.
- `start_change`: parameter 11 (`Bytes`, read by `change_llm::read`, `None` is `return
  false`) gives the row's `producing_charter` (with no charter, `domain.config.forge_charter`
  as today), `producing_llm` and `review_llm`; without parameter 11, as today with both
  choices `None`.
- `change_spend`: the coding floor from the row's `producing_charter`'s charter and the review
  floor from its `review_charter`'s (today both read `domain.config.forge_charter`).
- `temper/docs/help/change.md`, the paragraph "With the control family ...": replace
  `` `after` (the changes
  it waits for, by task number or a `name` from the same call) and
  `changes` (several at once, all or none). `` with exactly this, each line indented two
  spaces as the list's are:

  ```
  `after` (the changes
  it waits for, by task number or a `name` from the same call),
  `changes` (several at once, all or none), `producer` (the charter
  number its coder, repairs and resolutions run under; left out, the
  forge's change charter) and `llm` (an LLM choice for its `producer`
  and its `review`, by configured model and effort names; a role left
  out takes its charter's default).
  ```

- Remove the **Not yet** entries "4.3, 4.3.1: `change`'s `producer` and `llm`" from
  `temper/docs/design/domain/agent.status.md` and "8.1, 8.9: a change's producer's charter and
  its agents' LLM choices" from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (the authority bullet, `producer`, `llm`).
- `temper/docs/design/domain/forge.md`, 8.1 (the parameters, the plan paragraph with
  parameter 11) and 8.9 ("A change keeps its charters and its LLM choices").
- `temper/docs/design/protocol/configuration.md`, 13.2 (the forge's charters paragraph).
- `jig/docs/design/domain/tasks.md`, section 4 (the check alone).
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C3, C4, C5.

## Owns and may change

Owns the files in the header.

- `forge_route.rs`: `change_batch`, `chat_change`, `change_parameters`' caller, `start_change`,
  `change_spend` and the `mod` line, no other hunk (other plans change this file). The kinds
  family's task that moves `start_change` and `chat_change` to the forge's `charters()` (C3)
  needs this task: leave `domain.config.forge_charter` and the review charter 4 read where
  they are, in one place each, so its hunk replaces them.
- Not yours: the decode, the row and its store, the delegates' `llm` (done by your needs).
- Options: if the change's start refused (`start_change` returning `false`) leaves the change
  task without a refusal its requester hears, keep it so (as a malformed parameter is today)
  and say so in the report.

## Steps

1. `change_llm.rs` with unit tests in `forge_route/tests.rs`: `spell` and `read` round trips
   (`;;`, a charter alone, a producer's choice with effort and two sub-agents, `M` without
   effort, both roles), each malformed text refused.
2. `change_batch`, `chat_change`, `start_change`, `change_spend`.
3. Unit tests in `forge_route/tests.rs` beside the change tests: a control run's change naming
   `producer` and `llm` gives a procedure delegate whose kinds hold the producer's and review
   charters and whose parameter 11 spells them; a named charter not offered is refused
   `Executor`; a producer choice naming a model outside the producer's charter is refused
   `Llm { model, why: Charter }`; a review choice is checked against the review charter, not
   the producer's; `start_change` with parameter 11 writes the row's `producing_charter`,
   `producing_llm`, `review_llm`, and with a malformed 11 starts nothing.
4. The help page, the two status entries.
5. Narrow: `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/forge_route/)'`
   (check the count includes the new ones); `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`; the help readers:
   `heavy cargo nextest run -p temper-docs`, `heavy cargo nextest run -p temper --lib -E
   'test(/help/)'`, `heavy cargo nextest run -p temper-client-tools --lib -E 'test(/help/)'`,
   `heavy cargo nextest run -p temper-engine-smith --lib -E 'test(/docs/)'`.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`;
   `python3 -I -B tools/citations/citations.py --base origin/main`. Commit ("Make a change
   under its named producer and LLM choices") and push.

## Report

As the implementer's text says, saying which of jig's checks refused a producer's charter
beyond the run's authority (admission, or this task's own), and whether a malformed parameter
leaves the change task unrefused.

## Rebase (2026-11-03, step merge, foreman reference-m3/change-root/foreman-1)

Rebase onto the landing trunk, preserving parameter-11 handling alongside landed row and tier changes.
The merge failed in three paths; resolve these overlaps without broadening the feature.
- `temper/crates/temper-engine-domain/src/route/forge_route.rs`: in `start_change`, keep `producing_llm: llm.producer` and `review_llm: llm.review`, replacing the `None` initializers from landed dependency `e42b8b1d77`. Preserve the parsed producer charter and the dependency’s `change_delegate` LLM forwarding.
- `temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`: retain both main’s `release_held_tier` helper from `f012f783bf` and this branch’s `choice` helper, codec tests, choice checks, authority test, start fixture and spend tests. Close each function independently; the insertion conflict shares a closing brace.
- Retain `change_delegates_take_their_roles_saved_llm_choice` from `e42b8b1d77` once, plus main’s tier projection changes.
- `temper/crates/temper-protocol-store/src/generated/v11.rs`: preserve the landing trunk’s version, which already includes the dependency’s LLM records and main’s `TierRow.observations` extension, integrated by `2598652ab2`. Its `BOUNDS_2` length is 24, not this branch’s 23.
Treat the generated-file resolution solely as preserving landed dependency code; introduce no schema or stored-format change.
Keep the automatically merged tier observations and row LLM fields in the surrounding files.
Run the spec’s focused forge-route unit and world checks, confirming both tier and change tests are selected, then complete its required checks.
