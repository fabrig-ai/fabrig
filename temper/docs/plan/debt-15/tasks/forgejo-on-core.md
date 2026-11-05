+++
id = "forgejo-on-core"
part = "d011-servers"
kind = "feature"
needs = ["route-move"]
files = ["skein/crates/skein-fake-forge-protocol/src/server.rs", "skein/crates/skein-fake-forge-protocol/src/server/git.rs", "skein/crates/skein-fake-forge-protocol/Cargo.toml", "skein/crates/skein-fake-forge-http/src/connection.rs", "skein/crates/skein-fake-forge-http/src/tests/connection.rs"]
effort = "high"
minutes = 90
+++

# The Forgejo fake's server stands on the core

## Goal

`Server` in the Forgejo fake holds a `connection::Connection` and keeps what `notes/contracts.md`, 2 says a server keeps.
No byte, event, fault draw or close moves. Its tests are black-box, which makes it the first reader of the core.

Acceptance:

- `Server` has no field the contract's `Connection` holds, and no copy of `http_down`, of the head's snapshot, of the
  reply's chunking or of `room`. Its `State` is `connection::State`.
- The pass keeps its bound of 8, its queue order (http events, json events, json's requests, http requests) and its break
  on `head_pending` after every http event.
- Forgejo's production lines fall by about 80.
- The packages of step 4 pass, the same tests as at base, in the `default` profile, and the three named there in `fuzzy`.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 2
- `skein/docs/design/fake-forge.md`, 4 and 12

## Owns and may change

- **Owns:** `P/server.rs`, `P/server/git.rs`, those lines that read the moved fields.
- **May change:** `skein-fake-forge-http/src/connection.rs` and its tests: an addition this server needs (a new `pub fn`, a
  new `Turn` arm that GitHub's will share). A change to a public signature the contract fixes is a question. Forgejo's
  `Cargo.toml`, its dependency line.
- **Not to touch:** GitHub's server; `Service`; `git`.
- **What pins the order,** so read them before you start, and a red in any of them is a stop and ask:
  - `pipelined_requests_reach_the_forgejo_host_one_head_each_in_wire_order`,
    `keeping_every_kind_reproduces_the_default_frozen_trace`: the 8-step bound, the queues, the break.
  - `a_reset_closes_before_reading_and_a_timeout_answers_500`, `git_byte_faults_reach_repository_and_store_calls`: the
    draws' order. The core never draws.
  - `shutdown_keeps_a_mid_request_client_until_its_terminal`: `retire`, json's close, http's close, then `Event::Close`,
    and a captured head survives a close.
  - `a_response_cut_at_each_byte_ends_its_call_once`, `each_call_has_one_answer_under_stream_cuts` (fuzzy): cuts.
  - `unauthorized_heads_are_drained_and_redacted`: the discarded body.
  - `a_reviewer_removal_answers_204_with_no_body`, `a_merge_answers_200_with_no_body`: `Body::None` against
    `Body::Length(0)` and `Finish`.
- **Options:** `closed`'s `server.state != State::Closing` guard is redundant, as `close` returns on `Closing`: keep it
  or drop it, as the code reads best.

## Steps

1. Field by field, into `server.connection`. This is mechanical: one commit.
2. `up`, `http_down`, `Reply(Room)`, the snapshot, `room`, `respond`'s tail and `close` onto the core: one commit. `send`
   goes: if a test passes different limits to `new` and to `up`, stop and ask.
3. The pass: `connection::turn`, with `http_event`'s arms split between `Turn` and the server's.
4. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-protocol -p skein-fake-forge-http --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-forge-http -p skein-fake-forge-protocol -p skein-fake-peers-tests -p
   skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-setup-world -p
   temper-engine-protocol-forge-world -p temper-fake -p temper-fake-git -p temper-one-process-world -p
   temper-real-loop-world`; `heavy cargo nextest run -p temper-service --lib -E 'test(setup_create) | test(setup_scopes)'`;
   `heavy cargo nextest run --profile fuzzy -p skein-fake-peers-tests -p skein-forge-protocol-world -p skein-forge-machine-world`; the
   citations check. Commit and push.

## Report

The common note's items. Production lines before and after, for this crate and the core. What you added to the core.
Break: make `connection::close` keep `requests`, and name the failure.
