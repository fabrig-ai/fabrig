# browser-budget: `browser-temper` back under 85 CPU seconds

The request: `inputs/request.md`. The repair of a flagged tier, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1. No budget, timeout or tier membership changes.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `figures` | `tasks/story-baseline.md` | | The four stories' *before*, with load, and the long story's CPU split |
| `history` | `tasks/history-ctl.md` | | Lever 1: the history made through `temper ctl`, the page opened afterwards |
| `window` | `tasks/window-seam.md` | | Lever 2's surface: `Limits::window_bytes` and the shell's `small-window`, off |
| `delivery` | `tasks/delivery-cost.md` | | Lever 4: what one delivery costs at the whole bound; a fix only if small |

The rest of the plan follows in the next batch: `window-story` (lever 2 on), `find-backoff` (lever 3) and `budget-close`.

Critical path as it stands: `history-ctl`, then `window-story`, `find-backoff` and `budget-close`.

## What this plan decided

- **The four stories run by name.** The lead's rules have a spec name no slow-tier run. The request overrides that for these four stories alone, and `AGENTS.md`, 5, lets a repair run what it repairs, by name, alone. So the specs name those runs, and no others. Never the whole tier.
- **The window is its own field, off a bundle feature.** The tier can't set `document_bytes` today: it's a constant in the web shell. Lowering it in the tier's bundle would bound every document in all seventeen stories, and thirteen of them can't be run here. `Limits::window_bytes` narrows a conversation's window alone. A feature keeps the seam out of a person's bundle, as `loopback-navigation` does; nothing reads a setting at runtime.
- **`latest_returns` loses its second tab and its delegate.** Its new words come through ctl, since what it proves is the view holding, not who sent them. Its Task 2 is read by nothing in the story. `SENT` is its one typed message.

## Overlaps this plan could not avoid

`temper/docs/design/client/browser.md`, `browser.status.md`, `README.md` and `README.status.md` are also `model-check/trial-close`'s, `ui-public-2/public-page`'s, `long-words`' and `ui-proposal-card/card-close`'s: our hunks are small and in sections of their own. `window-seam` adds one literal field to `temper-client-view/src/tests/fixtures.rs` (`ui-qa-small/no-false-empties`) and a few lines to `temper-client-domain/src/pages/chat.rs` (`ui-proposal-card`, `long-words/start-reads-back`), away from what they change. `browser_pages.rs` and `temper/tests/client-browser/src/history.rs` are this plan's alone.
