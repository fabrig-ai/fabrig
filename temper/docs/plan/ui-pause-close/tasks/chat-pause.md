+++
id = "chat-pause"
part = "chat-pause"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chats.rs",
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/chat.rs",
  "temper/tools/gallery/**",
]
effort = "high"
minutes = 90
+++

# A chat's Pause and Resume, its "Paused", and a message that resumes it

## Goal

A chat you stop has no way back, and a message to it sits with no run. Make a chat's bar say "Pause" and then "Resume", its
status "Paused" on the person's own pause, its composer's hint "Sending resumes this chat.", and a message to it one act with
its release. "Applied" no longer stands under the conversation after a Pause or Resume.

Acceptance:

- **The bar:** "Pause" where Stop is allowed and the chat isn't paused; "Resume" in its place on a paused chat where Release is
  allowed; then "Close". A chat held for any other reason says "Held" and has no "Resume". Browser and terminal alike.
- **The status:** "Paused" on a person's own pause, on the bar and on its Chats and Home rows.
- **The message:** to a paused chat, one `Ask::Compound` of the words and the release, with the hint saying so; to any other
  chat, a lone `Ask::Message`. Refused, the chat stays paused and the composer says why.
- **"Applied"** isn't drawn after an accepted Pause or Resume; a refusal still says why there.
- **A world test** on temper's real engine root: Pause, Resume, Pause, send a message, see it answered.
- **The gallery state** `chat-paused`, in `names.txt`, `samples.rs` and `tests.rs`.
- **A real browser check** on a fresh `temper box --fake-backends`: start a chat, Pause, Resume, Pause, send a message, see it
  answered. Screenshots at 1440 and 420 wide of each state in `~/src/fabrig/ui-pause-close/` on your builder, outside any
  worktree, never committed; the report says which builder.

## Inputs

- `temper/docs/plan/ui-pause-close/notes/common.md`.
- `temper/docs/plan/ui-pause-close/notes/contracts.md`: *A person's pause, in the domain*, *The asks*, *Words*, *The chat's
  status*, *Gallery names*. They bind.
- `temper/docs/design/client/README.md`, 3.5, *A person's pause* and the chat's bar; 3.6, *Closed ends on one line*.
- `jig/docs/design/client/domain/contract.md`, 3.4 (a compound request), 5.6 (placed holds).
- `temper/docs/design/client/browser.md`, 7.2, for the gallery's states.
- `temper help box`, for a box on fake backends.

## Owns and may change

**Owns:** the `files` above.

**Readers, each edited only as said:**

- `Sending` gains `Resume`: compiler-forced arms wherever it is matched (`pages/chat.rs` in the view lists it in `act_status`).
  `Chat.pending` is `List::with_capacity(4)`, one request per kind: make it 5.
- `Word::GoalPause` and `Word::GoalResume` are renamed: `temper-client-view/src/pages/goal.rs` takes the new names, nothing else
  there. `Word::Stop` stays until `close-words`, for `pages/goal.rs` and `pages/agent.rs`.
- `session::hint` is read by `pages/new_chat.rs` and `pages/configure.rs`: call sites only, if its signature changes. Default:
  give it the `Word` to say, each caller choosing.
- `temper-client-domain/src/pages/task.rs`: its private `chip_hold` may become `pub(super)` for the chat's Resume, nothing else.
- A declared count of the word table (`english_declarations`, its limits tests) moves with the new words, as a counted baseline.

**Pre-approved options:**

- **The journeys' prints and recordings** (`temper/docs/design/client/journeys/**`) go stale here, since they print a chat's
  "Stop". They aren't yours: `close-words`, which needs this task, re-records them once. If a test reading them goes red, list
  it under `## Left` and go on.
- **If the world test finds the real engine refusing** the compound on a person's own hold, or `Release` not offered on it: that
  contradicts `contract.md`, 3.4: read jig's admission first (`jig/crates/jig-domain-core/src/routing/party/`), and only if it
  truly refuses, report `Question:` with what you found.
- **If a world can't move a hold's revision between the read and the send:** the refusal is tested in the domain alone, with an
  answered `Refused`; jig's `jig/tests/tasks/tests/lifecycle/compound.rs` keeps all-or-none.
- **If `Stop` is offered on a chat held for another reason:** draw "Pause", as today's bar draws "Stop". That is the request's
  rule, "Pause" where Stop is allowed.

**Not yours:** a goal's and an agent's task's words and acts, `confirm.rs`, `docs/design/`, `docs/help/`.

## Steps

1. **The domain's failing tests,** in `temper-client-domain/src/tests/pages/chat.rs`, as its neighbours are written
   (`f.chat.act(...)`, `fixtures::acts::sent(&mut f.out)`):
   - `paused_is_the_person_s_own_stop_alone`: own placed stop over its subtree is; a hold of another person's, one with its scope
     another task's, a lifted one, a drain, an `Exceeded` hold and a live chat aren't.
   - `resume_on_a_paused_chat_releases_at_its_hold_revision`; and nothing sent where Release isn't offered or the chat isn't paused.
   - `a_message_to_a_paused_chat_sends_its_words_and_release_together`: one ask, one key, in the contract's order, the thread's
     words `BeingSent`, and answered `Compound`, `Sent` at `first_message`.
   - `a_message_to_a_chat_held_otherwise_is_words_alone`.
   - `a_refused_message_to_a_paused_chat_stays_in_the_composer_s_refusal`: answered `Refused`, the words refused, the outcome kept
     with `Sending::Message`, nothing further sent.
2. **The domain:** `chat::paused`, `Sending::Resume`, `Chat::act`'s `Act::Resume`, the message's ask in `Chat::send`. `me` is read
   before the ask is built. In `pages.rs`, `Act::Resume` on an open chat goes to the chat, before the arm that sends it to a
   tracked task's page, as `Act::Stop if pages.chat.is_some()` does.
3. **The view's failing tests,** `temper-client-view/src/tests/chat.rs` and `chats.rs`, in both shells:
   `a_running_chat_says_pause_and_close`, `a_paused_chat_says_resume_paused_and_its_hint`,
   `a_chat_held_otherwise_says_held_and_has_no_resume`, `an_accepted_pause_or_resume_says_no_applied`, `a_refused_resume_says_why`,
   `a_paused_chat_s_row_says_paused`. The existing tests reading `b"Stop"` on a chat (about lines 851, 2348, 2356, 2381) read
   `b"Pause"`, with `Act::Stop` still its act. The terminal's `src/tests/fixtures/chat.rs` and its golden follow.
4. **The view:** in `pages/chat.rs`, `bar_actions` takes `root` and draws as the contract says, `Word::Pause` bound to `Act::Stop`,
   `Word::Resume` to `Act::Resume`; `session_composer` picks `Word::ChatSendResumes` on a paused chat where Release is allowed, in
   place of its usual hint; `act_status` hides an accepted Stop's or Resume's outcome as it hides Close's, by an exhaustive match
   (no `matches!`). In `pages/chats.rs`, `status_words` as the contract says.
5. **The world test,** `temper/tests/client/tests/suite/chat.rs`, `a_chat_pauses_resumes_and_a_message_resumes_it`, after
   `a_message_in_a_chat_is_answered_in_its_conversation`'s shape: press "Pause", see the status "Paused" and "Resume", gone
   "Pause" and "Applied", see "Sending resumes this chat."; press "Resume", gone "Paused" and "Applied"; "Pause" again; type and
   press "Send"; see the reply in the conversation; assert the engine's task isn't `Phase::Held` and the words reached its inbox
   once. Run it first against the unfixed code to see it fail.
6. **The gallery:** `chat-paused` through `tools/gallery`'s samples, with a task whose chip carries the person's placed stop and
   Release, Stop, Message and Cancel offered. `Holds` in `tests.rs` names "Resume", "Close", the status and the hint.
7. **The browser check** of the acceptance, by a throwaway driver (`notes/common.md`, *Render proofs*). Fix what it finds; the
   status token and hint take the rules they have, and a new rule goes under `/* ==== ui: chats ==== */`.
8. **Checks, once:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p
   temper-client-view -p temper-client-shell-terminal -p temper-client-world -p temper-client-gallery --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-gallery`;
   the world's `heavy cargo nextest run -p temper-client-world --test suite -E 'test(chat) | test(task)'`. This changes what a
   chat's status says, so it also runs `temper-client-shell-web`'s tests.
9. Commit in small steps that say why, and push.

## Report

As your text says, and: which builder holds the screenshots and their directory; the rule each new test bit on and the failure
seen; what jig does with the compound, from the world test; anything under `## Left`, the journeys' staleness among it.
