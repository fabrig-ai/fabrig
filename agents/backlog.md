# Backlog: the pipeline, its tools and its next ideas

Provisional, 2026-10-29. Kept by the coordinator and the operator: what is decided and not yet done, and the ideas to try, each with what would show it is worth it. Incidents and their remedies are in `agents/ledger.md`; this file is what comes next.

## 1. In one page
- **Throughput is the goal:** (1) a pipeline that widens, (2) short builds on warm caches, (3) no spurious blocking. An item names which it serves.
- **Data before structure:** an idea that changes the code's shape (crates, tiers) waits for the numbers that justify it; collecting them is an item of its own.
- **Done items leave this file;** what they taught goes into the ledger or a role file.

## 2. Queue and gate
- **Merge train (1):** check batch N+1 on N's merge on a second gate machine while N checks; if N passes, N+1 lands next. Worth it when the queue is busy most of the time (96.5% completed busy in the 01:08–02:10Z window on 10-29). Hetzner2 is drained and reserved; the local worker has landed, controller and authority are implemented, and transport recovery is under repair/review with production activation disabled. The reviewed frozen repair lane landed port-race and port-hold as full-gate singletons; ordinary16 resumed. Transport integration and real dedicated h8/h2 proof still gate train activation.
- **Cheap precheck invariants (3):** compile-only prechecks missed Jig vocabulary held by jig-boundary (ledger57). Jig now runs it manually. Inventory the other trees’ equivalent cheap invariant suites, measure their cost, then add an explicit touched-tree mapping to precheck beside clippy and test compilation, ordinary heavy and existing budgets/admission/restoration. Do not silently run whole suites or weaken the gate; this follows the pending corrected-precheck deployment.\n- **Process tiers after phase D (3):** temper, jig and smith process tests currently have no gate or periodic baseline; two temper GitHub stories failed on main tonight with no known first-bad revision (ledger52). Keep them outside the gate. Ops first inventories explicit process binaries and measures a pinned-main baseline, then schedules serial nightly checks on claimed builder slots through ordinary heavy, with the existing profiles/referees and fresh 80000000000-byte admission. Record revision, selection/counts, elapsed/CPU/disk and log/receipt paths; report failures and missing runs to each owning lead. Skip unavailable capacity without inventing success. Added capacity or changed budgets require the measured request through the coordinator; no timeout change is authorized here.
- **Partitioned full runs (2):** nextest `--partition` across two gate machines for the periodic full run.
- **Binaries a test runs (3):** a test that runs another package's binary passes only when something built it. Either each such test gets its own package's target (as smith's program check did), or the queue's selection adds the providing package (`tools/queue/binaries.txt`). 88 such launches, mostly browser tiers (hetzner2 `~/src/fabrig/notes/smith-hermetic-report-2.md`).
- **Starved counts (3):** count `skein-world: starved:` lines in passing tests' output per (binary, test), reported apart from passes and failures (testing-strategy 2.10).
- **CPU seconds per suite (3):** each tree's focused and fuzzy suites in CPU seconds on full runs (a nextest wrapper script); then the user sets the CPU budgets and the global timeout's backstop.
- **Checks the old gates ran:** wasm32 clippy (the browser client) first, then darwin clippy, skein's fail_stop.py, taste.sh, check-contracts, recovery-shape: the ones still worth it.

## 3. Machines and tools
- **Every tool in fabrig (2, 3):** `codex-run` first, then `heavy`, `post`, `kache`'s setup, the rest, each moved, fixed and installed by `tools/install` from `~/src/fabrig/main`.
- **Elasticity (1, 2):** capacity follows the bottleneck of the moment: gate machines when the queue is the wait, builders when tasks wait for slots, leads when slots idle for want of tasks. Needs the hourly numbers (the operator's) first.

## 4. Code shape (data first)
- **Crate granularity (2):** per crate: build time (cargo `--timings` of a full build), how often it changes (git log per crate path), rebuild fan-out (its reverse dependencies and their build cost). A crate that changes often and drags a large fan-out is a split candidate; one that never changes alone may merge.
- **Wave 2.5 effect (2):** measure the queue’s build step and targets’ size before and after the completed one-binary-per-package consolidation.
- **Tiers we dropped for the spike:** browser, process, live, qa: their scripts assume per-tree targets and profiles the root `.config` lacks; restore the ones worth it once the spike's numbers are in.

## 5. More than one designer
- **Operator effort (1, 3):** measure the Codex operator’s response times and missed incidents at high before deciding whether to try medium.
- **Crate leases (1):** one writer per crate across designers: a registry like the slots' claims, per wave.
- **Interfaces by expand, move, contract (1):** additive upstream changes let designers work at once; a skein lead shared by the designers orders the removals.
- **Mailboxes per designer;** the queue posts the branch's lead; the operator gets the rest; the ledger and operations are shared.
