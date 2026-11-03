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
4. **smith-agent-channel** (smith, with jig's and temper's lines naming what leaves;
   `smith/docs/design/protocol/channel.md, 6`), slice 4: two tasks, in order, since smith's own
   worlds can grant Codex only through the withdrawn grant.
   - `tasks/smith-agent-llm.md`, high, ~180 minutes, needs the three parts above: the agent's
     configuration refuses an endpoint's `account` and `identity` and the `codex` provider; the
     LLM component drops the identity profile, serves `openai` and `anthropic` alone, and installs
     a key alone on `anthropic` (llm.md 7: a scheme the dialect does not take); smith's worlds
     move to the public route and to keys (`limits.openai`, `public_call`); jig's and temper's
     `identity:` lines go. smith's arms on skein's `OpenAiCodex`, `Cap::AccountId` and
     `CredentialLimits.account_id` stay, inert, for skein's task.
   - `tasks/smith-channel-v6.md`, high, ~150 minutes, needs `smith-agent-llm`: `channel-v5` is
     renamed `channel-v6` (as v4 became v5), without `GrantValue`'s withdrawn arm (tag 2) and
     `Cap`'s `account_id` (tag 20), both last, so nothing renumbers; every half and reader drops
     them, jig's `jig-test-service` arm and temper's `temper-service` and `temper-worker-domain`
     arms included.
5. **skein-routes** (skein, with smith's and jig's last lines; `skein/docs/design/llm.md, 4.7`),
   slice 5: two tasks, in order, since they share skein-llm's types, client and worlds.
   - `tasks/skein-anthropic-bearer.md`, high, ~120 minutes, needs `smith-agent-llm` (smith then
     installs a key alone on Anthropic and names no identity profile): a bearer on Anthropic is
     refused at admission as `Invalid`; the OAuth beta, `Credential::anthropic`,
     `anthropic::identity` and the unified plan-limit rule go; the Anthropic recordings stay as
     grammar; the live suite's Anthropic tests take a key file. Runs beside `smith-channel-v6`.
   - `tasks/skein-codex-route.md`, high, ~240 minutes, needs `skein-anthropic-bearer` and
     `smith-channel-v6`: `Endpoint::codex()`, `Provider::OpenAiCodex`, the `codex` limits,
     replay tag 1 (refused, 2 and 3 unmoved), the account id (`Credential`, `CredentialLimits`,
     `Cap::AccountId`), `openai::identity` and the Codex recordings go; their coverage becomes
     synthetic, labelled so; the shared worlds move to the public route; section 4.6 is deleted,
     its number unused; smith's last arms and fields on these items and jig's one line go with it.
6. **skein-live** (skein; `skein/docs/design/llm.md, 5`), slice 6: **a question** (`live-logins`).
   The live suite runs only the withdrawn routes; which logins it uses next is the user's. The
   skein tasks build the recommended default (the ChatGPT login's token on the public route, an
   Anthropic key file), so nothing waits; another answer is a small later plan.

## Critical path

`smith-product` (~180 minutes; `jig-dialects`, ~90, beside it; `temper-configs` landed) →
`smith-agent-llm` (~180) → `smith-channel-v6` (~150) → `skein-codex-route` (~240): about 750
implementer-minutes, up to twice that with the buffer. `skein-anthropic-bearer` (~120) runs beside
`smith-channel-v6` and is off the path.

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
- Parts 4 and 5 touch only smith, skein and the reader lines listed in their specs; no draft beside
  this one changes smith's channel, agent or LLM crates or skein-llm. Within the plan, files shared
  between tasks are always ordered by `needs` (`smith-agent-llm` and `smith-channel-v6` share
  `agent-shell/src/config/endpoints.rs` and `agent-process/src/lib.rs`; the skein tasks share
  skein-llm's types, client and worlds).

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
- **Seven tasks for the decision's five:** each of the five is a part. smith's agent and channel and
  skein's routes each take two ordered branches, because one branch would hold 100 or more files.
  Each half compiles and passes alone, in the order the surveys found: smith's agent before its
  channel (its worlds grant Codex only through the withdrawn grant), Anthropic's bearer before the
  Codex route (they share skein-llm's types and worlds).
- **Arms that cannot leave early:** `wildcard_enum_match_arm` is denied, so smith's exhaustive
  arms on skein's `Provider::OpenAiCodex` and `Cap::AccountId`, and its `CredentialLimits.account_id`
  fields, stay inert through part 4 and go in `skein-codex-route`, the branch that removes them
  (the producer carries its readers' lines).
- **Stored data starts afresh, as decided:** no format version moves but the channel's. A stored
  dialect-1 transcript is carried over by compaction, the transcript's existing rule
  (`transcript.md`, 4), not migrated; a tag-1 replay is refused; skein's replay tags 2 and 3 are
  never renumbered.
- **Reserved header names stay reserved** (`session-id`, `thread-id`, `chatgpt-account-id`), so a
  configured endpoint cannot rebuild the Codex route; gzip stays as a mechanism with no shipped
  endpoint using it; the decoder constants measured from Codex captures keep their values,
  written out and pinned by synthetic tests.
- **One question, `live-logins`,** the user's (sign-ins and credentials): it holds only the
  `skein-live` part, which has no tasks.
