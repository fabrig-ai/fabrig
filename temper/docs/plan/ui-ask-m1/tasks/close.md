+++
id = "close"
part = "close"
kind = "docs"
needs = ["claude-start", "mechanics", "family", "client", "declare", "workspace", "stock", "bound", "see", "see-forge", "proposal-read", "see-proposal", "document", "propose"]
files = ["temper/docs/design/draft/ask-temper.execution.md", "temper/docs/design/draft/ask-temper.status.md", "temper/docs/design/STATUS.md"]
effort = "medium"
minutes = 20
+++

# ui-ask-m1: M1's state, once its tasks have merged

## Goal

With every task of the plan merged, the execution file says M1 is built, and no status
file the plan wrote still holds a **Not yet** it closed.

Acceptance:

- `temper/docs/design/draft/ask-temper.execution.md`, section 3's table: M1's state
  `being built: plan ui-ask-m1` becomes `built: plan ui-ask-m1; its walk follows its
  landing`, and nothing else in the file changes (a departure from the implementer's
  text, which closes drafts: the request asks that M1's state be kept current).
- Every **Not yet** this plan added, listed below, is gone from the trunk, each deleted
  by the task that built it. For one still there, read the task's report on the trunk's
  history (`git log --grep`): if the code it names is built, delete the entry and say so
  in the report; if not, leave it and name it under `## Left`.
  - `smith/docs/design/protocol/programs.status.md` (5.1, 5.3, 8, no workspace);
  - `smith/docs/design/domain/run.status.md` (3.3, no workspace);
  - `jig/docs/design/domain/authority.status.md` (3, the asking family);
  - `jig/docs/design/domain/engine.status.md` (7.3, a run's read of a settings proposal);
  - `temper/docs/design/client/README.status.md` (3.5 and 5.13, Ask temper);
  - `temper/docs/design/domain/agent.status.md` (4.2, 4.3, 8.4, 8.4.1, 8.4.2, 8.4.3);
  - `temper/docs/design/domain/root.status.md` (7.1);
  - `temper/docs/design/protocol/configuration.status.md` (13.4, charter 11);
  - `temper/docs/design/protocol/documentation.status.md` (5, `read_docs`).
- `temper/docs/design/STATUS.md`'s summary paragraph gains one sentence: Ask temper's
  first milestone stands in the code beside the configuring chat
  (`draft/ask-temper.execution.md`, M1), its walk following landing.
- `python3 -I -B tools/citations/citations.py --base origin/main` lists nothing new.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `temper/docs/plan/ui-ask-m1/plan.md`.
- `DESIGN.md`, sections 2 and 3 (status files; execution files).

## Owns and may change

- Owns the files in the header; the status files listed above for the deletion
  described and nothing else.

## Steps

1. The table's state; the status check; the summary sentence.
2. The citations check.
3. Commit and push.

## Report

As the implementer's text says; list each status entry found and what was done.
