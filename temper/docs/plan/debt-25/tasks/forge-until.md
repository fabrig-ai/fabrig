+++
id = "forge-until"
part = "spot-review"
kind = "feature"
needs = ["connect-cells"]
files = ["skein/crates/skein-forge-connection/src/**", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 40
+++

# skein-forge-connection: a holder hands its deadline to `exchange.start`

## Goal

Finding 4 in `temper/docs/plan/debt-25/inputs/debt23-review.md`, checked against the code. Nine `rest` calls choose a connection's deadline, against `skein/docs/design/forge-connection.md`, 8.1. Three of them come just before a send, where the deadline is the holder's, so it belongs in `exchange.start`. The other six set what a connection falls back to as it leaves a holder.

Acceptance:

- `Connection::rest` is `Connection::until`, and `seam.rs`'s `test_rest` is `test_until`.
- `forgejo::Exchange::start` and `github::Exchange::start` take `until: Option<Time>`, and their first line is `c.until(until)`, so a refused start leaves the deadline it does today.
- The three pre-send `rest` calls go: `component/calls.rs` (`Some(deadline)`), `component/github.rs` (`env.limits.deadline`) and `component/github_calls.rs` (`Some(deadline)`), each passing it to `start` instead.
- `send_start` in `component.rs`, which sets nothing today, passes `connection.deadline()`.
- The six calls left keep their values: `component.rs`'s `start_heard`, `component/calls.rs`'s two `idle`s, `component/github.rs`'s `idle` and `None`, `component/github_calls.rs`'s `idle`.
- `rg -n '\.rest\(|test_rest' skein/crates/skein-forge-connection skein/tests/forge skein/testing/skein-forge-kit` reads 0.
- `skein/docs/design/forge-connection.status.md`, 8.1: "nine `rest` calls in `component.rs` and `component/` set it as a holder takes or leaves a connection" becomes "a holder passes it to `exchange.start` as it takes a connection, and six `until` calls in `component.rs` and `component/` set it as one leaves"; and "`component/github.rs` rests one with no deadline" becomes "`component/github.rs` leaves one with no deadline". Date the file as your commit's day.
- No test's assertion changes.

## Inputs

- `temper/docs/plan/debt-25/notes/common.md`
- `temper/docs/plan/debt-25/inputs/debt23-review.md`, finding 4
- `skein/docs/design/forge-connection.md`, 8.1
- `skein/docs/design/forge-connection.status.md`

## Owns and may change

- **Owns:** `skein/crates/skein-forge-connection/src/**`; in the status file only the two phrases above, which departs from the implementer's text.
- **Reader:** `skein/tests/forge/machine/tests/suite/hand/forgejo/connection.rs`, for `test_rest`'s spelling only.
- **Arguments:** the Forgejo `start` goes past clippy's limit. A scoped `#[expect(clippy::too_many_arguments, reason = "...")]`, as this crate's other calls have, is the default. Not a tuple, and not a parcel unpacked on the first line.

## Steps

1. The rename, with the seam and its reader. Commit.
2. `start`'s parameter, its four callers, then the status phrases.
3. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-forge-connection --lib`, then `-p skein-forge-machine-world` and `-p skein-forge-protocol-world`. Commit, run the citations tool and push.

## Report

The common note's items. This is a move: no rule to break by hand.
