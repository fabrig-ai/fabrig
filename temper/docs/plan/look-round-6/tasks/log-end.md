+++
id = "log-end"
part = "log-end"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui/src/tree.rs",
  "skein/crates/skein-ui/src/builder.rs",
  "skein/crates/skein-ui-web/src/mapping.rs",
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/crates/skein-ui-web/src/tests/mapping.rs",
  "skein/crates/skein-ui-text/src/**",
  "skein/examples/ui/shell-web/src/**",
  "skein/tests/browser/tests/suite/**",
  "skein/docs/design/ui.status.md",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 40
+++

# A log says when the person has scrolled away, and a button takes it back to its end

## Goal

temper's "Latest ↓" pill shows while a chat's end is out of view and, pressed, returns to it. Only skein's page part knows a log's scroll position (`follow_log`), so this generic mechanism goes into skein first; nothing here names temper.

Acceptance:

- A log's element has `data-log-away` from the moment a `Scrolled` event leaves it away from its end, and loses it when one brings it back. It survives a later `Op::Attributes` of that log, and new content while away doesn't move the view or clear it.
- A press of a button with `State::ToLogEnd` sets every log of the page following, emits `Op::ScrollEnd` for each, clears `data-log-away`, and reaches the owner only if the button is bound.
- In real Chromium: a log scrolled up gets the attribute, a press of such a button brings its end into view and clears it, and new lines then keep following.
- The builder refuses the state on any role but a button.
- This task's **Not yet** entries in `skein/docs/design/ui-web.status.md` and `ui.status.md` are gone; `ui-web.status.md` is back to `No gap.`.

## Inputs

- `skein/docs/design/ui-web.md`, 7 (*A log*), and `skein/docs/design/ui.md`, 3.2 and the mapping table beside *folds away*.
- `temper/docs/plan/look-round-6/notes/contracts.md`, *skein*, binding.
- `temper/docs/plan/look-round-6/notes/common.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1, for the browser tier's budget.

## Owns and may change

Owns the files in the header.

Readers and neighbours, for the compiler-forced arm of a new `State` only: `skein-ui-text/src/render.rs` and any other exhaustive match on `State`, in any tree.

Options, each decided:

- If `ui-rename-focus`'s commits to `page.rs` and `page/web.rs` arrive by catch-up, keep both.
- If `State` is carried through a wire or a table that lists every state, add it there as the compiler or that table's test says.
- If the page part's press path has several exits (`Press`, a shortcut's press), the state acts on each, in one function.

Not this task: a log's head (chat-reading's section 5), any temper file, `skein-ui.css`.

## Steps

1. `skein-ui`: `State::ToLogEnd`, documented in the state's words, and the builder's assertion beside `FoldsAway`'s (`builder.rs`, the `state` function).
2. `skein-ui-web/src/mapping.rs`: `State::ToLogEnd => e.set("data-to-log-end", "")`, beside `data-folds-away`, and a mapping test.
3. `skein-ui-web/src/page.rs`: where `own_element` builds a log's element, `data-log-away` while `follow_log` is false. In `Event::Scrolled`, when `follow_log` changes, push `Op::Attributes`. On a press of the state, the mechanism in the contract, in one function.
4. Test sketches in `skein-ui-web/src/tests/page/events.rs`, beside `scrolling_a_log_away_stops_its_following` and `a_log_follows_its_end`:
   - `a_log_scrolled_away_says_so_until_it_is_back`
   - `new_lines_while_away_keep_the_attribute_and_the_view`
   - `a_press_to_log_end_takes_every_log_to_its_end`
   - `an_unbound_press_to_log_end_reaches_no_owner_and_a_bound_one_does`
5. A browser story beside `skein/tests/browser/tests/suite/folds_away.rs`, on a fixture of its own beside `skein/examples/ui/shell-web/src/folds_fixture.rs`: the log scrolled up by a wheel or `scrollTop`, the attribute read, the button pressed, the end in view, a new line followed. It takes the fewest loads that prove it: the tier's budget is the user's.
6. Each test bites: break its rule once, see it fail for that reason, restore; the report says so.
7. `cargo fmt`, `python3 -I -B skein/scripts/old_forms.py`, `heavy cargo clippy -p skein-ui -p skein-ui-web -p skein-ui-text --all-targets -- -D warnings`, then the focused tests of those packages and of `skein/tests/browser`. `State` is public: `heavy cargo check -p temper-client-view --tests -p temper-client-shell-web --tests -p jig-client --tests` for its readers. Commit with the trailer and push.

## Report

As the implementer's text asks, with: the attribute and state as built, whether the press path had more than one exit, and the browser story's wall time against its tier.
