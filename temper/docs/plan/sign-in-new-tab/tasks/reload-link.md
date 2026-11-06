+++
id = "reload-link"
part = "tab-root"
kind = "feature"
needs = ["sign-in-holds", "view-tab", "row-tab"]
files = ["temper/crates/temper-client-shell-web/src/records.rs", "temper/crates/temper-client-domain/src/pages/accounts/request.rs"]
effort = "high"
minutes = 45
+++

## Goal

A reload while a sign-in waits draws the same page: "Open it again ›" and the tab's refusal come back from the small store, read only for the page's own source.

Acceptance:

- `records::SignIn`, `encode_sign_in`, `decode_sign_in`, `SIGN_IN_KEY`, `Request::KeepSignIn` and `Event::SignInKept`, as `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The small store* has them.
- A root made at the source's address and given `SignInKept` for that source draws the note with "Open it again" at the kept address. For another source it draws none, and the way's card rules stand.
- A kept refusal draws "ChatGPT’s sign-in" and the primary "Open ChatGPT’s sign-in ›".
- A host test: the answer's duties hold one `Asked::Save` under `sign-in`, `writes` unchanged, and a `start` given that record gives `SignInKept` right after `Event::Start`. A malformed value is dropped, with no panic.
- The sign-in key is 7 bytes, so `Shell::finish`'s closing warning doesn't count it: a test or a comment holds that, whichever the code lets you pin.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The small store*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *A reload while it waits*; section 7, *A sign-in's address in the small store*.
- `temper/docs/design/client/browser.md`, 2.1, `records`.

## Owns and may change

- Owns the files in the header, and `temper-client-domain/src/boundary.rs` and `src/sign_in_tab.rs` for the request and event.
- May change `temper-client-shell-web/src/host.rs` (its `start` and `duties`), the `Request::sign_in_address` readers' tests, and one line in `temper-client-domain/src/pages.rs` to give `SignInKept` to the page's form.
- Compiler-forced arms for `Request` and `Event` are allowed, as the implementer's text has them. A shell with no small store of this kind drops `KeepSignIn`.
- Remove the sentence "A page reloaded holds no address, so it draws no "Open it again"." from `temper/docs/design/client/README.status.md`'s first **Not yet** entry. This task may make that one edit, against the implementer's text.

## Steps

1. The record's codec in `records.rs`, with a round trip and malformed values: no space, a source past `u32`, a first byte other than `0` or `1`.
2. The root asks `KeepSignIn` as the note has it, from `sign_in_tab.rs`'s answer.
3. `Host::duties` carries it as `Asked::Save`. `Host::start` reads the record from the records it's given, before they go to `records::decode`, and gives `Event::SignInKept` right after `Event::Start`.
4. The form: `Request` keeps `kept: Option<(u32, Box<[u8]>, bool)>`. `sign_in_address` falls back on it while `returned` is its source. `ConnectModel::tab_refused` reads the kept refusal the same way.
5. Tests as the acceptance has them, failing first.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-shell-web --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-web`; `heavy cargo check -p temper-client-world -p temper-client-gallery --tests`. Commit and push.

## Report

As the implementer's text asks.
