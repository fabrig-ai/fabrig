# withdrawn-forms: the plan

From `inputs/request.md` (status-gaps' item 5, question `withdrawn-forms`, answered (a) on
2026-11-03 at the director's word): the withdrawn compatibility forms leave every tree, as skein's
`llm.md` 4.6 and 4.7 and smith's designs say. Five parts, one per task of the decision, each in
its tree; every task updates its tree's design text and status files (`notes/common.md`).

## Before the tasks

- **smith's `shell.status.md` records the product's presets as a gap** (the decision's "First"),
  written by this cut with the first push: shell.md 5.2, 5.3, 6.2 and 7.1 not yet built, since
  the product's `codex` preset, the `anthropic` preset's `identity = 'claude-code'`, the settings
  keys `identity`, `account` and `dialect = "codex"`, the `chatgpt_account_id` read and the
  local service's withdrawn grant are still there. `smith/docs/design/STATUS.md`'s clause on the
  withdrawn forms now names the product too.
- **sign-in-wait has landed on main** (`e9ed8d5260`, 2026-11-02, on this plan's base), so smith's
  product task waits on nothing outside this plan.

## Parts and tasks

1. **temper-configs** (temper; `smith/docs/design/protocol/agent.md, 4`), slice 1.
   - `tasks/temper-configs.md`, medium, ~30 minutes, needs nothing: every agent configuration
     temper writes (init's template, the owned example, the fixtures and the test literals) loses
     its endpoint `"identity": "plain"`, which smith reads as plain when absent; five skein-llm
     `Credential` literals become `Credential::key`, so no temper line names `account_id`.
2. **jig-dialects** (jig): to come.
3. **smith-product** (smith): to come.
4. **smith-channel-v6** (smith, with jig's and temper's lines naming what leaves): to come.
5. **skein-routes** (skein): to come.

## Critical path

As it stands: `temper-configs` (~30 minutes). The order of the rest is the decision's: parts 1, 2
and 3 share no code and run side by side; part 4 needs all three; part 5 needs part 4.

## Overlaps

- `temper-configs` touches no file an open draft beside this one changes (checked against the
  brief's list, 2026-11-03): configuring-p1's `temper-fixture-sections` edited the cluster's and
  github's `engine.json`, not their `agent.json`.

## Calls made

- **The order is the decision's, as a landing order:** a part waits only for the parts whose code
  it reads. temper's configurations, jig's dialect rule and smith's product are three consumers
  that each stop using a withdrawn form while it still exists, so they run side by side; smith's
  agent, protocol and channel remove the forms those three stopped using, and skein's routes go
  last, once smith names none of them.
