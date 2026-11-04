# qa-out: contracts between the tasks

Binding calls for this run. `tasks/qa-page.md` and `tasks/acceptance.md` run side by side, so each
builds to this note and to no other task's branch.

## C1. `temper/docs/development/qa.md` keeps its eight headings

Exactly these, in this order, for the citations in `tools.md`, `agent-docs.md`, `configuring.md`
and `rung.rs` and for the document outside the repository that cites 3, 5 and 6:

```
## 1. In one page
## 2. Whom a pass plays
## 3. The deployment
## 4. The hands
## 5. Reading the forge
## 6. Checking an outcome
## 7. What it asks of temper
## 8. Open questions
```

Section 3 still defines a rung as a pair of backends (fakes, a real model, GitHub, Claude) and
still says how a pass sets a model, its room and its budget, without figures of its own:
`acceptance.md`, section 2, and `rung.rs` cite it for that.

## C2. `temper/docs/development/acceptance.md`, its form

The path is exact. Its sections, numbered:

```
## 1. In one page
## 2. What every story holds
## 3. The backends
## 4. Rung 0, the ground
## 5. Rung A, the playground
## 6. Rung B0, the green light
## 7. Rung B1, the repair
## 8. Rung B2, two delegates
## 9. Rung M3, a goal for a draft
## 10. Rung M4, surveys as sub-agents
## 11. Rung M8, tiers and verdicts
## 12. Open questions
```

Each story section holds, under these bold leads and in this order: **Backends**, **What the person
does** (numbered steps, the person's words verbatim in block quotes), **What must hold**, **No pass**
(where a story has one), **Its phases** (what a run times, from the request), **Twin and fixtures**.

`temper/docs/design/domain/testing.md` cites a story's "What must hold" by those words.

## C3. Who edits a shared file, and when

| File | Task | Edit |
|---|---|---|
| `temper/docs/development/qa.md` | `qa-page` | the page |
| | `plans-out`, after it | one sentence citing `acceptance.md` |
| `temper/docs/design/qa.md` | `qa-page` | the pointer |
| `tests/boundary/outward.txt` | `qa-page` | drops `temper/docs/development/qa.md 3` |
| | `plans-out`, after it | drops the two backlogs' lines |
| `temper/docs/design/draft/v0.1.0.md` | `acceptance` | named passages |
| `temper/docs/design/domain/testing.md` | `acceptance` | named passages |
| `temper/docs/design/release.md` | `acceptance` | one passage |
| `temper/docs/design/draft/reference-case.execution.md` | `acceptance` | named passages |
| `tools/citations/allow.txt` | `acceptance` | pre-approved lines only |

`qa-page` never names `acceptance.md`: that page is not on the trunk when it runs, and the
citations check would fail. `plans-out` adds the link once both have merged.
