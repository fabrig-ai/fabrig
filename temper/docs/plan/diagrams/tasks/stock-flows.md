+++
id = "stock-flows"
part = "temper-flow"
kind = "feature"
needs = ["temper-flow"]
files = [
  "temper/crates/temper-service/src/stock/chat.txt",
  "temper/crates/temper-service/src/stock/configuring.txt",
  "temper/crates/temper-service/src/tests/stock.rs",
]
effort = "medium"
minutes = 15
+++

# temper's agents write a flow

## Goal

The stock's "Mermaid is not drawn, so do not draw diagrams." becomes what `skein/docs/design/draft/diagrams.md`, section 7 says agents write: a flow where a sequence says more drawn, the subset named. It waits for `temper-flow`, so no agent is told to draw what isn't drawn.

Acceptance:

- In both `chat.txt` and `configuring.txt`, the sentence `Mermaid is not drawn, so do not draw diagrams.` is replaced, whole, by:

  `Where a sequence says more drawn, write a flow: a fenced block, info string mermaid, first line flowchart LR or flowchart TD, then accTitle: its name, then one path joined by -->, each node id[Name<br>caption], :::strong after the one that matters. A branch, a merge, an edge's label or another shape is shown as code.`

- `temper-service/src/tests/stock.rs`: the phrase row near line 1503 holds `Where a sequence says more drawn, write a flow` and `one path joined by -->`, and the row that held the old sentence is gone. The pin near line 298 holds `configuring.txt`'s new length.
- A row beside them checks each stock's words make no claim that Mermaid isn't drawn.

## Inputs

- `temper/docs/design/domain/agent.md`, section 8, *Its instructions*, the last sentence of the first bullet.
- `skein/docs/design/ui.md`, 10.1, the subset the words name.
- `temper/docs/plan/diagrams/notes/common.md`.

## Owns and may change

Owns the files in the header, each only for these lines.

Readers, surveyed by `rg -il mermaid`: `tests/stock.rs` and `agent.md` alone read these words. `configuring.txt` is `charters[2]`, whose length `tests/stock.rs` pins.

Overlap: `model-catalogue-2`'s tasks `configuring-learns` and `stock-models` changed `configuring.txt` and `tests/stock*.rs`, and may land first. This is why this task comes last in the plan.

Options, each pre-approved:

- If a catch-up finds `configuring.txt` changed by another plan, keep its lines and edit only this sentence. Set the pin to what the merged file measures.
- If another pinned figure follows `chat.txt`'s or `configuring.txt`'s size (a stock limit, a floor's row), set it to what the new words need, and name it in the report. Don't lower a budget or raise a timeout.

## Steps

1. The two sentences, by exact replacement. Each file's other bytes don't change.
2. `tests/stock.rs`: the phrase rows and the pin.
3. Narrow test: `heavy cargo nextest run -p temper-service --lib -E 'test(stock)'`. Then the closing checks of your text. Commit and push.

## Report

The implementer's report, with the rule you broke by hand for the phrase rows, and any figure you moved.
