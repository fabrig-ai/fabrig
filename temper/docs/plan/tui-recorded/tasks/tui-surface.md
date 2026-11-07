+++
id = "tui-surface"
part = "tui-approval-flow"
kind = "interface"
needs = []
files = ["temper/crates/temper-client-tui/src/state.rs", "temper/crates/temper-client-tui/src/approval.rs", "temper/crates/temper-client-tui/src/lib.rs", "temper/crates/temper-client-tui/src/view.rs", "temper/crates/temper-client-tui/src/tests/state.rs", "temper/crates/temper-client-tui/src/tests/approval.rs", "temper/crates/temper-client-tui/src/tests/fixtures.rs", "temper/crates/temper-client-tui/src/tests.rs"]
effort = "high"
minutes = 60
+++

## Goal

The surface the TUI's host and view build the browser's approval on, in `temper-client-tui`: the TUI's state learns of the tab (`Told::Browse`, `Told::Opened`, `Browser`, `Tui::browser()`, `Tui::pasted()`), the approval page's address made from the TUI's own parts (`approval_address`), whether Enter asks again (`may_ask_again`), and the view's clock (`build`'s `wall`, `View::tick`). Small on purpose: `approval-lines` draws on it and `approval-host` drives it, at once.

Acceptance:

- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 1, holds in `temper-client-tui` as written, re-exported from `lib.rs`; `View::tick` returns `None` and `build` takes and ignores `wall` beyond its rebuild rule (section 1).
- State rows in `src/tests/state.rs`'s table: `Browse` sets `Opening` and clears `pasted`; `Opened { opened: true | false }` sets `Opened` or `None`; `Asked` sets `pasted`; signed out after a sign-in clears both; each change moves `revision`, a repeat does not.
- A table test of `approval_address` (`src/tests/approval.rs`): rows of addresses (`127.0.0.1:8367`, `127.0.0.1:9000`, `[::1]:8367`) and codes, each the exact bytes `http://HOST:PORT/terminal?code=XXXX-XXXX`, the code as `ApprovalCode::shown()` gives it.
- A table test of `may_ask_again`: each `asked()` state (`None`, `Asking`, `Asked`, each `NotAsked`) against each `not_signed_in` (`None`, each `Approval(..)`, `Full`, `Busy`, `NotReady`) and `pasted`, read from the design's rule (contracts, section 1); the link signed in makes it false.
- `heavy cargo nextest run -p temper-client-tui` passes; clippy clean for `temper-client-tui` and `temper-client-shell-tui`.

## Inputs

- `temper/docs/design/client/README.md`, 6.14 (the TUI's state) and 7.1 (*The root's half*, *The TUI's flow*); 5.12, *The tick*.
- `temper/docs/plan/tui-recorded/notes/contracts.md`, section 1 (binding).
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The code: `temper/crates/temper-client-tui/src/state.rs` (`Tui::step`'s rule that a change moves `revision`), `src/view.rs` (`build` and its `built` key), `temper/crates/temper-client-domain/src/asked.rs` (`asked::State`), `src/address.rs` (`print_approve`), `jig/crates/jig-client/src/client.rs` (`NotSignedIn`, `LinkView`), `jig/crates/jig-client-protocol-link/src/boundary.rs` (`Taking`, `NotAsked`).

## Owns and may change

- Owned: the files above. `approval.rs` is new: `approval_address` and `may_ask_again`, with a module comment citing `client/README.md`, 7.1.
- Readers, call sites only: `temper/crates/temper-client-shell-tui/src/host.rs`'s one `tui::build` call gains `self.env.wall`; `temper-client-tui`'s test fixtures' `build` helper gains a fixed wall (the fixtures' `NOW`, or `Wall::EPOCH` where they keep none). Nothing else of the host changes: `approval-host` owns it.
- Options:
  - `build`'s rebuild key: keep `built` as it is and add the tick's passing (default); if the key must grow a field, grow it and say so.
  - `may_ask_again` reading `root.client().link()` and `root.asked()` (default); if a reading needs a root accessor that does not exist, stop: `temper-client-domain` is not this task's.

## Steps

1. Read the inputs. Add `Browser`, the two `Told` variants and their steps, the two accessors; `approval.rs`; `build`'s `wall` and `View::tick`.
2. Tests: the state rows, then the two tables. Break each rule once by hand (a step that forgets to clear `pasted`; an address joined without `http://`; `may_ask_again` true while `Asking`), see the row fail, restore; name them in the report.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-tui -p temper-client-shell-tui --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-tui -p temper-client-shell-tui`.
4. Commit and push.

## Report

The implementer's report, with: the rows added and what each broken rule's failure showed; the call sites changed outside the crate; anything of the contract you found unclear.
