# client-api: jig's client API, and temper on it

2026-10-15. Start commits: jig `a3b53546` (on `28901d98`, "Design jig's
client API: plumbing, sugar, and the application's server"), temper
`c7095516` ("Stand temper's client on jig's client API"), skein
`ecccbac`; each session starts from its repository's `origin/main`.
smith is read only: jig keeps its pin of smith, `29fa97c9`, and temper
moves to it (decisions.md, 4).

jig's client becomes presentation-agnostic: jig's primitives as
plumbing operations, shapes of work as sugar the client fills in, the
server and the URL space the application's. The plan builds the six
steps of jig's `docs/design/STATUS.md`, section 3, "jig's client API":
(1) the client crates renamed by the naming rule, (2) the engine's
plumbing, (3) the wire collapsed, (4) the client, `jig-client`, with
the reference UI on it, (5) the server, (6) the browser. It adds a check
that keeps every client crate under its project's `-client` prefix,
in jig and in temper. temper follows each step that breaks it, builds
what its gate needs of "temper on jig's client API" (temper's
`STATUS.md`), and ends pinned to jig's last landing. One skein increment
gives skein's browser harness what jig's attack stories need.

## Inputs

- **jig's design,** at `a3b53546`: `docs/design/STATUS.md`, "jig's
  client API" (the steps, the renames, "What the code does against the
  design"), and each row naming "(jig's client API)"; the sections each
  session's "Read first" names, in `README.md` (1, 10, 11, 15),
  `client/domain/*.md`, `domain/*.md`, `protocol/*.md` and
  `examples.md`.
- **temper's design,** at `c7095516`: `docs/design/STATUS.md`, "temper
  on jig's client API" and its rows; `client.md`, `protocol.md`,
  `configuration.md`, `credentials.md`, `domain/testing.md`, by the sections
  each temper session names.
- **skein's design:** `ui-web.md`, 14.2 (skein 01).
- **The workflows:** jig's and temper's `docs/development/workflow.md`,
  1 and 2; skein's `AGENTS.md` and `testing.md`, 7.
- **`decisions.md`:** the plan's decisions, and what the user may veto.
- **`findings/`:** the code audit at the base, one file per area
  (`renames.md`, `engine-parties-and-tasks.md`,
  `engine-agents-store-config.md`, `wire-and-client.md`,
  `server-and-browser.md`, `temper.md`), and `measurements.md` (the
  suites at the base).
- **`later.md`:** what the designs name that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"jig 03" means that session's last increment has landed on jig's
`origin/main`.

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01-renames.md` | jig 01: the client crates renamed, and the naming rule's check | nothing | |
| `jig/02-task-fields.md` | jig 02: a task's label, flags and last activity; `AsAllowed`; the escalation holder; a task's goal | nothing | |
| `jig/03-parties.md` | jig 03: parties' `Create`, `Propose` and `Withdraw`; proposing made explicit | jig 02 | |
| `jig/04-grace-and-compound.md` | jig 04: cancel with a grace, `Move`'s check, tracked and priority amended, goals per project, the compound request | jig 02.1 | 04.3 to 04.7: jig 02.3 |
| `jig/04-s1-numbered-by-place.md` | jig-04.s1 (ruling 184): a compound's operations numbered by place | jig 04.7 | |
| `jig/05-me-and-queries.md` | jig 05: waiting on, the task line, task queries, `me`, conversations by their flag | nothing | 05.2, 05.5: jig 02.1 |
| `jig/06-agents-tools.md` | jig 06: the agents' tools name revisions | nothing | 06.3: jig 02.1; 06.4: jig 03; 06.5: jig 03, jig 04.5 |
| `jig/07-connectors.md` | jig 07: connectors' topics and procedures, the party's own scope and windows, untracking | nothing | 07.5: jig 04.5; 07.6: jig 02.3 |
| `jig/08-store-indexes.md` | jig 08: the ended indexes, ended read positions, task queries' ended reads | jig 02.1 | 08.3: jig 03.1; 08.4: jig 02.3; 08.5: jig 05 |
| `jig/09-templates.md` | jig 09: templates and policies' overrides, operations per request | jig 02.1, jig 04.1 | 09.2: jig 02.3; 09.3: jig 05; 09.4: jig 04.6 |
| `jig/10-transport.md` | jig 10: `jig-client-protocol-http`, the native shell on it, the link's vocabulary | jig 01 | |
| `jig/11-client-domain.md` | jig 11: `jig-client-domain`: the contract, handles and askers, the request table, kept work | jig 01 | |
| `jig/12-client.md` | jig 12: `jig-client`: the link's states, `me`, `SignIn { with }`, families as bytes, the thin story against the real engine | jig 10, jig 11 | 12.2: skein-01.s3 |
| `jig/12-s1-watch-waits-for-sign-in.md` | jig-12.s1 (ruling 246): a watch asked before signing in waits for it | nothing (lands before jig 21.3) | |
| `jig/13-reference-ui.md` | jig 13: the reference UI on the public client; `ops`'s and the test application's roots by asker; `jig-domain-client` gone | jig 12 | |
| `jig/13-s1-path-segments.md` | jig-13.s1 (ruling 143): a path's segments bounded apart from the rows; the view cuts a long path | jig 13.2 | |
| `jig/13-s2-scripted-wire.md` | jig-13.s2 (ruling 205): the test client wire carries the scripted family's records | jig 13.3 | |
| `jig/14-wire.md` | jig 14: the wire collapsed, its requests: the version refusal, scopes and windows, the operations, the compound request | jig 03 to 09, jig 21.2; lands nothing before jig 21's last landing (the fence below; ruling 247) | |
| `jig/14-s1-words-release.md` | jig-14.s1 (ruling 210): words release nothing; the words' release leaves the engine | jig 14.3 | |
| `jig/14-s2-focused-suite.md` | jig-14.s2 (ruling 269): jig's focused suite cheaper, beside jig 14 | nothing (beside jig 14.4) | |
| `jig/22-documents.md` | jig 22: the wire's documents: `me`, the task line and queries, offers and a decision's effect | jig 14 | |
| `jig/22-s1-text-parameters.md` | jig-22.s1 (ruling 399): a task's words and byte parameters refused when they are not UTF-8 | nothing | |
| `jig/15-helpers.md` | jig 15: `jig-client`'s helpers: `shapes`, `facts`, `thread`, `words` | jig 22 | 15.2, 15.3, 15.4: 15.1; 15.5: 15.2; 15.6: 15.3, 15.5 (ruling 248) |
| `jig/15-s1-facts-in-words.md` | jig-15.s1 (ruling 394): a decision's facts in `facts`, their words in `words`, the referee in the application's phrasebook; the joint transport on `jig-client`'s limits | nothing (beside jig 20.5) | |
| `jig/21-ops-and-ui.md` | jig 21: `ops` and the reference UI's state on the client alone; `ops`'s wire crate | jig 13 | |
| `jig/16-shed.md` | jig 16: the shapes of work leave the engine and the wire; the reference UI's lists on task queries | jig 15.1 to 15.3, jig 21 | 16.3: 15.6; 16.5: every jig 15 step (ruling 248) |
| `jig/16-s1-read-positions.md` | jig-16.s1 (ruling 305): a conversation's read position named for no shape | nothing (lands before temper 04.3) | |
| `jig/17-engine-http.md` | jig 17: `Mint`, the link's secret, `jig-protocol-engine-clients-http` and its guards | jig 16 | |
| `jig/17-s1-authority-whole.md` | jig-17.s1 (ruling 319): an authority crosses whole in the engine's documents, the link and the view | nothing (not part of jig 17's last increment, 17.5) | |
| `jig/17-s2-busy-watches.md` | jig-17.s2 (ruling 319): the client opens a busy watch and `me` again, closes a watch once its opening is answered, keeps one watch for `me` | nothing (not part of jig 17's last increment, 17.5) | |
| `jig/17-s3-unwatch-answered.md` | jig-17.s3 (ruling 372): an unwatch of an ended watch is answered | nothing (beside jig 19.5 and jig 20) | |
| `jig/18-server.md` | jig 18: the test application's server owns the listener, mounts `/jig`, serves its bundle | jig 17 | |
| `jig/19-browser.md` | jig 19: the browser parts, the test shell, the bundle and the browser tier | jig 18 | 19.4, 19.5: skein 01; 19.4: skein-01.s2 |
| `jig/20-attack-stories.md` | jig 20: jig's stories for a browser shell, and its attack stories | jig 19 | |
| `jig/23-referee.md` | jig 23: the conformance referee's rules for proposals and revisions | jig 03 | 23.2: jig 06.1, jig 06.2, jig 07.3 |

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-harness.md` | skein 01: the harness opens the printed URL, serves other origins, awaits trouble | nothing | |
| `skein/01-s1-terminal-room.md` | skein-01.s1 (ruling 134): the terminal's room for the application's lines | nothing | |
| `skein/01-s2-terminal-price.md` | skein-01.s2 (ruling 143): the terminal's worst case, operations and line bytes | nothing | |
| `skein/01-s3-not-kept.md` | skein-01.s3 (ruling 170): the request table retires a request not kept, by its key | nothing | |
| `skein/01-s4-close-after-closed.md` | skein-01.s4 (ruling 173): no Close after Closed in the browser harness's HTTP client | nothing | |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-skein-ui-repin.md` | temper 01: temper on jig's base with skein-ui; its client crates renamed; the terminal on `skein-ui-text` | nothing | |
| `temper/02-names.md` | temper 02: jig's renames followed, and the naming check | jig 01, temper 01 | |
| `temper/03-client.md` | temper 03: temper's client on `jig-client` | temper 02 | 03.1: skein-01.s1, skein-01.s2, forgejo-17's skein step; 03.2: jig 10, temper-03.s1; 03.3: jig 11; 03.4: jig 13.2; 03.5: jig 21.2, jig-12.s1; 03.6: jig 21.4 |
| `temper/03-s1-engine-revisions.md` | temper-03.s1 (ruling 153): the repin onto jig 10.3, with jig 06's revisions and answers in temper's engine tools | jig 10.3 | |
| `temper/04-engine.md` | temper 04: the engine and the forge's family on templates and the collapsed wire | jig 09, temper 03 | 04.1: temper 03.4 (ruling 231); 04.2: jig 22; 04.3: jig 16; 04.4: jig-13.s1 |
| `temper/04-s1-repin-rooms.md` | temper-04.s1 (ruling 319): the repin onto jig-17.s1 and jig-17.s2; a stream's watch room 16 | jig-17.s1, jig-17.s2, temper 04 | |
| `temper/05-server.md` | temper 05: `temper-service` owns the clients' listener and mounts jig's component | jig 17, temper 04 | 05.2: jig 18.1 |
| `temper/05-s1-landing-story.md` | temper-05.s1 (ruling 347): the process tier's landing story passes again, past the regressions of 03.5, 04.3 and 05.1 | temper-04.s1 (ruling 352) | |
| `temper/05-s2-process-tier.md` | temper-05.s2 (ruling 367): the whole process tier passes, past the signed delivery's stop and each test the 13:49Z run did not reach | temper-05.s1 | |
| `temper/06-pages.md` | temper 06: the frame, the inbox and the chats, temper's own | temper 04 (its repin past jig 16 brings jig 15) | 06.5: temper-05.s2, jig-17.s3 (ruling 372) |
| `temper/07-task-pages.md` | temper 07: the task's page, adoption, the confirmations; `jig-client-view` dropped | temper 06 | |
| `temper/07-s1-own-cards.md` | temper-07.s1 (ruling 394): temper's own cards after its mockups; `jig-client-view` dropped | temper 07, jig-15.s1 | |

The closing repin, `temper-repin` (development.md, 5.7), has no file:
it starts once every jig and skein step has landed and temper 07 and
temper-07.s1 have (ruling 394), jig-22.s1 among them (ruling 399),
and moves temper's pins of jig, skein and smith to jig's (Rules).

31 sessions and the repin: jig 23, skein 1, temper 7. jig 21, 22 and 23
are numbered last because they were added after the others: jig 21 and
22 split from jig 13 and jig 14 (jig 21 runs after jig 13, jig 22 after
jig 14), jig 23 from the design trace's referee rules. The increments
per session are in each file.

### How many at once, and in what order

- **As many sessions at once as dependencies allow, up to eight** (the
  coordinator's Codex profile refuses a ninth thread, development.md,
  7.2), held only by development.md 11.4's memory and disk bounds, which
  the coordinator checks before each start (decisions.md, 16). The build
  lane's width stays as the kit sets it (2 tasks; the gate lane 1;
  development.md, 11.1).
- **At the start,** the seven that need nothing: jig 01, jig 02, jig 05,
  jig 06, jig 07, skein 01, temper 01; the eighth slot goes to the first
  session whose Needs land.
- **Then,** as each lands: jig 10 and jig 11 once jig 01 has; temper 02
  once jig 01 and temper 01 have;
  jig 04 and jig 08 once jig 02.1 has; jig 03 once jig 02 has; jig 09
  once jig 02.1 and 04.1 have; jig 12 once jig 10 and 11 have; jig 13
  once jig 12 has; jig 21 once jig 13 has; jig 14 once jig 03 to 09 and
  jig 21.2 have (it lands after jig 21, the fence below); jig 22 once jig 14 has; jig 15 once jig 22 has: 15.1 first, then 15.2, 15.3 and 15.4 each once 15.1 has,
  in slots of their own, 15.5 once 15.2 has, 15.6 once 15.3 and 15.5 have;
  jig 16 once jig 21 has too: 16.1 once 15.1 to 15.3 have, beside 15.4 to
  15.6; 16.2 once 16.1 has; 16.3 once 16.2 and 15.6 have (15.6's Proof stands
  on the scripted model 16.3 rewrites); 16.4 once 16.2 has, beside 16.3; 16.5
  last, once 16.3, 16.4 and every jig 15 step have (temper 04.3 repins onto
  16.5's landing, and temper 06.1 relies on it holding 15.5's check) (ruling
  248); jig 17, 18, 19, 20 in that order, jig 17 once jig 16 has; jig 23 once jig 03 has; the
  temper sessions as their rows say.
- **The fence:** jig 14 lands nothing before jig 21's last landing (14.1
  is built beside jig 21.3 and 21.4, ruling 247), so that
  no repin of temper 03 (onto jig 10.3, 11.5, 13.2, 21.2, 21.4) takes
  the collapsed wire (decisions.md, 12).
- **When slots are short** (eight running, or 11.4's bounds holding
  starts), **start first** the sessions on the longest chain, then those
  the chain's next session waits on: jig 01, jig 02, jig 11, jig 10, jig
  12, jig 13, jig 21, jig 03, jig 04, jig 06, jig 09, jig 08, jig 05, jig
  07, jig 14, jig 22, jig 15, jig 16, jig 17, jig 18, skein 01 (jig 19
  waits on it), jig 19, jig 20; then jig 23. The temper sessions take a
  slot as soon as their Needs land, before any jig session not on the
  longest chain.
- **The longest chain:** the client track, jig 01 → jig 11 (jig 10
  beside it) → jig 12 → jig 13 → jig 21, then jig 14 → jig 22 → jig 15 →
  jig 16 → jig 17 → jig 18 → jig 19 → jig 20 → `temper-repin`; step 2's
  sessions (jig 02 to 09, the longest jig 02 → jig 03 → jig 06.5, 11
  increments against the client track's 22) run beside the client track
  and land before jig 14.

## Rules of this plan

- **Every session reads,** besides its "Read first": its repository's
  `AGENTS.md` and `docs/development/workflow.md`, this README,
  `code-organization.md`, and the rulings of `decisions.md` that name its
  session. Nothing else is read whole.
- **The process is read in the run's kit** (ruling 331): development.md,
  the implementation rules and testing-strategy.md in
  `/srv/data/git/runner/src/tmp/temper/client-api/kit/docs/foundation/`,
  not in skein's checkout, which has moved since the run's claim.
- **No code graph** (ruling 331): its indexes of skein, smith, jig and
  temper are older than their `origin/main`. Search with `rg` in the
  worktree, and never call the graph's index tools (`list_projects`,
  `index_status`, `check_index_coverage`, `index_repository`), until a
  steer says the indexes are current.
- **jig never names temper:** jig's session files say "the repository
  downstream of jig", and no file of jig's code names it: `git -C <the
  session's jig worktree> grep -n -i -w temper -- crates testing tests
  examples scripts` prints nothing at each jig increment's end
  (code-organization.md, 6). jig's documents are not this plan's to
  check.
- **Pins.**
  - **A step's landing** is the commit `git -C ~/src/rust/<repository>
    fetch origin && git -C ~/src/rust/<repository> log origin/main
    --all-match --grep='^Plan: client-api$' --grep='^Step:
    <repository>-NN.k$' --format=%H -1` prints (older plans reuse step
    names such as `jig-02.3`); for a whole session, its last step's.
  - jig moves skein once, never smith: jig 12.2's first commit (the
    design answer to jig 12.2's question of 2026-10-16T19:46:42Z, a
    request not kept) runs `cargo update -p skein-lib --precise
    <skein-01.s3's landing>` (which holds skein-01.s2's, skein-01.s1's,
    skein 01.3's and forgejo-17's skein step's), in a commit of its own
    with its one repair, `lines: 0, line_bytes: 0` in jig's
    `skein_ui_text::Settings` literal (`git -C ~/src/rust/jig grep -n
    'skein_ui_text::Settings'`: `tests/client-protocol/src/native/world.rs:232`
    at jig `bb977c909`); after it, `grep -o 'skein.git#[0-9a-f]*'
    Cargo.lock | sort -u` prints that one commit, and each later `cargo
    update` of a skein package holds it with `--precise`. jig 19 makes
    no move of its own. A failure in a smith crate is a question.
  - Each temper increment whose Needs name a jig capability starts with
    a repin commit of its own onto the landing of the newest jig step its
    Needs name, never jig's `origin/main`, so that one repin takes as few
    other jig landings as it can. When `git -C ~/src/rust/jig merge-base
    --is-ancestor <that landing> <temper's present jig pin>` succeeds,
    no repin commit is made. Otherwise: `cargo update -p
    jig-domain-core --precise <that landing>`, then `cargo update
    -p skein-lib --precise <jig's pin of skein, or skein-01.s2's landing
    while that pin does not hold it (below)>`, not run when temper's
    skein already holds that commit ("temper's skein ahead", below), and
    `cargo update
    -p smith-host-protocol --precise <jig's pin of smith>` (each read with
    `git -C ~/src/rust/jig show <that landing>:Cargo.lock | grep -o
    'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*' | sort -u`).
    Then `grep -o 'git+https://github.com/forgetic/[a-z]*.git#[0-9a-f]*'
    Cargo.lock | sort -u` prints three lines: `jig.git#<that landing>`,
    and skein's and smith's lines as jig's lock at that landing prints
    them, skein's being skein-01.s2's landing instead while jig's does
    not hold it (below), or a later skein commit that holds it: skein's
    line passes when `git -C ~/src/rust/skein merge-base --is-ancestor
    <that commit> <skein's line>` succeeds. A `cargo update` that leaves a second revision
    of a source is run once more; a second revision still there after it
    is a question.
  - **What a temper repin repairs:** the capability its increment's Needs
    name, by its Shape; and what else jig landed since temper's last
    repin, adapted the way jig adapted its own reference code
    (`jig-client-test-domain`, `jig-test-service`, `ops`), as the commit
    of that change shows, in at most 200 added lines outside the Shape's
    Files: `git diff --numstat origin/main...HEAD -- <the changed paths
    outside the increment's Files> ':!Cargo.lock'`, the first column
    summed once before landing and given in the report. More is a
    question (development.md, 5.7), never a reason to pin a jig older
    than temper's present pin.
    A boundary type of temper's (a field, a variant, a `CoreRange`) that
    carries what jig landed, as jig's reference roots and kit carry it
    (`ops`, `jig-test-service`, `jig_protocol_store`), is such a repair,
    not the implementation rules's question: carry it losslessly, name each
    user in the commit body, and count it in the 200 (rulings 236 and
    237, temper 03.5's blocks of 2026-10-17T08:14:33Z and 08:25:44Z).
    A change jig did not make is still the question.
  - **jig 08.4's census** (the design answer of 2026-10-16): the first
    temper increment whose repin includes jig 08.4's landing (`git -C
    ~/src/rust/jig merge-base --is-ancestor <jig 08.4's landing> <the
    pinned commit>` succeeds) repairs temper's census walk, which jig's
    reference code does not show (temper's `store/README.md`, section 5;
    jig's `protocol/store.md`, 7.2); at the latest temper 04.2's, since
    jig 14 starts after jig 08. Counted within the repin's 200 lines:
    - `crates/temper-engine-domain/src/store.rs`: `census_interval`
      goes, and its callers (`loads.rs`, `census_position`;
      `crates/temper-service/src/store/ranges.rs`, `cursor`) call jig's
      `Census::interval`; `Range::contains` for `Range::Census` admits
      an ended task, not task 0, and an `EndedRead` key the census
      admits;
    - `restart.rs`, `census_page`: each core row goes to
      `restore_inbox_row` as it comes, a forge row refuses;
    - `route/held.rs`: `core::Held::Load(now)`, a mark's load held
      behind the commits before it (the design answer of
      2026-10-16T19:32Z), routes the `Now` it wraps as `route/now.rs:141`
      routes `Now::SeenChat` at the base;
    - the root's limits check (`limits.rs`, beside `loads.loads < 2`)
      refuses `loads.rows` below 2, and the small profile
      (`crates/temper-service/src/limits.rs`, `root.loads.rows`) and the
      walking world (`tests/engine/domain/src/walking.rs`) ask 2;
    - `tests/client/src/fixtures.rs:238`'s `reads::Row` gains
      `ended_read: None` (ruling 172);
    - tests: `tests/engine/domain/tests/loads.rs`,
      `a_census_page_takes_an_ended_read_position_then_its_task` (the
      real store seeded with an `EndedRead` row, its ended task and a
      `Result` row, paged two rows at a time: the position and its task,
      the cursor the `EndedRead` key; then the result's task, the cursor
      its `Result` key), and `a_census_cursor_behind_its_start_is_out_of_order`
      with an `EndedRead` cursor before a `Result` one; and, beside the
      root world's restart stories (`tests/engine/domain/tests/root_routes.rs`),
      `an_ended_chats_read_position_survives_a_restart` (a chat of two
      turns, the first marked, ends; after the restart its line reads
      one unread turn).
  - **The terminal's room** (the design answer of 2026-10-16): skein's
    `Terminal` gives the application's lines a room of their own (skein's
    `ui-text.md`, 3 and 4: the settings' lines, `Terminal::room`), which
    the supplement step skein-01.s1 builds; and each of those lines
    bounded in bytes, with the terminal's worst case and operations,
    which its host adds to its own (the design answer to temper 03.1's
    question of 2026-10-16T14:54:59Z: `ui-text.md`, 3 and 4, the line bytes,
    `Terminal::worst_case`, `Terminal::operations`), which skein-01.s2
    builds, after forgejo-17's skein step. temper 03.1 needs both: its
    host prices skein's terminal by them, and copies none of its count.
    temper takes them alone, ahead of jig: 03.1's first commit moves
    temper's skein to skein-01.s2's landing (03.1's Shape), and jig's pin
    of skein stays until jig 12.2's move above. From 03.1 on, while `git -C
    ~/src/rust/skein merge-base --is-ancestor <skein-01.s2's landing>
    <jig's pin of skein>` fails, each temper repin takes skein-01.s2's
    landing in place of jig's pin of skein, and its check reads skein's
    line as that landing; once it succeeds (jig 12.2's move), jig's pin
    again. So jig's move takes a landing that holds skein-01.s2's
    (skein-01.s3's), not skein-01.s1's alone: the closing repin, which moves temper's skein to jig's pin, would
    otherwise take temper back before skein-01.s2. At skein `7e8fb700`, skein's code differs from jig's pin
    `f12f29d4` only by skein 01's harness (`git -C ~/src/rust/skein diff
    --stat f12f29d4 7e8fb700 -- crates` prints nothing), which no jig,
    smith or temper crate names; skein-01.s1's and skein-01.s2's one
    downstream `Settings` literal is jig's test world's, outside temper's
    build (`git grep -n 'skein_ui_text::\(Settings\|Terminal\)' --
    '*.rs'` prints nothing in temper at `e73d3d5f`). A build
    failure in a jig or smith crate after temper's move is a question.
  - **The print's room** (the design answer to forgejo-17's question of
    2026-10-16T14:19:38Z): skein's text shell takes each print's room
    from its own tree, and reserves no worst case (skein's `ui-text.md`,
    3 and 4, at skein `054c06e1`), which forgejo-17's skein step builds,
    after skein-01.s1. Before it, `Print::new` refuses temper's view
    limits, whose worst-case print is 257,411,441,911 bytes, past its
    u32, so every `temper open` panics
    (`crates/temper-client-shell-terminal/src/terminal.rs:72`), and
    03.1's `Terminal::new` would too; under ruling 111 no process Proof
    of this plan meets it. temper's skein floor is that step's landing,
    `git -C ~/src/rust/skein log origin/main --all-match
    --grep='^Plan: forgejo-17$' --grep='^Step: skein-' --format=%H -1`,
    which holds skein-01.s1's. Each skein move of this plan in temper
    (03.1's, each temper repin's, the closing repin's) goes to the later
    of its target and the floor, the one that holds the other (`git -C
    ~/src/rust/skein merge-base --is-ancestor <one> <other>`), and is
    not made when temper's pin already holds that commit: forgejo-17's
    temper step may move it there first. skein-01.s2 needs that step, so
    its landing holds the floor: where this plan reads skein-01.s2's
    landing as temper's skein (this section, the session files), the
    later of the two is skein-01.s2's; jig's own pin of skein is
    unchanged until jig 12.2's move. The
    step keeps skein-ui-text's API (`Print`'s text a `List<u8>`,
    `worst_case`'s `(u32, u64)`, both constructors' signatures), and
    `temper-client-shell-terminal` is the one temper crate that names
    `skein-ui-text`, so a move onto it is `Cargo.lock` alone.
  - **temper's skein ahead** (ruling 154, the request of
    2026-10-16T17:16Z): temper's skein may be ahead of this plan's
    target for it (the floor above, skein-01.s2's landing, or jig's
    pin): another temper plan moves it forward, forge-setup's first
    (skein lib's `sha256::Hmac` and `sha256::equal`, additive). No skein
    move of this plan takes temper's skein back: each goes to the later
    of its target and temper's present skein (`git -C ~/src/rust/skein
    merge-base --is-ancestor <target> <temper's skein>` succeeding means
    no move), and each check of skein's line reads that command, not an
    equality. Cargo takes every skein crate from one git source, locked
    in temper's `Cargo.lock`, so jig's and smith's crates build on
    temper's skein. A build failure in a jig or smith crate on it is a
    question.
  - **A path's segments** (the design answer of 2026-10-16T15:20Z,
    ruling 143): jig-13.s1 adds `segments` to jig's client limits, beside
    `rows` and `text`. The first temper increment whose repin includes
    jig-13.s1's landing (`git -C ~/src/rust/jig merge-base --is-ancestor
    <jig-13.s1's landing> <the pinned commit>` succeeds), at the latest
    temper 04.4's, adds `segments: 9` (`client.md`, 5.4) to each temper
    literal of those limits, counted within the repin's 200 lines: at
    temper `df37f032`, `git grep -n 'conversation_words:' -- '*.rs'`
    prints 4 (`crates/temper-client-domain/src/tests/fixtures.rs:37`,
    `crates/temper-client-shell-terminal/src/limits.rs:52`,
    `crates/temper-client-view/src/tests/fixtures.rs:34`,
    `tests/client/src/fixtures.rs:48`); a further literal gets the same,
    listed in the commit body. temper 04.4 then holds the engine's name
    bounds at most it.
  - **The closing repin, `temper-repin`:** temper's README-declared
    repin. The coordinator adds it at setup, `plan set temper-repin
    repo=temper state=waiting` (development.md 7.1 adds none, since the
    plan lives in temper), and starts it once every other step of the
    plan has landed. Its step 1 moves jig to `origin/main`, smith to
    jig's pin of it, and skein to jig's pin of it only where that is
    forward ("temper's skein ahead", below) (each read as above), in
    place of 5.7's "each to its `origin/main`".
    Step 1 also moves temper's server count (`crates/temper-service/src/limits.rs:314`,
    `connections: 4`, "jig's shipped small profile") to jig's `Profile::Small`, 8
    (ruling 362), and refreshes each value derived from it, a forge-setup
    file in a commit of its own (rulings 354, 356).
    Step 1 also spells a change's delegates' commits as text (temper's
    `domain/root.md`, 6; the designer's answer of 2026-10-18T22:06Z), in
    a commit of its own before the gate runs: `change_delegate`'s
    Resolve (name 6) and Gate (name 8) parameters
    (`crates/temper-engine-domain/src/route/forge_route.rs:1712`,
    `:1725`) take the commit's lowercase hexadecimal digits, 40 when its
    last 12 bytes are zero (a SHA-1 id's padding) and 64 otherwise.
  - **skein's cap** (ruling 296, the request of
    2026-10-17T23:46Z): no skein move of this plan, the closing repin's
    included, takes jig's or temper's skein past skein `2733882` ("Mark
    hearth's render active on debian-8gb-hel1-1"). hearth's `skein_io`
    changes (`Locked`, `Busy`, `Lock`, `ListPage`) start at its child
    `089ed83`, and jig and temper take them in temper's next plan,
    `tools` (`later.md`). "temper's skein ahead" holds: nothing moves
    back. A target past the cap (`git -C ~/src/rust/skein merge-base
    --is-ancestor <target> 2733882` failing) is a question.
- **Format versions** (development.md, 5.1): each changes once in this
  plan, at its owner's landing, and is edited in place after:
  - jig's store: tasks v3, owner jig 02.1; people v3, owner jig 02.3;
    core v3, owner jig 06.1. An increment that changes one of these
    families' records needs its owner's landing, and its Needs say so.
    The owner moves the family's schema and goldens to the new version
    (`schema/<family>-v3.schema`, `golden/<family>/v3/`), and the old
    version's go: nothing reads them.
  - jig's client wire: v8, owner jig 11's first increment that changes a
    record (its file names it); jig 10 changes no record.
  - temper's store (`STORE_VERSION`): it moves once, in the first temper
    increment whose repin includes jig 02.3's landing (people v3, whose
    refusal bytes temper's store keeps); that increment's report says so.
    temper's client store and wire: the temper session that first changes
    one names it and bumps it; no other temper session changes it before.
- **Generated code** (the steer of 2026-10-16T12:47Z, jig 07's crate
  under logs/ after jig 06's): a schema's code and goldens are written
  only by its drift test's write mode, in the build lane. No session
  builds a crate of its own to generate them, under logs/ or elsewhere:
  such a crate compiles `skein-codegen` from skein's `main`, not jig's
  pin.
  - A store family, or the test connector
    (`testing/jig-test-store/schema/test-connector-v1.schema`): `sh
    -c 'SKEIN_CODEGEN_WRITE=1 cargo nextest run -p jig-store-codecs-world
    --test drift -E "test(<its drift test>)"'`, the drift tests being
    `the_<family>_familys_code_and_goldens_have_no_drift` and
    `the_test_connectors_code_and_goldens_have_no_drift`
    (`tests/store-codecs/tests/drift.rs`).
  - jig's client wire, and the test client's:
    `sh -c 'SKEIN_CODEGEN_WRITE=1 cargo nextest run -p
    jig-client-wire-world --test drift --test test_client_wire'`.
- **The meeting point:** a client increment proves against the real
  engine, in `tests/client-protocol`, the joint world `tests/client`, or
  the process tier, from jig 12 on; never only against
  `jig-client-scripted-engine`. jig 12's thin story (the printed link,
  `me` live, `Stop` answered once durable, kept across a reload) grows
  with each client increment after it.
- **A joint or process proof that fails** (ruling 229, after rulings 226
  to 228 each counted a proof's runs): reproduce each new failure point
  in a focused test first, log its cause there, and
  change only what that log names; then run the proof once. A run that
  fails at a later story point than the run before it is progress: go on
  the same way. A run that fails at the same point as the run before it,
  a cause in production code outside the increment's Files, or a sixth
  run of one proof after this rule reaches it, is a block with the logs.
  A referee's or a judge's check is never changed to pass.
- **Logs cited as `logs/wN/…`** in the session files are the planner's
  evidence, under `~/src/rust/tmp/plan/client-api/logs/`, outside the
  plan; nothing asks to read them.
- **Edits outside an increment's Files** that are no question (temper's
  protocol plan learned each; they replace the implementation rules's "it is
  in your session's own files" for these cases only):
  - an arm each exhaustive match needs for a variant the increment adds,
    in any crate, at most 5 lines per match;
  - the users of a type the Shape changes, edited to its new form with no
    other change, every assertion, input, expected value and name kept,
    in any number of lines (code-organization.md, 6);
  - a fix the change, its Proof or the gate needs, at most 20 lines, in
    any session's files, every test's name and expected value kept;
  - an expected literal that prints a vocabulary the Shape renames, at
    most 10 lines;
  - a removed mechanism's tests, moved to its replacement or removed;
  - code that moved since the plan was written, followed to its file.

  These limits are guides: each landing's report lists its lines outside
  its Files (`git diff --numstat origin/main...HEAD -- <those paths>`),
  and a limit passed is a question before landing.
- **The browser suite is a suite of its own** in jig, from jig 19:
  `--profile browser`, outside the focused and fuzzy suites, Chromium
  only there; its budget is 60 seconds (`global-timeout`).
- **A browser story's Proof** is the gate: the author runs no
  gate of its own for it, and runs a failing story alone with the
  commands of "One test of the browser suite", below.
- **A Proof's "Bound: N s"** is an aim for the test's time as nextest
  prints it, building aside; a bound missed is an aim missed: the report
  says so, and the landing stands.
- **Named numbers in tests** (ruling 116): a new test names each number
  that identifies a party, a task, a message, an entry or a token, when
  an assertion names it, as a `const` at the test file's top or in the
  crate's fixtures module (as `tests/core/src/fixtures.rs:2-3` does);
  counts and expected values stay literals; base tests stay as they are.
  A bit set or mask a new test passes is a `const` built from named bits
  (`const ESCALATION: authority::Proposals = authority::Proposals(16);`,
  then `Proposals(BATCH.0 | ESCALATION.0)`), never a bare number such as
  `Proposals(17)`; `Proposals(0)` stays a literal; production types gain
  no constants for it (the steer of 2026-10-16T15:30Z).
- **A test's helpers once** (the steer of 2026-10-16T15:30Z): before a
  helper goes into a test file, the session searches the crate's
  fixtures module (`src/tests/fixtures.rs`, or a world's
  `src/fixtures.rs`), its `tests.rs` and the world's `src/` for what the
  helper sends or builds, not only for its name (`rg -n 'people::Event::Ask
  \{' tests/core/src` finds `World::decide_proposal`). A helper a second
  test file needs goes into the fixtures module, `pub`, and every new
  user takes it. Base tests keep their own helpers. The first jig
  increment that adds a test under `tests/core/tests/` moves `ask_holder`
  (`tests/core/tests/fields.rs:63-70`, a copy of `ask_as` in
  `refusals.rs:12-19` and `stories.rs:19-26`) into
  `tests/core/src/fixtures.rs` as `pub fn ask_as`, uses it in
  `fields.rs`, and uses `World::decide_proposal` where `fields.rs:178-188`
  ask the same, keeping every assertion, its lines listed outside its
  Files.
- **Test time** (findings/measurements.md):
  - jig: focused median 10.33 s, fuzzy 14.91 s. The run may add, as
    aims, at most 3.0 s to the focused suite (to about 13.3 s of its
    15) and 10 s to the fuzzy suite.
  - jig's focused budget is 30 s for the rest of this run (ruling 275,
    the user's rule of 2026-10-13: "a budget that bites (gate fails on
    its global timeout, every test passing) is doubled temporarily for
    the rest of that plan; the root-cause test-cost pass comes right
    after the plan ends, then the budget is restored"): `.config/nextest.toml`'s
    `[profile.default]` `global-timeout = "30s"`, in a commit of its own.
    Its aim stays 13.3 s, a guide; a jig session no longer starts with a
    repair of the focused suite. The pass after the run brings it to at
    most 12 s (80% of 15 s, the median of three gate-lane runs) and puts
    15 s back (`later.md`). Unchanged: jig's fuzzy budget and its repair
    rule; temper's budgets; a test the gate flags `SLOW` is made cheaper
    before its increment lands.
  - temper: focused median 4.00 s, fuzzy 14.36 s. The run may add at
    most 2.0 s to the focused suite and 5 s to the fuzzy suite.
    From ruling 326 (2026-10-18), temper's aims stay guides, and a temper
    session no longer starts an increment with a repair of a suite that
    missed its aim: each landing reports both suites' times, and a repair
    comes first only when a suite's time passes 80% of its budget (12 s
    of the focused suite's 15 s, 48 s of the fuzzy suite's 60 s). A test
    the gate flags `SLOW` is made cheaper before its increment lands, as
    below. At temper 06.2's second repair, 6.580 s and 26.785 s were 44%
    and 45% of the budgets. Unchanged: temper's budgets, jig's rules, the
    measure commands, and the "Never" below.
  - Each landing's report names its suites' times from its
    `Gate-Result` line, and nothing else is measured. An aim exceeded is
    a guide missed: the landing stands, and the session's next increment
    first makes the slowest tests cheaper, found once by the measure run
    of the suite that missed (`.config/nextest.toml`'s two commands):
    the focused suite's `cargo nextest run --workspace
    --profile measure -j 1`, the fuzzy suite's `cargo
    nextest run --workspace --profile measure -j 1 --ignore-default-filter
    -E 'binary(/^fuzzy_/)'` (ruling 109), until its own gate is back
    under the aim; after a session's last increment the
    supervisor decides. temper's
    `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`
    is no test to make cheaper in this run (ruling 122, `later.md`): the
    slowest tests a repair finds are the others. Never, apart from ruling 275's: raising a `global-timeout`, a slow period
    or a per-test ceiling, or moving a focused test to the fuzzy suite.
  - A test's own time in a session file ("under 20 ms", "under 0.5 s")
    is an aim, development.md 5.6's ceiling: nothing measures one test,
    and a test the gate's log flags `SLOW` is made cheaper before its
    increment lands.
- **Every crate's lints in one round** (the steer of 2026-10-16T16:45Z,
  the third sighting: jig 06 and 07, jig 10, jig 03): `--keep-going`
  builds only crates whose dependencies built, so a crate failing under
  `-D warnings` hides its dependents' lints. After `cargo check
  --workspace --all-targets --keep-going --message-format short` passes,
  the first scoped clippy round caps lints at warnings: `cargo
  clippy --keep-going -p <crate> … --all-targets -- --cap-lints warn`;
  every warning it prints is fixed, then the same command with `-- -D
  warnings` runs once to confirm. the prebuild keeps `-D warnings`.
- **Shared files,** which parallel sessions may both change: jig's
  workspace `Cargo.toml` and `Cargo.lock` (jig 01, 10, 11, 12, 16, 17,
  19); `tests/boundary/src/tests/graph.rs` (jig 01, 10 to 13, 16, 17,
  19); `crates/jig-domain-core/src/{lib.rs, documents.rs,
  routing/mod.rs}` (jig 02 to 09, 14, 16);
  `crates/jig-domain-people/src/boundary.rs` and
  `crates/jig-domain-tasks/src/boundary.rs` (jig 02 to 09, 16);
  `crates/jig-protocol-store/src/{format.rs, keys/, codec/}` (jig 02,
  03, 06, 08, 09, 16); `crates/jig-protocol-config/src/` (jig 03, 04,
  09, 17, 18), and in it `tests/domain.rs`'s
  `memory_below_the_components_sum_is_refused` (`over`, the small
  profile's memory sum) by each session that changes a stored record's or
  a component's worst case (jig 02 to 09, 16; ruling 110); and, by the
  same sessions, the profiles' store bounds (`value`, commit, snapshot) in
  `crates/jig-protocol-config/src/profile/{small,standard}.rs` and their
  expected tuples in `testing/jig-test-service/src/tests.rs`
  (`both_jig_profiles_have_a_bounded_service`): an increment that grows a
  stored record's worst case raises them to what the build requires, and
  sets the tuples to match, its report naming each old and new value
  (ruling 112); `crates/jig-protocol-engine-clients/src/translate/` (jig 14, 16,
  17); `testing/jig-test-domain/` (jig 06, 07, 09);
  `testing/jig-conformance/` (jig 04.7, 06.1, 08.3 to 08.6, 23);
  `crates/jig-client-protocol-link/src/{boundary,component,store,lib}.rs`,
  `crates/jig-client-shell-native/src/` and
  `tests/client-protocol/src/{front,front/persistence,native,fixtures}.rs`
  (jig 10, 11, 12); `tests/boundary/src/tests.rs` (jig 01, 10, 13);
  `crates/jig-client-view/src/` and `examples/ops/client-*` (jig 13, 14,
  15, 16, 21, 22); `crates/jig-protocol-store/src/historical.rs` and the
  roots' `Now` routes (`examples/ops/domain/src/route.rs`,
  `testing/jig-test-service/src/{route.rs, clients/translation.rs}`,
  `tests/client-protocol/src/engine.rs`) (jig 06.5, 07.2); temper's
  `crates/temper-engine-domain/src/store.rs` (temper 02 to 04), temper's
  `Cargo.toml` and `Cargo.lock` (temper 05.1, 06.1, 07.4) and
  `tests/one-process/src/` (temper 05.2, 06.1, 07.4). The session that
  lands second rebases onto the first and follows its change.
- **Files over 800 lines at the base** that an increment changes grow
  by at most 40 lines each, a rule with its count in code-organization.md
  7; their splits are `later.md`'s.
- **A move's Size** (rulings 312 and 314, the same block twice): a step
  whose count passes its stop because it moves code that `-M` does not
  follow lands without a block when the count less the lines the same
  command removes is within the stop. Its report names the files moved
  from and to, and gives the added, the removed and the difference on
  its Size line. A difference past the stop is a block, as before.
- **The run's steers** (ruling 167, the coordinator's block of
  2026-10-16T19:39:21Z): each "Pass on" text of the run's steers that
  holds until the run's end and that no rule above says, as the steer gave
  it, under the steer's time and point and the sessions it applies to.
  They bind as this README's other rules do. The coordinator gives none of
  them again in a prompt: each session reads them here. A later steer's
  text that holds until the run's end lands here, by the supervisor,
  before the steer is sent.
  - 09:29Z, point 1 (every session): After adding a variant or a field,
    find every match and literal of the type with `git grep -n '<Type>' --
    crates testing tests examples`, and edit them all. Then build once
    with `cargo check --workspace --all-targets --keep-going
    --message-format short`, which shows every crate's errors in one
    round. Run scoped clippy only after it passes.
  - 09:29Z, point 2 (jig 02 to 09, and 16):
    `memory_below_the_components_sum_is_refused`
    (`crates/jig-protocol-config/src/tests/domain.rs:166`) pins the small
    profile's memory sum, which moves with every stored record's worst
    case. If your Shape changes a record's or a component's size, run
    `cargo nextest run -p jig-protocol-config -E
    'test(memory_below_the_components_sum_is_refused)'` before land, and
    set `over` to the value it prints. Name the old and new values in your
    report. If another session landed it first, rebase and set it again
    from your own run.
  - 09:29Z, point 3 (every session): Before merging, run once in the build
    lane the focused tests of each crate your diff changes (`git diff
    --name-only origin/main...HEAD`): `cargo nextest run
    --no-fail-fast -p <crate> -p <crate> …`. Land only when it passes.
    Never `--workspace` or the fuzzy profile: those stay in the gate.
    After a gate fails on a test, run that test alone before landing
    again.
  - 10:08Z, point 2 (jig 03 to 09, and 16):
    `both_jig_profiles_have_a_bounded_service`
    (`testing/jig-test-service/src/tests.rs:22-24`) pins the small and
    standard profiles' store bounds, which move with every stored record's
    worst case (ruling 112). If your Shape grows a record's or a
    component's worst case, rebase on `origin/main`, then before land run
    `cargo nextest run -p jig-test-service -E
    'test(both_jig_profiles_have_a_bounded_service)'` with the memory-sum
    test. Raise the bounds (`value`, commit, snapshot) in
    `crates/jig-protocol-config/src/profile/{small,standard}.rs` to what
    it prints, and set both tuples to match. Name each old and new value
    in your report and commit body. A bound that would fall is a question.
  - 12:48Z, point 2 (every session): Add no new
    `#[expect(clippy::too_many_arguments)]`. When clippy finds too many
    arguments on a function you add or widen, pass the arguments that
    travel together as one struct of plain fields, named for what they
    are. Example: `ask(domain, env, number, by, until, message, words,
    reason, out)` becomes `ask(domain, env, Finish { number, by, until,
    message }, words, reason, out)`, with `pub(crate) struct Finish { pub
    number: u64, pub by: Party, pub until: Wall, pub message: u64 }`. A
    function excused at the base keeps its exception. Each report names
    any lint exception its increment adds.
  - 14:09Z, point 1 (every session): The plan forbids `legacy`, `compat`,
    `hack`, `workaround`, `actual` and `genuine` in code, tests included
    (code-organization.md, 6; the README's Pulse). Name a value for what
    it holds, not as the other side of `expected`: `let answer =
    lines(…).map(|_| ()); assert_eq!(answer, expected, …);`, `let mut
    paged = Vec::new();`. Not `observed` or `got`: a synonym is the same
    word. Leave base code and base test names as they are.
  - 16:41Z, point 1 (jig): When your validation has passed on a rebase,
    run `flock
    /srv/data/git/runner/src/tmp/temper/client-api/land-jig.lock land …`
    at once. If main has moved since, rebase, run only `cargo
    nextest run -p jig-protocol-config -p jig-test-service -E
    'test(memory_below_the_components_sum_is_refused) |
    test(both_jig_profiles_have_a_bounded_service)'`, set the values it
    prints, and land: the prebuild and gate check the rest. Run clippy
    and your focused tests again only after a rebase that conflicted in a
    file you changed, or after a gate that failed on something else.
  - 17:52Z, point 2 (every session): In a new test, build a
    `tasks::Amendment` that changes one field from your crate's fixtures
    with struct update, never by writing its unchanged fields as `None`.
    In jig's core world, the first of you to need it adds `pub fn
    unchanged(reason: &[u8]) -> tasks::Amendment` (every change `None`,
    the reason given) to `tests/core/src/fixtures.rs`, and writes
    `tasks::Amendment { spec: Some(…), ..fixtures::unchanged(b"changed")
    }`. Report each new test past 40 lines, with its length. Do not change
    base tests for this.
  - 18:42Z, point 1 (every session): A case table runs one behaviour over
    several inputs. When a case needs its own setup (a `bool` or a tag
    that picks it), each case is its own test with its own name, and the
    shared decision and expected result go in a helper that takes the
    cause (`assert_refused_whole(&mut world, cause)`), not in a loop with
    `if pending { … } else { … }`.
  - 18:42Z, point 2 (every session): Before a new test builds a value,
    search the crate's fixtures for what it builds (README, "A test's
    helpers once"). Three were missing at jig `bb977c909`. A pending
    `tasks::Proposal`: add `pending_proposal(world, number)` to
    `tests/tasks/src/world.rs`, moving 07.3's `proposal` from
    `tests/tasks/tests/procedures.rs`. A words-only `Spec`: add
    `spec(words)` beside `amendment()` in
    `crates/jig-domain-tasks/src/tests/revisions.rs`. A commit's saved
    live row: add `saved_live(commit, task)` to
    `tests/core/src/fixtures.rs`. The first of you whose new test needs
    one adds it, `pub`, and uses it; the rest take it.
  - 19:38Z, point 1 (jig): Before a new test in tests/core defines a
    helper, search the crate for the call it makes (`git grep -n
    'End::Finished'`, `git grep -n 'Outcome::Refused('`). Two exist twice
    now: `finish_chat(world, cumulative, words)` (in
    `tests/core/tests/store.rs`, copied as `finish` in `parties.rs`) and
    `refusal(world, why)` (in `parties.rs`). The first of you who needs
    either moves it, `pub`, to `tests/core/src/fixtures.rs` and uses it
    from there; the rest take it from the fixtures.
  - 19:38Z, point 2 (jig): A commit that must edit a `tasks::Amendment` or
    `people::Amendment` literal in a test (a field added, a type moved)
    rebuilds the literal with struct update from your crate's fixture
    (`..person_unchanged(b"reason")` in jig's core world,
    `..fixtures::unchanged(b"reason")` in `jig-domain-core`'s tests; both
    from jig 04.5), rather than adding the field by hand. Do this only for
    the literals your commit edits anyway; a literal that maps every field
    from another value keeps its fields.
  - 21:01Z, point 1 (jig): A .rs file of at most 800 lines at origin/main
    stays at most 800 after your commit: run `git diff --name-only
    origin/main -- '*.rs' | xargs wc -l` before merging, and put new code
    in a module beside it. Over 800 at jig `795ca6bc3` (the 02:00Z steer,
    point 2): `tests/ops/tests/fuzzy_joint.rs` (805),
    `jig-protocol-engine-clients/src/tests/fixtures.rs` (814),
    `jig-protocol-engine-clients/src/component.rs` (808) and
    `tests/core/tests/parties.rs` (816);
    `jig-domain-people/src/boundary.rs` is back to 796. The next increment
    that edits one brings it to 800 or less. Your report lists
    every file over 800 after your commit, with base and now. Generated
    code is exempt; files over 800 at the base keep their 40 lines.
  - 21:01Z, point 3 (jig): `person_unchanged` is defined in six fixtures
    modules. `tests/conformance` takes
    `jig_core_world::fixtures::person_unchanged` and its copy goes. A crate
    that cannot depend on `jig-core-world` keeps one copy and names why in
    its commit body.
  - 22:44Z, point 2 (jig 12): Before a new test repeats a block, search
    the crate's fixtures for it (README, "A test's helpers once").
    jig-client's `src/tests/fixtures.rs` has no `Client::fire(at)`: add it
    beside `step` (clear the requests, fire the machine, drain its output
    as `step` does) and use it. The sign-in sequence is `Client::sign_in`:
    reuse it. Name each time as a const, never
    `Time::from_nanos(100_000_000)` or `Duration::from_millis(10)` inline.
    When you next edit `tests/store.rs`, move it onto `fire` too.
  - 23:45Z, point 1 (every session that adds production code): A
    function clippy finds too long is split, not excused: extract its
    largest arm into a function named for what it judges (as `check`'s
    `Amend` arm, `crates/jig-domain-core/src/compound.rs:177-204`, would
    become `check_amend`, bringing `check` under 100 lines), and pass the
    arguments that travel together as one struct (12:48Z, point 2). Only
    a codec table over a closed enum may carry
    `#[expect(clippy::too_many_lines)]`, with its reason. A function
    already excused, at the base or since, keeps its exception when a
    Shape widens it (as 04.7 widens `tag_tasks`, ruling 186). Each report
    names any such exception it adds and the function's length.
  - 23:45Z, point 2 (every session that adds or edits tests): Name each
    request id, duration and time in a test as a const beside the file's
    others, by what it stands for: `const CLOSE: u8 = 11;`, `const GRACE:
    Duration = Duration::from_secs(5);`, `const PAST_GRACE: Duration =
    Duration::from_secs(601);`, `const RETRY_AT: Time =
    Time::from_nanos(100_000_000);`. Never `request(&mut world, 11, …)` or
    `Duration::from_secs(601)` inline. A line of a test you edit anyway
    takes its numbers named, as jig-client's `tests/store.rs` :69, :91,
    :111 and :120 do. Seeds passed to `world(…)` and expected values stay
    literals.
  - 00:40Z, point 1 (jig, tests in `tests/core`): Before a new test in
    tests/core pages the inbox read or signs in a second party, search
    the crate for it (README, "A test's helpers once"). The paging loop
    over `reads::Read::Inbox` is written out in
    `tests/core/tests/parties.rs:177` and `:458`, `reads.rs:112` and
    `:184`, `restart.rs:518` and `src/stories/reads.rs:186` and `:308`. A
    second party's sign-in and roles is written out in `parties.rs:160`
    (`other_party`), `fields.rs:40` (`member_world`) and
    `operations.rs:303` (`another_requester`). Add to
    `tests/core/src/fixtures.rs`, `pub`: `inbox_objects(world, sign_in,
    group) -> Vec<InboxObject>` (`parties.rs`'s `inbox`, moved, with
    `proposal_object`) and `second_party(world, subject, role) -> u64`
    (sign in, send `Roles`, return the new sign-in; `other_party` and
    `owner` take it). The first of you whose new test needs one adds it
    and uses it; the rest take it.
  - 00:40Z, point 3 (jig, checks in `tests/conformance` and
    `testing/jig-conformance`): Build a failure's message only when the
    check fails. `check(holds, &format!(...))` formats on every pass, and
    the conformance sweep passes through every cut:
    `tests/conformance/src/scenarios/lifecycle.rs:100`, `:152` and
    `:163`. Write `if !holds { return check(false, &format!(...)); }`,
    and keep a literal message as it is. A trace stays typed and is
    formatted in the failing branch; the first session to edit
    `lifecycle.rs` fixes its three sites.
  - 02:00Z, point 1 (jig): README 23:45Z, point 2 binds every new test
    line: a request id, a duration and a time are consts. jig 03.5 left
    `9100` at `tests/core/tests/parties.rs:795` and `30` at
    `tests/engine-clients/tests/requests.rs:175`. Before merging, run
    `git diff -U0 origin/main -- '*.rs' | grep -nE '^\+ +[0-9]{2,5},$|ask_as\([^)]*, *[0-9]+ *,'`
    and name what it prints for your lines
    (`const READ_PROPOSAL: u64 = 9100;`). The first of you to edit either
    file names its site and reports it as a small fix outside your Files.
  - 02:00Z, point 2 (jig): README 21:01Z, point 1 is a stop, not an aim: a
    .rs file of at most 800 lines at origin/main stays at most 800 after
    your commit
    (`git diff --name-only origin/main -- '*.rs' | xargs wc -l` before
    the merge). jig 03.5 left `component.rs` at 808 and
    `tests/core/tests/parties.rs` at 816; `tests/ops/tests/fuzzy_joint.rs`
    (805) and `crates/jig-protocol-engine-clients/src/tests/fixtures.rs`
    (814) are over too. The next of you to edit one puts new code in a
    module beside it and brings it to 800 or less; a crossed 800 you
    cannot avoid is a question before merging.
  - 02:00Z, point 3 (jig, tests under `tests/engine-clients/tests/`):
    README 'A test's helpers once': before a helper goes into a test file
    under `tests/engine-clients/tests/`, search
    `tests/engine-clients/src/fixtures.rs` and the other test files for
    what it builds. `closing.rs:44` has
    `answer(peer, socket) -> w::Answer` and jig 03.5 added `final_answer`
    in `requests.rs:159` beside an inline copy at :62-66. The first of you
    to add a test there moves `answer` into `fixtures.rs`, `pub`, builds
    `final_answer` on it, and uses both.
  - 04:24Z, point 1 (temper, sessions that add or edit tests): README
    23:45Z, point 2 binds temper's new tests as it binds jig's: a request
    id, a duration and a time are consts, and a stage counter is consts
    or a small enum. temper 03.s2's scenarios wrote `110 + *phase`
    (`tests/conformance/src/scenarios/queries.rs:47`),
    `world.advance(6_000_000_000)` (`queries.rs:75`; `lifecycle.rs:18`
    already names it `ADOPTION_WAIT`), `6_000 + u64::from(KEY)` again
    (`lifecycle.rs:151`, from `scenarios.rs:47`) and a `phase: &mut u8`
    compared with bare 0, 2, 3 and 6. The first of you to edit
    `queries.rs` or `lifecycle.rs` names these sites and reports it as a
    small fix outside your Files. Base lines elsewhere, seeds and
    expected values keep their literals.
  - 05:20Z, point 1 (every session whose commit changes more than 20
    files, and jig 13.4): A failed gate shows only its first one
    to five failures: it stops at the first. jig 13.4 ran 19 gates in 46
    minutes, each fixing the next failure, and two fixes were undone by
    later edits. After a failed gate, run the tests of the crates your
    diff touches, in the build lane, every failure listed: `cargo
    nextest run --no-fail-fast -p <crate> …`, and the same with
    `--profile fuzzy` (the gate's second half). Fix all that fail, then
    the gate again. Use focused tests for diagnosis. the gate is
    unchanged.
  - 06:20Z, point 1 (jig 21.2, 21.3 and 21.4): code-organization.md, 7: a
    test function is at most 40 lines, and a miss is noted in the landing's
    report. jig 13.4 grew seven base tests past 40 lines and its report
    names none. `testing/jig-client-test-domain/src/tests/root.rs` spells
    `Event::Client { event: jig_client::Event::Up { up:
    jig_client::Up::Answer { attempt, envelope: RequestEnvelope::Final,
    answer, why: None } } }` ten times, about 12 lines each, in
    `a_final_answer_reaches_the_child_that_asked_unread_and_once` (30 to 60
    lines) among them. In a test you add or edit, build it once: `fn
    final_answer(attempt: Token, answer: Answer) -> Event` in
    `tests/root/fixtures.rs` beside `drive` (21.2: in the ops tests' support
    module); the 60-line test then takes about 38 lines. In your landing
    report, name each test function over 40 lines that your commit leaves,
    with its length at the base and now.
  - 08:16Z, point 1 (jig 21.3, 21.4, and every jig session that adds tests
    to the default suite): The focused suite has 15 s, and jig 21.2
    landed at 13.744 s (92%; 11.806 s before it). Six of its new tests are
    cause-finding reproductions that stayed in the default suite, each
    building a whole signed-in world;
    `an_accepted_restart_opens_its_remediation_panels` (over 2 s) repeats
    the steps of p at `tests/ops/tests/fuzzy_joint.rs:100-114`. Before
    merging, run `grep 'SLOW \[' <the gate's log>` and name in your report
    each test you added that it lists. A test you wrote to find a cause
    stays in the default suite only if it checks what the Proof does not
    and takes under 0.5 s; otherwise it goes, or into a `fuzzy_` binary,
    and the log keeps the cause. 21.3 deletes the panels test and moves
    the two over 1 s (`an_environment_answer_can_be_lost_at_its_native_byte_face`,
    `an_environment_chat_stays_live_while_its_provision_finishes`), with
    their helpers `ready_environment` and `environment_chat`, to a new
    `tests/ops/tests/fuzzy_bytes.rs` (09:08Z, point 2: `fuzzy_joint.rs`
    is over 800 lines and ends at 800 or fewer), as a small fix outside
    its Files; where p and the eight survival tests already check what
    one of the two checks, it deletes that test instead. It reports the
    gate's focused and fuzzy times.
  - 08:16Z, point 2 (the jig sessions that edit `tests/ops`): README "A
    test's helpers once": before a helper goes into a test file, search
    the crate's fixtures and the other test files. `see(words)` is defined
    in `tests/ops/tests/client_stories.rs:27`, `fuzzy_joint.rs:20` and
    (jig 21.2) `tests/ops/src/joint/bytes/tests.rs:7`, `press` in the same
    three, and the incident search and `remediation` twice
    (`fuzzy_joint.rs:35, :48`; `bytes/tests.rs:41, :102`). The first of
    you to edit one of them moves `see(words)`, `press(role, words)`,
    `incident_lead(engine)` and `remediation(engine, lead)` into
    `tests/ops/src/joint/fixture.rs`, `pub`, and uses them in the three
    files; report it as a small fix outside your Files.
  - 2026-10-17 13:00Z, point 2 (jig 21.4, then each session that adds a
    test to `crates/jig-client-view`): README 'A test's helpers once':
    `crates/jig-client-view/src/tests/domain/native.rs:24-45` builds a
    page or section tree with `Builder::new`, `JigBindings::new`,
    `pages::main` and `finish`, which nine other test sites also write.
    Move that `tree` to `crates/jig-client-view/src/tests/cards.rs`,
    `pub(crate)`, beside `contains` (:157), use it in `native.rs`, and
    use it in every new view test that builds a tree. Add a parameter to
    it if a test needs one; write no new copy. Report the move as a small
    fix outside your Files.
  - 2026-10-17 15:00Z, point 1 (every session): Your landing report names
    only what your commit changed: each test over 40 lines that it adds
    or lengthens (name, base, now), each file over 800 lines that it
    grows (base, now), and the lines outside Files as one count. The
    complete lists go in `logs/<session>/audit-<step>.md`; give its path
    in one line. Do not list a test or a file whose length your commit
    leaves as it was. jig 14.2's report was 10,164 bytes, 7,658 of them
    in lists its audit file repeats; keep yours under about 3,000. This
    replaces 06:20Z point 1's 'each test function over 40 lines that your
    commit leaves' and 21:01Z point 1's 'every file over 800 after your
    commit' where they differ.
  - 2026-10-17 18:11Z, point 1 (every jig session): When your commit
    removes or renames a record, an ask or an answer, rename in it the
    tests whose names say the old word, in about eight words that state
    the behaviour; a rename moves no assertion. List the renames outside
    your Files as one count. Before your landing commit, for each word
    your commit removes or renames, run `git grep -n -E
    'fn [a-z0-9_]*<word>' -- <the crates you change>` and rename each
    test it prints whose name names the old thing (ruling 277).
  - 2026-10-17 20:13Z, point 2 (every session): After your landing
    remove your worktree's `target/` with `cargo clean`, never `rm -rf
    target`: Codex's command review refuses it and drops the rest of its
    cell. Run Python as `python3`, the only name on this machine, never
    `python`.
  - 2026-10-17 22:24Z, point 2 (every jig session; ruling 286): Put new
    client-protocol stories in `tests/client-protocol/tests/client/<topic>.rs`,
    as `client/words.rs` does; add nothing to
    `tests/client-protocol/tests/client.rs` (797 lines at jig
    `738c20adc`). Until `tests/core/src/peers.rs` is at most 800 lines
    (804 at `738c20adc`), the steer's point 2 says how the next session
    that edits it brings it there.
  - 2026-10-17 22:24Z, point 3 (every session that writes a core test;
    ruling 286): In `tests/core`, find a live row with
    `fixtures::live_row(&world, task)`, never by scanning
    `world.store.rows`. An assertion message is one line naming the task
    and what was expected, not a dump of the workers' state. Name a seed
    or a budget that matters (`831`, `10`) as a constant.
    `a_scripted_chat_question_waits_for_its_requester` (49 lines) is the
    example to bring to 40 or fewer.
  - 2026-10-18 11:42Z, point 1 (every session, from its next report):
    In your report's Evidence, give each suite's time from the
    Gate-Result line beside its aim and budget as numbers (jig: focused
    aim 13.3 s, budget 30 s; fuzzy aim 24.91 s, budget 60 s. temper:
    focused aim 6.0 s, budget 15 s; fuzzy aim 19.36 s, budget 60 s), and
    say 'over' or 'below' only as those numbers say.
  - 2026-10-18 11:42Z, point 3 (jig 19.4, 19.5 and every later
    increment's new tests; jig 19.3's two landed tests at the first
    increment that edits `crates/jig-client-shell-web/src/tests.rs`,
    split in place, assertions kept): Tests: one behaviour per test,
    named for it in about eight words; every number in a test is a const
    named for what it stands for; expected values are literals. Split a
    test that asserts several behaviours under one name (development.md
    14.3, Shape).
  - 2026-10-18 20:14Z, point 1 (every temper session, from its next
    increment, until the run's end; ruling 395): One behaviour per test
    (README, 16:47Z point 1). If you change
    `crates/temper-client-view/src/tests/frame.rs`, split
    `signed_out_the_main_says_so_and_nothing_else` and
    `an_engine_not_serving_jigs_version_says_temper_is_out_of_date` in
    place into one test per behaviour (the main's text, the banner's
    status, no Sign out button, no navigation), assertions kept, shared
    setup in `tests/fixtures.rs`, and name them in your report.
  - 2026-10-18 20:14Z, point 2 (every temper session, from its next
    report, until the run's end; ruling 395): Before merging, run
    `git diff --name-only --diff-filter=d origin/main -- '*.rs' | xargs
    -r wc -l`. Your report names, in one line, each file your commit
    leaves over 800 lines, with base and now, even if it was at most 800
    at the base (15:00Z point 1; code-organization.md 7).
    `crates/temper-client-shell-terminal/src/tests/fixtures.rs` is at
    803: put new code for that crate's tests in a module beside it.
  - 2026-10-18 20:14Z, point 3 (the coordinator and every session, from
    its next report, until the run's end; ruling 395): Your landing
    report names only what your commit changed (15:00Z point 1) and
    stays under about 3,000 bytes. Give a by-file Size count once, in
    the commit body or a file under `logs/<session>/`, and cite where in
    the Size line; do not list it again in the report. Keep Evidence to
    one line.
  - 2026-10-18 18:44Z, point 1 (the coordinator and every session,
    until the run's end; ruling 387): File one landing report per
    landing. To correct it, say so in one line of your next report; do
    not repeat the Gate and Proof lines.
  - 2026-10-18 18:44Z, point 2 (every temper session, until the run's
    end; ruling 387): One behaviour per test (README, 16:47Z point 1).
    Keep a setup that two tests share in `tests/fixtures.rs`. If you
    change `crates/temper-client-view/src/tests/chat.rs`, split its four
    two-behaviour tests in place, assertions kept, and name them in your
    report.
  - 2026-10-18 18:44Z, point 3 (every temper session, until the run's
    end; ruling 387): Before you define a test helper, search the
    crate's tests with `rg -n 'fn <name>'` and for what it builds
    (README, "A test's helpers once"). Use the shared one, or move it
    into `tests/fixtures.rs` and use it from there; add no further copy.
  - 2026-10-18 16:47Z, point 1 (every temper and jig session, in the
    tests it has not yet written, until the run's end; rulings 378 and
    379): One behaviour per test, named in about eight words, every
    number a named const (README, 11:42Z point 3). Where the plan names
    one test for two behaviours, write two that share a helper, and name
    both in your report.
  - 2026-10-18 16:47Z, point 2 (temper 06.6, temper 07.1 and every later
    temper increment, until the run's end; rulings 378 and 379):
    an_adopted_repository_gets_its_change_pushed_through_git is repaired
    already (0.947 s alone, ruling 368, point 4). If your gate flags it
    SLOW under load, do not repair it again: name it in your report with
    the gate's time and its time alone. Repair every other test your gate
    flags SLOW (README, 13:34Z point 2).
  - 2026-10-18 14:05Z, point 1 (every temper increment that lands from
    now, until the run's end; ruling 369, the request; it
    replaces the 12:20Z point 2, ruling 347): The whole process tier:
    before landing, run it once, with both exports
    (temper/05-s1-landing-story.md's Proof): `sh -c 'export
    NEXTEST_DOUBLE_SPAWN=0 && cargo build -p temper --bin temper && cargo
    nextest run -p temper --profile process --no-fail-fast'`. A failing
    test is the increment's, under ruling 229, except, until temper-05.s2
    lands, the five that fail on `main` too (ruling 371 adds the fifth):
    `temper_serve_answers_a_signed_delivery_204`,
    `temper_serve_answers_an_unknown_forge_404`,
    `temper_serve_answers_an_unsigned_delivery_403`,
    `temper_serve_prints_its_link_and_drains_on_a_signal` and
    `a_running_engine_lends_the_record_temper_sign_in_kept`. Name each in
    your report with its first failure line.
  - 2026-10-18 13:34Z, point 3 (every temper session, until the run's
    end; ruling 363): The owned example: `docs/examples/owned.json` is
    exact (forge-setup `temper-14.1`, `configuration.md` 7). If your
    change can raise a worst case, run `cargo nextest run -p temper
    --test startup -E 'test(serve_prepares_once_on_the_owned_example)'`
    before your gate. When a start refuses a memory, raise the named value
    (`engine.memory_bytes`, `workers[0].memory_bytes`) by the shortfall it
    names, in a commit of its own that names `temper-14.1` and rulings
    354 and 356, and list any further copy of the value. That is no block
    and no failure outside your Files. A refusal of another kind, or a
    shortfall one repair does not close, is a block. Ruling 377's hold
    ended when forge-setup's `temper-14.s2` landed (temper `ab4c62884`,
    2026-10-18T16:57Z), whose test lays the longest owned paths (ruling
    398): name `temper-14.s2` in that commit too.
  - 2026-10-18 13:34Z, point 1 (jig-19.5, jig-20.1 and every later jig
    increment, until the run's end; ruling 364): Test time, jig: the
    fuzzy aim of 24.91 s is a guide, as the focused aim is (ruling 275).
    Make no test-cost repair for an aim, and report each suite's
    Gate-Result time. Repair first only a test the gate flags `SLOW`, or
    when the fuzzy suite passes 48 s. Leave out of your commit a speed
    change that no gate time shows. Except a test that runs under its
    period alone (`a_resumed_run_is_sent_only_its_current_window`, 0.257
    s under the 500 ms period, ruling 384; the 26 of jig-22.s1's gate,
    ruling 400): if your gate flags it SLOW under load, do not repair
    it: name it in your report with the gate's time and its time alone,
    from one serialized run of the flagged tests.
  - 2026-10-18 13:34Z, point 2 (temper-06.5, temper-07.1 and every later
    temper increment, until a landing gate shows no focused `SLOW`;
    ruling 364): A test the gate flags `SLOW` is made cheaper before your
    increment lands (README, Test time), whether or not your change made
    it slow. Time the test alone, cut the cost where the time goes, and
    give the before and after times in the report. Name each SLOW test of
    your gate, except ruling 122's fuzzy walking sweep, and except the
    test the 16:47Z point 2 names.
  - 2026-10-18 13:34Z, point 4 (temper-06.6 and temper-07.1, until each
    lands; ruling 364): Size, temper 06.6 and 07.1: four steps of this
    part of the plan landed at 2.0 to 2.5 times their Size by ruling (330,
    342, 345, 359). Yours lands whole at up to 550 (06.6) or 650 (07.1)
    production lines with no question. Give the count by file in the
    commit and the report, name files outside Files and each lint
    exception with its reason. Past the stop, block with the count.

## Prerequisites the run checks at its start

**The run's machine is carbon-forge** (decisions.md, 13): it has the
`wasm32-unknown-unknown` target, which skein's gate and jig 19 onward
need. x1, where the plan was written, has none, so the planner could not
run these at the base; the supervisor's ready check runs them on
carbon-forge when their session becomes ready (development.md, 5.4,
last paragraph): skein 01's and jig 19's and 20's "One test" commands
below, jig 19.4's wasm32 builds and clippy, and the browser suite's
`cargo nextest list` with `--profile browser`.

The coordinator runs each check once, before the first session, and
writes `blocked.md` (kind: process) on a failure:

1. `cargo nextest --version` prints a version.
2. `ls "$(rustc --print sysroot)/lib/rustlib"` lists
   `wasm32-unknown-unknown` (skein's gate; jig 19's wasm32 builds).
3. `/usr/bin/chromium --version` prints `Chromium 1`, or
   `SKEIN_TEST_CHROMIUM` names a file (jig's and skein's browser suites).

## One test of the browser suite

A browser story that fails is run alone, in the session's worktree,
through the build lane: after each fix aimed at a cause its log names,
the two commands below run once each; what counts is the second's
`Summary` line, 1 test run, 1 passed. The same symptom a third time is a
question (development.md, 5.5). Nothing else is run or kept. `target/`
is the worktree's own. The session files point here; this is the one
copy of the commands.

**skein's browser suite** (skein 01), as skein's `scripts/check.sh`, its
last block, builds it:

```sh
sh -c 'cargo build --target wasm32-unknown-unknown -p skein-ui-example-shell-web && cargo run -p skein-ui-bundle -- --module target/wasm32-unknown-unknown/debug/skein_ui_example_shell_web.wasm --name skein_ui_example_shell_web --prefix /bundle/ --stylesheet examples/ui/shell-web/notes.css --out target/ui-example-bundle --noscript "Notes needs WebAssembly and scripts." && cargo build -p skein-ui-example-server'
sh -c 'SKEIN_UI_EXAMPLE_BUNDLE="$(pwd)/target/ui-example-bundle" cargo nextest run -p skein-ui-web-tests --profile browser -E "test(<name>)"'
```

**jig's browser suite** (from jig 19.4 on; the bundler's arguments are
those jig 19.4 writes into jig's `scripts/check.sh`; where the two
differ once landed, `scripts/check.sh` wins):

```sh
sh -c 'cargo build --target wasm32-unknown-unknown -p jig-client-test-shell-web && cargo build -p skein-ui-bundle && target/debug/skein-ui-bundle --module target/wasm32-unknown-unknown/debug/jig_client_test_shell_web.wasm --name jig_client_test_shell_web --prefix /bundle/ --stylesheet crates/jig-client-shell-web/jig.css --stylesheet testing/jig-client-test-shell-web/jig_client_test_shell_web.css --pictures jig=crates/jig-client-shell-web/pictures --pictures jig-test=testing/jig-client-test-shell-web/pictures --files testing/jig-client-test-shell-web/files --noscript "jig needs WebAssembly and scripts." --out target/jig-test-bundle && cargo build -p jig-test'
sh -c 'JIG_TEST_BUNDLE="$(pwd)/target/jig-test-bundle" cargo nextest run -p jig-client-browser-world --profile browser -E "test(<name>)"'
```

`cargo run -p skein-ui-bundle` does not resolve in jig's workspace; the
bundle's path is absolute, since nextest runs each test from its
package's directory.

## Live services

None. The browser tier runs a local Chromium against jig's test
application on loopback; no agent signs in anywhere.

## Repin

temper is repinned by its sessions as they follow jig, and by the
closing `temper-repin` (Rules, Pins). smith is read only, and nothing
downstream of temper exists.

## Pulse

```
words: legacy|compat|hack|workaround|actual|genuine
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, reading `origin/main`.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. jig's old client crates are gone: `git -C ~/src/rust/jig ls-tree -d
   --name-only origin/main crates/jig-protocol-client-wire
   crates/jig-protocol-client-link crates/jig-shell-native crates/jig-view
   crates/jig-domain-client testing/jig-scripted-engine
   testing/jig-test-domain-client testing/jig-test-view
   testing/jig-test-client-kit testing/jig-test-client-records
   testing/jig-test-protocol-client-wire examples/ops/domain-client
   examples/ops/view` prints nothing.
3. jig's new client crates exist: `git -C ~/src/rust/jig ls-tree -d
   --name-only origin/main crates/jig-client crates/jig-client-domain
   crates/jig-client-protocol-http crates/jig-client-view
   crates/jig-client-shell-web crates/jig-protocol-engine-clients-http
   testing/jig-client-test-shell-web
   testing/jig-client-fake-person-browser tests/client-browser
   examples/ops/client-protocol-wire | wc -l` prints 10.
4. The naming checks hold and wait for nothing: `git -C ~/src/rust/jig
   grep -n 'PENDING' origin/main -- tests/boundary/src` prints a line
   with `[]`, and in a worktree of each at `origin/main`, `cargo
   nextest run -p jig-boundary -E 'test(client_crates_carry_the_client_prefix)'`
   (jig) and `cargo nextest run -p temper-client-world -E
   'test(client_crates_carry_the_client_prefix)'` (temper) each pass one
   test.
5. No shape of work on the wire: `git -C ~/src/rust/jig grep -n -E
   '^(record|enum) (StartChat|SetGoal|Prioritise|Conversations|ConversationLine|GoalLine|Goals|Place|Draft)\b'
   origin/main -- crates/jig-client-protocol-wire/schema` prints nothing.
6. jig's component owns no listener: `git -C ~/src/rust/jig grep -n -w
   -e Listen -e skein_http origin/main --
   crates/jig-protocol-engine-clients/src` prints nothing.
7. jig's gate has the browser suite and the wasm32 builds: `git -C
   ~/src/rust/jig grep -n -e wasm32-unknown-unknown -e 'profile browser'
   origin/main -- scripts/check.sh` prints a line for each.
8. temper's client is its own: `git -C ~/src/rust/temper grep -n -e
   jig-client-view -e jig-domain-client -e jig-view origin/main --
   '*Cargo.toml'` prints nothing.
9. temper pins jig's last landing of this plan: `git -C ~/src/rust/temper
   show origin/main:Cargo.lock | grep -o 'jig.git#[0-9a-f]*' | sort -u`
   prints one commit, which `git -C ~/src/rust/jig log origin/main
   --grep='^Plan: client-api$' --format=%H -1` prints, or a later one.
10. The plan's commits: `git -C ~/src/rust/<repository> log origin/main
    --grep='^Plan: client-api$' --format='%(trailers:key=Step,valueonly,separator=)' | sort -u`
    lists each planned step: in jig the 107 steps `jig-01.1` to
    `jig-23.2`, in skein `skein-01.1` to `skein-01.3`, in temper the 26
    steps `temper-01.1` to `temper-07.4` and `temper-repin`. A supplementary
    step (`.s<k>`) adds a line; a step dropped by a ruling is named in
    decisions.md's rulings instead.
11. The test-time aims, measured once in a worktree of jig and of temper
    at `origin/main`. jig: `sh -c 'export
    NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
    --workspace 2>&1 | grep Summary; cargo nextest run --workspace
    --profile fuzzy 2>&1 | grep Summary; done'`; temper: the same
    without the `export`. The medians of the three `Summary` times per
    suite, against the aims above. An aim missed is recorded with the three slowest tests by name
    in decisions.md's rulings, for the user; the run is done either way.
12. No budget was raised: `git -C ~/src/rust/jig diff a3b53546 origin/main
    -- .config/nextest.toml | grep -E '^[-+].*(global-timeout|period)'`
    prints only the browser profile's added lines and ruling 275's
    focused `global-timeout` (`-` 15s, `+` 30s, until the pass after the
    run puts 15 s back), and `git -C
    ~/src/rust/temper diff c7095516 origin/main -- .config/nextest.toml`
    prints only ruling 326's scheduling override, which raises no budget
    (`[[profile.fuzzy.overrides]]`, its `filter` and `priority = 20`;
    ruling 401).
13. Each store family moved once: `git -C ~/src/rust/jig show
    origin/main:crates/jig-protocol-store/src/format.rs | grep -o -E
    'core: 3|tasks: 3|people: 3' | wc -l` prints 3 (at the base, the same
    command with `2` prints 3).
14. One definition of the client wire's version:
    `git -C ~/src/rust/jig grep -n -E '(CLIENT_VERSION|VERSION): u32 = [0-9]'
    origin/main -- crates` prints one line, in
    `crates/jig-client-protocol-wire/src/lib.rs` (code-organization.md,
    4; at the base it prints two).
15. The client's transitional pieces are gone (code-organization.md,
    6): these two commands print nothing.

    ```sh
    git -C ~/src/rust/jig ls-tree --name-only origin/main -- crates/jig-domain-client crates/jig-client-view/src/bridge.rs crates/jig-client-view/src/watches.rs crates/jig-client-view/src/reads.rs crates/jig-client-view/src/link.rs crates/jig-client-view/src/versions.rs
    git -C ~/src/rust/jig grep -n -E -e 'FrameIs|ByValue|RequestTable' -e 'Kept \{ scope' -e 'fn watch_event' -e 'Event::Frame' origin/main -- crates/jig-client-view/src testing/jig-client-test-domain/src testing/jig-client-test-view/src examples/ops/client-domain/src examples/ops/client-view/src
    ```
16. The aims stated above are these items; none other is checked.
