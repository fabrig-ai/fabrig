# ui-proposal-card: a settings proposal's card in a chat, as a real model makes it

The request: `inputs/request.md`. Its design is on the trunk with this plan: `temper/docs/design/client/README.md`, 5.13 *The card
in a chat*, and `skein/docs/design/ui-web.md`, 7 *A focus request*. This plan is released in pieces; this file follows each push.

## Parts and tasks so far

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `card-reason` | `tasks/reason-markdown.md` | | The request's case 4: the reason's Markdown in the card, and the question line without a heading's marks |
| `composer-reveal` | `tasks/focus-reveal.md` | | A focus request brings its node into view, even where it already held the focus |
| `card-change` | `tasks/change-something.md` | `reason-markdown` | The request's case 5: "Change something" asks "What should change?" in the composer |

Still to come: cases 1, 2 and 3 (a card's place, its state lines, a decided card after a reload), the render proof and the closing of
the status entries.

## Critical path, as it stands

`reason-markdown`, then `change-something`, since both write `temper-client-view` and a crate has one writer at a time; `focus-reveal` runs
beside them. `card-proof`, last, renders them together.

## What this plan decided

- **The focus request's scroll is skein's.** A browser's `focus()` moves nothing on an element that already holds the focus, so that part of
  case 5 is a generic mechanism, in skein first, and temper's composer gets it through the focus count it already moves.
- **`Act::FocusComposer` goes.** Its one producer was the card, so `Act::ChangeProposal { proposal }` replaces it.
- **The Needs you question line drops a heading's marks too.** The request names the card, but the same reason leads that card's question
  line through `pages/ask.rs`, and it would say "## temper enforces" there.

## Overlaps not avoided

- `temper-client-view/src/pages/chat.rs`, `temper-client-domain/src/pages/chat.rs` and `words.rs`: `long-words`' composer tasks and
  `chat-reading-c`. Expect catch-ups around `session_composer`'s placeholder line and `Chat::act`.
- `skein-ui-web`'s `page.rs` and `page/web.rs`: `chat-reading-c`, whose `Op`s sit beside `Focus`.
