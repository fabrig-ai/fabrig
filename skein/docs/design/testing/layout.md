# Testing skein: layout

## 1. Layout

```
crates/*/src/tests.rs           step tests; lib's, io's, JSON's, HTTP's and TLS's in a module per area, under src/tests/
crates/skein-ui                 the UI kit (ui.md): its step tests, and its golden scenes beside them
crates/skein-ui-text            the text shell (ui-text.md): its step tests, and its golden text beside them in src/tests/golden/
crates/skein-ui-screen          the full-screen shell's renderer (ui-screen.md): its step tests, and its golden frames beside them in src/tests/golden/
crates/skein-ui-screen-terminal the full-screen shell's keys, drawing and terminal (ui-screen.md): their step tests
crates/skein-ui-web             the browser parts (ui-web.md): each part's pure half tested on the host in its step tests
crates/skein-ui-bundle          the bundler, a tool built optimized (../ui/web/bundle.md, 2)
crates/skein-fake-llm-*         the scripted LLM peer: its domain and its byte peer (fake-llm.md), a product the worlds and the real loop use
crates/skein-fake-forge-domain  the fake forge's domain (fakes/forge.md, 2 and 3)
crates/skein-fake-forge-http    what its two layers share (fakes/github.md, 3)
crates/skein-fake-forge-forgejo its Forgejo protocol layer (fakes/forgejo.md, 1)
crates/skein-fake-oauth         an independent rotating OAuth issuer (../oauth/README.md, 5)
crates/skein-fake-peers         the LLM, OAuth and forge peers hosted on loopback io, in a world or as an application's own process (fakes/forge.md, 5); links skein-fake-forge-github for GitHub's API/browser and blob faces
crates/skein-fake-checkout      files, scripted commands and local git for a service's domain worlds, and for a service's fake git as a program (fake-checkout.md)
crates/skein-fake-forge-github  the fake forge's GitHub API/browser and signed-log blob layer (fakes/github.md, 1), which the peers link
crates/skein-fake-machine       the minimal fake machine: files beneath a root, its programs, and its face behind the simulator (section 4)
crates/skein-sim-world          the simulated world: processes hosted over the simulator and stepped, which the harness and a playground run (world.md)
testing/skein-conformance       the conformance suite: the backend interface, the scenarios, the driver, the checks
testing/skein-heap              the counting allocator, the meter that checks a step under its scenario's ceiling, and the span a world meters its processes with
testing/skein-scratch           a scratch directory beneath the system's temporary one, removed when dropped, for tests of files on the real kernel; and the paths a test finds when it runs
testing/skein-fork              a run forked at a chosen moment of a test and waited for, so that a sweep of cuts holds its prefix once
testing/skein-rows              one table test's rows: each named as it runs, and the first failure ends the table (../../foundation/testing-strategy.md, 8)
testing/skein-world             the world harness: the simulated world's run with the referee, the trace and the heap, and the real ring's loop
testing/skein-echo-client       the fake echo client, a step machine
testing/skein-throwaway-forgejo the pinned Forgejo run on loopback in a scratch directory (fakes/throwaway.md, 1)
testing/skein-forge-conformance the conformance records the forge tools share, and their secret scans (fakes/throwaway.md, 2)
testing/skein-fake-channel      a scripted framed peer for a service's protocol worlds (channel.md)
testing/skein-protocol-kit      what the protocol worlds share: a demand's reads and the draws below them, the sweep's tally, the transcript loader
testing/skein-json-kit          what a second crate takes of the JSON world: the reference reader, the generator and their outcome types
testing/skein-http-kit          the HTTP world's reference reader
testing/skein-tls-kit           the TLS world's certificates, in-memory server, byte driver and step meter, over tests/tls/fixtures
testing/skein-llm-kit           the LLM world's wire fixtures and its faces of skein's fake LLM
testing/skein-fake-peers-kit    shared peer configurations and simulated socket helpers
testing/skein-forge-kit         the forge machine world's machine, peers, pipes and joint world, and its GitHub face
testing/skein-forge-connection-fixtures the forge connection's fixtures for the machine world's hand, a copy of the crate's step tests' own (../forge/connection.md, 10; ../../foundation/cli.md, 3)
testing/skein-codegen           the codec generator, run by a consumer's tests (codec.md)
testing/skein-browser           the browser kit, a step machine: headless Chromium over the DevTools protocol on a pipe (browser.md)
testing/skein-fake-person       the person: targets by role, name and text, steps, the tree face, the random person (../ui/testing.md, 1.1)
testing/skein-fake-person-browser the person's face over the browser kit, and the harness: an application's server binary, Chromium and the peers a test lends in one loop; skein's stories for a browser shell (../ui/web/testing.md, 2)
tests/heap                      the counting allocator's own tests, skein-heap-tests
tests/echo                      the echo's simulated worlds and its real loop, skein-echo-world
tests/io                        io worlds: io over the simulator, a scripted owner, a referee, skein-io-world
tests/lib                       lib's comparisons with naive functions, run long, and its allocation prices against the counting allocator, skein-lib-tests
tests/sim                       the simulator's own tests, skein-sim-tests
tests/world                     the world harness's own tests, over scripted processes of raw records, skein-world-tests
tests/ring                      the ring adapter's own tests, skein-ring-tests
tests/readiness                 the readiness backend's own tests, on each system, skein-readiness-tests
tests/conformance/sim           the suite against the simulator, skein-conformance-sim
tests/conformance/real          the suite against the real kernel's backends, the ring and the readiness backend, skein-conformance-real
tests/json                      the JSON tokenizer's machine worlds, the writer against it, and their peaks under scenario ceilings with the counting allocator, skein-json-world
tests/json/transcripts          its transcripts, each with what it must decode to
tests/http                      the HTTP client's and server's and the event stream reader's and writer's machine worlds, each read by a reference reader of `skein-http-kit`, the client and the reader stacked with JSON, and their peaks under scenario ceilings with the counting allocator, skein-http-world
tests/http/transcripts          its transcripts, responses and, in requests/, requests, each with what it must decode to
tests/protocol                  the protocol worlds: an LLM client's stack against a server's, joined by bytes cut at random, skein-protocol-world
tests/llm                       skein-llm's subscription protocol worlds, fragmentation and fault sweeps, and bounds against the counting allocator, skein-llm-world
tests/llm-connection            the LLM connection component's worlds, skein-llm-connection-world
tests/oauth                     the OAuth client against the independent issuer, skein-oauth-world
tests/oauth-accounts            the OAuth driver's worlds, its owner scripted, skein-oauth-accounts-world (oauth.md)
tests/oauth-issuer              the client against the OAuth issuer, skein-oauth-issuer-world (oauth-issuer.md)
tests/forge/machine             the forge component's machine world and the listener's, against a scripted HTTP peer and the fake's GitHub layer on its clock, and their peaks under scenario ceilings with the counting allocator, every holder filled (../forge/connection.md, 10)
tests/forge/protocol            the protocol world: the client and the component against the fake forge's protocol layer and domain, joined by streams cut at random, under a scripted durable owner that restarts; its fuzzy runs take seeds (../forge/connection.md, 10)
tests/forge/client              the client world: the client alone under a scripted parent, over the fake forge's domain (../forge/client.md, 10)
tests/forge/setup               the setup world: the host component over a store of items and the fake forge's setup routes and GitHub layer, through HTTP bytes, and its peak under a scenario ceiling with the counting allocator (../forge/setup/README.md, 5)
tests/forge/dialect             both dialects' documents, plans and bounds driven from outside, with their fuzzy mutations, skein-forge-dialect-tests
tests/forge/conformance         the conformance tool, outside the suites: a throwaway Forgejo driven, each fact the dialect relies on recorded (fakes/throwaway.md, 2)
tests/forge/github              GitHub's conformance tool: the hosted fake and owner fixtures (fakes/github.md, 2); tests/suite is github-conformance::suite, simulated (the hosted fake over the simulator, its directories in the fake machine), and the library's own tests hold its checks that drive no runner
tests/fake-forge                the fake forge's domain, its Forgejo layer and its GitHub service against the counting allocator, skein-fake-forge-tests
tests/fail-stop                 a panic on any thread aborts the process after its report, through a fixture binary, skein-fail-stop-tests
tests/fake-peers                the LLM, OAuth and forge peers on loopback io against their clients, including GitHub's job-log redirect to its blob listener, skein-fake-peers-tests
tests/fake-checkout             the fake checkout's leaf tests and its replay sweep against a small private remote, skein-fake-checkout-tests
tests/channel                   the framed channel's machine and protocol worlds, skein-channel-world
tests/codec                     a generated sample codec and its worlds, skein-codec-tests
tests/kv                        the store over file io, the simulated kernel and a crashing fake disk, skein-kv-tests
tests/browser                   the browser kit's machine world against a fake browser, its tests against real Chromium, and its peaks under scenario ceilings with the counting allocator, skein-browser-world
tests/tls                       the TLS client's machine worlds against a rustls server in memory, the HTTP client stacked on it, and its peaks under scenario ceilings with the counting allocator, skein-tls-world
tests/tls/fixtures              its certificates and key, and the script that makes them
tests/link                      a link's close rule against a rustls server in memory, skein-link-world (../net/link.md, 4)
tests/ui                        skein-ui's peaks under scenario ceilings with the counting allocator, and its comparisons with naive functions, skein-ui-tests (../ui/testing.md, 1)
tests/ui-text                   the text shell's terminal on the simulated machine, skein-ui-text-tests (../ui/text.md, 8)
tests/ui-screen                 the full-screen shell's terminal on the simulated machine, skein-ui-screen-tests (../ui/screen.md, 9)
tests/ui-web                    the browser parts' host tests that need more than a crate's step tests, and the UI example's browser tier, skein-ui-web-tests (../ui/web/testing.md)
tests/**/tests/suite/*.rs             a crate's focused tests
tests/**/tests/suite/fuzzy/*.rs       its fuzzy tests: sweeps over many seeds
tests/clippy.toml               what the crates under tests/ may not use
examples/echo/*                 the echo: its domain, protocol layer, service and shell, a crate each (../examples.md, 2)
examples/ui/*                   the UI example: its view, its browser shell and its server, a crate each (../examples.md, 9)
```

**A world's library is its own.** No package, whichever tree it's in, depends on a package under `skein/tests/`: what a second crate needs from a world lives in a kit under `testing/` (../../foundation/testing-strategy.md, 7), and the world's own suite reads it through its library, which re-exports the kit. No `#[path]` under `skein/` reaches outside its package. `tests/boundary/tests/skein_worlds.rs` holds both, for `skein/` alone: it doesn't read the worlds under smith's and temper's `tests/`, and packages still depend on those. It reads the sources in one walk (`tools/crates/design.md`, 3): git lists the tracked Rust sources once and each is read once, the `#[path]` rule taking those under `skein/` and the seam's rule (../forge/connection.md, 10) every one, and a source both rules parse is parsed once. A source git doesn't track is in neither rule.

`crates/*` keeps step tests in `src/`, not `tests/`; outside tests are
crates under `tests/`, named by path. Fakes follow this too: the forge
domain's memory test, the peers' tests and the checkout's are there;
protocol fixtures are in `src/tests/`. A test crate is ordinary Rust
(../../foundation/programming-model.md, 10.2). Its binaries share its
`src/` library: the simulator's harness and scripted exchange, a suite
backend, lib's naive comparisons, a machine's world and, through its
re-exported kit, the reference parser it is checked against,
and its tests share the `suite` binary: focused modules under
`tests/suite/`, seeded sweeps under `tests/suite/fuzzy/`. Mixed binaries split within their package:
`tests/<layer>/main.rs` holds tests that replay at their highest layer;
`suite` keeps real tests. Module paths and `fuzzy::` stay
(`tools/tiers/layers.md`, 6.1).
`tests/clippy.toml` bans what would
make a run unrepeatable: hash maps with a random seed, the system clocks,
threads. Each backend of the conformance suite is implemented beside the
tests that run the suite against it. The global allocator a binary
declares covers the whole binary; memory modules are `tests/suite/memory.rs`, and `tests/suite/fuzzy/memory.rs` for a sweep
under scenario ceilings (../../foundation/testing-strategy.md, 8). Each binary that checks
memory declares the counting allocator once. `Memory` selects whether a
world meters its processes; a simulated world checking memory meters
every iteration.

What a crate under `testing/` holds is shared by the tests of more than
one crate, and is ordinary Rust on a lint profile, each with a
`clippy.toml` of its own, or the step crates' (`tools/lints/design.md`, 3).
`skein-heap` needs `unsafe` for its global allocator, an `unsafe impl`.
../../foundation/programming-model.md (1, 2.1, 10.2) confines `unsafe` to the ring adapter
in code a service runs, and in tests to this allocator and the fork kit's
process boundary below. Both are test-only crates, never linked into a
service. The allocator allows its `unsafe` in
place, with a scoped
`#[expect(unsafe_code, reason = "…")]` and a `SAFETY` comment on each
block.

**A sweep's fork** is `skein-fork`, on the `ring` lint profile
(`tools/lints/design.md`, 2.4). One documented module holds its scoped
`unsafe` for `fork`, `waitpid` and `_exit`. A single-threaded test world
calls `split` at a cut, after stdout and stderr are flushed. The child
continues with its owned state; its first panic prints through the panic
hook and exits 101 without unwinding into libtest. `passed` exits a child
with code 0 and returns in the parent. The parent reaps its child and
receives `Exit::Code` or `Exit::Signal`; a child cannot fork again. Each
parent waits for each child, so the suite's CPU includes its descendants
(../../foundation/testing-strategy.md, 8.1). The kit supplies the process boundary alone;
the consuming harness takes the crash and checks every cut.

**A table test's rows** are `skein-rows`, on the `harness` lint profile.
`skein_rows::check(rows, body)` takes rows of `(&'static str, R)` and a
`body: Fn(R)`, and runs the body on each row in order, naming the row
before it runs: a line `skein-rows: row <name>` on stderr, through the
test harness's capture, which the runner prints for a failed test.
Nothing catches a panic (../../foundation/programming-model.md, 10.4),
so the first failing row, an `assert_eq!`'s panic as any other, ends the
table and its test, its name the last line before its message, and the
rows after it don't run. Rows carry data (../../foundation/testing-strategy.md, 10): the kit
hands a body the row's data and never its name. Fewer than two rows, or
two rows of one name, panics before any row runs: the caller's bug, an
assertion no test triggers. A test
that reads what an earlier row of its process would disturb, such as a
heap ceiling, stays a test of its own. The kit shares nothing between
rows; a fixture a body builds is built per row. The kit has no helper
that expects a panic, as no kit has: a row that would panic tests an
assertion, which goes, or a refusal, which its body reads as a value.

**A test's paths** are `skein-scratch`'s `paths`, read when the test
runs (../../foundation/testing-strategy.md, 8): its package, its tree,
its workspace, the target's scratch space and the binaries it starts.
A binary of the test's own package is `paths::binary(name)`, from
`CARGO_BIN_EXE_<name>`, which cargo and nextest set for that package's
binaries alone. A binary of another package, which the tier builds
first with `cargo build -p <package> --bin <name>`
(`tools/tiers/design.md`, 2.1), is
`paths::built(package: &str, name: &str) -> PathBuf`: the file `name`
in the nearest directory above the test's executable that holds a file
of that name. Cargo puts that binary in the target's profile directory
and the test's executable somewhere below it, in `deps/` or, with the
pinned nightly's cargo, in `build/<test package>/<hash>/out/`, so no
count of directories finds it; a directory of that name, such as
`build/<name>/` for a package named as its binary, is passed over.
Where no directory above holds the file, `built` fails naming the
test's executable and the command that builds it. Every world that
starts another package's binary takes it from `built`: jig's and
temper's browser worlds, skein's browser suite and the UI example's
tier. None counts directories up from its executable. A kit under a
tree's `testing/`, which any package's test may call, takes every
binary from `built`, its own callers' package's too, and never from
`paths::binary` (`tools/crates/design.md`, 3.2).

A fake that runs as a step machine, as `skein-echo-client` does,
is held to the subset instead (../../foundation/testing-strategy.md, 4): it has no
`clippy.toml` of its own, so the step crates' applies.

A machine's worlds are a crate of their own, `tests/<machine>`, with its
transcripts in `transcripts/` beside its tests. The minimal machine joins
the io worlds with files and processes; the protocol worlds are a crate
of their own, `tests/protocol`. The gate runs formatting, the lints as errors,
clippy's lints as errors for `wasm32-unknown-unknown` over the portable
crates (`README.md`, section 10) and the UI example's view and browser shell, since the
parts' platform halves compile only there (../ui/web/testing.md); then
the focused suite and the fuzzy suite, with their profiles (`README.md`, section 7), and
nothing slower. The browser tier, outside the gate, is
`tools/tiers/run skein browser`: the UI example's
bundle, made with `skein-ui-bundle` from a module built with the dev
profile (../ui/web/bundle.md, 1), and the browser suite, which runs the browser
kit's tests against Chromium and the UI example's tier on that bundle
(../ui/web/testing.md, 1). The builds for that target need only the
target installed: they run nothing, since the portable crates' tests run
on the host, and skein-ui-web's platform half runs in the browser tier.
