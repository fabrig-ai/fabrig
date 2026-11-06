# first-usable-model: the plan

From `inputs/request.md`: a box with one kind of account does its work, and a waiting run says why.
The trunk is `first-usable-model/integration`. This plan grows batch by batch; what follows stands as
of the batch last pushed.

## The cause, read from the code

The request's reading holds, and is narrower in one place. `jig-domain-core`'s `runs/llm.rs`,
`default_llm`, already takes the first model with a usable account at Create, as
`jig/docs/design/domain/tasks.md`, section 3 says. It asks `accounts.select` with **empty host facts**,
and `host_allows` holds a program source unusable without positive evidence. So a Claude Code login never
counts there, the default falls to the charter's first, `gpt-6.1-sol`, and the choice is recorded for
good. A Claude API key or a sign-in would have been chosen: that is the fake box's and the rung's cover.

"T1 · Claude subscription · gpt-6.1-sol" isn't drawn by `choice.rs`, whose `eligible` is right. It is
`pages/configure.rs`, `byline`, the configuring chat's line.

## Parts and tasks

| # | Task | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| 1 | `roots-quiet` | | medium, 30 | request's 5: the line only with a malformed block |
| 2 | `configure-byline` | | high, 60 | request's 2: the configuring line from the composer's |

Still to come: the engine's choice (request's 1), a wait reaching the person (3), the printed command (4).

## Critical path

As it stands: `configure-byline`, ~60 implementer-minutes.

## Overlaps

- `configure-byline`: `temper/docs/design/client/README.status.md`, one line, and
  `temper/docs/design/client/README.md`, 3.5, open in `look-round-6`, `ui-rename-focus` and others;
  `temper-client-view/src/choice.rs` and `pages/configure.rs` are in no live draft's files.
