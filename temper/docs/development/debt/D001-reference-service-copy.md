# D001: temper-service's adapter overlap exceeds the debt measure's bound

- **Kind:** duplication
- **Where:** `temper-service/src/{accounts,program_sources,store,worker_route,clients,workers}.rs`, `temper-service/src/clients/{bundle,translation}.rs`; counterparts under `jig/testing/jig-test-service/src/`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig review of that day (`jig/docs/development/debt/reviews/2026-10-31-jig.md`), findings A4 F3, A11 F6 and A11 F7
- **Seen:** 2026-11-03 at `41d63da8c3`, in jig's backfill (jig's D005)
- **Checked:** 2026-11-04 at `85c7e36cd2`
- **Value:** low (356 shared distinct lines, largely braces and adapter calls; the hosts' invariants already share one implementation)
- **Size:** M
- **Status:** open
- **Paid when:** the measure falls below 200 while the services keep their application-owned passes, routing, tokens and settings.
- **Measure:** `for f in accounts program_sources server store worker_route clients workers; do comm -12 <(sed 's/^ *//' temper/crates/temper-service/src/$f.rs | sort -u) <(sed 's/^ *//' jig/testing/jig-test-service/src/$f.rs | sort -u) | wc -l; done | awk '{s+=$1} END {print s}'` (bash): 355

## What

Both services compose all seven jig protocol parts (`jig/docs/design/protocol/service.md`,
11): accounts, program sources, worker route, store, workers, server and clients.
The composition half of this item's condition holds; its numerical bound does not.
The measure is 355: `accounts.rs` 25, `program_sources.rs` 6, `server.rs` 0,
`store.rs` 80, `worker_route.rs` 17, `clients.rs` 169, `workers.rs` 58.
The absent temper `server.rs` produces sed's missing-file message and counts 0;
`lib.rs` re-exports jig's server. The command counts distinct shared lines,
including blank lines, braces, imports and tests, not duplicated host logic.

The pass in each `clients.rs` belongs to its application (11.6): calls, families,
startup and store timing differ. The adapters' `publish` and `complete` tag
submissions and completions with their owner's tokens; `limits` constructs
application sizes and `sizing_tests` checks that composition (11.1–11.5).
These seams stay with the application. The program environment's `Debug` is
hidden in the shared `jig_protocol_worker_agents::sources::Resources`.

Outside the measure, `temper/crates/temper-service/src/clients/bundle.rs` and
`jig/testing/jig-test-service/src/server/bundle.rs` share file kinds, digest-path
lookup and reply chunking through `jig_protocol_engine_clients_http::bundle`;
the bundles, routes, page fields and response policies remain the owner's
(11.4). Each service's
`clients/translation.rs` maps its own root vocabulary and stays there (11.6).

## Why it matters

The bound remains unmet, but 355 lines of overlap are not 355 lines of copied
protocol machinery. Bundle helper fixes reach both services through their
shared implementation; application passes and root translations are expected
to differ.

## What pays it back

The remaining overlap was sorted by hand at debt-10's cut, from 356 lines:
about 127 generic (braces, imports, signatures), about 176 owned by the
application under `jig/docs/design/protocol/service.md`, 11 or by this item
(passes, tokens, sizes), and about 53 movable. Every move consistent with
`jig/docs/design/protocol/service.md`, section 11 would leave about 310;
below 200 needs a clients' part holding the pass, which
11.6 rules out. The bundle helpers are shared, outside the measure. The
question of the bound is the user's; application-owned code stays with its
owner rather than moving to chase it. Bundle and translation files remain
outside that measure.
