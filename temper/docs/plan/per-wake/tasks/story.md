+++
id = "story"
part = "story"
kind = "feature"
needs = ["offering", "parts-core", "later"]
files = ["jig/tests/core/tests/suite/per_wake.rs", "jig/tests/core/tests/suite/main.rs", "jig/testing/jig-core-kit/src/stories/per_wake.rs", "jig/testing/jig-core-kit/src/stories.rs", "jig/testing/jig-test-profile/src/charters.rs"]
effort = "high"
minutes = 150
+++

# per-wake: the story, in the core world: a run reads the instructions for why it runs

## Goal

The request's first item, in jig's lowest world that holds a task with its inbox and runs
(`jig/tests/core`, the real core and its children with scripted hosts): a task whose
charter has a shared text and two parts, one `on` `first` and one `on` `question`, and an
LLM choice with a `later`, runs as `jig/docs/design/domain/tasks.md`, 8.6 says. Everything
it needs has merged; this task writes the story and fixes nothing outside its files.

Acceptance:

- A charter fixture (`jig/testing/jig-test-profile/src/charters.rs`, beside
  `research_charter`): resume false, instructions `S`, parts `[{on: FIRST, F}, {on:
  QUESTION, Q}, {on: ESCALATION, H}]`; two models its charter and the chat's authority
  allow, `M1` and `M2`, each with an effort, both with usable accounts. The task under it is
  a delegate the world's chat makes (`World::delegate_agent`'s event, `DelegateValidated`,
  with its member's `llm` set at the domain level to main `M1`, `later` `M2`: a run's tool
  names no `later`, `jig/docs/design/domain/engine.md`, 7.3).
- `jig/tests/core/tests/suite/per_wake.rs`, its scenarios in one bundle `#[test]` as
  `runs.rs` does (`super::bundle::Bundle`), each a fresh world, a table where rows share a
  shape. Each asserts on the assignment (`run.policy.instructions`, the run's main model and
  effort, `messages`), on what is relayed while it runs (`world.inbound`) and on the task's
  stored row:
  1. its first run reads `S\n\nF` at `M1`;
  2. a question arriving during that run (from a delegate the task made in it, asking its
     requester, as the kit's question stories do) is not relayed to it, keeps its place in
     the inbox, and, when the run parks idle, wakes the next run, which carries the
     question at its start and reads `S\n\nQ` at `M2`;
  3. an escalation (a delegate held, waiting for the task's decision) offered to a run that
     ends leaving it undecided wakes the next run, which reads `S\n\nH`;
  4. a person's `words` and an `amendment` sent during the first run are relayed to it,
     though no part of it is for them;
  5. a charter without parts behaves as today: the same question is relayed during the
     first run, which reads `S`, and with no `later` every run takes `M1`;
  6. if the world can hold a run between its preparation and its claim (its transcript or
     brief pending), a question arriving then is kept from it and wakes the next; otherwise
     leave this row out and say why in the report.
- Each scenario passes the core world's restart oracle as its neighbours do; a scenario
  that can't names its finding with `broken()` and the report says why.
- The world's cost stays within its suite's: one bundle, no sleeps, the clock driven.
  Report the bundle's time beside `runs_scenarios`'.
- Break one rule by hand at a time (relay regardless of `offered`; read every part; take the
  first choice always), run the bundle, see its row fail for that reason, restore; the report
  names each.
- `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/per_wake/)'` passes, and
  the core world's focused suite with it.

## Inputs

- `temper/docs/plan/per-wake/notes/common.md`; `notes/contracts.md`, sections 1, 2, 4 and 5.
- `jig/docs/design/domain/tasks.md`, 8.2, 8.6 and section 9 ("Waking");
  `jig/docs/design/domain/engine.md`, 7.1 and 7.4; `jig/docs/design/domain/testing.md`, the
  core world's section (how its stories are written and bundled).
- The closest stories: `jig/tests/core/tests/suite/runs.rs` (`runs_scenarios`, a message left
  unread waking the next run), `suite/charters.rs`
  (`a_delegate_on_another_charter_runs_its_instructions_and_account`), `peers/facts.rs` (a
  scripted question), `stories/kept.rs`.

## Owns and may change

- Owns the files in the header: the story, its mod line, a kit helper module
  (`jig-core-kit/src/stories/per_wake.rs`) for drivers the story needs and the kit lacks, and
  the charter fixture.
- Pre-approved: a driver the kit lacks (a delegate asking its requester, an escalation
  raised to the task) added in the helper module, built from the kit's existing events.
- A row that fails for a reason in jig's code, not the story, is not yours to fix: stop
  with `Question:`, naming the row, what it saw and the design line it breaks, with the
  story pushed.
- Not yours: anything outside the files above.

## Steps

1. The fixture and the helpers; the scenarios, one at a time, each run alone.
2. The bites above.
3. `cargo fmt`; `heavy cargo clippy -p jig-core-world -p jig-core-kit -p jig-test-profile
   --all-targets -- -D warnings`; the core world's focused suite.
4. Commit (why: the per-wake story runs whole in jig's lowest world); push your branch.

## Report

As the implementer's text says, and also: each scenario and what it asserts; the bundle's
time; any row left out and why.
