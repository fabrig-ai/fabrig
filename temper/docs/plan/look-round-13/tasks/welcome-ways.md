+++
id = "welcome-ways"
part = "welcome"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-view/src/pages/welcome_model.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/docs/design/client/journeys/welcome-*/**"]
effort = "medium"
minutes = 20
+++

## Goal

"All ways" goes from the welcome's way pages: it goes to the same place as their "‹ Back". Only that control changes.

Acceptance: no way page of the welcome (`Key`, `Program`, `SignIn`) draws "All ways", in the browser and the terminal; its "‹ Back" still goes to Model Choose; `temper-client-view`'s focused tests and `the_committed_journeys_are_their_print` pass.

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *Model*; `temper/docs/design/client/browser.md`, where it says the foot's Back returns to Model Choose.
- `temper/docs/plan/look-round-13/notes/common.md`.

## Owns and may change

- Owns: `temper/crates/temper-client-view/src/pages/welcome_model.rs` (`all_ways` and its three calls go), its tests in `tests/welcome.rs`, the journeys' prints.
- May change: `temper-client-view/src/tests/welcome.rs`'s assertions at lines that press "All ways": they press "‹ Back" where they walked it, or drop it. `temper/tests/client-browser` stories that press it: edit and compile only. `temper/tools/gallery` goldens.
- Keeps: `Word::WelcomeAllWays` and `pages/model_check.rs`'s link: a stopped try's pane is not a way page and has no "‹ Back" (`temper/docs/design/client/README.md`, *The try*). Its tests stay.
- `accounts/words.rs`'s `AllWays` (`‹ All ways`, standalone) is `model-cards`'s, not this task's.

## Steps

1. Remove; repair tests; reprint journeys; gallery.
2. Narrow tests, `cargo fmt`, old forms, clippy, commit with the trailer, push.

## Report

As the implementer's text has it.
