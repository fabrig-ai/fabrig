# first-usable-model: the plan

From `inputs/request.md`: a box with one kind of account does its work, and a waiting run says why.
The trunk is `first-usable-model/integration`.

## The cause, read from the code

The request's reading holds, and is narrower in three places.

- **The default.** `jig-domain-core`'s `runs/llm.rs`, `default_llm`, already takes the first model with a usable
  account, as `jig/docs/design/domain/tasks.md`, section 3 said, not the main's provider alone. But it asks at Create,
  and with **empty host facts**, and `host_allows` holds a program source unusable without positive evidence. So a Claude
  Code login never counts there, the default falls to the charter's first, `gpt-6.1-sol`, and is recorded for good.
- **The byline.** "T1 · Claude subscription · gpt-6.1-sol" isn't drawn by `choice.rs`, whose `eligible` is right. It is
  `pages/configure.rs`, `byline`, the configuring chat's line, joining the first usable account to the first model.
- **The dots.** T1 is that configuring chat, and `pages/chat.rs` draws `above_composer` only when `!configuring`, so the
  reasons `choice.rs` holds never showed. Beside that, `TaskWaitWhy::Host` is unreachable from `run_choice`: an unsigned
  login reads "Waiting for a host".

**What hid it, as the request asks.** Yes. `r0-claude` isn't a name in this repository; the Claude kit that exists,
`forge-land-claude`'s `configure_live_model` (`temper/crates/temper/tests/support/forge_land.rs`), removes OpenAI and puts
Claude first. The fake box keeps the stock order, `gpt-6.1-sol, gpt-6-luna, fake-claude, demo`, and signs in to OpenAI by
default, so the first model has its account. `jig-core-kit`'s world answers every declared source usable.

**`check_interval_ms`, from reading.** A check is expected about 60 s after each answer, per declared source, while the link
is up (`jig-domain-host/src/sources.rs`). temper's worker root doesn't wake for it: `next_deadline` and `is_due` read the agent
alone (`temper-worker-domain/src/domain.rs`), though `jig-test-worker` includes the hub's. So an idle worker fires it only when
something else steps it, and a lapsed login stays usable until its next run. That matches the 140 s of silence. The fix is
cheap: `login-check`.

## What the cut decided

- **The choice is at the check, in jig, and a task given none stores none.** Better than the request's wording in one way: no new
  field is needed. `TaskRecord.llm` `None` is exactly "named none", `Some` is named, and `kept_account` is "what it's using",
  so no stored or wire format changes, which keeps this off `ui-walk-large/proposal-projects`' open wire. A task created
  before its account exists is served too, which Create can't do.
- **`Core::next_llm` is the one rule,** read by `chosen_policy`, `chosen_main` and the Task document, so the byline's `task.llm` tells
  the truth with no client change. `run_choice` itself barely moves.
- **Usable is the accounts document's join,** drawn out of `documents/catalogue.rs`, so what the page shows as ready and what the
  engine takes can't differ.
- **Failing that, the first with a serving account,** so a lapsed Claude login on a Claude-only box waits naming Claude, not OpenAI.
- **An unnamed task's authority still bounds it:** refusals at Create, amendment and move stand in for those the recorded default gave.
- **The printed command comes from the box,** in `DeploymentAnswer`: the only way to fix `temper` being off `PATH`. `program-login`
  doesn't find a box by itself; the pasted line says its directories. Only standalone sends it: an engine alone doesn't know its
  workers' paths, and its pages print placeholders.
- **A Chats row holds no wait,** and says "Starting": computing `run_choice` per row is a cost its query doesn't carry
  (`jig/docs/design/client/domain/contract.md`, 5.1). The chat's page says why.

## Parts and tasks

| # | Task | Tree | Needs | Effort, minutes | Request's | What it lands |
|---|---|---|---|---|---|---|
| 1 | `roots-quiet` | smith | | medium, 30 | 5 | the line only with a malformed block |
| 2 | `configure-byline` | temper | | high, 60 | 2 | the configuring line from the composer's |
| 3 | `first-usable-choice` | jig | | high, 150 | 1, part of 3 | `next_llm`; an unsigned login waits as its account's |
| 4 | `stock-one-account` | temper | 3 | high, 80 | Tests | init's document, one account of each kind |
| 5 | `run-wait-status` | jig | | medium, 45 | part of 3 | `ChatStatus::RunWaiting`, no dots |
| 6 | `printed-command` | temper | | high, 100 | 4 | the box's whole command on the pages |
| 7 | `wait-says-why` | temper | 2, 5, 6 | high, 75 | 3 | the wait's words and acts, configuring chats too, `task show` |
| 8 | `login-check` | temper | | medium, 40 | the lower priority | an idle worker checks at its interval |

Eight, past the request's three to five: 5 and 7 split at the tree line, and 8 and 1 are small. Six go out at once.

## Critical path

`first-usable-choice`, `stock-one-account`: ~230 implementer-minutes, then the trunk's end check. Beside it, `printed-command`,
`wait-says-why`: ~175.

## Overlaps that couldn't be avoided

Each kept to named, small hunks (`notes/common.md`):

- `temper-client-view/src/pages/chat.rs` (`wait-says-why`, one condition): `look-round-6`, `chat-reading-a`.
- `temper-client-view/src/pages/chats.rs` (`run-wait-status`, three arms): `look-round-6/call-line`, `chat-reading-a/chat-live`.
- `temper-client-view/src/words.rs` (`wait-says-why`): `look-round-6`, `look-round-7`, `ui-walk-large/home-true`.
- `temper-client-view/src/accounts/words.rs`, `pages/welcome_model.rs` (`printed-command`): `look-round-7/sign-in-wait-page`.
- `temper-client-view/src/limits.rs` (`printed-command`, only if a bound must grow): `look-round-6`, `chat-reading-a`.
- `temper-service/src/system.rs` (`printed-command`, one hunk): `configuring-words/project-people`.
- `temper/docs/design/client/README.md` and `README.status.md`: `look-round-6/reading`, and others on the status file.
- Kept off: `temper/crates/temper/tests/support/{ctl,boxed,person}.rs`, `tests/fake_backends/**`, `tests/suite/startup.rs` (`box-link-first`), and
  `jig-client-protocol-wire` (`ui-walk-large/proposal-projects`).

## The walk

`walk = true` on slice 3. On a real box with one account, unedited, fabrig/qa walks: a chat starts and answers on that account's
model, its byline naming that account and model, and a box with no usable account says why it waits, with no dots, and offers the
act. **What can't be walked:** a real `temper program-login` needs a person's sign-in, so the walk starts from a login already
signed in, or a key, and pastes the printed command only as far as the program's own sign-in prompt.

**A box from before this:** its tasks recorded a default, which now reads as named, so a task already hung stays hung until amended or
closed. A configuring chat has no chooser on its page. No migration, as the request allows before v0.1.0; walk a fresh box.
