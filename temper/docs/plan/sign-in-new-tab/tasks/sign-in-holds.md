+++
id = "sign-in-holds"
part = "tab-root"
kind = "feature"
needs = ["tab-surface"]
files = ["temper/crates/temper-client-domain/src/sign_in_tab.rs", "temper/crates/temper-client-domain/src/departure.rs", "temper/crates/temper-client-domain/src/domain.rs", "temper/crates/temper-client-domain/src/lib.rs", "temper/crates/temper-client-domain/src/pages/connect_model.rs", "temper/crates/temper-client-domain/src/tests/pages/paste.rs", "temper/crates/temper-client-shell-web/src/host.rs", "temper/crates/temper-client-shell-web/src/host_tests_departure.rs", "temper/crates/temper-client-shell-web/src/host_tests_tab.rs", "temper/crates/temper-client-shell-terminal/src/client.rs", "temper/crates/temper-client-shell-terminal/Cargo.toml", "temper/crates/temper/Cargo.toml", "temper/crates/temper/tests/support/person.rs", "temper/crates/temper/tests/support/sign_in_welcome.rs", "temper/crates/temper/tests/support/mod.rs", "temper/crates/temper/tests/fake_backends/walk/sign_in.rs", "temper/tests/client/src/front/account_acts.rs", "temper/tests/client/tests/suite/connect_model.rs", "temper/tests/client/tests/suite/sign_in.rs", "temper/tests/client/tests/suite/sign_in_paste.rs", "temper/tests/client/tests/suite/welcome_m3.rs", "temper/tests/client/tests/suite/welcome_m4.rs"]
effort = "high"
minutes = 110
+++

## Goal

Press opens a sign-in's tab in its own turn, the answer points it or closes it, and the page holds: no `Return` goes out, no `Request::Leave`. The root's departure for a sign-in goes, `departing` and `returning` with it.

Acceptance, failing tests first, each in the lowest tier that shows it:

- In `temper-client-world`: the press asks `Tab::Open { provider }` before the engine has answered anything; the answer asks one `Tab::Point { to }` with the address; no `Return` reaches the fake engine, no `Leave` comes out, and a rebuild or `settle` asks nothing more; the page's address then names the source, its history entry replaced, on `ConnectModel` and on the welcome's Model step.
- In `temper-client-world`: a refused answer, and a lost one, ask `Tab::Close`, and the page says why as it does today. An answer for a page the person has left asks `Tab::Close`.
- In `temper-client-world`: `Event::TabRefused` then the answer asks no `Point`, and the page holds the address.
- In `temper-client-shell-web`, `src/host_tests_tab.rs`: the press's `Duties` hold `TabAct::Open { words: "Opening ChatGPT’s sign-in…" }` with no `navigate`; the answer's hold `TabAct::Point { to }`, a `show` that replaces, and no `navigate`, and no exchange with a `Return`; `Report::TabRefused` then the answer holds no `Point`.
- `temper-client-domain` has no `departure` module, no `departing`, `returning`, `new_departing` or `unlinked_departing`. `temper-client-shell-terminal` has no `departing` or `browser-root`.

## Inputs

- `temper/docs/plan/sign-in-new-tab/notes/contracts.md`, *The root*, *The host*.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.
- `temper/docs/design/client/README.md`, 3.5, `ConnectModel`, *Signing in*, *The sign-in's tab*; section 10, its sentence on the press.
- `temper/docs/design/client/browser.md`, 2.1, `host`; 2.3.
- `temper/docs/design/draft/sign-in-new-tab.md`, 2, 3, 5.

## Owns and may change

- Owns the files in the header.
- One line in `temper-client-domain/src/route.rs`: `departure::told` at the head of `tell` becomes `sign_in_tab::told`. Keep this hunk to that, with the pages' `Output` routing the press needs: fabrig/ui has priority in `pages.rs` and `route.rs`.
- `Act::SignInElsewhere` and `ConnectModel::elsewhere` stay for now, doing nothing but being set: the task `elsewhere-goes` removes them and their readers. Don't touch `temper-client-view`.
- `temper/tests/client/src/front.rs` keeps its fake `Return` route: the task `return-goes` removes it. The wire's `Return` stays too.
- `reopen(.., departing)` loses its last argument at all its callers in `temper/tests/client/tests/suite/`: a call-site edit in any file there.
- `temper/crates/temper-client-shell-web/tests/suite/host.rs` and `tests/suite/host/walk.rs` are the task `journey-goes-on`'s. Their walk ends on a departure, so tests there fail once this lands: report them under `## Left`, and don't edit them. The draft `real-journeys` is changing them too.
- `temper_client_domain::Request::Leave` stays for a forge's and an authorization's departure, and `temper-client-shell-web/src/lib.rs`'s comment on it may lose the word "sign-in" if it has it.
- If `sign_in_welcome` is confirmed dead by the compiler, once `Person::departing` goes, remove the module. If it has a caller, take the caller's departure out and say so in the report.
- If a page's address move to the same way with a source resets its sign-in (`ConnectModel::changing_way`, `pages.rs`'s address move), change it so that it doesn't, in the fewest lines, and say so. It must stay true that a source the watch doesn't know, on a reload, is dropped.
- Remove from `temper/docs/design/STATUS.md` the words ", whose words say a new tab while a press still leaves in its own,
  until that draft is built", which this task makes false. This task may make that one edit, against the implementer's text.
- Remove from `temper/docs/design/client/README.status.md`'s first **Not yet** entry the sentence beginning "the card's line and the note say a new tab". This task may make that one edit, against the implementer's text, and may reword what's left of that sentence's list so that it stands.

## Steps

The sketch, binding in the names:

```rust
// temper-client-domain/src/sign_in_tab.rs
//! A sign-in's tab: the press opens it, the answer points or closes it, never the state
//! (client/README.md, 3.5, `ConnectModel`, The sign-in's tab).
pub(crate) fn told(root: &mut Domain, told: &jig_client::Told, out: &mut Queue<Request>);
pub(crate) fn refused(root: &mut Domain);
```

- `Act::SignIn`, past the guards at the head of `ConnectModel::act`, sends its `Output` for the tab right before `self.request.send(..)`, and the route sets `Domain::tab` to the page's pending key.
- `told`: for a `Told::Answered` of that key, clear it. If the page still holds that key and the answer is `AccountSignInStarted { source, address }`, push `Tab::Point`, then, on `ConnectModel` or the welcome's Model step, `Request::Show` of that way with `source: Some(source)`, `replace: true`. Any other answer, or a page left, pushes `Tab::Close`. It never consumes the answer: the pages read it as before. A press from the welcome's Ready step, which has no way in its address, asks `Point` and moves no address.
- `refused`: clears `Domain::tab`, sets the page's refusal.
- `Host::new` makes the root with `root::new`.
- `temper-client-world`'s `front`: `front.tabs: Vec<Tab>`, recorded where `leaves` is, and a helper that presses without settling, so a test can read the tabs before the answer.
- Rewrite, in the client world, by the page's holding: `a_departing_page_returns_then_leaves_once_and_a_rebuild_asks_nothing`, `a_non_departing_page_asks_no_return_and_never_leaves`, `a_departing_welcome_model_step_returns_to_ready`, `a_welcome_sign_in_come_back_at_ready_reads_it_under_way_then_its_account` (at a typed `/welcome?at=ready`), `native_browser_and_terminal_keep_their_departure_policies`, and any of `sign_in_paste.rs` that presumes a departure. Remove `lost_return_does_not_depart`, `browser_waits_for_the_return_answer_before_leaving` and `a_refused_return_keeps_the_page_and_its_sign_in_link`, whose stories this task's new tests replace.
- `temper-client-domain/src/tests/pages/paste.rs`: rewrite `signing_in_elsewhere_sends_the_same_sign_in_and_departs_nowhere`; remove `signing_in_natively_after_elsewhere_departs_again`.
- `temper-client-shell-web/src/host_tests_departure.rs` goes. Its `linked()` helper, which `host_tests_authorize.rs` reads, moves to `host_tests_tab.rs`.
- `temper/crates/temper/tests/fake_backends/walk/sign_in.rs`: `a_chatgpt_browser_return_on_the_fake_box_leaves_ready_able_to_create` becomes `a_chatgpt_browser_tab_on_the_fake_box_leaves_ready_able_to_create`, on `NativeClient`'s plain root. Its play follows `Notice::Leave`, which the terminal's `route.rs` makes of `Tab::Point`, and reads the fixed page in place of the 303: `Sign-in received. You can close this tab.` if the task `listener-words` has merged, which is in your base only if your branch's `PAGE` says so; read it from there. `person.rs` loses `Departure`, `come_back` and `welcome_sign_in` where they're only the departure's.

Break a rule by hand once for each new rule: let `told` ask `Point` from `Told` for any key, see the once-only test fail; let the press ask no `Open`, see the press's test fail. Restore them.

This change of meaning runs its readers' tests: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-shell-web -p temper-client-shell-terminal -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-shell-terminal -p temper-client-world`; `heavy cargo nextest run -p temper-client-shell-web --lib`; `heavy cargo nextest run --profile process-temper -p temper --test process -E 'test(a_chatgpt)'`, reading the count; `heavy cargo check -p temper -p temper-client-gallery -p temper-client-shell-cli --tests`. Commit and push.

## Report

As the implementer's text asks. List what's left red under `## Left`, with the task that owns it.
