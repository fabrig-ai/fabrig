# design-economy: execution

2026-11-14. How [design-economy.md](design-economy.md) gets carried out. It designs nothing: the pilot's goal writes its own design from the draft as it runs. Plans use the focused and fuzzy suites alone.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its figure after landing | State |
|---|---|---|---|---|---|
| E1, the pilot's template: copies of charters 5 and 2, built from the text they resolve to on the deployment (the stock's set, fabrig's lines and the deployment's own) with `design-economy.md`, 8's changes, and with their fields (models, bounds, shell, agents); a copy of template 4 run by the first copy, whose coding changes name the second as their producer; the rules, policy and delegation that let them run, after checking the room for two more executors (`temper/docs/design/protocol/configuration/stock.md`). Accepted on reading back the resolved charters and template. Template 4 and charters 5 and 2 unchanged | 8 | temper/dev | none | template 8, "goal for a lean design", run by charter 17 (the copy of 5), whose coding changes' producer is charter 18 (the copy of 2) | applied, settings revision 15, 2026-11-14 |
| E2, the pilot: hearth's render domain (7.1), `hearth/docs/design/domain/render/` rewritten under `design-economy.md`, 4 and 5, with lean specs (6), and `hearth-domain-render` refactored to it, as one goal from E1's template, carrying out `design-economy.md`, sections 4 to 7, as its request. `hearth-render-world`, `hearth-builder-world` and `hearth/tests/render/`'s expected files pass with their cases, expectations and oracles unchanged, the expected files byte for byte (7.2). Its last change deletes the contracts document (4.3) | 4 to 7 | temper/dev | E1 | the commit its goal started from; its note: concepts added and removed, detail added on demand | not started |
| E3, the count: `design-economy.md`, 7.3's figures at its two endpoints, the commit E2's goal started from and E2's landing; the guidance and the detail added on demand collected from the goal's and every change's records (specs, reports, questions and answers, refused deliveries) and reconciled with the goal's note, since a change that lands without a question never reaches the designer; one code review of E2's diff; as a review in `temper/docs/development/debt/reviews/` | 7.3 | temper/draft/debt | E2 | the review | not started |

## 2. Order

- **No priority mark:** the pilot isn't one of v0.1.0's critical pieces. The user approved its execution (`notes.md`, 18): E2 starts once E1 is applied, when places are free and marked work isn't about to arrive.
- **E1, then E2, then E3.** Adoption (`design-economy.md`, 9) waits for the user's word on E3's figures.
- **The area stays quiet from E2's start to its landing:** another goal changing the area's crates, its design or the tests that pin it waits for E2 to land, or E2 waits for it.
