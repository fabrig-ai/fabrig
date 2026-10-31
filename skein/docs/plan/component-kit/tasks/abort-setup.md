+++
id = "abort-setup"
part = "abort"
kind = "feature"
repo = "fabrig"
needs = ["ratchets"]
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/**", "skein/tests/forge/setup/src/host.rs", "skein/tests/forge/setup/tests/suite/host.rs", "skein/tests/forge/setup/tests/suite/main.rs", "skein/scripts/old-forms.txt"]
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
- The host handles it: each run it holds ends as cancelled, each socket
  is aborted (`host/connection.rs:294` already aborts one), and its
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
