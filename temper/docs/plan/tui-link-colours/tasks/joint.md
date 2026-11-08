+++
id = "joint"
part = "joint"
kind = "feature"
needs = ["host-local"]
files = ["temper/tests/client/src/tui.rs", "temper/tests/client/src/component.rs", "temper/tests/client/src/front.rs", "temper/tests/client/tests/suite/tui.rs", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 150
+++

# tui-link-colours: the TUI signed in from the engine's link, in the client's joint world

## Goal

The client's joint world (`temper/docs/design/domain/testing.md`, 6.3) proves the
sign-in from the engine's link with temper's real engine root and jig's real clients'
component under the shipped TUI host: a fresh TUI signed in with no key pressed; the same
after a browser spent the first link, by the one its own try drew; a stale file falling
back with nothing alarming; an expired sign-in renewed; a signed-out one never renewed;
and the file's secret in no frame.

Acceptance:

- The world's stand-in for the file (`src/component.rs`): the composition keeps the
  latest secret its clients' component drew (`Request::Link { secret }`, dropped today),
  starting with the printed secret it was given; `Joint` (`src/front.rs`) gives the URL
  of that latest link at `127.0.0.1:8367`, as `printed_link()` spells the first, by a
  function of your naming (`latest_link`, say).
- The harness (`src/tui.rs`): `Tui::start` keeps its signature and behaviour (`local:
  false`, `note: None`). A second constructor of your naming takes the file and the
  note: the file `Absent`, given `Bytes`, or `Engine`, the world's latest link read at
  each duty `local`. `carry` answers `Duties.local` with `Report::Local { link }` at once,
  counting the reads (`pub reads: usize`), and keeps each `Duties.note` (`pub notes:
  Vec<Note>`).
- Stories in `tests/suite/tui.rs`, beside its siblings and their `fixtures::tui_world`,
  each a failing test first where it pins a refusal:
  1. `a_fresh_tui_signs_in_from_the_engines_link_with_no_key`: file `Engine`, no
     credential, no key: the first screen, "Connected to 127.0.0.1:8367 as Person", "●
     Live", a credential kept, one read, no approval asked, nothing opened, and the last
     note `Until` some wall time.
  2. `a_tui_after_a_browser_spent_the_link_signs_in_by_the_next`: another client signs in
     by the printed link first (a second `Tui` by its paste, or the world's browser as
     `approval()` makes one); the TUI with file `Engine` reads it, is answered used, reads
     again after a second, and signs in by the fresh link: two reads or three, live, no
     approval asked.
  3. `a_stale_files_link_falls_back_with_nothing_alarming`: file `Bytes` of a link at
     `127.0.0.1:8367` whose secret the engine never drew: the sign-in screen, "Sign in
     this terminal to 127.0.0.1:8367", one approval asked, and none of the paste's refusal
     lines ("isn't from this temper", "already used", "refused"); and a row with a link
     for `127.0.0.1:9000`: no sign-in sent, the same screen.
  4. `an_expired_sign_in_is_renewed_from_the_file`: started with a credential the engine
     never made and the note `Until` an hour before the world's wall time, file
     `Engine`: the credential erased, then signed in from the file, live, no key pressed.
  5. The existing `a_tui_signed_out_elsewhere_asks_for_a_link_with_its_token_erased`,
     started with file `Engine` (it signs in by its paste as now): after the sign-out
     elsewhere, the last note is `SignedOut` and no read follows; a fresh host started
     with that note and file `Engine` reads nothing and asks the approval; its paste
     signs it in and notes `Until` again.
  6. In stories 1 and 4, every frame drawn holds neither the latest link's secret in
     URL-base64 nor its URL.
- Status: delete `temper/docs/design/client/README.status.md`'s **Not yet** "10, *The TUI
  in the joint world*: the stories ... are not written." and nothing else (a stated
  departure from the implementer's text).

## Inputs

- `temper/docs/design/client/README.md`: 6.13 (the seam), 6.14 (*Signed in from the
  engine's link*, all of it), 7.1 (*The browser's way in after the TUI's link*), 10 (*The
  TUI in the joint world*).
- `temper/docs/design/domain/testing.md`, 6.3.
- `jig/docs/design/protocol/client.md`, section 8, *The printed link*.
- `temper/docs/plan/tui-link-colours/notes/common.md`;
  `temper/docs/plan/tui-link-colours/notes/contracts.md`, sections 3 and 4.

## Owns and may change

- Owns the files in `files`. Other suites of `temper/tests/client` are other drafts'
  (`ui-account-again` changes `account_tools.rs` and `front/account_acts.rs`): leave them.
- `component.rs`'s change only keeps the latest drawn secret: every other world's
  behaviour stays as it is. Its `Request::Link` arm keeps the secret; nothing prints it.
- If the joint component draws no fresh link after a used one (it should: jig's
  component draws one when a spent link is tried and none is live), story 2 fails for
  that reason: report it as a question with the failure, and keep the other stories.
- If the world's wall time can't be read before the start for story 4's note, give the
  note `Until(Wall::EPOCH)` plus one nanosecond, and say so.
- Budgets: each story within its siblings' cost; no longer `run` spans than theirs but
  the one second story 2 waits for its second read.

## Steps

1. The harness, then the stories; `heavy cargo nextest run -p temper-client-world -E
   'test(/tui::/)'`; see each refusal's story fail first.
2. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-client-world`.
3. Commit, push.

## Report

As the implementer's text says; the reads each story saw, and each failure seen.
