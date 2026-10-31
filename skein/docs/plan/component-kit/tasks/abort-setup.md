+++
id = "abort-setup"
part = "abort"
kind = "feature"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/**", "skein/tests/forge/setup/src/host.rs", "skein/tests/forge/setup/src/memory.rs", "skein/tests/forge/setup/tests/suite/host.rs", "skein/tests/forge/setup/tests/suite/main.rs", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 45
+++
# Abort in skein-forge-setup's host

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.6; review D9; programming-model.md 5.2 (as abort-connection's
spec quotes it).

## What changes

- `src/host/boundary.rs:111` `Request` (Run, Answer, Close) gains
  `Abort`, documented in 5.2's words.
- The host handles it: each run it holds ends unheard, as `Close` ends
  them (forge-setup.md, 6.3; see Answers), each socket is aborted (`host/connection.rs:294` already aborts one), and its
  closed event is the terminal. Abort after close turns what still
  closes into aborts; close after abort changes nothing. The model is
  skein-llm-connection's (`src/component.rs:339-357`).
- No TLS `close_notify`.

## Acceptance

- Step tests in `src/host/tests.rs`: abort while running, after close,
  close after abort, a second abort.
- A world test in `skein/tests/forge/setup/tests/suite/host.rs`: a run
  aborted mid-call ends with the closed event and nothing left held.
- `heavy cargo nextest run -p skein-forge-setup -p <tests/forge/setup's
  package>`, and `heavy cargo check --tests -p temper-forge-setup-shell`;
  an owner that matches `Request` exhaustively is a `Question:`.

## Answers

2026-10-31. How does Abort report each held run as cancelled, with
`host::Event` holding only Need, Done, Stop and Closed? By `Closed`
alone: no per-run event, no new `Why`, no new `Event` variant. The
design already makes `Closed` the terminal of every run a close ends
("`Close` ends every run unheard", forge-setup.md, 6.3), and this
host's `Close` already aborts each socket rather than waiting; `Abort`
ends the runs the same way, marks the host aborting, and `Closed`
follows once io reports every socket closed. Abort after close changes
what is left (any socket still closing gracefully is aborted); close
after abort, or a second abort, changes nothing. A `Why::Cancelled`
or an `Event::Cancelled` would change public types temper matches
exhaustively (temper-service's `setup/lifecycle.rs` matches `Why`):
not this task's. The world test checks that no `Done`, `Stop` or
`Need` follows the abort and that `Closed` is the last event. Put
in your report the sentence 6.3 should gain for `Abort`; the design
is the designer's.

2026-10-31. May the task change `skein/tests/forge/setup/src/memory.rs`,
whose `request_bytes` matches `Request` exhaustively? Yes: it is now in
`files`. `Request::Abort` carries no bytes, so it joins the zero-byte
arm beside `Request::Close`; nothing else in that file changes. The
file belongs to the world package this task already changes and no
other task owns it; the "exhaustive owner is a `Question:`" rule in
Acceptance stays for owners outside `files` (temper's included).
