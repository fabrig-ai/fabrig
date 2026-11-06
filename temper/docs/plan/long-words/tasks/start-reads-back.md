+++
id = "start-reads-back"
part = "composer"
kind = "feature"
needs = ["composer-view"]
files = [
  "temper/crates/temper-client-domain/src/pages/chat/request.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/chat.rs",
  "temper/tests/client/tests/suite/composer.rs",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 45
+++

# A chat's first message reads back whole

## Goal

A new chat's words are its task's spec, and the engine's document cuts that at `documents.text_bytes`, 512 on `temper init`'s box. The chat's page draws what it gets, unmarked, which a 256-byte composer hid. Read the rest, as a goal's page does.

Acceptance:

- A chat whose task's spec comes marked cut reads it through `Read::TaskText` from the start, piece by piece, and draws it whole as its request. One that comes uncut asks nothing.
- Until it has it all, and if that read ends for good, the request draws what it has, ending in `…`. It is never cut silently. A transient end asks again on activity's waits (`Asking`); a lasting end stops.
- A piece that doesn't follow the last, or a total past `long_text`, stops the read as `goal/specification.rs` stops its own, and the request draws what it has, ending in `…`.
- In `temper/tests/client`, through the real engine: a chat started with a kilobyte of words shows them whole, at an engine whose `documents.text_bytes` is below that.
- The line `5.4, *A chat's first message*` leaves `temper/docs/design/client/README.status.md`'s **Not yet**.

## Inputs

- `temper/docs/design/client/README.md`, 5.4, *A chat's first message*.
- `temper/crates/temper-client-domain/src/pages/goal/specification.rs`: the same read, for a goal.
- `temper/docs/plan/long-words/notes/common.md`.

## Owns and may change

Owns the files in the header. `README.status.md` is shared: remove your line alone. May also change, these edits alone:

- `temper/tests/client/src/**`: the world's `documents.text_bytes`, on the one world that needs it low.

Options, each decided:

- A module of its own, `pages/chat/request.rs`, with its own small reader. Don't move `goal/specification.rs` or share it: that one ends in a draft's provenance, and it isn't this plan's. If the two turn out nearly one, say so in the report.
- Its handle is the chat's next fixed one, after `goal_read`, if its handle count allows; `handles` is 19 and this asks no watch. A read, like `goal_read`, with `Told::Page` routed to it. If the fixed numbers would collide with `self.next`, take one from `self.next`.
- `request_message`'s words come from a `Chat` accessor giving the bytes and whether they're whole. The mark is appended there, in the view, as `jig`'s `client/domain/view.md`, section 7 has a cut.
- The read goes out when the task's document first shows the spec cut, not again on later deliveries of the same task.

Not this task: an agent's or a goal's spec: they draw through their own readers. A chat's heading, whose title is a marked document text.

## Steps

1. Red first: `a_cut_request_is_read_whole`, `an_uncut_request_asks_nothing`, `a_request_read_that_ends_keeps_what_it_has` in `tests/pages/chat.rs`; the view's `a_request_being_read_ends_in_the_mark`, `a_whole_request_draws_whole`; the world's `a_new_chats_kilobyte_reads_back_whole`. The world's red shows the first 512 bytes, unmarked.
2. The reader, its routing, the accessor, `request_message`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal`; the world's narrow test. Commit and push.

## Report

As the implementer's text asks.
