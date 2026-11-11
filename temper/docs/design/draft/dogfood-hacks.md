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

- **What:** `TEMPER_WORKSPACE_REUSE` on the worker, unset by default. A new workstream that takes an idle workspace with the same repositories keeps the clone, and fetches and checks out in place. After a worker's restart, a workspace whose directory already holds those repositories is reused too. Any value counts as set; it's read once, at the worker's start. On main (`../domain/worker/README.status.md`).
- **Why:** a checkout otherwise starts empty for each workstream (`../domain/worker/checkouts.md`), so every file is new to cargo, and each crate recompiles. Kept, a new piece of work rebuilds only what changed.
- **Cost and risk:** a run can write its `.git`. With no contained trees (`../domain/worker/README.md`, section 8), its hooks reach later workstreams, and the worker's own git runs them, with push credentials in reach. Acceptable for one trusted person only. Old objects also pile up in a clone.
- **The pipeline:** a fixed set of long-lived checkouts, each claimed by one run, fetched and reset in place.
- **A real design:** the checkout's cache keyed by repository, not workstream, once contained trees make `.git` read-only and hooks are off for the worker's git. Possibly no switch at all then.

### 2.2 `cargo` on the worker's PATH

- **What:** a wrapper outside this repository, first on `PATH` through the agent's `environment` (`../protocol/configuration/README.md`). For a command inside `workspaces/<id>/`, it sets `CARGO_TARGET_DIR` to a directory of that workspace's own, outside the worker's root; and it runs the real cargo through our build admission.
- **Why:** a target kept outside the checkout survives the worker's `git clean -ffdxq`, a workspace made again, a restart and a wiped data directory. The admission gives a run's cargo what the pipeline's builds have: a limit on builds at once, a wait for disk, reclaiming idle targets, a lane's CPU weight, a memory kill reported as infrastructure, and a compile cache.
- **Cost and risk:**
  - it rests on `<id>` staying the same for a workspace, held by a test of 2.1;
  - **never share or copy a target across workspaces:** cargo's dep-info holds absolute paths, so a copied one misses changes, and a test goes green on old code;
  - a nested cargo must not re-enter the wrapper, or it deadlocks on its own place: the wrapper takes itself off `PATH`;
  - it reads the checkout's `.git/HEAD` time to see a new hold, since the worker tells it nothing, so a change of git's habits would stop its sweep silently.
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

### 2.8 An owned forge that registers no one through temper

- **What:** `TEMPER_FORGE_NO_REGISTRATION`, read where `temper forge prepare` and `temper forge sign-in` run, unset by default; an empty value counts as unset and any other, `0` too, as set. Set, a forge `application.sign_in.forges` names gets `[oauth2_client] ENABLE_AUTO_REGISTRATION = false` in its `app.ini`, with `USERNAME` and `ACCOUNT_LINKING = login` as ever, and no `[service] ALLOW_ONLY_EXTERNAL_REGISTRATION = true`; a forge it doesn't name gets what it gets unset. An account already linked keeps signing in, one that a first sign-in made before the switch among them. With `DISABLE_REGISTRATION`, Forgejo's link page keeps an empty signup form wrapper with no registration fields; both `/user/link_account_signup` and `/user/sign_up` refuse a registration POST with 403 and make no account. `temper forge sign-in` prints the same settings. Unset, both give the bytes of `../protocol/setup/owned.md`, 2 and 8.
- **Why:** the user wants registration off on the deployment's forge (2026-11-05). `DISABLE_REGISTRATION` is already true, but a member's first sign-in through temper makes their account (`../protocol/openid-provider.md`, 4). temper mints for a member alone, so a stranger makes none today (`../protocol/sign-in.md`, section 3); making one stays the default, since a deployment's other people need it.
- **Cost and risk:**
  - a member with no account gets no further on the forge than its link page until the forge's admin makes one, and then needs its password once, to link it;
  - every preparation writes `app.ini` whole and reads the variable anew, so one run without it turns auto-registration back on at Forgejo's next start, with no word of it. Whatever prepares the forge sets it every time; a `temper box` hands its own environment to its prepare step;
  - a managed Forgejo's settings are pasted by hand from what `temper forge sign-in` prints: the variable changes the print, not the forge;
  - whether `ENABLE_INTERNAL_SIGNIN = false` also closes the link page's password form hasn't been run, so internal sign-in stays on until every account is linked;
  - what these settings do is shown at Forgejo 17.0.1 alone, by the process tier's Forgejo sign-in story; a new pin runs it again.
- **The pipeline:** has no such sign-in: its forge is a public one, whose accounts are its own.
- **A real design:** a setting on the owned forge, beside `application.sign_in.forges`, with this as its value: whether a first sign-in makes an account or only links one.

### 2.9 CI that runs our gate on a build machine

- **What:** a script outside this repository, on the build machine, that a forge's workflow calls with its head and base. It runs the pipeline's selection of checks through the build admission of 2.2, in one fixed clone, swept as a target of 2.2 is: incremental output, and test executables a day old. A commit in the walk's copy of the repository takes its workflow file out of the index, so that the check of what this repository may name still passes, since a workflow naming a machine fails it.
- **Why:** temper has no gate of its own, by design, and CI on a forge needs a job that does what the pipeline's does, for a fair comparison.
- **Cost and risk:** about 35 GB, one job at a time, outside the pipeline's disk pool, so the machine's budget counts it by hand. The workflow file is out of step with the repository's index, which the next base stops needing.
- **The pipeline:** its merge queue runs this gate on a machine of its own, with its checks in order.
- **A real design:** a question: what a deployment's CI on a build machine looks like, and how it keeps a warm build of its own.

### 2.10 A comparison's charters

- **What:** for one comparison, a settings file outside this repository makes the deployment's charters carry the pipeline's prompts, models and efforts exactly. The goal's charter becomes a tick, at medium effort, that starts what a plan says. A producer charter becomes the plan's cutter, with the shell and write access, its instructions a pointer to a brief. The coding charter runs at high effort and the foreman at medium, both with the pipeline's text. The worker runs 8 at once with 10 workspaces, the pipeline's own table for that machine, by the user's word. A delivery's checks keep their 60 s bound, where the pipeline's have none, so a timeout under load retries and is reported as a skew. Raising it means raising `reserve_time` first, on every charter that allows a change. The OpenAI models' `idle` is 300 s, where `temper init` writes 30 s for every model, as 101 of 947 responses on a real run hit exactly 30 s with no event.
- **Why:** the user asked, on 2026-11-05, for prompts, models and effort to match exactly, so that a difference in the results is temper's and not its words.
- **Cost and risk:** a settings revision that's wrong for anything else, and these charters assume this one repository's plan format. The tick is still a model's run where the pipeline's is code, so its minutes are reported apart; writing it as a program would be a feature, not a hack. A planner with a shell reaches what 2.4 says the worker reaches. Ten targets could reach the machine's disk budget, but a workspace fills only when used, so a small plan fills few.
- **The pipeline:** its cutter and foreman have a shell, and its tick is code. Its run ends after asking a question, where temper's parks, so the coding charter's two lines on that say `wait`: the one place these words can't match.
- **A real design:** a question: whether a planner writes through a change at all, and how stock charters take what the comparison shows.
- **Per wake:** once `per-wake.md` is built, the comparison's goal charter is generated in its parts, the pipeline's cut as the cut and its answerer as the questions, each charter holding only the rules its role acts on (`per-wake.md`, sections 6 and 7).

### 2.11 Briefs on the worker's disk

- **What:** the cutter's charter points at a file on the one worker's disk that holds the pipeline's brief, along with a charter's reference copies, outside this repository.
- **Why:** the brief is the pipeline's, is larger than a charter should hold, and must match byte for byte, by checksum.
- **Cost and risk:** it works on one worker alone, and a change to the file changes runs with no settings revision to show it.
- **The pipeline:** it starts each run with a pointer to a brief on disk, in the same way.
- **A real design:** none: the brief is the pipeline's and goes with it.

### 2.12 Reading what a run was sent

- **What:** smith's trace, turned on in the worker's agent document with everything captured, gives each smith run's system text, tools and effort. A Claude run's effort is read from its own session file. 2.13 adds a dump of what leaves for the provider.
- **Why:** a match can't be claimed from settings alone: what a run was sent has to be read.
- **Cost and risk:** the trace and session files hold whatever the run was sent, so repository text and anything an agent read, on the worker's disk. The trace's delivery is best effort, so a count of its runs against the task list is part of reading it.
- **The pipeline:** its runs keep their own session records on the machine that ran them.
- **A real design:** a question: what a deployment's operators may read of a run's prompts.

### 2.13 Each model request kept in a file

- **What:** `TEMPER_DUMP_LLM=DIR` on the worker, unset by default; an empty value counts as unset, and a relative one is made absolute. It's read once, at the worker's start, and goes to each agent as the one variable in the environment the worker gives it; nothing reaches an agent's commands or its program. A `temper box` hands its own environment to its serve step, so there it's set where the box runs. Set, a run writes files to `DIR`, each whole by rename and mode 0600, in a directory made 0700; a directory open to others, or another's, is refused.
  - **Names:** `<identity>-a<activation>-<number>-<what>`: the Start's 16-byte identity in hex, or `none` where it has none, then its activation and a number counting this agent's files, each at least six digits. An agent knows no task. temper's Start leaves the identity for the run to mint (`temper/crates/temper-engine-smith/src/start.rs`; `smith/docs/design/domain/run.md`, 3.2) and the writer reads the Start's, so on a worker every name is `none-a<attempt>-...`. The attempt is what says whose a file is: the deployment numbers its attempts as one count (`runs` in `jig/crates/jig-domain-core/src/numbers.rs`), so no two runs share one, and a task's activity names each of its own. The text a run was sent says what it was working on.
  - **The API, with an OpenAI-style account or Anthropic's:** `openai.json` or `anthropic.json`, one per request before it's sent: the body as the client rendered it for the provider, those very bytes, before any gzip. Its encoder takes the prompt and limits alone, never a credential or an endpoint, so no header, token or URL reaches it.
  - **Claude Code, a subscription's included: a gap.** The program makes its own calls, so temper never sees a request of its. What temper gives it is kept instead: `claude-code-argv.txt` (model and effort among them), `claude-code-instructions` (Markdown), `claude-code-tools.json` (the MCP tools it's offered), `claude-code-stdin.json` (each line on its input) and `claude-code-init.json` (its `system/init`, naming the model it took and its tools). Not kept: the program's own system prompt and built-in tools' descriptions, its later requests' messages, effort as it sends it, tool answers (they go over MCP), its environment, and `mcp.json`, which holds the MCP secret.
  - **One start file a run,** `start.txt`: mode, budget, reserve, waiting, main's and sub-agents' models with effort, tools granted, host tools by name, the brief's titles.
  - **Bounded:** before each file, an agent sums the sizes in `DIR`; past 4 GiB it stops, with one line. A failed write takes one line too, and the run goes on. A line goes to the agent's standard error and to `DIR/dump.log`, since a worker shows that error only as a failed run's tail.
- **Why:** dogfooding compares temper with the pipeline, and prompts, models and effort must match exactly. Nobody could read what a run was sent: the rendered prompt wasn't kept, and a Claude run's effort couldn't be read.
- **Cost and risk:**
  - the files hold whatever the run was sent: repository text and anything an agent read. They stay on the worker's disk, and nothing removes them;
  - unset, one branch at each request. Set, a second copy of each body until it's written, and a write and a scan of `DIR` that block the agent's loop;
  - **a store started empty counts its attempts again (2.6),** so a `DIR` kept across that can have a new run's file replace an old one's of the same name, with no line. Such a redeploy takes an empty `DIR`;
  - the bound counts what's already in `DIR`, so a full directory stops every later run's files until a person empties it; agents writing at once can pass it by one request each;
  - no test runs the variable from a worker through a real agent process to a file: the worlds cover each half, and the next comparison's run is the proof.
- **The pipeline:** keeps each call's rendered brief as a plain file beside its report. What a CLI then sends on the wire isn't kept there.
- **A real design:** a question: what a deployment's operators may read of a run's prompts.

### 2.14 A goal's result reaches `main` by hand

- **What:** a goal runs on the development deployment, against its forge's copy of this repository, and lands there with CI green. Then whoever drives it takes the diff from that copy's base to its landed main, leaving out the copy's workflow files (2.9) and the plan's own directory, applies it to this repository's current `main` as one commit naming the plan and its pull request, and gives that to the pipeline's merge queue, whose gate has the last word. Nothing in temper changes: it's a procedure outside this repository.
- **Why:** the user's next step towards moving our development to temper: real work done by temper, while `main` keeps one writer and the pipeline's slow tiers go on watching after each landing.
- **Cost and risk:** a second gate on work already checked, one commit where temper's history held several, and someone's hands on each landing. A refusal, or a red after landing, gets a repair from neither side: whoever carried it over answers for it. The copy drifts from `main` by what the pipeline lands meanwhile, so the diff can conflict when it's applied.
- **The pipeline:** its trunks land through the same queue directly, their commits kept.
- **A real design:** temper landing on this repository's `main` itself, with 2.9's CI as the gate, once new plans start in temper by default. This entry goes then.

## 3. Left out, with its gap

For the user to weigh; each can become an entry.

- **A saved-work resume, or a git operation that broke:** the workspace is made again by design. 2.2's target survives, but the new clone's files are new to cargo, so temper's own crates recompile for that run, through the compile cache. Keeping the clone means trusting a tree the design refuses to.
- **Between a run's commands** the claim is dropped, since the worker tells the wrapper nothing of a hold. Under disk pressure a held workspace's target can be reclaimed, and its next build is cold.
- **Two commands at once in one workspace:** only the first holds the claim. cargo's own lock serialises them, but reclaim could reach the second under disk pressure.
- **A new machine starts cold:** the pipeline's checkouts are warmed from a template. A target can't be copied across paths safely (2.2), so each workspace's first build goes through the compile cache alone.
- **The benchmark's noise:** on a shared machine both draw on one set of places and one disk. The user reads that as mostly noise, since a temper goal should run much as a plan of the pipeline's does, its slots mostly independent. So they may run side by side; the figures say so.
- **No figure yet:** nothing here has been measured with a real run.
- **What a Claude Code run sends:** 2.13 keeps what the program is given, not its requests. Its own system prompt and built-in tools' descriptions, its later messages and its effort as sent stay unread, so a match on a Claude run is a match of what temper gives it. Reading them would take a proxy in front of the program, which costs more than 2.13 did and changes the path a subscription's call takes. Not planned.

## 4. Open questions

- Which machine runs the worker (2.4)?
- Does any entry become a real feature before the switch, rather than after v0.1.0?
- Does `ENABLE_INTERNAL_SIGNIN = false` leave the link page's password form (2.8), so that internal sign-in can go off once every account is linked? Nothing has run it.
- What may a deployment's operators read of a run's prompts (2.12, 2.13)? `../domain/worker/README.md`, section 10 holds the question.
