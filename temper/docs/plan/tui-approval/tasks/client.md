+++
id = "client"
part = "jig-approval"
kind = "feature"
needs = ["surface"]
files = ["jig/crates/jig-client-protocol-link/**", "jig/crates/jig-client-protocol-http/**", "jig/crates/jig-client/**"]
effort = "high"
minutes = 180
+++

## Goal

jig's client carries the approval, both halves: a signed-out client asks one and takes it with its secret, sending the take again by itself while it waits; a browser's client reads one by its code and decides it. Each answer is told once, and the secret is held only while the take runs, never printed or kept. As `jig/docs/design/client/domain/client.md`, 4.1, 4.2, 4.4 and 12, and `temper/docs/plan/tui-approval/notes/contracts.md`, section 6, say.

Acceptance:

- `jig-client-protocol-link` has contracts 6.1's types, `Message`'s four variants (`ApprovalAsk`, `ApprovalRead`, `ApprovalDecide`, `SignInApproval`, each carrying its encoded record of 2.1), and their encoding and decoding; `jig-client` re-exports them, and `ApprovalCode` and `ApprovalAct` from where they live.
- `jig-client-protocol-http` maps the four messages to contracts 5's paths.
- `jig-client`: `Event::AskApproval`, `ReadApproval { code }`, `DecideApproval { code, approve }`, `Request::Approval(Approval)`, `With::Approval { secret }`, `NotSignedIn::Approval(Taking)`, the take sent again after `APPROVAL_AGAIN` (one second) while it is answered waiting, and the unlinked mode's `UnlinkedRequest::ApprovalAct`, `Event::ApprovalAnswer` and `UnlinkedRequest::Approval`, each as contracts 6.2 says, each pinned by a test.
- No `Debug` of the link, the client, their requests or their events shows the secret's or the credential's bytes (a canary test).
- Every compiler-forced arm in another crate maps the new variant to its nearest existing behaviour, each named in the report; the packages that hold them pass their focused tests.

## Inputs

- `jig/docs/design/client/domain/client.md`, 4.1, 4.2, 4.4 and section 12 (the approval's bullets: *`Approval { secret }`* and *An approval, asked and decided*).
- `jig/docs/design/protocol/client.md`, section 8, *A sign-in approved by a browser*; 4.3; 17.1.
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 2, 5 and 6 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- The precedents: the printed link's sign-in (`jig-client-protocol-link/src/sign_in.rs`, `jig-client/src/client.rs`'s `SendAgain`, `signing`, `not_signed_in`) and the authorization (`jig-client-protocol-link/src/authorize.rs`, `Authorized`; `jig-client`'s `Event::Authorize`, `Request::Authorized`, `UnlinkedRequest::Authorized`).

## Owns and may change

- Owned: `jig/crates/jig-client-protocol-link/**` (its boundary, a new `approval.rs` beside `authorize.rs`, `sign_in.rs`, `component.rs`'s exchange kinds and reply routing, `store.rs`'s arms, its tests); `jig/crates/jig-client-protocol-http/**` (`call.rs`'s paths, its tests' fixtures); `jig/crates/jig-client/**` (`client.rs`, `lib.rs`'s re-exports, its tests).
- Readers, compiler-forced arms only, each the nearest existing behaviour (the compiler lists them; `rg` for `Signing::LinkUnknown`, `NotSignedIn::LinkUnknown`, `Request::Authorized`, `UnlinkedRequest::Authorized`, `Message::SignInService` finds today's): in jig, `jig-client-shell-native` (`diagnostics.rs`: a kind per message), `jig/testing/jig-client-kit/src/joint/component.rs` (each new `link::Message` to the component's message of the same name; `Signing::Approval` as the link reasons are), `jig/testing/jig-client-protocol-kit/src/native/wire.rs` (the take a sign-in's kind, the others an exchange's), `jig/testing/jig-client-test-domain`, `jig/testing/jig-client-test-shell-web`, `jig/tests/client`, `jig/tests/ops`, `jig/examples/ops`; in temper, `temper-client-shell-terminal` (`client.rs`, `terminal/native.rs`, `client/tests/fixtures.rs`), `temper-client-shell-cli` (`host.rs`), `temper-client-tools` (`root.rs`, `recovery.rs`), `temper-client-view` (`src/frame.rs`: `NotSignedIn::Approval(_)` draws the ordinary signed-out paragraph, which a browser never meets), `temper-client-domain` (`src/route.rs`: drop `Request::Approval`, `UnlinkedRequest::ApprovalAct` and `UnlinkedRequest::Approval` for now; `temper-domain` gives them meaning), `temper/crates/temper/tests/support` (`managed_client.rs`, `person.rs`), `temper/tests/client/src/component.rs` and `app.rs` (the bridge, as jig's joint's).
- Options:
  - The take's exchange kind: the link's `ExchangeKind::SignIn` with the approval's answer decoded by its own record (`ApprovalSignInAnswer`), or a kind of its own; say which.
  - One approval exchange of each kind in flight, a second answered at once as `Busy`, as `authorize` does; or several. Default: one ask and one read or decision at a time, as `authorize`.
  - If `ApprovalAct` fits better in `jig-client` than in the link (the link never sees it), define it there; it is re-exported from `jig-client` either way.

## Steps

1. **Tests first, red, each refusal before its code** (in the crates' own test modules, beside their siblings; rows of existing tables where one fits, as `each_link_retry_answer_is_visible_and_told_until_the_next_sign_in`):
   - Link, `tests/approval.rs`: `each_approval_message_is_encoded_and_its_answer_decoded`: for the ask, a read and a decision, each wire answer of 2.1 to its `Approval`, a malformed or trailing answer and `Lost` to `Lost`; a second ask in flight `Busy` (if you take that option).
   - Link, `tests/sign_in.rs`: the take's answers, rows `(w::ApprovalSignInAnswer, Signing)`: signed in (the token installed, 40 bytes), waiting, refused, expired, used, unknown, not ready (`Again`), busy, full (`Full { link_until: until }`), and a token not 40 bytes (`Lost`); the take sent with no credential.
   - HTTP, the existing message-to-path table: the four messages to contracts 5's paths, POST, `application/octet-stream`.
   - `jig-client`, `tests/approval.rs`: `an_approval_is_asked_read_and_decided_once_each`: each event gives one `Exchange` and its answer one `Request::Approval`; a read or decision with the credential held, the ask with none; `signed_out` signs the client out.
   - `jig-client`, `tests/sign_in.rs`: `a_waiting_take_is_sent_again_each_second_until_it_ends`: `With::Approval` answered waiting is told `NotSignedIn::Approval(Waiting)` and sent again at `APPROVAL_AGAIN`, the same secret; then signed in (the credential in `link()`), or refused, expired, used, unknown, full (no further send); a `SignIn` of another kind or a `SignOut` meanwhile stops it; not ready, busy and lost take the existing backoff (rows of `a_link_sign_in_answered_not_ready_retries_until_signed_in` and `a_busy_sign_in_retries_until_the_engine_signs_in` if they fit).
   - `jig-client`, `tests/unlinked.rs`: `an_unlinked_approval_is_carried_by_the_host`: each event gives one `UnlinkedRequest::ApprovalAct`, and `ApprovalAnswer` is told back as `UnlinkedRequest::Approval`; `ApprovalAnswer` to a linked client is the existing fault.
   - `jig-client`: `no_print_of_an_approval_shows_its_secret`: with a canary secret, the client's, its requests' and its events' `{:?}` and `{:#?}` hold neither the canary's text nor its byte list (as the wire's `link_sign_in.rs`).

   Commit them red; the report says what each failed on.
2. The types, the link's `approval.rs`, the take beside the link's sign-in, the HTTP paths, then `jig-client`'s events, requests, `SendAgain::Approval` with its one-second wait (the client's `fire` and `next_deadline` take it), and the unlinked mode, until each row passes.
3. The readers' arms, as the compiler lists them.
4. Break each rule once by hand (send the take with the held credential; keep sending after refused; print the secret in a `Debug`) and see its row fail, then restore.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-protocol-link -p jig-client-protocol-http -p jig-client --all-targets -- -D warnings`, and with each reader package you touched; focused tests: `heavy cargo nextest run -p jig-client-protocol-link`, `-p jig-client-protocol-http`, `-p jig-client`, then each reader package you touched, one at a time (at least `-p jig-client-shell-native`, `-p jig-client-world`, `-p jig-client-protocol-world`, `-p temper-client-shell-terminal`, `-p temper-client-shell-cli`, `-p temper-client-view`, `-p temper-client-domain`, `-p temper-client-world`); the wasm32 build of the client crates is the gate's: `heavy cargo check -p jig-client -p jig-client-shell-web --target wasm32-unknown-unknown` once.
6. Commit and push.

## Report

As the implementer's text says, and: the choices among the options; each reader arm, one line each; each red test and what it failed on; each rule broken by hand and the failure seen.
