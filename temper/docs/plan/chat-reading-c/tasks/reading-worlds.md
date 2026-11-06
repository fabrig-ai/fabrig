+++
id = "reading-worlds"
part = "worlds"
kind = "feature"
needs = ["chat-page", "beside-store"]
files = [
  "temper/tests/client/tests/suite/reading.rs",
  "temper/tests/client/tests/suite/main.rs",
  "temper/tests/client/src/fixtures.rs",
  "temper/tests/client/src/front.rs",
  "temper/tests/real-loop/tests/suite/terminal.rs",
  "temper/tests/real-loop/src/terminal.rs",
]
effort = "high"
minutes = 60
+++

# The user's journey runs through the real engine and store

## Goal

The request's stories on a real chat's whole path, in temper's client world. Lower tiers hold each rule, and these hold their meeting: a window, the engine's forward read, the page, the person. Nothing here changes production code but a fault a story finds, fixed in the lowest tier that shows it.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance, in `temper/tests/client/tests/suite/reading.rs`, a person on a long chat through the real engine, its pages small enough to make the window bound bite in few steps:

- **The journey:** the person reads back with the log's `Near` at its head until the first lines show. The end is gone from the tree, as it should be. `Near` at the foot brings it back, every line once, and a new turn then shows.
- **A head near reads earlier lines,** whatever moved it: there is no "Older" on the page at any step.
- **Past the bound,** the lines furthest from the view go, whole, at either end, and come back when scrolled to. A line in the tree is whole, and nothing says how many left.
- **"Latest"** from far back: pressed, the end shows and the window is the end's. A person sending from far back does the same, and sees their words.
- **New words while reading back:** a run's live text and a committed turn arrive while the person is far back. They aren't in the tree, the lines being read don't change, and they show on coming back to the end.
- **The lines bound is gone:** a chat of more than 16 short lines holds them all, read back.
- **A failed read** says so at the head, and the next `Near` reads again and gets the page.
- **In the terminal,** in `temper/tests/real-loop`: `up` shows earlier lines and `down` brings the end back, and the line PgUp makes does what `up` does. A real terminal's echo of it is read, not run.

## Inputs

- `temper/docs/design/client/README.md`, 5.15.
- `temper/docs/design/draft/chat-reading.md`, section 5.
- `temper/docs/plan/chat-reading-c/notes/common.md`.
- `temper/docs/design/domain/testing.md`, for this world's rules.

## Owns and may change

Owns the files in the header.

May also change: `skein/testing/skein-fake-person/src/lib.rs`, adding `Step::Near { find, edge }`, if the world's person has no way to act `Near` on a log: the smallest arm, beside `Press`. `skein-fake-person-browser/src/face.rs` keeps its error there.

Options, each decided:

- **The world's limits:** `fixtures::world_with_whole_turns`'s are a start. A smaller `document_bytes` and small pages make the bound bite in few turns, and budgets aren't raised for it: this tier's CPU budget is the user's. If a story can't fit it, make the chat shorter.
- **A failed read:** by the world's existing way of failing a read, or `hold_next_playground_read`'s kind. If none reaches the activity read, a one-function hold beside it in `front.rs`.
- **A story that shows a fault below** is fixed in that tier with its test, in a commit that says so, and named in your report. If it needs another task's interface changed, it's a `Question:`.

Not this task: the browser's stories (`reading-browser`), the journeys' prints.

## Steps

1. Each story, red first against what's unfixed; where everything below has landed, a story that passes at once is broken once to see it bite.
2. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world`; the real-loop's narrow test. Commit and push.

## Report

As the implementer's text asks, with each story's CPU against its tier, and any fault found below.
