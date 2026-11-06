+++
id = "stock-seven"
part = "stock-seven"
kind = "feature"
needs = ["effort-config", "offer-surface", "ctl-offer", "picker-offer"]
files = [
  "temper/crates/temper-init/template/**",
  "temper/crates/temper-init/src/tests/**",
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/startup/members.rs",
  "temper/crates/temper-service/src/tests/first_model.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper/tests/support/first_repository.rs",
  "temper/crates/temper/tests/support/forge_land.rs",
  "temper/crates/temper/tests/support/forge_land/rung.rs",
  "temper/crates/temper/tests/suite/forge_land.rs",
  "temper/crates/temper-fake/src/backends.rs",
  "temper/crates/temper-fake/src/backends/tests.rs",
  "temper/docs/help/deployment.md",
  "temper/docs/help/account.md",
  "temper/docs/help/reference/configuration-engine.md",
  "jig/crates/jig-domain-core/src/tests/runs/reserve.rs",
  "jig/crates/jig-domain-core/src/tests.rs",
]
effort = "high"
minutes = 200
+++

# Seven stock models, with efforts, in the stock's order

## Goal

`temper init` writes section 13.4's seven models with their efforts, default efforts and prices. Stock charters take
Opus 5.5 then Sol 6.1, the configuring charter Opus 5.5, Astra 6, Sol 6.1, then the rest. C is the costliest model's, so
that a run on Astra 6 or Fable 5.1 is admitted. The default model moves away from `gpt-6.1-sol`, and after-landing went
red the last time init's order moved, in tests a fixture hid, so the survey's list is part of this task.

Acceptance:

1. Each of the four documents parses and starts, its `engine.llm.models` in the table's order, `demo` last where it has
   it, and the agent's models matching.
2. `stock::checked` on `temper init`'s document: charter 1 `[opus, sol, sonnet, fable, astra, terra, luna, demo]`,
   charter 3 `[opus, astra, sol, sonnet, fable, terra, luna, demo]`, charters 2, 4, 6 and 9 as charter 1. On a document
   naming no first choice, its own order (`temper-service/src/tests/stock.rs`).
3. C 10,896, K 43,584, a chat's 24K 1,046,016, on that document. A document with the cheap model first and Fable later
   takes Fable's C.
4. `first_model.rs`'s stories still hold under the new order, none of them naming a model: Claude alone takes Opus at
   `high`; a ChatGPT plan alone takes Sol at `high` on a chat and Astra at `high` on a configuring chat, and each is
   admitted. No account keeps `claude-opus-5-5` as the charter's first in `next_llm`.
5. `temper-service`'s `stock_brief.rs` holds at the new figures, the brief still fitting whole.
6. T: `startup::turn_bounds` run, and its figure said in your report (`notes/stock-readers.md`, *The largest turn*).
7. The three process tests and `--model` walks below pass.
8. No stock change reaches a stored charter: `settings/restart.rs` skips a changed setting, so a charter stored
   before keeps its value. Write a story in `first_model.rs` or `stock.rs` if none holds this.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*A configured model*, *The stock*), `temper/docs/plan/model-catalogue-1/notes/stock-readers.md`, which is this
  task's reader list.
- `temper/docs/design/protocol/configuration.md`, 13.4 and 13.8; `temper/docs/design/protocol/setup.md`, 8.2 and 8.3.

## Owns and may change

- Owns its `files`, and may change every site `notes/stock-readers.md` names, each only as that row says.
- **Departure from the implementer's text:** `temper/docs/help/deployment.md`'s **Offering another model** and
  `account.md`'s line may change as the note says, keeping their bounds; the reference by its tool.
- May change `jig/crates/jig-domain-core/src/lib.rs`, the one line exporting `most_costly`.
- May change `jig/crates/jig-domain-core/src/tests/runs/reserve.rs`, its two `run::most_costly` calls to `most_costly`.
- May change `jig/crates/jig-domain-core/src/tests.rs`, removing its one line `runs::state as run,`.
- `temper/crates/temper-fake/src/**` is another draft's too (`ui-walk-3/fix-flow`): small hunks.
- Removes `temper/docs/design/protocol/configuration.status.md`'s **Not yet** 13.4, 13.8 about the seven models, and
  `temper/docs/design/protocol/setup.status.md`'s **Not yet** 8.3.
- Options, pre-approved:
  - A figure the note gives that a test contradicts: take the test's and report it, where the rule holds.
  - A site the note misses, of the same kind: edit it as its neighbours, and report it.
  - Not `temper/docs/examples/owned*`, nor `reference/`: their declarations are their own. If a test holds them to init's,
    change the test's edit.
  - `reference_settings.rs` builds on the owned example, so needs nothing. A charter's value copies its model's
    prices and efforts, and `settings/check.rs`'s `model_valid` holds it equal to `engine.llm`: a charter written against
    the old figures is refused on a box from the new init. Don't relax that; say in your report whether anything you ran
    met it.

## Steps

1. The four templates, by hand, from section 13.4's table. Prices: input and cached the table's input, output its
   output, `unit` 1,000,000. Then the scripted delegate's spend and tokens.
2. `stock.rs`: `FIRST_CHOICES`, `CONFIGURING_FIRST_CHOICES`; `charter` takes its order; C the largest `most_costly` over
   `llm.models`, `minimum_run` from it.
3. The readers, in the note's order: `temper-init`, `temper-service`, the fake, temper's engine world, `temper`.
4. `startup/members.rs`, the reference, the help pages.
5. The stories above, each broken once to see it bite.
6. The implementer's text's closing checks over `temper-init`, `temper-service`, `temper-fake`, `temper` and
   `temper-engine-domain-world`; `heavy cargo check -p temper --tests` compiles the live targets, which nothing runs.
7. The three process tests, each alone, then these walks, by name from `temper/crates/temper/tests/fake_backends/walk/`
   (`heavy cargo nextest list --profile process-temper -p temper --test process` gives them): `goal.rs`, `here_key.rs`,
   `away.rs`, `status.rs`, `chat_document.rs`, `note_parts.rs`. Commit and push.

## Report

As the implementer's text says, naming: T's figure; each process test and walk with its result; each live-tier site read,
what it does now, and that it wasn't run; every figure that differed from the note.

## Answers

2026-11-06. May this task remove the two redundant `run::` qualifiers in
`jig/crates/jig-domain-core/src/tests/runs/reserve.rs`, which the permitted export makes `unused_qualifications`?

Yes. At lines 12 and 25, `run::most_costly(&model)` becomes `most_costly(&model)`: the export's own consequence, under
`tests.rs`'s wildcard, and neither test's expectation changes. Keep the export; no lint exception and no test-only
export. Should the closing clippy show the same lint on another call of `most_costly` in `jig-domain-core`'s tests, the
same edit holds there; anything else in jig is a new question. Then finish what the first run's report leaves unobserved:
the closing clippy, the focused checks and dependents, `heavy cargo check -p temper --tests`, the three process tests and
the walks.

2026-11-06. May this task remove the `runs::state as run,` import in `jig/crates/jig-domain-core/src/tests.rs`, which the
two approved removals leave unused?

Yes. Remove that one line, line 29: `reserve.rs`'s two calls were its last uses, and `-D warnings` refuses it unused. It
is the same export's consequence, in a test module. No lint exception. The same holds without a new question for any
further `unused_imports` or `unused_qualifications` the closing clippy reports in `jig-domain-core`'s tests from that
export, each edit the removal of a name alone, and named in your report. An expectation, a test's body otherwise, or
production code in jig other than the one export line stays a new question. Then finish the outstanding checks the last
answer lists, with `startup::turn_bounds`'s T.

2026-11-06 (temper/forge). **The user's rule, 2026-11-06: a plan's execution only ever runs the focused and fuzzy suites.** No after-landing tier runs during a plan: we run optimistically, and repair if after-landing flags. A new story in a slow tier is written and compiled, not run. Step 7 goes: no process test and no walk runs, and `notes/stock-readers.md`, *The live and process tiers*, is for reading alone. Read those tests and walks and change what the stock's change plainly needs; `heavy cargo check -p temper --tests` compiles them, as step 6 has. Your report names what you changed without running and what you didn't check, so that a red after landing starts from there.

### 2026-11-06: The required CSV process story fails on two printed sign-in links at `temper/crates/temper/tests/process.rs:577`; have its owner repair that path and then resume (recommended), or extend stock-seven’s scope to diagnose and repair it? (model-catalogue-1/stock-seven-3, foreman model-catalogue-1/stock-seven/foreman-3)

No process test or walk runs; the CSV failure does not require extending stock-seven’s scope or waiting for repair.
The spec’s final Answer settles this: “Step 7 goes: no process test and no walk runs”.
It also says: “Read those tests and walks and change what the stock’s change plainly needs”.
This supersedes acceptance criterion 7’s requirement to run them.
The next run finishes that source survey, retaining the reported passing `heavy cargo check -p temper --tests`.
Report the observed CSV failure and all unobserved tests accurately; do not rerun them.
The duplicate-link path at `temper/crates/temper/tests/process.rs:577` remains outside this task’s repair scope.
The governing Answer says: “we run optimistically, and repair if after-landing flags.”

## Rebase (2026-11-06, step merge, foreman model-catalogue-1/stock-seven/foreman-4)

rebase the four conflicts, preserving seven-model stock alongside updated idle bounds, brief wording and status.
The merge failed on four content conflicts; resolve each by combining the task’s changes with the landing target’s changes.
- `temper/crates/temper-init/src/tests/documents.rs`: retain `stock_models`, `agent_models` and the expanded catalogue assertions. Incorporate idle presets from `d6dd165929` and `43e556233e`: `agent_models` must assign OpenAI 300,000 ms and Anthropic 120,000 ms.
- Inspect the auto-merged `temper/crates/temper-init/template/{agent,bare-agent}.json`: apply those presets to every model of each provider, including newly added rows; retain demo’s bound.
- `temper/crates/temper/tests/suite/forge_land.rs`, `a_claude_configuration_replaces_openai_and_keeps_the_demo`: keep stock-seven’s prices (2,000 input/cached, 10,000 output, unit 1,000,000) and take `2141c26092`’s idle 300,000. `configure_live_model` copies the OpenAI agent model before renaming it Claude.
- `temper/crates/temper-service/src/tests/stock_brief.rs`: preserve wording changes from `189d163c56` and requester guidance from `2361aeef21`, together with the eight-model fixture. Recalculate the combined non-brief/remaining figures, room assertion and `MARGIN`; neither side’s complete table represents the combined code.
- `temper/docs/design/protocol/setup.status.md`: take the target’s current status, including `103267b8e8`’s terminal coverage update and removal of the superseded bare-URL departure; remove only stock-seven’s obsolete “Not yet: 8.3” model entry.
Verify the affected document and stock-brief tests through heavy, using focused checks; retain both catalogue and wording assertions in the auto-merged service stock tests.
Compile temper’s test targets with `heavy cargo check -p temper --tests`.
Follow the final Answers: run no process tests, walks or after-landing tiers; leave the duplicate-sign-in-link failure outside this repair’s scope and report unobserved coverage accurately.
