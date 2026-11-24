# slow-tests-down: what a lower tier can show runs at the gate

Draft, 2026-11-23, on the user's word: first move to the gate's deterministic layers whatever belongs there, then repair after landing and run it regularly. The gate runs only the layers that replay (`tools/tiers/layers.md`, 9.1); the real loop and end to end run after landing, in the real pairs and the slow pairs beside them (9.2). Of those slow tests, about two thirds show behaviour a lower tier could show, and most of that ran nowhere before 2026-11-23. This draft moves that behaviour down, in the order that makes after landing green and short first, and keeps above only what the strategy says only the real loop, end to end, a browser or a live backend can show.

## 1. In one page

- **What moves** (section 3): a slow test's assertions that a lower tier can show move to the lowest tier that shows them (`skein/docs/foundation/testing-strategy.md`, 9). What stays is the strategy's: what only the real kernel shows (2.8), one test per path through `main` or an integration point (2.9), what only a browser shows, and live backends.
- **Nothing is checked less** (section 3.2): a slow test is deleted only where a named lower test holds every assertion it made, or the same change adds one that does. Duplicates retire; everything else moves.
- **The order** (section 5): first the stories that fail after landing, shortened to their boundary, so that the slow pairs come back green and fast; then the large tables that ran nowhere (the box's suite, hearth's builder, codegen, QA, smith's benchmarks, GitHub's fake); then the shell and CLI matrices; then the browser kits' bookkeeping; then the rest.
- **What waits** (section 6): the cluster, restore and checkpoint stories' policy waits for failure-worlds' restart faults (`failure-worlds.md`, 4.8), and the real-loop world's scripted stories are failure-worlds' already (its execution file, F7).
- **The browser's deadlines** (section 7) go through skein's starvation rule (strategy, 2.10), and a red story's wall is bounded, so a busy builder starves a story instead of failing it, and a red run costs minutes.
- **The budget** (section 8): the cheap layers grow within their weekly accepted level; a move past it, or into a layer with none (system, simulated), brings its figures to the user. Every move is measured by the gate before it lands.
- **Done** (section 9) when every slow binary's remaining tests are of the kinds section 4 keeps, the slow pairs run green on main, and the gate's counts of checks done rose by what the slow pairs' counts fell.

## 2. Where it starts

- **The slow tests, read on 2026-11-23** at main `ea29d3cbd0`, binary by binary, by four read-only surveys: `../../development/slow-tests-survey-2026-11-23-skein-hearth.md`, `../../development/slow-tests-survey-2026-11-23-smith-jig.md`, `../../development/slow-tests-survey-2026-11-23-temper-1.md` and `../../development/slow-tests-survey-2026-11-23-temper-2.md`. Each classes every test or table row as real kernel, end-to-end path, browser, live, or could move lower, and names the lower test that already holds it or the seam that's missing. Their split is a reading of the strategy, not a measurement, and their line numbers describe that commit. By tree:

  | Tree | Slow binaries | Cases | Real kernel | End-to-end path | Browser | Live | Could move lower |
  |---|---:|---:|---:|---:|---:|---:|---:|
  | skein | 25 | 711 | 340 | 13 | 70 | 6 | 282 |
  | hearth | 7 | 256 | 25 | 5 | 0 | 0 | 226 |
  | smith | 11 | 303 | 23 | 41 | 0 | 22 | 217 |
  | jig | 10 | 31 | 4 | 6 | 6 | 1 | 14 |
  | temper | 41 | 1,104 | 94 | 162 | 14 | 10 | 814 |

  skein's real-kernel count holds its kernel conformance cases twice, for Linux and macOS. temper's leaves out the ten scripted stories that are failure-worlds' F7.
- **What ran after landing.** From 2026-11-16 to 11-19, after landing ran five slow pairs, `browser-jig`, `browser-skein`, `conformance-hearth`, `browser-temper` and `process-temper`: 16 of the 94 slow binaries, about 230 of their 2,400 cases. Green, they took about 20 minutes. Red, `browser-temper` alone took close to two hours, as each failing story waited out its patience. It was held from 11-20 on runner faults.
- **What runs now.** On 2026-11-23 after landing restarted with the five real pairs alone, `real-skein`, `real-smith`, `real-jig`, `real-temper` and `real-hearth`, which had never run: at `da0e2fcfe1`, 1,547 tests, all green once the run's scratch sat at a path short enough for a Unix socket, in about five minutes, none starved. It is held again, on the user's word the same day, until the moves of section 5 have landed: there is no point reading tests that are about to move.
- **Their budgets warn** (`tools/tiers/budgets.md`, 4.2): temper's end-to-end layer took 70.3 CPU s against its 37.9 s ceiling, `temper-box::suite` 51.0 s of it; skein's real loop 6.0 s against 5.7 s, and jig's 1.6 s against 1.5 s. temper's real loop took 191.0 s of its 239.7 s, the real-loop world's suite 144.5 s of it.
- **What failed, 11-16 to 11-19**, in 148 runs, at least 59 tests. The most frequent were temper's: the public board at two widths (59 runs), history (29), reading a chat (26), an outside Forgejo connected as its admin (36), a deployment restored at its place (34) and a landing found after a restart (26). Each walks a whole deployment to reach one assertion, and most of what it walks through is held lower already (`../../development/slow-tests-survey-2026-11-23-temper-1.md`, "What the November 16–19 failures show").
- **The worlds below have what most moves need.** skein's simulator and fake machine model descriptors, locks, modes, owners, links and the clock (`skein/docs/design/testing/README.md`); hearth's fake machine models file kinds and metadata (`hearth/docs/design/testing/fake-machine.md`); temper has the composed system world and the one-process world over skein's simulator (`../testing/tiers.md`, 1.1 and 1.5); smith and jig have their process, machine and protocol worlds (`smith/docs/design/testing.md`; `jig/docs/design/domain/testing.md`). What a move most often lacks is a seam: a shell entry point that calls the operating system directly, where a step or simulated test needs to hand it scripted facts.
- **Failure-worlds** (`failure-worlds.md`) gives the worlds the faults and checks production showed. Its F7 moved the real-loop world's scripted stories down; F8, the restart faults, and F9, the workloads and sweeps, are still to land (`failure-worlds.execution.md`, 1).

## 3. A move

### 3.1 What moves, and where

- **To the lowest tier that shows it** (strategy, 9): a step test where the behaviour is a pure decision over inputs, a domain world for a domain's policy, a system world for domains together, a machine, protocol or io world for a boundary, the simulated world for the processes together.
- **Into a world that exists,** named in the goal's design: a new world only where none can express the behaviour, and then the smallest that can.
- **A missing seam is added at the boundary that lacks it,** the smallest face that hands the decision its facts: a file's metadata, a command's exit and output, a socket's bytes, a clock. The production path keeps its behaviour: the seam's production argument is what the code calls today. A seam in another tree's package belongs to that tree's owner, and its goal says so.
- **Tables move as tables:** a slow table whose rows start a process each becomes a step or world table with the same rows, and one row stays above as the wiring's witness where the path is a real one (section 4).

### 3.2 Nothing is checked less

- **A slow test is deleted only where its every assertion is held below,** by a named test that already holds it (a duplicate) or by one the same change adds. The goal's design lists, for each slow test it removes or shortens, the lower test holding each of its assertions.
- **No sample, seed, case or assertion goes.** A move keeps every row of a table and every seed of a sweep; a shortened story keeps its boundary's assertions above and carries the rest down. This is the strategy's "same checks for less" (8.1), and the gate counts checks done per layer (`tools/tiers/budgets.md`, 13): the slow pair's count falls by what the gate's rises, and a drop anywhere else is the change's to explain.
- **A real behaviour keeps its real witness.** Where a story's last native observation is the only one of its kind (a kill, a stop, an exit, a descriptor handed on, a lock between processes), it stays above, alone, as section 4 lists.

### 3.3 One goal's work

- **Its design** names the binaries and groups it moves, the lower world of each, the holder of each assertion, the seams it adds and the witnesses it keeps, from the surveys, checked again at the goal's own commit.
- **Its tests** run in the focused and fuzzy suites, as every plan's (`AGENTS.md`, 5). The slow tests it changes or deletes are compiled, not run: after landing runs them.
- **Its figures**: the gate's own report of the layers it grows, against section 8's allowance.

## 4. What stays above

The surveys list it binary by binary (each survey's "What must stay" or "Coverage to keep"). In short:

- **skein:** the kernel conformance suite; the ring adapter, its startup probe and trust roots; signals, a process tree that ends, descriptors and the fork helper; TLS; the throwaway Forgejo's owned start and stop; one browser witness for each browser behaviour of the UI kit (layout, focus, real events, reload); the live provider stories.
- **hearth:** its Git conformance over real fixtures; real mmdebstrap resolution once; a render's native file modes and links where the fake machine can't show them; one path per command through `main`.
- **smith:** one path per CLI command and exit; a real program's run and its child; the real store's recovery on the real file adapter; real key generation; the live stories.
- **jig:** real reloads and `beforeunload`; the browser's origin and framing refusals; one printed link to a running bundle; the real store file on disk; the engine-clients' native signing boundary.
- **temper:** one real owned-Forgejo landing (the CSV story); the shipped deployment binaries' start; native kill, stop and exit of a cluster's processes; a restored generation's real start, and a checkpoint's command and exit; the box's real descriptors, ports and locks; one fetch over local TLS; the browser's geometry, focus, history entries, cookies, popups, framing and a real download; the TUI's native program; the real-loop world's real-kernel stories; the live stories.

## 5. The work, in order

Each group names its binaries; the surveys hold the cases, the holders and the missing seams. Sizes are the surveys': small, medium or large.

### 5.1 The stories that fail after landing

First, so that the five held pairs come back green and fast.

- **temper's real Forgejo paths** (`temper-box::process`): ordinary landing and restart, the plan face's landing tail and the playground's adoption move to the system and one-process worlds, which hold them already in part; the outside-Forgejo story shrinks to connecting the instance, its first repository and one reopening. One real landing witness stays. Medium.
- **temper's public board, history and reading** (`temper-client-browser-world::browser_all`): publish and unpublish, private fields, addresses and phase and time selection move to the domain, protocol and client system worlds; the board is seeded directly instead of walked to; only geometry, history entries and scroll stay in Chromium. Medium to large.
- **temper's workers story** leaves the browser for the client system world; the journey chain's sequencing is already held by the driver; the TUI journey's native path leaves `browser-temper`. Small to medium.
- **temper's repeated command paths** in `temper-box::process` and `proof_live`: the older-runner-only story, two client sign-in wrappers and one member exchange fold into the stories that already cross them. Small to medium.
- **jig's browser stories** (`jig-client-browser-world`): the panic recovery that skein's browser stories already show, and the pending request's resend that jig's protocol tier holds. Small.
- **skein's browser kit** (`skein-ui-web-tests::browser_harness`, `browser_stories`, `browser_example`): the harness's own bookkeeping, peers, keeps, commands and substitutions, to step and simulated tests; the patch shapes and refused writes to the UI kit's step and machine tests; one browser witness each. Medium.
- **hearth's conformance**: the include split and the recordings' replay are duplicates; the index and progress checks move to recorded resolution. Small to medium.

### 5.2 The large tables that ran nowhere

They ran in no suite before 11-23; moved, they run at every gate.

- **`temper-box::suite`**, 331 of 388 cases: fixture and harness states, admission and configuration, redaction, parsers, the fake backends' admission and git tables, forge setup policy, startup bounds, support checks. Step and simulated tests in `temper-box::domain`, `temper-service`, `temper-forge-setup` and the box kit. Large, in several goals by module.
- **hearth's builder world** (`hearth-builder-world::real` and `::suite`), 220 of 227: render and diff semantics, formatting and outcomes, generated-repository properties, resolution and store policy, to hearth's render domain and the builder's simulated and protocol worlds. Large.
- **`skein-codegen::suite`**, 142 of 149: schema and map validation, version refusals, generated modules and golden bytes, to `skein-codegen::step`, with an in-memory tree for the patch commands; one refusal stays as the CLI's wiring. Large for the tree seam.
- **temper's QA** (`temper-qa-tool::suite`, 136 of 149, and `temper-qa::suite`, 51 of 63): the ledger, queue, triage and recipe policy to a QA runner domain world over scripted files, clock, git and agents; the CLI's help, switches and exits to its command table's steps. Large.
- **`smith-bench::suite`**, 100 of 110: recorded-event judgement, task and probe declarations, verification and calibration, to `smith-bench::step`, with a test filesystem for rollout, mounts and staging. Small for the step work, large for the seams.
- **`github-conformance::suite`**, 44 of 44: its facts, clocks, budgets, token lifecycle, framing, queue faults, masking and journal, to step and protocol tests on the fake, with a supplied file tree for the journal and transcripts. Large for that seam.

### 5.3 The shell and CLI matrices

- **smith's shipped CLI** (`smith::end_to_end`, 60 of 92): direct and interactive behaviour, credentials and program checks, program runs and rechecks to smith's local- and agent-process worlds and shell step tests, with a file-operation seam around the CLI's finalizer; 32 process cases stay. Medium to large.
- **smith's store and native setup** (`smith-local-protocol::suite`, `smith-real-world::suite`, `smith-local-shell::real`, `smith-machine-world::suite`, `smith::live_configuration`): to the local-process world on the fake disk and step tests over path facts; the removal matrix is a duplicate. Small to medium.
- **temper's shells** (`temper-forge-setup-shell::suite`, `temper-box-shell::real`, `temper-shell::real`, `temper-client-shell-tui::real`, `temper-service-shell::real`, `temper-fetch-shell::real`, `temper-engine-shell::suite`): fetch, file handling, preparation, init and upgrade policy, private paths and lookup behind fake filesystem and command faces; native contention, descriptors and key generation stay. Medium.
- **temper's journey tools** (`temper-client-journeys-tool::suite`, `temper-client-journeys::suite`): print admission and redaction are duplicates; the writer and look-refresh scripts' git, file and process decisions get a step face. Medium.
- **temper's end-to-end binaries' tables** (`temper::suite`, `temper-engine::suite`, `temper-worker::suite`, `temper-agent::suite`, `temper-stage-one::suite`, `temper-twin::suite`): the help, version and refusal rows beyond one per path, to their command tables' steps. Small.
- **skein's shell and world harness** (`skein-world-tests::suite`, `skein-io-world::real`, `skein-ring-tests::suite`, `skein-shell::real`, offline login cases in `skein-llm-world::live`): duplicates retire; the drive's ordering and hosted kill report need a supplied clock and completion; login admission and root preparation to steps. Small to medium.
- **jig's provider and process controls** (`jig-engine-clients-world::suite`, `jig-test::process`, `jig-test::real`): token and code lifetimes, claims and the late-reply races to the engine-clients protocol world with a fixed signing fixture; status rows and startup refusals to steps. Small to medium.

### 5.4 The rest

The remaining real-loop world's stories that aren't F7's (`../../development/slow-tests-survey-2026-11-23-temper-2.md`, its groups 1 and 2), skein's remaining small groups (KV golden bytes, bundle routes, the fail-stop tail, the throwaway Forgejo's controls), the checkout's last-page refusal, and each single-row move the surveys list. Small each.

## 6. What waits for failure-worlds

- **The cluster stories** (killed, hung or restarted workers, an engine killed mid-proof): their loss, retry and fencing policy is held in the engine's domain and jig's fleet worlds already, and those assertions retire now; the mid-turn crash, drain and restart combinations move to F8's simulated restarts once they land, and the native kill, stop and exit witnesses stay.
- **Deployment restore, checkpoint and restore, and start-over**: the gate workflow, landing equality and store checks move to the one-process upgrade world and the forge's domain world now; the previous-build fixtures and arbitrary restart cuts are F8's; the copied-place launch, the real command dispatch and one start on the restored generation stay.
- **The real-loop world's scripted stories** are F7's, landed.
- **F9's workloads** may hold some of section 5.4's compositions better; a goal that finds so says so, and leaves the composition to F9.

## 7. The browser's deadlines

- **Through skein's rule** (strategy, 2.10): skein's browser harness classes a story's startup and its whole-run expiry by the rule, but a step's own expired observation, five to ten seconds in temper's and jig's stories, fails at once, before the rule reads it (`../../development/slow-tests-survey-2026-11-23-temper-1.md` and `../../development/slow-tests-survey-2026-11-23-smith-jig.md`, their flake causes). Each step's expiry goes through the same rule, so that under load a step starves and the story passes as starved, counted, instead of failing.
- **A red story's wall is bounded** at a small multiple of its green wall, within the tier's wall hard limit (8.1), so a failing story ends in seconds, not in a long wait, and a red run costs minutes.
- **One goal,** in skein's browser kit and the trees' browser worlds, before the browser pairs come back.

## 8. The budget

- **The cheap layers grow within their allowance.** A change growing a layer's instructions past 2% of main's count is red unless the layer stays within its accepted level (`tools/tiers/budgets.md`, 2 and 8), which `tools/tiers/budgets.toml` sets each week for the step, machine, protocol, io and domain layers at main's count plus 5% (the user, 11-23). Goals run side by side (the execution file, 2), and the accepted level holds them together: each gate measures its change against main with every earlier landing in it.
- **Past it, the figures are the user's.** A goal whose growth would pass the allowance, or that grows a layer with none (system, simulated), stops before landing and brings its measured figures to the user as a need, as a move down of the slow layer's ceiling (`tools/tiers/budgets.md`, 10) or a raise, never a cut of checks.
- **The slow layers' ceilings fall** by what moves out of them, as a move records it.
- **What a move saves** shows in the slow pairs' runs; it isn't the gate's to count.

## 9. Done

- Every slow binary's remaining tests are of the kinds section 4 keeps, each with its reason in `tools/tiers/layers.toml`'s line.
- The ten slow pairs run green on main after landing, the five browser, conformance and process pairs back beside the real pairs, a green run within the slow pairs' budgets, and a red run within minutes.
- The gate's checks done rose by what the slow pairs' fell, layer by layer, with every slow test deleted named beside its holder in its change.

## 10. Out

- **The live stories,** run by choice and daily; their offline admission cases are in, their calls are not.
- **macOS's runs** follow the Linux moves; its own degraded tests are its draft's.
- **The restart faults and workloads** are failure-worlds' (section 6).
- **Changing a layer's ceiling or allowance** is the user's (section 8).

## 11. Tests

This draft's goals are checked by the suites: the gate's focused and fuzzy runs, its counts of checks done per layer, and the slow pairs after landing. A goal adds no tier run of its own.

## 12. Later

- **Splitting binaries** that span a replaying layer and a slow one, where a move leaves one so (`tools/tiers/layers.md`, 10).
- **A per-binary note of what each slow test still shows,** generated from `layers.toml`'s reasons, once the moves have settled.
