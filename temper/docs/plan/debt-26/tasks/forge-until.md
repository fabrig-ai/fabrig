+++
id = "forge-until"
part = "forge-until"
kind = "feature"
needs = []
files = ["skein/crates/skein-forge-connection/src/connection.rs", "skein/crates/skein-forge-connection/src/connection/forgejo.rs", "skein/crates/skein-forge-connection/src/connection/github.rs", "skein/crates/skein-forge-connection/src/component.rs", "skein/crates/skein-forge-connection/src/component/calls.rs", "skein/crates/skein-forge-connection/src/component/github.rs", "skein/crates/skein-forge-connection/src/component/github_calls.rs", "skein/crates/skein-forge-connection/src/tests/fixtures.rs", "skein/docs/design/forge-connection.status.md"]
effort = "high"
minutes = 45
+++

# Forge connections: undo the `until` parameter, and a connect that can't reach `Start::Ready` from Closed

## Goal

debt-25 moved a holder's deadline into `exchange.start`: about +32 production, two new `too_many_arguments` expects and a self-write, and holders still choose the deadline. fabrig/debt decided to undo the parameter (the spot review of debt-25, finding 3). The same review's finding 6 names a state that drops `ended`, in the same file.

Acceptance:

- `connection/forgejo.rs`'s and `connection/github.rs`'s `Exchange::start` lose `until` and its `c.until(until)` line, and their two `#[expect(clippy::too_many_arguments, ...)]`.
- Each holder sets the deadline above its `start`, `c.until(Some(deadline));`: `component/calls.rs`, `component/github.rs` and `component/github_calls.rs`.
- `component.rs`'s `send_start` loses `let until = connection.deadline();`, and `tests/fixtures.rs`'s two `start_*` lose theirs: a start read runs under the deadline its connection already holds.
- `Connection::connect` on `Closed` reaches no `state.idle()`: `Closed` + `Start::Ready` is `unreachable!`, with a message saying why, as long as the tests below hold it.
- `skein/docs/design/forge-connection.status.md`, 8.1 says what's true, in the words below.
- `skein-forge-connection`'s lib tests and `skein-forge-machine-world`'s, focused and fuzzy, pass with no test changed but `tests/fixtures.rs`'s call sites. No assertion and no time moves.
- Production falls by about 30 lines: report the count.

## Inputs

- `skein/docs/design/forge-connection.md`, 8.1.
- `skein/docs/design/forge-connection.status.md`, the 8.1 entries.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, second half, findings 3 and 6.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the files in `files`, the status entries being a stated departure from the implementer's text. The review writes `c.set_deadline(..)`; on the trunk the method is `Connection::until`, and its name stays.

Options, each pre-approved:

- `until` runs before anything can refuse in `start`, so moving it above the call keeps the order: check it against `eaca3ccee9`'s diff (`git show eaca3ccee9`), which is what you undo. If a holder's `c` is borrowed where the line must go, reorder only that borrow.
- `component/github.rs`'s holder passes `Some(env.now.saturating_add(env.limits.deadline))`: the line above its `start` says the same.
- **The `connect` arm.** The link answers `Start::Closing` whenever it is ending (`skein-link/src/link.rs`, `connected`), and the review reads `Closed` as always ending. Shape it as you find clearest, listing `State`'s variants, with no tuple scrutinee. If `Closed` + `Start::Handshaking` is unreachable by the same reading, it goes the same way. **If any test reaches the `unreachable!`, the finding is wrong:** restore that arm and say so in the report, no question needed.

`seam-shapes` follows this task in this crate: leave `mod own`, the `cfg_attr` blocks, `lib.rs` and `component/seam.rs` alone.

## Steps

In `forge-connection.status.md`, the entry that begins `8.1: which deadline runs` becomes:

> 8.1: which deadline runs is the `until` a state holds, which
> `Connection::deadline` reads, where the section has it a function of the
> phase set in one place: a holder sets it with `Connection::until` above
> `exchange.start` as it takes a connection, and six more `until` calls in
> `component.rs` and `component/` set it, five as one leaves and one, in
> `start_heard`, as a forge's start goes on to its settings read on a
> connection still held, because a connection knows no holder's deadline, and
> the request of 2026-11-05 decided how the deadline is read, not who sets it.

Delete the entry that begins `8.1: the debt-25 request of 2026-11-06 has its spot review's net lines fall`: a plan's arithmetic is no gap in the code, and this undo makes its numbers false. Name the deletion in the report. Put `2026-11-06` on the date line if it isn't.

Tests: `heavy cargo nextest run -p skein-forge-connection --lib`, `heavy cargo nextest run -p skein-forge-machine-world`, and `heavy cargo nextest run -p skein-forge-machine-world --profile fuzzy`: a reached `unreachable!` would show there. Then fmt, `python3 -I -B scripts/old_forms.py` from `skein/`, and `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`. `Exchange` isn't reachable from outside the crate, so no other package's check is needed. Commit, push.

## Report

As your text says: production lines before and after, what you found on `connect`, and the status entry you deleted.
