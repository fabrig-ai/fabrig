+++
id = "help-stock"
part = "configuring-agent"
kind = "docs"
needs = ["stock-configuring", "agent-kinds"]
files = ["temper/docs/help/charter.md", "temper/docs/help/template.md", "temper/docs/help/chat.md", "temper/docs/help/goal.md", "temper/docs/help/settings.md", "temper/docs/help/configure.md", "temper/docs/help/proposal.md", "temper/crates/temper-docs/src/lib.rs"]
effort = "medium"
minutes = 60
+++

# help-stock: the help pages say the stock's configuring agent

## Goal

temper's pages in `docs/help/` (compiled into the binary, read by `temper help` and a run's
`read_docs`) say what `stock-configuring` and `agent-kinds` built: the stock's charter 3 and
template 3, the configuring chat started with `chat start --project PROJECT --template 3
TEXT`, a project's first web chat starting it, a charter's `tools` deciding whether its runs
are offered `settings` and `propose_settings`, and the kinds a run reads and proposes. This
task's departure from the implementer's text: it edits `docs/help/` pages, which are the
documentation, not the design.

Acceptance:

- `charter.md`: the stock charters as they are (1, 2, 3, 4, 6), charter 3 the configuring
  agent's; a charter's `tools` (its host families; temper offers by it the settings family
  alone, the others by the task's authority); "A run reads charters with the host tool
  `settings`" says which runs have it: a charter naming the settings family, the stock's 3.
- `template.md`: a chat defaults to template 1, a goal to template 2, and template 3 is the
  configuring chat; `--template 3` starts it.
- `chat.md`: the configuring chat, how to start it through ctl, and that a project's first
  chat on the web starts it.
- `settings.md`: the stock's floors as the stock now has them (five charters, three
  templates, the executors), and the configuring agent as the way to change settings by
  describing how one works.
- `configure.md` (a walk): one line in its opening or its first step offering the configuring
  chat as the other way, `temper help chat`; its steps unchanged.
- `goal.md` and `proposal.md`: what they say of the stock's templates, or of a run's
  proposal of settings (proposed, accepted, rejected, withdrawn as stale), corrected where
  `stock-configuring`'s report listed them wrong.
- Each page within its kind's bound and template, its `temper ctl` lines parsing, its links
  resolving (`temper-docs`' and `temper`'s step tests); a page's summary changed only if it is
  wrong; `temper-docs/src/lib.rs`' `PAGES` entries changed only where a page's title or summary
  did.
- The focused tests of `temper-docs` and `temper` pass.

## Inputs

- `temper/docs/design/protocol/documentation.md`, 2 (a page, its kinds, the voice) and 6.
- `temper/docs/design/draft/agent-docs.md`, section 6 (the voice).
- `temper/docs/design/domain/agent.md`, 4.3 and 8.1; `temper/docs/design/protocol/configuration.md`,
  13.4; `temper/docs/design/client/tools.md`, 5.3.
- `stock-configuring`'s report (its "help pages' wrong lines"), in the plan's reports as fab
  keeps them, or by reading the pages against `temper-service/src/stock.rs`.
- `temper/docs/plan/configuring-p3/notes/common.md`.

## Owns and may change

- Owned: the files in this spec's `files`.
- Options, pre-approved: another written page (not under `reference/`) that states the
  stock's charters or templates wrongly: correct that sentence alone, and list it.

## Steps

1. Each page, its sentences about the stock alone.
2. `heavy cargo nextest run -p temper-docs -p temper -E 'test(page) or test(help) or
   test(rules) or test(commands)'`, then each package's focused tests.
3. `python3 -I -B tools/citations/citations.py --base origin/main`.
4. Commit ("help: the stock's configuring agent, charter 3 and template 3") and push.

## Report

As the implementer's text says, and each sentence changed, by page.
