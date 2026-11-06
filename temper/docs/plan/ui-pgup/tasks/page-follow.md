+++
id = "page-follow"
part = "log-page"
kind = "feature"
needs = ["reproduce"]
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 40
+++

# A key's scroll counts at once, and a press to the log's end leaves the focus

## Goal

PgUp while a log follows its end must stop the following before the next delivery can restore the foot, and PgDn that reaches the foot must start it again. Today both wait for the browser's scroll event, which comes on the next frame. A wheel doesn't have this gap, so the key path is the one to fix, and they then match. A pointer's press on "Latest ↓" must also leave the focus in the composer. Both are generic rules of skein's page part: nothing here names a chat.

Acceptance:

- `Page::key` follows `Op::ScrollScreen` with `Op::Measure` for the same log, in the same ops.
- `Page::press_keeps_focus` says true for a node with `State::ToLogEnd` and false for any other, and the web half prevents a `mousedown`'s default on such a node, so the focus doesn't move.
- The tests below pass, in `skein-ui-web`'s own tests, and no browser story is added.
- The **Not yet** bullet this plan added to `skein/docs/design/ui-web.status.md` is gone.

## Inputs

- `temper/docs/plan/ui-pgup/notes/common.md`, then `temper/docs/plan/ui-pgup/reproduced.md`.
- `skein/docs/design/ui-web.md`, section 7: *A log*, and *Reading a log*: *Following*, *The view stays*, *Keys*. Those lines are written; you build them.
- `skein/docs/foundation/programming-model.md`, section 10, for `page.rs`, which is step code.

**Departure from `common.md`:** the `Op::Measure` goes in whatever `reproduced.md` counts for the race. The platform's rule makes the gap real, a sampled run can miss it, and a count of 0 shows only that it was rare. If the count is 0, say so in your report. The focus rule follows `common.md`: if `reproduced.md` shows the focus staying in the composer through a pointer's press on "Latest ↓", leave out `press_keeps_focus` and the listener, and say so.

## Owns and may change

- **Owns:** the files in this spec's header.
- **May change, how:**
  - `skein/crates/skein-ui-web/src/tests/page.rs`: only assertions on a page key's ops, if any reads them.
  - `temper/crates/temper-client-shell-web/src/lib.rs` names `Op::ScrollScreen` in a list of ops: no new variant comes, so it needs no change. If it does, a call site only, and say so.
- **Options:**
  - If a press's `mousedown` must also stand aside for a component or an open menu, stand aside where `in_component` is true, as the listener does for its other events.
  - If `reproduced.md` names a further cause in these files (a measure the key path still misses, a wheel path that differs), fix it here, with its test.

## Steps

### The key

1. `skein/crates/skein-ui-web/src/page.rs`, `key`, where it pushes the scroll:

   ```rust
   ops.push(Op::ScrollScreen { node, down: key.as_ref() == b"PageDown" });
   ops.push(Op::Measure { node });
   ```

   `Op::Measure` already runs at the end of the web half's apply, with the position the scroll has just set, and goes through `scrolled`, the rules a person's scroll goes through. The browser's own scroll event still comes a frame later with the same numbers: check that it changes nothing and reports no second `Near`. `report_near` should hold that; a test below does.

### The press

2. `page.rs`, beside `press`:

   ```rust
   /// A pointer's press on this node leaves the focus where it was (ui-web.md, 7).
   #[must_use]
   pub fn press_keeps_focus(&self, node: NodeId) -> bool
   ```

3. `page/web.rs`: add `("mousedown", false)` to the listeners, and handle it early in the closure, before `read`:

   ```rust
   if event.type_() == "mousedown" {
       if target.nearest(&element).is_some_and(|node| model.borrow().press_keeps_focus(node)) {
           event.prevent_default();
       }
       return;
   }
   ```

   Preventing `mousedown`'s default keeps the focus and leaves the `click`, and so the press, as they are. `web.rs` compiles only for wasm32: check it with `heavy cargo check --target wasm32-unknown-unknown -p skein-ui-web`.

### Tests

4. In `skein/crates/skein-ui-web/src/tests/page/events.rs`, beside `pgup_and_pgdn_scroll_the_first_log_from_a_field`, using `reading_page`, `scroll`, `page_key`, `log_page`, `scroll_ends` and `LOG_END` as they stand. The pure half can't run the measure, so each test calls `scroll` where the web half's measure would report:
   - `a_page_key_measures_its_log_in_the_same_apply`: for each of PageUp and PageDown, the ops are exactly `[Op::ScrollScreen { node: LOG, down }, Op::Measure { node: LOG }]`. This is the one that goes red first.
   - `a_page_up_while_following_stops_following`: at the foot, following; PageUp; then the measure's numbers, a screen up: the log has `data-log-away`.
   - `a_delivery_after_a_page_up_leaves_the_position`: from there, an apply with a patch: `Op::HoldView` first, `Op::PutView` before the measure, and `scroll_ends` empty.
   - `a_page_down_to_the_foot_follows_again`: from there, PageDown; then the measure's numbers at the foot: no `data-log-away`, and an apply with a patch has `Op::ScrollEnd` and no `Op::HoldView`.
   - `the_browsers_scroll_event_after_a_page_key_changes_nothing`: after a PageUp's measure on a bound log, the same numbers again: no ops, and no `Near`.
   - `a_press_to_log_end_keeps_the_focus`: `press_keeps_focus(LOG_END)` is true, and false for a log and for a plain button.
5. Update `pgup_and_pgdn_scroll_the_first_log_from_a_field`'s two assertions on the ops to the pair. It keeps everything else it checks.
6. Each new test must bite (the implementer's text). Remove the `Op::Measure` push once, invert `press_keeps_focus` once, and say what you saw.

### Close

7. Remove the **Not yet** bullet from `skein/docs/design/ui-web.status.md`, whichever way the focus rule went: the design's lines are then true of the code.
8. Once: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web -p skein-ui-web-tests --all-targets -- -D warnings`; `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui-web -- -D warnings`; `heavy cargo nextest run -p skein-ui-web -p skein-ui-web-tests`; `heavy cargo check -p skein-browser-world --tests`; `heavy cargo check --target wasm32-unknown-unknown -p temper-client-shell-web`; `python3 -I -B tools/citations/citations.py --base origin/main`.

   `skein/tests/browser/tests/suite/log_reading.rs` holds the stories that read this, in a tier: they are compiled here, not run.
9. Commit and push.

## Report

What `reproduced.md` said for these steps, and what you changed for it. The red you saw first. The rule you broke for each new test and the failure seen. The tests, how many, and your minutes.
