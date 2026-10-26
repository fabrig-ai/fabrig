# protocol: temper's protocol layer, standalone

2026-10-14. Start commit: temper `cc2697cf`, then this plan's two
Markdown landings (`Design: close the gaps the protocol plan's code
audit found`, and the plan itself); each session starts from temper's
`origin/main`. jig `d91a89c4`, smith `dfc0fe57` and skein `b5d3a2c3`
are temper's pins, read only.

The design of temper's protocol layer, standalone mode, is landed
(`protocol.md`, the documents its section 2 lists, and `STATUS.md`).
This plan builds it, in the order of protocol.md, section 12: the
codecs; `temper-service` on jig's reference service; `temper serve` and
`temper agent`; the client root and view, joined to the engine's root;
a chat answered by the real binary; the forge component, through bytes
and against a real Forgejo; the adoption form; a change landed on the
throwaway Forgejo with real git; the terminal and `temper open`;
`temper sign-in`; the one-process world; the live tier; and the web
crates retired. It also makes the domain changes the design owns
(protocol.md, section 5, "Kept"), and moves temper's engine worlds onto
jig's real store (store/README.md, section 6). It changes temper only.

## Inputs

- **The design,** on temper's `main`: `protocol.md` first (its sections
  5, 12 and 13), then `configuration.md`, `credentials.md`, `llm.md`,
  `channel.md`, `git.md`, `forge.md`, `store/README.md`, `client.md`,
  `domain/testing.md`, `performance.md`, `domain/*.md` and `STATUS.md`; jig's,
  smith's and skein's documents they cite, at the pins.
- **skein's foundation:** `development.md` (this plan follows it),
  `programming-model.md`, `testing-strategy.md`.
- **temper's `AGENTS.md` and `docs/development/workflow.md`** (the gate,
  the suites' budgets).
- **`decisions.md`:** what this plan settled, and the user's choices.
- **`findings/`:** the planner's code audit, one file per area, and
  `findings/README.md`, which maps each finding to the increment that
  acts on it.
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and in its file. Each increment also builds on
the ones before it in its session. Dependencies are named by
capability, with the increment that lands it.

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-domain-parts.md` | temper 01: an undecodable call relayed, a failed preparation reported, the worker's parts by tag | nothing | |
| `temper/02-adoption.md` | temper 02: locate, and the adoption as the forge's own keyed request, kept durably | nothing | |
| `temper/03-wire-codecs.md` | temper 03: temper's parts on jig's channel, and the forge family's client records | nothing | |
| `temper/04-store-codecs.md` | temper 04: temper's families in jig's store | nothing | 04.2: 03.1's `tests/codecs` world; 04.4b: the `Application` family, once 02.4's `Stored::Application` has landed (else 02.4 adds its codec) |
| `temper/05-forgejo-tools.md` | temper 05: the throwaway Forgejo, its transcripts and facts, git's transcripts | nothing | |
| `temper/06-forgejo-documents.md` | temper 06: Forgejo's documents, both sides | 05.2's transcripts; 03.1's `tests/codecs` world | 06.4: 05.3's Actions transcripts |
| `temper/07-git-component.md` | temper 07: the git component and its machine world | nothing | 07.2: 05.4's git transcripts |
| `temper/08-service-engine.md` | temper 08: `temper-service`'s engine (nine increments) | 04.5's store codecs | 08.3: 01.2's `Parts.preparation`, 03.1's records; 08.4: 01.1's relay; 08.5b: 02.3's adoption, 03.2's records |
| `temper/09-service-worker.md` | temper 09: `temper-service`'s worker, and one process | 08.3's translations and fixtures; 01.3's parts by tag; 03.1's records; 07.1's `Identity` | 09.2: 07.3b's component; 09.3: 08.5b, the engine whole |
| `temper/10-startup-and-binary.md` | temper 10: configuration's startup, `temper serve` and `temper agent`, the process tier's support | 09.3's one process | |
| `temper/11-client.md` | temper 11: the client root and view, joined to the engine's root; the web crates retired | nothing | |
| `temper/12-native-client.md` | temper 12: the native client, and a chat answered by the real binary | 11.2's view root; 03.2's records | 12.2: 10.4's process support |
| `temper/13-forge-component.md` | temper 13: the forge component and its machine world | 06.5's documents | 13.3: 02.1's `Read::Locate` |
| `temper/14-forge-worlds.md` | temper 14: the forge component wired, the fake forge's protocol layer, the protocol world, the conformance run through the component | 13.5's component; 08.5b's engine | 14.5: 05.3's facts |
| `temper/15-adoption-and-landing.md` | temper 15: the adoption form, a change landed on the throwaway Forgejo, the live tier | 12.1's native client and routing; 03.2's records | 15.2: 15.1b's form, 11.3's joint world, 02.3's adoption; 15.3: 14.1, 09.2, 10.4, 12.2; 15.4: 15.3 |
| `temper/16-terminal.md` | temper 16: the terminal shell, `temper open`, `temper sign-in` | 12.1's native client | 16.4: 10.3's binary, 10.4's process support, its Proof after 12.2 (ruling 138); 16.5: 10.1b's startup, 10.4, 12.2; 16.6: 15.1b's form, 15.3's landed change |
| `temper/17-one-process.md` | temper 17: the fake git program and the one-process world | 14.3's fake forge protocol; 12.1's native client; 10.1b's startup; 07.2's transcripts world | 17.3: 15.1b's form; 17.4: 02.4's durable outcome |
| `temper/18-real-store-worlds.md` | temper 18: the engine worlds on jig's real store; the system world on the service's translations | 04.5's store codecs; 08.2b's store driver | 18.3: 08.3's translations and charter, 09.1a's worker translations |

81 increments in all. The domain's changes, the codecs and the client
come first, since the service, the forge's worlds and the tiers above
them wait on them.

### How many at once, and in what order

- **At most three sessions run at once** on this machine (`x1`: 8 cores,
  15 GB, about 8 GB free; pueue's gate lane 1, build lane 2). The
  coordinator holds new starts below 6 GB of memory or 30 GB of disk
  (development.md, 11.4's 40 GB lowered for x1 by decisions.md, ruling
  128).
- **At the start,** seven sessions need nothing: 01, 02, 03, 04, 05,
  07 and 11.
- **When slots are short, start first** the sessions others wait on, in
  this order:
  1. 05, 04, 03;
  2. 01, 07, 06 (once 05.2 has landed), 02;
  3. 08 (once 04.5 has landed), 11;
  4. 09, 13, 18;
  5. 10, 14, 12;
  6. 15, 16, 17.
- **The longest chains:** 04 → 08 → 09 → 10 → 12.2 → 15.3 → 16.6 (26
  increments); and 05 → 06 → 13 → 14 → 17 (23).
- **Session 08 holds nine increments,** copied from jig's reference
  service; its author hands over by context between them (Rules).

## Rules of this plan

- **Every session reads,** besides its "Read first": temper's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the rulings of `decisions.md` that name its
  session (its numbered decisions and the rulings from 101). Nothing
  else is read whole.
- **The code graph** is the codebase-memory server's project
  `home-free-src-rust-temper`; never call `list_projects`. Its jig, smith
  and skein projects index those repositories' newer mains: read their
  code at the pins instead, in `~/.cargo/git/checkouts/jig-aba39083bf77fefe/d91a89c`,
  `~/.cargo/git/checkouts/smith-5ef4a620eb985fe2/dfc0fe5` and
  `~/.cargo/git/checkouts/skein-336d5ae5eecfa0f9/b5d3a2c` (after the
  first build in a worktree, cargo keeps them there).
- **Keep the pins** (code-organization.md, section 3): an increment that
  adds a crate of jig, smith or skein that `Cargo.lock` does not hold
  yet runs the three `cargo update --precise` commands, in that order,
  and the check prints exactly the three pinned revisions before it
  lands.
- **Large files.** These 16 files were past 800 lines at the base:
  `tests/engine/domain/tests/root_routes.rs`,
  `crates/temper-engine-domain-forge/src/domain.rs`,
  `tests/engine/domain/tests/forge_routes.rs`,
  `crates/temper-engine-domain/src/route/forge_route.rs`,
  `testing/temper-fake-forge-domain/src/tests.rs`,
  `tests/engine/forge/tests/simulation.rs`,
  `crates/temper-web-domain/src/domain.rs`,
  `crates/temper-engine-domain-forge-client/src/keep.rs`,
  `tests/worker/checkout/src/world.rs`,
  `crates/temper-worker-domain-checkout/src/hold.rs`,
  `crates/temper-worker-domain-checkout/src/tests.rs`,
  `crates/temper-engine-domain-forge-client/src/outbox.rs`,
  `tests/engine/domain/src/escalation.rs`,
  `tests/engine/domain/src/walking.rs`,
  `crates/temper-engine-domain-forge-client/src/api.rs`,
  `tests/engine/forge/client/tests/simulation.rs`.
  An increment adds at most 40 lines to each (`git diff --numstat
  origin/main...HEAD -- <file>`, its first column); its new code goes in
  a new module beside it, which is inside its Files. Their splits are
  `later.md`'s.
- **How the pulse reads them.** `plan pulse`'s "over 800 lines" lists
  every file a landing touched that is past 800 lines on `main`: these
  16, and generated files under `src/generated/`, show there whenever
  they are touched. Neither is a finding by itself. The hourly check
  reads the list against this rule: a file of these 16 that grew past 40
  lines in one step, or a hand-written file not among them, is one.
- **A durable record's codec follows its record.** Once 04 has landed a
  family's codec, an increment that changes a `Stored` variant or a
  value it holds changes the schema, the generated code and the
  translation in the same increment; the translation's build breaks
  otherwise, which is the check. Those files are in that increment's
  Files: `crates/temper-protocol-store/{schema/*.schema,
  src/generated/v1.rs, src/records/*.rs, src/tests/*.rs}` and its
  goldens, the hand-written lines counted in its Size (decisions.md,
  ruling 110). `STORE_VERSION` stays 1 in this plan (no store of
  temper's exists yet).
- **Edits outside an increment's Files** that are no question (jig's
  `jig-protocol` plan learned each, rulings 108 to 262; they replace
  development.md, 8.2's "it is in your session's own files" and "it
  changes no test's expected value or name" for these cases only):
  - an arm each exhaustive match needs for a variant the increment adds,
    in any crate, at most 5 lines per match;
  - a fix the change, its Proof or the gate needs, at most 20 lines, in
    any session's files, every test's name and expected value kept (a
    fixture's inputs may change so the test checks the same behaviour);
    in an increment whose Size is world code only, those production
    lines count against 20 (decisions.md, ruling 120);
  - an expected literal that prints a vocabulary the Shape renames, at
    most 10 lines in all, replaced by what the new code prints;
  - an expected value for the very case the Shape changes, at most 4
    assertions, none loosened or removed;
  - a removed mechanism's tests, moved to its replacement or removed;
  - code that moved since the plan was written, followed to its file;
  - the users of a type the Shape changes, edited to its new form with
    no other change, every assertion, input, expected value and name
    kept, in any number of lines (code-organization.md, 6: "it changes
    the type in place, and its users in the same increment";
    decisions.md, ruling 123);
  - the fixtures a Test shape names, in the crate's fixture module;
  - an item made visible within its own crate;
  - the session's own earlier code, replaced or extended: a file its
    own earlier increments created is in each later increment's Files,
    and an expected literal there gains a field the Shape adds, with the
    value its transcript or fixture gives, no other value changed
    (decisions.md, ruling 108);
  - a fixture that gains what a real component requires (18's real
    store), its expected values literals built from the fixture's
    constants, none loosened or removed.

  Each is named in the commit body. Anything else outside Files is a
  question.
- **Size:** an increment lands within twice its Size without asking; a
  codec or document increment (03.x, 04.x, 06.x, and 02.4's family part)
  within 2.5 times it; the parts of a split increment count together;
  each report names "Size: <increment> <count> of <allowance>"
  (code-organization.md, section 8, which names the count's command;
  decisions.md, ruling 106, which replaced 1.5 and twice).
- **Waiting.** An author waits on a running command with one wait
  of 300,000 ms on its cell, as development.md, 7.4 says for the
  coordinator; it never sends empty input to a running command, and
  never runs `ps` or `pueue status`.
- **A context past 250,000 tokens inside an increment** commits its work
  on its branch, writes its checkpoint (the files, fixtures and traps,
  never the standing rules), and hands over.
- **A commit's last paragraph is its trailers:** `Proof:`, `Plan:` and
  `Step:`, one per line, no blank line between them. Before merging,
  `git log -1 --format='%(trailers:key=Step,valueonly)' HEAD` prints the
  step.
- **Each report names the suites' times** from its landing's
  `Gate-Result` (focused and fuzzy), so the hourly check follows them.
- **The throwaway Forgejo** is Forgejo v16.0.5, downloaded from
  codeberg.org's release page and verified, at
  `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64`;
  before relying on it, `cd ~/src/rust/tmp/temper/forgejo/v16.0.5 &&
  sha256sum -c forgejo-16.0.5-linux-amd64.sha256` prints `OK` (SHA-256
  `2c548bf1bfa09a441a3a674b1dd4959d77576816ad50534c23496908a3ea5a19`).
  The Actions facts' runner is
  `TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v12.13.2/forgejo-runner-12.13.2-linux-amd64`
  (`sha256sum -c forgejo-runner-12.13.2-linux-amd64.sha256` in its
  directory). Nothing else is a Forgejo of this plan (code-organization.md,
  section 6).
- **The process and live tiers run only where a Proof line or "Done
  when" says,** once; never in the gate.
- **A Proof's time bound** is the time nextest's `Summary [ N s]` line
  prints, build time aside; a script's is its own wall time.
- **A design gap is a question,** with the design's text quoted
  (development.md, 8.7); the designer answers it.

## Live services

- **An LLM provider, through the user's Codex login,** for the live tier
  only (15.4; domain/testing.md, 2.7). Borrowed read only: `tools/live-login`,
  run by `scripts/proof-live.sh`, reads the access token, its expiry and
  the account's id from `~/.codex/auth.json` and writes them, and
  nothing else, to a private file under `$TEMPER_RUN_LOGS`, removed at
  exit. It refuses a token that expires within 20 minutes. Nobody signs
  in, nothing refreshes the token, nothing writes `~/.codex`.
  - **Budget per run:** at most 25 completions and 15 minutes, the
    design's (domain/testing.md, 2.7); the test fails past 25, the script stops
    the run at 15 minutes. development.md, 5.5's "at most about 5
    minutes" is a guide this tier's design bound replaces.
  - **Budget for the plan:** at most 3 runs and 50 completions. 15.4's
    live run (`scripts/proof-live.sh`) runs once, and its run against
    the fake peer does not count; each further live run, at most two,
    needs a supervisor's ruling, given only when the run failed on the
    model's words rather than on the code.
  - **Before it,** the same stories, fixture and answer check pass in the
    process tier against skein's fake LLM peer (15.3).
- **The throwaway Forgejo** is no live service: a local binary on
  loopback, started and removed by the tools and the process tier, its
  tokens made by the test (Rules, above).
- **Nothing else:** no forge but the throwaway one, and no network peer
  off the machine but the live tier's provider.

## Repin

None: temper is the last repository in the workspace's order
(development.md, 5.7), and this plan changes no other.

## Pulse

```
words: legacy|compat|shim|hack|workaround|TODO|FIXME
types: ^(Legacy|Old|Compat|New|Temp|Temper)[A-Z]|V[0-9]+$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, in a worktree of temper at `origin/main` after
the last step (`plan worktree temper done`), unless it names
`~/src/rust/temper`.

1. **Every step is landed or dropped:** `plan status` lists no step
   `waiting`, `ready`, `running` or `blocked`.
2. **The gate passes on `main` within the budgets:** `git -C
   ~/src/rust/temper log -1 --grep='^Gate-Result: scripts/check.sh passed'
   --format='%h %(trailers:key=Gate-Result,valueonly)' origin/main`
   names the last code step's commit (or a later one), a focused time at
   most 15 s and a fuzzy time at most 60 s. (Markdown landings carry
   `Gate-Result: none (Markdown only)`, which the grep skips.)
3. **The crates and worlds exist:** `for p in crates/temper
   crates/temper-service crates/temper-forge-forgejo
   crates/temper-engine-protocol-forge crates/temper-worker-protocol-git
   crates/temper-protocol-worker-wire crates/temper-protocol-store
   crates/temper-protocol-client-wire crates/temper-client-domain
   crates/temper-client-view crates/temper-shell-terminal
   testing/temper-fake-forge-protocol testing/temper-fake-git
   testing/temper-throwaway-forgejo tools/forgejo-conformance
   tools/git-transcripts tools/live-login tests/codecs tests/client
   tests/engine/protocol-forge tests/worker/protocol-git
   tests/one-process; do test -f $p/Cargo.toml || echo missing $p; done`
   prints nothing.
4. **The web crates are gone:** `git grep -n -e temper-web- -e temper_web_
   -e temper-fake-person -e temper_fake_person -- ':!docs'` prints
   nothing.
5. **No Python is left:** `git ls-files '*.py'` prints nothing.
6. **The pins held:** `grep -oE 'forgetic/(jig|smith|skein)\.git#[0-9a-f]+'
   Cargo.lock | sort -u` prints exactly the three pinned revisions
   (code-organization.md, section 3).
7. **The engine worlds run on jig's real store:** `git grep -n
   jig-fake-store -- '*.toml'` prints nothing.
8. **The process tier passes:** `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v16.0.5/forgejo-16.0.5-linux-amd64
   scripts/proof-one-process.sh` exits 0, its log naming
   each test of `crates/temper/tests/process.rs` passed (10.4's drain,
   15.3's story, 16.4's and 16.6's terminal, 16.5's sign-in).
9. **The live tier ran within its budget:** `git -C ~/src/rust/temper
   log -1 --grep='^Step: temper-15.4$' --format=%B origin/main | grep -E
   '^(Completions|Wall): '` prints `Completions: N` with N at most 25 and
   `Wall: S s` with S at most 900.
10. **Forgejo's facts are recorded:** `test -s
    tools/forgejo-conformance/facts.txt && grep -E '^fails ' tools/forgejo-conformance/facts.txt`
    prints no line that `decisions.md` does not name in a ruling.
11. **The new worlds keep their share of the suites:** `cargo nextest run --workspace --profile measure -j 1 -E 'not
    binary(/^fuzzy_/) & (package(temper-client-world) |
    package(temper-engine-protocol-forge-world) |
    package(temper-worker-protocol-git-world) |
    package(temper-one-process-world) | (package(temper-shell-terminal) &
    kind(test)))'`, then `awk -F'[][]' '/PASS \[/ {s += $2} END {printf
    "%.1f\n", s}' <its log>`, prints at most 4.0; the same with
    `--ignore-default-filter` added and `binary(/^fuzzy_/)` in place of
    `not binary(/^fuzzy_/)` prints at most 25.0.
12. **The code rules hold:** `git grep -nE 'env::var|var_os|option_env!'
    -- '*.rs'` names only the variables of code-organization.md, section
    1; `git grep -nE 'thread::(spawn|scope)' -- '*.rs'` prints nothing; `git diff cc2697cf
    origin/main -- Cargo.lock | grep '^+name = '` names only temper's own
    crates, crates of jig, smith and skein, and `serde_json` with its
    dependencies (`serde`, `serde_core`, `itoa`, `ryu`, `memchr`).
13. **The aims above are these items;** the STATUS rows are the
    supervisor's and the designer's, and no other aim is checked.
