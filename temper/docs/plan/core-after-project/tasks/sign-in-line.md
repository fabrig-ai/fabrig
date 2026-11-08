+++
id = "sign-in-line"
part = "core-says-why"
kind = "feature"
needs = []
files = ["temper/crates/temper-client-shell-cli/src/host.rs", "temper/crates/temper-client-shell-cli/src/tests.rs", "temper/docs/design/client/tools.status.md"]
effort = "medium"
minutes = 45
+++

## Goal

A `temper ctl sign-in` whose link the engine answered *not ready* until the deadline says so: temper is starting, or its core is not answering, and `temper serve`'s log says why. It replaces the deadline's generic `did not answer within 30 s`, which a person reads as the engine being down (`temper/docs/design/client/tools.md`, 8.1, `sign-in`'s *not ready* and *busy*). In the incident (`temper/docs/plan/core-after-project/inputs/request.md`, act 7) a fresh client's sign-in said only that.

Acceptance:

- Past the sign-in's deadline, a sign-in whose last answer was `jig_client::NotSignedIn::NotReady` exits 1 with one stderr line, exactly: `temper ctl: temper is not ready: it is starting, or its core is not answering, and temper serve's log says why; this link stays usable until it expires`.
- The busy line keeps its place and words: a sign-in last answered busy still prints `temper is busy; this link stays usable until it expires`. A sign-in never answered (`quiet` link path, or nothing heard) keeps `the engine at ADDRESS did not answer within 30 s`. `sign-out` keeps its generic line (`tests/deadline.rs`, `sign_out_keeps_the_generic_deadline_line`).
- Rows, red first: `repeated_again_keeps_the_original_sign_in_deadline` (`src/tests.rs`) asserts the new line, keeping its deadline arithmetic as it is. A new row beside it, `a_sign_in_answered_not_ready_then_never_again_says_not_ready`: one not-ready answer (`again_links = 1`), then the link path quiet (`quiet.push(b"/jig/v1/sign-in/link")`) to the deadline, prints the new line. Break the rule once (the not-ready check removed) and see both rows fail on the generic line; restore.
- `temper/docs/design/client/tools.status.md`: the *Not yet* entry for 8.1's not-ready line is removed.
- `heavy cargo nextest run -p temper-client-shell-cli` passes; clippy clean for it.

## Inputs

- `temper/docs/plan/core-after-project/notes/common.md`.
- `temper/docs/design/client/tools.md`, 8.1, the `sign-in` bullet (its answers), and 6.3's *The deadline says what it waited for* (the order: busy's and the full streams' lines come first).
- The code: `temper-client-shell-cli/src/host.rs`, the deadline branch (`self.signing.as_ref().is_some_and(|signing| signing.last == Some(jig_client::NotSignedIn::Busy))`); `src/tests/fixtures.rs`, `again_links`, `busy_links`, `quiet`.

## Owns and may change

- Owned: the files above.
- Option: the check sits beside busy's, after it (default), so a sign-in last answered busy keeps busy's line. If the client's `signing.last` is overwritten by a later answer before the deadline, the line follows the *last* answer, as busy's does; say so in the report.
- Not yours: jig's client (`jig-client`), the wire, and every other deadline line (`src/waited.rs`).

## Steps

1. Read the inputs.
2. The rows, red first; then the check and the line.
3. Break the check by hand, see both rows fail, restore.
4. Remove the status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; clippy and the tests above.
6. Commit and push.

## Report

The rows and their red failure; the broken-by-hand failure; the new line as the test asserts it.
