+++
id = "undecoded"
part = "bundle-helpers"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-host/src/boundary.rs", "jig/crates/jig-domain-host/src/lib.rs", "jig/crates/jig-protocol-worker-agents/src/route.rs", "jig/crates/jig-protocol-worker-agents/src/tests/**", "jig/testing/jig-test-worker/src/**", "jig/testing/jig-test-service/src/worker_route.rs", "temper/crates/temper-worker-domain/src/boundary.rs", "temper/crates/temper-worker-domain/src/lib.rs", "temper/crates/temper-worker-domain/src/agents.rs", "temper/crates/temper-service/src/worker_route.rs", "jig/docs/design/protocol/service.status.md"]
effort = "medium"
minutes = 30
+++

# B4: one `Undecoded`, in the hub

## Goal

What of a start did not decode exists three times: `route::Invalid`, `temper_worker_domain::Invalid` and
`jig_test_worker::Invalid`, mapped by hand, jig's with an `unreachable!`. One enum in `jig-domain-host`, which links
no smith. Behaviour stays unchanged.

Acceptance:

- `jig_domain_host::Undecoded` exists as `notes/contracts.md` gives it; `route::decode` and `route::evidence` return it.
- The three `Invalid`s are gone, with their re-exports, and neither `worker_route.rs` maps one onto another: `decode`
  hands `route::decode`'s `Err` on. The `unreachable!("text evidence is never decoded")` in jig's goes.
- A search for `route::Invalid` in `*.rs` and in `jig/docs/design/` finds nothing. `host::Invalid`, the hub's refusal,
  and `RunInvalid`, smith's, are other things, and stay.
- The packages below pass with no test's assertion changed; a test changes only in the name it writes.
- `jig/docs/design/protocol/service.status.md` loses its **Not yet** line for 11.2 and its date is today's.
- **Size cap:** net at most -15 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `temper/docs/plan/debt-10/notes/contracts.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/protocol/service.md`, 11.2, on the trunk.
- `jig/AGENTS.md`: the hub links no smith. `Undecoded` names nothing of smith's.

## Owns and may change

- Owns: `jig-domain-host`'s new enum, beside the hub's boundary types, and its export; `jig-protocol-worker-agents`'s
  `route.rs`.
- May change, each only for the name: `jig/testing/jig-test-worker/src/{boundary,lib,tests}.rs`,
  `jig/testing/jig-test-service/src/worker_route.rs`, `temper/crates/temper-worker-domain/src/{boundary,lib,agents}.rs` and
  its tests, `temper/crates/temper-service/src/worker_route.rs`. This branch crosses into temper because the type
  temper reads goes: every temper line it changes names the type.
- jig's worker has no arm for `Evidence` today. With one enum it holds a value it never gets. If a match there needs
  an arm, the arm treats it as `Charter` is treated, and your report says so; no `unreachable!`.

## Steps

1. Add the enum and export it. In `route.rs`, delete `Invalid`, and return `hub::Undecoded`.
2. Readers. `heavy cargo check -p jig-protocol-worker-agents -p jig-test-worker -p jig-test-service -p
   temper-worker-domain -p temper-service --tests` while errors last.
3. `heavy cargo nextest run -p jig-domain-host -p jig-protocol-worker-agents -p jig-test-worker -p temper-worker-domain`,
   and `-p jig-test-service`, `-p temper-service --lib`, narrowed by `-E` to `worker_route` and `decod` where you can.
4. Update the status file. fmt, clippy, old forms, citations. Commit, push.

## Report

As the implementer's text says. No test here pins a new rule, so `notes/common.md`'s table is empty; give the net
lines.

## Rebase (2026-11-04, step merge, foreman debt-10/undecoded/foreman-1)

resolve the service status conflict by preserving trunk’s gaps and removing only the completed 11.2 gap.
The merge failed in `jig/docs/design/protocol/service.status.md`.
Trunk commit `fcf65bd271` reconciled main’s status changes; task commit `ec8a949bf2` removes the adjacent 11.2 entry.
Rebase onto the current landing trunk and resolve this file as follows:
- Remove the **Not yet:** 11.2 line mentioning `route::Invalid`.
- Preserve trunk’s revised 11.4 entry describing temper’s remaining bundle duplication.
- Preserve trunk’s section 11 entry recording 356 shared distinct lines against the bound of 200.
- Preserve the other remaining gaps and set the date to the rebase day.
Keep the task’s `Undecoded` implementation unchanged for this conflict.

## Rebase (2026-11-04, step merge, foreman debt-10/undecoded/foreman-2)

resolve the status conflict by retaining both completed-gap deletions.
The merge conflicts only in `jig/docs/design/protocol/service.status.md`.
Rebase onto the current landing trunk.
Task commit `513f22c271` removes the 11.2 `route::Invalid` gap.
Trunk commit `83e7de431b` removes the adjacent 11.4 bundle gap because D005 is complete.
Resolve by removing both entries; this supersedes the earlier Rebase instruction to preserve 11.4.
Preserve the remaining gaps, including 11/11.5 and the section 11 measurement of 356 shared distinct lines against 200.
Set the status date to the rebase day.
Keep the task’s `Undecoded` implementation unchanged.
