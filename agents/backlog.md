# Backlog: the pipeline, its tools and its next ideas

Provisional, 2026-10-29. Kept by the coordinator and the operator: what is decided and not yet done, and the ideas to try, each with what would show it is worth it. Incidents and their remedies are in `agents/ledger.md`; this file is what comes next.

## 1. In one page
- **Throughput is the goal:** (1) a pipeline that widens, (2) short builds on warm caches, (3) no spurious blocking. An item names which it serves.
- **Data before structure:** an idea that changes the code's shape (crates, tiers) waits for the numbers that justify it; collecting them is an item of its own.
- **Done items leave this file;** what they taught goes into the ledger or a role file.

## 2. Queue and gate
- **Merge train (1), paused:** the user retired hetzner2 for this phase on 2026-10-29: capacity is subscription-bound and the builders and queue are lightly used. Release its second-prover reservation; pause the two-gate train and transport work that requires it until measured queue busy time is again most of the available time. Keep the existing controller/prover disabled and retain the unlanded transport work and evidence. Revisit a second gate only after that bottleneck returns and a machine is explicitly assigned; h8 continues ordinary batch16.
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
- **Tiers we dropped for the spike (3):** browser, process, live, qa: their scripts assume per-tree targets and profiles the root `.config` lacks. First, a periodic run outside the gate (nightly, or after each wave) of the test targets no gate selects, temper's `qa_` binaries and the process tier, reporting failures to the tree's lead: retained `qa_` stories failed unchanged on main with nobody told (ledger 52; `qa_github_land`'s checked-head story from 10-28 until its fixture was fixed on 10-29). Restore the rest once the spike's numbers are in.

## 5. More than one designer
- **Operator effort (1, 3):** measure the Codex operator’s response times and missed incidents at high before deciding whether to try medium.
- **Crate leases (1):** one writer per crate across designers: a registry like the slots' claims, per wave.
- **Interfaces by expand, move, contract (1):** additive upstream changes let designers work at once; a skein lead shared by the designers orders the removals.
- **Mailboxes per designer;** the queue posts the branch's lead; the operator gets the rest; the ledger and operations are shared.

## 6. The ladder's findings off the happy path (qa)

Kept by coordinator q (s/qa). v0.1.0's happy path is the ladder's stories used as intended on the
four backends (temper `docs/design/draft/v0.1.0.md`, 4); a walk's finding off it waits here, by
the user's rule (10-29 13:35Z), until v0.1.0 is out or a walk shows it is on the path. Each line:
what, whose code, the evidence.

### 6.1 jig

- **A refusal names no tool family.** `Finding::Tool` carries no family or source, so a chat's
  refused `decide_escalation` reads "Authority refused: tool family; tool family; tool family;"
  (one per authority level checked: task, project, deployment). Rung A walk 12 attempt 2,
  hetzner6 `~/src/rust/tmp/qa-focused/rung-a-12/evidence/a2-empty-chat-held.txt`; temper renders
  it at `temper-engine-smith` `answers.rs:436`.

### 6.2 temper

- **A custom charter's shell writes for a run that cannot deliver.** The rail "modify only with
  deliver" (agent.md 4.3) does not cover `shell`; the stock gives shell only to charter 2 (change
  coders). lead/q/temper, g-modify, 10-29.
- **ctl names a not-yet-made task "unknown".** `task show N` before task N exists exits 2
  `{"error":"unknown"}`, not "no task N yet" (jig `documents/small.rs:24` Missing::Unknown to
  ctl's `jig/render.rs:272`). green's fixture investigation, 10-29.
- **Walk 12's tooling list:** `task show` cuts a result to 128 bytes even in `--json`; a whole
  activity takes one cursor call per page; a budget refusal "Authority missing" names no limiting
  setting; `task show --until replied` returns while delegates run. Rung A walk 12 reports.
- **The playground's starter CI only echoes:** rung A's "passed by CI" is a workflow that
  compiles and tests nothing (walk 12 attempt 2, `a2-csv-job3.log`).
