# D001: temper-service's adapter overlap exceeds the debt measure's bound

- **Kind:** duplication
- **Where:** `temper-service/src/{accounts,program_sources,store,worker_route,clients,workers}.rs`, `temper-service/src/clients/{bundle,translation}.rs`; counterparts under `jig/testing/jig-test-service/src/`
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig review of that day (`jig/docs/development/debt/reviews/2026-10-31-jig.md`), findings A4 F3, A11 F6 and A11 F7
- **Seen:** 2026-11-03 at `41d63da8c3`, in jig's backfill (jig's D005)
- **Checked:** 2026-11-04 at `4e32610862`
- **Value:** low (356 shared distinct lines, largely braces and adapter calls; the hosts' invariants already share one implementation)
- **Size:** M
- **Status:** open
- **Paid when:** the measure falls below 200 while the services keep their application-owned passes, routing, tokens and settings.
- **Measure:** `for f in accounts program_sources server store worker_route clients workers; do comm -12 <(sed 's/^ *//' temper/crates/temper-service/src/$f.rs | sort -u) <(sed 's/^ *//' jig/testing/jig-test-service/src/$f.rs | sort -u) | wc -l; done | awk '{s+=$1} END {print s}'` (bash): 356

## What

Both services compose all seven jig protocol parts (`jig/docs/design/protocol/service.md`,
11): accounts, program sources, worker route, store, workers, server and clients.
The composition half of this item's condition holds; its numerical bound does not.
The measure is 356: `accounts.rs` 25, `program_sources.rs` 6, `server.rs` 0,
`store.rs` 80, `worker_route.rs` 18, `clients.rs` 169, `workers.rs` 58.
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
`jig/testing/jig-test-service/src/server/bundle.rs` copy file kinds, digest-path
lookup and reply chunking. These helpers can move; the bundles, routes, page
fields and response policies remain the owner's (11.4). Each service's
`clients/translation.rs` maps its own root vocabulary and stays there (11.6).

## Why it matters

The bound remains unmet, but 356 lines of overlap are not 356 lines of copied
protocol machinery. Bundle helper fixes still need to reach both services;
application passes and root translations are expected to differ.

## What pays it back

Share the remaining reusable helpers with jig (the scope of jig's D005),
retaining application policies. Recheck the unchanged measure after any such
work; its below-200 bound remains open, without moving application-owned code
to chase it. Bundle and translation files remain outside that measure.
