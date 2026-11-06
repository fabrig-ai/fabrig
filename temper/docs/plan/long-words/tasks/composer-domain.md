+++
id = "composer-domain"
part = "composer"
kind = "feature"
needs = ["me-words", "line-prices", "read-ends-page"]
files = [
  "temper/crates/temper-client-domain/src/pages/composer.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages/chats.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/pages/needs.rs",
  "temper/crates/temper-client-domain/src/acts.rs",
  "temper/crates/temper-client-domain/src/tests/pages/composer.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chats.rs",
  "temper/crates/temper-client-domain/src/tests/pages/task.rs",
  "temper/crates/temper-client-domain/src/tests/pages/needs.rs",
  "temper/crates/temper-client-domain/src/tests/acts.rs",
]
effort = "high"
minutes = 75
+++

# A composer holds what a person typed

## Goal

A chat's and a new chat's composers drop an edit past 256 bytes and keep the old words, and an agent's, a goal's and a reply's past 512, with nothing said, so a pasted paragraph vanishes. Hold what was typed. Send what fits within what one message holds, and nothing past it, the words staying.

Acceptance:

- **Paste 1 KB,** on a chat's, a new chat's, an agent's and a goal's composer, and a question's reply: the words are held whole, and Send sends them whole, within `Me`'s `words_bytes`.
- **Paste past the bound,** on each: the words are held as typed, Send sends nothing, no request is pending and no line is added, the words stay, and `composer::over` says how far over. A shorter edit sends again.
- At exactly `words_bytes`, a send goes. One byte past, it doesn't.
- An edit past `long_text` holds its first `long_text` bytes, cut back to a character, and `over` still counts what lay past them. The next edit within it holds nothing beyond.
- A character straddling the bound counts whole as over. Multi-byte words count characters, not bytes.
- A new chat with a draft sends words at the bound with `WRITE_THIS_UP` after them: the bound is on what was typed.
- A suggestion's words (`Chats::suggest`) and a configuring chat's (`configure_chat`) stay within `CUE_BYTES`. So does a choice's answer, and `question.rs`' choices. A card's other words, a reason, stay within `text`, as they do.
- `thread_limits`' `text` is `long_text`. A send `Thread::sending` refuses keeps the words and sets the composer's outcome to `Outcome::Limit`, on a chat's page and an agent's or a goal's: it no longer returns with nothing said. So a composer's page says "Too many requests are pending." with the words still there.

## Inputs

- `temper/docs/design/client/README.md`, 5.4, *A composer*.
- `jig/docs/design/client/domain/contract.md`, 5.1.
- `temper/docs/plan/long-words/notes/contracts.md`, *Which bound holds what*, *The thread* and *The composer*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. May also change, these edits alone:

- `temper/crates/temper-client-domain/src/pages.rs`: the `composer` module's line; `needs::act`'s new `client` argument at its one call.
- `temper/crates/temper-client-domain/src/tests/pages.rs`: the `composer` test module's line.
- `temper/crates/temper-client-view/src/tests/chat.rs`, `a_line_break_counts_within_the_message_bound`: it pins the dropped edit, and now pins the words as typed. No other view file.
- `temper/tests/client/src/**` and other tests that send a composer's words: the least, if a test's words pass its limits' `long_text` or its `Me`'s `words_bytes`: raise that limit, and never shorten the words.

Options, each decided:

- These tests pin the old bound, and turn: `the_chat_composer_keeps_the_send_bound_when_receiving_grows` (its message `an oversized edit keeps the previous words`), `new_chats_and_cues_keep_the_send_bound_when_receiving_grows` (its cue half stays), `a_drafts_first_message_stays_within_the_cue_bound`. `temper-client-view`'s `tests/toast.rs` and `question.rs`' tests are about a choice: they stay.
- `fixtures::limits()` has `long_text` 32, and `specification_sources_stop_at_the_long_text_bound` reads it: leave the fixture, and raise `long_text` in a test whose typed words pass it.
- `Chat::send`'s `c::Text::new(..).expect("composer bounded")` goes: past the check it's `let Ok(..) else { return }`, within `long_text`, since a shell's bytes aren't this function's to trust.
- `needs::answer`'s reply bound is what `over` says. Past it, return with no toast: the view's line says it, and the words stay. Its `Toasted::NotSent` stays for what it says now.
- `Act::CardWords` follows the reply's rule only for the object `pages.needs.reply` names.
- `task.rs` holds two `words` fields (line 211's belongs to another struct): only the page's composer changes.

Not this task: any view code but the test above: Send's state, the line and its words are `composer-view`'s. A chat's first message reading back is `start-reads-back`'s. `README.status.md`: `composer-view` removes its line.

## Steps

1. Red first, each by its name: `tests/pages/composer.rs`' `held_keeps_what_fits_and_counts_what_doesnt`, `over_counts_characters_past_the_bound`. In the composers' tests, for each composer, `a_kilobyte_pasted_is_sent_whole` and `a_paste_past_the_bound_stays_and_sends_nothing`; `tests/pages/chat.rs`' `a_send_the_thread_refuses_keeps_the_words_and_says_so`.
2. `pages/composer.rs`, as the contract has it.
3. Chat, then `Chats`, `Task`, then the reply.
4. This changes what a shared bound means: besides the implementer's text's checks, run `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-world`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text asks, with each test that pinned the old bound and what it pins now, and what a person's send does at the engine's own refusal, `Limit::Words`, if you met one.
