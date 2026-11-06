+++
id = "window-seam"
part = "window"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-client-domain/src/limits.rs",
  "temper/crates/temper-client-domain/src/pages/chat.rs",
  "temper/crates/temper-client-domain/src/tests/pages/chat.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-shell-web/src/host_tests.rs",
  "temper/crates/temper-client-shell-web/Cargo.toml",
]
effort = "medium"
minutes = 25
+++

# A conversation's window may be narrower than `document_bytes`

## Goal

The surface the tier's smaller window needs, off by default: a named window in temper's client limits, read by `thread_limits`, and a feature of the web shell that names one. No bundle turns the feature on yet, so nothing a person or a story sees changes; `window-story` turns it on with the stories that fit it.

Acceptance:

- `temper_client_domain::Limits` has `window_bytes: Option<u64>`, and `thread_limits` reads it, both as `notes/contracts.md` has them. With `None`, `thread_limits` answers what it answers today, in `bytes` and `lines`.
- `temper_client_domain::valid` refuses a named window below `long_text` or above `document_bytes`, and admits both edges.
- `temper-client-shell-web` has the feature `small-window`. With it, `limits::client().window_bytes` is `Some(131_072)`, and that passes `limits::valid`. Without it, `None`.
- `heavy cargo check -p temper-client-shell-web --target wasm32-unknown-unknown --features small-window` builds.
- The `**Not yet:** 5.15: `Limits::window_bytes`` entry leaves `temper/docs/design/client/README.status.md`.

## Inputs

- `temper/docs/design/client/README.md`, 5.15, *The window*.
- `temper/docs/design/client/browser.md`, 4.3, *The tier's window*.
- `temper/docs/plan/browser-budget/notes/common.md`, `temper/docs/plan/browser-budget/notes/contracts.md`.

## Owns and may change

Owns the files in the header.

Readers, each edited only for the new field, `window_bytes: None`, in the literal: `temper/tests/client/src/fixtures.rs`, `temper/crates/temper-client-domain/tests/suite/task_pages.rs`, `temper/crates/temper-client-domain/tests/suite/change.rs`, `temper/crates/temper-client-domain/src/tests/fixtures.rs`, `temper/crates/temper-client-view/tests/suite/task_pages.rs`, `temper/crates/temper-client-view/tests/suite/change_page.rs`, `temper/crates/temper-client-view/src/tests/fixtures.rs`, `temper/crates/temper-client-tools/src/limits.rs`, `temper/crates/temper-client-shell-terminal/src/limits.rs`. Any other literal the compiler names takes the same edit. The tests that set `document_bytes` and read `thread_limits` (`temper-client-view`'s `tests/limits.rs` and `tests/chat.rs`, `temper-client-domain`'s `tests/pages/chat.rs`) keep working unchanged through `None`, and stay as they are.

`temper/docs/design/client/README.status.md` loses the one entry above.

Not this task: `tools/tiers/run`, `tools/tiers/test_run.py`, `temper/scripts/bundle.sh`, any story.

Options, each decided:

- **The view's pricing** stays on `document_bytes` (`temper-client-view/src/limits.rs`): a narrower window fits inside it. If a check in the view would refuse a narrower window, change that check only, and say so in your report.
- **Where the feature's value lives:** a `const` beside `CONVERSATION_TEXT`, chosen by `cfg`.

## Steps

1. Failing tests first, in `temper-client-domain`:
   - `a_named_window_bounds_the_thread_and_its_lines`, beside the test that reads `thread_limits(&limits).lines` in `src/tests/pages/chat.rs`: `Some(131_072)` gives those `bytes` and `131_072 / LINE_ROOM` lines, and `None` gives `document_bytes` and its lines.
   - `a_window_is_refused_outside_long_text_and_document_bytes`: each edge admitted, one byte past each refused.
2. The field, `thread_limits`, `valid`, and the readers' literals.
3. The feature and its `const`. In `host_tests.rs`, beside `validity_accepts_stock_limits_and_refuses_invalid_edges`, `the_ordinary_bundle_keeps_the_whole_document_bound`: without the feature, `limits::client().window_bytes` is `None`.
4. `cargo fmt`, the wasm check above, `heavy cargo clippy -p temper-client-domain -p temper-client-shell-web -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal --all-targets -- -D warnings`, and those packages' focused tests with `temper-client-world` (`temper/tests/client`).
5. Commit and push.

## Report

The rules you broke to see the tests bite, each file the field reached beyond those listed, and your implementation and check minutes apart.
