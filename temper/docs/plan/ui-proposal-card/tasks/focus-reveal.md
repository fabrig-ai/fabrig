+++
id = "focus-reveal"
part = "composer-reveal"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page.rs",
  "skein/crates/skein-ui-web/src/tests/page/events.rs",
  "skein/crates/skein-ui-web/Cargo.toml",
  "skein/docs/design/ui-web.status.md",
]
effort = "medium"
minutes = 25
+++

# A focus request brings its node into view

## Goal

A browser's `focus()` moves nothing on an element that already holds the focus, so asking again shows nothing. temper's
"Change something" needs the composer brought into view on a phone. This is a generic mechanism, so it goes into skein first.

Acceptance:

- A focus request (`Page::apply`: a node inserted with one, or whose count moved) pushes `Op::Focus` and then `Op::Reveal` for that node.
- A focus moved by a removal or by keys pushes `Op::Focus` alone, and an unchanged count pushes nothing.
- `page/web.rs` draws `Op::Reveal` as `scrollIntoView` with `block` and `inline` `nearest`.
- skein-ui-web's focused tests hold this, and its wasm32 clippy is clean.

## Inputs

- `temper/docs/plan/ui-proposal-card/notes/common.md`.
- `temper/docs/plan/ui-proposal-card/notes/contracts.md`: *A focus request comes into view*. It binds.
- `skein/docs/design/ui-web.md`, 7, *A focus request*.

## Owns and may change

**Owns:** the `files` above. **Departure from your text:** you remove this task's own **Not yet** entry from
`skein/docs/design/ui-web.status.md` (7, *A focus request*), since the code then matches the design. No other design file is yours.

**Readers, each edited only as said:**

- `temper/crates/temper-client-shell-web/src/lib.rs`, near line 347: its exhaustive match on `Op` gains the arm
  `skein_ui_web::page::Op::Reveal { .. }`, with the other arms that answer `false`. This line goes on this branch, as a compiler-forced
  arm of another tree.
- `skein-ui-web/Cargo.toml`: `web-sys` gains the features `ScrollIntoViewOptions` and `ScrollLogicalPosition`.
- `tests/page.rs` and `tests/page/events.rs`: exhaustive `Op` matches gain the arm, each with `Op::Focus`'s or `Op::ScrollEnd`'s
  neighbours; `a_focus_request_focuses_its_node` and `a_moved_focus_count_focuses_again` read `ops.last()` and move with the new shape.

**Pre-approved options:**

- **If `chat-reading-c`'s `Op`s have landed** (`HoldView`, `PutView`, `Measure`, `ScrollScreen`): take them as they are and add yours
  beside `Focus`.
- **No browser story** for this: the plan's `card-proof` render shows it at 420, and the page's model tests hold which `Op`s go out.

**Not yours:** temper's use of it, in the task `change-something`.

## Steps

1. **The failing tests,** in `tests/page.rs` beside `a_focus_request_focuses_its_node`:
   `a_focus_request_brings_its_node_into_view` (`Op::Focus` then `Op::Reveal`, the same node);
   `a_moved_focus_count_brings_its_node_into_view_again` (the case a browser's own focus can't do);
   and, in the removal tests, the assert that a moved focus pushes no `Op::Reveal` (`a_removal_taking_the_focus_moves_it_to_an_ancestor`
   already compares the whole `ops`, so it holds).
2. **`page.rs`:** the variant, pushed only at the request's `ops.push(Op::Focus { node })` near the end of `apply`.
3. **`page/web.rs`:** the arm, beside `Op::ScrollEnd`, on `self.group(*node).element`.
4. **Checks, once:** `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web --all-targets -- -D
   warnings`; `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui-web -p temper-client-shell-web -- -D warnings`;
   `heavy cargo nextest run -p skein-ui-web`; `heavy cargo nextest run -p temper-client-shell-web`. This changes what a focus request does
   in every page, so it also runs `heavy cargo nextest run -p skein-ui-example-shell-web` and compiles the browser's world, `heavy cargo
   check -p skein-browser-world --tests`, never running it.
5. Commit in small steps that say why, and push.

## Report

As your text says, and: the rule each new test bit on and the failure seen; whether a reader outside the list needed the arm; anything under
`## Left`.
