# ui-proposal-card: a settings proposal's card in a chat, as a real model makes it

The request: `inputs/request.md`. Its design is on the trunk with this plan: `temper/docs/design/client/README.md`, 3.5's `Chat { task }` and
5.13's *The card in a chat*, *Where a card stands*, *A decided card after a reload* and *"Decision made"*; `jig/docs/design/client/domain/contract.md`, 5.1,
*an activity line*; `skein/docs/design/ui-web.md`, 7, *A focus request*.

## Parts and tasks

| Part | Task | Needs | Request's case | What it makes run |
|---|---|---|---|---|
| `card-reason` | `tasks/reason-markdown.md` | | 4 | The reason's Markdown in the card; the question line without a heading's marks |
| `composer-reveal` | `tasks/focus-reveal.md` | | 5 | A focus request brings its node into view, even where it already held the focus |
| `card-change` | `tasks/change-something.md` | `reason-markdown` | 5 | "Change something" asks "What should change?" until something is sent or it is decided |
| `proposal-facts` | `tasks/proposal-facts.md` | | 1, 3 | A `Decided` line names its proposal and its making; an ended proposal keeps its reason and `projects` |
| `card-reads` | `tasks/card-reads.md` | `change-something` | 2 | A card's read that ends without a value is asked again; cards hold number order |
| `card-states` | `tasks/card-states.md` | `change-something` | 2 | The route proved; rejected and withdrawn lines; a decision from elsewhere reaches the card; the ignored test runs |
| `card-place` | `tasks/card-place.md` | `proposal-facts`, `card-reads`, `card-states` | 1 | A card stands after the turn running when it was made |
| `card-reload` | `tasks/card-reload.md` | `card-place` | 3 | A decided card stays at its place after a reload, with no row beside it; the session's snapshots go |
| `card-proof` | `tasks/card-proof.md` | `card-reload`, `focus-reveal` | Proof | The render at 1440 and 420 on a fake box, and what it finds |
| `card-close` | `tasks/card-close.md` | `card-proof` | | The status entries closed; the stale journeys recorded anew |

## Critical path

`reason-markdown`, `change-something`, `card-states`, `card-place`, `card-reload`, `card-proof`, `card-close`: about 450 implementer-minutes, since each writes
`temper-client-view` and a crate has one writer at a time. `focus-reveal` (skein) and `proposal-facts` (jig) run beside it from the start, and `card-reads` (the domain) beside
`card-states` (the view).

## What this plan decided

- **A card's place comes from jig's activity line, not a clock.** jig already writes a `Decided` line when a proposal is made, which is why "Decision made" showed too early. That line
  now names the proposal and says it was its making. A turn holds its calls' results and commits as it ends, so the making line precedes the turn that made it: the card stands after the first
  committed turn following the line, and the reply, a later completion, follows the card. Commit position needs no time comparison, which ties within a step and a resent turn could upset. A making time
  (`waiting.since`) is the fallback for a pending card with no line in the window, a delegate's among them. So no `made` time is added to an ended proposal: the request allowed one, and the line does its work.
- **The same line brings a decided card back.** A task's document lists only waiting proposals, so nothing told a reloaded chat of its decided ones. The line does, in the window where the card would stand, so
  the card comes with its place and costs a read per line in the window, with no new read and no list on the document. Past jig's bound (`inbox_messages`, 82 on standard and 8 on small) its read answers `Gone`: no card
  stands, and that proposal keeps its row.
- **The session's memory of a proposal goes.** `Needs`' snapshots and `Chat.settings_proposals` existed so a return could draw a decided card. With the card read back from jig they have no use, and a restored snapshot,
  acts cleared and page unread, is the likeliest route to the false "Waiting for an owner". `card-states` proves that first, as the request asks.
- **Calls don't name their proposal.** Confirmed by reading: `jig-protocol-transcript`'s `translate_block` leaves `objects` empty and nothing fills it, though `jig/docs/design/domain/engine.md` says the core writes what a call made
  beside its turn (that engine status records the gap). Naming it isn't cheap: a new field on the core's turn record and the archived turn, the link filling `objects`, and a mapping from a call's position to its number that no one
  has checked; verdict and procedure proposals have no call at all. It would also place the card almost exactly where the line does. So the anchor stays for a call that names one, and this plan doesn't fill it.
- **The rejection's refusal looks stale.** `temper-client-domain`'s `acts.rs` sends `running` on Accept alone, and `chat_accept.rs` rejects with no refusal. `card-states` un-ignores the test and says what it found.
- **The focus request's scroll is skein's.** A browser's `focus()` moves nothing on an element that already holds the focus, so that part of case 5 is generic, and temper's composer gets it through the focus count it already moves.
- **`Act::FocusComposer` goes.** Its one producer was the card, so `Act::ChangeProposal { proposal }` replaces it.
- **The Needs you question line drops a heading's marks too.** The request names the card, but the same reason leads that card's question line through `pages/ask.rs`, and it would say "## temper enforces" there.
- **A rejected card stays "Rejected" with a later proposal below it;** only a withdrawn one reads "Replaced by the proposal below", and replaced no longer needs an anchor: a settings card with a higher number replaces it.
- **Journeys are recorded once, last,** in `card-close`, since `proposal-facts` and `card-reload` both change the client's exchange. This departs from the request's "in that task", because recording earlier would be thrown away.

## Formats changed, with no compatibility

As the request allows: the store's `tasks-v9` (`DecidedLine` gains `proposal`, `ProposedSettings` gains `projects`) and the wire's `client-v19` (`ActivityDecided` gains `proposal`, `ProposalHistory` gains `reason`,
`RetainedActionSettings` gains `projects`), in place under their names.

## Overlaps not avoided

- `temper-client-view/src/pages/chat.rs`, `temper-client-domain/src/pages/chat.rs` and `words.rs`: `long-words`' composer tasks and `chat-reading-c`, whose window `card-place` takes as it is on a catch-up, its keyed lines and no `Line::Left` among it.
- `jig-client-protocol-wire`'s `client-v19.schema`, generated code and goldens: `long-words/me-words` and `chat-reading-c`. Regenerated on a catch-up, never hand-merged.
- `skein-ui-web`'s `page.rs` and `page/web.rs`: `chat-reading-c`, whose `Op`s sit beside `Focus`.
- `temper/docs/design/client/README.md` and its status: `chat-reading-c`, `model-catalogue-1`, `model-check` and `ui-repo-link`, in other lines.
- `temper/tests/client/tests/suite/configuring_chat.rs`: `model-catalogue-1/configure-picker`, merged on its trunk; and `configuring-walks` is queued.
