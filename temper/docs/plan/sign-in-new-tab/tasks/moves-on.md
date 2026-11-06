+++
id = "moves-on"
part = "tab-root"
kind = "feature"
needs = ["sign-in-holds"]
files = ["temper/crates/temper-client-domain/src/route.rs", "temper/crates/temper-client-domain/src/domain.rs", "temper/tests/client/tests/suite/connect_model.rs"]
effort = "high"
minutes = 35
+++

## Goal

The welcome's Model step, holding a sign-in, moves on to Ready by itself on the `accounts` watch, its address replacing the step's. `ConnectModel` already draws its Ready in place, and a held sign-in's ending draws its words on both.

Acceptance, failing tests first, in `temper-client-world`:

- A press on the welcome's Model step, its answer, then the watch's made account: `Request::Show` of `Welcome { step: Ready }`, `replace: true`, once, and no `Leave`.
- The same from a root made at the source's address (a reload while it waits).
- A declined or failed ending on the welcome's Model step stays on the step, drawing the ending's words and acts, as it does today.
- A person who types `/welcome?at=model&...&way=sign-in` with a model already usable stays on *A model usable*, with its "Continue": only a held sign-in moves on.
- `ConnectModel`'s held sign-in draws Ready in place on the watch, with no address but its own. If a test holds this already, name it in the report and don't add one.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *Its end*; the welcome's Model step, *A model usable*.
- `temper/docs/design/draft/sign-in-new-tab.md`, 3.

## Owns and may change

- Owns the tests it adds in the files in the header. In `route.rs`, only `welcome_program_progress` and its neighbour: fabrig/ui has priority there, so keep the hunk small.
- May rename `welcome_program_progress` to `welcome_model_progress`, with its two callers in `domain.rs`, or add a second function beside it, whichever is the smaller diff.
- May read and call `pages::connect_model` and `pages::accounts` as they stand; no new public item there, unless a held sign-in can't be told otherwise, in which case add one `pub(crate)` reader and say so.

## Steps

`welcome_program_progress` in `route.rs` is the pattern: it reads the address, `model_usable`, and the page's state, and pushes the `Show`. Here the page holds a sign-in when `page.request.sign_in_state(client)` is `Signing` or `Ended`, on a sign-in way of the address. Usable and held moves on.

Finish: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world`; `heavy cargo check -p temper-client-shell-web --tests`. Commit and push.

## Report

As the implementer's text asks, with the rule you broke by hand and what failed.
