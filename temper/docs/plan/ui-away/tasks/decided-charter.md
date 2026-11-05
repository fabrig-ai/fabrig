+++
id = "decided-charter"
part = "decided-charter"
kind = "feature"
needs = ["decided-tool"]
files = [
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/crates/temper-service/src/tests/stock_brief.rs",
  "temper/crates/temper-service/src/limits.rs",
  "temper/crates/temper-service/src/tests/limits.rs",
]
effort = "medium"
minutes = 40
+++

# The charter's words for `decided`

## Goal

Goals call the tool before they ask: `temper/docs/design/protocol/configuration.md`, 13.4; `domain/agent.md`, 4.3.

Acceptance:

- `temper-service/src/stock/chat.txt`, the goal's paragraph, right after "Otherwise do the work without asking.", these words:
  "Before you ask your requester anything, call decided with the question, its options, the one you'd take, why, and its kind; on
  Decided carry on with it and ask no one; on Ask, ask them with message, as a question. Funds, sign-ins, machines, a plan's go-ahead
  and anything hard to reverse are kind other. If they tell you a decision was overturned, ask it with message, as a question, and
  wait. If your words hold the line Asks: always, never call decided: ask them with message, as a question."
- The rest of the paragraph stands. "Otherwise do the work without asking" still reads true: a goal asks only what it can't settle.
- The figures move as `decided-tool`'s did, each to what its test measures. No budget is raised: it's a stock's measure.
- `temper/docs/design/protocol/configuration.status.md` loses 13.4's entry.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`.
- `temper/docs/design/protocol/configuration.md`, 13.4; `temper/docs/design/domain/agent.md`, 4.3.

## Owns and may change

Owns, as `files` lists them. May also change: the fixtures `decided-tool`'s spec lists under *The figures*, each to what its test
measures; `temper/tests/real-loop/src/case/scripts.rs` only for a cue that quotes the paragraph; a help page that quotes it
(`temper/docs/help/charter.md`), keeping its bound.

Options, each pre-approved:

- **A cue in `temper-fake` or the real loop** that matches on the paragraph's words and breaks: keep the cue, moving what it matches.

## Steps

1. The words. In `src/tests/stock.rs`: `a_goal_is_told_to_call_decided_before_it_asks`,
   `asks_always_is_told_never_to_call_it`, each reading `chat.txt`.
2. The figures, `heavy cargo nextest run -p temper-service`, `-p temper --test suite -E 'test(startup)'`.
3. The check of the implementer's text, and push.

## Report

As `notes/common.md`, Your report: each figure, old and new.
