+++
id = "jig-story"
part = "jig-approval"
kind = "feature"
needs = ["component", "client"]
files = ["jig/testing/jig-client-kit/src/joint/**", "jig/tests/client/tests/suite/approval.rs", "jig/tests/client/tests/suite/main.rs"]
effort = "high"
minutes = 120
+++

## Goal

The approval's meeting point in jig: its stories in jig's joint client world, where real clients meet the real clients' component and the real core (`jig/docs/design/client/domain/testing.md`; `jig/testing/jig-client-kit/src/joint`). A signed-out client asks; a second client, signed in and played as a browser, reads the code and approves; the first takes the approval and is signed in as the second's party, and reads `me` as that party. A client that isn't a browser is refused, and an approval never taken makes no sign-in, with the real people child counting.

Acceptance:

- `jig/tests/client/tests/suite/approval.rs` holds the three stories below, each passing in the focused suite, each within the world's usual cost (name their times).
- The joint kit can play a client as a browser (its exchanges' `browser: true`) and enables the approval in its component's settings; every existing story is unchanged and passes.

## Inputs

- `jig/docs/design/protocol/client.md`, section 8, *A sign-in approved by a browser*.
- `jig/docs/design/client/domain/client.md`, 4.4 and section 12 (the approval's bullets).
- `jig/docs/design/domain/people.md`, section 3 (the third of jig's own ways; *Full sign-ins say so*).
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 3 and 6 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- The joint kit: `session.rs` (`bind`, `bind_unsigned`), `session/signing.rs`, `component.rs` (the composition's per-client links and `link_request`), `front.rs`, `truth.rs`.

## Owns and may change

- Owned: `jig/testing/jig-client-kit/src/joint/**`: a per-client browser mark (say `Session::as_browser(client)`), carried as `browser` on that client's `FromTransport::Exchange`; `Settings.approval: true` in its component; the helpers the stories need to ask, read, decide and take at a client's link and to read the people child's sign-ins (names yours); `jig/tests/client/tests/suite/approval.rs` and its `mod` line in `main.rs`.
- Readers: other worlds that build the joint kit's composition (`rg 'joint::'` in `jig/tests`, `temper/tests`): a new constructor argument is a question; prefer a setter that leaves every caller as it is.
- Options:
  - Ask and take at the composition's link directly (as `session/signing.rs` signs in by `Mint`), and read and decide through the browser client's front, or all at the links: your choice, said in the report. The stories' assertions are on what the engine and the clients show, not on the helpers.
  - If the world's clock cannot pass 10 minutes cheaply, it's a question with the measurement, not a shorter span.

## Steps

1. Write the stories first, red (they fail until the kit carries the mark and the helpers):
   - `a_terminals_approval_approved_by_a_browser_signs_it_in_as_the_browsers_party`: client B bound and played as a browser; client A bound unsigned. A asks: a code and an expiry ten minutes on. B reads the code: waiting. B approves: approved. A takes: signed in, a 40-byte credential handed out once; A's `me` is B's party; B is still signed in, its `me` unchanged; the people child holds one sign-in more. A takes again with the same secret: used. B reads: taken.
   - `a_decision_from_a_client_that_isnt_a_browser_is_refused`: client C bound, not marked; A asks; C reads and decides with its bearer: `not_a_browser` each; A's take: still waiting; then B approves and A signs in.
   - `an_approval_never_taken_makes_no_sign_in`: A asks, B approves, the world's clocks past ten minutes and the component's deadline fired: A's take is expired, B's read expired, and the people child's sign-ins are as before the ask.
2. The kit's mark, settings and helpers, until the stories pass.
3. Break each rule once by hand in the component (`jig-protocol-engine-clients/src/approval.rs`, restored after): let a non-browser decide; skip the expiry; and see the story fail for that reason.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-client-kit -p jig-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-client-world`, then the other worlds that use the joint kit, one at a time (`rg -l jig-client-kit jig/tests/*/Cargo.toml temper/tests/*/Cargo.toml`), at least `-p temper-client-world`.
5. Commit and push.

## Report

As the implementer's text says, and: the helpers' names and where each step of a story happens (link or front); each rule broken and the failure seen; the stories' times.
