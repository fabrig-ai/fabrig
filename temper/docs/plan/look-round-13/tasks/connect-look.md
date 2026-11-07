+++
id = "connect-look"
part = "look"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-shell-web/temper.css", "temper/crates/temper-client-view/src/look.rs"]
effort = "medium"
minutes = 25
+++

## Goal

Land the request's stylesheet: every rule of its `connect.css` goes into `temper.css` under `/* ==== ui: settings ==== */` (its `@media` block's rules under `/* ==== ui: phone ==== */` or beside the marker's own media block, as that file does), selectors and declarations as they are, and `look.rs` gains the tokens that name them. Later tasks draw with these; this task draws nothing.

Acceptance: every selector of `connect.css` is in `temper.css` once, unchanged but where a variable it names does not exist (fix to the file's own, and report it); `NAMES` holds each token; `temper-client-view`'s and `temper-client-shell-web`'s focused tests pass.

## Inputs

- `temper/docs/plan/look-round-13/notes/common.md`, and `notes/contracts.md`, *Look tokens*.
- `temper/docs/design/draft/look-requests.md`, 3, step 4. `temper/docs/design/client/browser.md`, 2.5, for how a token names a class.
- The request's `connect.css`.

## Owns and may change

- Owns: `temper/crates/temper-client-shell-web/temper.css` (new rules alone), `temper/crates/temper-client-view/src/look.rs`.
- May change: a test in `temper-client-view` or `temper-client-shell-web` that counts tokens or classes, its count alone.
- If a token is unused and a lint or test objects: keep it, with a scoped `#[expect(..., reason = "drawn by look-round-13's later tasks")]`, and say so.
- The rules the request replaces are removed by the tasks that stop drawing them, not here.

## Steps

1. Take new token numbers after the file's last, in its style, each with a doc line citing `client/browser.md`, 2.5.
2. Copy the rules. Check `--gf-*` and `--temper-mono` exist in `temper.css`.
3. Narrow tests, `cargo fmt`, clippy, commit with the trailer, push.

## Report

As the implementer's text has it.
