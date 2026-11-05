+++
id = "github-on-core"
part = "d011-servers"
kind = "feature"
needs = ["forgejo-on-core"]
files = ["skein/crates/skein-fake-forge-github/src/server.rs", "skein/crates/skein-fake-forge-github/Cargo.toml", "skein/crates/skein-fake-forge-http/src/connection.rs", "skein/crates/skein-fake-forge-http/src/tests/connection.rs"]
effort = "high"
minutes = 90
+++

# The fake GitHub's server stands on the core

## Goal

`Server` in the GitHub fake holds a `connection::Connection`, so the two fake servers share one connection core: D011's
last clause of code. No byte, event, fault draw or close moves.

Acceptance:

- `Server` has no field the contract's `Connection` holds, and no copy of `http_down`, of the head's snapshot, of the
  reply's chunking or of `reply_room`. Its `State` is `connection::State`.
- A bodyless status passes an empty body with `bodyless: true`, so 204, 304 and a HEAD answer as at base.
- GitHub's production lines fall by about 80.
- The packages of step 4 pass, the same tests as at base, in the `default` profile, and the three named there in `fuzzy`.

## Inputs

- `temper/docs/plan/debt-15/notes/common.md`
- `temper/docs/plan/debt-15/notes/contracts.md`, 2
- `skein/docs/design/fake-forge.md`, 10.1, 10.6 and 12

## Owns and may change

- **Owns:** `G/server.rs`, those lines that read the moved fields, and its white-box tests at the end of the file, which
  reach them through `server.connection`: a test's way in, not its assertion.
- **May change:** `skein-fake-forge-http/src/connection.rs` and its tests: an addition this server needs. A change to a
  public signature the contract fixes, or to what Forgejo's server sees, is a question. GitHub's `Cargo.toml`.
- **Not to touch:** Forgejo's server; `Service`; `Inline`, beyond a field's path.
- **What pins the order,** so read them before you start, and a red in any of them is a stop and ask:
  - `github_fake_byte_faults_reset_drop_and_server_error`: the draws, which here follow the last continuation in `down`.
  - `github_peer_shutdown_settles_a_mid_request_client`: a close, and a captured head surviving it.
  - `github_peer_head_is_a_bodyless_404_and_allows_reuse`: `Body::Length(0)` and `Finish` against `Body::None`.
  - `github_http_intake_refuses_one_byte_past_its_bound`, `a_private_git_credential_refusal_precedes_the_body_demand`:
    the discarded body. Both are in `G/server.rs`.
  - `github_conformance_transcripts_put_every_status_and_headers_on_their_own_lines`,
    `skein/tests/forge/setup/tests/suite/github_convert.rs`: heads as written.
  - `github_calls_costs_and_credentials_survive_cuts_renewal_and_replacement` (fuzzy): cuts.
- **Options:** `Failed` of a body closes here, where Forgejo's goes to the tokenizer: `Turn::Body(Up::Failed(_))` is this
  server's to close.

## Steps

1. Field by field, into `server.connection`: one commit.
2. `up`, `http_down`, `Reply(Room)`, the snapshot, `reply_room`, `respond`'s tail and `close` onto the core: one commit.
3. The pass: `connection::turn`, with `event_up`'s arms split between `Turn` and the server's.
4. `cargo fmt`; `heavy cargo clippy -p skein-fake-forge-github -p skein-fake-forge-http --all-targets -- -D warnings`;
   `heavy cargo nextest run -p skein-fake-forge-http -p skein-fake-forge-github -p skein-fake-peers-tests -p
   github-conformance -p skein-forge-machine-world -p skein-forge-protocol-world -p skein-forge-setup-world -p
   temper-engine-protocol-forge-world -p temper-fake -p temper-one-process-world -p temper-real-loop-world`; `heavy cargo
   check --tests -p temper-client-browser-world`; `heavy cargo nextest run --profile fuzzy -p skein-fake-peers-tests -p
   skein-forge-protocol-world -p skein-forge-machine-world`; the citations check. Commit and push.

## Report

The common note's items. Production lines before and after, for this crate and the core. What you added to the core. Break:
make `respond` send `Finish` on a bodyless reply, and name the failure.
