# tools: temper's tools, area 1, first part, and what jig and skein give them

2026-10-19. Start commits: skein `064c55f` (its code and the design this
plan reads are those of `b4c6241`: everything after is the forge kit's
design and STATUS marks), smith `a277e922`, jig `6ad8bb40`, temper
`1c918a77c`; each session starts from its repository's `origin/main`.
smith changes only by its closing repin.

temper's tools (`docs/design/client/tools.md`), area 1, are planned in two
consecutive plans (decisions.md, 16): the whole of area 1 had a longest
chain of about 31 steps. This plan, the first, builds section 13 and the
tools as a library. skein's request table takes a key given by the asker
(skein 01). jig moves to skein's and smith's main (jig 01), sends under a
named key (jig 02), reads one turn (jig 03), and takes a deadline as a
span that the engine makes a wall (jig 05), which a named request sent
again needs (client/tools.md 7). temper moves to them (temper 01, 03.1) and
builds `temper-client-tools`: the table and a call's argv (temper 02),
the root and the runner with the reads and acts (temper 03), the waits
(temper 04), the forge's parts and `adopt` (temper 11), help and a
call's JSON (temper 12). jig's focused budget goes back to 15 s after a
test-cost pass (jig 04).

The next plan builds on this one's `main`: `temper ctl`'s host, the
client stories, the fake forge's host, the real-loop world and its
stories, and the process tier. `later.md`'s first row names where it
starts; its drafts, written against this plan's names, are in
`findings/next/`, as input for its planner, not as sessions of this
plan.

## Inputs

- **temper's design,** at `1c918a77c`: `docs/design/client/tools.md` (area 1's
  tools as 5.5 lists them: 4, 5.1 to 5.4, 6, 7, 8.2, 11.1, 11.5, 12,
  13; 5.6, 8, 11.2 to 11.4 and 15 for the next plan), `docs/design/client/README.md` (3.1, 4.1 to 4.7, 5.2),
  `docs/design/STATUS.md` ("tools"), and `docs/design/domain/testing.md` (2.4,
  the real loop, for the next plan).
- **jig's design,** at `6ad8bb40`: `docs/design/client/domain/client.md`
  (4.1, 4.6, 6, 7, 8.1, 10, 11, 19, 20),
  `docs/design/client/domain/contract.md` (3.1, 5.1, 5.3, 6, 8),
  `docs/design/client/domain/testing.md` (2),
  `docs/design/protocol/README.md` (7), `docs/design/protocol/client.md`,
  `docs/design/protocol/configuration.md` (3.1),
  `docs/design/domain/authority.md` (7), `docs/design/domain/tasks.md`
  (4), `docs/design/domain/engine.md` (4), and the rows of
  `docs/design/STATUS.md` that name "the downstream repository's tools
  plan".
- **skein's design,** at `b4c6241`: `docs/design/lib.md` (12).
- **The workflows:** jig's and temper's `docs/development/workflow.md`;
  skein's `AGENTS.md` and `docs/foundation/testing-strategy.md` (8, 10).
- **`decisions.md`:** the design questions and their answers, the plan's
  decisions, and what the user may veto.
- **`findings/`:** the base builds at the new pins (`base-repin.md`),
  the audits (`jig-section-13.md`, `temper-area-1.md`), jig's test cost
  (`jig-test-cost.md`, `jig-test-profile.md`), the suites' times at the
  base (`base-times.md`), who drafted each file (`drafters.md`), and the
  next plan's drafts (`next/`).
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"jig 03" means that session's last increment has landed on jig's
`origin/main`. temper's sessions are numbered 01 to 04, 11 and 12: 05 to
10 are the next plan's drafts.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-given-keys.md` | skein 01: given keys in skein-lib's request table: typed refusals, each admission's number, a key given by the asker | nothing | |

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-repin.md` | jig 01: jig on skein `b4c6241` and smith `a277e922`; jobs as call outcomes; the test configuration on smith's shipped memory | nothing | |
| `jig/02-named-send.md` | jig 02: the send under a named key, and `Outcome::Held` | skein 01, jig 01 | |
| `jig/03-turn-read.md` | jig 03: the read of one turn, and the client wire at version 9 | jig 01 | |
| `jig/04-test-cost.md` | jig 04: the test-cost pass, and the focused budget back to 15 s | nothing | 04.4: jig 01 |
| `jig/05-deadline-spans.md` | jig 05: deadlines given as spans, made walls by the engine | jig 01 | 05.2: jig 03; 05.5: jig 02 |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-repin.md` | temper 01: temper on skein's, smith's and jig 01's main | jig 01 | |
| `temper/02-tools-table.md` | temper 02: the tools' table: `temper-client-tools`, its bounds, the named key, a call and its argv | temper 01 | |
| `temper/03-tools-root.md` | temper 03: the tools' root: temper on jig's named send, turn read and spans; the root and the runner, the reads and the acts | temper 02, jig 02, jig 03, jig 05 | |
| `temper/04-waits.md` | temper 04: the tools' waits: digests, conditions, the wait, a watch's end | temper 03 | |
| `temper/11-tools-forge.md` | temper 11: the tools' forge parts: the forge's words in `temper-client-domain`, and `adopt` | temper 03 | |
| `temper/12-tools-help.md` | temper 12: the tools' help and a call's JSON | temper 02 | |

### The closing repins

Development.md 5.7: no session file. Each lands with its repository's
gate.

| Step | Starts once | Moves |
|---|---|---|
| `smith-repin` | skein 01 has landed | smith's skein to skein's `origin/main` |
| `jig-repin` | `smith-repin` and every jig step have landed | jig's skein and smith, together, to their `origin/main` |
| `jig-repin.s1` | added by ruling 138 for `temper-repin`'s block of 2026-10-19T11:25:37Z; starts at once | jig's skein and smith, together, to their `origin/main` again, past live-text's smith: `live: None` in each start, an ignored `OpenEvent::Live`, `smith_channel::v3` for `v2` in jig's four files (smith's live-text README, Repin) |
| `hearth-repin` | `jig-repin` has landed, and `~/src/rust/hearth` is a checkout (Prerequisites) | hearth's skein to skein's `origin/main` |
| `temper-repin` | `jig-repin.s1`, `hearth-repin` and every temper step have landed | temper's skein, smith and jig to their `origin/main`, with live-text's `live: None` and ignored `OpenEvent::Live` in temper too |

Neither smith nor hearth uses skein's request table, so their repins
should move the pin and repair nothing (findings/base-repin.md).

### How many at once, and in what order

- **As many sessions at once as their Needs allow;** this plan never has
  more than seven ready together, under the coordinator's eight.
- **At the start,** the three that need nothing: jig 01, skein 01, jig
  04.
- **Then,** as each lands: jig 03, jig 05 and temper 01 once jig 01 has;
  jig 02 once skein 01 and jig 01 have; `smith-repin` once skein 01 has;
  temper 02 once temper 01 has; temper 12 once temper 02 has; temper 03
  once temper 02, jig 02, jig 03 and jig 05 have; temper 04 and temper
  11 once temper 03 has; the other repins as their table says.
- **When slots are short,** start first: jig 01, jig 03, jig 05, temper
  01, temper 02, skein 01, jig 02, temper 03, temper 04; then temper 11,
  temper 12, jig 04 and the repins.
- **The longest chain,** 21 steps: jig 01.1 to 01.3 → jig 03.1 to 03.5
  → jig 05.2 to 05.5 → temper 03.1 to 03.4 → temper 04.1 to 04.4 →
  `temper-repin`; at 0.09 to 0.14 h a step, about 1.9 to 2.9 hours. The
  temper chain to temper 03 (temper 01, 02: four steps after jig 01) and
  jig 02 (after skein 01's two) run beside it with room; temper 11 (two
  steps) and temper 12 (two) run beside temper 04.
- **The plan's size:** 38 steps and 4 repins, against development.md's
  bound for one plan of about 20 on the longest chain and about 80 in all
  (decisions.md, 16).

## Rules of this plan

- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the items of `decisions.md` that name its
  session. Nothing else is read whole.
- **Waiting on a command, on Codex:** the one way to wait on a running
  command is `// @exec: {"yield_time_ms": 300000}`, never repeated
  `write_stdin` polls. In client-api, 4,830 polls cost 21% of the
  authors' input.
- **The code graph** is not used for a repository until the supervisor
  has re-indexed it past its `origin/main` (AGENTS.md) and a steer says
  so. Until then, search with `rg` in the worktree.
- **Naming, upstream.** skein's request table names no repository
  downstream of it: `git -C <the session's skein worktree> grep -n -i -w
  -E 'temper|jig|smith|hearth' -- crates/skein-lib tests/lib` prints
  nothing at each skein increment's end. jig names no temper: jig's
  session files say "the repository downstream of jig", and `git -C
  <the session's jig worktree> grep -n -i -w temper -- crates testing
  tests examples scripts` prints nothing at each jig increment's end.
- **Pins.**
  - **A step's landing** is the commit the run's ledger records for it
    (`plan status` prints it); a session's landing is its last step's.
  - jig 01.1 pins skein `b4c6241` and smith `a277e922`; jig 02.1 moves
    jig's skein to skein 01's landing and leaves smith.
  - temper 01.1 pins jig 01's landing, with skein and smith as jig's
    `Cargo.lock` holds them at it.
  - temper 03.1 pins the newest of jig 02's, jig 03's and jig 05's
    landings, which holds the other two on jig's linear `main`, with
    jig's skein and smith at it.
  - After any `cargo update`, `grep -o
    'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' Cargo.lock |
    sort -u` prints one line per repository. A source left with two
    revisions by a later command (smith's update moves the skein that
    only smith's crates use to skein's `main`) gets its own command once
    more, last; a second revision still there is a question.
- **Version owners.** jig's client wire goes to version 9 once, in jig
  03.1; jig 03.2, 03.3, 05.2 and 05.3 change version 9 in place, jig
  05.2 after jig 03's last landing; nothing makes a version 10. jig's
  store families `tasks` and `people` go to version 4 in jig 05.1;
  temper's `people_v3` names follow in temper 03.1. jig 02 changes no wire: the name is the engine
  key the wire carries already (decisions.md, 4).
- **Order on shared files.** jig 02, 03 and 05 all change
  `jig-client-domain` and its stories: jig 05.2 waits for jig 03, and
  jig 05.5 for jig 02, as their rows say; elsewhere the later landing
  rebases. temper 11 and temper 12 both add to
  `crates/temper-client-tools/src/tests/fixtures.rs` and the crate's
  `lib.rs`: the later rebases, and neither renames what the other added.
- **jig's focused budget** stays 30 s (its ruling 275) until jig 04.4
  lands; from then on 15 s binds every jig landing, jig 02's, 03's and
  05's included.
- **No optimized test builds and no `[profile.*]` change** in any
  repository (testing-strategy.md, 8, the user's ruling on design
  question 20). A slow test is profiled and fixed in the code or the
  test.
- **Forge- and LLM-facing work** stays in the increments the Hotspots
  section names, behind the interfaces that exist; no other increment
  edits those files beyond a compiler-forced arm.
- **Joint stories share the fixture** (testing-strategy.md, 10: "A
  setup written once", "Named fixtures and sizes"). A joint story in
  jig's `tests/client/src/joint/` builds its turn with `fixture.rs`'s
  `turn`, given a parameter there if it needs one, and writes no turn,
  call or result of its own; a Shape that asks for one gets it once, in
  `fixture.rs`, said in the commit body. Each size and job id is a
  named const with a line on how its number is derived. The first
  increment that edits `stories/job_stories.rs` moves its own `turn`,
  `call` and `result` (jig-01.2) there too.
- **A setup written once, and rows carry data** (testing-strategy.md,
  10: "Rows carry data", "A setup written once", "Named fixtures and
  sizes", "A bound's two sides"). An opener or destructuring two tests of
  a file share is one helper of its fixtures; a size an assertion reads
  is a named const; a table's row carries the value it varies (a
  `load_bytes: u32`), never a flag that switches the setup; a bound is
  tested on both sides. It holds for each test an increment adds or
  edits. jig's `crates/jig-domain-core/src/tests/read_activity.rs` (its
  `tiny` row and its three-line opener) and
  `crates/jig-client-domain/src/tests/requests.rs` (its save-then-send
  destructuring), and `tests/core/tests/deadline_spans.rs` (jig 05.3:
  its aged-span test of three behaviours, its row loop that branches on
  the expected value, its signed-in opener in three tests, its bare
  seeds, request keys and walls; the steer of 08:18) are mended by the
  first increment that edits them,
  said in its commit body. A case that extends a fixture is built from
  it (`[fixtures::ADOPTION, &["--no-home"]].concat()`), never retyped;
  each size a test uses (a depth, a capacity, a chunk) is a named const
  of its fixtures, with a line on what a smaller value would miss. Both
  hold for each test an increment touches in a file it edits (the
  steer of 07:05).
- **One behaviour per test** (testing-strategy.md, 10). A test checks
  one behaviour and is named for it, never with "and": temper's
  `adopts_flags_parse_and_its_absent_flags_stay_absent` becomes
  `adopt_flags_parse_to_their_values` and
  `adopt_absent_flags_stay_absent`. A two-behaviour test in a file an
  increment edits is split there, said in its commit body; 12.1's
  plan-given names stay.
- **No lint exception for a plan's words.** A Shape or sketch that needs
  a call clippy.toml bans is a question, raised before any `#[expect]`;
  each new `#[expect]` or `#[allow]` is named in the report. temper's
  `crates/temper-client-tools/src/tests/fixtures.rs` `sources()` drops
  its `#[expect(clippy::disallowed_macros)]` and `env!` for relative
  paths in the next temper 02 increment, said in its commit body.

## Hotspots

Work that may run alongside this plan: the forge kit (skein's
`docs/design/forge.md`, `forge-client.md`, `forge-connection.md`,
`forge-setup.md` and `fake-forge.md`, landed at `064c55f`, and temper's
`docs/design/draft/forge-kit.md`), which moves the forge client, the
component, the dialect, setup and the fakes out of temper; and the LLM
accounts drafts (temper's `docs/design/draft/llm-setup.md`, jig's
`docs/design/draft/llm-accounts.md` and `draft/providers.md`). The files
of this plan they would also touch:

| Files | Steps | Kind |
|---|---|---|
| `crates/temper-client-domain/src/forge.rs` (`told`'s `Outcome::Held` arm) | temper-03.1 | **forge kit** (the client's forge family) |
| `crates/temper-service/src/family.rs`, `src/family/refusals.rs` (the owner routing's `Runner` arm) | temper-03.1, temper-03.2 | **forge kit** (the forge family's service side) |
| `crates/temper-client-domain/Cargo.toml`, `src/forge.rs`, `src/forge/words.rs` (new), `src/forge/form.rs` | temper-11.1, temper-11.2 | **forge kit** (the forge's words, its ask and answer) |
| `crates/temper-client-view/src/forge.rs`, `src/forge/{words,bindings,page}.rs`, `src/words.rs`, `src/root.rs` | temper-11.1 | **forge kit** (the view's forge page) |
| `crates/temper-client-tools/src/forge.rs` (new) | temper-11.2 | **forge kit** (`adopt` and the forge's parts) |
| `testing/temper-fake-git/tests/transcripts.rs` (`Disk::run`, `Disk::file`) | temper-01.1 | **forge kit** (a fake; it stays in temper by draft/forge-kit.md, 2) |
| `crates/temper-engine-smith/src/finish.rs`, `src/tests.rs` (a job request answered as no turn) | temper-01.1 | **LLM** (smith's agent in the engine) |
| `Cargo.lock`, in every repository | each repin, temper-11.1 | shared |

No step changes the fake LLM, the providers, the forge client
(`temper-engine-domain-forge-client`), the component
(`temper-engine-protocol-forge`), `temper-forge-forgejo`,
`temper-forge-setup`, or the fake forge's domain and protocol.

## Prerequisites the run checks at its start

The run's machine is debian-16gb-hel1-1. The coordinator runs each check
once, before the first session (4 and 5 when their step becomes ready),
and writes `blocked.md` (kind: process) on a failure:

1. `cargo nextest --version` prints a version.
2. `ls "$(rustc --print sysroot)/lib/rustlib"` lists
   `wasm32-unknown-unknown` (skein's and jig's gates).
3. `/usr/bin/chromium --version` prints `Chromium 1`, or
   `SKEIN_TEST_CHROMIUM` names a file (skein's and jig's browser
   suites).
4. **When temper 01 becomes ready:** `test -x "$TEMPER_FORGEJO"`.
   temper 01.1's process proof runs `a_native_chat_reaches_the_model`,
   which starts a throwaway Forgejo from that binary
   (`testing/temper-throwaway-forgejo/src/binary.rs`). It is set on
   this machine: Forgejo 17.0.0, the pinned build (decisions.md, C).
5. **When `hearth-repin` becomes ready:** `git -C ~/src/rust/hearth
   rev-parse origin/main` prints a commit. hearth has no checkout on this
   machine at the plan's start.

## One test of a suite that needs a built artifact

A browser story that fails is run alone, in the session's worktree,
through the build lane: after each fix aimed at a cause its log names,
the two commands below run once each; what counts is the second's
`Summary` line, 1 test run, 1 passed. The same symptom a third time is a
question (development.md, 5.5). jig 02, 03 and 05 change what jig's
browser client decodes, and skein 01 the request table under it.

**skein's browser suite,** as skein's `scripts/check.sh` builds it:

```sh
sh -c 'cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server'
sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run --workspace --profile browser -E "test(=<name>)"'
```

**jig's browser suite,** as jig's `scripts/check.sh` builds it (where
the two differ, `scripts/check.sh` wins):

```sh
sh -c 'cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web && cargo build -p skein-ui-bundle && target/debug/skein-ui-bundle --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm --name jig_client_test_shell_web --prefix /bundle/ --stylesheet crates/jig-client-shell-web/jig.css --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css --pictures jig=crates/jig-client-shell-web/pictures --pictures jig-test=testing/jig-client-test-shell-web/pictures --files testing/jig-client-test-shell-web/files --noscript "jig needs WebAssembly and scripts." --out target/jig-test-bundle && cargo build -p jig-test'
sh -c 'JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle" cargo nextest run --workspace --profile browser -E "test(=<name>)"'
```

The bundle's path is absolute: nextest runs each test from its package's
directory.

## Live services

None. The process proofs (jig 01.3, temper 01.1) run a fake LLM and a
throwaway Forgejo on loopback; no agent signs in anywhere.

## Process tier

In a checkout of each repository's `origin/main`, outside its gate:
temper `scripts/proof-one-process.sh`, with
`TEMPER_FORGEJO` set (its stories start a throwaway Forgejo); jig `scripts/proof-one-process.sh`. This plan adds no process story;
jig 01.3 and temper 01.1 run one or two of them as their Proofs.

## Repin

Nothing: the closing repins repin every repository downstream of skein
and jig (smith, jig, hearth, temper).

## Kit steps

None: the plan changes no `scripts/dev`.

## Pulse

The aims are the base's medians on the run's machine
(`findings/base-times.md`: jig 5.23 s and 6.72 s, temper 2.74 s and
7.04 s) plus what the run may add: jig's new stories and units, less
jig 04's fixes, about 1.5 s focused and 1 s fuzzy; temper's new crate's
units about 1.3 s focused, nothing fuzzy.

```
words: legacy|compat|hack|workaround|actual|genuine
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
aims: jig 7 8
aims: temper 5 8
```

## Done when

Each command runs once, after `done.md`, reading `origin/main`, after
`git -C ~/src/rust/<repository> fetch` in each of skein, smith, jig,
hearth and temper. Each prints nothing, or what its item says.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. skein's table takes a given key and refuses a held one; `ask` no
   longer returns a bare key. The first prints 2, the second 0:

   ```
   git -C ~/src/rust/skein show origin/main:crates/skein-lib/src/requests.rs | grep -c -E 'pub fn ask_given\(|^ +Held\(T\),$'
   git -C ~/src/rust/skein show origin/main:crates/skein-lib/src/requests.rs | grep -c -F -- '-> Result<RequestKey, R>'
   ```

3. jig sends under a name; prints 1:

   ```
   git -C ~/src/rust/jig show origin/main:crates/jig-client-domain/src/boundary.rs | grep -c -F 'Send { key: Key, name: Option<RequestKey>, request: Ask },'
   ```

4. jig reads one turn, at one wire version. The first prints one line,
   `crates/jig-client-protocol-wire/src/lib.rs` with `9`; the second 1
   or more:

   ```
   git -C ~/src/rust/jig grep -n -E '(CLIENT_VERSION|VERSION): u32 = [0-9]' origin/main -- crates
   git -C ~/src/rust/jig show origin/main:crates/jig-client-domain/src/contract/reads.rs | grep -c -F 'Turn { task: u64, attempt: u64, turn: u32 }'
   ```

5. jig takes a deadline as a span; each prints 1:

   ```
   git -C ~/src/rust/jig show origin/main:crates/jig-client-domain/src/contract/operations.rs | grep -c 'pub enum NewDeadline'
   git -C ~/src/rust/jig show origin/main:crates/jig-protocol-store/src/format.rs | grep -c 'tasks: 4, people: 4'
   ```

6. jig's budgets are 15 s and 60 s again; prints `15s 60s`:

   ```
   git -C ~/src/rust/jig show origin/main:.config/nextest.toml | python3 -c 'import sys, tomllib; p = tomllib.loads(sys.stdin.read())["profile"]; print(p["default"]["global-timeout"], p["fuzzy"]["global-timeout"])'
   ```

7. No profile changed; prints nothing:

   ```
   for s in jig:6ad8bb40 temper:1c918a77c skein:b4c6241; do r=${s%%:*}; c=${s#*:}; diff <(git -C ~/src/rust/$r show $c:Cargo.toml | awk '/^\[/{p=/^\[profile/} p') <(git -C ~/src/rust/$r show origin/main:Cargo.toml | awk '/^\[/{p=/^\[profile/} p'); done
   ```

8. temper's tools exist as a library; each prints 1:

   ```
   git -C ~/src/rust/temper show origin/main:Cargo.toml | grep -c -x '    "crates/temper-client-tools",'
   git -C ~/src/rust/temper show origin/main:crates/temper-client-tools/src/root.rs | grep -c 'pub fn client('
   git -C ~/src/rust/temper show origin/main:crates/temper-client-protocol-wire/src/lib.rs | grep -c 'Owner::Runner => 3'
   ```

9. The table's usage lines for the tools whose syntax the design
   settled last (`close`, `decide`, `adopt`); prints nothing:

   ```
   for u in 'close N [--move D]... [--reason TEXT]' 'decide tN/K|pN/K accept|reject|pass-up [REASON]' 'decide N --hold R release|leave|pass-up [REASON]' 'adopt --project P [--forge F] --owner O --name N --landing B'; do git -C ~/src/rust/temper grep -q -F "$u" origin/main -- crates/temper-client-tools || echo "missing: $u"; done
   ```

10. The naming checks hold; each prints nothing:

    ```
    git -C ~/src/rust/jig grep -n -i -w temper origin/main -- crates testing tests examples scripts
    git -C ~/src/rust/skein grep -n -i -w -E 'temper|jig|smith|hearth' origin/main -- crates/skein-lib tests/lib
    ```

11. Every repository downstream of skein takes one revision of each
    source, and its skein has the given keys; each prints nothing:

    ```
    for r in smith jig hearth temper; do git -C ~/src/rust/$r show origin/main:Cargo.lock | grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u | cut -d'#' -f1 | uniq -d; done
    for r in smith jig hearth temper; do s=$(git -C ~/src/rust/$r show origin/main:Cargo.lock | grep -o -m1 'forgetic/skein.git#[0-9a-f]*' | cut -d'#' -f2); git -C ~/src/rust/skein show "$s:crates/skein-lib/src/requests.rs" | grep -q 'pub fn ask_given(' || echo "$r: skein $s has no ask_given"; done
    ```

12. temper holds jig's wire version 9; prints nothing:

    ```
    j=$(git -C ~/src/rust/temper show origin/main:Cargo.lock | grep -o -m1 'forgetic/jig.git#[0-9a-f]*' | cut -d'#' -f2); git -C ~/src/rust/jig show "$j:crates/jig-client-protocol-wire/src/lib.rs" | grep -q -x 'pub const VERSION: u32 = 9;' || echo "temper's jig $j is not at version 9"
    ```

13. The test-time aims, measured once in a worktree of jig and of
    temper at `origin/main`, on the run's machine. The medians of the
    three `Summary` times per suite, against the Pulse's aims. An aim
    missed is recorded with the three slowest tests by name in
    decisions.md's rulings, for the user; the run is done either way.

    ```
    sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary; cargo nextest run --workspace --profile fuzzy 2>&1 | grep Summary; done'
    ```

14. Nothing outside these items is checked.
