# long-words: the shapes that cross tasks

Binding. Each task builds its half to these, and needs no other task's words for them.

## Which bound holds what

| What | Who writes it | The store's bound (`temper init`'s box) | The client's |
|---|---|---|---|
| A person's message or reply | a composer, `temper ctl` | `people.words` (8,192), `AskSay.words`' ceiling | `long_text` to read; `words_bytes` to send |
| An agent's message, question or answer | a run | `tasks.message_bytes` (10,660) | `long_text` to read |
| A result | a run | `tasks.result_bytes` (10,660) | `long_text` to read |
| A new chat's first words | a composer | `tasks.spec_bytes` (131,072) | `words_bytes` to send; `TaskText` to read |
| A suggestion's words, a choice | a cue, a button | `CUE_BYTES` (256), the forge wire's | `CUE_BYTES` |

`long_text` is 131,072 on the browser, the terminal and `temper ctl`. `text` stays 512 on the browser and the terminal, 32,768 on `temper ctl`, for every other field.

## The client wire (`client-v19`, changed in place)

```
record Word { ... words: text 262144 ... }            # 65536 before
record ActivityWords { ... words: text 262144 }       # 65536 before
record Me { ... key_bytes: u32 redirect_bytes: u32 words_bytes: u32 }
```

262,144 is the largest `tasks.message_bytes` a shipped profile holds (`standard`), so the engine can encode any line a shipped profile's store took. `Message.words` and `AnswerQuestion.words` keep their ceiling, 65,536, above people's 8,192.

## The link's bounds (`jig-client-protocol-link/src/limits.rs`)

`word_words`, `activity_words_words`, `message_words` and `answer_question_words` leave the `text` class and join `narrow(client.long_text, ...)`, which keeps a member's lower ceiling. `translate/pages.rs` gains:

```rust
/// A line's words, whole within `long_text` (client/domain/client.md, 14).
fn long_text(value: &[u8], limits: &Limits) -> Result<c::Text, w::Problem>
```

`word_fields` and `activity_fields`' `Words` arm take it. Every other caller of `text` stays.

## Reads

| What happened | Jig read ends | Family read ends |
|---|---|---|
| The answer came and did not decode, whatever `Undecodable` says | `ReadNone::Malformed`, with `Fact::UndecodableDocument` | `Readout::Family { record: None }`, with the fact |
| The exchange was lost, or never sent for want of room | `ReadNone::Unreachable` | `Readout::Family { record: None }` |

`failed` in `component.rs` keeps every lost exchange. `read_answer`'s decode arm stops calling it for a read. The clients' component answers `routes::read_none(Why::Malformed, ...)` where `read_response` now closes.

## The thread (`jig-client`)

```rust
impl KeptWords {
    /// The bytes of the words kept.
    #[must_use]
    pub fn bytes(&self) -> u64
}
```

`Thread::sending` returns false, keeping nothing, when `words` passes `limits.text` (as now) or `self.words.bytes() + words` passes `limits.bytes`. temper's `thread_limits` sets `text: long_text`.

## `Me`

```rust
pub struct Me {
    ...
    /// The most bytes a party's words may have: a task's spec, a message, an answer.
    pub words_bytes: u32,
}
```

The engine fills it from `limits.people.words` in `jig-domain-core`'s `documents/small.rs`, on its `Frame`, beside `redirect_bytes`.

## The composer (`temper-client-domain/src/pages/composer.rs`, new)

```rust
//! What a person types into a composer, held as typed (client/README.md, 5.4).

/// An edit's first `hold` bytes, cut back to a character, and the characters past them.
#[must_use]
pub fn held(words: Box<[u8]>, hold: u32) -> (Box<[u8]>, u64);

/// The characters of what was typed past `bound` bytes: none within it.
#[must_use]
pub fn over(words: &[u8], beyond: u64, bound: u32) -> u64;

/// What one message holds on this client: `Me`'s `words_bytes`, within `long_text`.
#[must_use]
pub fn bound(me: &c::Me, limits: &Limits) -> u32;
```

A character is a byte that isn't a UTF-8 continuation byte (`byte & 0xc0 != 0x80`), as `needs.rs`' `reply_label` counts them. `over` counts from the first character that doesn't fit whole within `bound`.

Each composer keeps its `words: Box<[u8]>` where it is and gains `beyond: u64` beside it: `Chat`, `Chats`, `Task` (an agent's and a goal's page), and `acts`' `Card`, for a question's reply alone. An edit sets both from `held(words, long_text)`. A send asks `over(...)` first and sends nothing past the bound, the words staying. A send clears both.

Before `Me` is read no composer knows its bound: `over` isn't asked, and sending stays as it is.

## The view's words (`temper-client-view/src/words.rs`, `pages/session.rs`)

```rust
Word::ComposerOverOne   // "That's 1 character over what one message holds."
Word::ComposerOver      // "That's {Hole::Name} characters over what one message holds."

/// The line under a composer past what one message holds (client/README.md, 5.4).
pub(crate) fn over(b: &mut Builder, w: &Words, characters: u64);
```

The number is grouped by thousands ("3,200"): `Hole::Number` writes plain digits, so `over` formats it and passes a `Hole::Name`.

## The view's prices (`temper-client-view/src/limits.rs`)

- A person's words waiting to be read: nodes `max_nodes(bytes)` plus ten a line; text `bytes`, not `lines × text`.
- A chat's request: nodes `(long_text + 1) × 2` plus its fixed room; text `long_text` once more in `text_bytes`.
- A composer's value: `long_text` once more in `text_bytes`.
- `text_per_node` stays: `turn × 2` already holds `long_text`.
