# long-words: one long message can't stop a chat being read, and a person can write one

From `temper/docs/plan/long-words/inputs/request.md`, on `chat-reading-c`'s trunk. In jig and temper.

## What didn't hold in the request

- **A person's message doesn't stop at 10,660.** `tasks.message_bytes` bounds an agent's message. A person's stops at `people.words`, 8,192, people-v14's `AskSay.words`, and so does a reply. So "what one message holds" is 8,192 on `temper init`'s box.
- **The client does know the store's bound, on paper.** `jig/docs/design/client/domain/contract.md`, 5.1 has `me` say the most bytes a party's words may have, "so that a client says the bound before it sends"; the code never built it, and no status said so. `me-words` builds it.
- **`question.rs` holds no reply.** It parses a question's choices, whose 256 stays a button's bound. The reply is `acts.rs`' `CardWords`, then `needs.rs`' `answer`, and stops at `text`, 512.
- **There's a second hang.** Past the wire's 65,536 ceiling on a line's words, the engine's `read_response` closes the exchange, which the client takes as lost, so it retries for ever whatever the link does. `standard`'s `tasks.message_bytes` is 262,144. `read-ends` answers malformed instead, and `line-surface` raises the ceiling.
- **Widening the composer alone would drop the send.** `Thread::sending` refuses words past the thread's `text`, 512, and the chat returns with nothing said. And the view prices kept words at `lines × text`, which overflows `u32` at `long_text`, so the client wouldn't start. `pending-words` bounds them by bytes.
- **A chat's first message wouldn't read back.** The engine cuts a spec at `documents.text_bytes`, 512 on the box, and the chat's page draws what it gets, unmarked. 256 hid it.
- **An agent's and a goal's composers have the same fault,** at `text`, in `task.rs`. The rule holds there too.

## Decisions

- **The bound for a line's words is `long_text`, 131,072,** to read. Past it, on a deployment with larger models only, the page is refused and its read ends: `temper/docs/design/client/README.md`, section 11, for the user.
- **One bound for every composer:** `Me`'s `words_bytes`, within `long_text`. A new chat's could be `tasks.spec_bytes`; section 11 says so.
- **A composer holds an edit's first `long_text` bytes** and counts the characters past them. An input past that is 16 messages long; the field keeps it in a browser, and the line still says how far over.

## Parts and tasks

| Part | Task | Needs | What it makes true |
|---|---|---|---|
| `line-words` | `line-surface` | | a line's words read to `long_text`; a message and an answer encode to it; the engine encodes a shipped profile's line |
| `kept-words` | `pending-words` | | a thread's kept words stop at its bytes |
| `view-prices` | `line-prices` | `line-surface`, `pending-words` | a long line draws whole in both shells; the request's run reads; the prices hold |
| `read-ends` | `read-ends` | `line-surface` | an answer that doesn't decode, or can't be encoded, ends its read; a lost exchange still retries |
| `read-ends-page` | `read-ends-page` | `read-ends`, `line-prices` | the page says so, on a chat and an agent's page; a failed first read says so; "Latest" and the rest go on |
| `person-bound` | `me-words` | `read-ends` | `me` says `words_bytes` |

The composer's part follows in a later push.

## Overlaps

- `line-surface`, `me-words`: `jig-client-protocol-wire`'s schema, generated code and goldens meet `model-catalogue-1/offer-surface`'s, merged on its trunk: regenerated at the merge.
- `me-words`: `jig-domain-core`'s `documents.rs` and `documents/small.rs`, and `jig-protocol-engine-clients`' `translate/owner.rs`, meet `model-catalogue-1/offer-engine`'s and `offer-surface`'s, one field beside `redirect_bytes`.
- `line-prices`: `temper-client-view`'s `limits.rs` meets `ui-repo-link/repo-link`'s. `read-ends-page`: `pages/chat.rs` and its tests meet `ui-pause-close/chat-pause`, `ui-composer-focus/composer-focus` and `chat-reading-c/reading-browser`.
- The design: `temper/docs/design/client/README.md` and its status meet `chat-reading-c/reading-close`, this trunk's base. New paragraphs, not edits of its own.

## Critical path

As it stands: `line-surface` (45) → `line-prices` (60) → `read-ends-page` (40), about 145 implementer-minutes. The composer's tasks follow it.
