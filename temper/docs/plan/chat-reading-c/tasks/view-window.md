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
