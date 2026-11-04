# debt-12: the spot review of skein D016, and a generated debt index

Two independent parts (`inputs/request.md`). No debt item closes here.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `debt-index` | `debt-index` | | high | 40 | `tools/debt/`, the five READMEs lose their tables |

The link part is being cut: its tasks follow in the next batch.

## Decisions

- **`debt-index` is one task:** the tool and the five READMEs land together, so no commit has a README naming a
  command that isn't there. It crosses the trees only in those READMEs, and one citation in hearth's design.
- **The highest ID is a line in each README,** not derived from history: a shallow clone would derive a wrong
  one. Paying an item never touches the line, so it causes none of the conflicts this part removes; two
  branches that add items do meet on it, which is the clash on an ID it should show. The command checks it,
  since it's the one hand-kept line left.

## Critical path

`debt-index`, so far.
