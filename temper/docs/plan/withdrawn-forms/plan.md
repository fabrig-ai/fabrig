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
2. **jig-dialects** (jig; `jig/docs/design/protocol/llm.md, 4`), slice 2.
   - `tasks/jig-dialects.md`, high, ~90 minutes, needs nothing: `jig-protocol-config` refuses an
     endpoint dialect but 2 and 3 (`What::Form` at the endpoint's `dialect`, no new variant,
     since temper matches `What` exhaustively); every jig fixture, literal and world writing 0,
     1 or 4 moves to 3, in the coherent groups charter encoding needs (number and dialect must
     both match); temper-service's test fixtures move with them (values only); jig's five
     `smith_channel::v5::X` paths become the root re-exports and jig-test's agent
     configurations drop `account` and `identity`, so smith's channel task touches less of jig.
     The rule was open in jig's design: this cut wrote it in `llm.md` 4 with a Not yet entry.
3. **smith-product** (smith; `smith/docs/design/shell.md, 5.3`), slice 3.
   - `tasks/smith-product.md`, high, ~180 minutes, needs nothing: the `codex` preset, its
     models and the `anthropic` preset's identity go; settings refuse `identity`,
     `dialect = "codex"` and the `codex` endpoint (5.3); the generated agent configuration
     writes neither; no account reads `chatgpt_account_id`; the local service lends a sign-in as
     a bearer and an `env` value as a key (6.2: Anthropic takes a key in `x-api-key`), never the
     withdrawn grant. The product's worlds, end-to-end tests and benchmark pins move to
     `openai`. One branch: the product's worlds go red without the product change.
4. **smith-channel-v6** (smith, with jig's and temper's lines naming what leaves): to come.
5. **skein-routes** (skein): to come.

## Critical path

As it stands: `smith-product` (~180 minutes) beside `jig-dialects` (~90) and `temper-configs`
(~30, done), then smith's channel at v6, then skein's routes. Parts 1, 2 and 3 share no code and
run side by side; part 4 needs all three; part 5 needs part 4.

## Overlaps

- `temper-configs` touches no file an open draft beside this one changes (checked against the
  brief's list, 2026-11-03): configuring-p1's `temper-fixture-sections` edited the cluster's and
  github's `engine.json`, not their `agent.json`.

- `jig-dialects` edits single lines in `jig-domain-core/src/tests.rs`,
  `jig-protocol-config/src/tests/{fixtures,charters}.rs`, which reference-m1's unmerged
  `settings-wake-charter` also edits, and `jig/tests/channel/src/fixtures.rs`,
  `jig/tests/client-protocol/src/native/config.rs` and `jig/tests/client-browser/src/configuration.json`,
  which configuring-p1's `jig-documents-config` edited on its trunk: one-value hunks, unavoidable,
  since the fixtures are the decision's.
- `smith-product` touches no file an open draft beside this one changes; sign-in-wait's
  `credentials/product.rs` change is on main.

## Calls made

- **The order is the decision's, as a landing order:** a part waits only for the parts whose code
  it reads. temper's configurations, jig's dialect rule and smith's product are three consumers
  that each stop using a withdrawn form while it still exists, so they run side by side; smith's
  agent, protocol and channel remove the forms those three stopped using, and skein's routes go
  last, once smith names none of them.
- **The jig dialect rule** (the request's item 4, open in jig's design): accept 2 and 3 only,
  refused at the configuration's read, the decision's (a). Written in jig's `protocol/llm.md` 4
  by this cut, with its Not yet entry in `llm.status.md`.
- **An `env` account lends a key** (smith's product): shell.md 6.2 already says Anthropic takes a
  key in `x-api-key`, and a key on OpenAI's public route is sent as a bearer, so the design
  settles it; without it skein's removal of Anthropic's bearer route would break every `env`
  account on Anthropic.
- **The anthropic preset's own account** (`account = 'anthropic'`, `[accounts.anthropic]`) departs
  from shell.md 5.2 ("Neither names an account") but is no withdrawn form: recorded as its own
  gap in `shell.status.md`, outside this plan.
- **Status sentences that name several parts' gaps** (smith's `STATUS.md`, `agent.status.md`,
  skein's `llm.status.md`) are left to the part that removes the whole entry, so that parallel
  branches never edit one sentence: jig's task closes only jig's own status.
