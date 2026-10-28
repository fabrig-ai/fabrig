# A hot restart of the engine: the investigation

Report, 2026-10-20. An investigation, read-only, of how feasible it is
for `temper serve` to rebuild its domain from a new configuration
document in the same process, its listeners and connections kept. Run on
Codex for the first-run design session (`first-run.md`, section 7),
against the primary checkouts' `main` named below; its citations are
`repository/path:line` at those commits. A record, not contract.

**Feasible, but there is no ready-made domain-swap operation.** My estimate is **medium** for a restricted restart at a safe point, **large** for transparent restart during active execution with arbitrary configuration changes.

I inspected the clean local `main` checkouts: temper `20d88b6bb`, jig `f758ea6f`, smith `a89126e1`, skein `9ab88f69`. No edits, branches, worktrees, or builds. Graph findings were checked with `rg` and source reads; several index entries lagged the source.

One material distinction: **temper’s docs are ahead of its implementation here.** They describe forge credentials loaded from the secrets store; this checkout still reads configured token files and accepts only Forgejo. The assessment below follows the code. Compare `temper/docs/design/protocol/configuration.md:677` with `temper/crates/temper-service/src/startup/application.rs:264` and `temper/crates/temper-service/src/startup/application.rs:630`.

**1. A start today**

- `serve` reads the bounded configuration file; `prepare` checks it, reads referenced files and the browser bundle, reads smith’s agent configuration, resolves endpoints/trust, prepares directory descriptors, and draws seeds. It then opens termination signals and the kernel and enters `skein_shell::drive`. `temper/crates/temper/src/lib.rs:319`, `temper/crates/temper/src/lib.rs:397`
- `startup::parse` parses jig’s engine/workers and temper’s application settings; checks standalone topology, credentials, listeners and limits. `startup` additionally checks files, channel/agent compatibility, root configuration and combined memory bounds, then constructs the worker and service. `temper/crates/temper-service/src/startup.rs:106`, `temper/crates/temper-service/src/startup.rs:202`
- `Service::prepared` builds the store, accounts driver/value tables, clients/server, worker-channel component, forge connection component, root configuration, shared `Io`, queues and operation accounting. `Engine::new` constructs `temper_engine_domain::Domain`: forge domain, journal and jig core. The core constructs tasks, authority, people, fleet, accounts, briefs, notes and views. `temper/crates/temper-service/src/system.rs:56`, `temper/crates/temper-engine-domain/src/domain.rs:155`, `jig/crates/jig-domain-core/src/lib.rs:639`
- `Worker::new` separately owns the worker domain, git/checkouts, engine link, agent settings, process/channel maps and credential tables. Actual LLM execution belongs to spawned smith agent services; rebuilding the engine alone would not rebuild those agents. `temper/crates/temper-service/src/worker.rs:30`, `smith/crates/smith-agent-shell/src/agent_shell.rs:230`

Recovery has two levels:

- Storage opens the deployment’s main, secrets and transcript machinery. Skein KV recovers/replays persisted data; jig checks format, deployment header and commit-number agreement. Temper’s accounts component loads saved source records before running its driver. `temper/crates/temper-service/src/store.rs:85`, `skein/crates/skein-kv/src/store.rs:279`, `jig/crates/jig-protocol-store/src/open.rs:7`, `temper/crates/temper-service/src/accounts.rs:442`
- `engine::iterate` announces credentials and sends `Event::Restart`. Jig orders **load core → rebuild inbox facts → restore connector → adopt runs → read external facts afresh → settle outbox → open decisions**. Temper pages and routes those records; the deployment header restores the journal’s durable commit number. Client admission remains “starting” meanwhile. `temper/crates/temper-service/src/engine.rs:169`, `jig/crates/jig-domain-core/src/restart.rs:159`, `temper/crates/temper-engine-domain/src/restart.rs:291`, `temper/crates/temper-engine-domain/src/domain.rs:418`

A restart does **not** overwrite everything with JSON: persisted projects/policies survive; configuration rules, charters, templates, limits and hosts apply according to startup semantics. `jig/docs/design/protocol/configuration.md:361`

**2. Resource disposition**

These are **proposed rules**, based on the ownership above—not existing reload capabilities. “Unchanged” must include referenced file contents and effective limits, not merely filenames.

| Resource | Rule across hot restart |
|---|---|
| Engine domain, core children, forge domain, journal | **Rebuild**, then recover from a settled durable boundary. Preserve deployment identity and commit numbering. |
| Store and its file drivers | **Keep** for the same deployment and compatible codecs/bounds. Drain old writes/reads and reset domain request correlations. Reject deployment-directory changes initially. |
| Kernel, shared `Io`, signals, completion/submission queues | **Keep**. They own outstanding operations and live descriptors. Reject changes exceeding the existing operation/memory envelope initially. |
| Client, worker and webhook listeners | **Keep** when address, transport and relevant limits are unchanged. Changed/removed listeners require controlled retirement/replacement; existing accepted connections need separate ownership. |
| Client HTTP/SSE sessions, authentication exchanges | **Keep** unchanged transport/security components. Drain pending request/secret-write correlations before swapping their domain counterpart. |
| Domain watchers | **Rebuild and rebind**. Preserve/replay full subscription descriptions and reconcile deliveries; retaining sockets alone is insufficient. |
| Worker links, worker domain, git operations, agent processes | **Keep together** when worker/agent configuration is compatible. Changed roots, programs, credentials or supervision bounds require draining/replacing affected workers or rejecting the reload. |
| Forge connections | Potentially **keep when unchanged and settled**; otherwise close/rebuild affected connections. Fence old replies before restored outbox recovery can issue effects. |
| LLM/account endpoints and credential caches | Keep unchanged drivers only with explicit resynchronization of grants, generations and expiry into the new domain. Drain/rebuild changed sources. Existing agents retain their own endpoint configuration. |
| Timers | **Rebuild domain timers** from persisted wall times; retain transport, heartbeat and worker-supervision timers with their owners. Do not restart the process clock. |
| Bundle, charters, trust/secret files | Reread and validate; replace immutable values where supported, otherwise rebuild the owning component. |

The important coupling is visible in client exchange/stream maps, worker channel state and account queues. `jig/crates/jig-protocol-engine-clients/src/component.rs:29`, `jig/crates/jig-protocol-engine-workers/src/connection.rs:20`, `temper/crates/temper-service/src/accounts.rs:44`. Persisted deadlines use wall time and are translated to monotonic deadlines. `jig/docs/design/domain/engine.md:346`

**3. In-flight work**

**“No tasks exist” is unnecessary.** Cold recovery already restores live tasks, attempts, committed calls and connector outboxes. It reconciles uncertain effects and resumes from durable progress; this does not promise preservation of every partially executing operation. `jig/docs/design/domain/engine.md:395`

The major hot-restart obstacle is **surviving workers**: `Core::adopt` deliberately treats in-process workers as engine-hosted; fleet restoration immediately reports their attempts lost/not-started. Keeping the worker alive while running that logic unchanged is incorrect. Hot recovery needs a distinct survival rule and a fresh inventory/hello delivered into the new fleet. `jig/crates/jig-domain-core/src/sibling_routes.rs:163`, `jig/crates/jig-domain-fleet/src/domain.rs:339`

Memory-only state requiring preservation, draining or explicit retry includes:

- Journal decisions, outstanding commits and held outputs; root load, effect, assignment and host-call correlations. `skein/crates/skein-lib/src/journal.rs:108`, `temper/crates/temper-engine-domain/src/domain.rs:31`
- Watches, pending deliveries and provisional text. Watches are explicitly not stored; the protocol’s `Watch` also lacks the full subject needed to reconstruct the domain subscription. `jig/docs/design/domain/engine.md:898`, `jig/crates/jig-protocol-engine-clients/src/stream.rs:8`
- Worker/agent calls, unacknowledged turns, queued messages, process supervision and current generation progress. Keeping the whole worker preserves these; dropping it requires cold-start loss/retry semantics. `smith/crates/smith-host-domain/src/domain.rs:88`

A useful starting predicate already exists: `Domain::quiescent()` permits sessions, assigned workers awaiting external answers and future timers, while requiring immediate handoffs and store operations to settle. **It is not sufficient alone**: protocol queues and session rebinding also matter. `temper/crates/temper-engine-domain/src/domain.rs:293`

**4. The smallest seam and its owner**

The structural seam is already present: `Service` owns shared IO and the worker; `Engine` holds the root domain separately from store/protocol children. Jig’s reference service has essentially the same split, and its shell `Host` simply delegates iteration and queues. `temper/crates/temper-service/src/system.rs:15`, `jig/testing/jig-test-service/src/engine.rs:9`, `jig/testing/jig-test/src/host.rs:18`

**Proposal:** split checked startup configuration from runtime construction; add an explicit pause/drain/recover/rebind lifecycle around the domain. Keep driving IO and protocol heartbeats, return bounded “not ready” responses during recovery, and fence old domain replies using an incarnation identifier or fully drained correlation tables.

This cannot reuse shutdown unchanged: shutdown stops clients/workers, closes components, and eventually aborts the in-process worker. Nor can it send `Restart` twice to the same core: `restart_begin` only starts from `Cold`. `temper/crates/temper-service/src/stop.rs:10`, `temper/crates/temper-service/src/system.rs:429`, `jig/crates/jig-domain-core/src/restart.rs:108`

**Jig should own** the lifecycle guarantees, surviving-host adoption, client/watch rebinding and reference implementation/tests. **Temper should own** configuration diffing, forge-specific reconciliation and its service wiring. Skein’s loop already knows nothing about domain state, so a generic shell redesign appears unnecessary. `skein/crates/skein-shell/src/drive.rs:1`

**5. Size, risks and first version**

**Estimate:** medium for a restricted version; large for active-run continuity plus arbitrary resource changes. Main risks are duplicate effects, stale replies matching new tokens, lost acknowledgements/watch deliveries, credential-generation mismatches, and failed recovery after the old domain has been discarded. Lowered limits can reject persisted live state; removal of referenced configuration is still partly an open design question. `jig/docs/design/protocol/configuration.md:378`

The simplest useful first version would:

- Accept changes to domain policy/charter/template values within fixed capacity bounds; require unchanged deployment, listeners, workers, endpoints and protocol/IO limits.
- Initially require **no active agent attempts**, but allow durable unfinished tasks. This is a scope reduction, not a fundamental recovery requirement.
- Drain outstanding operations, preserve listeners/client streams and idle worker connections, rebuild/recover the domain, reannounce workers and reopen watches internally.
- Validate before cutover; define recovery-failure behavior explicitly. Startup currently performs synchronous file reads/DNS, so reloading preparation also needs care to keep the single IO loop responsive.

Removing the active-attempt restriction should be a subsequent jig capability, tested with surviving workers, replayed turns/calls, slow watchers and late completions. **“No listener changes” alone does not make the swap safe.**