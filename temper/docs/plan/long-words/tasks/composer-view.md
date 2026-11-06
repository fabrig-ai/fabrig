+++
id = "composer-view"
part = "composer"
kind = "feature"
needs = ["composer-domain"]
files = [
  "temper/crates/temper-client-view/src/pages/session.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/new_chat.rs",
  "temper/crates/temper-client-view/src/pages/agent.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/ask.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/frame.rs",
  "temper/crates/temper-client-view/src/tests/toast.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/crates/temper-client-view/src/tests/chats.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/crates/temper-client-shell-terminal/src/tests/**",
  "temper/tests/client/tests/suite/composer.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tools/gallery/**",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 60
+++

# A composer says how far over it is

## Goal

A person past what one message holds has to be told, and can't send. temper/ui's words: the words stay as typed, Send is disabled, and a line says how far over they are.

Acceptance:

- On a chat's, a new chat's, an agent's and a goal's composer, and a question's reply: past `words_bytes`, Send is `State::Disabled`, Enter doesn't send (no `State::SubmitsOnEnter`), and a line under the composer reads "That's 3,200 characters over what one message holds.", the number grouped by thousands, or "That's 1 character over what one message holds." Within it, no line, and everything as it was.
- A chat's composer holds its words as typed: a browser's field isn't written (`Value`'s `written` doesn't move on an edit), and the terminal prints them.
- The terminal prints the line, and a Send past the bound sends nothing.
- In `temper/tests/client`, through the real link and engine: a kilobyte pasted into a chat sends and reads back whole as its line; a paste past the bound shows the line, with Send disabled, and an edit back within it sends. The same on a question's reply, and on a new chat's composer up to the disabled Send.
- With no `Me` read, no line, as nothing's known.
- The line `5.4, *A composer*` leaves `temper/docs/design/client/README.status.md`'s **Not yet**.

## Inputs

- `temper/docs/design/client/README.md`, 5.4, *A composer*.
- `skein/docs/design/ui.md`, 3.4, *A value*.
- `temper/docs/plan/long-words/notes/contracts.md`, *The composer* and *The view's words*: binding.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `README.status.md` is shared: remove your line alone. May also change, these edits alone:

- `temper/crates/temper-client-view/src/tests/fixtures.rs`, `temper/tests/client/src/**`: a helper for a composer's state, if none serves.

Options, each decided:

- The line is a `Role::Status`, in the look a composer's refusal has beside it (`super::chats::outcome_text`, `Words::status`). No new style and no change to `temper.css`: temper/ui reviews the words, and may ask for a look.
- Where it stands: after the hint on a chat's and a new chat's composer (`session::hint`), and after the composer's acts on an agent's, a goal's and a reply's.
- `Word::ComposerOver`'s number is a `Hole::Name` that `session::over` formats: `Hole::Number` writes plain digits, and the phrase has no grouped form. Its ids in `words.rs` are the next free ones. If a merge finds another draft took them, take the next.
- `ask.rs`' `button` takes no state: open the Send button by hand there, or give `button` a parameter, whichever changes fewer of its callers.
- A gallery sample `chat-composer-over`, for temper/ui's review: add it with its line in `names.txt`, as its neighbours have, if `temper/tools/gallery/src/tests.rs` asks no more than a `Holds` row. If it asks a journey or a mock's mapping, drop the sample and say so.

Not this task: a chat's first message reading back: `start-reads-back`'s. The domain, but a read-only accessor a view needs. No browser story.

## Steps

1. Red first: the view's `past_the_bound_send_is_disabled_and_a_line_says_how_far`, on each of the five, with 1 and 3,200 characters; `within_the_bound_nothing_changes`; `no_me_no_line`. The terminal's `a_composer_past_the_bound_prints_its_line`. The world's `a_paragraph_pasted_into_a_chat_reads_back`, `a_paste_past_the_bound_stays_and_says_how_far_over`, `a_long_reply_is_answered`.
2. `session::over`, the two words, the five composers.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery`; the world's narrow tests. Commit and push.

## Report

As the implementer's text asks, with the line's two words as they read, and where each stands on each composer, for temper/ui.

## Answers

2026-11-06. *May this task wrap the reply's toast in a landmark in `frame.rs`, or should the toast's owner repair it first?* This task wraps it. `a_long_reply_is_answered` is this task's acceptance and can't pass while the toast is a child of the page (`skein/docs/design/ui.md`, 14), and no task in this plan owns the toast.

- `frame::toast`, these edits alone: after its `let Some(toast)` return, open a `Role::Region`, `b.name` a new `Word::ToastRegion`, "Notices" (as `Word::AccountRegion` names its region); close it after the toast's `b.close()`. The `Role::Status` keeps `look::TOAST` and everything in it. The region has no style, and `temper.css` doesn't change: the toast is fixed, so the region takes no room. A view with no toast draws no region.
- `Word::ToastRegion` takes the next free id in `words.rs`. `root.rs` doesn't change.
- `tests/toast.rs`: add `skein_ui::outline::check(view.tree())` is `None` to `toast_is_last_on_home_and_chats_in_both_shells_and_undo_is_only_held`, red before the wrapper. Its other assertions stay as they are: the toast stays last, with the status role.
- `temper/docs/design/client/README.md`, 3.5, *The toast*, already says this: no design edit is left for the task.
- The report names "Notices" for temper/ui, beside the line's two words.
- `host::redacted::a_committed_journey_redacted_replays` stays under `## Left`: no recording is edited for this task. The gallery's `chat-composer-over` stays dropped, as the spec's option says.
- Finish by running `a_long_reply_is_answered` to its end, and `temper-client-view`'s and `temper-client-gallery`'s tests.
