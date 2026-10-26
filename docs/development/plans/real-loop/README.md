# real-loop: temper's tools, area 1, second part, and the real-loop world

2026-10-19. Start commits: temper `f1a57db2b` (past the organization's
rename, `5ecc737ca`, and the design's move and rewording, `5587ae49b`
and `a959f6e10`; past part 1's temper-repin, `8c02b3b9e`), with skein `92e177c3`, smith `f47f4df9`
and jig `149970b9` as its `Cargo.lock` holds them; each session starts from temper's
`origin/main`. The plan changes temper alone (decisions.md, 3).

temper's tools (`docs/design/client/tools.md`), area 1, are planned in two
consecutive plans (the tools plan's decisions.md, 16). The tools plan,
the first, built `temper-client-tools` as a library. This plan, the
second, builds on it: `temper ctl`'s one-shot host,
`temper-client-shell-cli` (temper 05); the client stories over the tools
(temper 06); the real-loop world, `tests/real-loop`, as the start of the
general layer testing.md 2.4 describes, with its first stories (temper
08); the stories that need more of the loop: a restart, a lost answer,
the lock, another party's kept requests (temper 09); the landing through
the loop, over skein's forge peer and temper's fake git (temper 13); and
the process tier's part of area 1 (temper 10); and the fuzzy walking
story's cost, spread over eight shards that name its cause until the
state digest, the next plan, cures it (temper 14; decisions.md, 12).

One wait leaves the plan: temper 13 waits for forge-kit's temper-05.2,
which teaches temper's one-process world skein's forge peer (skein-05.5
has landed, inside the start pins). No step waits for live text or LLM
accounts (design question 2).

## Inputs

- **temper's design,** at `f1a57db2b`: `docs/design/client/tools.md` (4,
  5.2 to 5.6, 6, 7, 8, 10, 11.2 to 11.4, 12, 15) and
  `docs/design/domain/testing.md` (2.4, 2.6, 4.3, 5.1, 5.2, 6.1, 6.3,
  10), which the plan calls `tools.md` and `testing.md` (moved in
  `5587ae49b`, their section numbers kept), `docs/design/domain/forge.md` (8),
  `docs/design/protocol/forge.md` (5), `docs/design/protocol/setup.md`
  (13.4), `docs/design/STATUS.md` ("tools", testing.md 2.4). tools.md
  holds the tools plan's decisions M to Q since `a959f6e10`
  (decisions.md, 13).
- **jig's design,** at `149970b9`: `docs/design/protocol/client.md` (8, 10),
  `docs/design/client/domain/client.md` (6, 11, 20),
  `docs/design/client/domain/contract.md` (3.1),
  `docs/design/client/domain/testing.md` (6.1, 6.2).
- **skein's design,** at `92e177c3`: `docs/design/fake-llm.md`
  (6), `docs/design/fake-forge.md` (2, 3, 5, 6), `docs/design/io.md` (5,
  7), `docs/design/shell.md` (6, 13), `docs/design/kernel.md` (6.2),
  `docs/design/lib.md` (12), `docs/foundation/programming-model.md` (2,
  4.2, 4.3, 6.3, 9, 10.2).
- **smith's design,** at `f47f4df9`: `docs/design/protocol/hosts.md`
  (3), and its agent-process world's `fake::limits`.
- **The workflows:** temper's `docs/development/workflow.md`; skein's
  `docs/foundation/testing-strategy.md` (2.8, 4.1, 6, 7, 8, 10).
- **The tools plan,** `docs/development/plans/tools/`: its decisions and
  the designer's answers (questions 3, 12 to 18, 25, 29 and 33), which
  this plan builds on; its `findings/next/`, the drafts this plan's
  sessions were refreshed from.
- **`decisions.md`:** the design questions and their answers, the plan's
  decisions, and what the user may veto.
- **`findings/`:** `base-times.md`, the suites' and the process tier's
  times at the base, per package; `fuzzy-walking.md`, the slow fuzzy
  story's profile (temper 14); `code-checks.md`, the sessions against the
  code, the tools plan's decisions M to Q, tool positions, and the
  carriers of what temper 05 prints. They ran at earlier commits
  (`e139ccaf4`, `cd4727819`); no Rust file differs from them to the
  start commit, and the manifests differ only in their URLs.
- **`later.md`:** what the design names that this plan does not build,
  and what the real-loop world still lacks to cover the rest of temper.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"temper 05" means that session's last increment has landed on temper's
`origin/main`; "temper 05's reads" names the increment its file says. A
step of another plan is named with its plan, as its `Step:` trailer
gives it: forge-kit's `temper-05.2`. The sessions keep their drafts'
numbers; there is no temper 07 (decisions.md, 4); temper 14 is this
plan's own.

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/14-fuzzy-walking.md` | temper 14: the fuzzy walking story in eight shards, its cause named, until the state digest, the next plan | nothing | |
| `temper/05-ctl.md` | temper 05: `temper ctl`'s one-shot host, `temper-client-shell-cli`: DIRECTORY and its lock, the transport and a read, `sign-in FILE`, the `chat` act first; then the other acts, `sign-out`, `sign-in -`, `pending`, the waits, `help` and the executable | nothing | |
| `temper/06-client-stories.md` | temper 06: the client stories over the tools, in `tests/client`: an agent beside the person, a chat's life, a turn too large for a page, an adoption and a released task, cut calls made once | nothing | |
| `temper/08-real-loop.md` | temper 08: the real-loop world, `tests/real-loop`: the scripted LLM peer, the standalone service checked and drained, the meeting point (a sign-in and a chat to its reply), another client's act meeting a running wait | nothing | 08.3: temper 05's `chat` act (05.4) and temper-05.s1 (ruling 111); 08.4: temper 05 |
| `temper/09-loop-stories.md` | temper 09: the real loop's stories over more than one call: a restart, a lost answer, DIRECTORY's lock, a record these tools cannot read | temper 08 | |
| `temper/13-landing.md` | temper 13: the landing through the real loop, over skein's forge peer and temper's fake git, unchecked (decisions.md, design question 4) | temper 08; forge-kit's skein-05.5 and temper-05.2 (`plan await coordinator forge-kit/skein-05.5 forge-kit/temper-05.2`) | |
| `temper/10-process.md` | temper 10: the process tier: `temper ctl`'s fixture and its paths through `main`, an adopt cut by a termination signal and made once | temper 05 | |

### How many at once, and in what order

- **As many sessions at once as their Needs allow;** this plan never has
  more than four ready together, under the coordinator's eight.
- **At the start,** the four that need nothing: temper 14, temper 05,
  temper 08 (08.1 and 08.2) and temper 06.
- **Then,** as each lands: temper 08.3 once temper 05.4 and
  temper-05.s1 (ruling 111) have, temper 08.4
  once temper 05 has; temper 10 once temper 05 has; temper 09 once temper
  08 has; temper 13 once temper 08 and forge-kit's temper-05.2 have;
  temper-13.s1 at once, and 13.4 after it (ruling 112); temper-13.s2
  after temper-13.5 (ruling 142); temper-13.s3 (ruling 145), then
  temper-13.s4 (ruling 148), before 13.5 goes on.
- **When slots are short,** start first: temper 05, temper 08, temper
  14; then temper 13, temper 09, temper 06 and temper 10.
- **The longest chain,** 12 steps: temper 05.1 to 05.6 → temper 08.4 →
  temper 13.1 to 13.5; at about 24 minutes a step (the tools plan's
  retrospective, proposal 5), about 4.8 hours, if forge-kit's
  temper-05.2 has landed by the time temper 08 ends; otherwise temper 13
  starts when it lands. temper 09 (four steps) and temper 10 (two) run
  beside temper 13; temper 06 (five) and temper 14 (one) beside temper
  05.
- **The plan's size:** 27 steps and no repin, against development.md's
  bound for one plan of about 20 on the longest chain and about 80 in
  all.

## Rules of this plan

- **Every session reads,** besides its "Read first": temper's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the items of `decisions.md` that name its
  session. Nothing else is read whole.
- **Waiting on a command, on Codex:** the one way to wait on a running
  command is `// @exec: {"yield_time_ms": 300000}`, never repeated
  `write_stdin` polls.
- **temper alone.** No step edits skein, smith or jig, nor moves a pin
  (decisions.md, 3). A need upstream is a question (development.md,
  10.1), raised before any workaround.
- **The real loop's pieces are the general layer's** (testing.md 2.4):
  `tests/real-loop`'s world and fixtures, built in temper 08, are
  extended by temper 09 and 13 and never redefined; a story builds its
  loop with the world's helpers, and a helper two stories need is the
  world's, said in the commit body.
- **The real loop is smoke** (testing.md 2.4; ruling 130): one
  end-to-end path per feature. A behaviour or corner case a step meets
  is tested in the lower world that owns it (2.4 names the accounts'
  owners), never in `tests/real-loop`; a real-loop test outside 2.4's
  stories and the steps' Scopes is a question (development.md, 10.1),
  raised before it is written. This rule replaces none; the steps' Scopes
  stay as ruling 130 left them.
- **A story's cost is understood, never hidden** (testing.md 6.1). Each
  increment that adds a real-loop story runs it alone, unoptimized,
  three times, and gives the median wall time in its commit body. A story
  over its 1 s aim is made cheaper or moved to the fuzzy profile, never
  merged with another and never excepted; what it waits on is named.
- **The worlds' sums below the domain hold no step** (testing.md 6.1, as
  the user's decision sets it, decisions.md, For the user, L: "the real
  loop is counted apart, with a standing ceiling of 6 s for its sum, in
  place of an aim per plan, and each story at most 1 s. The other worlds
  below the domain keep 4 s"). Each sum adds each test's own time, idle
  (`--profile measure -j 1`), the median of
  three runs:
  - the packages `temper-client-shell-terminal`, `temper-client-world`,
    `temper-engine-protocol-forge-world`, `temper-one-process-world` and
    `temper-worker-protocol-git-world` together, against 6.1's 4 s
    (reconcile's world is skein's since forge-kit's temper-07.1, ruling
    150);
  - `temper-real-loop-world` alone, against 6.1's 6 s, with each story
    at its 1 s.

  At `94df159d3`, with 08.2, the seven took 3.91 s, the real loop 0.09
  to 0.10 s of it (ruling 112's answer, point 1). A test within its own
  guide lands whatever the sums. A test is made cheaper or moved to the
  fuzzy suite for a sum only by a step the supervisor names (6.1: "by
  steps a plan names"), such as temper-13.s1, never by a session's own
  move, cut or merge. Done when 5 gives both sums at the end.
- **A world failure's cause, within a budget** (ruling 110;
  the implementation rules and 8.8). A focused world test that fails with a
  symptom its three diagnostic runs did not name gets eight more runs
  and 40 minutes for that symptom, reproduced in its world through
  with diagnostics removed before the commit. A fix within the step's
  Files and rulings needs no question; past the budget, or for a fix
  outside them, raise it with the cause found, or each run's finding.
- **No optimized test builds and no change to Cargo's `[profile.*]`**
  (testing-strategy.md, 8). A slow test is profiled and fixed in the
  code or the test.
- **Kernel-assigned ports.** Every listener the real loop starts binds
  port 0 and is found by its bound address; no story names a fixed port
  (testing.md 2.4).
- **A setup written once, and rows carry data** (testing-strategy.md,
  10). An opener or helper two tests share, in one file or two, is one
  helper of the crate's fixtures: before defining one, `rg 'fn <name>'
  <crate>/tests <crate>/src/tests`, and move a sibling file's there. A
  size or a count an assertion reads (a call's `tries`, a number of
  chats) is a named const with a line on what a smaller value would miss
  (ruling 137); a story's parties, tasks, projects and tokens, and so its
  calls' positions, are named consts with a line on their value, as
  `tests/real-loop/tests/restart.rs`'s `CHAT`, `READ` and `CHATS` are:
  no integer literal is the argument of `task(`, `watching(`, `join(` or
  `number(`, nor an index of `calls()`, in a test body, and the gate
  refuses one (temper-09.s2, ruling 133); a story's calls are read by
  those consts (`ran.calls()[AGAIN]`, as `tests/lock.rs` does), never
  bound by a slice pattern over `calls()` with `_` for the calls it
  skips (ruling 137); a case that extends a fixture is built from it, never
  retyped. A table's body checks every row alike and never branches on a
  row: rows that expect different outcomes are tests of their own
  (ruling 126).
- **One behaviour per test** (testing-strategy.md, 10), named for it,
  never with "and": it sets up, acts once, then asserts that behaviour.
  Nothing is asserted on the way to the step under test, and a bound
  has both sides, from named constants (ruling 117). The expected value
  is a literal, a golden file or a named size, never computed by the
  code under test (ruling 126).
- **No lint exception for a plan's words.** A Shape or sketch that needs
  a call clippy.toml bans is a question, raised before any `#[expect]`;
  each new `#[expect]` or `#[allow]` is named in the report.
- **Repository links name github.com/fabrig-dev** (the organization's
  new name). A step that adds a dependency line copies the URL its
  `Cargo.toml` already uses, never retypes it: Cargo treats two
  spellings of one URL as two sources. Every Cargo line names fabrig-dev
  since temper `5ecc737ca`.
## Hotspots

Work that may run alongside this plan: the forge kit (forge-kit's plan,
`docs/development/plans/forge-kit/` in temper), which moves the fake
forge into skein and teaches temper's one-process world skein's forge
peer; live text (smith's `docs/development/plans/live-text/`) and LLM
accounts (smith's `docs/development/plans/accounts/`), which add parts
to skein's fake LLM and repin smith; the browser shell (browser-1, on
hel8), whose 01.1 adds `"bundle": null` to every configuration fixture.
The files of this plan they would also touch:

| Files | Steps | Kind |
|---|---|---|
| `Cargo.toml`, `Cargo.lock` (temper's root) | 05.1, 05.5, 08.1, 08.2; `Cargo.lock` alone: 06.1, 06.3, 09.2, 13.1, 13.2, 13.3 | **forge kit** (each repin, its crate removals), shared |
| `tests/client/{Cargo.toml, src/front.rs, src/fixtures.rs, src/translate.rs}` | 06.1, 06.3 to 06.5; 06.2 `src/front.rs` (ruling 113) | **forge kit**'s temper-02.1, not landed at the start: the later landing rebases |
| `crates/temper/tests/support/configuration.rs`, copied by 08.2 into `tests/real-loop/src/` | 08.2 (the copy); 10.1 and 13.3 read it | **browser-1**'s 01.1 adds `"bundle": null` to every configuration fixture: whichever lands later carries the field into the world's copy |
| `crates/temper/tests/support/{forgejo.rs, mod.rs}`, `crates/temper/tests/process.rs` | 10.1, 10.2, 13.3 | within the plan: 10.2 and 13.3 on disjoint lines; the second to land rebases, neither waits (temper 13.3) |
| `crates/temper/{Cargo.toml, src/lib.rs, src/tests.rs}` | 05.6; 08.2 `src/{lib.rs, tests.rs}` (`envelope_in`, ruling 104); 13.3 `Cargo.toml` alone | within the plan: the second to land rebases |
| `tests/real-loop/src/` | 08, 09, 13 | within the plan: 08 owns the world's fixtures, 09 and 13 add to them (`code-organization.md`) |
| skein's fake LLM (`skein-fake-peers`, its domain and protocol) | none: read only (08.1's `llm::scripted`) | **live text**, **LLM accounts** (each a smith repin); this plan moves no pin |

The stale comments go with the steps that touch their files: temper
14.1 rewrites `.config/nextest.toml`'s "arrives in 15.4" comment; 10.2
gives `Throwaway`, in `support/forgejo.rs`, a doc it lacks; and the
comments that cite the moved design documents bare go to `client/tools.md`
and `domain/testing.md`: the tools crate's nine in 05.5,
`support/mod.rs`'s landing bound in 10.1. Three stay, as no step touches
their files: `tests/system/src/lib.rs`'s "jig's fake store",
`tests/worker/checkout/src/forge.rs`'s and
`scripts/proof-one-process.sh`'s bare `testing.md`.

No step edits skein's `skein-fake-peers` (its LLM or forge peer), skein's
fake LLM's domain or protocol, smith's crates, `tests/one-process`, or
`testing/temper-fake-git`'s transport, but temper-13.s1 (ruling 112):
the one-process world's cost, in `tests/one-process` and fake git's
`src/`, which the forge kit's temper-05.2 also changes: the later landing
rebases.

## Prerequisites the run checks at its start

The run's machine is debian-16gb-hel1-1. The coordinator runs each check
once, before the first session (3 when its step becomes ready), and
writes `blocked.md` (kind: process) on a failure:

1. `cargo nextest --version` prints a version.
2. `git -C ~/src/rust/temper log origin/main
   --format='%(trailers:key=Plan,valueonly,separator=%x20)/%(trailers:key=Step,valueonly,separator=%x20)'
   | grep -c -x 'tools/temper-repin'` prints 1: the tools plan, part 1,
   has ended (other plans have their own `temper-repin`).
3. **When temper 10 becomes ready:** `test -x "$TEMPER_FORGEJO"` and
   `test -x "$TEMPER_FORGEJO_RUNNER"`: its process proofs start a
   throwaway Forgejo and its runner.

## Live services

None. The real loop runs skein's fake LLM and forge peers on loopback;
the process tier runs a fake LLM and a throwaway Forgejo on loopback; no
agent signs in anywhere.

## Process tier

In a checkout of temper's `origin/main`, outside its gate: `scripts/proof-one-process.sh`, with `TEMPER_FORGEJO` and
`TEMPER_FORGEJO_RUNNER` set. temper 10 adds five tests, about 12 s
expected (temper 10's table): the tier goes from 152.9 s at the base
(`findings/base-times.md`) to about 165 s, against its 3 min aim
(testing.md 2.6). temper 13.3 moves the landing's scripts and
configuration into the real-loop world and adds no test; it runs the
tier once.

## Repin

Nothing: the plan moves no pin (decisions.md, 3). Its forge stories
start on the pins forge-kit's temper-05.2 leaves.

## Kit steps

None: the plan changes no `scripts/dev`.

## Pulse

forge-kit lands on the same `main`, so the aims cover the packages this
plan's sessions change, measured as their tests' summed times
(development.md, 5.2): `temper`, `temper-client-tools`,
`temper-client-world`, `temper-engine-domain-world`, and the new
`temper-client-shell-cli` and `temper-real-loop-world`. At the base
(`findings/base-times.md`) they sum to 9.90 s focused and 11.86 s fuzzy,
and the sums swing by 3.6 s and 2.6 s between gate runs. The run may add
what its Proof bounds sum to (development.md, 5.6):

- **Focused, 26.4 s.**
  - temper 05's units and 05.5's renderer tests: about 45, each under
    20 ms, 0.9 s.
  - temper 06's 14 client stories, each under 0.5 s: 7 s.
  - temper 08.1's and 13.1's five world tests, each under 0.5 s: 2.5 s.
  - Sixteen real-loop stories (08.2 to 08.4, 09, 13.2, 13.4, 13.5),
    each at its 1 s aim (testing.md, 6.1): 16 s, their Proof bounds
    summed; 6.1 guides their idle sum to 6 s (Rules, "The worlds'
    sums"), and this aim stays as C set it (decisions.md, For the
    user).

  Expected: about half. The one-process world's nearest stories take
  0.25 s to 0.9 s.
- **Fuzzy, 10 s.** temper 09 and 13 may move a story past 1 s into
  `fuzzy_restart.rs` or `fuzzy_landing.rs`, each about 5 s. temper 14
  shards the walking story, which takes it off the suite's critical path
  but not off its summed time (decisions.md, 12).

No session is given a share of either aim: the summed times swing by
more than any one session's add except the real loop's, and each of its
stories has its own bound, which its increment measures.

```
words: legacy|compat|hack|workaround|actual|genuine
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
aims: temper 37 22 temper temper-client-tools temper-client-world temper-engine-domain-world temper-client-shell-cli temper-real-loop-world
```

## Done when

Each command runs once, after `done.md`, reading `origin/main`, after
`git -C ~/src/rust/temper fetch`. Each prints nothing, or what its item
says.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. Each session's own "Done when" holds: the commands under that heading
   in `temper/05-ctl.md`, `06-client-stories.md`, `08-real-loop.md`,
   `09-loop-stories.md`, `10-process.md`, `13-landing.md` and
   `14-fuzzy-walking.md` print what they say.
3. No test this plan adds is named with `_and_` (the tools plan's
   retrospective, proposal 6); prints nothing:

   ```
   git -C ~/src/rust/temper diff f1a57db2b origin/main -- crates/temper-client-shell-cli tests/real-loop crates/temper/tests crates/temper/src/tests.rs tests/client crates/temper-client-tools/src tests/engine/domain/tests/fuzzy_walking.rs | grep -E '^\+ *fn [a-z0-9_]*_and_[a-z0-9_]*\('
   ```

4. The process tier passes within its 3 min aim (testing.md, 2.6): in a
   clean checkout of temper's `origin/main`, with `TEMPER_FORGEJO` and
   `TEMPER_FORGEJO_RUNNER` exported, prints one `Summary` of at least 24 tests
   passed (the base's 19 and temper 10's five), under 180 s:

   ```
   sh -c 'export NEXTEST_DOUBLE_SPAWN=0; scripts/proof-one-process.sh 2>&1 | grep Summary'
   ```

5. The test-time aims, measured once in a clean checkout of temper's
   `origin/main` on the run's machine: three gate runs of each suite,
   then the plan's packages' summed test times per run. The medians
   against the Pulse's aims. An aim missed is recorded with the three
   slowest of those tests by name in decisions.md's rulings, for the
   user; the run is done either way.

   ```
   sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace >focused-$i.log 2>&1; cargo nextest run --workspace --profile fuzzy >fuzzy-$i.log 2>&1; done'
   for k in focused fuzzy; do for i in 1 2 3; do sed -nE 's/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^: ]+)[: ].*/\1 \2/p' $k-$i.log | awk -v run=$k-$i '$2 ~ /^temper(-client-tools|-client-world|-engine-domain-world|-client-shell-cli|-real-loop-world)?$/ {s += $1} END {printf "%s %.2f\n", run, s}'; done; done
   ```

   Then the worlds' sums below the domain (testing.md 6.1; Rules, "The
   worlds' sums"), four runs after one build, the last three counted:
   the median of the other worlds' sums against 4 s, the median of the
   real loop's against 6 s, and each real-loop story against its 1 s: a
   story the third command names as past its 1 s in two of the three runs
   is past it. A sum past its guide, or a story past its 1 s, is recorded
   with the three slowest of those tests by name in decisions.md's
   rulings, for the user; a real-loop median at 6 s or past it puts the
   ceiling to the user (6.1). The run is done either way. A run's line of
   the third command fails, and says so, when it reads no test of either
   part:

   ```
   P='-p temper-client-shell-terminal -p temper-client-world -p temper-engine-protocol-forge-world -p temper-one-process-world -p temper-real-loop-world -p temper-worker-protocol-git-world'
   sh -c "cargo nextest run $P --no-run" && sh -c "export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3 4; do cargo nextest run $P --profile measure -j 1 >below-\$i.log 2>&1; done"
   for i in 2 3 4; do sed -nE 's/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^ ]+) ([^ ]+).*/\1 \2 \3/p' below-$i.log | awk -v run=below-$i '{split($2, b, ":")} b[1] != "temper-real-loop-world" {o += $1; n++; next} {r += $1; m++} $1 > 1 {printf "%s: %s %s %.2f s, past its 1 s\n", run, $2, $3, $1} END {if (!n || !m) {print run ": no test read of the other worlds or of the real loop"; exit 1} printf "%s: the other worlds %.2f s against 4 s, the real loop %.2f s against 6 s\n", run, o, r}'; done
   ```

6. Nothing outside these items is checked.
