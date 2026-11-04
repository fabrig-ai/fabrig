+++
id = "temper-server-part"
part = "service-clients"
kind = "migration"
needs = ["jig-server-part", "temper-workers-host"]
files = ["temper/crates/temper-service/src/server.rs", "temper/crates/temper-service/src/lib.rs", "temper/crates/temper-service/src/limits.rs", "temper/crates/temper-service/src/clients/policy.rs", "temper/crates/temper-service/src/tests/server.rs"]
effort = "medium"
minutes = 45
+++

# temper: compose jig's server

## Goal

`T/server.rs` (`T` and `J` are the paths `notes/service-parts.md` gives) goes. `temper-service` composes
`jig_protocol_engine_clients_http::server`, which `jig-server-part` shipped, under the path `crate::server`, so nothing
that reads it changes. temper's `clients/policy.rs` keeps the playground's fields and the cache values, and drops what
jig now ships.

Acceptance:

- `T/server.rs` is gone. `crate::server::{Server, Routed, Limits, valid}` still resolve.
- `temper-service`'s tests pass, with their assertions unchanged; the one test that lived in `T/server.rs` lives in
  `T/tests/server.rs`.
- temper's behaviour holds: `peer_end` is `PeerEnd::Connection`, so a peer's end during a call ends the connection, as
  it does today. Queues change from `with_capacity(8)` to uncapped, by `notes/service-parts.md`: an assert goes, nothing else.
- **Size cap:** the branch's net lines are at most -340. Measure it, and stop and ask if it's passed.

## Inputs

- `temper/docs/plan/debt-7/notes/common.md`, `temper/docs/plan/debt-7/notes/service-parts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11 and 11.4.
- `jig/crates/jig-protocol-engine-clients-http/src/server.rs`, from `jig-server-part`.

## Owns and may change

- Owns: the deletion of `T/server.rs`; `T/clients/policy.rs`.
- May change, in the way listed:

  | File | The edit |
  | --- | --- |
  | `T/lib.rs` | `pub mod server;` becomes `pub use jig_protocol_engine_clients_http::server;` |
  | `T/limits.rs` | `peer_end: crate::server::PeerEnd::Connection` in `server`'s literal |
  | `T/tests/server.rs` | the moved test; `the_policy_keeps_a_field_the_answer_already_holds`'s `add` path |

- Checked by reading, needing no edit: `T/clients.rs` (`crate::server::Server`, `Routed`, `valid`; 7 sites), `T/clients/google.rs`
  (`hold`, `release`, now `pub`), `T/tests/fixtures/server.rs`, `T/tests/server.rs` but for the above, `T/clients/bundle.rs`
  (`policy::FILES_CACHE`, `PAGE_CACHE`). Nothing outside `temper-service` reads `temper_service::server`.
- Not yours: anything in jig. If the shipped server can't meet something temper needs, it's a `Question:`.

## What differs, and which holds

The shipped server is temper's form. The two lines where it isn't:

- **A peer's end during a call:** temper passes `PeerEnd::Connection`, which is what `T/server.rs` does unconditionally.
- **The queues:** uncapped. Nothing but the assert in `Queue::push` can see it.

`Routed`'s and `Server`'s `Debug` show counts now. No temper test reads either.

## Steps

1. `T/lib.rs`, `T/limits.rs`.
2. `T/clients/policy.rs`: `add` goes, and `FIELDS` becomes the shipped one, with `fields` built on it:

   ```rust
   pub(crate) use jig_protocol_engine_clients_http::server::policy::FIELDS;

   pub const fn fields(page: Page) -> [(&'static [u8], &'static [u8]); 5] {
       let [(name, _), nosniff, opener, resource, referrer] = FIELDS;
       match page {
           Page::Service => FIELDS,
           Page::Playground => [(name, PLAYGROUND_POLICY), nosniff, opener, resource, referrer],
       }
   }
   ```

   `PLAYGROUND_POLICY` is the `Page::Playground` arm's bytes, as they stand. `the_playground_policy_adds_only_its_worker_source`
   pins that it stays the service's policy plus `worker-src`.
3. `T/tests/server.rs`: `crate::clients::policy::add` becomes `jig_protocol_engine_clients_http::server::policy::add`.
   Move `listener_limits_accept_stock_and_refuse_zero_connections_or_stall` here from `T/server.rs`; its assertions don't
   change, `valid` is `crate::server::valid`, `crate::tests::fixtures` is `fixtures`.
4. Delete `T/server.rs`.
5. Checks, one at a time: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-service`; `heavy cargo check -p temper --tests`; the
   citations check. Read `temper-service`'s direct dependents by `heavy cargo check --tests -p temper-client-world -p
   temper-engine-domain-world -p temper-one-process-world -p temper-system-world -p temper-playground -p temper-fake`.
6. **The browser tier.** temper's browser tier runs this server and never runs in the gate. Run `TIERS_WRAPPER=heavy
   tools/tiers/run temper browser`, the pair `browser-temper`, once, after step 5, and report its counts. It's a check you
   run and report: red is yours to read and fix, as any test of yours; if the tier can't start in your slot, say so in the
   report, and it doesn't stop your acceptance.
7. D001's measure, per file, as `notes/service-parts.md` says. `T/server.rs` is gone, so `sed` says so on stderr and
   `server` reads 0.
8. Commit and push.

## Report

The common note's items, the measure by file, the browser tier's counts, and net lines against the cap. No test is added;
the one that moves needs no break.

## Answers

2026-11-04. Does the -340 cap read against the base with the needs merged (-397), or against the integration base the
run started from (-186)? **Against the base with the needs merged: -397 meets the cap, nothing to cut.** The cap was
set for this task's own lines; the +211 between the two figures is `jig-server-part`'s and `temper-workers-host`'s
work, counted against their caps. `notes/common.md` now says so. The figure is `git diff --shortstat 85e80b7286
fcfee88275`: 42 insertions, 439 deletions, and `fcfee88275` is the only commit of the branch that isn't a merge.

Keep the pushed work at `fcfee88275` and finish what the stop left, on this branch: confirm `git status` shows
`debt-7/temper-server-part` at that commit first, since the last run's slot was switched under it, and leave out
anything in `jig/crates/jig-domain-tasks/src/control.rs`, which isn't this task's. Then `cargo fmt` and the old-forms
check on this branch, which last ran on the switched checkout; step 5's clippy, which also checks the `expect` on
`Page::Service`; the full `temper-service` nextest; `temper`'s and the direct dependents' checks; step 6's browser
tier; and the citations check.

### 2026-11-04: Run the remaining checks on the supplied merge HEAD 9fdba1c656 (recommended), or have fab restore the slot to fcfee88275 first? The task's answer requires confirming HEAD at fcfee88275 before continuing, but this slot starts at a later merge. (debt-7/temper-server-part-2, foreman debt-7/temper-server-part/foreman-2)

Have fab restore the slot to fcfee88275 before continuing the remaining checks.
The task’s Answers section requires: “confirm `git status` shows `debt-7/temper-server-part` at that commit first”.
The reported merge HEAD 9fdba1c656 does not meet that explicit precondition, even though the five task paths are unchanged.
The next run confirms the branch and commit, preserves the pushed implementation, and completes the checks listed in Answers.
Leave `jig/crates/jig-domain-tasks/src/control.rs` outside this task.

### 2026-11-04: Have fab restore this slot to fcfee88275 before resuming (recommended, as already answered), or explicitly amend the required HEAD precondition to permit 175cd8b355? (debt-7/temper-server-part-3, foreman debt-7/temper-server-part/foreman-3)

Have fab restore the slot to fcfee88275 before continuing the remaining checks.
The task’s Answers explicitly say: “Have fab restore the slot to fcfee88275 before continuing the remaining checks.”
They require: “confirm `git status` shows `debt-7/temper-server-part` at that commit first”.
The reported HEAD 175cd8b355 does not meet that precondition.
The next run confirms the branch and commit, preserves the pushed implementation, and completes the remaining checks and measurements.
Leave `jig/crates/jig-domain-tasks/src/control.rs` outside this task.

### 2026-11-04: Have fab restore slot 4 to fcfee88275 before resuming (recommended, as already answered), or explicitly amend the required HEAD precondition to permit d138fc2c7d? (debt-7/temper-server-part-4, foreman debt-7/temper-server-part/foreman-4)

Have fab restore the slot to fcfee88275 before continuing the remaining checks.
The task’s Answers explicitly say: “Have fab restore the slot to fcfee88275 before continuing the remaining checks.”
They require: “confirm `git status` shows `debt-7/temper-server-part` at that commit first”.
The reported HEAD d138fc2c7d does not meet that precondition.
The next run confirms the branch and commit, preserves the pushed implementation, and completes the remaining checks and measurements.
Leave `jig/crates/jig-domain-tasks/src/control.rs` outside this task.

2026-11-04 (agents/loop, for fabrig/debt). The earlier answer's exact-HEAD precondition can't hold: fab merges the trunk into the branch at each run's start, so HEAD is never fcfee88275 itself. **Drop it: accept a merge whose first parent is fcfee88275, and carry on with the remaining checks.** The size figure stands: -397 against the cap of -340, from `git diff --shortstat 85e80b7286 fcfee88275`.
