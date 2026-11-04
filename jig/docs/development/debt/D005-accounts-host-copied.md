# D005: Service adapters and bundle helpers still repeat shared mechanics

- **Kind:** duplication
- **Where:** `testing/jig-test-service/src/{accounts,program_sources,store,worker_route,clients,workers}.rs`, `testing/jig-test-service/src/server/bundle.rs`, `testing/jig-test-service/src/clients/translation.rs`; counterparts under `temper/crates/temper-service/src/`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T4, A4 F3, A11 F6, A11 F7)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 11: temper's accounts adapter differs from the test service's copy)
- **Seen:** 2026-11-03 at `41d63da8c3`, in temper's backfill (`temper/docs/development/debt/D001-reference-service-copy.md`)
- **Checked:** 2026-11-04 at `4e32610862`
- **Value:** low (the 356 measured lines are mostly adapter calls and braces; sharing bundle helpers would reduce repeated review)
- **Size:** M
- **Status:** open
- **Paid when:** the common bundle helpers (file kinds, digest-path lookup and reply chunking) are shared product code both services compose, with their routing and page policies kept by their owners.

## What

`J` is `jig/testing/jig-test-service/src`, `T` is `temper/crates/temper-service/src`.
Both compose all seven parts: `jig_protocol_accounts::host`,
`jig_protocol_worker_agents::{sources, route}`, `jig_protocol_store::host`,
`jig_protocol_engine_workers::host`, `jig_protocol_engine_clients_http::server`
and `jig_protocol_engine_clients::host`. The program environment's `Debug`
is hidden in the shared `sources::Resources`.

D001's measure is 356 shared distinct lines, including braces, imports and tests.
The pass in each `clients.rs` stays the application's (service.md, 11.6).
The adapters' `publish`, `complete`, `limits` and `sizing_tests` repeat token
handoffs, size construction and composition checks, not copies of the hosts.
Tokens, routes, sizes and their checks belong to the owner (11.1–11.5).

`T/clients/bundle.rs` and `J/server/bundle.rs` copy file kinds, digest-path
lookup and chunking; those helpers can move without moving the owner's bundle,
routes, page fields or response policy (`jig/docs/design/protocol/service.md`, 11.4).
Each `clients/translation.rs` maps its root's vocabulary; those mappings stay
with the application (11.6). Neither the bundles nor these translations is measured.

## Why it matters

The hosts share jig's invariants. Remaining helper copies still require fixes
in two places, but the measure largely counts application composition.

## What pays it back

Factor the common bundle helpers into product code, keeping the services'
different revalidation, missing-bundle and call-ending policies. Keep the owner
passes and translations with their owners (`jig/docs/design/protocol/service.md`, 11).
