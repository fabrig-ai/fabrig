+++
id = "elsewhere-goes"
part = "tab-root"
kind = "migration"
needs = ["sign-in-holds"]
files = ["temper/crates/temper-client-domain/src/pages/connect_model.rs", "temper/crates/temper-client-domain/src/tests/pages/paste.rs", "temper/crates/temper-client-domain/src/tests/pages/welcome.rs", "temper/crates/temper-client-view/src/tests/connect_model.rs", "temper/crates/temper-client-view/src/tests/welcome.rs", "temper/tests/client/tests/suite/sign_in_paste.rs", "temper/tools/gallery/src/samples.rs"]
effort = "medium"
minutes = 25
+++

## Goal

Every sign-in now holds its page with the paste offered, so the separate way in goes: `Act::SignInElsewhere` and `ConnectModel::elsewhere`.

Acceptance:

- Neither exists. Every reader presses `Act::SignIn`, or goes, as below.
- `temper-client-gallery`'s `welcome-paste-*` states still draw, and its `elsewhere=1` mock query at `samples.rs:975` goes if nothing else reads it.
- The negative asserts of the words "This browser isn't on temper's computer" and "Go forward to that page" stay in the tests that hold them (`temper-client-view/src/tests/connect_model.rs`, `src/tests/welcome.rs`, `temper/tests/client/tests/suite/sign_in_paste.rs`): the entry stays gone. `PasteHistory` is in no code, and there's nothing to remove for it.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *From another computer*.
- `temper/docs/design/draft/sign-in-new-tab.md`, 5.

## Owns and may change

- Owns the files in the header, each edited only for this:
  - `temper-client-domain/src/pages/connect_model.rs`: lines 31, 38, 136, 164, 204 to 213, 242 and 248 as they stood, and the field's default at 468.
  - `temper-client-domain/src/tests/pages/welcome.rs`, its one `A::SignInElsewhere`: `A::SignIn`.
  - `temper-client-view/src/tests/connect_model.rs`: its `M::SignInElsewhere` at `pasting` and near 888 become `M::SignIn`; `signing_in_elsewhere_uses_the_same_waiting_page` goes, with the `elsewhere()` assert.
  - `temper-client-view/src/tests/welcome.rs`, near 1955, its `A::SignInElsewhere`.
  - `temper/tests/client/tests/suite/sign_in_paste.rs`: `elsewhere_keeps_connect_in_the_page_and_the_welcome_offers_none` keeps only what isn't the press's kind; `both_sign_in_acts_offer_the_paste_folded` becomes one act.
  - `temper/tools/gallery/src/samples.rs`, near 1702.
- A test whose two rows become the same press collapses to one row.
- If a test name says "elsewhere" and what it holds is still true of every sign-in, rename it. Otherwise it goes.
- Remove from `temper/docs/design/client/README.status.md`'s first **Not yet** entry its sentence on `Act::SignInElsewhere`. This task may make that one edit, against the implementer's text.

## Steps

`rg -n "SignInElsewhere|\.elsewhere\b|elsewhere\(\)" temper`: `Choice::Elsewhere` and "A Forgejo elsewhere" are a forge's, and stay.

Finish: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-view -p temper-client-world -p temper-client-gallery --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world -p temper-client-gallery`. Commit and push.

## Report

As the implementer's text asks. If the gallery's print changes, that's a red: name it.
