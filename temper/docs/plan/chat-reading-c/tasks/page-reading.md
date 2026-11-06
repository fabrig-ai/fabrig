+++
id = "page-reading"
part = "page-part"
kind = "feature"
needs = ["log-surface"]
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page/**",
  "skein/examples/ui/shell-web/src/lib.rs",
  "skein/examples/ui/shell-web/src/reading_fixture.rs",
  "skein/tests/browser/tests/browser_real.rs",
  "skein/tests/browser/tests/suite/log_reading.rs",
  "skein/testing/skein-fake-person-browser/src/**",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 110
+++

# The page part measures a log, says its edges are near, holds the view, and takes PgUp and PgDn

## Goal

Everything a browser does for a conversation read by scrolling, generic, naming no temper: `Near` from what it measures, a log that continues, an end request, the words being read staying put in every browser, and PgUp and PgDn from anywhere on the page.

Acceptance, the pure half, in `skein-ui-web/src/tests/page`:

- **Near:** with `viewport` v, a bound log's head is near while `top < 3v/2`; a log that continues, its foot while `height - top - v < 3v/2`. `Event::Scrolled` returns `Near` when the edge comes near, again when `height` differs from the height it was last reported at while it stays near, and again when the distance to it has fallen by `v/4` or more since. Otherwise it returns nothing, and its leaving the zone forgets what was reported. The head comes before the foot, one act an event. An unbound log reports nothing. A log with `viewport` 0 (not laid out) reports nothing.
- **Measure:** an apply with patches ends with `Op::Measure` for each bound log. The first apply of a page with a bound log has one.
- **Following:** `follow_log` is `top + v >= height - 1` and no `State::Continues`. An end request that moves, or a press of `State::ToLogEnd`, makes it true with `Op::ScrollEnd` and holds it true, through `Continues` and later measures at any height, until a `Scrolled` whose `top` fell, or until the log is at its end with no `Continues`.
- **`data-log-away`** is on a log while it isn't following, a log that continues among them.
- **The view:** an apply with patches starts with `Op::HoldView` and ends (before `Measure`) with `Op::PutView` for each log not following. A log following has neither.
- **Keys:** a plain PageUp or PageDown, `in_field` or not, with no menu open and no `Role::Dialog` in the tree, gives `Op::ScrollScreen` on the first log in preorder that measured `height > viewport`, and no act. With a modifier, a menu or a dialog open, or no such log: no op. The platform half prevents the key's default where there are ops, as it does for any key.

Acceptance, real Chromium, in `skein/tests/browser/tests/suite/log_reading.rs`, as few page loads as prove it:

- **Opening:** a bound log, its owner adding a page of lines at each `Near { Head }`, opens with its foot in view and its head at least one and a half screens off, and stops reading; one with little to read shows whole and asks once a page until it has no more. The fixture counts its reads, and the story reads the count.
- **The position stays:** with `overflow-anchor: none` on the log, which is what Safari gives us, lines inserted above leave a chosen line where it was on the screen, within 1 px, and lines removed above do the same. With the browser's own anchoring on, the same, within 1 px: the two don't add up.
- **New lines while reading back** leave the view unmoved.
- **Continues:** at the foot of a log that continues, `data-log-away` is there, the view doesn't follow a new line, and `Near { Foot }` comes.
- **An end request** at a log scrolled up brings its end into view, and `data-log-away` goes.
- **Keys:** real PageUp and PageDown keys scroll the log with the focus in a text area, and with the focus on nothing (`document.activeElement` the body); with a dialog open they don't. PageUp to the head brings `Near { Head }`.

The **Not yet** line beginning `7: Reading a log` is gone from `skein/docs/design/ui-web.status.md`. What only reading checks is said in the report: this tier runs Chromium alone (`skein/tests/browser`), so Firefox and Safari are read, not run. `overflow-anchor: none` is how a story shows the keeping is the page part's and not the browser's.

## Inputs

- `skein/docs/design/ui-web.md`, section 7, *Reading a log* and *A log*.
- `skein/docs/design/ui.md`, 3.2 and section 6, *A log's edge near*.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *The page part*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1, for the browser tier's budget.

## Owns and may change

Owns the files in the header.

Readers and neighbours:

- `skein/crates/skein-ui-web/src/mapping.rs`: only if the attribute is written there.
- `skein/examples/ui/shell-web/Cargo.toml`: a dependency line, if the fixture needs one. `skein/tests/browser/tests/browser_real.rs`: the `mod` line.
- Any exhaustive match on `Op`, for the forced arm.

Options, each decided:

- **`look-round-6/log-end` has landed** before this runs, in all likelihood: `State::ToLogEnd`, `press`, `data-log-away` and `Event::Scrolled`'s attribute are in `page.rs`. If they are not on your base, add `data-log-away` and leave the press to its landing: your hunks must then merge with its (`git show origin/look-round-6/integration:skein/crates/skein-ui-web/src/page.rs`), and you say so in the report. `temper/docs/plan/look-round-6/notes/contracts.md` on that branch holds its shapes.
- **A measure from the platform half:** the borrow of the page part is released before ops are replayed (`Dom::apply`'s comment), so `Op::Measure` may dispatch a synthetic `scroll` on the element, or feed `Event::Scrolled` by whichever path keeps that true. A real scroll's event and a measure read the same three numbers.
- **What the held view notes:** the first few entries that reach into the view, deepest level you need, with `getBoundingClientRect().top`. Keep it bounded and small: it runs at every apply a person reads back through.
- **Keys with no focus in the mount:** `keydown` listens on the mount, so a press with the focus on the body misses it. Add the page's two keys on the document, as `click` is. The other keys' paths stay.
- **The browser person:** add `Key::PageUp` and `Key::PageDown` beside `Key::Tab`, and a step that reads a value from the page if `Keep` can't say a position. If that costs more than a small change there, the fixture measures and writes its result to the page, as `log_fixture.rs` does. Say which.
- **Resizing** a window measures nothing until the next event, and `ui-web.md` doesn't claim more. If a `ResizeObserver` entry for a log costs one line, take it; if not, leave it, and say so.

Not this task: `skein-ui`'s types (`log-surface`), the text shell, temper.

## Steps

1. Red first: in `tests/page/events.rs`, `a_head_within_a_screen_and_a_half_is_near`, `near_comes_again_as_the_height_moves_and_not_on_every_scroll`, `a_quarter_screen_nearer_says_near_again`, `the_foot_is_near_only_where_the_log_continues`, `an_unbound_log_says_nothing`, `an_apply_measures_each_bound_log`, `a_log_that_continues_is_away_and_is_not_followed`, `an_end_request_follows_through_continues_until_the_person_scrolls_up`, `an_away_log_holds_its_view_across_an_apply`, `pgup_and_pgdn_scroll_the_first_log_from_a_field`, `a_menu_or_a_dialog_keeps_them`.
2. The pure half; then the platform half, its ops, its measure, the document's two keys.
3. `reading_fixture.rs` beside `log_fixture.rs`, its route in `lib.rs`, and the browser stories. Run them with `heavy cargo nextest run --profile browser-skein -p skein-browser-world --test browser_real -E 'test(/log_reading/)'`.
4. Each test bites: break its rule once, see it fail for that reason, restore; the report says so. For the held view: remove `PutView`'s move, and see the `overflow-anchor: none` story fail.
5. `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web -p skein-ui-example-shell-web -p skein-fake-person-browser --all-targets -- -D warnings`, and for wasm32 as the crate's checks do; the focused tests of those. `heavy cargo check --tests` for `temper-client-shell-web` and `jig-client-view`. Commit and push.

## Report

As the implementer's text asks, with: the browser stories' wall time against their tier; what's run and what's only read, plainly, for Chromium, Firefox and Safari; what the held view notes; and whether the press path has more than one exit.
