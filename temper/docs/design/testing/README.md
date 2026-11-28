# Testing temper

Provisional, 2026-10-03, revised 2026-11-02. How temper is tested. The
strategy is skein's `../foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is temper's: its tiers (`tiers.md`, section 1), its neighbours and their
fakes (`README.md`, section 3 and `fakes.md`), its scenarios, referee and its case library
(`scenarios.md`, section 1), what its worlds check beyond the strategy (`checks.md`; the
engine's, `engine-checks.md`; blame and tries for gates, `blame.md`; the worker's
waits, disk and faults, `worker.md`), where the box's own checks run
(`box/README.md`) and the shells' (`shells.md`), the restarts and stops they inject (`restarts.md`),
the workloads their worlds run (`workloads.md`), the seeded draw and outcome
comparison (`workloads-kit.md`), the sweeps that mix every failure
(`workloads-sweeps.md`), the kinds the composed sweep adds in a campaign
(`composed-catalogue.md`), the
shipped settings checked against themselves (`settings-tests.md`), every commit's store
checked against a start (`store-check.md`), its
layout (`README.md`, section 5), its tools and fixtures as binaries (`tools.md`), what
macOS tests (`macos.md`), QA after landing beside the deployment (`qa/README.md`), and what
is still open (`README.md`, section 6).

- `tiers.md`: temper's testing tiers and their open work.
- `scenarios.md`: scenarios and their referee.
- `checks.md`: what the worlds check beyond the strategy.
  - `checks-stage-one.md`: stage 1's golden, its check and its judge.
- `reference-case.md`: the case library and its opening act.
  - `reference-case-changing-direction.md`: the case's change of direction.
  - `reference-case-milestones.md`: the case's milestones and hand-offs.
- `workloads.md`: the fault vocabulary, seeded workloads in each world and restart comparisons.
  - `workloads-kit.md`: the seeded draw, logical names, parties' acts and outcomes.
  - `workloads-sweeps.md`: per-family and composed sweeps, exclusions and campaigns.

## 1. In one page

- **Behaviour is tested in the domain layer.** The engine and the worker
  are each a root domain with child domains, and each has worlds of its
  own; the system world runs the real engine and worker together, with
  smith's agents.
- **Below the domain, one tier per layer it adds:** step tests of every
  codec and translation with built-value round trips and records drift,
  and each parser against transcripts of real peers; machine worlds of the forge component, the git component
  and the terminal shell; the forge's protocol world; temper's client
  joined to the engine's root; the one-process world on skein's
  simulator; the real-loop world on one ring; reconcile's world; the
  process tier, the real binary with an owned Forgejo, its runner and
  real git; and the live tier, with a real model.
- **Peers across a wire, the machine beneath.** LLM providers, their
  sign-in's issuers and the forge are peers, whose fakes grow into
  services and meet temper at the wire; Claude Code, the program smith's
  agent runs on a program's login, is faked by smith's fake `claude`
  (`fakes.md`, 1); people use temper's client, and their fake is skein's fake
  person, in front of it. The machine (files, processes, and the
  programs they run: git, rg, sh, temper's own) stays one set of state
  machines, with a face for each layer of temper it can sit under.
- **The fake forge speaks Forgejo's dialect and, through its GitHub
  layer, GitHub's** (skein's `testing/fakes/github.md`, section 1), one world per
  forge kind, API and git in one store, so a branch the worker pushes is
  the head the engine reads. Its protocol layer serves the dialect's
  API; its git is reached by the fake git program, through the forge's
  own calls, never git's protocol over HTTP.
- **Checks follow content.** The machine's sh is a small subset of POSIX
  sh, in which fake repositories write their checks (`fakes.md`, 3), and the fake
  forge's CI follows a cue in the pushed tree (`fakes.md`, 2), so a change that is
  wrong fails in the run and on the forge alike.
- **A referee in every engine world,** on skein's world harness, holds
  its scenarios' expectations (`scenarios.md`, 1.2).
- **The case library runs in the real loop,** step by step, on its cast,
  its scripts and an oracle over effects that every world running the
  case reuses (`reference-case.md`).
- **No fake of temper's own components:** the system world runs the real
  engine and worker.
- **Two suites, and two tiers outside them.** The focused and the fuzzy
  suites have a CPU budget for each layer, held at the gate
  (`checks.md`, 1; `tools/tiers/budgets.md`); the process
  and live tiers run by their scripts, never at the gate. The process
  tier holds only what processes show, within its 540 CPU s (`process-tier.md`). The
  540 is the tier's 6-minute aim, raised by half by the user's word of
  11-08, for now, to be revisited.

## 3. Neighbours and their faces

### 3.1 Peers

A peer's fake grows the layers that the temper component facing it
grows, and each tier joins the two at the lowest layer both have
(skein/docs/foundation/testing-strategy.md, 4.1):

```
smith's agent  domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake LLM
engine         domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   skein's fake forge
```

People are no peer of temper's: they use temper's client, temper's own,
which reaches the engine over jig's client protocol, and their fake is
skein's fake person in front of it (`fakes.md`, 4). A peer can be a client as well as
a server: the fake forge's protocol layer sends signed deliveries to the
engine's webhooks' listener (../protocol/forge/README.md, section 4; skein's
`testing/fakes/forgejo.md`, section 1); and reconcile meets it as the engine does,
through the setup's routes (../protocol/setup/testing.md, 2).

### 3.2 The machine

The machine's fake grows no protocol or io layers, and shows a face to
whichever layer of temper sits just above it (skein/docs/foundation/testing-strategy.md,
4.3). Its domain face answers the tools' file operations, commands and
searches and the checkout's git operations; its io face, io's records,
a spawn with an argument vector among them. Every component's files and
processes go through it: the worker's workspaces and agent processes,
the agent's tools and checks, and the engine's store.

## 5. Layout

```
crates/*/src/tests.rs               step tests
tests/codecs                        temper-codecs-world: the generated codecs and the foreign documents' decoders
bin/temper-box/tests/domain          step and domain checks over values: profiles composed in memory and admitted, the kit's harness checks, the relay's framing, the landing kit's decisions, scripted openings and client-tool reads (box/README.md, 2)
bin/temper-box/tests/system          the live profiles' settings through the client's world joined to the engine (box/README.md, 2.7)
bin/temper-box/tests/suite           what only real files, sockets, locks, children and shipped wiring show (box/README.md, 4)
crates/temper-box-shell/tests/simulated  temper-box fake's hosts on skein's simulated kernel with their real peers (box/README.md, 2.4)
crates/temper-forge-setup-shell/tests/step  process-boundary checks over committed source and sample calls, and preparation, private files, locks and commands' captures over supplied facts (shells.md, 2.1); suite keeps TLS fetch, a child's lock, a real child and its deadline (shells.md, 4)
bin/temper-box/tests/process.rs      owned Forgejo integration and OS process boundaries (2.6); nextest profile process-temper
bin/temper-box/tests/proof_live.rs   B0's offline driver proof, its twin's walk (2.6, 2.7), nextest profile process-temper; scripts/proof-live.sh rung-b0-offline runs it
testing/temper-box-kit              what temper-box's test binaries share, compiled once for them all (tools/crates/design.md, 3.2): the stories' processes, forges, people and fixtures
testing/temper-box-kit/src/github.rs   GitHub's App as processes (2.6): the fake GitHub, the fake LLM, an engine alone and a worker alone, the App made on the layer and connected through ctl
testing/temper-box-kit/src/cluster.rs  the cluster's reader: `bin/temper-box/tests/fixtures/cluster/` started, awaited and stopped on skein-world's `Binary` (../protocol/cluster.md, 5)
testing/temper-process-guard        the kit's process guard: step observes scripted read/exit results, parent observes real /bin/cat identities
bin/temper-box/tests/fixtures/cluster  the retained cluster's description, read by the process tier
bin/temper-box/tests/fixtures/github   the GitHub App's documents, its key and its workers' secrets, read by the kit's GitHub stories
bin/temper-box/tests/live.rs         the live tier (2.7), nextest profile live-temper
bin/temper-box/tests/live_github.rs  the live tier's GitHub story (2.7), run by scripts/proof-live-github.sh
bin/temper-box/tests/live_github_managed.rs  the managed GitHub sitting: its Ready on the fake, nextest profile process-temper (2.6); its relay against github.com, owner-run (2.7)
tests/client-browser                temper-client-browser-world: the browser tier (../client/web/testing.md), Chromium against temper-box serve on the bundle scripts/browser.sh builds; nextest profile browser-temper
crates/temper-client-shell-terminal/tests  the terminal's machine world (../client/testing.md)
crates/temper-fake-git              the fake git program's core: temper-box fake git (4.3)
crates/temper-fake-git-hosted       the fake git program hosted by skein's simulated world (4.3), which the playground ships; its fixtures with the tests
crates/temper-playground            the world's composition, the playground's and the one-process world's (../playground.md, 3)
crates/temper-playground-serve-tool the browser tier's static host for the playground's bundle, called by bin/temper-playground-serve (tools.md, 3)
testing/temper-github-app           the App on the fake GitHub layer's first fixed key, its records, seeded in jig's secrets' store before a start (2.4)
testing/temper-fixtures             the documents, transcripts, memory literals and local browser that temper's client crates and worlds share
testing/temper-agent-fixtures       the agents' configuration as the worker's writer makes it
testing/temper-landing-fixtures     a project's landing settings, typed, for the engine's and client's worlds
testing/temper-engine-inspect       the engine's inspection adapter, read by its worlds alone (engine-checks.md, 2)
testing/temper-blame                the check of blame and tries for gates, over a world's fault ledger (blame.md)
testing/temper-faults               the fault vocabularies every world and sweep names (restarts.md, 2; workloads.md, 2)
testing/temper-workload             the workload drawn from the seed, the parties' acts and a run's outcome (workloads-kit.md, 1)
tests/<component>/<child>           a child domain's world, temper-<component>-<child>-world
tests/<component>/domain            a component's world: the engine's, the worker's
tests/engine/forge                  the forge connector's world
tests/engine/protocol-forge         the forge's protocol world (2.2)
tests/worker/protocol-git           the git component's machine world (../protocol/git.md, section 9)
tests/system                        the composed system world (2.1)
tests/conformance                   the engine root on jig's conformance harness
tests/client                        temper-client-world: temper's client with skein's fake person, joined to the engine's root (2.1, 6.3)
tests/one-process                   the one-process world (2.5)
tests/playground                    the playground's stories walked natively through the client's host, and a session log replayed (../playground.md, 9)
tests/real-loop                     the real-loop world (2.4), in the focused suite
tests/*/*/tests/suite/*.rs                a world's focused tests
tests/*/*/tests/suite/fuzzy/*.rs          its fuzzy tests: random worlds, domains driven at random
tests/fake-checkout                 temper-fake-checkout: working trees' git, over skein's fake checkout
crates/temper-twin-tool             the live run's twin: bin/temper-twin, selected transcripts as a fake LLM document (fakes.md, 1)
crates/temper-qa-tool               QA after landing: bin/temper-qa, main's landings triaged into cases and builds kept for them (qa/README.md); step holds values and seeded ledger sequences, suite keeps real scratch homes, sockets, locks and children
crates/temper-git-transcripts-tool  git's transcripts, recorded by bin/temper-git-transcripts against skein's throwaway Forgejo (../protocol/git.md, section 9)
crates/temper-client-gallery-tool    bin/temper-client-gallery: the browser's pages in named states, printed for screenshots (../client/web/gallery.md), its tests in the view's world
crates/temper-client-journeys-tool   recorded walks replayed through the shipped hosts and printed (../client/web/journeys.md; ../client/tui/journeys.md), run by bin/temper-client-journeys, its tests on walks they record
crates/temper-stage-one-tool        bin/temper-stage-one: configuring's stage 1, the golden, its check, its judge's brief and the evidence's collection (6.5), its step tests in the focused suite
scripts/proof-live.sh               the live tier
```

- **The worlds and the harness** are skein's world harness
  (`skein-world`); each world's crate is a library other worlds may use
  (`fakes.md`, 5).
- **The machine-level fakes are skein's:** the fake machine
  (`skein-fake-machine`), the fake checkout, the fake LLM and its peer,
  the fake issuer. temper's fakes sit on them.
- **The fake forge is skein's** (skein's `testing/fakes/forge.md`): its domain
  (`crates/skein-fake-forge-domain`), its protocol layer
  (`crates/skein-fake-forge-forgejo`), its forge peer (in
  `crates/skein-fake-peers`), the throwaway Forgejo
  (`testing/skein-throwaway-forgejo`), its tests in ordinary Rust, and
  the conformance (`testing/skein-forge-conformance`). Reconcile's world is
  in skein's `forge/setup/README.md`, section 5. temper's fake git program,
  its core in `crates/temper-fake-git` and its hosted half in
  `crates/temper-fake-git-hosted`, sits on the fake forge (`fakes.md`, 3).
- **Machine worlds live beside what they test,** as the layout places
  them (../protocol/forge/README.md, section 9); the forge component's is
  skein's (`forge/connection.md`, section 10).
- **The setup's crates** test as step code does: `temper-forge-setup`'s
  files, specs and steps, and `temper-box-machine`'s machine, by their step
  tests (../protocol/setup/testing.md, 1); reconcile's machine by skein's world;
  the hosts, `temper-forge-setup-shell` and `temper-box-machine`'s own, by the
  process tier (`process-tier.md`), as the binary's are.

## 6. Open questions

- **Real git against the fake forge.** How to connect the real-loop
  world's git subprocesses to the fake forge while retaining one
  authoritative repository state. A candidate is real bare repositories
  in the scratch directory, with an adapter for git access and the fake
  API's reads and merges. The design must keep object identities, refs,
  protection and merge results consistent, and reuse the fake's policies
  and scenario expectations. It is designed against skein's fake forge,
  in skein's `testing/fakes/forge.md`, section 3. What needs it: a real agent on
  the fake forge, which runs git in its working tree as a person does;
  nothing else does, since the fake git program reaches the fake forge
  across processes too (`fakes.md`, 3), and real git's own behaviour is the
  transcripts' and the process tier's. A likely shape: the domain stays
  authoritative over refs and policy, the bare repositories mirror it,
  the protocol layer keeps a table of the domain's counts to git's ids,
  a push's ref updates are decided by the domain before git writes them
  and its tree imported as the domain's commit, and a merge the domain
  makes is written to the mirror with a fixed author and time, so its id
  is the same on every run. Default: the real loop's clone, push and
  landing stories run on the fake git program over the fake forge's git
  route (`real-loop.md`), real git meets a forge in the real-loop and system
  worlds, and no story waits for the bridge.
- **What the real loop leaves out:** the real-git bridge (above),
  CI that runs a workflow's steps in the shell subset (how much sh,
  below), and faults in the loop. Several workers in one world are the
  cluster's services (`real-loop.md`); the native stop and kill witnesses
  are the process tier's (`process-tier.md`, *The cluster*), while loss
  and fencing policy are the engine's domain world's (`../protocol/cluster.md`, 6).
- **How much sh:** which commands and syntax the subset takes. It stays a
  subset of POSIX, so that real sh agrees.
- **Where the shell's interpreter lives,** so that the machine and the
  forge's CI share it.

## 7. Later

- **Further fuzzy-suite cost work** (`tiers.md`, 1.5, `README.md`, 6, `checks.md`, 1):
  `temper/docs/development/limits-fuzzy.md` holds the instruction pairs,
  complete-suite CPU samples and remaining causes. Keep all seeds,
  drawn inputs, crash cuts, invariants, endings and independent readbacks;
  neither fewer draws nor a cost-driven move resolves an overage.
  Further work needs:
  - A different admitted allocation contract for eager decoder lists,
    or coalesced scalar writing that preserves field order, whole-put
    refusal and overflow progress (skein's `runtime/lib-containers.md`, section 1, and `runtime/lib-bytes.md`, section 1).
    Required decoding and scalar reads remain; smaller measurement,
    conversion and scratch costs stay below the roughly 2% stop rule.
  - Cheaper independent native sessions and equivalent fake-person and
    service stepping, retaining every deadline, root secret check and
    cold readback. One-process prefixes have different process graphs
    and startup sources, so sharing them needs a mechanism that preserves
    each story and fault. Smaller setup and observation tails stay.
  - Cheaper complete mutable-state traversal and store/restore work,
    retaining both engine roots, exact trace comparisons and the final
    complete text. Required final text stays; the measured streaming
    alternative costs more, and old trace-prefix equality is below 2%.
  - Retained tool-schema admission with designed ownership and
    invalidation by charter, namespace and grants. Independent JSON
    validation, configuration admissions, prompt checks, allocation
    headers and byte searches remain; their inclusive footprints do
    not establish a removable real-loop gain above about 2 CPU s.
  - Separate investigation of instrumented observation-cap failures and
    intermittent raised-patience real-loop nonsettling. The suspected
    release/hold ordering is unproved; ordinary exact-name base and
    changed runs passed. Preserve the cap and ordinary patience.
  Default: fix a profiled cause with the same checks, or take the
  measurements to the user under skein/docs/foundation/testing-strategy.md, 8.1. Serial runs
  including the driver do not replace the gate's suite accounting;
  the request's roughly 15 CPU s saving and reliable headroom require
  further evidence. Budgets and enforcement settings remain separate
  user decisions.

- **Further focused-test cost reductions** (`tiers.md`, 1.1, 1.2, `process-tier.md`, `checks.md`, 1):
  the row profiles and their remaining causes are in
  `temper/docs/development/limits-heavy.md`. Follow skein/docs/foundation/testing-strategy.md,
  8.1: keep each behaviour in its tier, profile its cause, and retain
  every bound, pattern, shell, ending order, request, token lifetime and
  independent oracle. The next cause's gain is priced against about
  0.2 CPU s for this repair; a larger cause needing another mechanism
  stays named rather than being declared cheap. Further work needs:
  - Shared checked startup and preparation documents, preserving retained
    edits, refusal order and independent fresh provider keys. Required
    RSA generation stays; sharing keys or weakening them changes behaviour.
  - Cheaper Markdown and semantic-tree construction and destruction at
    the byte bounds, and retained-word admission without rebuilding rows.
    Repeated-position pages must keep their pairwise tie order.
  - Gallery phrasebook ownership or view reuse, checked capacity reuse,
    shared world construction and equivalent HTML rendering or batching.
    Each state's bindings, clock, sign-in configuration and every page's
    output still need their checks; cheap rules stay in the state holder.
  - Shared JSON decoding and escaping with exact validation, sizing and
    output; ready-call routing and maintained deadlines in the connection
    driver; streaming secret checks and chunked framing observations with
    the same cut and first-byte accounting. Shortening the simulated hour
    or skipping transport fragments removes coverage.
  - Projection feeds with less copying while retaining the whole-tree
    contract, and cheaper store joins and codecs under the independent
    restart/read oracle. Sharing a restart prefix needs snapshot support
    across the harness, box, I/O, simulator and fake machine, preserving
    all 720 ending orders and each fault and quiescence check.
  - Launcher interpreter startup and executable-loader work, without
    changing build settings or combining distinct behaviours for cost.
  Default: keep the required work and all assertions until a profile
  justifies the representation or algorithm change. Instruction ratios
  for selected tests do not establish a whole-suite CPU total.

- **Further fixture and startup cost reductions** (`tiers.md`, 1.1, 1.3, `checks.md`, 1):
  follow skein/docs/foundation/testing-strategy.md, 8 and 8.1, with unchanged behaviour,
  assertions, suites, build settings and budgets. A process-wide cache
  cannot remove the first construction in each binary's process, or in
  each test process of a slow tier. Work beyond the small fixture repairs
  needs these mechanisms:
  - Configuration: a checked generated stock representation or cheaper
    parsing and validation; sharing a bounded parsed startup document
    between its consumers while preserving diagnostic order; cheaper
    stock edits, init generation and measured turn-bound encoding.
  - Service and engine construction: shared checked admission results
    and cheaper owned policy/configuration snapshots; complete durable
    first-start rows checked against the real two-commit start; a hosted
    store constructor over supplied maps, retaining secret exchanges;
    shared checked sizing translations and generated bound validation.
    A consuming restart transfer must preserve the retained driver's
    independent oracle; reusable store transfer still needs replacement
    storage.
  - Store execution: borrowed row translation and commit encoding,
    cheaper independent history/mirror/inflight ownership, and batched
    restore/map reads preserving encoded bounds, lookahead, first-row
    refusals and corruption diagnostics. Engine decisions, page readback
    and the independent restart and final-store oracle remain exercised.
  - Words: checked ordered vocabulary construction, fewer nested owned
    allocations and clones, and shared capacity/presence/maximum scans
    that still detect public vocabulary mutation, missing words and
    overflow. Fusing validation passes must preserve whole-load error
    precedence, every form and hole, and bounded occurrence counters.
  - Drawing: indexed semantic-tree matching that retains stable keyed
    and role identities and exact patches; incremental application and
    checking that retains an independent patch oracle. Domain setup and
    rendering consumed by assertions still run, with the complete View
    in drawing stories.
  - Small setup and command costs: paired configurations, piece copies,
    idle bookkeeping, referee/default words, fakes, components and engine
    limits; Face and connection settling, routing and manifest work;
    process configuration templates and runtime-path scripts. Required
    GitHub signing needs cryptographic work or a separately justified
    seam preserving authentication. File/exec kernel cost needs its own
    measurement. Loader/process floors require separate binary/loader
    work; cheap rule assertions remain the behaviour checked.
  Default: retain these paths and their proofs until a profile justifies
  the required interface, representation or algorithm change. Stop a
  fixture repair when its next gain is below about 1% of its binary;
  larger remaining causes need the work above, never skipped checks.
