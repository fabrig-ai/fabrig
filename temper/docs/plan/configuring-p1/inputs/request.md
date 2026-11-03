# configuring-p1: documents, revisions, a chat's template

Carry out phase **P1** of `temper/docs/design/draft/configuring.execution.md`. Its entry is the scope: Executes, Design, Builds, Tests, Old code and Walk. It runs as a reference milestone does (`temper/docs/design/draft/reference-case.execution.md`, section 2): design and code go through this plan as one pipeline. Its designers write into the trunk only where a cited section is incomplete, and no more (`DESIGN.md`).

## The scope, in short

- **A charter's documents:** jig plans them with the brief (jig's `domain/engine.md`, 17.9); temper gathers each as a brief section (temper's `protocol/configuration.md`, 13.7).
- **A run reads its charter, documents included, at its task's revision:** jig's `domain/engine.md`, 17.3; temper's `domain/agent.md`, 4.1 and 4.6.
- **`chat start --template`,** any template a chat's: temper's `client/tools.md`, 5.3.
- **Tests:** `temper/docs/design/domain/testing.md`, 6.4's rows for a charter's documents and for running work keeping its revision.

## What runs beside it

Several plans are building at once. Keep out of what they own:
- `reference-m1`: the goal's own branch. It writes temper's `forge.md`, `agent.md` 4.3.1, `configuration.md` 13.4 and 13.9, and jig's `tasks.md` 6 and 8.3.
- `reference-m3`: starts after M1. It writes jig's `engine.md` 17.5 to 17.7, temper's `configuration.md` 13.2 and 13.3, and `agent.md` 4.3.1.

M3's charters will name documents. They wait for this plan, so build the documents generally, for any charter, not for one.

## Old code, for reference only

jig's and temper's `configuring-revisions/documents-r1`, temper's `configuring-revisions/fleet` and `configuring-revisions/story`, in hel8's mirrors at `~/src/fabrig/attic/<tree>/mirror.git`. The assessment is in hel8's `~/src/fabrig/coord/old-repos-value/`, which notes that documents-r1's temper forge does not yet gather a document.

They are a reference, never a plan. Whether to use them, what of them, and how is the designers' and implementers' choice, case by case: adapting old code may cost more than writing it fresh. Implementers cannot reach the attic, so whatever a designer takes goes into the specs or notes.

## Walk

None of its own: it folds into P2's.
