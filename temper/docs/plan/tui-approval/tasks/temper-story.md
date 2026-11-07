+++
id = "temper-story"
part = "temper-approval"
kind = "feature"
needs = ["temper-domain", "component", "client"]
files = ["temper/tests/client/src/component.rs", "temper/tests/client/src/front.rs", "temper/tests/client/tests/suite/terminal_approval.rs", "temper/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 150
+++

## Goal

The approval's meeting point in temper: stories in temper's client world (`temper/tests/client`, `temper-client-world`), where temper's client roots meet jig's real clients' component and temper's engine root (`temper/docs/design/domain/testing.md`, 6.3). A client, signed out, asks; a second client, signed in and played as a browser, opens the approval page at the code through its root, reads it waiting and approves; the first takes the approval and is signed in as the second's party, and its root draws `me` as that party. A client that isn't a browser opening the page is answered not a browser, and its page says "Sign in first".

Acceptance:

- `temper/tests/client/tests/suite/terminal_approval.rs` holds the two stories below, each passing in the focused suite, each within the world's usual cost (name their times).
- The world can enable the approval in its composition's component, play a client as a browser (its exchanges' `browser: true`), carry a root's `UnlinkedRequest::ApprovalAct` to that client's link and its answer back as `jig_client::Event::ApprovalAnswer`, and ask and take an approval at a client's link; every existing story is unchanged and passes.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.3 (the client and the engine, joined).
- `temper/docs/design/client/README.md`, 3.5, *The approval page*, and 7.1, *The root's half*.
- `jig/docs/design/protocol/client.md`, section 8, *A sign-in approved by a browser*.
- `jig/docs/design/client/domain/client.md`, 4.4 (the unlinked mode's approval) and section 12.
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 3, 6 and 7 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- The world: `temper/tests/client/src/component.rs` (`Composition`: its settings literal in `new`, `link`, `link_request`, `Incoming`), `src/front.rs` (`Joint`: its constructors, `request`'s parked path, `incoming`, `bind`), `src/app.rs`, and the two-client pump of `tests/suite/chat.rs` (`probe`, `pump`, `accepting_the_goal_with_chat_and_inbox_watches_settles_the_engine`), which the stories copy.
- jig's joint kit after `jig-story`, if it has landed (`jig/testing/jig-client-kit/src/joint/`): its browser mark and approval helpers are the shape to copy; temper's composition is its own, so nothing of it is shared but `joint::transport`.

## Owns and may change

- Owned: `temper/tests/client/src/component.rs` and `src/front.rs` (below); the new story file and its `mod terminal_approval;` line in `tests/suite/main.rs`.
  - *The approval enabled:* a constructor that builds the composition with `Settings.approval` true, `Joint::with_approval(seed, limits)` beside `Joint::new`, through `Composition::new`'s settings (a `pub fn new_with_approval(configuration, limits)` that `new` shares its body with, or an argument of a private builder both call). No existing constructor or caller changes.
  - *A browser:* `Joint::as_browser(client)` and `Composition::as_browser(client)`, a set of marked clients, read in `link_request` as `browser: self.browsers.contains(&client)` on that client's `FromTransport::Exchange`, where `surface` left `browser: false`.
  - *The page's exchanges:* in `Joint::request`, before the parked path, a `Request::Unlinked { request: UnlinkedRequest::ApprovalAct(act) }` goes to `Composition::approval(client, act)`, which steps that client's link with the link's approval event for the act (the `client` task's name; read `jig/crates/jig-client-protocol-link/src/boundary.rs`); the link's answer, `link::Request::Approval(answer)` or the name `client` gave it, becomes `Incoming::Callback { client, event: jig_client_test_domain::Event::Client { event: jig_client::Event::ApprovalAnswer { answer } } }` in `link_request`, replacing the arm `client` left there.
  - *The ask and the take at a client's link:* `Composition::ask_approval(client)` sends the ask on that client's link and keeps its `Asked` answer (secret, code, expiry) for that client in the composition, never printed; `Composition::take_approval(client)` sends `link::Event::SignIn { with: link::With::Approval { secret } }` with the kept secret; `Joint::ask_approval(client) -> (ApprovalCode, Wall)` and `Joint::take_approval(client)` call them and pump. The take's `Signing::Approval(taking)` becomes a new `Incoming::Taking { client, taking }`, which the front records for the story (`Joint::taking(client) -> Option<jig_client::Taking>`), never a panic; `Signing::SignedIn { party }` stays `Incoming::Authenticated`, on which the front binds that client in its transport to the answered party, as `bind` binds a minted one, before it releases the client's parked requests, and gives its root `Up::SignedIn` if the world's roots need it to reopen `me` (read how `bind` and `incoming` start a client; say which in the report).
- Readers: no other world builds temper's composition (`rg 'Composition::new' temper/tests`); `jig-story` edits jig's joint kit, not this world.
- Options:
  - A's ask through A's root (`temper_client_domain::Event::AskApproval`, which needs A's root signed out, an `Up::SignedOut` given it first, and its `ApprovalAct::Ask` carried before the parked path), or at A's composition link (default, the lead's preference): if the root path costs no more than the helpers above, take it, since it also exercises `asked` unlinked (`Domain::asked()` holding the code and the expiry); say which.
  - The page's acts as root events (`Event::Went { place: Place::Approve { code: Some(code) } }`, `Event::Act { action: Action::Approve(approve::Act::Approve) }`, default) or through the view with skein's fake person, as the world's other stories do where they test words; the stories' assertions are on what the engine and the roots show either way.
  - If the world's clock cannot reach a state the stories need cheaply, it's a question with the measurement.

## Steps

1. **The stories first, red** (they fail until the world carries the approval):
   - `a_terminal_approved_by_a_browser_is_signed_in_as_the_browsers_party`: `Joint::with_approval`; client B bound as `fixtures::PARTY` and marked a browser, its root started at the inbox and live; client A not bound, its root started. A asks: a code, an expiry ten minutes past the world's wall. B's root goes to `Place::Approve { code: Some(code) }`: one read leaves, and B's page is the ask (`approve::State::Asking { asked }`, `asked` the ask's wall). B approves: one decision leaves, and the page is approved, then, at the next `approve::AGAIN`, read again, still approved. A takes: signed in; A's root reads `me` and draws `fixtures::PARTY`'s party; B is still signed in and its `me` unchanged. B's next read: taken, its page signed in ("Signed in. You can close this tab." if the story draws the view). A takes again with the same secret: used.
   - `a_page_in_a_client_that_isnt_a_browser_is_refused`: `Joint::with_approval`; B as above, and client C bound as the same party, not marked. A asks. C's root goes to the page at the code: its read is answered not a browser, C's page is signed out ("Sign in first"), and C stays signed in. A's take: waiting (`Joint::taking(A)` is `Taking::Waiting`). Then B approves and A's next take signs it in.
   Commit them red; the report says what each failed on.
2. The world's constructor, mark, carrying and helpers, until both pass.
3. Break each rule once by hand, restored after: mark C a browser and see the second story fail on it; drop the browser mark from B and see the first fail at its read.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world -E 'test(terminal_approval)'`, then the whole `heavy cargo nextest run -p temper-client-world` once, its count checked.
5. Commit and push.

## Report

As the implementer's text says, and: the choices among the options; the helpers' names and where each step of a story happens (link, root or view); how A's root learns it is signed in; each rule broken and the failure seen; the stories' times.
