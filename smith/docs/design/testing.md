# Testing smith

Provisional, 2026-10-08, revised 2026-11-02. How smith is tested. The
strategy is skein's `docs/foundation/testing-strategy.md`: the tiers,
faults, what every fake shares, what every world checks, scenarios and
the referee, the two suites, and where a failure is fixed. This document
keeps what is smith's:
- its tiers, end to end included, and benchmarks beside them (`testing-tiers.md`);
- its neighbours and their faces (section 3);
- its fakes (section 4);
- its scenarios and referee (section 5);
- what its worlds check beyond the strategy (section 6);
- its layout (section 7);
- what is still open (section 8);
- what is left for later (section 9).

The stories each world tells are in the design of what it tests, in that
document's "The world" section.

## 1. In one page

- **Behaviour is tested in the domain layer.** The agent's child domains,
  the agent's root, the host domain and the local host each have worlds
  of their own. The local host's world runs the agent inside it, and so is
  a system world.
- **Protocol worlds join smith's own halves:** the channel's host and
  agent halves over byte streams cut at random, and the LLM component
  against skein's fake provider.
- **Simulated worlds run processes** on skein's world harness: the
  machine component, the agent service, a host spawning the agent
  service, and the local host with its peers. The harness hosts a service
  that a spawn starts, so one world holds a host and its agent. That
  agent is the shipped adapter, never a test-only copy.
- **The real loop is one thread, one loop, one ring.** The product runs
  as it ships, with its agent inline or spawned and hosted in the same
  loop over pipes. The fakes serve on loopback. Only git, rg, sh and the
  checks run outside the loop.
- **Plaintext below the real loop.** The replaying tiers connect to their
  fakes in plaintext on loopback, so they replay. TLS runs in the real
  loop.
- **End to end, smith runs as it ships:** the root package's binaries as
  processes, against fakes in the focused suite, and against the real
  providers, their issuers, Claude Code and a git remote in a live suite
  outside the gate, on smith's own accounts, signed in once: the
  after-landing tier runs it after every landing, and the owner runs the
  story on a key (`testing-tiers.md`, 3).
- **Benchmarks are not tests.** Real agents on real models are measured
  in a tier of their own, outside both suites and the gate (`testing-tiers.md`, 4).
- **One scenario, every tier its fakes reach.** The local host's
  simulated world and the real loop run the same processes, scenarios
  and referee, in both placements where placement matters. Only the loop
  beneath them changes.
- **People react, and no line is lost.** The scripted person types on
  what it observes, with its timing drawn from the seed, and the referee
  checks that every line ends once (section 5).
- **Worlds end as the product ends:** with shipped keep and idle times,
  and under skein's teardown invariant (section 6).
- **The fakes are skein's.** smith adds only the scripted neighbours its
  own worlds need: a host, a parent, a person at a terminal, a browser,
  and the fake program a program backend runs in place of Claude Code
  (section 4).

## 2. smith's tiers

Moved to [`testing-tiers.md`](testing-tiers.md).

### 2.1 Simulated worlds

Moved to [`testing-tiers.md`](testing-tiers.md), section 1.

### 2.2 The real loop

Moved to [`testing-tiers.md`](testing-tiers.md), section 2.

### 2.3 End to end

Moved to [`testing-tiers.md`](testing-tiers.md), section 3.

### 2.4 Benchmarks

Moved to [`testing-tiers.md`](testing-tiers.md), section 4.

### 2.5 Which tier a test belongs to

Moved to [`testing-tiers.md`](testing-tiers.md), section 5.

### 2.6 Placement

Moved to [`testing-tiers.md`](testing-tiers.md), section 6.

## 3. Neighbours and their faces

### 3.1 Peers

A peer's fake grows the layers that the smith component facing it grows,
and each tier joins the two at the lowest layer both have
(testing-strategy.md, section 4.1):

```
agent        domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   fake LLM provider
local host   domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   fake OAuth issuer
host         domain ─ protocol ─ io   ⇄   io ─ protocol ─ domain   agent
```

- **A host and its agent are each other's peer.** Both halves are smith's.
  Their worlds use the other half, or skein's fake channel scripted with
  the channel's records (protocol/channel.md, section 10).
- **The person** meets the local host as lines at a terminal, and as a
  browser that follows the sign-in's redirect.

### 3.2 The machine

The machine's fake grows no protocol or io layers (testing-strategy.md,
section 4.3). It shows a face to whichever layer of smith sits just above
it:

| smith is real down to | The machine's face | It answers |
|---|---|---|
| the domain | skein's fake checkout | the tools' files, searches, commands and checks; the local host's git |
| io | behind the simulator: skein's fake machine and fake checkout, with smith's adapters for its programs | the file and process operations the simulator passes on |
| everything | none | the real kernel, in a scratch directory; in contained trees once they exist |

Every component's files and processes go through it:
- the agent's tools and checks;
- the local host's chat files, the credentials directory's files and
  its lock, and deliveries;
- a host's agent process, and the program an agent runs as its child.

## 4. The fakes

- **The LLM provider:** skein's fake provider. Its domain face serves the
  domain worlds. Its protocol face serves bytes to the protocol worlds.
  As a service on io, it listens on loopback for the simulated worlds and
  the real loop. Its recorded exchanges are skein's.
- **The menu:** the argument bodies the fake copies into the calls it
  draws, `smith-session-world`'s `provider::menu`, which the session,
  agent, host and local worlds share. The fake draws a tool and a body
  apart, so each body carries the fields of every tool: a path, a
  content, a command, a pattern and edits, and in some of them `job`'s
  number and a shell's `yield_seconds`, 0. A drawn call reads its
  own tool's fields, so a `job`, `shell`, `read` or `write` call takes
  its own from any body. Its invalid entries are an unknown tool's name,
  a cut object and an empty one.
- **The `job` slot:** `job`'s number in a body is the menu's numeric
  slot `job` (`skein/docs/design/testing/fakes/drawn-agent.md`, 2),
  between `job ` and a space, as the product renders a job
  (`domain/session.md`, 10), within a job number's bounds, 1 to the
  largest `u32`. Its never-issued value, 999999, is past any number a
  world's session issues, and its weight is a tenth. So a drawn `job`
  call names a job its conversation was shown, stale or not, or one no
  kit issued, and the session world records the class of each
  (`domain/session.md`, 10) beside the slot's draw in the fake's
  capture where it captures.
- **The seeds it draws from:** smith's session world gives the fake
  the parts of its seed's streams (`skein/docs/design/testing/sweeps.md`,
  2): `agent/model`, and `Faults`' and `Scheduling`'s `llm`. The agent,
  host and local worlds, not on streams, give their one seed, so their
  scripted turns' failures and latencies draw as they did. A test that
  shows what a seed drew names its seed for what it shows, so that a
  change of the draws picks it again by that name, or scripts it.
- **The OAuth issuer:** skein's fake issuer, with the same three faces
  (skein's `oauth/README.md`, section 5), reached through a `sign_in` account's
  moved addresses (shell/credentials.md, 1), so that smith's own registration is
  what the worlds exercise: the entry client, the issued client id, the
  scopes and the ID token.
- **The program:** smith's own fake `claude` (protocol/programs.md,
  section 8), which the agent spawns by its path in place of Claude
  Code: a step machine the machine's face runs in the simulated worlds,
  and a binary of the tests' own in the real loop and end to end. It
  answers the program's status, sign-in and sign-out, and speaks its
  stream both ways and its MCP client, as its story scripts.
- **The machine:** skein's fake checkout and fake machine (3.2). smith
  seeds repositories for its scenarios, and adapts programs to them.
- **The channel's far end:** skein's fake channel, a scripted peer that
  speaks the channel's records.
- **smith's scripted neighbours** are ordinary Rust, each in the world
  that needs it: the host in the agent's world, the parent in the host
  domain's, the partner in the run's. The terminal and the browser run in
  both the local host's simulated world and the real loop, so they are
  step machines (testing-strategy.md, section 4).
- **No copies of shipped code.** A world hosts what ships: the agent
  process is `smith-agent-shell::Agent` and the local host is
  `smith-local-shell`'s `Local`. The agent's standard error is a writer
  the world observes, so the referee can check the tail its host
  reports.
- **Fakes do not end things for their clients.** A fake peer stays live
  until its client closes, unless the scenario is about the peer hanging
  up (section 6).
- **Serving a fake on loopback is skein's:** one service shape for any
  fake peer, which smith's worlds configure.
- **Their addresses on a real host** (`testing-tiers.md`, 2 and 3) are the test
  process's own, so that tests running side by side never meet. On
  Linux, which answers every address of `127.0.0.0/8`, a fake takes
  `127.1.0.0` plus the test process's id and its fixed port, as it does
  now. On macOS, whose loopback answers `127.0.0.1` alone unless aliases
  are made, it takes `127.0.0.1` and a port of its own: a listener bound
  to `127.0.0.1:0` and held from before the fake starts, the address the
  kernel gave it written into the settings that reach the fake, and the
  listener handed to the fake, which serves on it without binding again
  (skein's fake peers' `adopt_listener`; `temper/docs/design/draft/macos.md`,
  4.1). The simulated worlds keep `127.0.0.1` and their fixed ports on
  both. A fake whose listener fails says the address it was given and the
  error, never only that it must bind. The held listeners are the test
  process's, in one table behind a process-wide lock, which a fake
  reaches only where it holds a listener there: a real host's, on
  macOS. A simulated world, and a real one on Linux, holds none, and
  making or adopting its fakes takes no lock: that lock's first use
  inside a peer's construction would be charged to the peer, which its
  drop never frees, on macOS alone
  (`skein/docs/design/testing/service-tiers.md`, 1).

## 5. Scenarios and the referee

- **A scenario** says, in the fakes' own terms:
  - what the person types, and on what it waits before typing it;
  - what the provider answers;
  - the issuer's accounts;
  - the workspace's repositories and checks;
  - the faults, drawn from its seed.
- **The person script** types on what it observes: a turn, an answer,
  waiting, a line shown, a notice, or a number of steps drawn from the
  seed. It may close its input at any point. In the fuzzy sweeps its
  lines, their triggers, the store's delays and where input ends are
  drawn from the seed.
- **The referee watches from outside:**
  - the lines the terminal received;
  - what the fake provider and the fake issuer saw;
  - the facts the services emit, and the event stream;
  - the checkout, read through one small face: its head, and a commit's
    message and files. The fake checkout answers it in simulation, and
    git does in the real loop.

  It never reads a service's state, the simulator's or the fake
  machine's.
- **Referees check message terminals** (domain/run-messages.md, section 1):
  - every message a run accepted ends exactly once: read (a turn's fence
    names it or a later one), refused at admission, or unread at the
    answer;
  - fences never go back, and the answer carries the last;
  - a line is relayed at most once per activation, and again only if it
    was never read;
  - the host's report of unread names comes once;
  - a waiting notice appears only when every relayed line was read.
- **No line lost** is liveness: by the end, every line the person entered
  was read or shown as not delivered, a refused one with why
  (domain/host.md, section 8). A line still pending when the referee's
  deadline fires fails the test, naming it.
- **One referee per scenario, every tier:** the local host's simulated
  world and the real loop share theirs. End to end, the referee checks
  the same outcomes from outside (`testing-tiers.md`, 3). A domain world's referee is its
  own, as its scenarios stay in its tier.
- **Its checks return their findings** (`skein/docs/design/testing/service-tiers.md`, 1;
  `skein/docs/foundation/programming-model.md`, 10.4): every referee and
  judge of smith's worlds implements `watch`, in every tier, and keeps
  what its `act` or `idle` finds for its next `watch`. The event
  checks, a world's event conformance and the program referee's
  comparison of the messages a program sent with those the fake saw,
  with the transport's own check beneath them, return the first
  difference as their finding, naming the record or the message by
  position; the world or the story panics on it at its top, and their
  own tests read it. So do the host world's common parent referee, over
  a run's answers, its terminals, its unread report, its fences and
  `Gone`, with the inline world's check of an answer to a parent call;
  the protocol world's reading of a native request the component sent,
  its cache markers and breakpoints (`protocol/llm.md`, 2.3); the
  session world's window referee, over a window's system, affinity,
  history and bound (`domain/session-context.md`, 1); and the local-process
  world's person, whose terminal write that fails where its script
  doesn't expect a failure is its finding. Nothing in smith's tests
  catches a panic, and none triggers an assertion: what an assertion
  guards is tested through the outcomes around it
  (`skein/docs/foundation/programming-model.md`, 10.4).

## 6. What the worlds check

Every world checks what the strategy lists (testing-strategy.md, section
6). In smith:

- **Facts change nothing:** a run is the same whether its facts are kept
  or dropped.
- **Events are complete:** the event stream matches what the fakes saw:
  one response record per provider response, tool records paired by the
  provider's call id, one end for each job a `running` verdict named,
  told by a later answer or by its `job.ended` (domain/tools.md, section
  5), usage summing to the answer's spend, and every loss counted exactly
  (protocol/events.md).
- **Memory** is measured by the counting allocator at every iteration
  and through teardown. Each fixed scenario has one measured total peak
  under a ceiling with headroom, named beside its test. Caller-held
  outputs and transient ownership belong to that total: neither a second
  held-bytes ceiling nor a doubled or tripled formula stands for it.
  Ceilings move by hand when a real change passes them, never with a
  type's size (programming-model.md, 6.3). The protocol-machine scenario
  finishing eight full 512-entry listings has a recorded total peak of
  124,960 bytes and `LISTING_TERMINALS_PEAK_CEILING = 160_000`: a quarter
  of headroom, rounded up, for the one total across held and transient
  ownership.
- **Domain output never waits for room.** Domain stages and their output
  queues are uncapped. No world exercises a domain's output flow
  control; load tests hold the named dispatch orders and the root's
  watermark, and test credit only at real edges. No smith world asks
  skein-world's `Stage` for room, and the run world's order log keeps no
  room field.
- **World settings are named inventories.** The agent memory fixture
  uses `MEMORY_CONTENT_EDGE_RECORDS = 98`, from two parallel tools,
  sixteen completion blocks and four jobs. Its wider fuzzy scenario uses
  `WIDE_CONTENT_EDGE_RECORDS = 105`, from three parallel tools with the
  same blocks and jobs. The agent world's restored-memory scenario uses
  `ROOT_MEMORY_CONTENT_EDGE_RECORDS = 283`, from one parallel tool,
  sixty-four completion blocks and four jobs. Each constant has an
  origin comment beside it.
  These settings are independent of any entrance allowance; the domain
  fact queues remain uncapped.
- **Memory returns when its use ends** (protocol/limits.md, sections 5
  and 11): in the agent's simulated world, the counting allocator shows
  the live heap falling once a compacted window's history is released,
  and no larger at the same point of each repetition of a story that
  compacts, reads jobs to their end, and opens and closes
  conversations.
- **Replay** compares traces, and digests of the state once skein's
  harness takes them.
- **The teardown invariant,** inherited from skein's world harness: a
  world that ends by itself ends with no deadline firing after the last
  word, other than io's close and retry deadlines (testing-strategy.md,
  section 6). In smith, an agent exits on its own after its answer, and
  its host sends it no signal.
- **Shipped times.** Keep and idle times, and every other time that
  decides how something ends, are the shipped values in every world,
  never ones chosen for tests (shell/settings.md, section 2). No world shortens
  the LLM connections' idle keep, or any other, so that it ends sooner:
  a world ends because its services close what they own after their
  last word, as the product does. Tiny limits are for counts and bytes.
  Simulated time makes the shipped times free.
- **The real loop, once settled:**
  - the scratch directory changed only where the scenario expects;
  - nothing is left running: the agent finished, and every child was
    reaped, each job's process group included. The test process reads
    its own unreaped children from outside the loop, a zombie awaiting
    its wait among them, through skein's shell
    (`skein/docs/design/runtime/readiness.md`, 5): Linux's `/proc` and
    macOS's libproc give the same reading (6.2).
- **Containment, with contained trees** (protocol/agent.md, sections
  2 and 3; protocol/host.md, section 3), in the real loop:
  - **writes stay in the view:** commands that try to write outside the
    writable directories, a git directory included, fail, and the
    scratch directory shows no such write;
  - **nothing outlives its tree:** commands that fork away, ignore the
    termination signal or hang carry a marker, and none is left once
    their call is answered;
  - **the environment is the configuration's, whole:** a command sees only
    the configured variables and its own pipes;
  - **the agent's tree holds the run:** a host that kills its agent leaves
    nothing the run started.

  These need cgroup v2 delegated to the user and unprivileged user
  namespaces. A test fails, saying so, where they are missing.
- **What smith holds, through its inspection adapter**
  (`skein/docs/design/testing/inspection.md`;
  `temper/docs/design/draft/failure-worlds.md`, 3.1 and 3.2):
  `smith/testing/smith-inspect`, `smith_inspect::inspect(domain:
  &smith_domain::Domain, into: &mut Inspection)`, component `smith`, over
  the domain's and its children's `pub` reads, each private table gaining
  a `pub` read documented as its adapter's.
  - **Waits:** a session's call waiting on its provider's answer, bound
    by the request's own deadline; a retry waiting out a rate window, its
    wake the window's reset; a run waiting on a tool's or a job's end.
    Each waiter is named by its run, `run N`, as its host numbers it.
  - **Tables:** the requests in flight, released by each one's answer or
    deadline; the sessions, runs and kits, released by their run's end.
  - **Called** at each world's quiet point: the agent world's drive
    once its agent reclaims, and the session, run and host worlds' own.
- **Faults, their ledger, blame for runs and the sweeps** are
  `faults.md`'s.

### 6.1 The suites

Every tier follows testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

The strategy's two suites (testing-strategy.md, section 8) make the gate:
- **focused tests:** the step tests; each world's scenarios, referee
  tests, replay, facts changing nothing and memory under scenario ceilings;
  the real loop; the end-to-end tests against fakes; and the benchmark
  harness's offline tests;
- **fuzzy tests:** each world's `tests/suite/fuzzy/`, sweeps over seeds
  under faults; the run's world also draws message timings, budgets and
  shares (domain/run.md, section 12), the session's world its usage,
  appended sizes, rendered sizes and their bytes-to-token ratios
  (domain/session.md, section 10), and the tools' world
  its jobs' polls, stops and ends among edits (domain/tools.md, section
  7).

Codec layout-plan checks run in skein's focused codec suite. Kit bounds, parsing and
landed-file mutations belong to skein's tests rather than a second codec
binary. The fixture owner's landed-data check remains here: the generated
`current` check compares generated source, not the landed-data findings.
The live suite's setup words and refresh report are step tests
(`tests/step/`, `testing-tiers.md`, 5); the unconditional `live_configuration` target keeps
the product adapter's user layer through the shipped `smith check`. Shared
live support contains no tests; its stories stay in `live-smith`.

The live tests are a separate suite, outside the gate, under the
`live-smith` profile only (`testing-tiers.md`, 3). Benchmarks are no suite: they run by their
own command (`testing-tiers.md`, 4).

### 6.2 On macOS

- **The real loop and the end-to-end tests run on macOS** as on Linux,
  in the `smith` part of `temper/docs/design/testing/macos.md`
  (section 4). Their settled check (6) holds there: every
  child of the test process, running or a zombie, has its test process
  as its parent until its wait, so macOS's listing by parent answers
  what Linux's threads' `children` files do, and no test is Linux's
  for it.
- **Linux's own tests, by name,** as skein's are
  (`skein/docs/design/runtime/shell.md`, 8): built for Linux alone,
  `cfg(target_os = "linux")` with the reason beside it, a helper only
  they use gated too, and listed here; none is dropped silently. Every
  other test runs on both systems. The benchmarks' are their own
  (`benchmarks/README.md`).

| Test | Its reason |
|---|---|
| `tests/end-to-end/program_lifetime.rs` (`smith::end_to_end`): `an_aborted_agent_takes_its_program_and_relay_with_it`, and `support/lifetime.rs`, its observation | a program dies with its aborted agent by Linux's parent-death signal; on macOS the host's tree must end (`protocol/programs-claude-code.md`, 1.1, and `protocol/programs.md`, 9) |

### 6.3 Campaigns

A campaign runs smith's fuzzy sweeps after landing with fresh seeds, in
rounds of smith's pair `campaign-smith` (`tools/tiers/layers.md`, 9.3;
`tools/tiers/campaign.md`). A sweep takes part through skein's seam
(`skein/docs/design/testing/sweeps.md`, 4.2 to 4.5), and a world adopts
it in a change of its own, its binary listed in the pair in the same
change (`temper/docs/design/draft/fuzzy-campaigns.md`, 9).

- **The tools, run and agent worlds take part:** every loop over seeds
  in their fuzzy modules takes its cases through `cases`, with its
  test's identity (`this_test!`) and its gate seeds, unchanged. A loop
  that hands some of its seeds to other settings (the run world's time
  pressure, odd stops, high spend and relay loss beside its noisy
  worlds; its slow checkouts, landings, relays and wind-downs beside
  them in the cancels' and deadlines' cells) gives each recipe a part of
  its test and that recipe's own seeds, so that the gate runs the worlds
  it ran and a campaign draws each recipe fresh. A helper serving
  several tests takes its caller's identity.
- **Their binaries,** `smith-tools-world::suite`, `smith-run-world::suite`
  and `smith-agent-world::suite`, are the pair's list.
- **Every seed a valid case** (sweeps.md, 4.5):
  - arithmetic on a seed wraps, and a moment drawn from it stays on the
    grid its gate seeds draw: the agent world's host cancels at a
    multiple of 500 ms within its forty steps at any seed;
  - every draw of a case comes from its seed, never from a stream that
    runs across the loop: the run world's drawn prices;
  - a scripted neighbour's draw keeps the order its world assumes: the
    agent world's parent sends an empty brief's first work after the
    run's admission, whatever delay it draws;
  - a fake's answer stays within the contract the product checks: the
    tools world's memory churn answers a search with no more hits,
    paths and text than its limits allow.

  A fix that moves a gate seed's draws keeps the sweep's reach at its
  gate seeds.
- **Checks over the loop wait for `gate()`:** the outcomes, refusals and
  terminals a sweep must see, its minimum totals, the cells and message
  classes it must reach, and the quota of operations each memory churn
  must run. `Sweep::check_fell` reports its reach itself.
- **A case's own checks run in every mode:** settlement, the referees,
  every call or start answered once, replay compared with its run, and
  the memory ceilings. A ceiling is a bound its scenario's limits
  promise (section 6), so a fresh seed past it is a campaign's finding,
  against the domain or the ceiling, for the user to route.
- **Kept fixed:** the focused tests and their seeds, and a fuzzy test
  with no loop over seeds (the agent world's longest child usage).
- **Counts:** each case counts a `seed` check (sweeps.md, 4.2), so a
  loop that counted none counts one a case, and a fault sweep that folds
  its ledger with `Sweep::ran` counts two; no binary's count falls.
- **The session world waits:** its loop of 1000 seeds draws its answers
  from the fake model's menu, and the misbehaving agents draft's answers
  drawn per request move that world and pick its seeds again
  (`temper/docs/design/draft/misbehaving-agents.execution.md`, A4), so
  its loop's cases and gate seeds are A4's to settle first. It takes
  part once they have landed (section 9). The agent world's loops draw none
  of their answers at random, so its seeds stand.

## 7. Layout

```
Cargo.toml                       the package smith, a member of the repository's workspace
src/main.rs                      the binary smith: glue
src/bin/smith-agent.rs           the binary smith-agent: glue
crates/*/src/tests.rs            step tests
crates/smith-*-service           the profile, and its derivation properties
crates/smith-*-shell             the shells' parsing, settings and refusals
crates/smith-local-protocol/tests the chat store's one recovery on the real file adapter (2.5)
tests/codecs                     the codecs: built values, records-only drift, bounds, fuzzy decoding
tests/tools                      the tools child domain's world
tests/session                    the session child domain's world
tests/run                        the run child domain's world
tests/agent                      the agent's root with its children, on a scripted host
tests/host                       the host world: the host domain and the inline agent, on a scripted parent
tests/local                      the local host with the agent in it: a system world
tests/channel                    protocol world: the channel's two halves
tests/protocol-llm               protocol world: the LLM component against skein's fake provider
tests/machine                    simulated world: the machine component
tests/agent-process              simulated world: the agent service
tests/hosts                      simulated world: a sample host and the agent it spawns
tests/local-process              simulated world: the local host and its peers
tests/real                       the real loop
tests/fake-claude                smith-fake-claude: the fake program a program backend runs (section 4)
tests/machine/tests/simulated/   machine seam checks and fake-kernel stories: step + simulated
tests/machine/tests/suite/       real rg children and a held job's teardown
tests/fake-claude/tests/simulated/ encodings and the hosted fake: step + simulated
tests/fake-claude/tests/suite/    the fake executables, real files and signals
tests/*/tests/suite/             a world's focused tests, apart from its split binaries
tests/*/tests/suite/fuzzy/       its fuzzy tests
tests/end-to-end/end_to_end.rs   end to end against fakes: a package target, with its modules beside it
tests/end-to-end/live.rs         the live suite: a package target
tests/end-to-end/live_configuration.rs  the product adapter's user layer through the shipped smith check
tests/step/                      offline configuration controls and the live suite's setup words: the step package target
testing/smith-end-to-end-kit/     their shared setup, compiled once
testing/<crate>-fixtures         a crate's fixtures for other crates' tests and its own integration tests
benchmarks/                      smith-bench, the benchmark harness (benchmarks.md)
benchmarks/tests/step/           offline harness checks, recorded parsers and the harness over a test tree
benchmarks/tests/suite/          real git, login shells, terminals, pipes and the shipped probe and suite commands
```

- **A crate's fixtures** (skein/docs/foundation/cli.md, 3): the small
  limits, configurations, turns and documents other crates' tests start
  from are the package `<crate>-fixtures` in `testing/`, built on the
  crate's public items, and its readers depend on it, a world's library
  as an ordinary dependency and a crate's tests as a dev-dependency.
  The crate's own step tests keep, in a `#[cfg(test)]` module, the
  fixtures they read, a copy where a fixtures crate holds the same.
  A fixture only re-exported, as `smith-domain` re-exports its
  children's, is read from its own crate's fixtures crate:
  `smith_domain_session_fixtures::turn`, not
  `smith_domain::session::fixture::turn`. smith's crates have no
  Cargo features.

- **The end-to-end and live tests are the package's `[[test]]`
  targets** (shell/README.md, section 3.1). Cargo gives a package's own tests
  the paths of its binaries, and `tests/end-to-end/` holds no manifest,
  so it is not a package of its own. The real-program probes in
  `smith-agent-process-world::live` also belong to `live-smith`.
- A world's package is `smith-<name>-world`. The world harness, the
  schedule, the referee plumbing and the counting allocator are skein's;
  smith keeps only its worlds, its scripted neighbours and its scenarios.

## 8. Open questions

- **The real loop in CI:** a kernel that allows `io_uring`, with git, rg and
  sh installed (skein's `runtime/shell.md`, section 9).
- **Containment in CI:** cgroup v2 delegation and user namespaces in the
  CI's containers.
- **The sample host in the real loop:** whether a second host earns a
  real loop of its own, beside the local host's, now that an end-to-end
  story spawns `smith-agent` from it.
- **A reacting person in skein:** whether a scripted neighbour that acts
  on what it observes is generic enough for skein's world kit. It moves
  there when a second service needs it.
- **Both placements within the budgets:** which scenarios run twice once
  the inline agent is the product, and whether the spawned placement's
  share moves to the fuzzy sweeps.

## 9. Later

- **A live red no offline world shows** (testing-tiers.status.md, 3): its
  repair starts from the first after-landing log whose failure says why
  (`testing-tiers.md`, 3), in the lowest tier that shows it.
- **The agent, run and host worlds' edges** (testing-tiers.status.md): each
  takes fakes of its own, held to the code by a protocol or simulated
  world as the session and tools worlds' are, or says why it can't.
- **The session world in campaigns** (6.3): its loops through skein's
  seam and its binary in `campaign-smith`, once the misbehaving agents
  draft's answers drawn per request have landed.
- **smith's other worlds in campaigns:** the host, local, protocol and
  simulated worlds' loops, each world once its draws are valid at any
  seed.
- **The key's story after landing.** The story on an Anthropic API key
  is the owner's for now (`testing-tiers.md`, 3): no key is among the after-landing tier's
  inputs. It returns to the after-landing tier when a key becomes one of
  them, which is the user's to give.
