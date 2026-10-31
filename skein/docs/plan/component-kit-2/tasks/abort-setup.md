+++
id = "abort-setup"
part = "abort"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-setup/src/host.rs", "skein/crates/skein-forge-setup/src/host/**", "skein/crates/skein-forge-setup/src/tests/host.rs", "skein/tests/forge/setup/src/memory.rs"]
effort = "high"
minutes = 40
+++
# `Request::Abort` in skein-forge-setup's host

## Goal

Draft, 3.6; programming-model.md, 5.2: every component takes an abort,
"a close that does not wait". skein-forge-setup's host component takes
none today. Add `Request::Abort`: it ends every run unheard, as `Close`
does (forge-setup.md, 6.3), aborts every connection it keeps, and the
host says `Event::Closed` once, when io has settled every socket. The TLS
`close_notify` is not part of this task.

Acceptance:

- `host::Request::Abort` exists, with a `///` contract.
- Tests, one behaviour each, in `src/tests/host.rs` (on its world `w`)
  pass:
  - an abort with runs open sends `skein_io::Request::Abort` for each
    connection, no `Done` or `Stop` for the runs, and `Closed` once io
    has settled them;
  - an abort after a close changes nothing a close had not already sent
    as an abort (the host's close already aborts its connections), and
    `Closed` goes up once;
  - a close or a second abort after an abort changes nothing.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world`
  passes.
- `heavy cargo check -p temper-forge-setup-shell -p temper-service --tests`
  passes.

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`.
- `skein/docs/foundation/programming-model.md`, 5.2;
  `skein/docs/design/forge-setup.md`, 6.3 (the host component, its
  boundary table and "Close ends every run unheard").
- The code: `src/host/boundary.rs:111-115` (`Request`), `src/host.rs:112-130`
  (`Close`: `closing`, the runs dropped, each connection's `abort()`),
  `src/host/connection.rs:275` (`abort`), `src/host/tests.rs:6`
  (`abort_upgrades_a_graceful_close_already_sent_to_io`).

## Owns and may change

- `skein/crates/skein-forge-setup/src/host.rs`, `src/host/**`,
  `src/tests/host.rs`.
- `skein/tests/forge/setup/src/memory.rs`: `Request::Abort` joins the
  zero-byte arm of `request_bytes` beside `Request::Close` (line ~151).
  Nothing else there.
- Any other reader in skein or temper that matches the host's `Request`
  exhaustively (the survey found only `memory.rs`): `Request::Abort`
  joins `Request::Close`'s arm, nothing else; name each in the report.
- Nothing else: `lib-forge-setup` and `stage-forge` change these crates
  later.

Settled (do not ask):

- **No new event or failure.** No per-run event, no `Why::Cancelled`,
  no `Event::Cancelled`: temper-service matches `Why` exhaustively. Runs
  end unheard, as under `Close`; `Closed` is the one terminal.
- If `Close` and `Abort` are the same path in this host (its close
  already aborts), say so in `Abort`'s contract and share the code; the
  difference 5.2 asks for is that `Abort` never waits on a graceful
  close.

## Steps

1. `Request::Abort` in `src/host/boundary.rs`, after `Close`, with a
   `///` contract: "A close that does not wait (programming-model.md,
   5.2): every run ends unheard, every connection is aborted;
   `Event::Closed` follows once, after a `Close` or an `Abort`."
   Give `Close` a one-line `///` too if it has none.
2. The host handles it (`src/host.rs`), sharing `Close`'s path.
3. The three tests above.
4. `memory.rs`'s arm.
5. Checks of `common.md` and the temper check. Commit; push.

## Report

The contract as written, whether close and abort differ in this host and
how, the tests added (names, counts), the readers edited, the checks run.
