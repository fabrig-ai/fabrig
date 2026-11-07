+++
id = "temper-domain"
part = "temper-approval"
kind = "feature"
needs = ["client"]
files = ["temper/crates/temper-client-domain/**", "temper/crates/temper-client-shell-web/src/host.rs", "temper/crates/temper-client-shell-web/src/public.rs", "temper/crates/temper-client-shell-web/src/host_tests.rs", "temper/crates/temper-client-shell-web/src/host_tests_approve.rs", "temper/crates/temper-client-view/src/root.rs", "temper/tests/client/src/app.rs"]
effort = "high"
minutes = 210
+++

## Goal

temper's client root carries a terminal's approval, both halves: the browser's approval page as a place of its own, `Place::Approve { code }` at `/terminal`, its state, acts and reads in a module `approve` beside `authorize`; and a native client's ask, `Event::AskApproval`, in a module `asked`, which signs in at once with the answer's secret and keeps only the code and the expiry. The web shell keeps the page's address and replaces one that prints otherwise. As `temper/docs/design/client/README.md`, 3.5, *The approval page*, and 7.1, *The root's half*, and `temper/docs/design/client/browser.md`, 2.3 and 2.4 say.

Acceptance:

- `address::parse` reads `/terminal` and its `code` query as `Place::Approve { code }`, and `address::print_approve(code: Option<ApprovalCode>) -> Box<[u8]>` prints it shown, each as `browser.md`, 2.3's bullet *The approval page's code is its query* says, pinned by a table test.
- `approve`'s page goes through every state of README 3.5's table by the root's public events alone (`Event::Went`, `Event::Act { action: Action::Approve(..) }`, `Event::Client`, `fire`), pinned by table tests: opening reads once and approves nothing; a decision is sent once; while approved the page reads again each `approve::AGAIN` and stops at any other answer; signed out, not a browser, busy and Try again as the table says; an answer for another code, or after the page is left, changes nothing.
- `asked`: `Event::AskApproval` while signed out sends one ask; `Asked` signs in at once with `With::Approval { secret }` (linked) and holds `asked::State::Asked { code, expires }`; unlinked, it holds them and sends no sign-in; `NotAsked` is held as it came; an ask while signed in or in flight changes nothing; once the link leaves signed out, `Domain::asked()` is none. No `Debug` of the root holds the secret's bytes.
- The web shell's host keeps `/terminal?code=WDJB-MJHT` as the page's address, replaces `/terminal?code=wdjb-mjht` with it and `/terminal?code=AEIO` with `/terminal`, and the page sends one read, pinned by a host test.
- Each refusal's test was committed red before its code (Steps, 1).

## Inputs

- `temper/docs/design/client/README.md`, 3.5, *The approval page* (whole: its states table and every bullet); 5.1, *at the approval place* and *signed out at the approval*; 7.1, *The root's half*.
- `temper/docs/design/client/browser.md`, 2.3, the table's `Approve { code }` row and the bullet *The approval page's code is its query*; 2.4, *A terminal's approval*.
- `jig/docs/design/client/domain/client.md`, 4.4 (unlinked) and section 12 (the approval's bullets).
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 2.2, 6 and 7 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- The precedent, read whole: `temper/crates/temper-client-domain/src/authorize.rs` (its `went`, `progress`, `answered`, its `Flight`, the revision, and its tests), with its call sites in `domain.rs` and `route.rs`, and the shell's handling of `Place::Authorize` in `temper/crates/temper-client-shell-web/src/host.rs` (`start`, `opened`) and `host_tests_authorize.rs`.

## Owns and may change

- Owned: `temper/crates/temper-client-domain/**`; in `temper/crates/temper-client-shell-web/src/`: `host.rs` (`opened`'s arm), `public.rs` (`opened`'s arm), `host_tests.rs` (one `#[path = "host_tests_approve.rs"] mod approve;` line beside the authorize one) and a new `host_tests_approve.rs`.
- Readers, compiler-forced arms only, checked on the trunk:
  - `temper/crates/temper-client-view/src/root.rs`, `href`: `Action::Approve(_)` joins the arm that answers `None`.
  - `temper/tests/client/src/app.rs`: in `window_changed`, `domain::Event::AskApproval` and `domain::Event::Act { action: Action::Approve(_) }` join the arm that answers `false`; in `received`, `domain::Event::AskApproval` joins the arm that answers `vec![]`.
  - `temper/crates/temper-client-shell-web/src/public.rs`, `opened`: `Place::Approve { .. }` joins the arm that opens `Public::Projects`.
  - No other match on `Place`, `Action` or the root's `Event` is exhaustive outside the domain (`rg 'Place::AuthorizeMalformed|Action::Copy \{ \.\. \}|Event::TabRefused' temper` lists them). The root's `Request` gains no variant, so none of its many readers changes.
- What the `client` task left: arms in `route.rs` (`route_client`'s `jig_client::Request::Approval`, `route_unlinked`'s `UnlinkedRequest::ApprovalAct` and `UnlinkedRequest::Approval`) that drop each; replace them as Steps, 3 says. If it left `ApprovalAct` passed on as `Request::Unlinked` already, keep that.
- Names from the jig tasks, read from their crates before use: `jig_client::{Approval, ApprovalState, Decided, NotAsked, ApprovalAct, ApprovalCode}`, `jig_client::Event::{AskApproval, ReadApproval, DecideApproval, ApprovalAnswer}`, `jig_client::With::Approval { secret }`, `jig_client::Message::{ApprovalAsk, SignInApproval, ..}`, and `jig_client_protocol_wire::{ApprovalCode, CODE_TYPED_BYTES}`. If one differs from contracts 6, use the crate's and say so.
- Options:
  - `approve::Page`'s fields are yours; its accessors are `code() -> Option<ApprovalCode>`, `words() -> &[u8]` and `state() -> approve::State`, the view's (`temper-view`) only reads.
  - Where `asked` drops its state once the link leaves signed out: in its `progress`, called from `step` and `fire` as `authorize::progress` is (default), or on `Told::Link` in `route::tell`; say which.
  - If the tests need a linked root signed in and live (the native half's take, a `Request::Client` exchange), and `authorize.rs`'s private `signed_in()` helper fits, move it to `src/tests/fixtures.rs` as `pub fn linked_live() -> Domain` and use it from both; or build what the test needs in its own module. Default: the unlinked root of `fixtures::on_adoption_page()` for the page, a fresh `crate::new` started signed out for the ask.

## Steps

1. **Tests first, red, each refusal before its code**, in the crate's own test modules beside their siblings (`src/tests/address.rs`; new `src/tests/approve.rs` and `src/tests/asked.rs` with their `mod` lines in `src/tests.rs`; the shell's `host_tests_approve.rs`). Write step 2's types and signatures first, each body doing nothing (a function returns at once, `parse` gives no `Approve`), with step 3's arms; then these tests; commit both, so that each test fails on behaviour, not on a missing name. The report says what each failed on.
   - `address.rs`, `the_approval_page_reads_its_code_and_prints_it_shown`: rows `(path, Place)`: `/terminal` none; `/terminal/` none; `/terminal?code=WDJB-MJHT`, `?code=wdjbmjht`, `?code=WDJB%20MJHT` (none: no percent-decoding), `?code=wdjb mjht`, `?code=AEIO` (none), `?code=` (none), `?x=1&code=WDJB-MJHT`, `?code=AEIO&code=WDJB-MJHT` (the second, the first that reads), `?code=WDJB-MJHT&code=BCDF-GHJK` (the first), a value past 32 bytes (none); `/terminal/x` and `/terminals` `Place::Missing`. Then `print_approve(Some(..))` is `/terminal?code=WDJB-MJHT` and `print_approve(None)` `/terminal`, and each printed path parses back to its place.
   - `approve.rs` (an unlinked live root, `fixtures::on_adoption_page()`, then `Event::Went { place: Place::Approve { code } }`; the page's exchanges leave as `Request::Unlinked { request: UnlinkedRequest::ApprovalAct(..) }`, and answers come back as `Event::Client { event: jig_client::Event::ApprovalAnswer { answer } }`):
     - `opening_the_page_reads_once_and_approves_nothing` (refusal): one `ApprovalAct::Read { code }`, no `Decide`; a `fire` and a second `step` send nothing more; with no code, nothing is sent and the state is the field's.
     - `each_answer_draws_its_state`, rows `(from, answer, expected, reads_again)`: from reading, each `ApprovalState` (waiting with its wall to the ask, approved to approved reading again, taken to signed in, refused, expired, unknown, not a browser and signed out to signed out, not ready, busy and lost to busy); from approved, not ready, busy and lost stay approved reading again; from deciding, each `Decided`.
     - `a_decision_is_sent_once` (refusal): at the ask, Approve sends one `Decide { code, approve: true }`; Approve and Refuse again while deciding send nothing; Refuse at a fresh ask sends one `approve: false`.
     - `while_approved_it_reads_again_each_second_until_another_answer`: after `approved`, `next_deadline` is now plus `approve::AGAIN`; a `fire` before it sends nothing, one at it sends one read; busy keeps it; taken ends it, with no deadline of the page's left.
     - `a_typed_code_reads_or_stays_with_its_alert` (refusal): rows `(typed, reads)`: `wdjb mjht` and `WDJB-MJHT` read (one `Read` of `WDJBMJHT`); `WDJB-MJH`, `AEIOUAEI`, 33 bytes of `B`, empty: the field kept, the alert's state, nothing sent.
     - `an_answer_for_another_code_or_a_page_left_changes_nothing` (refusal): a `Read` of another code; any answer after `Went` to an address; each leaves the state and sends nothing.
     - `signed_out_draws_sign_in_first`: the link signed out (`Up::SignedOut`) before the read, and a read answered signed out: the signed-out state, nothing sent.
     - `try_again_reads_and_type_a_code_returns_to_the_field`: busy, Try again, one read; unknown, Type a code, the empty field, no code.
   - `asked.rs`:
     - `an_ask_while_signed_in_or_in_flight_changes_nothing` (refusal): on a live root, `Event::AskApproval` sends nothing and `asked()` is none; on a signed-out root, a second ask while the first is in flight sends nothing more.
     - `an_answered_ask_signs_in_with_its_secret_and_keeps_the_code_and_expiry`: a linked root (`crate::new`, started with no sign-in, signed out): `AskApproval` gives one `Request::Client { request: Exchange { credential: None, message: Message::ApprovalAsk(_), .. } }`; its reply, an encoded `w::ApprovalAskAnswer::Asked` with a fake secret (`[7; 32]`), gives `asked()` `Asked { code, expires }` and one `Exchange` whose message is `SignInApproval`, its decoded secret the reply's; `format!("{root:?}")` and `{:#?}` hold neither the secret's bytes as a list (`7, 7, 7`) nor as text. Then the take's reply, an encoded `signed_in` with a fake 40-byte token: the link leaves signed out and `asked()` is none.
     - `not_asked_is_held_as_it_came`: rows over `NotAsked`, each from its wire answer (refused, not ready, busy) and a lost exchange.
     - `an_unlinked_root_holds_the_code_and_leaves_the_take_to_its_host`: an unlinked root signed out: `AskApproval` gives `Request::Unlinked { request: UnlinkedRequest::ApprovalAct(ApprovalAct::Ask) }`; `ApprovalAnswer { answer: Asked { .. } }` gives `Asked { code, expires }`, no request out, and no secret in the root's `Debug`.
   - `host_tests_approve.rs`, `the_approval_address_prints_its_code_and_reads_once`: on `super::tab::linked()`, `host.went(b"/terminal?code=wdjb-mjht")`: `host.show` is `/terminal?code=WDJB-MJHT` replacing; the requests hold one exchange to `/jig/v1/approval/read` whose `w::ApprovalRead` code is `WDJBMJHT`; `host.due()` adds none. `/terminal?code=AEIO`: `show` `/terminal`, no exchange. `/terminal?code=WDJB-MJHT`: no `show`.
2. **Types and the codec:**
   - `address.rs`: `Place::Approve { code: Option<jig_client::ApprovalCode> }`, doc `/// The approval page (client/README.md, 3.5, The approval page): a code, or none to type one.`; in `parse`, the arm `(b"terminal", b"", b"") => return Place::Approve { code: approve_code(fields) }` in the existing `match (name, number, tail)`, so one trailing slash is ignored as for every page; `fn approve_code(query: &[u8]) -> Option<ApprovalCode>`, the first `code=` field whose value `ApprovalCode::parse` reads, in the form of `forge_choice`; `pub fn print_approve(code: Option<ApprovalCode>) -> Box<[u8]>` (`/terminal`, or `/terminal?code=` and `shown()`), measured and written as `print_choice` is.
   - `boundary.rs`: `Action::Approve(crate::approve::Act)`; `Event::AskApproval` with the doc `/// The native client's ask for an approval (client/README.md, 7.1).`
   - `approve.rs`, step code, `//! The browser's approval page (client/README.md, 3.5, The approval page; client/browser.md, 2.3 and 2.4).`:
     ```rust
     /// How long an approved page waits before it reads again.
     pub const AGAIN: Duration = Duration::from_secs(1);
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum State {
         /// No code: the field; `unread` after a Check that didn't read.
         Typing { unread: bool },
         Reading,
         /// The engine's waiting, asked at that wall time.
         Asking { asked: Wall },
         Deciding { approve: bool, asked: Wall },
         /// Approved, read again each `AGAIN` until another answer.
         Approved,
         SignedIn,
         Refused,
         Expired,
         Used,
         Unknown,
         SignedOut,
         /// Busy, not ready or lost: Try again reads again.
         Busy,
     }
     #[derive(Clone, Debug, PartialEq, Eq)]
     pub enum Act { Words { words: Box<[u8]> }, Check, Approve, Refuse, Type, Again }
     #[derive(Debug)]
     pub struct Page { /* code, the field's words (at most CODE_TYPED_BYTES) and whether typed past it, state, the next read's time */ }
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub(crate) enum Flight { Idle, Pending }
     pub(crate) fn went(root: &mut Domain, place: &Place);
     pub(crate) fn act(root: &mut Domain, act: Act);
     pub(crate) fn progress(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
     pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval);
     pub(crate) fn next_deadline(root: &Domain) -> Option<Time>;
     ```
     `went` makes the page for `Place::Approve` (reading with a code, the field without) and drops it for any other place. `act` changes the state as README 3.5 says (each act only in its state, else nothing: Words and Check at the field, Approve and Refuse at the ask, Type at unknown, Again at busy, with no code back to the field). `progress`, as `authorize::progress`: while the flight is pending, nothing; the link signed out turns any state but the ended ones (signed in, refused, expired, used, unknown) to signed out; with the link live and `root.client.admission().attempts` below `env.limits.client.exchanges`, it sends, through `crate::route::client`, `ReadApproval { code }` when reading or when approved and its read is due (`now >= again`, then `again` none), and `DecideApproval { code, approve }` when deciding, setting the flight pending. `answered` sets the flight idle, then applies a `Read` or `Decided` only when the page is there, its code is the answer's and its state is reading, approved or deciding as the answer's kind fits; approved sets `again` to `env.now.saturating_add(AGAIN)`. Every change bumps `root.approval_revision`.
   - `asked.rs`, step code, `//! A native client's ask for an approval (client/README.md, 7.1).`:
     ```rust
     #[derive(Clone, Copy, Debug, PartialEq, Eq)]
     pub enum State { Asking, Asked { code: ApprovalCode, expires: Wall }, NotAsked(jig_client::NotAsked) }
     pub(crate) fn ask(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
     pub(crate) fn answered(root: &mut Domain, env: &Env<Limits>, answer: jig_client::Approval, out: &mut Queue<Request>);
     pub(crate) fn progress(root: &mut Domain);
     ```
     `ask`, only with the link signed out and no ask in flight: `Asking`, and `jig_client::Event::AskApproval` through `crate::route::client`. `answered`, only while asking: `Asked { secret, code, expires }` holds `Asked { code, expires }` and, linked (`!root.unlinked`), sends `jig_client::Event::SignIn { with: With::Approval { secret } }` through `crate::route::client`; unlinked, the secret is dropped there. `NotAsked(why)` holds it. `progress` drops the state once the link isn't signed out. Bump `root.approval_revision` on each change.
   - `domain.rs`: fields `approval: Option<approve::Page>`, `approving: approve::Flight`, `approval_revision: u64`, `asked: Option<asked::State>`, built empty; `pub const fn approval(&self) -> Option<&approve::Page>` and `pub const fn asked(&self) -> Option<asked::State>` with doc lines citing README 3.5 and 7.1; `step` and `fire` call `crate::approve::progress` and `crate::asked::progress` beside `crate::authorize::progress`; `next_deadline` takes `crate::approve::next_deadline(root)` in its list; `shown` adds `approval_revision`; `welcome` returns at once while `root.approval` is some, as for `authorization`.
   - `lib.rs`: `pub mod approve;` and `pub mod asked;` in order.
3. **Routing**, `route.rs`: `went` calls `crate::approve::went(root, &place)` beside `authorize::went`, and `Place::Approve { .. }` joins the arm that tells the pages and the forge's side they left; `authorize::went`'s match gains `Place::Approve { .. }` with the places that drop the authorization. `event`: `Event::AskApproval => crate::asked::ask(root, env, out)`; `Action::Approve(act) => crate::approve::act(root, act)`. `route_client`: `jig_client::Request::Approval(answer)` to `asked::answered` for `Asked` and `NotAsked`, to `approve::answered` for `Read` and `Decided` (a `fn approval(root, env, answer, out)` both routes call). `route_unlinked`: `UnlinkedRequest::Approval(answer)` the same; `UnlinkedRequest::ApprovalAct(_)` joins `Down` and `SignOut` in passing the request on as `Request::Unlinked { request }`.
4. **The web shell**, `host.rs`'s `opened`: `Place::Approve { code }` prints `address::print_approve(code)`, sets `self.show = Some((printed, true))` when it differs from the path, and returns `self.address.clone()`, as the authorization's arm keeps the address; `start` already sends `Went { place }` for a place that isn't an address. `public.rs` as above. The readers' arms (above).
5. Break each rule once by hand and see its test fail, then restore: send a read on `went` regardless of the link; send the decision on each press; let a `Read` of another code apply; keep the secret in `asked::State`.
6. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-domain -p temper-client-shell-web -p temper-client-view -p temper-client-world --all-targets -- -D warnings`; focused tests, one at a time: `heavy cargo nextest run -p temper-client-domain`, `-p temper-client-shell-web`, `-p temper-client-view`, `-p temper-client-world`; and the domain's wasm32 build once, `heavy cargo check -p temper-client-domain -p temper-client-shell-web --target wasm32-unknown-unknown`.
7. Commit and push.

## Report

As the implementer's text says, and: the choices among the options; each reader arm, one line each; each red test and what it failed on; each rule broken by hand and the failure seen; any name of the jig tasks that differed from contracts 6.
