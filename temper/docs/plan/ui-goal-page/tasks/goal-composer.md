+++
id = "goal-composer"
part = "goal-composer"
kind = "feature"
needs = ["goal-acts"]
files = [
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/pages/task.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goal.rs",
  "temper/crates/temper-client-view/src/pages/goal.rs",
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/tests/goal.rs",
  "temper/tools/gallery/names.txt",
  "temper/tools/gallery/src/samples.rs",
  "temper/tools/gallery/src/worlds.rs",
  "temper/tools/gallery/src/tests.rs",
]
effort = "high"
minutes = 75
+++

# What the person says to a goal

## Goal

`.sent` and the composer on a goal's page, as `temper/docs/design/client/README.md`, 5.8 and 5.11, say. `Act::MessageWords` and `Act::Send` now route on
`Shape::Tracked` too.

Acceptance:

- **`.sent`,** when the person has sent words: each `Activity::Words` by the person in the latest activity page, `p.you`, oldest first, then the pending one,
  "Being sent". No "Its agent:" line.
- **The composer,** while live and Message offered, in `GOAL_COMPOSER_BOX`: `GOAL_COMPOSER`, a `Role::TextArea` named and with the placeholder "Tell TITLE
  something", "Send" disabled while empty, Enter sends, Shift+Enter a new line; `GOAL_HINT` "Its agent reads it at once. A change of plan becomes an amendment it
  shows you first."
- **While Message is refused as `NotWaiting`,** the composer's place holds nothing. Ended, no composer.
- A sent message empties the box, as a chat's. A refusal is the toast's (`goal-acts`).
- The latest activity page reads again as the document's `activity` moves, so a sent word arrives.
- **Gallery:** `goal-sending`.
- The tests below pass, each having bitten; the crates green whole.

## Inputs

- `temper/docs/plan/ui-goal-page/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.8, 5.11; 5.1, the composer.
- The mock's `composer` in `app.js`; `choice.rs`'s `composer`.
- The domain's `pages/task.rs`, `Act::MessageWords`, `Sending::Message`, `latest`.

## Owns and may change

Owns, as `files` lists them.

Options, each pre-approved:

- **Enter, Shift+Enter, growing:** as the chat's composer does it. The stylesheet's `max-height: 220px` is `goal-look`'s; if growth takes a rule, add it under the
  marker in `temper.css`, naming it in your report.
- **Whose words:** `Activity::Words`'s party against `me`'s number. A tool's or another party's words aren't drawn.
- **A word sent and read back:** drawn once, the pending line gone with the answer, as the chat's "Being sent" does.
- **Send's icon or word:** the mock's button as the chat's composer draws it.

## Steps

1. The route; the `latest` read on a tracked page.
2. The view.
3. **Tests:** `the_composer_shows_while_message_is_offered`, `not_offered_not_drawn`, `not_waiting_holds_nothing`, `ended_has_none`, `its_placeholder_and_hint`,
   `send_disabled_while_empty`, `sent_words_oldest_first_then_being_sent`, `another_partys_words_are_not_drawn`; domain: `send_routes_on_a_tracked_task`.
4. `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-gallery`.
5. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
