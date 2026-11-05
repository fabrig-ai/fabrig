# Dogfood hacks

Draft, 2026-11-05, at the user's request that day. **A plan adding a temporary change adds its entry here, in the same landing.** This is the list of what was done to run our own development on temper quickly, and the starting point of a cleanup after v0.1.0: each entry says what would replace it, even if only as a question.

The aim that excuses all of it, the user's: match what the pipeline we develop with today does, so that the two can be benchmarked fairly and we can switch with no loss of performance. Hack-ish is fine at reasonable cost. What's hard or costly goes to the user first, with its cost and the gap left without it.

## 1. In one page

- **A hack is off by default, or lives outside this repository.** No person running temper meets one unless they turn it on. None is a shipped default.
- **Each entry says:** what it does and its switch or file; why it exists; what it costs or risks; what the pipeline does there; what a real design might be.
- **Outside this repository** means the deployment's own tooling. This repository names none of it (`AGENTS.md`, 1), so this draft says what each does and not what it is called.
- **Section 3 holds what's still left out,** each with its gap, for the user to weigh.

## 2. The hacks

### 2.1 A workspace kept between workstreams

- **What:** `TEMPER_WORKSPACE_REUSE` on the worker, unset by default. A new workstream that takes an idle workspace with the same repositories keeps the clone, and fetches and checks out in place. After a worker's restart, a workspace whose directory already holds those repositories is reused too. Any value counts as set; it's read once, at the worker's start. On main (`../domain/worker.status.md`).
- **Why:** a checkout otherwise starts empty for each workstream (`../domain/worker.md`, section 5), so every file is new to cargo, and each crate recompiles. Kept, a new piece of work rebuilds only what changed.
- **Cost and risk:** a run can write its `.git`. With no contained trees (`../domain/worker.md`, section 9), its hooks reach later workstreams, and the worker's own git runs them, with push credentials in reach. Acceptable for one trusted person only. Old objects also pile up in a clone.
- **The pipeline:** a fixed set of long-lived checkouts, each claimed by one run, fetched and reset in place.
- **A real design:** the checkout's cache keyed by repository, not workstream, once contained trees make `.git` read-only and hooks are off for the worker's git. Possibly no switch at all then.

### 2.2 `cargo` on the worker's PATH

- **What:** a wrapper outside this repository, first on `PATH` through the agent's `environment` (`../protocol/configuration.md`). For a command inside `workspaces/<id>/`, it sets `CARGO_TARGET_DIR` to a directory of that workspace's own, outside the worker's root; and it runs the real cargo through our build admission.
- **Why:** a target kept outside the checkout survives the worker's `git clean -ffdxq`, a workspace made again, a restart and a wiped data directory. The admission gives a run's cargo what the pipeline's builds have: a limit on builds at once, a wait for disk, reclaiming idle targets, a lane's CPU weight, a memory kill reported as infrastructure, and a compile cache.
- **Cost and risk:**
  - it rests on `<id>` staying the same for a workspace, held by a test of 2.1;
  - **never share or copy a target across workspaces:** cargo's dep-info holds absolute paths, so a copied one misses changes, and a test goes green on old code;
  - a nested cargo must not re-enter the wrapper, or it deadlocks on its own place: the wrapper takes itself off `PATH`;
  - it reads the checkout's `FETCH_HEAD` to see a new hold, since the worker tells it nothing.
  - **its disk is a pool of its own:** it reclaims its own idle targets only, and the pipeline's reclaim frees only the pipeline's, so a machine's disk budget counts both by hand: about 35 GB a workspace, and `checkout.workspaces` bounds how many.
- **The pipeline:** every build goes through the same admission by hand, with the claim held for the whole run.
- **A real design:** temper knows a repository's build. A worker with a build's disk budget, its places and its sweep as settings; or a question: does temper own this, or does a deployment always bring its own?

### 2.3 What the agent's `environment` holds

- **What:** on a builder, `PATH`, `HOME`, `RUSTUP_HOME`, `TEMPER_BUNDLE=none` and the wrapper's and admission's own variables, set by hand in the agent's document. A setting that exists; the values are the hack.
- **Why:** commands get exactly this environment, independent of the unit's.
- **Cost and risk:** a copy of the machine's own environment that can drift from it, and a test tier's variables must be copied too.
- **The pipeline:** the machine's user units read one file.
- **A real design:** a project's build environment as a setting a person can see, with secrets kept out.

### 2.4 A worker on a build machine, as its user

- **What:** a temper worker on one of the machines we build on, running as that machine's own user, dialling the deployment's engine over a private network, with the engine and its forge elsewhere. The user's choice, 2026-11-05. Which machine is being settled.
- **Why:** the machine that runs the engine can't build this workspace, and temper leaves clusters across machines out of v0.1.0. As the same user, 2.2's admission is shared: one set of build places, one disk budget, one account of memory with the pipeline's builds.
- **Cost and risk:**
  - with no contained trees, a run has all of that user's permissions: every checkout of the pipeline's on the machine, what its logins and keys are while lent, and everything else that user can reach. A run's mistake or a bad prompt can reach them;
  - with 2.1, a run's `.git/hooks` reach later workstreams, and the worker's own git runs them with push credentials in reach; and here, as the same user;
  - both are acceptable for one trusted person's work only.
- **The pipeline:** assigns runs to machines by ssh, with its places counted, under the same user and with the same exposure.
- **A real design:** a worker's own user and contained trees, with clusters.

### 2.5 The deployed binary

- **What:** the development deployment runs a dev-profile `cargo build`, through 2.2's admission, never `--release`.
- **Why:** no CPU or compile-cache space spent on release builds, and 32 to 43 s from a warm target.
- **Cost and risk:** a slower engine and worker than a release, so **a benchmark's figures carry that.** Compare like with like.
- **The pipeline:** runs no engine of ours; what it runs is a script.
- **A real design:** the release's binary, once releases exist (`../release.md`).

### 2.6 A store change means a redeploy

- **What:** until the user says otherwise, a change to a stored family means the development deployment starts from an empty store. `upgrades.md` stays on the wish-list, with no baseline. A start refuses a store whose families changed.
- **Why:** no patches to write while formats move.
- **Cost and risk:** the deployment's history goes at each such change, so nothing that anyone needs lives only there. Restarts stay off the tested path.
- **The pipeline:** keeps its state in plain files of its own, which a change of temper doesn't touch.
- **A real design:** `upgrades.md`.

### 2.7 Things fed from outside

- **What:** accounts fed through `temper ctl` by our own tooling; a status page reading `temper ctl`; no backups or mirrors of the deployment's forge, so nothing lives only there.
- **Why:** each is outside what temper offers today.
- **Cost and risk:** rotation across many accounts isn't checked, and the status reader may tie itself to `ctl`'s words.
- **The pipeline:** a shelf of logins lent per run, a status page of its own, the repositories on a public forge.
- **A real design:** each a question for after v0.1.0.

## 3. Left out, with its gap

For the user to weigh; each can become an entry.

- **A saved-work resume, or a git operation that broke:** the workspace is made again by design. 2.2's target survives, but the new clone's files are new to cargo, so temper's own crates recompile for that run, through the compile cache. Keeping the clone means trusting a tree the design refuses to.
- **Between a run's commands** the claim is dropped, since the worker tells the wrapper nothing of a hold. Under disk pressure a held workspace's target can be reclaimed, and its next build is cold.
- **Two commands at once in one workspace:** only the first holds the claim. cargo's own lock serialises them, but reclaim could reach the second under disk pressure.
- **A new machine starts cold:** the pipeline's checkouts are warmed from a template. A target can't be copied across paths safely (2.2), so each workspace's first build goes through the compile cache alone.
- **The benchmark's noise:** on a shared machine both draw on one set of places and one disk. The user reads that as mostly noise, since a temper goal should run much as a plan of the pipeline's does, its slots mostly independent. So they may run side by side; the figures say so.
- **No figure yet:** nothing here has been measured with a real run.

## 4. Open questions

- Which machine runs the worker (2.4)?
- Does any entry become a real feature before the switch, rather than after v0.1.0?
