+++
id = "log-busy"
part = "page-busy"
kind = "feature"
needs = ["page-reading"]
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/tests/page/**",
  "skein/examples/ui/shell-web/src/reading_fixture.rs",
  "skein/tests/browser/tests/suite/log_reading.rs",
]
effort = "medium"
minutes = 25
+++

# A log no longer busy says again that its edge is near

## Goal

A page of lines that draws nothing moves no height, and no `Near` would come after it: the reading would stop with an earlier page unread. So `Near` also comes when an owner's `State::Busy` leaves a log whose edge is still near. An owner that keeps `Busy` through a failure hears nothing until the person moves, so a failed read doesn't loop.

Acceptance:

- The rule in the contract's *A log's busy state*, in the pure half's tests: `near_comes_again_when_busy_leaves`, `a_log_kept_busy_says_nothing_until_the_person_moves`, `near_says_nothing_if_the_edge_went_far_while_busy`.
- In real Chromium, the fixture answers a read with a page that draws nothing while its log is busy and then one that draws: the head's reading goes on through it, added to `log_reading.rs`'s opening story, with no new page load.

## Inputs

- `skein/docs/design/ui-web.md`, section 7, *Reading a log*, *Near*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *A log's busy state*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header. No other file.

Not this task: `skein-ui`; `State::Busy` exists.

## Steps

1. Red first: the three tests.
2. The rule; the fixture's page that draws nothing, and the story.
3. Each test bites: break its rule once, see it fail for that reason, restore.
4. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web -p skein-ui-example-shell-web --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui-web`; `heavy cargo nextest run --profile browser-skein -p skein-browser-world --test browser_real -E 'test(/log_reading/)'`. Commit and push.

## Report

As the implementer's text asks, with the story's wall time against its tier.
