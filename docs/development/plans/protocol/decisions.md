# protocol: decisions

The decisions this plan rests on, each with its reason; the supervisor's
rulings during the run follow them, numbered from 101; the last section
lists what the user may veto. The planner took these on 2026-10-14 under
the user's delegation (development.md, 2.2 and 10.4).

## The plan's decisions

1. **One plan, temper only, the design's thirteen steps.** It builds
   protocol.md, section 12, steps 1 to 13, and the domain changes of its
   section 5 ("Kept"), and moves the engine worlds onto jig's real store
   (store/README.md, section 6; domain/testing.md, 4.5). It changes no other
   repository: the design needs nothing upstream (protocol.md, section
   11), and jig's test-budgets plan moves jig's, smith's and skein's
   mains on another machine. temper keeps its pins, jig `d91a89c4`,
   smith `dfc0fe57`, skein `b5d3a2c3`; a step that would need more is a
   question, never a repin.
2. **The plan is named `protocol`,** the name `STATUS.md` already gave
   it, in `docs/development/plans/protocol/`.
3. **The audit's design gaps were closed first,** in a Markdown-only
   landing before the plan (`Design: close the gaps the protocol plan's
   code audit found`): a failed preparation reported as a fact of the
   attempt, since jig's core keeps no attempt's failure detail; only the
   identity's token is secret in its `Debug`; the adoption's outcome kept
   durably in a sixteenth family, `Application`, at byte 15, as jig asks
   of a connector's keyed requests; jig-view's `Cell` role and a status
   with no text in the terminal's table; the fake git program hosted by
   skein's world harness; the sweep's `trash/<n>` counted from a number
   drawn at each start, its failed pass resumed at the next Make or
   start, and `sweep_bytes` at least the 255 bytes io's listing needs.
   `findings/README.md` maps every finding. The session writers met
   smaller gaps where the design is silent on a mechanism; each is a
   question to raise with its default in its session file (for example
   06's cap on a list inside one answer, 13's missing connect and
   handshake deadlines, 08's `PartyInput::SignOut`).
4. **Eighteen sessions, 81 increments; at most three sessions at once**
   on `x1` (8 cores, 15 GB, about 8 GB free; pueue's gate lane 1, build
   lane 2). The README orders them for short slots. 81 steps is above
   the 60 that jig-protocol's retrospective (proposal 3) suggests for a
   plan; the dependency structure lets the supervisor stop at a session's
   end, and `later.md` holds nothing that this plan's steps need. The
   session writers split increments past about 600 lines (04.4, 07.3,
   08.2, 08.5, 09.1, 10.1, 15.1, into `a` and `b`, or four for 08.2) and added
   16.6 (the terminal's adoption, after the landed change). Session 08
   keeps its nine increments in one session: the second half needs the
   first, so two sessions would run them in the same order, and the
   handover by context (README) bounds each thread.
5. **Adding a crate of jig, smith or skein keeps the pins** by three
   `cargo update --precise` commands, jig first, then smith, then skein
   (code-organization.md, section 3). Cargo otherwise moves a git
   source to its newest commit when a package it does not hold yet is
   added; the planner saw it move all three, and the three commands in
   that order put them back.
6. **Crate kinds.** Every crate is step code but the binary `temper`
   (ordinary Rust, after jig's `jig-test`), `temper-shell-terminal`
   (ordinary Rust, after `jig-shell-native`; its `route`, `render`,
   `command` and `limits` modules are step code held to the subset by
   code-organization.md, section 6's check, since one crate has one
   lints table), the test support crates, the tools and the worlds.
7. **The git component's file operations are kernel operations.**
   skein-io's file layer renames within one directory only
   (`file_layer.rs:576-583` at the pin), while git.md, 4.1 moves a
   workspace into `trash/`. The component issues `skein_io::kernel::Op`
   `Rename`, `Remove`, `MakeDirectory`, `Open` and `List` itself, through
   the worker's submissions, as jig's reference worker does for its
   root's close. It spawns git through io's records.
8. **temper's application parts plug in where jig's reference leaves
   them empty:** the four `parts` sites of jig's reference service, the
   store's `Put`, `Remove` and application interval, the clients'
   application family, the application credentials' table
   (`findings/jig-service.md`, section 1). temper writes the application
   member's parser with `jig_protocol_config::read`, and its own
   formatter of problems, one line each with its path.
9. **`temper-service` takes typed settings.** `startup` (10.1) parses
   configuration's bytes into `temper_service::settings`, and the engine
   (08) and the worker (09) are built from those values, so their tests
   need no document. This is the interface between parsing and
   composition, not a transitional constructor.
10. **Transcripts are raw bytes, one directory per case:** Forgejo's as
    `request` and `response` (HTTP/1.1 as sent and received, masked),
    git's as `argv`, `exit`, `stdout` and `stderr`. Consumers read them
    with no parser beyond their own. The tools that record them are
    ordinary Rust and may use `serde_json` to mask JSON bodies; no step
    crate links it.
11. **The live tier's login is borrowed by a Rust tool,**
    `tools/live-login`, which `scripts/proof-live.sh` runs: jig's script
    read it with Python, which this plan does not write. Its budget is
    the design's per run (25 completions, 15 minutes) and the README's
    for the plan (3 runs, 50 completions); it runs only after the same
    stories passed against the fake peer.
12. **The throwaway Forgejo is v16.0.5,** downloaded by the planner from
    codeberg.org's release page and verified against its published
    SHA-256 (`2c548bf1bfa09a441a3a674b1dd4959d77576816ad50534c23496908a3ea5a19`),
    kept at `~/src/rust/tmp/temper/forgejo/v16.0.5/`; it started on
    loopback with SQLite in a scratch directory in about two seconds.
    The Actions facts' runner is forgejo-runner v12.13.2, verified the
    same way (`2b4c3751fc2f9a60f753ef97414949de25587dfa2b921870b0cb6cb7c9bf7080`),
    the last release before the runner's v13, whose breaking changes
    postdate Forgejo v16. The user's own Forgejo instance and its 7.0.12
    binary are never touched.
13. **Scratch directories are never under `/tmp`:** tests use
    `CARGO_TARGET_TMPDIR`; the tools take their directory as an argument.
    jig's process tests used the system's temporary directory; temper's
    do not.
14. **temper writes its own world driver for jig's store** (18.1): jig
    ships none that is not tied to its test application. It reuses
    `jig_test_store_files::Files` for the transcripts' file requests the
    component makes in memory mode.
15. **The forge component replaces the routing's unavailable answers in
    one place** (14.1). Until then the engine's routing answers every
    `Request::Forge` with `Error::Unavailable` at cost 0, as protocol.md,
    section 12, step 2 says; 14.1 removes that arm.
16. **The client is copied from ops's,** as client.md, section 3 and
    jig's `client/domain/root.md`, section 12 say: ops's client root and
    view root, its `tests/ops/src/client/app.rs` for `tests/client`'s
    `App`, and its `tests/ops/src/joint/` for the `Front` over temper's
    engine root, on `jig-client-world`'s kit. The forge connector's
    number is given at construction, as client.md, 3.1 says.
17. **The native client is temper's own** (12.1): jig's `NativeClient`
    is test code bound to jig's test application and panics on sign-in
    paths temper needs (`findings/jig-client.md`, Discrepancies 1 to 3).
18. **The new worlds' shares of the suites** are code-organization.md,
    section 5's guides, summing to 3.8 s of the focused suite and 24 s
    of the fuzzy one, within domain/testing.md, 6.1's 4 s and 25 s. The base
    gate ran 682 focused tests in 1.236 s and 31 fuzzy in 7.849 s
    (`./scripts/check.sh` at `cc2697cf`, 2026-10-14).
19. **Sizes:** every increment lands within 1.5 times its Size without
    asking, a codec or document increment within twice it
    (jig-protocol's retrospective, proposal 6, and its rulings 132 to
    214).
20. **The web crates go in one increment with the stories that replace
    them** (11.4), as client.md, section 9 says ("in the same change").
21. **The design's open questions keep their defaults:** repositories
    keyed by Forgejo's id; one period per project at the first start;
    a renamed repository keeps its path and remote; the merge style is
    the repository's default; the side that hands the core a delivery
    encodes its evidence; `Landed` capped by the forge's limits; a run's
    live text waits for smith's channel; nothing of temper's restarts
    `temper serve`; two clients on one directory are not supported; no
    run saves (the root names no saved-work branch, so the routing sends
    neither `save`).
22. **The large files are not split in this plan** (later.md): an
    increment adds at most 40 lines to each, and its new code goes in a
    new module beside it (README, "Large files").
23. **The adoption form's defaults** are client.md, 4.4's: the forge by
    number (1), home checked, prefix `temper/`, CI checked.
24. **The plan's commands, run once at the base** (development.md,
    5.4), on 2026-10-14 at `cc2697cf` with this plan's Markdown on top,
   :
    - the gate, `./scripts/check.sh`: passed, 682 focused
      tests in 1.236 s, 31 fuzzy in 7.849 s, 25 s alone;
    - "Done when" 1: `plan: no active run on this machine`, as expected
      before the run; 2: no commit yet carries `Gate-Result:
      scripts/check.sh passed` (temper's last code landing predates
      the current merge trailer), as expected; 3: 22 paths missing, all
      of them, as expected; 4: 133 lines name the web crates, as
      expected; 5: the two Python files, as expected; 6: holds, the
      three pins; 7: 4 manifests name `jig-fake-store`, as expected; 8:
      `scripts/proof-one-process.sh` does not exist yet, as expected;
      9: no `temper-15.4` commit, as expected; 10: no `facts.txt`, as
      expected; 11: nextest refuses the filter, "no packages matched
      this" (exit 94), as expected; 12: holds at the base (no Rust file
      reads an environment variable or spawns a thread; `Cargo.lock` is
      unchanged);
    - the pins' commands of code-organization.md, section 3, after
      adding 30 crates of jig, smith and skein to a scratch crate: the
      check printed the three pinned revisions;
    - the throwaway Forgejo's smoke start (decision 12); `git --version`
      2.53.0, the version git.md's transcripts name;
    - every interface sketch, compiled under clippy with `-D warnings`
      in a scratch crate of a scratch worktree of temper at `cc2697cf`
      (`~/src/rust/tmp/plan/temper-sketches`, `cargo clippy -p
      plan-sketches --all-targets -- -D warnings`); the sketches of
      ordinary-Rust code are marked "not compiled" in their files.

## Rulings during the run

The supervisor's rulings follow, numbered from 101 (development.md,
10.4).

101. **The store crate links the forge's three children** (temper 04.1,
     blocked 2026-10-14T03:26:27Z; plan defect). `Stored` and `Key` hold
     types of `temper-engine-domain-forge-client` (`api::Repository`,
     `api::Commit`, `Key`, ...), `-change` (`Status`, `GateReport`, ...) and
     `-issues` (`Projected`, `GoalView`), which the forge names through
     private `use ... as client` aliases in `boundary.rs:6-8` and does not
     re-export. `temper-protocol-store` lists all three in its
     `[dependencies]`, with the workspace entries that exist at
     `Cargo.toml:77-79`; it adds no `pub use` to the forge. They are
     domain crates of one connector, not a root, so "no codec crate links
     an engine or worker root" holds, and 04's check
     (`grep -cE '^temper-(engine|worker)-domain[ .=]'`) still prints 0.
     code-organization.md, section 3 and session 04 say so.

102. **05.3 lands as one increment, at most 800 lines** (temper 05.3,
     blocked 2026-10-14T03:57:07Z; plan defect). Its Size is 450, so 675
     with 1.5 times; the author counted about 740 production lines
     already written (`facts.rs` 490, `runner.rs` 222, about 30 more),
     most of it the 63 facts of forge.md, section 13 that the Shape lists
     one by one. A split into 05.3a and 05.3b, proposed as the
     alternative, would add a gate and a landing for code already
     written, and leave `facts.txt` with six Actions facts unchecked
     between them. The bound is code-organization.md, section 8's count
     on the increment's commit, at most 800; past 800 it splits as the
     author proposed. Its Files, Test shape, Proof (one process run,
     63 facts, 83 transcripts) and README's 10 lines are unchanged, and
     06.4's and 14.5's dependencies on 05.3 stay as they are.

103. **Forgejo v16.0.5's four `fails` lines** (temper 05.3, blocked
     2026-10-14T04:06:04Z; design gap). 05.3 lands `facts.txt` as
     observed, its Proof's "the run goes on". Two lines have the
     design's fallback, and need nothing:
     - `fails effective-protection-name-for-a-writer:
       effective_branch_protection_name=""`: forge.md, 3.2's
       `Read::Protection` row reads "`GET R/branch_protections` when it
       names none", answered "a `403` (a writer who is not an admin)
       `Forbidden`".
     - `fails delete-missing-branch-is-404: HTTP 500: ""`: 3.4's "`5xx`"
       is `Timeout`, and the find after it reads the branch
       (`outbox.rs:310-312`), whose `404` is `Missing`.

     Two are the designer's, in the run's `question.md` of this time:
     `fails pull-for-names-the-newest-of-several: 12` (the direct lookup
     names the oldest, closed pull) and `fails open-pull-no-changes-is-422:
     HTTP 201: null` (Forgejo creates a pull with no changes). The
     designer answered at 2026-10-14T04:27:56Z, landed in `ba448afd`
     ("Design: PullFor finds the open pull; OpenPull compares first"):
     `PullFor` answers the open pull request for the head and base, from
     the open listing when the lookup names a closed one, else the one
     the lookup named; `OpenPull` compares the branches' commits first
     and answers `NothingToMerge` with nothing written when
     `total_commits` is 0. forge.md, 3.2, 3.4, 9 and 13, and sessions
     06.3, 06.5, 13.3, 13.4, 14.2, 14.3 and 14.5 say so; 05's
     `facts.txt` keeps its four `fails` lines as the record.

105. **A fix a session's own "Done when" needs is no new step**
     (temper 05.4, blocked 2026-10-14T04:33:15Z; plan defect). Session
     05's check `git grep -nE 'Authorization: (token|Basic)
     [0-9A-Za-z+/=]{20,}' -- tools` matched 05.4's test literal
     `Basic d3JpdGVyOnNlY3JldA==` (`writer:secret`, no issued token).
     The author shortened it to `writer:t` and landed it as
     `2838a7a8`, test-only, under `Step: temper-05.4`. That stands. From
     now: a fix that the session's own "Done when" needs, at most 20
     lines, in that session's files, every test's name and expected
     behaviour kept, lands under its last step's `Step:` trailer, with
     the "Done when" item it serves named in the commit body; the
     coordinator records it in that step's ledger note, and no step is
     added. Anything larger, or outside the session's files, is a
     question.

106. **Sizes: twice, and 2.5 times for a codec or document increment**
     (temper 06.2, blocked 2026-10-14T04:35:36Z; plan defect; the second
     size block after ruling 102). 06.2 (Size 550, allowance 1100) is
     complete at 1287 production lines (`client/items.rs` 440,
     `server/items.rs` 275, `decode/walk.rs` 263, ...), both sides of
     six request families, and lands as it is. The plan's Sizes run low
     for these increments: 04.3's parts summed 2.35 times its Size, 04.4a
     1.99, 06.1 1.98, 06.2 2.34, 05.3 1.85 (a tool). As a standing rule,
     replacing decision 19's 1.5 and twice and the steer of
     2026-10-14T04:21:44Z's point 1: an increment lands within twice its
     Size without asking, a codec or document increment within 2.5 times
     it; a split increment's parts count together; each report names
     "Size: <increment> <count> of <allowance>"; past the allowance it
     asks. README, code-organization.md, section 8, and sessions 03, 04
     and 06 say so.

107. **The codecs world keeps its own store limits** (temper 04.5,
     blocked 2026-10-14T04:35:36Z; plan defect). The fuzzy store test
     needs a `temper_engine_domain_forge::Limits`; the store's
     `fixtures::LIMITS` (`src/tests/fixtures.rs:31`) is in a unit test
     module a world cannot import. code-organization.md, section 5's
     "never redefined" binds a crate's own fixtures; a world keeps its
     own, as `tests/codecs/src/forgejo.rs:50`'s `limits()` does beside
     `temper-forge-forgejo`'s unit `fixtures::limits()`. So
     `tests/codecs/tests/fuzzy_store.rs` defines its limits as one
     constant at its top, the unit fixture's values copied, and
     `tests/codecs/Cargo.toml` adds the forge crates it needs as
     dev-dependencies; both are in 04.5's Files. The unit fixture stays
     where it is; nothing is moved or included by path.

108. **A session's own earlier files are in its later increments' Files**
     (temper 06.3, blocked 2026-10-14T04:43:10Z; plan defect). 06.3's
     Files omit `decode/walk.rs`, `client/items.rs`, `server/items.rs`,
     `client/start.rs` and `server/start.rs`, which 06.1 and 06.2 created
     and which its Shape must extend (the walker's tables and dispatch;
     `PullBranch::repo_id` in 06.2's pull), and
     `tests/codecs/tests/forgejo_items.rs`, whose pull literals gain
     `repo_id`. As a standing rule, the README's "the session's own
     earlier code, replaced" becomes "replaced or extended": a file the
     session's own earlier increments created is in each later
     increment's Files, its lines counted in the Size, and an expected
     literal there gains a field the Shape adds, with the value its
     transcript or fixture gives, no other value changed. A file another
     session created stays outside.

109. **`owner-permission` is a `403`** (temper 06.3, blocked
     2026-10-14T04:43:37Z; plan defect). 05.2 recorded it as `HTTP 403`
     with `{message, url}`, no permission. 06.3's rows assert its status
     and its error body is decoded by 06.5's `error_decoder`, in 06.5's
     failure table; `Permission::Owner` is read from a literal unit
     document, and 06.3's Proof counts eleven documents. forge.md, 3.4
     reads a `403` as `Forbidden` ("| `401`, `403` | `Forbidden` |").

110. **The store's codec files are in the Files of an increment that
     changes a stored value** (temper 02.1, blocked 2026-10-14T04:48:10Z;
     cross-session scope). 02.1 adds `Answer::Located`, which
     `Stored::Call` holds, so the README's "A durable record's codec
     follows its record" already asks it to change "the schema, the
     generated code and the translation in the same increment". Its
     files were not named in any Files: they now are, in that README
     rule, for every increment that changes a stored value (02.1, 02.4,
     and any other). The hand-written lines count in its Size; the
     schema, generated code and goldens do not.

111. **A commit status's creator may be `null`** (temper 06.4, blocked
     2026-10-14T04:58:18Z; design gap, in part). 05.3's
     `actions-combined-status` transcript has two statuses with
     `"creator":null`, written by Actions. 06's documents "carry what
     Forgejo says, typed, not translated", so `CommitStatus.creator`
     becomes `Option<User>`: none for `null`, the key still required,
     and the server writes none as `null`. What the component makes of a
     status with no creator (the domain's `api::Status.author` is a
     `u64`), and the job link's shape (forge.md, 3.2 says
     `<html_url>/actions/runs/<n>/jobs/<i>`; v16.0.5 writes
     `/fixture/specimen/actions/runs/1/jobs/0`, relative to the origin),
     are the designer's, in the run's `question.md` of this time. The
     designer answered at 2026-10-14T05:04:58Z, landed in `b65b41ef`
     ("Design: Actions statuses carry no author, and their links are
     relative"): `api::Status` loses `author`, which nothing reads (13.3's
     Files, with the store's codec by ruling 110); `job_link` reads the
     prefix as `html_url` whole or its path, and refuses `//`, another
     scheme or host; the component makes an origin-relative link absolute
     with `html_url`'s origin (forge.md, 3.2, 3.5, 9, 13; sessions 06.4,
     13.3, 13.4, 14).

113. **The small profile's numbers pass every worst case** (temper 08.1,
     blocked 2026-10-14T05:18:05Z; plan defect). 08.1 takes `small`'s
     constants from the engine worlds' fixtures, with `name_bytes` 256
     and the client's `answer_bytes` 65,536, beside jig's small children;
     the root's worst case refuses that mix
     (`temper-engine-domain/src/limits.rs:233-243`:
     `forge.named_calls < call_records`,
     `journal.result_bytes < people.words`,
     `journal.transcript_bytes < row_bound`, and the journal's room for
     deliveries). configuration.md, section 7 fixes no number ("Its
     numbers are constants in `temper-service`'s limits module, each
     documented with what it bounds"). So the author's profile
     stands: root journal `{commits: 3, held: 1536, writes: 6144,
     deliveries: 512, transcript_bytes: 65536, result_bytes: 2048,
     run_bytes: 16384}`, root loads `{loads: 2, rows: 1, bytes: 65536,
     reply_bytes: 65536, transcript_bytes: 65536}`, `forge.named_calls`
     32, the other fixture values kept; each raised constant's doc line
     names the check that needs it, and
     `the_small_profile_passes_each_domains_worst_case` is the check.
     Tighter numbers wait for startup's memory measure. `writes` was 4096
     in the first answer; 08.1's proof measured the root's worst case
     at 6048 (`route_bound` 6016 with jig small's `tasks::max_out` 724,
     and `call_records` 32), so it is 6144 (blocked
     2026-10-14T05:24:23Z).

114. **Make's rename needs no probe** (temper 07.3b, blocked
     2026-10-14T05:23:47Z; plan defect). Session 07's Choices promise
     "a Make whose rename meets a name anyway ends `Broken`", while
     skein's pinned `kernel::Op::Rename` reads "replacing what `to`
     named" (`kernel.rs:447-448`) and git.md, 4.1 names two operations.
     The pin submits a plain `renameat` (`skein-shell/src/ring.rs:827-831`,
     no flags), and a workspace is a directory: rename(2) refuses to
     replace a directory that holds anything (`ENOTEMPTY`) and replaces
     an empty one. Checked on x1 with `mv -T ws trash/7` onto a
     non-empty `trash/7`: "cannot overwrite 'trash/7': Directory not
     empty", exit 1; onto an empty one, exit 0. So Make keeps git.md's
     two operations, with no probe, and the promise holds where a
     collision would lose anything. Session 07 says so.

115. **After a reboot, `plan resume` must load the coordinator's
     thread** (2026-10-14; infrastructure). The machine rebooted for an
     apt upgrade at about 05:26Z and was back at 06:56Z. `plan resume`
     failed: `{"code":-32600,"message":"thread not found:
     01a1386e-..."}`, "the Codex daemon did not start the coordinator's
     turn". After a restart the daemon's `thread/read` answers the thread
     with `"status":{"type":"notLoaded"}`, while `plan`'s `resume()`
     (skein `scripts/dev/plan:812-818`) calls `thread/resume` only when
     `thread/read` fails. The supervisor called `thread/resume` with
     `plan`'s parameters (sandbox `danger-full-access`, approval
     `never`); the thread read `idle`, and `plan resume` then started the
     coordinator's turn (log.md, "Coordinator resumed,
     2026-10-14T07:03:22Z"). The fix, `resume()` also calling
     `thread/resume` when the status is `notLoaded`, is skein's kit, a
     proposal for the retrospective.

116. **`PartyInput::SignOut` lands in 11.3** (temper 11.3, blocked
     2026-10-14T07:17:06Z; plan defect, cross-session). 11.3's joint
     world copies ops's `Front`, whose `sign_out` sends
     `people::Event::SignOut { reply_to, sign_in }`; temper's root has no
     input for it. 08.5a's file already plans it ("add
     `PartyInput::SignOut`, field for field, routed to the people child
     as the core takes it"), but 08 is at 08.2a. So 11.3 adds the variant
     and its routing in `crates/temper-engine-domain/src/{boundary.rs,
     route.rs}`, with one test of the routing, and 08.5a translates
     jig's event to it. Sessions 11 and 08 say so. 11.3's Size, world
     code only, takes those production lines: at most 20 (8 at
     2026-10-14T07:25:24Z, `boundary.rs` 5 and `route.rs` 3).

117. **The core's restore ranges and the census are jig's** (temper
     08.2a, blocked 2026-10-14T07:21:33Z; plan defect crossing the root's
     load seam, put to the designer). The designer answered at
     2026-10-14T07:29:45Z, landed in `f5e4672d` ("Design: the core's
     restart ranges and the census are jig's"): `CoreRange::Tasks` is
     jig's `[Core, Tasks]`, filtered, a page that keeps nothing asked
     again; the census is jig's index walk, its rows in walk order,
     repeated owners kept, its cursor the last index row's key; the
     root's part (`store.rs`, `loads.rs` for `Range::Census` only) and
     the engine world's fake census join 08.2a, whose Size is at most
     570 (store/README.md, 5; session 08).

120. **A world-only increment may land the README's 20-line fix**
     (temper 18.1, blocked 2026-10-14T08:35:59Z; plan defect; the second
     after ruling 116's 11.3). On jig's real store, a goal's transcript
     continuation fails: the root's `archive.rs:377-385` derives
     `TaskTranscript.from` from the preceding page's last turn, so the
     opening window (1,1) becomes (1,2), which jig's pinned
     `transcripts/read.rs:605-619` answers `Unread`. store/README.md, 5:
     "A root cursor is a key of the root's range, and names the jig
     interval that holds it". The fix, about 6 lines, keeps the opening
     window and carries `after` as the cursor. As a standing rule
     (README), the README's "a fix the change, its Proof or the gate
     needs, at most 20 lines" in an increment whose Size is world code
     only counts against 20 production lines, with no question.

121. **The forge connection's deadlines are its holder's** (temper
     13.2, blocked 2026-10-14T08:38:38Z; design gap, session 13's
     questions). The designer answered at 2026-10-14T08:41:34Z, landed
     in `0be9dd34`: a start read's deadline is the component's
     `deadline` from when it is asked; connect and handshake have no
     limit of their own; `idle` is 4 s in the profile `small`, below
     Apache's 5 s `KeepAliveTimeout` default (forge.md, 7 and 8; session
     13, its limits table, 13.2).

122. **The native client's limits rise only as far as the pin accepts**
     (temper 12.1a, blocked 2026-10-14T08:49:46Z; plan defect). Session
     12's `text` 4,096, `document_bytes` 1,048,576, `rows` 64, `watches`
     16 are refused at jig `d91a89c4`: `link/limits.rs` copies `rows`
     into wire fields whose ceiling is 32, and jig-view's
     `Limits::compose` returns none with `text` 4,096 at every `rows`
     from 64 to 2 (`logs/temper-12/12.1a-preset-diagnosis.log`).
     configuration.md and client.md fix no number. So each starts from
     jig's ops world's value, which jig's tests run with, and rises toward
     the plan's target only as far as `link::Limits::new` and
     `Limits::compose` accept, `rows` at most 32; no repin.

123. **A changed type's users are in the increment's Files** (temper
     18.2, blocked 2026-10-14T09:18:48Z; plan defect). 18.2's Shape
     removes `observations::Record`'s wrapper; 29 uses of
     `.map(|record| &record.0)` in `tests/conformance/src/stories.rs`
     (14) and `src/scenarios.rs` (15) follow it, past the README's
     20-line fix. code-organization.md, 6 already asks it: "it changes
     the type in place, and its users in the same increment". As a
     standing rule (README, edits outside Files), a type's users are
     edited to its new form with no other change, in any number of
     lines, every assertion, input, expected value and name kept.

119. **02.4 lands before 02.3** (temper 02.3, blocked
     2026-10-14T07:52:40Z; plan defect, order). 02.3 moves the
     conformance worlds' adoption to `PartyInput::Application`; after a
     crash cut with the people's token durable, the outcome is gone on
     restart until 02.4 keeps it durably, so eight conformance fuzzy
     stories fail (`tests/conformance/src/lib.rs:641`; the gate at
     `6693b866`, 8 of 35 fuzzy failed). 02.4 needs only 02.2's kept
     outcomes. So 02.4 runs next and lands first, and 02.3's commit is
     replayed on it, its assertions kept; 08.5b's and 17.4's needs are
     unchanged.

104. **A combined status with no status** (temper 06.2, blocked
     2026-10-14T04:16:35Z; plan defect). Forgejo v16.0.5 answers
     `{"state":"","sha":"","total_count":0,"statuses":null,...}`
     (05.2's transcript `combined-status-empty`). `CombinedStatus.state`
     becomes `Option<StatusState>`, none for `""`, as forge.md, 3.2 reads
     `base.sha` ("none when empty"); the server's
     `Answer::CombinedStatus` takes `sha: Option<ObjectId>` the same way,
     and writes no statuses as `null`. `StatusState` gains no variant: it
     is also a commit status's and `PostStatus`'s word, where `""` is
     none of Forgejo's. The decoder takes `statuses` `null` or `[]` as no
     rows, and checks nothing across fields: "a `total_count` of 0 is
     `None`, whatever the state says" (forge.md, 3.2) is the component's
     reading (13.3).

124. **An adoption's scope is the confirmed party** (temper 15.1b,
     blocked 2026-10-14T09:42:59Z; plan defect, not design). 15.1b
     said "one `Request::Send` through the table with the project as
     its scope"; no design text names the project. jig's
     `client/domain/root.md`, 4, at the pin: "a scope per request: an
     opaque number its asker gives, which `jig-domain-client` names: the
     party the request is for. Only the requests of the party confirmed
     are ever sent"; temper's client.md, 3.3 keeps it ("restored with
     `Kept { scope }`, parked until their party is confirmed"). So the
     forge side does as jig's ops sides do at the pin: the root passes
     `jig_domain_client::Request::Confirmed { party }` to the side as
     `forge::Event::Confirmed { party }`, beside `root.table.confirmed`
     (ops `route.rs:193-196`); the side keeps it (`infrastructure.rs:348-352`);
     `Adopt` sends `scope: party`, and nothing while no party is
     confirmed (`infrastructure.rs:657-659`, `693-695`). The project
     stays in the `Ask`'s `Adoption`. 15.1b's Files hold it
     (`forge.rs`, `forge/form.rs`, `route.rs`); its fixture
     `on_adoption_page` confirms a party, and
     `adopt_sends_one_keyed_request_and_a_second_press_none` asserts the
     sent scope is it. Its Size (550), its ten tests' names and its
     Proof are unchanged; no upstream edit or repin.

125. **An out-of-date family's requests stay in the table, unsent**
     (temper 15.1b, blocked 2026-10-14T09:46:45Z; plan defect, not
     design). The design says what happens: jig's `protocol/client.md`,
     9, at the pin, "A client outside a range sends nothing more of that
     owner, shows that it is out of date and how to update ..., and
     keeps what is pending in its small store; updated, it sends what is
     pending under its keys"; temper's client.md, 4.4, "the button is
     disabled, and a pending adoption stays in the small store". skein's
     pinned `RequestTable` (`skein-lib/src/requests.rs`) parks by link
     and scope only, not by owner, and 15.1b named no mechanism. So,
     with no change to the table and no repin: the root's `take`
     (`temper-client-domain/src/domain.rs`) pushes no `Request::Send`
     whose ask is `Ask::Forge(_)` while the forge side is out of date
     (the ask, not the owner, so a kept adoption is held too); the
     attempt stays unanswered until the table parks it at a link or
     party change and offers it again, and it is held again. The side
     shows its pending request's progress as `Parked` while out of date.
     Retention still retires it as unknown (jig's
     `client/domain/root.md`, 4: "A request first sent longer ago than
     that, less a margin, is never sent again"), the 4.7 row "unknown
     past the retention". `domain.rs` joins 15.1b's Files;
     `out_of_date_disables_adopt_and_keeps_the_pending_request` also
     asserts that after the out-of-range hello and a link down and up,
     the root's `take` pushes no forge `Send` and the table still holds
     the request. No other test; 15.1b's Size and allowance (550, 1100,
     its parts 15.1ba and 15.1bb summed) are unchanged.

126. **13.3 within 2100 lines; 13.4's Size 1000** (temper 13.3b,
     blocked 2026-10-14T10:06:56Z; plan defect, size). 13.3 (Size 600,
     allowance 1200) counts 1819: 13.3a landed 356 (`1f433ab1`), 13.3b
     holds 1463 (`reads.rs` 540, `component/calls.rs` 349,
     `reads/done.rs` 319, `reading.rs` 253, `reads/size.rs` 123,
     `component.rs` 106, `connection.rs` 80, the rest 31). Each part is
     in 13.3's Scope: fifteen reads' plans, the resolution by id, the
     answer's running size, a call run when a connection is free (an
     idle one, a new one, or another forge's closed), and the reads'
     failures. Its Size ran low, as 13.1's (998 of 1100) and 13.2's
     (962 of 1100) nearly did. So 13.3 lands within 2100, its parts
     summed, with no further split; past 2100 it asks. 13.4's Scope is
     as dense (five reads, twelve writes, 3.4's classification), so its
     Size becomes 1000 before it starts, its allowance 2000 by ruling
     106. Its Scope, Files, Test shape and Proof are unchanged. Session
     13 says so.

127. **The worker's secret comes from the `Files` `Worker::new` takes**
     (temper 09.1b, blocked 2026-10-14T10:16:19Z; plan defect). jig's
     pinned `config::Worker.secret` is a file's path; 09.1b's sketch gave
     `Worker::new(settings, agent, limits, runtime, seed)` no file's
     bytes. 09.1b copies jig's reference `worker.rs`, whose
     `Worker::new(config, agent, files: &Files, ...)` reads the secret
     with `files.contents(&config.secret)` and checks it against
     `limits.link.secret` (`testing/jig-test-service/src/worker.rs:60-68`
     at the pin), its startup passing the files it read
     (`startup.rs:600`); 08.1's `settings::Files` is that type, "the
     private files jig's parts name, by path, as jig's reference takes
     them", already in `settings::Engine`. So `Worker::new` takes
     `files: &settings::Files` before `runtime`, as the reference does,
     and 10.1b passes the files its `check_files` read; configuration.md,
     6: a worker's secret is "Named by jig's parts", "Read by jig's
     components". `settings::Worker` stays `{ jig, root, git }`
     (decision 9), and no `Secret` is added to it. Sessions 09 and 10
     say so.

129. **A node's text is its text nodes' runs, printed once** (temper
     16.1, blocked 2026-10-14T11:45:30Z; plan defect, read from the
     design). client.md, 6.4 prints "every node in preorder, by its
     role", a heading as "its text", a button as `[n] NAME`; at jig's pin
     a node's text lives in text-node children (`jig-view` `Builder::run`
     admits only `Role::Text`, `builder.rs:134-135`), and `frame::control`
     gives a button both a name and a text child with the same label
     (`frame.rs:98-111`), so a literal walk prints every label twice.
     jig's `client/domain/view.md`, 3 decides it: a node carries "text,
     on text nodes only, as runs" and "a name, where its text does not
     give it". So a node's text is the runs of the text nodes under it
     that no nearer node of another role holds, NAME is its name or else
     its text, and the text nodes a line took print nothing more; a text
     node nobody took prints its own line, and a node of another role
     under a text-bearing one prints its own line. Session 16's 16.1
     Shape says so; its Size (900), Files (the golden `control_text`
     among them) and tests are unchanged.

130. **Startup checks the service's worst case through `system.rs`'s
     own** (temper 10.2, blocked 2026-10-14T12:07:40Z; cross-session
     scope). 10.2 checks "the two worst cases against `memory_bytes` with
     `jig_protocol_config::fits`" before anything is made; the service's
     envelope is 09.3's private `memory(settings, limits, store_limits,
     accounts, queue, operations)` (`crates/temper-service/src/system.rs:503`),
     which `Service::new` calls after the worker exists
     (`system.rs:55-71`, `worker.operations()`). So `system.rs` gains one
     `pub(crate) fn startup_worst_case`, taking the worker's operations
     as a number, that computes the queue, the operations and `memory`
     as `Service::new` does; `Service::new` calls it in place of its own
     lines, so the envelope is computed in one place (14.1 adds the
     forge's share there once). At most 30 lines change in `system.rs`,
     counted in 10.2's Size; every test of `system` keeps its name and
     expected values. The worker's operations come from its settings by
     the same function `Worker::operations` uses, made visible within
     the crate if needed, never copied. Session 10 says so.
     At 2026-10-14T12:10:13Z 10.2 found that formula across two crates
     (`Component::operations`, `temper-worker-protocol-git/src/component.rs:432`,
     `limits.operations + 1`; `Worker::operations` adds 1): so
     `temper-worker-protocol-git/src/limits.rs` gains `pub fn
     operations(limits: &Limits) -> Option<u32>`, which
     `Component::operations` calls, and `temper-service/src/worker_settings.rs`
     a settings-only `operations`, which `Worker::operations` and startup
     call; about 10 lines in `limits.rs`, `component.rs`, `worker.rs` and
     `worker_settings.rs`. A further shared helper ruling 130 needs, at
     most 20 lines, is the README's 20-line fix, with no question.

131. **The forge component's small values fit the client's answers**
     (temper 14.1, blocked 2026-10-14T12:13:36Z; plan defect). forge.md,
     7 checks at start "for each listing, at least one of its largest
     rows fits the client's `answer_bytes`, so no page size is 0", and
     fixes no number: `string_bytes` "at least the forge's
     `name_bytes`" (256), `body_bytes` "at least the client's `op_bytes`
     with a marker" (`2 × 512 + 21` = 1,045). Session 13's table chose
     `body_bytes` 65,536, `string_bytes` 1,024 and `inner` 64, which make
     `Items`' largest row (`body + body/2 + string + inner × (string +
     16) + Summary`, the author's reading of `limits::largest_row`)
     about 166 KB, past the small client's `answer_bytes` 65,536 (ruling
     113), so its page size is 0. Raising `answer_bytes` to 262,144
     breaks jig's small store preset and still gives a page of 1. So the
     component's small values fall, within forge.md's floors:
     `string_bytes` the larger of 512 and `name_bytes`, `body_bytes` the
     larger of 16,384 and `2 × op_bytes + 21`, `inner` 32; `Items`' row is
     then about 42 KB. A body past 16,384 bytes reads as `Long` (forge.md,
     3.3), as any body past `body_bytes` does. If a listing's page size is
     still 0, `inner` halves until it is not, at least 8, and the report
     names the values. The client's, the root's and the store's presets
     are unchanged; a larger profile, later, raises them together.

132. **One process's io is its parts' io, summed** (temper 10.2,
     blocked 2026-10-14T12:16:25Z; plan defect). 09's and 10's default,
     "one io from the engine's limits", refuses every small document:
     jig's small `io.sockets` is 16 (`jig-protocol-config/src/profile/small.rs`),
     the pinned parser has no `io.*` override (`overrides.rs:8-35`), and
     one process needs at least 24 (the author's count: the client
     listener and 4 client connections, the worker listener, the accepted
     worker and the worker's link, `slots × 4` agents, `git_component.operations
     × 4`). jig's reference startup composes one process's io from each
     part's: the engine's `limits.io`, then each worker's `sockets` and
     `names` added and `intake`, `receive`, `output` and `sends` the
     larger (`testing/jig-test-service/src/startup.rs:547-600`), and jig's
     worker part has `limits.io` of its own (`worker.rs:17`). So temper
     does the same, in one function `Service::new` and startup's
     envelope share (ruling 130's `startup_worst_case` takes it); 10.2's
     check counts the entities the configuration enables (the count
     above, the accounts' connections when accounts are configured)
     against the summed `sockets`, refusing at `engine.limits.io.sockets`
     "by how much". No profile constant and no parser change. Session
     10 says so (09 has ended).
     At 2026-10-14T12:19:43Z 10.2 asked for the means: the crate-internal
     `Service::new` takes the composed `io::Limits` as a parameter
     (`Service::new` holds a built `Worker`, which keeps no settings),
     startup computes it once with `checks::process_io` and passes it to
     `startup_worst_case` and `Service::new`, and the fixtures' callers
     pass it; no second constructor. From then, any change rulings 130
     and 132 need inside `temper-service` and the git component (a
     crate-internal function's signature, its callers, a fixture's call),
     every test's name and expected value kept and no public type's
     fields changed, is in 10.2's Files, named in its commit body, with
     no question.

133. **A TLS forge's trust roots are priced as the accounts' are**
     (temper 14.1, blocked 2026-10-14T12:22:01Z; plan defect, a boundary
     type). skein-tls's client worst case leaves out the shared trust
     configuration, "counted once by its service"
     (`skein_tls::roots_worst_case(count, each)`); 14's sketch gave
     `ForgeRuntime` its number, address and transport only, so a TLS
     forge's roots go unpriced, or are priced at the reader's cap (1,024
     roots of 65,536 bytes). `AccountsRuntime` already carries
     `trust_roots: u32` and `trust_root_bytes: u32`
     (`crates/temper-service/src/accounts.rs:26-27`), checked at
     `accounts.rs:82` and priced at `system.rs:556`. So `ForgeRuntime`
     gains the same two fields, filled by whoever reads the trust roots
     for it as for the accounts; `system`'s memory adds
     `roots_worst_case` for each TLS forge, as for the accounts; a TLS
     forge with no roots is refused as `accounts.rs:82` refuses; a
     plaintext forge carries 0 and 0. Session 14 says so.

134. **A native shell idle before it loaded is a refused directory**
     (temper 16.3b, blocked 2026-10-14T12:35:21Z; plan defect). Session
     16's default printed a line "on 12.1's `DirectoryRefused` notice";
     at jig `d91a89c4`, `jig-shell-native`'s `files.rs:230-256` meets a
     failed private open by marking the failure and closing its roots,
     and `Shell::is_empty` also needs `closing`, so no notice reaches the
     native client, which stalls (findings/jig-client.md, Discrepancy 4;
     later.md's row). A malformed token or store file is met the same
     way. So the native client reads a shell that has not loaded and
     holds no work, deadline, submission or completion as
     `DirectoryRefused`, closes it, and `temper open` prints `temper
     open: DIRECTORY: refused (not a private directory of this user's,
     or its records unreadable)` and exits 1: client.md, 6.1's exit 1,
     with no cause the pin cannot tell apart. The terminal does not
     check the directory itself (no second reading of what the shell
     reads). One test, `a_refused_directory_exits_one_with_its_line`
     (a directory with group bits set), joins 16.3's Test shape. Saying
     why stays jig's (later.md).

136. **The binary advertises raw message bytes, the domain keeps
     rendered ones** (temper 10.4, blocked 2026-10-14T13:10:30Z; plan
     defect). 10.4's first real-binary proof refused `workers[0].limits`:
     `temper-worker-domain/src/limits.rs` checks `host.event_bytes +
     SEPARATOR <= agent.message_bytes` (`:84`, `SEPARATOR` 2 at `:39`)
     and `bounds.message_bytes <= host.event_bytes` (`:89`), while 10.3's
     `agent()` (`crates/temper/src/lib.rs:134-175`), copied from jig's
     reference, advertises `bounds.message_bytes =
     limits.domain.run.message_bytes`, the rendered cap; smith renders a
     message as its label, two bytes and its text. So no derived profile
     could start; 09's and 10's fixtures set the three apart and hid it.
     The hello's bound and the host's event bound are raw bytes, the
     domain's `message_bytes` rendered ones, and both checks stay (a hello
     must not promise more than the domain keeps). So the binary's
     `agent()` advertises `run.message_bytes - SEPARATOR` (checked; a
     profile where it underflows is refused naming the agent's
     `message_bytes`), with smith's host limits as they are, and the
     fixtures use raw and rendered values two bytes apart (the worker's
     `hub.event_bytes` 598 beside `tool_payload` 600). Those lines in
     `crates/temper/src/lib.rs` are in 10.4's Files, counted in its Size;
     the check is 10.4's Proof, the real binary starting.

137. **A fake forge write that Forgejo answers with a document reads it
     back through a call** (temper 14.3b, blocked 2026-10-14T13:21:42Z;
     plan defect). 14.3 makes "each other route ... one domain `Call`",
     "its reply encoded by 06's `server::encode`", and its Don't forbids
     answering "from the domain's store but through a call". The fake
     forge domain answers writes with ids (`Created(number)`,
     `Commented(id)`, `Reviewed(id)`, `Done`;
     `testing/temper-fake-forge-domain/src/api.rs:428-442`), while
     Forgejo answers them with documents the component reads
     (`Review.commit_id`, an issue's user, times, title and body, a
     pull's `head.sha`, `base.sha` and `merge_base`;
     `temper-engine-protocol-forge/src/writes.rs:588`). So such a write
     makes its one write `Call`, then one read `Call` of the document
     Forgejo returns, and answers with what the read returns: nothing
     made up, the write made once, the HTTP request counted once in the
     tally, and a fault drawn after the write as before. The fake
     domain's boundary types are unchanged. Session 14 says so.

138. **16.4's process proof waits for 12.2** (temper 16.4, blocked
     2026-10-14T13:46:53Z; order). 16.4's proof signs in by the printed
     link, then prints `== Connecting ==` and `== Offline ==` and never the
     inbox (`logs/temper-16/16.4-proof{,-diagnostic}.log`); 12.2's
     process proof, "a chat answered by the real binary" through 12.1's
     native client, failed at 13:34Z on the same path (exit 100). The
     native client's stream to the real binary is 12.2's to prove, and
     12.1's code is session 12's. So session 12 diagnoses and fixes it
     within 12.2 (the README's edits outside Files apply to a fault in
     another session's file); session 16 edits neither the native client
     nor the service's client side for it, finishes 16.4's other work on
     its branch, and runs its process proof once more after 12.2 lands.
     The coordinator gives session 12 16.4's evidence now. If 12.2's
     proof passes without meeting the fault, 16.4 diagnoses against it
     and asks with a concrete fix. README's 16.4 row gains "its Proof
     after 12.2".

139. **14.3 within 2400 lines** (temper 14.3c, blocked
     2026-10-14T13:46:53Z; plan defect, size; as ruling 126 for 13.3).
     14.3 (Size 600, allowance 1200) counts 1559 before its HTTP
     machines: 14.3a 551, 14.3b 459, 14.3c 549, all in
     `testing/temper-fake-forge-protocol` (`response.rs` 548, `answers.rs`
     458, `routes.rs` 320, `tally.rs` 125, `lib.rs` 48, `errors.rs` 43,
     `Cargo.toml` 17), each within 14.3's Scope: every Forgejo route the
     component reads or writes, as requests, documents and responses.
     About 600 more remain (the Service and Server machines, ruling 137's
     read-back, Pipe and Joint). So 14.3 lands within 2400, its parts
     summed; past 2400 it asks. Its Scope, Test shape and Proof are
     unchanged, the nine HTTP and three joint tests among them.
     At 2026-10-14T14:19:32Z 14.3d, complete with its proof passed (35
     tests, the nine HTTP and three joint among them), counted 2452:
     14.3d's 893 (`server.rs` 574, `service.rs` 161, `response.rs` 150, the
     rest 8) on 1559. It lands at 2452, 14.3 closed; nothing more is
     added under 14.3.

140. **The engine root takes the configured document limits** (temper
     12.2, blocked 2026-10-14T14:47:48Z; plan defect). 12.2's chat
     reached an Activity page of three rows, which the native client
     (`rows: 2`, ruling 122; `temper-shell-terminal/src/limits.rs:51`)
     refuses as `TooLarge`, while the process configuration's
     `documents.rows` 2 never reached the root: `core_limits` uses the
     constant `DOCUMENTS` (`temper-engine-domain/src/limits.rs:97-103`,
     jig small's rows 4) and 08.1's `Limits` has no `documents`.
     domain/engine.md, 13: documents' limits "take jig's small profile's
     values until temper's configuration names its own". So the root's
     `Limits` gains `documents: jig_domain_core::documents::DocumentLimits`,
     `core_limits` uses it, the service's engine builder (`limits::engine`,
     which one process uses too) copies the configured
     `jig.core.documents` (corrected at 2026-10-14T14:55:30Z: the worker's
     `root_limits` builds `temper_worker_domain::Limits`, which holds no
     documents), `DOCUMENTS` goes, and every
     other constructor passes jig small's values, unchanged; one test
     shows configured rows reaching the core. Its users are edited by
     ruling 123. Whether the shipped configuration's default (profile
     `small`, no overrides: rows 4) must change, since the native client
     takes 2, is the designer's, in the run's `question.md` of this time.
     The designer answered at 2026-10-14T14:55:21Z, landed in `b5dd62bc`:
     the configuration moves, for texts as well as rows (the client's
     wire bounds a document's texts by its `text`, 128): `engine.limits`
     overrides `documents.rows` 2 and `documents.text_bytes` 128, and
     12.2's `the_process_configurations_documents_fit_the_native_client`
     holds the process tier's configuration to the native client's
     limits; no start check (configuration.md, 7; sessions 12 and 17).

141. **The fake forge can hold an unborn repository** (temper 17.1b,
     blocked 2026-10-14T14:55:30Z; plan defect). 17.1 answers "each
     transcript's case ... over the equivalent fixture"; 05.4's
     `clone-empty` transcript comes from an empty repository
     (`tools/git-transcripts/src/cases.rs:44`, `Init::Empty`), while the
     fake forge domain's `repository()` always stores a first commit and
     its default branch (`testing/temper-fake-forge-domain/src/scenario.rs:43-46`)
     and `Setup` has no unborn state (`api.rs:734-743`). So the fake
     forge domain gains one scenario helper for an unborn repository (no
     commit, no branch), beside `repository()`, `Setup`'s shared fields
     unchanged; its clone reports no heads. At most 30 lines in
     `scenario.rs` and its users, outside 17.1's Files, named in the
     commit body and counted in 17.1's Size; every existing test's name
     and expected value kept. clone-empty stays in 17.1's proof.

142. **17.1 within 2000 lines** (temper 17.1b, blocked
     2026-10-14T15:21:49Z; plan defect, size; as rulings 126 and 139).
     17.1 (Size 600, allowance 1200) counts 1745: 17.1a 368, 17.1b's WIP
     1377 (`testing/temper-fake-git`: `program.rs` 424, `fixtures.rs` 373,
     `answer.rs` 312, `transport.rs` 218, the rest 16; ruling 141's
     `scenario.rs` 30 and `lib.rs` 4), each within 17.1's Scope: a fake
     git program answering 41 transcripts' cases over the fake forge.
     Its final proof and the hosted file-system check remain. So 17.1
     lands within 2000, its parts summed; past 2000 it asks. Its Scope,
     Test shape and Proof are unchanged. For the retrospective: the
     planner's Sizes for test support and fakes ran low as for component
     code (rulings 102, 106, 126, 139, 142).

143. **14.5 lands at 787 lines** (temper 14.5, blocked
     2026-10-14T15:23:14Z; plan defect, size). 14.5 (Size 300, allowance
     600) is complete, its Proof passed against the real Forgejo and
     runner (exit 0, the 39 component facts holding, 25 s), at 787:
     `tools/forgejo-conformance`'s `component/hosted.rs` 293,
     `component/writes.rs` 194, `component/reads.rs` 110, `component.rs`
     85, `facts.txt` 39 and the rest 66, with one line of
     `temper-engine-protocol-forge/src/writes.rs` (a review's success is
     Forgejo's 200, not 201) under the README's 20-line fix. It lands as
     it is; session 14 ends.

144. **A workspace's agent start carries one path per directory**
     (temper 15.3, blocked 2026-10-14T16:08:21Z; cross-session scope, a
     gap in 09's worker layer). domain/worker.md, 4.2: "Prepare the
     workspace", then "Start an agent process with the charter"; pinned
     smith's `translate.rs` (`encode_workspace`) refuses a start whose
     `paths` do not match its directories one for one (`MissingValue`),
     and `temper-service/src/worker.rs:664-670`'s `start_values` sends
     `paths: Box::default()` always, so every workspace start went
     `Unsent` and 15.3's story never reached its second model turn. So
     the worker keeps `WorkerRuntime.path`, and `start_values` fills one
     path per directory, in the start's directory order, as git.md,
     4's layout names them (`<root>/workspaces/<token>/<repository>`,
     the token the prepared directory's own); the retained path is
     priced in the worker's worst case. Session 15 makes the fix within
     15.3, outside its Files (09 has ended), at most 60 production lines
     in `worker.rs` and its pricing, named in the commit body, with one
     test, `a_workspace_start_carries_one_path_per_directory`. 15.3's
     Proof is its check.

145. **15.3 traces its stopped change in stored state and the
     one-process world, not by re-running the process tier** (temper
     15.3, blocked 2026-10-14T16:34:51Z; process). After ruling 144, 15.3's
     process proof reaches a real push (`temper/2/c3` on the throwaway
     Forgejo) and stops there: no pull request, no gate's run reaching
     the fake model, `main` unchanged, the test's 30 s deadline passed
     (`logs/temper-15/15.3-proof-25.log`); its 19 runs of the hour each
     took that deadline. Session 15's own question ("if no gate's run
     reaches the fake model, raise it with the forge's change state") is
     answered: trace the producer's delivery into the forge's procedure
     (the engine's forge records, the outbox, the component's
     `OpenPull`) in the last run's stored state, read by the store's
     codecs, and reproduce it where it is cheap, the one-process world
     (17.2's, on skein's simulator, whose 17.3 adopts and pushes a change
     now) or the system world; fix it where it is found, a production
     fault outside Files by the README's 20-line fix or else a question
     naming its cause. The process proof runs again only after a change
     aimed at a named cause, once per change; a third failure on the
     same symptom is a question with the logs.
     At 2026-10-14T17:30:54Z, after ruling 146, the third failure came:
     the producer's change done and pull 1 opened, the gate's task
     created and its preparation refused three times before any model
     query (`logs/temper-15/15.3-trace-after-146.md`). 15.3 goes on
     tracing the same way, first in the root's
     `Fact::PreparationFailed { task, attempt, preparation }`
     (`temper-engine-domain/src/link.rs:224`; decision 3) of that run,
     then in the system or one-process world; the count of same-symptom
     process failures starts again at the next named cause. Ruling 147
     replaces this amendment's first read.

146. **A produced change's result names the resource the worker and the
     forge settled** (temper 15.3, blocked 2026-10-14T16:54:15Z;
     cross-session scope, a gap of 08.4's). llm.md, 5: the change's
     "resource comes from what the worker and the forge settled, never
     from" the model's words; `temper-engine-smith/src/finish.rs:107`
     refuses a declared change with no resource (`Problem::Missing`),
     and `temper-service/src/tools.rs:66` passes `None` always, so every
     produced change ended `Failed(Invalid)` and no `OpenPull` was asked
     (15.3's trace of its stored state, `logs/temper-15/15.3-trace-findings.md`;
     the system story reproduces it in 0.08 s). So the root gains a
     fenced accessor of the settled change resource for a run, and the
     service's result routing passes it to `temper_engine_smith::result`;
     boundary fields unchanged. At most 40 production lines in
     `temper-engine-domain` and `temper-service`, outside 15.3's Files,
     named in the commit body, with three tests: none settled (`Missing`),
     one settled (the resource), more than one (refused as ambiguous),
     each under 20 ms. The parent chat's first `Invalid` stays to trace,
     under ruling 145.

147. **15.3 finds its gate's refused preparation in a world on the
     process configuration's limits, else in one marked process run**
     (temper 15.3, blocked 2026-10-14T17:45:25Z; plan defect, in ruling
     145's amendment). The amendment asked first for the run's
     `Fact::PreparationFailed`, which cannot exist: `result_event`
     (`temper-engine-domain/src/link.rs:250-254`) queues it only from a
     worker's `Payload::Answer` whose parts name a preparation; the gate,
     task 5, never claimed an attempt; and the service's `drain_facts`
     (`temper-service/src/engine.rs:100-107`) counts the root's facts and
     keeps none. That read goes. Proof 28's retained state shows task 5
     Preparing, then BackingOff, three times, 0.41 to 0.49 s after each
     start, its brief's gather 5 s: a refusal before claim, at one of
     these sites:
     - temper's root: `archive.rs`'s `transcript_failed`,
       `begin_dependency_read` (its read table full) and
       `dependency_failed`; `assemble.rs`'s `BriefAssembled { ready:
       false }` (a connector section with no owner, or bytes other than
       its size);
     - jig's core at the pin (`jig-domain-core/src/routing/`):
       `children.rs:68-74` (the brief child's `Failed` or `Refused`),
       `run_preparation.rs`'s `plan_brief` (a part `read_brief_part`
       refuses, against `brief.read_bytes`, `brief.brief_bytes` and
       `brief_parts`), `brief_assembled` and `workspace_prepared` (no
       context, no fresh attempt, the proofs' table full), and
       `route_claim_refused`.

     15.3 names the site by the first of these two means that shows it:
     1. **The system world's gate story** (15.3's diagnostic of
        `15.3-system-gate-gather-5s.log`, the gate Running in 0.086 s),
        its engine limits built from the process tier's configuration by
        `temper_service::limits::engine`, as startup builds them, in place
        of the world's fixture limits, and its fake forge holding the
        proof's pull (its files and its comparison). The site is read from
        the world's domain state between steps, or from temporary
        `eprintln!` lines at temper's sites above. At most three such
        world runs; if none refuses the gate, the second means follows.
     2. **One process run** with temporary
        `eprintln!` lines (the task, the site, and at `assemble.rs` the
        section's size and byte count) at temper's sites above and in
        `route/now.rs`'s `Now::RunPreparationFailed` arm, which jig's
        `brief_assembled` and `workspace_prepared` paths request and its
        `plan_brief` and `children.rs` paths do not. It is the process run
        aimed at this cause; its log is kept under `logs/temper-15/`.

     No diagnostic line is committed, jig's checkout is never edited, and
     no production observer is added. 15.3 then fixes the cause where it
     is: temper's code within the README's 20-line fix, or the process
     tier's configuration in its fixture. A fix in jig's core, past 20
     production lines, or in the shipped configuration is a question
     naming the site and its numbers. Unchanged: ruling 145's other fences
     (a process proof once per change aimed at a named cause; a third
     same-symptom failure is a question, counted from this cause), every
     assertion and expected literal, the gate's script, boundary fields,
     the pins, and rulings 144's and 146's fixes, landed as 15.3a
     (`ead325c3`).

148. **17.3 runs its own world with typed diagnostics; only an
     unchanged re-run is barred** (temper 17.3, blocked
     2026-10-14T17:53:06Z; process). The steer of 2026-10-14T17:34:50Z
     told session 17 to trace 17.3's adoption failure "not by
     re-running", after ruling 145, which bars re-running the process
     tier (30 s a run through the gate lane, 19 runs in one hour). 17.3's
     Proof is the one-process world, whose stories fail in 0.47 to 0.75
     s, and its records (`logs/temper-17/17.3-stored-trace.md`) hold no
     task subphase and no agent's `Start`, `Failed` or `Refused`, while
     `drain_facts` keeps no root fact (ruling 147). So, for 17.3 until it
     lands, the steer's words read "not by re-running unchanged": each run
     of `temper-one-process-world` follows a change aimed at a named
     cause, or a diagnostic added to the world that names what it will
     show. The world's failure output may carry, typed and formatted only
     when a story fails (development.md, 14.3's checklist), each task's record
     with its phase and subphase, and each agent's `Start`, `Failed` and
     `Refused`; that code is world code, in 17.3's Files, and stays only
     if it serves the stories' failure messages. At most five diagnostic
     runs; a sixth without a named cause is a question with their logs.
     No production observer, no process-tier run (15.3's), and a fault
     in a file session 15 is changing still goes to the coordinator for
     session 15. 17.3's Scope, Files, Test shape, Proof and its 20-line
     fix allowance (12 used, all in `testing/temper-fake-git`) are
     unchanged.

149. **15.3 makes one more marked process run, at the brief's and the
     claim's routing** (temper 15.3, blocked 2026-10-14T18:04:26Z;
     process, ruling 147's means spent). Ruling 147's three system-world
     runs stopped before any gate (the fixture's authority and permission
     tables were priced for its own limits, then its adoption failed), and
     its marked process run (`logs/temper-15/147-process-diagnostic.log`,
     31.2 s) printed none of its markers. Ruling 147 left one of its
     candidates unmarked: jig's `route_claim_refused` requests
     `Now::DropAssignment`, not `Now::RunPreparationFailed`. So 15.3
     makes one more process run with temporary
     `eprintln!` lines that print the task, the site, sizes and counts,
     never a section's bytes, at:
     - `route/now.rs`'s `Now::BriefCorePlanned` arm (the count of
       sections), `Now::CompleteBrief` arm (the count of placed sections)
       and `Now::DropAssignment` arm;
     - `route/forge_route.rs`'s `forge::Request::BriefSized` arm (the
       section, the size or none, `cutting`);
     - `temper-engine-domain-forge/src/brief.rs`'s `brief_answer`, where
       a fetch ends without a section (its stage and the error's kind).

     The gate with no `BriefCorePlanned` names jig's `plan_brief`; a
     `DropAssignment` names the claim's refusal; sections sized with no
     `CompleteBrief` name the brief child's `Failed` or `Refused`
     (`children.rs:68-74`), read against that run's root brief limits
     (`147-result.md`: sections 3, `brief_bytes` 384, each budget 128).
     Its log is kept under `logs/temper-15/`. The rest of ruling 147
     holds: nothing diagnostic committed, jig unedited, the fix where the
     cause is, else a question. If this run names no branch either, 15.3
     makes no further run and asks, with both runs' logs.

150. **15.3 marks the pull-files read's own sites once, then fixes with
     a test below the process tier** (temper 15.3, blocked
     2026-10-14T18:17:05Z; process). Ruling 149's run
     (`logs/temper-15/149-process-diagnostic.log`, 31.4 s) named the
     branch: for the gate, task 5, the forge's `brief_answer` ends at
     `stage=Files` with `InvalidAnswer` and `BriefSized` is none, three
     times, so jig's brief child fails the required Pull section
     (`jig-domain-brief/src/orchestrated.rs:322-329`, then
     `children.rs:68-74`). `BriefStage::Files` is the whole
     `api::Read::PullFiles` operation of
     `temper-engine-protocol-forge/src/contents.rs`, whose plan rereads
     the pull, lists the files and reads each side, with about a dozen
     `InvalidAnswer` returns. So 15.3 makes one more process run through
     with temporary `eprintln!` lines at `contents.rs`'s
     `heard`, `row`, `done`, `next_side` and `side` returns of
     `InvalidAnswer`, and at the domain client's operation and answer
     check (`temper-engine-domain-forge-client/src/calls.rs`), each
     printing its line number, the HTTP status, the document's or row's
     kind, the presence and format checks, counts and byte sizes, never
     a value. Then, the site named, 15.3 fixes it as ruling 147 says, and
     adds one test in the fixed crate that meets the same answer below the
     process tier (a document of the same shape, its values made up, the
     test named for the behaviour), outside the 20 production lines; then
     the process proof runs once. If this run names no site, 15.3 asks
     with its log and makes no further run. Rulings 147's and 149's other
     terms hold.
     At 2026-10-14T18:36:45Z the run had named `contents.rs`'s `heard`
     (HTTP 200, a files row whose `status` Forgejo v16.0.5 writes
     `changed`, refused by `temper-forge-forgejo/src/client/files.rs:211`);
     15.3 fixed it in one line under the README's 20-line fix, with a
     decoder test, and the gate's task now runs.

151. **The process story's fake gate answers with `label`** (temper 15.3,
     blocked 2026-10-14T18:36:45Z; plan defect). After ruling 150's fix,
     the gate's run reaches the fake model, whose scripted finish
     (`crates/temper/tests/support/forgejo.rs`, 15.3's own file) sends
     `{"form":"verdict","verdict":"1",...}`; pinned smith's `declared`
     reads a verdict's name from `label`
     (`smith-protocol-llm/src/contract.rs:279-280`), and temper's start
     names each verdict rule by its code's decimal
     (`temper-engine-smith/src/start.rs:171`, the gate's choices codes 1
     and 2, `forge_route.rs:1740-1750`), so `"1"` is right and the member
     is wrong. "The gate's script" kept unchanged in rulings 145 to 150
     and the steer of 2026-10-14T17:34:50Z means: no assertion, expected
     literal or scripted step loosened or removed so the story passes. A
     member or value corrected to the pinned contract is a fixture's input
     corrected so the test checks the same behaviour (README, edits
     outside Files). So the member becomes `label`, its value `"1"` and
     every other member kept; the gate is the check below the process
     tier, and the process proof runs once, as the change aimed at this
     cause. Ruling 145's same-symptom count runs on from ruling 150's fix.

152. **The one-process world's share is measured as "Done when" 11
     measures it** (temper 17.4, blocked 2026-10-14T18:53:13Z; process).
     code-organization.md, 5's shares are "guides on the summed time of
     their tests"; the gate runs its tests side by side, so its log's
     times (`temper-one-process-world`'s focused 2.278 s at 17.3's gate,
     its own proof's 0.246 s and 0.815 s) are not the share. "Done when"
     11 measures serially. So session 17, after 17.s1 (ruling 153) and
     before 17.4, runs that item's focused command once, as the README
     quotes it, and its `awk` sum, then the same `awk` over the lines
     naming `temper-one-process-world`, and reports both sums:
     - the world within 2.0 s and the total within 4.0 s: 17.4 goes on,
       the world made no cheaper now, and 17.4's new focused stories each
       aim at 0.25 s, as its Test shape says. This replaces
       code-organization.md, 5's "made cheaper before more stories land
       in it" for session 17 while the total holds;
     - the world past 2.0 s, or the total past 4.0 s: the world is made
       cheaper first, its slowest story first, with the cause named, then
       the command runs once more.

     No other measurement; the fuzzy half waits for "Done when" 11 at the
     run's end.

153. **The shipped profile admits a merge's conflicts: step 17.s1**
     (temper 17.3, found in its landing `c554fa64`; plan defect). The
     profile `small`'s checkout limits carry `conflicts: 0` and
     `path_bytes: 0`, "from the checkout world fixture"
     (`crates/temper-service/src/limits.rs:170-173`), and
     `crates/temper-worker-domain/src/assignment.rs:52-54` refuses every
     `Start::Merge` at `conflicts == 0` ("merge in progress"), so a
     default service never resolves a merge (git.md, 4.6). 17.3's world
     passed by overriding them to 1 and 64 in its document
     (`tests/one-process/src/fixtures.rs`). configuration.md, 7 fixes no
     number. So the profile takes `conflicts` 4, the checkout merges
     world's (`tests/worker/checkout/src/merges.rs:49`), and `path_bytes`
     255, so that `largest_output`'s paths, 4 × (255 + 1) = 1,024
     (`temper-worker-protocol-git/src/limits.rs:63-70`), fit the profile's
     `stdout_bytes` 1,024. Step `temper-17.s1`, session 17's, before
     17.4; its session file is `temper/17-one-process.md`'s 17.s1.
     `reserve: 0` (`limits.rs:45`) is the profile's other zero, and stays:
     at 0 the forge client keeps no share of the rate for its keep and
     slow calls, and refuses nothing
     (`temper-engine-domain-forge-client/src/calls.rs:191`).

154. **Session 17 explains its referee's stop on a two-turn chat: step
     17.s2** (temper 15.3, blocked 2026-10-14T19:02:44Z; cross-session
     scope). After ruling 151, 15.3's process proof merges the pull and
     edits `main`, and the native person still misses "Change requested"
     at step 20, though the stored transcript holds it, uncut, among the
     last two activity rows (`logs/temper-15/151-result.md`). 15.3's cheap
     probe, the one-process chat with one `read` before its text and wait
     (`logs/temper-15/151-world-two-turn-{world,fixtures}.rs`), stops in
     0.150 s at `tests/one-process/src/referee.rs:420`: `Violation {
     promise: Commit, why: "outside delivery preceded its durable
     prerequisite: Turn(1, 1, 1)" }` (`151-world-two-turn.log`). The
     referee and its world are session 17's, and 17.3 already met one
     observer error of this promise (a worker's acknowledgement taken for
     durability, its report of 17:40:12Z). So session 17, after 17.s1 and
     before ruling 152's measure, runs step `temper-17.s2`
     (`temper/17-one-process.md`): it names the cause; a referee's
     misreading is fixed there; a turn delivered before it is durable is
     a production fault, and a question naming its site. Session 15 in
     the meantime traces the native reply's delivery and read in source
     and its retained states, with no run of the one-process world or the
     process tier, until 17.s2 reports.

155. **17.s1's startup fixture agent takes the profile's conflicts**
     (temper 17.s1, blocked 2026-10-14T19:10:13Z; plan defect). With
     ruling 153's 4 and 255, the gate's `startup_makes_the_one_process_from_the_example`
     and `a_workers_missing_or_oversized_secret_keeps_its_absolute_path`
     refuse `workers[0].limits`: `temper_worker_domain`'s worst case needs
     the checkout's `conflicts` and `path_bytes` within the agent's
     (`crates/temper-worker-domain/src/limits.rs:74-75`), and the service's
     test agent is written by hand with 0 and 64
     (`crates/temper-service/src/tests/fixtures/agent.rs:11-12`), while the
     shipped worker derives its agent's from the checkout's
     (`crates/temper-service/src/worker_settings.rs:87`). So the fixture's
     `conflicts` becomes 4 and its `path_bytes` 255: a fixture's inputs
     changed so its tests check the same behaviour, the README's 20-line
     fix, outside 17.s1's Files, named in its commit body; every check,
     test name and expected value kept.

156. **17.4 has three more typed diagnostic runs** (temper 17.4, blocked
     2026-10-14T20:08:02Z; process). Ruling 148's five diagnostic runs,
     which the unblock of 2026-10-14T18:58:31Z carried into 17.4, are
     spent; after a power-loss cut the world's restarted service sends and
     answers nothing (`logs/20261014T195559-0.log`), and the source trace
     (`logs/temper-17/17.4-w76-source-trace.md`) leaves two branches: the
     cut came before the native client's private save of its pending
     request, or the restored table holds it under a scope or retention
     that never sends it again. A world run is under 2 s and its
     diagnostic is typed and printed only on failure. So 17.4 may make
     three more diagnostic runs, each naming before it runs which branch
     it tells apart; the first prints, on failure only, the request keys
     sent before the cut, the private cache's table key, scope, write,
     sync and rename completion, and the restored table's keys and
     scopes. Past three, a question with their logs. A fault in the native
     client (session 12's), the service or jig's shell is a question
     naming its site, with no production edit. Ruling 148's other terms
     hold.

157. **Adopt is disabled until a party is confirmed: step 17.s3**
     (temper 17.4, blocked 2026-10-14T20:21:44Z; cross-session scope, a
     gap in ruling 124's means). Ruling 124 has the forge side send
     "nothing while no party is confirmed", and
     `temper-client-domain/src/forge.rs:194-196` returns from `Act::Adopt`
     with no party; but `temper-client-view/src/forge/page.rs:75-81`
     disables the button only while a request is pending or the side is
     out of date, so a press before the first frame's confirmation is
     dropped unseen (17.4's diagnostic 2: "Adopt presses had frame
     [(1,false)]"; `logs/temper-17/17.4-w76-cause.md`). client.md, 4.4:
     "Adopt makes one keyed request through the table", and the button
     "is disabled" in the other states that send nothing (pending, out of
     date). So `forge::Form` gains `confirmed: bool`, the side's `party`
     being some, and the view disables Adopt while it is false; a press
     then reaches `act_on` only with a party. Its users are edited by
     ruling 123. Step `temper-17.s3`, session 17's, from `origin/main`
     before 17.4 lands (session 15's author waits on the designer,
     and these files are in no running session's WIP); its session file
     is `temper/17-one-process.md`'s 17.s3. 17.4's power-loss story then
     presses Adopt once it is enabled, its pending-at-cut assertion kept,
     with no second press standing in for the replay.

158. **Ruling 145 counts failures by symptom** (temper 15.3, blocked
     2026-10-14T20:40:44Z; process). After the designer's answer
     (`318a21ba`), 15.3's three process proofs each stopped at a
     different place, each after a change aimed at the cause the run
     before named: the navigation's context (39.8 s), a fresh sign-in
     after the restart answered `NotReady` (28.6 s), and the returning
     person's inbox expecting task 1's entry, which the script before the
     kill never made (60.6 s; `logs/temper-15/w77-third-result.md`).
     Ruling 145's "a third failure on the same symptom is a question"
     counts failures of one symptom: three symptoms, each named, are
     three counts of one. So 15.3 runs the process proof once more, with
     `found` checking the same task 2 entry with the saved token, and from
     now on runs it once after each change aimed at a newly named cause
     without asking; a symptom met a third time is still a question with
     the logs. Every assertion, expected literal and limit stays.

159. **17.4 draws its cut while the adoption is pending** (temper 17.4,
     blocked 2026-10-14T20:41:35Z; process and plan defect). 17.4's Scope
     cuts "before a drawn submission" with "an adoption pending among
     what it finds", and its reference test draws `before = 75 +
     draw.below(59)` of the service's submissions; the world counts every
     submission from startup (`World.cut`), so a drawn cut may fall
     before the native client's adoption is saved or after it is
     answered, and 17.4's pending-at-cut assertion fails without saying
     which (ruling 156's third run, `logs/temper-17/17.4-156-diagnostic-3.md`).
     So the world draws the cut among the submissions after the native
     client's durable save of the pending adoption and before the
     service's answer to it, both of which the world observes; the
     reference test's range `75 + draw.below(59)` becomes that window's
     length, every assertion and literal kept, and "before a drawn
     submission" still holds. One more diagnostic run is granted, typed and
     printed only on failure, at the pending-at-cut assertion as at the
     person's failure. After it, runs follow named changes as ruling 148
     says, counted by symptom as ruling 158 says. The adoption story's
     focused time (3.38 s at 17.4's WIP, 0.79 s at 17.3's measure) rose
     inside 17.4: before 17.4 lands, one run of that story prints the
     world's time per phase once, and the change aimed at the largest
     brings the world within ruling 152's 2.0 s.

160. **After a restart, a kept adoption is checked by its key and the
     engine's record, not the form's sentence** (temper 17.4, blocked
     2026-10-14T20:54:46Z as a design gap; the design answers it). Ruling
     159's run shows the original key saved, restored, sent again and
     answered once, one durable outcome, and no second press; the story
     then fails waiting for the outcome's sentence on the form after the
     restart. client.md, 3.3: "A kept adoption has no form to show it:
     its answer goes to no child"; 3.4: "a client closed on the adoption
     page opens at the inbox, and the form's values are not kept across
     a restart". The unblock of 2026-10-14T20:24:20Z said "keep the
     pending-at-cut assertion and its literal", meaning the assertion's
     own words; it kept no sentence after a restart. So the power-loss
     story's person waits for no form sentence after the restart; the
     world checks the original key answered once (`adoptions_answered`)
     and the engine's durable `Stored::Application`; the sentence is read
     only where the press is answered in the same life. The cost (159's
     phase times: formatting roots 1.36 s, searching them 0.98 s of the
     adoption story's 3.23 s): the referee's secrets check formats
     `Service::fmt_roots` at the end of each life, before each cut and at
     the story's end, not after every step, and checks each store record
     and agent environment as the world observes it, as before. 17.4's
     Scope is unchanged.

161. **16.6 traces its process proof's sign-in failure in its own 16.5
     support** (temper 16.6, blocked 2026-10-14T21:01:29Z; cross-session
     scope as asked, in fact session 16's own files). 16.6's one run of
     `scripts/proof-one-process.sh` (`logs/20261014T205900-0.log`)
     passed the landing story and failed
     `a_running_engine_lends_the_record_temper_sign_in_kept` on its world
     deadline after 30.1 s (`crates/temper/tests/support/sign_in.rs:271`:
     "stopped: false; empty hosts: [true, true, false, true, false,
     false, false]; issuer deadline: None"); the same test passed in
     0.279 s, second of five as here, in 15.3's landing run on
     `eb9ec31f` (`logs/20261014T204556-0.log`), the tree 16.6's WIP
     builds on, and the profile runs one test at a time. `sign_in.rs` is
     16.5's, so in 16.6's Files (ruling 108). So session 16 first runs that
     test alone once (`cargo nextest run -p temper --profile
     process -E 'test(a_running_engine_lends_the_record_temper_sign_in_kept)'`)
     on its WIP: a pass names an interaction with the run before it or
     with 16.6's diff, a failure names 16.6's diff (`git diff origin/main`).
     It fixes the named cause in its own files, then runs the whole proof
     once; later proofs follow each newly named cause, counted by symptom
     as ruling 158 says. A cause in another session's file, or in
     production code past the README's 20-line fix, is a question.
     At 2026-10-14T21:11:55Z the test passed alone (0.248 s); 16.6 named
     the fake LLM's listener not yet ready, waited for it, and the whole
     proof then passed the sign-in story.

162. **16.6's typist presses the frame's project for its context only**
     (temper 16.6, blocked 2026-10-14T21:11:55Z; plan defect). The
     terminal starts at the inbox with no project, and the navigation's
     adoption link shows only with a project chosen (client.md, 5.3:
     "with no project chosen, the slot holds nothing"); the inbox's
     project filter changes its rows, not the frame's project; jig's
     frame button for project 1 opens its overview. 16.6's Scope reads
     "The project's page its link opens is not waited for", its Test
     shape "The project's page is not opened": the page the native client
     cannot take (client.md, 5.4) must not be read or waited for. So the
     typist presses the frame's project 1 button for the context only,
     follows the navigation's adoption link at once without waiting for
     the overview, and goes on as 16.6 says: the form, Adopt, "Adopted
     acme/widget on forge 1 as owned.", `q`. If the refused overview
     keeps the navigation's link from showing, that is a question with
     the printed frames. The process proof then runs once.

163. **15.4's second live run, after a charter clarification** (temper
     15.4, blocked 2026-10-14T21:29:25Z; process, the README's "Live
     services"). The first live run (`logs/temper-15/w80-real-live-proof.log`;
     5 completions, 849 s, exit 100) failed on the model's words: the
     greeting chat answered, the parent delegated the coding task with
     the right specification, and the coding task called `delegate`
     again instead of reading, editing and finishing, then finished with
     `form: failure`, "Delegation request was rejected because additional
     authority must be proposed." (`w80-live-model-words.txt`); no
     authentication or code fault. The README: "each further live run, at
     most two, needs a supervisor's ruling, given only when the run failed
     on the model's words rather than on the code". So one more live run
     is granted, the plan's second of three (5 of 50 completions used),
     after these, in order:
     1. the process configuration's charter (`support::configuration`'s
        `engine.charters[0]`, a fixture) gains one sentence: a task whose
        contract is a change reads, edits and finishes with a change, and
        delegates nothing. Temper's charter rendering in production is not
        changed; a clarification that needs it is a question;
     2. the stories against skein's fake LLM peer pass once, as the README
        asks before a live run;
     3. `scripts/proof-live.sh` once, within 25 completions and 900 s.

     The model, the stories, the answer checks, the literals and the pins
     are unchanged. A third live run needs another ruling, on the same
     terms.

164. **"Done when" 12's dependency check reads as code-organization.md,
     3's rule** (the run's final check, blocked 2026-10-14T21:46:31Z;
     plan defect). Item 12 says `git diff cc2697cf origin/main --
     Cargo.lock | grep '^+name = '` "names only temper's own crates,
     crates of jig, smith and skein, and `serde_json` with its
     dependencies"; it names 16 more: `equivalent`, `hashbrown`,
     `indexmap`, `proc-macro2`, `quote`, `serde_derive`, `serde_spanned`,
     `subtle`, `syn`, `toml`, `toml_datetime`, `toml_parser`,
     `toml_writer`, `unicode-ident`, `winnow`, `zmij`
     (`logs/temper-17/done-report.md`). The rule the item checks is
     code-organization.md, 3: "No crate adds any other dependency from
     outside skein, smith and jig". No temper manifest declares any of the
     16 (`git grep` of `*Cargo.toml` prints nothing); the workspace's
     registry dependencies are `serde_json` and `sha2`, which is in
     `Cargo.lock` at `cc2697cf`; and `cargo tree --offline -i <crate>
     --depth 1` at `origin/main` names their dependents: `zmij`, serde_json;
     `serde_derive`, serde (with `proc-macro2`, `quote`, `syn`,
     `unicode-ident`); `subtle`, rustls (skein's TLS); `toml`, smith's
     `smith-local-shell` (with `indexmap`, `hashbrown`, `equivalent`,
     `serde_spanned`, `toml_datetime`, `toml_parser`, `toml_writer`,
     `winnow`). So item 12's third check passes as the rule reads: the new
     entries are temper's crates, crates of jig, smith and skein,
     `serde_json`, and the dependencies those crates bring at the pins.
     That the item's list named the pins' own dependencies too narrowly is
     a proposal for the retrospective.

## For the user

Long-term choices the user may veto; each is the default the plan runs
with.

- **The designer's defaults, taken:** repositories keyed by Forgejo's
  repository id; the first client is a terminal (`temper open`), the
  browser waits for jig's browser shell, and temper's web crates are
  deleted; temper keeps its own charter and tools translation
  (`temper-engine-smith`) instead of jig's; git credentials ride in
  git's environment (readable by an agent of the same user in
  standalone); configuration is one strict JSON file with temper's
  members under `application`; the live tier uses a throwaway local
  Forgejo, never the user's forge.
- **A failed preparation's words are not on the task's page yet**
  (decision 3): jig's core keeps no attempt's failure detail at temper's
  pin, so the root reports them as a fact until jig's core does. The
  alternative is a jig change, which this plan does not make.
- **The plan has 81 steps** (decision 4), above the retrospective's 60.
  The alternative was two plans run in turn (the domain, codecs and
  service first; the forge's worlds, the client's tiers and the
  one-process world second).
- **`serde_json` in tools** (decision 10): the transcript tools mask JSON
  with it, a crates.io dependency outside skein, smith and jig, in
  ordinary-Rust tools only, as jig's process tests already use it.
- **forgejo-runner v12.13.2** for the Actions facts (decision 12), not
  the newer v13 line.
- **The live tier's budget for the plan:** at most 3 runs and 50
  completions of the user's Codex account (README, "Live services").
- **No repin of jig,** although jig's `main` is ahead of temper's pin;
  a later plan repins once jig's test-budgets plan ends.
- **An adopted repository is not listed on the project's resources**
  (the designer's answer to temper 15.2, 2026-10-14). jig's core, at
  temper's pin and at jig's `main`, lists on a project's page only the
  roles a live task's resources narrowed, and never an owned one
  (`jig-domain-core/src/resources.rs:170-182` keeps narrowings, not
  adoptions), though jig's `domain/core.md`, 6.1 and
  `client/domain/pages.md`, 6.5 ask for every adopted resource. temper's
  client.md, 4.3 and 10 and domain/testing.md, 2.6 now say so, and its tests
  check the outcome's sentence and the engine's record of the adoption
  in place of the listing. The fix is jig's core's (keep each adoption's
  role, owned included, and list it), then a repin of jig in temper; the
  alternative, a temper-side list in jig's project document, puts
  temper's records in a document jig's core owns. jig's `STATUS.md`
  records the shortfall.
- **The engine's pages hold 2 rows and texts of 128 bytes** (the
  designer's answer to the shipped configuration's documents,
  2026-10-14). temper's native client composes no more at temper's pin
  of jig (jig-view's `Limits::compose`, ruling 122), and a page past
  what it takes is refused whole, so configuration.md, 7 has
  `engine.limits` override jig small's `documents.rows` (4) and
  `documents.text_bytes` (4,096) down to the client's: longer texts on a
  page are cut and say so, and lists page 2 rows at a time. Raising them
  needs jig-view to compose larger, a jig change and a repin.
- **The native client shows no page of the forge's work** (the
  designer's answer to temper 15.3, 2026-10-14). At temper's pin of jig,
  jig's core refuses a whole document when a part's path or a pattern
  has more segments than `documents.lines`, and the native client bounds
  those segments by its `rows`, 2, the most jig-view composes. temper's
  branch holds have three segments and its forge patterns five or more,
  so no page of a task that holds or is granted the forge's resources,
  nor of a project whose policy names them, reaches the terminal; the
  inbox, outcomes' sentences and the forge show the work (client.md,
  5.4). The process and live stories check that, and the forge's state.
  temper keeps its names: shortening them to two segments would change
  how holds, grants and configured rules name repositories, which people
  write by host, owner and name before adoption. The fix is jig's: its
  client link bounding a path's and a pattern's segments by a limit of
  their own (`jig-protocol-client-link/src/limits.rs` sets them from
  `rows`), and its core cutting a long path or pattern with a mark
  rather than refusing the page (`documents/task_tree.rs`, `path`), then
  a repin in a later plan. The designer's recommendation: make that jig
  change, and repin, before temper is used on a forge, since a person
  cannot open their own forge chats in the terminal until then.
