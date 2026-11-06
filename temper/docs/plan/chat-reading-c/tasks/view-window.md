+++
id = "view-window"
part = "view-bounds"
kind = "feature"
needs = ["thread-window"]
files = [
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/limits.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-shell-web/tests/suite/memory.rs",
  "temper/crates/temper-client-shell-terminal/src/limits.rs",
]
effort = "high"
minutes = 60
+++

# The page's bounds are priced from the window's bytes

## Goal

A chat's window may now hold many more than 16 lines. `temper_client_view::Limits::of` prices the conversation as `LINES` lines, so a full window could panic the builder. Reprice it from `document_bytes` and `LINE_ROOM`, before anything lets a window fill.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance:

- `Limits::of` no longer reads `pages::chat::LINES`, and prices the window as the contract's *temper's view* says. It returns `Some` at both shells' limits, `valid` holds, and the terminal's `render_bound`, `Terminal::operations` and `Terminal::new` still hold: their tests stay green unchanged.
- At a small domain's limits, a thread filled to its `bytes` by the Markdown that makes the most nodes and text, and again by the most lines of the smallest weight, each a turn of as many blocks as its weight allows, builds without the builder's panic. This test bites: halve the priced window, and see the panic.
- `pages/chat.rs`'s `Run::with_capacity` no longer reads `LINES`.
- A page's figures at the shells' limits, against today's, are in your report: `nodes`, `text_bytes`, the view's heap for a full window. `temper-client-shell-web/tests/suite/memory.rs` holds a full window, in its idiom, if that's cheap (its ceiling the peak plus 25%, rounded up to two figures). If not, measure with throwaway code, give the figures, and add no feed to the host for it.
- If `text_bytes` doesn't fit a `u32`, stop and report the figures: that is a `Question:`, as it bounds what a page may hold.

## Inputs

- `temper/docs/design/client/README.md`, 5.4 and 5.15.
- `jig/docs/design/client/domain/client.md`, section 10, *A window bounded by size*.
- `skein/docs/design/ui.md`, section 15, on limits.
- `temper/docs/plan/chat-reading-c/notes/contracts.md`, *temper's view*, binding.
- `temper/docs/plan/chat-reading-c/notes/common.md`.

## Owns and may change

Owns the files in the header.

May also change, these edits alone:

- `temper/crates/temper-client-view/src/pages/chat.rs`: `Run::with_capacity`'s argument, from `rows * rows * thread.limits.lines`, saturating into the `List`'s `u32`: a list grows as it fills. `chat-page` owns the rest of this file.
- `temper/crates/temper-client-view/src/tests/**`: a test that pins the view's old figures: its expected value.
- `temper/tools/gallery/src/worlds.rs`: compiles as it is; change it only if a gallery test proves a figure there is needed.

Options, each decided:

- **`structure`'s worst case:** a turn's `rows * rows` blocks of a page stops being priced per line, since `LINE_ROOM` charges each block. If reading the walk shows a block can make more than the 10 nodes the contract prices, price what it makes and say so.
- **Live lines and words** sit outside the window's bytes: price them as `Thread` bounds them, `live` bytes in total. If that's more than the structure above can bear, take the smaller bound, as `prune_live` holds it, and say so.
- **Heap:** `temper/tests`' ceilings shouldn't move, as the engine holds nothing new. Any that moves is named, with its measurement before and after.

**A change of meaning runs its readers' tests:** run `temper-client-view`, `temper-client-shell-web`, `temper-client-shell-terminal`, `temper-client-gallery`.

Not this task: `thread_limits` (`chat-reads`); any drawing.

## Steps

1. Red first: `a_window_full_of_markdown_builds` and `a_window_full_of_small_lines_builds` in `temper-client-view/src/tests/limits.rs`, set to a thread at its `bytes`, panicking under `LINES`'s price.
2. The price.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-view -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-gallery`. Commit and push.

## Report

As the implementer's text asks, with the figures above. Whichever you measured, say how.

## Answers

2026-11-06. **Question:** the candidate price of `text_bytes` is 5,140,013,034, beyond `u32`: tighten the per-room text accounting, or widen the page's accounting?

**Tighten; `skein_ui::Limits.text_bytes` stays a `u32`.** The overflow is the candidate's, not a page's: it charges each room's 10 structure nodes at `text` (4,281, the longest phrase in the whole book), twice, 701,399,040 bytes each, where the contract's *temper's view* puts `max_text_bytes(document_bytes)` alone beside `window_nodes`.

- **A room's structure:** a line's own bytes are already in `max_text_bytes`. What a structure node adds is a short phrase and what a row cuts: `chat_line.rs` cuts a run's line at 253 bytes and a row at 160. Charge those nodes a named constant in `limits.rs`, 512 bytes a node, its comment saying why, in both `window_text` and `live_text`. It is a constant, not `receiving.text`: those cuts don't shrink with a small domain.
- **The figure:** from your own intermediates, 3,724,136,082 + 85,983,232 (window) + 94,371,840 (live) + 495,960 = 3,904,987,114, below today's 3,942,944,842. That sum is calculated, not built: your report gives the built figure.
- **Leave `text` and the outside-window price as they are.** They aren't this task's question, and they fit.
- **One more test, since the price is new:** `a_window_full_of_calls_builds`, in `tests/limits.rs`, a window of turns filled with calls that draw rows and a run's line, and its bite shown as the others': halve the 512, and see the panic. If it can't be made to panic, say so, with the text it measured.
- **If a line draws more than the price holds** (a phrase carrying a line's bytes beyond twice), price what it draws and say so. If the sum then leaves `u32`, stop as the acceptance says, with the figures.

The rest of the acceptance stands, the steps you did not reach included: both shells' `Some`, the halved-window mutation, the heap figures for a full window, clippy and the four packages' tests. Your `nodes` figure, 22,518,676, is mostly `lines * plain` in `live_nodes`: if it moves a heap ceiling, name it with its measurement, as *Heap* says, and take the smaller bound *Live lines and words* allows.
