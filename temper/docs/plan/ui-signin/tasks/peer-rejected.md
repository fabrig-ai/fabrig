+++
id = "peer-rejected"
part = "peer-rejected"
kind = "interface"
needs = []
files = [
  "skein/crates/skein-fake-peers/src/oauth.rs",
  "skein/crates/skein-fake-peers/README.md",
  "skein/tests/fake-peers/tests/suite/oauth.rs",
]
effort = "medium"
minutes = 20
+++

## Goal

`skein_fake_peers::oauth::Peer` answers `io::Request::Reject` at its `connections` cap and says nothing. temper's fake issuer runs it `without_records`, so no one can tell (`temper/docs/plan/ui-signin/inputs/request.md`, step 3). Give the peer a count of what it turned away, so that `issuer-connections` can write a line for each.

Acceptance:

- `oauth::Peer::rejected(&self) -> u64` is the count of connections turned away at the cap since the peer started.
- A connection refused because the peer is stopping is not counted.
- The count works under `without_records`.
- The test below passes and bites; `skein-fake-peers-tests` passes whole.

## Inputs

- `skein/docs/design/oauth.md`, 5: the fake issuer, for its words. This task changes no design.
- `temper/docs/plan/ui-signin/notes/common.md`.
- `skein/crates/skein-fake-peers/src/oauth.rs`, `Peer` and its `io::Event::Accepted` arm.

## Owns and may change

Owns: `skein/crates/skein-fake-peers/src/oauth.rs`, and `skein/tests/fake-peers/tests/suite/oauth.rs` for the test below.

May change: `skein/crates/skein-fake-peers/README.md`, one sentence beside the issuer's paragraph that names `rejected`.

The `llm` and `forge` peers have the same `Reject` arm. Leave them: nothing reads their count yet.

## Steps

1. In `Peer`, add a field `rejected: u64`, starting at 0.

   ```rust
   /// Connections turned away at the `connections` cap since the peer started;
   /// a connection refused while stopping is not one. It holds without records.
   #[must_use]
   pub fn rejected(&self) -> u64 {
       self.rejected
   }
   ```

2. Split the `io::Event::Accepted` arm's condition, so that only the cap counts. Keep its order and its one `Reject`:

   ```rust
   let full = self.connections.len() == usize::try_from(self.face.limits.connections).expect("u32 fits");
   if self.face.stopping || full {
       if !self.face.stopping {
           self.rejected = self.rejected.checked_add(1).expect("finite world connections");
       }
       self.face.requests.push(io::Request::Reject { socket });
       return;
   }
   ```

3. In `skein/tests/fake-peers/tests/suite/oauth.rs`, add a test beside `a_connection_past_the_peers_limit_is_rejected_and_the_peer_settles`. `make` gives `connections: 2`; `world()`, `pass` and `support::call` are there:

   ```rust
   #[test]
   fn a_connection_past_the_cap_is_counted_and_admission_goes_on() {
       // Two connections held: nothing counted.
       // A third is turned away: rejected() is 1, and it reads the peer's close.
       // Close one held connection; a fourth is admitted: rejected() stays 1.
   }
   ```

   Build it `.without_records()` if that is one line on the helper; otherwise a second assertion on a peer `without_records` is enough. Assert `peer.rejected() == 0` at the end of `a_connection_past_the_peers_limit_is_rejected_and_the_peer_settles`: it refuses while stopping.

4. Bite: remove the increment, run the new test, see it fail on the count, and restore.

5. Narrow test, then the checks:

   - `heavy cargo nextest run -p skein-fake-peers-tests --test suite -E 'test(/a_connection_past_the/)'`, and check its count. Confirm the binary's name from `skein/tests/fake-peers/Cargo.toml` first.
   - `cargo fmt`, `python3 -I -B scripts/old_forms.py` from `skein/`, `heavy cargo clippy -p skein-fake-peers -p skein-fake-peers-tests --all-targets -- -D warnings`.
   - `heavy cargo nextest run -p skein-fake-peers -p skein-fake-peers-tests`.
   - `heavy cargo check -p temper-fake --tests`: the one cross-tree reader that matters to the next task.

   Commit and push.

## Report

As the implementer's text asks, plus the rule you broke for the bite and the failure you saw.
