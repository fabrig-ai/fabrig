+++
id = "approval-story"
part = "tui-approval-flow"
kind = "feature"
needs = ["approval-host", "approval-lines"]
files = ["temper/tests/client/src/tui.rs", "temper/tests/client/tests/suite/tui.rs", "temper/tests/client/tests/suite/terminal_approval.rs"]
effort = "high"
minutes = 120
+++

## Goal

The TUI's approval, end to end, in temper's client world (`temper/docs/design/domain/testing.md`, 6.3): the TUI's shipped host, signed out, asks; its sign-in screen draws the code and the address it opens; a second client, signed in and played as a browser, opens the approval page at that code through its root and approves; the TUI takes it, keeps the credential and draws its first screen, "Connected to 127.0.0.1:8367 as Owner". And the endings' lines through the real engine: refused, and asked again.

Acceptance:

- `temper/tests/client/tests/suite/tui.rs` holds, each passing in the focused suite within the world's usual cost (name their times):
  - `a_tuis_approval_approved_by_a_browser_draws_the_first_screen`: the TUI started with no credential on a world with the approval enabled (`Joint::with_approval`); its first duties ask; once answered, its duty `open` is `http://127.0.0.1:8367/terminal?code=CODE`, CODE the code the engine drew, and its frame shows CODE and that address; the browser's root opens `Place::Approve { code: Some(CODE) }` and approves; the TUI's host keeps a credential (`Tui::credential()`), its frame says "Connected to 127.0.0.1:8367 as" the browser's party's name, and its status "● Live";
  - `a_tuis_approval_refused_says_so_and_asks_again`: the browser refuses; the TUI's frame says "The browser refused this sign-in. Press Enter to ask again."; Enter in the empty field asks again, and a new `open` carries the new code.
- The code is read from the TUI's own duty, never from the engine's answer: the story parses it from the opened address with `ApprovalCode`'s reader.
- Every existing story of the client world passes (`heavy cargo nextest run -p temper-client-world`).
- Clippy clean for `temper-client-world`.

## Inputs

- `temper/docs/design/domain/testing.md`, 6.3.
- `temper/docs/design/client/README.md`, 7.1, *The TUI's flow*, and 3.5, *The approval page*.
- `temper/docs/plan/tui-recorded/notes/contracts.md`, sections 1 and 2.
- `temper/docs/plan/tui-recorded/notes/common.md`.
- The world: `temper/tests/client/src/tui.rs` (the TUI's harness: `start`, `press`, `run`, `text`), `tests/suite/tui.rs` (its stories and `signed_in`), `tests/suite/terminal_approval.rs` (`setup`, `probe`, `pump`, `open`, `approve`: a browser client played through its root), `src/front.rs` (`Joint::with_approval`, `as_browser`, `bind`).

## Owns and may change

- Owned: the files above. The harness gains what the stories need: carrying `duties.open` (kept for the story to read, `Opened { opened: true }` reported back), and a second client played as a browser beside the TUI in one world, driven through its root as `terminal_approval.rs`'s probes are.
- Options:
  - The browser's client in the TUI's `World` (default): a root client of the world's, bound to the owner's party and marked a browser; if the `World` harness cannot hold a root client beside the TUI's host, drive both in `terminal_approval.rs`'s pump instead, with the TUI's host stepped by it, and say so.
  - Shared helpers between `tui.rs` and `terminal_approval.rs` move into the harness (`src/tui.rs`) only where both use them; never change what an existing story checks.

## Steps

1. Read the inputs; build the harness's half; write the two stories.
2. Break each rule once by hand (the host opening the engine's bytes; Enter not asking again), see the story fail, restore; name them.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world`.
4. Commit and push.

## Report

The implementer's report, with: the stories' times; how the browser's client stands beside the TUI's host; each broken rule's failure.

## Answers

### 2026-11-07: Resume after approval-host pushes its implementation (recommended), or expand this task to implement the missing host behavior? (tui-recorded/approval-story-1, foreman tui-recorded/approval-story/foreman-1)

Resume after approval-host pushes its implementation; keep approval-story within its owned files and complete its acceptance.
The spec declares `needs = ["approval-host", "approval-lines"]`.
Contracts §2 assigns “The host's seam (`temper-client-shell-tui`'s `host`), task `approval-host`”, including `Report::Opened`, the startup ask, opening duty and Enter asking again.
The checkout's `temper/crates/temper-client-shell-tui/src/host.rs` still lacks `Report::Opened`; that implementation belongs to the prerequisite.
Acceptance requires both stories “each passing in the focused suite within the world's usual cost (name their times)”; the report explicitly says acceptance is unfinished.
The next run incorporates the completed dependency, verifies the final harness, completes both hand-mutation proofs, runs the required checks, records passing story times, and commits and pushes the completed work.
