# ui-chat-accept: the plan

Accepting a settings proposal in the chat that made it: `temper/docs/plan/ui-chat-accept/inputs/request.md`, the walk's B1.

## The cause

Read, not yet run; `chat-decides` shows it red first.

- `temper/crates/temper-client-domain/src/acts.rs`, `send`, decides under `card.project`.
- `pages/chat.rs`, `proposals`, gives its card `task.project`, 1 on a fresh box. `pages/needs.rs` gives its card `proposal.project`, 0.
- `jig/crates/jig-domain-core/src/routing/party/decisions.rs`, `task_settings_decide`: `project != people::DEPLOYMENT` is `Refusal::Unknown`, whose words are "No visible object with that name". The decision never reaches its judgment.

**It never worked; it isn't a regression.** The engine's check predates 2026-10-31 (`d969a1ff74` only moved it). The chat's card first came with `3fcbdfe16b`, 2026-11-05, already under `task.project`. `ui-walk-large` added `projects` and left `project` at 0, which `configuring_chat.rs` pins; M3 changed only Reject's `running`. The second walk didn't press this primary.

**Why the client and not the engine:** a settings proposal belongs to the deployment and jig is right to know it only there. The card already reads the proposal, which names its project.

## The decisions

- **Reject:** the request asks for both of the card's acts, reading "the unavailable direct Reject control" as the same family. It isn't: the walk found no control because 5.13 and the mock's `chat.html` draw only Accept and "Change something". The mock is the target, so no Reject is drawn. The same fix corrects Reject's request, a world holds it, and a person rejects from Needs you. To draw one later is a view change and a design line.
- **After a rejection** the chat's card shows neither acts nor a state line (`cards/proposal.rs`, `settings`). No design covers it and it isn't changed here; `chat-walk` records what's seen.
- **No protocol or stored-format change,** so nothing for temper/forge.
- **`finding-400` isn't the same cause.** B1 is refused before anything is accepted. `finding-400` is a startup fault after acceptance (`temper-worker-domain`'s `agents.rs` drops a fault before admission), and at 12:47Z it stood on a `Question:` about jig's host contract, not landed. `chat-walk` shows whether a fresh box meets it, and doesn't repair it.

## The tasks

| # | Task | Needs | Effort, minutes | What it gives |
|---|---|---|---|---|
| 1 | `chat-decides` | | high, 40 | the fix, red first in the domain and the joined world; Accept and Reject from the chat |
| 2 | `chat-walk` | 1 | high, 50 | the whole path in a browser at 1440 and 420; `walk.md`; the screenshots |

The critical path is 1 then 2. Design moved with this cut: `temper/docs/design/client/README.md`, 5.13, and its status.

## Overlaps

- `temper/docs/design/client/README.status.md` with `chat-reading-c/reading-close` and `model-check/trial-close`: separate entries. Unavoidable: this is where the gap is recorded.
- `temper/tests/client/tests/suite/main.rs`, one `mod` line. The tests are in a new `chat_accept.rs` to stay off `configuring_chat.rs`, which `model-catalogue-1/configure-picker` changes.
- `configuring-words-2` and `ui-pause-close` change this chat's words and bar: `chat-walk` may need a catch-up, and has no tracked code to conflict.
