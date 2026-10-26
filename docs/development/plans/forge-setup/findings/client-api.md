# Findings: client-api's remaining temper work, and the forge-setup hotspots
Read at temper `main` 0ff45022e (rulings to 152); ledger of 2026-10-16T16:53Z (43 of 140 steps landed, temper 5 of 27).
ETAs are UTC, from ~1.1 h per sequential step (observed: jig 10, jig 11, temper 01 at 1.0 to 1.8 h); +-20% moves the end from 10-18 22Z to 10-19 14Z.

## 1. Steps, their files, their ETA
| Temper step | Files / crates (only those that matter to forge-setup, plus the pin) |
|---|---|
| 01.1, 01.2, 02.1 landed | Cargo.toml/.lock; crates/temper (Cargo.toml, open.rs, tests/{configuration,support/*}.rs); temper-service (clients.rs, family*.rs, route.rs, worker_route.rs, Cargo.toml); tests/one-process |
| 01.3, 03.1 landed | crates/temper/src/open.rs only; 03.1 also Cargo.lock: skein to 8a10437f (skein-01.s2) |
| 03.2 running since 16:41Z, ~10-16 18Z | terminal crate; crates/temper open.rs, tests/support/person.rs; one-process world.rs, referee/client.rs; Cargo.lock (jig 10.3) |
| 03.3 ~10-16 19Z | client-domain, terminal; temper-service clients.rs, family.rs, family/refusals.rs, tests/clients*; one-process referee/*; tests/codecs; Cargo.lock (jig 11.5). Probably STORE_VERSION 1 to 2 (temper-protocol-store, goldens): 11.5 follows jig 02.3 |
| 03.4 ~10-17 02Z | client-domain/view/terminal; manifests of crates/temper, tests/one-process, tests/client (path rewrite); Cargo.lock (jig 13.2) |
| 03.5 ~10-17 08Z | client-protocol-wire, client-domain/view/terminal; temper-service family.rs:22-28, clients.rs:84-90; one-process world.rs, referee/client.rs; crates/temper person.rs; Cargo.lock (jig 21.2) |
| 03.6 ~10-17 10Z | client-domain/view; tests/client; one-process world.rs; crates/temper person.rs; Cargo.lock (jig 21.4) |
| 04.1 ~10-17 11Z (waits for 03.6; in "check") | temper-service tests/fixtures.rs (ENGINE), limits.rs:264-278, startup/problems.rs, tests/startup.rs; engine-domain limits.rs; crates/temper tests/support/forgejo.rs (:135-168); store bump if 03.3 did not; no repin (jig 09.4 held) |
| 04.2 ~10-17 20Z (jig 22) | temper-service family.rs, family/refusals.rs, clients.rs, tests; tests/codecs; one-process referee/client.rs, cache.rs, world.rs; Cargo.lock; census repair of jig 08.4 at the latest here (engine-domain store.rs, restart.rs; temper-service limits.rs) |
| 04.3 ~10-18 08Z (jig 16) | engine-domain, protocol-store (Disposal), temper-service clients/translation.rs, family/refusals.rs; tests/{engine,conformance,system,client}; one-process stories.rs, world.rs; crates/temper tests/{process.rs,support/person.rs}; Cargo.lock |
| 04.4 ~10-18 09Z | tests/engine forge_routes.rs; terminal limits.rs; crates/temper/tests/configuration.rs |
| 05.1 ~10-18 13Z (jig 17) | temper-service: new server.rs, clients.rs, system.rs, engine.rs, lib.rs (Stop), limits.rs, tokens.rs, Cargo.toml, tests; crates/temper/src/host.rs; root Cargo.toml, Cargo.lock |
| 05.2 ~10-18 14Z (jig 18.1) | temper-service startup/{application,checks}.rs, startup.rs, settings.rs, tests/{fixtures,fixtures/startup,checks}.rs; crates/temper src/lib.rs:268, tests/support/configuration.rs; one-process fixtures.rs:98; Cargo.lock |
| 06.1 to 06.6 ~10-18 09Z to 16Z | client-domain/view/terminal, tests/client; crates/temper src/open.rs (06.1), tests/process.rs, support/{terminal,person}.rs; one-process stories.rs, world.rs |
| 07.1 to 07.4 ~10-18 16Z to 20Z | same; 07.4: crates/temper Cargo.toml, open.rs, person.rs; one-process stories.rs, Cargo.toml; root Cargo.toml, Cargo.lock |
| temper-repin ~10-19 04Z | Cargo.lock; repairs of at most 200 lines anywhere that breaks |

## 2. Hotspots for forge-setup (free = after the last toucher lands)
- **No client-api step touches** testing/temper-fake-forge-protocol, testing/temper-throwaway-forgejo, crates/temper-forge-forgejo, crates/temper-engine-protocol-forge, tests/fake-forge, tests/engine/forge, scripts/proof-one-process.sh (05.1 and 03.x only run it as a Proof). None depends on jig and skein's pin does not move: free now, repairs included.
- **crates/temper/src/lib.rs** (Command, `forges()`, `prepare`) and main.rs: 05.2 alone (:268). Quiet until ~10-18 14Z; adding `forge` and `box` commands first is safe, `forges()` reading `webhook_secret` and `user` files collides with 05.2. src/host.rs: 05.1. src/open.rs: 06.1, 07.4. crates/temper/Cargo.toml: 03.4, 07.4 (quiet 10-17 03Z to 10-18 16Z).
- **Configuration decode and checks** (temper-service startup/application.rs, checks.rs, startup.rs, settings.rs): untouched since 10-14; 05.2 is the one toucher (`Members` gains `clients`, the loopback check moves to `application.clients.listener`). 04.1 touches only problems.rs, tests/startup.rs, tests/fixtures.rs. Window 10-17 11Z to 10-18 13Z. The webhooks listener's uniqueness check names the clients' listener, which moves in 05.2: write it after 05.2.
- **Listener plumbing** (temper-service system.rs, engine.rs, tokens.rs, lib.rs `Stop`, limits.rs): 05.1 moves the clients' io into server.rs here (limits.rs also 04.1, census repair). A webhooks listener's io belongs after 05.1 (~10-18 13Z), copying server.rs.
- **Process tier** (crates/temper/tests/{process,configuration,live,startup}.rs, support/*): forgejo.rs only 04.1 (free 10-17 11Z); support/configuration.rs 05.2; process.rs 04.3, 06.1, 06.2 (to ~10-18 15Z); person.rs every step to 07.4 (~10-18 20Z). Every client-api Proof runs `temper_open_signs_in_prints_the_inbox_and_exits_on_q` on carbon-forge: keep it green.
- **tests/one-process:** busy in every step to 07.4 (world.rs, referee/*, stories.rs, Cargo.toml); fixtures.rs only 05.2 (:98); forge.rs and machine.rs untouched. Free after ~10-18 20Z.
- **Cargo.toml/Cargo.lock:** the lock moves at 03.2, 03.3, 03.4, 03.5, 03.6, 04.2, 04.3, 05.1, 05.2, 07.4 and the repin (jig only); root Cargo.toml at 05.1, 07.4. New crates (temper-forge-setup, -shell, temper-box) conflict only in `members` and the lock: regenerate the lock after each rebase.
- Also: crates/temper-protocol-store (store v2 at 03.3 or 04.1, 04.3), temper-engine-domain: not forge-setup's.

## 3. Order and when free
Chain (jig sessions in the long pole; steps left in brackets): jig 11.5 (1, 17:20Z) -> 12 (5) -> 13 (5; 13.2 at 10-17 01Z) -> 21 (4; 21.2 06Z, 21.4 09Z) -> 14 (5) -> 22 (4) -> 15 (6) -> 16 (5) -> 17 (5) -> 18 (3) -> 19 (5) -> 20 (5) -> temper-repin. 53 steps, no real parallel slack.
temper: 03.2 -> 03.3 (jig 11.5) -> 03.4 (jig 13.2) -> 03.5 (21.2) -> 03.6 (21.4) -> 04.1 (jig 09.4, held) -> 04.2 (jig 22) -> 04.3 (jig 16) -> 04.4 (04.3, jig-13.s1) -> 05.1 (jig 17) -> 05.2 (jig 18.1); 06 after 04, beside 05; 07 after 06; temper-repin after jig 20 and 07.
Free from: fake forge, throwaway, forgejo crate, protocol-forge, proof script: now. crates/temper src (except open.rs): now, to 05.2. startup/config: 10-17 11Z (before 05.2 starts ~10-18 13Z), uniqueness check after 05.2. Listener plumbing: after 05.1. Process-tier support and one-process: ~10-18 20Z. Lock: only after the repin (10-19 04Z, up to 10-19 14Z).

## 4. Pins
- temper pins skein 8a10437f (= skein-01.s2, landed in 03.1), jig 2900e9b3b, smith 29fa97c9. client-api moves smith never, skein never again (every repin holds 8a10437f; jig 19 moves jig's skein to the same commit), jig at each repin above.
- **Skein steps left: none** (skein 01, s1, s2 all landed). A further supplement step (like s1, s2) could still be added by ruling, in jig 19/20's browser work.
- **Hazard:** the README's Pins make each repin end with `grep -o 'git+...#sha' Cargo.lock | sort -u` printing skein's line as jig's lock or 8a10437f, and the closing repin "moves skein and smith to jig's pins". A temper skein newer than 8a10437f makes that check fail (a question) or, followed literally with `--precise`, moves it back.
- **Advice:** land SHA-256/HMAC in skein `main` now (jig's `--precise` pin is unaffected; the change is additive for jig and smith). Take it in temper's lock after temper-repin, or earlier only after the review adds a client-api ruling: "temper's skein may hold more than the floor; the check reads `merge-base --is-ancestor`; no repin moves skein back", before the first repin that follows the pin move (repins start at 03.3, ~18Z today; every later one up to 05.2 is exposed).
