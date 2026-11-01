+++
id = "bx-story-playground"
part = "pg"
kind = "feature"
needs = ["bx-walks", "fe-playground-forge", "fe-playground-root", "fe-kept-root", "pd-core-release", "sv-removal", "sv-playground", "sv-family", "cl-ctl"]
files = ["temper/crates/temper/tests/reset/**", "temper/crates/temper/tests/process.rs", "temper/crates/temper-service/src/**", "temper/crates/temper-engine-domain/src/**", "temper/crates/temper-engine-domain-forge/src/**", "temper/crates/temper-box/src/**", "temper/crates/temper/src/box_command/**"]
effort = "high"
minutes = 75
+++

## Goal

Draft section 8's first process story: a first run into the playground and out by its
reset, the store then read as empty, the welcome at Forge. On the shipped binary under
`temper box`, through `temper ctl` alone.

Acceptance:
- A new module of binary `process`, `tests/reset/mod.rs` (`#[path = "reset/mod.rs"] mod
  reset;` in `tests/process.rs`), with `tests/reset/playground.rs` holding
  `a_first_run_into_the_playground_and_out_by_its_reset_leaves_the_store_empty`:
  - a fresh scratch, `temper init`, `support::own_listener_files`, then a ctl story
    (`support::boxed::run_ctl`): sign in; `playground start` until it answers running
    (`support::forges::playground_start`, then `forge list --json` until the playground's
    forge reads connected); the scripted account is listed (`accounts list --json` or the
    tool that lists them); the playground's project `Playground` is listed;
    `box/groups/N` exists for the playground's forge N, and `forge-N/admin_token`;
  - then `playground reset` (`support::forges::playground_reset`) until it answers `reset`;
  - then, read through ctl: no forge (`forge list --json` lists none), no account of the
    scripted provider, no project named `Playground`, the playground's state `not_started`;
    on disk: no `box/groups/N`, no `box/started/N`, no `forge-N`;
  - the box exits 0 at the story's end with nothing left running.
- Run alone:
  `heavy cargo nextest run -p temper --profile process-temper --test process -E 'test(=reset::playground::a_first_run_into_the_playground_and_out_by_its_reset_leaves_the_store_empty)'`.

## Inputs

- Draft `temper/docs/design/draft/playground-reset.md`, sections 6.1, 6.2, 6.3 and 8.
- `temper/docs/plan/playground-reset-r6/notes/common.md`.
- `temper/docs/plan/playground-reset-r6/notes/contracts.md`, T3, T5, T6.
- `temper/docs/plan/playground-reset-r6/tasks/cl-ctl.md` (the tools' words); bx-harness's
  report (`support::forges`, `KillBox`/`StartBox`).
- The story it replaces in spirit: `a_first_run_answers_on_the_demo` and
  `a_first_run_ctl_demo_start_answers_running_twice` in `temper/crates/temper/tests/process.rs`.

## Owns and may change

- Owns: `temper/crates/temper/tests/reset/**` and its `mod` line in `tests/process.rs`.
- May change: `temper/crates/temper/tests/support/forges.rs`, adding a read helper (a ctl
  call and its JSON reading) the story needs.
- Options, pre-approved:
  - "The welcome at Forge" is read as the client's state from ctl if a tool reads it;
    otherwise the empty reads above stand for it (no forge, no account, `not_started`):
    say which in the report.
  - The demo's old process stories (`a_first_run_answers_on_the_demo`,
    `a_first_run_ctl_demo_start_answers_running_twice`,
    `a_first_run_answers_on_the_demo_and_a_second_finds_it_running`,
    `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`) that fail
    on the playground of records: change the first three to the new words if that is
    all; mark the last `#[ignore = "the demo's end on another forge's connect is gone (draft playground-reset.md, 6.3)"]`.
    List each.

## Steps

1. The story; `heavy cargo check -p temper --tests`.
2. Run it alone (command above); read a failure's log before anything reruns.
3. `cargo fmt`; `heavy cargo clippy -p temper --all-targets`. Old forms never raised.
   Commit and push.

## Report

The story's steps as run, its time, and what each read showed.

## Answers

2026-11-01. Q: the connected forge has no `forge-N/admin_token`, and ctl has no
playground-state read: which assertions stand instead?

A: Both acceptance items change; no tool or production code is added.
- **While connected,** assert `forge-N/` exists and `forge-N/admin_token` does *not*:
  the engine has taken the handover and removed its file before it connects
  (`temper/docs/design/protocol/setup.md`, 6.5, and its files table). The directory's
  removal after the reset stays as specified.
- **`not_started`** is read through ctl as a second `playground reset`, after the first
  answered `reset`, under a key of its own (never the first call's, whose kept answer
  would print `reset` again): it must print `refused: not offered` and exit 2, which
  the tool's notes define as "no playground runs" (cl-ctl's words). That, with the empty
  forge, account and project reads, stands for the welcome at Forge; say so in the
  report. Never `playground start` as a read.
- The inherited `fake::check::tests::verb` failure is not this task's: leave it under
  `## Left`. The four old demo stories stay as the pre-approved options say: run each
  once, and change or mark only those that fail.

2026-11-01. Q: `playground reset` stops and removes the box group (`Playground: removed`)
but ctl never hears an answer (30 s timeout): repair it here, or leave it to its owner?

A: Repair it here; the task widens to the production fault, and only to it. The process
tier is not in fab's gate, so no trunk check or repair would meet this fault, and this run
holds its only reproduction; `bx-story-reset-cut` waits on the same reset.
- **Locate first.** The box sent `removed 1` (its `Fact::GroupRemoved` and
  `Said::Removed` are pushed together, `temper-box/src/machine.rs`, `removed`), so the
  Project step answered and the hang lies after it, along T5/T6's chain
  (`notes/contracts.md`): serve's box link reading `removed N` into
  `Event::GroupRemoved` (`temper-service/src/route/box_link.rs`); `EraseSecrets` answered
  by `Event::SecretsErased` once all ten items are durable (`temper-service/src/erase.rs`;
  check every item's erase answers, an absent secret included, and that the event is not
  dropped while `engine.drain` is set); `removal::finish` erasing the `Forge` row and
  calling `playground::forge_removed` (`temper-engine-domain-forge/src/removal.rs`); the
  Account step's `RemoveAccount` and its answer (`temper-engine-domain/src/route/kept.rs`);
  then the row erased and `EndAnswer::Reset` delivered to the waiting client
  (`temper-service/src/family.rs`). Read serve's log of the failed run and the store's
  rows after it (`playground` state through ctl, `forge list --json`, `account list
  --json`) to see which step stood.
- **Fix in the lowest tier that shows it,** with a unit test there that fails before the
  fix (the forge domain's `src/tests/playground.rs`, the root's, or serve's
  `src/tests/box_link.rs`/`erase` tests), then the story here passes unchanged in intent.
- **May change, newly:** `temper-service/src/**`, `temper-engine-domain/src/**`,
  `temper-engine-domain-forge/src/**`, `temper-box/src/**`, `temper/src/box_command/**`,
  for this fault only: no new request, event, stored row or wire shape. If the fix needs
  one, stop and ask.
- Run each touched package's tests (`heavy cargo nextest run -p PKG`), then the story
  alone, then `temper-one-process-world` and `temper-real-loop-world` once each.
- Unchanged: the two legacy stories failing at `support/boxed.rs:345` and the seven
  inherited lib failures stay under `## Left`.
