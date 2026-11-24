# Testing skein

Provisional, 2026-10-03, revised 2026-10-30. How ../../foundation/testing-strategy.md
applies to skein itself: which tiers skein has, what they run, the
example services and the minimal fake machine they need, where the tests
live, what skein supplies for a service's own tiers, and where things
stand. How each part is tested in its own tier is in that part's
document. The conformance suite, which holds the simulator and each
backend to the kernel boundary's contract, is
[`conformance.md`](conformance.md). The shared LLM, forge and checkout
fakes are indexed in [`fakes/README.md`](fakes/README.md).

- [`service-tiers.md`](service-tiers.md): what skein supplies for a service's tiers.
- [`layout.md`](layout.md): test crates, shared harnesses and suites.

## 1. In one page

- **skein tests the kit.** Its tiers stop at io under a small example
  service, the echo server and its fake client; and at a
  browser under the UI example, a small application on skein-ui. A
  service's domain, protocol layer and fakes are tested in the service's
  own tiers, which stand on skein's and do not retest them.
- **Each part has its tier:** lib in step tests, each machine in machine
  worlds, the machines stacked in protocol worlds, io in io worlds, the
  loop in simulated worlds of the examples, and the ring in the real loop
  and the conformance suite.
- **The examples are the template.** skein's simulated worlds and real
  loop run them, and a service's worlds copy them.
- **skein ships no fake machine.** Its own io and simulated worlds use a
  minimal one, which stays that small.

## 2. skein's tiers

| Tier | In skein | Its design |
|---|---|---|
| step tests | lib's containers and value types; each crate's step functions | ../runtime/lib.md, 10 |
| machine worlds | HTTP, server-sent events, JSON, each side alone; the TLS client, against rustls's server in memory | http.md, 6; ../data/json.md, 6; ../net/tls.md, 5 |
| protocol worlds | an LLM client's HTTP, server-sent events and JSON against a server's; skein-llm's subscription client against independent wire scenarios | http.md, 6; ../llm/README.md, 5 |
| io worlds | io over the simulator, with a scripted owner | ../runtime/io/README.md, 5 |
| simulated worlds | the examples' `iterate`, each a process of the simulator | section 3 |
| real loop | the examples under the shell, on the real kernel | section 3 |
| beside them | the conformance suite, against the simulator and the ring | conformance.md, 1 |
| beside them | the simulator's own tests; the ring's own tests | simulator.md, 6; ../runtime/shell.md, 8 |
| durable store | `skein-kv` step tests; `tests/kv` over the crashing fake disk and the real ring | ../data/kv.md, 9 |
| browser tier | the browser kit against real Chromium; the UI example's bundle in Chromium, through the person's browser face and harness | browser.md, 9; ../ui/web/testing.md, 1 |

TLS is tested on its own (../net/tls.md, 5), and the replaying tiers run in
plaintext.

For a domain world, `skein_world::domain::Stage::uncapped(limits)` owns
its environment, FIFO inbox and uncapped output queue. The world calls
the domain for each event and due alarm without reserving output room;
the stage holds neither back. Incoming work is bounded by the world.

## 3. The examples

A small service in `examples/`: the echo server, with its fake client.
It exists to be tested, and to be copied.

- **In simulated worlds,** it runs as a process of the simulator, with
  every layer real. It is the first tier where the loop runs, so it tests
  what the loop adds: the two passes, room checked at the kernel's queues
  and new work held at the root's watermark (../../foundation/programming-model.md, 7.1),
  deadlines across layers, and requests made in the down pass reaped in
  a later iteration.
- **In the real loop,** it runs under the shell on the real kernel: the
  ring, loopback sockets, a scratch directory as the root, real child
  processes and the real clock. It shows what only the real kernel
  can: the ring adapter's `unsafe`, the probe at startup, signals read
  from a signalfd, and a process tree that ends.
- **As a template,** a service's simulated worlds start from its.

The UI example (../examples.md, section 9) is an application's rather than
a service's: a view on skein-ui, its browser shell and its server. Its
view and its server are tested in step tests, and the three together in
the browser tier (../ui/web/testing.md).

Their design, the steps before code of each, their fakes and the harness
their worlds run on are ../examples.md. The echo has a domain, a protocol
layer, a service and a `main`, its fake client, and its worlds in
`tests/echo`, over the world harness, `skein-world` (../examples.md, 6).

## 4. The minimal fake machine

The simulator plays the kernel, not what a program does: that is the
embedder's fake machine (simulator.md, 3). skein's own io and simulated
worlds need a minimal one:

- a few files beneath a root;
- a program that echoes its input, one that exits with a given status,
  and one that never exits;
- for a spawn's reach, the probe, which reports from inside a tree what
  the tree may reach.

It stays that small. A service's fake machine is the service's, built
on it or beside it.

It is `skein-fake-machine`, in `crates/`: a playground ships it in its
world, as it ships the simulated world (world.md, 2). A world lays a
root in it from a scenario's items (files, directories, symbolic links,
each with its mode) and gives its handle to a process as the shell would
a root opened at startup; after each submit, the world takes the
simulator's calls and has the machine answer each (simulator.md, 3.1).
The machine resolves paths beneath a root as `openat2` with
`RESOLVE_BENEATH` does, follows the links that stay beneath it, keeps its
owner's permissions, and refuses what a real filesystem refuses, in the
order Linux checks; the conformance suite holds it, through the
simulator, to the real kernel in a scratch directory. It keeps a step
machine's shape, a call in and an answer out, in a vocabulary of its own
that a face translates to and from the simulator's. Its programs are
the fixture of the conformance suite's processes, and its crash model
keeps what a crash keeps (simulator.md, 3.3). It
keeps each file's lock with the handle that took it (simulator.md, 3.1).
A test reads back what it laid, or what the code under test left there,
without the simulator: each entry's kind, mode, owner and bytes, a link's
target, and the node a handle names, so that a file replaced by another
with the same bytes and mode shows as replaced.
For a tree's metadata (kernel-files.md, 2) it keeps each entry's inode,
group, mode up to `0o7777`, times and attributes, and hard links, and
lays FIFOs; it has no devices, no sockets and no btrfs. It answers every
subvolume call `NotSupported`, and its status a filesystem that is not
btrfs, as Linux does off btrfs, so skein's btrfs scenarios run on the
ring alone (conformance.md, 1). A service whose fake machine models btrfs,
such as hearth's, holds that machine to those scenarios itself.

For trees (../runtime/io/processes.md, 2 to 4; simulator.md, 3.4) its programs add: one
that starts a descendant which leaves its group (`setsid`) and outlives
it; one that forks a child each time it is signalled; and the probe,
which for a contained tree, on the ring, also writes its ids and groups,
its hostname, its network's interfaces, the processes in its `/proc` and
its rlimits, so that a conformance scenario checks a contained tree from
inside it (conformance.md, 1).

**The probe,** for a spawn's reach (../runtime/io/processes.md, 6), is
the machine's program and a fixture of the real suite alike, and answers
every question a scenario asks in one run. It reports on its output
pipe, one answer for each question its arguments ask:

- for each path its arguments name, relative to its working directory:
  whether it can read the file, write it, list the directory and execute
  the program, each asked by name;
- its `no_new_privs`;
- the ids a set-id program it runs has: its own kept, or others gained;
- whether it can signal its parent;
- the descriptors it holds;
- and, asked to nest, the same report from a probe it starts beneath its
  own reach with a narrower one: a rule relative to its working
  directory with its rights, plus the machine's rules it was handed to
  pass on, which on the ring let the nested probe run at all.

An answer is `granted`, or the error the kernel answered (`Permission`,
`NotFound`, or another by its number). One module,
`skein_fake_machine::probe`, writes the probe's arguments and reads its
report; the machine's probe, the real suite's fixture and the scenarios
(conformance.md, 1) all use it, so the three agree on what was asked and
answered. On the ring, the probe is a mode of `tests/conformance/real`'s
`signal_sender_fixture` binary, chosen by its first argument,
`reach-probe`. Its `main` hands its arguments to the helper's dispatch
first, so it is also the helper (`Kernel::set_helper`) of the spawn that
starts it and of the one it starts to nest (kernel-cgroups.md, 1).
The machine selects its program by its name's last component,
`reach_probe`, as it selects its other programs.

**The machine honours a reach.** A world sets the machine's Landlock
(`Machine::set_landlock`, taking `Landlock::Abi(u32)`, `Absent` or
`Disabled`), ABI 6 unless it says otherwise. A spawn with a reach comes to
the machine with each rule's root as the machine's handle (simulator.md,
3.4), and the machine answers it as Linux would:

- **Refused, naming the reach,** as the kernel refuses it, checked in
  this order: `Absent`; `Disabled`; `Abi { found, needs }`; a rule whose
  path does not resolve beneath its root, `Rule { index, errno }`, with
  the error number of the machine's refusal (2 for `NotFound`). Nothing
  runs.
- **Otherwise run,** with the reach a layer of the child's: whatever the
  child starts, the nested probe among them, holds its parent's layers
  and its own.
- **What a restricted program reaches:** an access to a node is granted
  where every layer grants it, on the node or on a directory above it
  that a rule names, along the node's ancestry beneath its root once its
  path is resolved, links followed. The lookup's own refusals come
  first, so a missing name is `NotFound`, as Linux tells; Landlock's
  denial is `Permission`, and the owner's permissions still apply.
  Reading a file takes `READ_FILE`; listing a directory, `READ_DIR`;
  writing a file, `WRITE_FILE`; creating one, `MAKE_REG` on its
  directory; executing a program, `EXECUTE` and the owner's execute bit.
- **The rest of a reach:** its `no_new_privs` is set, so a set-id
  program keeps the probe's ids (the machine keeps no set-id bit and
  needs none for this); a signal to a process outside the tree, its
  parent among them, is refused `Permission`; and it holds the
  descriptors its spawn named and no other.
- **Only the machine's programs are restricted.** A service the world
  hosts is not the machine's program: it runs as the simulator plays it,
  unrestricted, and the reach it was spawned with stays in its spawn's
  call for the world to check (simulator.md, 3.4).

A separate dependency-free kit, `skein-fake-checkout`, supplies generic
byte-path files, scripted commands and local git mechanics for domain
worlds (fakes/checkout.md). It has no simulated-kernel adapter, inode or
permission model. Services retain their policy, remote implementations,
clock, cancellation and delivery ledgers; the kit answers synchronous
mechanical operations only. It lives in `crates/`, as temper's `temper
fake git` ships it, and its focused leaf tests and small replay sweep are
the crate `tests/fake-checkout`, by the rule below.

A code forge has fakes of its own (fakes/forge.md). `skein-fake-forge-domain`,
a step crate on skein-lib alone, plays a forge's state, CI, merges and git,
and `skein-fake-forge-forgejo` serves Forgejo's API over it on skein's HTTP
server. The forge peer in `skein-fake-peers` hosts that layer on loopback
io. Its GitHub layer (`skein-fake-forge-github`) shares the domain and
hosts API/browser routes on one listener and signed job logs on a second
blob listener, with one shared connection limit. `skein-throwaway-forgejo`
runs the pinned Forgejo in a scratch directory, for the tools and process tiers that need a real one, and a
conformance tool outside the suites holds the dialect to it. A service
keeps its fake of its own git component, its worlds and its cost ceilings.

## 5. What skein supplies for a service's tiers

Moved to [`service-tiers.md`](service-tiers.md), section 1.

## 6. Layout

Moved to [`layout.md`](layout.md), section 1.

## 7. Where things stand

How each part is tested, and what its worlds cover, is in the part's own
document; the echo's worlds are ../examples.md, 7.

Every tier follows ../../foundation/testing-strategy.md, 8.1: CPU budget, wall hard limit,
starved warning and the three-question repair rule.

The two gate suites use the root `.config/nextest.toml`'s `default` and
`fuzzy` profiles, with a CPU budget for each of skein's layers in
`tools/tiers/budgets.toml` (`tools/tiers/budgets.md`, 2 and 6) and ideals of
40 s and 120 s, enforced at the merge gate under
../../foundation/testing-strategy.md, 8.1. The browser kit against real
Chromium and the UI example's browser tier use `browser-skein`
(`tools/tiers/run skein browser`; browser.md, section 9; ../ui/web/testing.md,
1); their four-minute global timeout is a wall
backstop, separate from their CPU budget. skein-llm against real endpoints
uses `live-skein`, with a key or a plan's token from the environment.
These two slow tiers stay out of the gate, each with the user's CPU
budget and wall hard limit under 8.1, in `tools/tiers/budgets.toml`. Schema, value, fingerprint, diff, patch, redirect
admission and fake-browser checks run in focused step tests and worlds;
seeded decoder comparisons, sign-in faults including restart, and the
forge machine world's every-byte TLS sweep run in fuzzy.
Measurements use the tier's profile and keep the same budget rule.

The exhaustive wall-day round trips and Gregorian phrase comparisons,
and HTTP response and request prefixes compared with a reference, belong
in fuzzy (../../foundation/testing-strategy.md, 8). Focused keeps named calendar edges and
incomplete-event behaviours. UTF-8's longer sequences use their range
boundaries in focused; random interiors go through the public writer in
fuzzy. The private Huffman builder's comparison keeps four seeds, two per
rarity regime, with its completeness, canonical decoding, width and
reference-weight checks. Scenario counts follow what their assertions
need: a ring still wraps, a store remains larger than its ceiling, and
every asserted close-state wait still occurs. Every retained turn of a
memory test stays metered.

Replay: a seed replays to the same trace of submissions and completions,
and a JSON world, an HTTP world, an io world and an echo world to the
same run; a TLS world does not, as rustls draws from the kernel. The
state digests replay with the trace (lib-containers.md, 1.1; `service-tiers.md`, section 1).

Memory: each part's peak is measured with the counting allocator in its
own tier, a call of an entry point at a time, under its scenario's named
ceiling with headroom; the echo's simulated worlds check each host at
every iteration (../examples.md, 6). A focused test drives the echo to its
edge limits and shows it by what it holds: every connection's intake,
receive and output full at once. Reaching the limits is checked by those
contents, never by nearly filling a formula (../../foundation/testing-strategy.md, 6).

The forge's worlds (`layout.md`, section 1) keep within a budget of their own: a step
test under 40 ms, a focused world test under one second, and a fuzzy
run within 10 seconds, its seeds included, so that `tests/forge` stays within half of each suite's.

## 8. Not built yet

What of this document is not built is in [`README.status.md`](README.status.md).

## 9. Open questions

- **One-process adoption and suite costs** (5, 6, 7). Switching a
  caller's focused and fuzzy steps is that caller's separate change,
  tree by tree (`tools/tiers/design.md`, 8). Measure suite CPU per
  process after that switch; per-test CPU in a shared process is not
  measured. A run that checks outcomes establishes no CPU margin.
- **Further scenario and product costs** (2, 7). KV's CRC-32C and row
  copying, LLM replay conversion, real TLS and RSA, HTTP byte and chunk
  decoding, protocol generation, conformance pattern generation, GitHub
  transcript and secret checks, and fail-stop symbolization retain work
  that the scenarios require. Further changes must preserve the same
  bytes, refusals, reports, waits and ceilings. Request-table admission
  scans, LRU operations, KV duplicate-key scans and file observation,
  Huffman's reference sorting and the remaining fixture work also have
  identified costs; the measured candidates below the repair's roughly
  0.05 CPU s stop rule stay. The per-row instruction pairs, causes and
  requirements for a further fix are in
  `skein/docs/development/skein-limits-heavy.md`. Shared helper and JSON
  costs recorded there overlap separate repairs; their original counts
  are not measurements of what remains after those repairs.
- **Complete process-world instruction measurements** (2, 5, 7).
  Profiling the process operations needs a profiler that supports them;
  Callgrind's unsupported operations leave incomplete counts. A native
  CPU observation is no instruction comparison. Containment, observation,
  deadlines and the fixture starts each behaviour requires stay.
- **Process startup cost** (6, 7). Isolating the cost before a test's
  body needs separate measurements. Further table tests must still be
  cases of one behaviour under ../../foundation/testing-strategy.md, 8, keeping every
  assertion; a test past its guide cannot be merged away to make room.
- **Further folds and row diagnostics** (6, 7). Different behaviours and
  process-local heap measurements stay separate, with fixtures built
  per row wherever sharing could change what is shown. Signing-heavy
  App JWT routes stay separate when their combined table exceeds the
  unit-test guide. A table reports its first failing row and stops
  there (6): collecting every row's failure would catch a panic
  (../../foundation/programming-model.md, 10.4). Default: no
  fold weakens these rules to reach a suite-margin target; further cost
  changes need a profile of their cause first.
- **The counting allocator's remaining cost** (5). One thread-local
  access, its cells, layout construction and preconditions, allocation
  numbers, size conversions and System allocation remain necessary.
  Whether to add a `[profile.dev.package.skein-heap]` override for the
  remaining unoptimized work is the user's decision under
  ../../foundation/testing-strategy.md, 8. The instruction counts and each cause's share
  are in `skein/docs/development/world-costs.md`.
- **Deployment-world allocation and fork costs** (5, 6). Required checked
  allocation-header layout and the kernel's fork, wait and copy-on-write
  work remain. User-space instruction counts do not measure that kernel
  floor. `temper/docs/development/limits-worlds.md` records the shares
  and native system CPU; its different profiling arms do not establish
  an isolated remaining saving or a whole-suite total. A build override
  remains the user's decision under ../../foundation/testing-strategy.md, 8.
- **Costs beyond the shared world helpers** (5, 6). Product work, a
  world's own fixtures and process startup need separate cost work;
  `skein/docs/development/world-costs.md` gives their measured shares.
  Further JSON string work belongs with ../data/json.md, 8. Required scheduling,
  trace observations and Unicode matching remain; the account identifies
  no further removable helper gain of about 1% of a measured test.
- **Heap handed between services in one thread.** A world checks each
  service under its scenario's ceiling by metering its own calls and
  harness operations on its containers (simulator.md, 5), which holds
  while nothing one service allocates is freed by another: true over the
  kernel boundary, whose backend hands every buffer back. A world that
  joins two services' layers in-process, passing a box from one to the
  other, would need the hand-off counted.
- **The real loop in CI and sanitizers on the ring adapter** are the
  shell's (../runtime/shell.md, 9).
- **A fixture a crate's unit tests and a world both read** (6). No `#[path]`
  crosses a package, and a kit would give a second copy of the crate's types,
  so the dialect world holds its own
  (`skein/tests/forge/dialect/tests/common/admitted.rs`) beside the crate's. No
  feature can join them (../../foundation/cli.md, 3): a fixtures crate,
  `skein-forge-fixtures`, would hold the world's copy, as
  `skein-forge-connection-fixtures` holds the machine world's
  (../forge/connection.md, 10), for 55 lines of test data each side keeps as
  its own numbers. Default: the copy stays.
- **The echo's client host** (6): `ClientHost` in
  `skein/examples/echo/shell/tests/end_to_end.rs` repeats `Proc`'s `Host` arms.
  The test sits in the shell's package so that Cargo supplies its binary,
  `skein-echo-client` is held to the step subset and can't implement `Host`, and
  a kit's manifest alone is longer than the copy. Default: the copy stays.
- **Helpers beside the forge kit's** (6):
  `skein/tests/fake-peers/tests/forge/fixtures.rs` and
  `temper/tests/engine/protocol-forge/src/fixtures.rs` repeat some of
  `skein_forge_kit::fixtures`. Default: they stay, until a debt item names them.

## 10. The portable target

A playground runs a service's simulated world in a browser
(../playground.md), so what that world links builds for
`wasm32-unknown-unknown` as well as natively.

- **What builds there:** lib, io, the simulator, the shell's portable
  face (../runtime/shell.md, 13), the simulated world and the fake machine, the
  fakes a world hosts (the LLM's and the forge's domains and protocol
  layers, the peers, GitHub's layer), the key-value store, HTTP, JSON,
  the channel, the codec, TLS, and the playground's kit (../playground.md).
  The counting allocator, the harness's referee and real loop, the
  browser kit, the conformance suite, the fork kit and the scratch directory
  do not: they are tests', and native.
- **Fenced by target:** what reads the system is under `cfg(unix)` where
  it sits, never behind a feature (../runtime/shell.md, 13).
- **No TLS there.** A world draws every choice from its seed and runs
  in plaintext; on this target `skein-tls` refuses every configuration,
  session and signature (../net/tls.md, 4), so no path of a world reaches
  ring's randomness, and one that tries fails at once. The peers serve
  plaintext alone there.
- **Checked in the gate** with clippy's lints as errors on this target,
  over every crate of the list above: a native item reached from the
  portable face fails the build, and a fence that leaves code dead warns.
  Nothing runs there in the gate: the simulated world's behaviour is the
  same code natively, which its tests run; a playground's browser tier
  runs it in Chromium (../playground.md, 8).
