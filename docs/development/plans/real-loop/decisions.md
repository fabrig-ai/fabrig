# real-loop: decisions

The questions this plan raised and their answers first; then the
decisions this plan rests on, numbered; the supervisor's rulings during
the run follow them, from 101; the choices the user may veto close the
file. The review answered for the user (development.md, 2.7).

This plan is the second part of temper's tools plan
(`docs/development/plans/tools/`), whose decisions and answers it
builds on and does not repeat: below, "the tools plan's question N"
names them.

## Design questions

1. **An account first, with no tools to run it.** testing.md 2.4 ("An
   account first"), 2.6 (Credentials) and tools.md 10, 11.3 and 11.4 make
   every real-loop story, and the process tier's first act, add the
   deployment's key with `add-key` from a private file and wait for
   `wait accounts --until usable`. Those are area 5's tools, which
   `temper-client-tools` lacks and STATUS.md lists as not planned (LLM
   accounts' second part). Today the engine's credential is a
   `handed_in` file its configuration names
   (`llm.accounts[].source.handed_in.file`).
   Answer: until area 5's tools land, the real-loop world and the process
   tier hand the deployment's credential in as a `handed_in` file, as
   main does. The account-first step comes with area 5's tools: a
   `later.md` row, and the item under "Not planned". testing.md 2.4 and
   2.6 and tools.md 11.3 and 11.4 are reworded at the designer's next
   pass. Decided, 2026-10-19.

2. **The fake LLM's parts from the live-text and accounts plans.**
   testing.md 2.4 names skein's fake LLM with several credentials,
   failures per credential and observations of route, scheme and
   credential index (`fake-llm.md`, 2.2, 2.3 and 3), which those plans
   build.
   Answer: area 1's stories need one credential and whole scripted turns,
   both on skein's main, so the plan awaits no live-text or accounts
   step. The real loop's fake-LLM helper passes the peer's own types
   through, so that their parts are usable once they land, and a
   `later.md` row names the stories that will use them. Decided,
   2026-10-19.

3. **A signal to `temper ctl`.** tools.md 8.2 says "the tools catch
   none, so its exit status is the OS's", and the process tier cuts an
   act with SIGTERM and expects a status other than 0 (tools.md 11.4;
   the tools plan's question 25). skein's startup blocks termination
   signals and delivers them as `Shutdown { signal }` (skein's io.md, 7;
   shell.md, 6), so a skein host is never killed by one: what it does on
   `Shutdown` is its own.
   Answer: on `Shutdown`, the host ends the call at once: it sends nothing
   more, leaves DIRECTORY's kept requests as they are, prints nothing
   more, and exits with 128 + the signal's number (130 for SIGINT, 143
   for SIGTERM), as a shell shows a process the signal killed. tools.md
   8.2 is reworded at the designer's next pass, beside question 1's
   rewordings. Decided, 2026-10-19.

4. **The shared fake forge and the checked world.** In temper 13's
   forge stories, hosted fake git writes into skein's forge peer's shared
   `Forge` handle (skein's fake-forge.md, 5); memory git allocated there
   outlives git's exit, so skein-world's checked world (`Heap::release`)
   fails it, while tools.md 11.3 has the harness check memory ownership,
   outstanding operations and release.
   Answer: the forge stories run on skein-world's `World::new_controlled`,
   unchecked; every other real-loop story stays checked. tools.md 11.3
   says so at the designer's next pass. A `later.md` row asks skein for a
   checked world that knows a shared handle's owner, and the plan's
   landing puts it in skein's `STATUS.md` under "Not planned", so that
   the forge stories become checked once it lands. Decided,
   2026-10-19.

5. **Three failures with no words** (tools.md 6.3): the link down past
   `DEADLINE`, a DIRECTORY never signed in, and an absent DIRECTORY given
   to a call other than `sign-in`. Drafted on: only `sign-in` creates
   DIRECTORY; the others exit 1 with `temper ctl: the engine at ADDRESS
   did not answer within 30 s`, `temper ctl: DIRECTORY is not signed in`
   and `temper ctl: DIRECTORY does not exist` (temper 05.1, 05.6).
   Answer: as drafted. Only `sign-in` creates DIRECTORY, so an act on a
   mistyped path never leaves a directory behind. Decided,
   2026-10-19.

6. **`pending`'s first-sent words** (tools.md 5.6, "wall time"). No tools
   carrier prints a wall time (`findings/code-checks.md`, 3). Drafted on:
   `first sent HH:MM UTC` through the tools' `document::utc`
   (`pub(super)`, visible to a sibling `render::pending`), the clock
   words the documents print; JSON keeps jig's integer (temper 05.5).
   Answer: as drafted. Decided, 2026-10-19.

### For the designer

What the answers above ask of temper's design. temper `a959f6e10` folded
in questions 1, 3, 4, 5 and 6 and adopt's two `kept:` forms
(`findings/code-checks.md`, 1). Three remain for the designer's next
pass:
- `domain/testing.md` 2.4: the real loop's paces gain `hinted`
  (decision 11).
- `client/tools.md` 11.3 still names "the fake forge's host", now
  skein's forge peer (skein 05).
- `client/tools.md` 5.6 and 8.1 say `sign-out` "erases the token"; jig's
  native shell leaves a zero-byte `token`, read as signed out
  (decision 14).

## The plan's own

1. **Skein's forge peer, hosted in the real loop.** forge-kit's skein 05
   has landed whole, the peer's webhook delivery (skein-05.5) included,
   and the tools plan's temper-repin pins it; forge-kit's temper-05.2,
   which puts fake git on the peer's shared handle and teaches the
   one-process world the port-0 hooks address, has not. The real loop's
   forge stories wait for it, and for skein-05.5 in name, so that the
   wait reads as the two steps it rests on. They host
   `skein_fake_peers::forge::Peer` themselves, and extend skein's fake
   rather than build one in temper. The tools plan's draft 07, a temper
   host in `testing/`, is dropped, as forge-kit's decision 26 says. The
   rest of this plan waits for no forge-kit step.

2. **The hooks listener's bound address is the world's to learn.** The
   world reads it from io's completion of the engine's port-0 listen, as
   forge-kit's temper 05.2 does in the one-process world, and hands it to
   the forge peer. The engine gains no accessor: draft 08.1 is dropped.

3. **No upstream change, no repin.** The plan changes temper alone. A
   need upstream that a session finds is a question (development.md,
   10.1), not an edit.

4. **Sessions keep their drafts' numbers:** temper 05, 06, 08, 09, 10 and
   13, so that the drafts' cross-references hold. 07 is not used.

5. **The meeting point comes first** (development.md, 5.3). The real
   loop's first story through `temper ctl` (temper 08.3), a sign-in and
   a chat to its reply, with `temper ctl`'s host hosted as a program per
   call, needs only what temper 05 lands first: DIRECTORY, the native
   transport, `sign-in FILE`, a read and the `chat` act. temper 05 lands
   those before the rest of its acts, its waits and `pending`, which then
   prove in the real loop as well as by units.

6. **The person on Forgejo's web is the box's.** tools.md 10 has the QA
   harness sign in on Forgejo's web as the person, change the password
   and make a read-only token; tools.md 11.4 does not ask it of the
   process tier, and skein's throwaway Forgejo has no web sign-in. The
   process tier's landing keeps the person its support makes today.
   Drafts 10.1 and 10.2 are dropped, with a `later.md` row for the QA
   loop.

7. **One landing in the process tier, today's.** testing.md 2.6's
   landing story (`crates/temper/tests/process.rs`, the person's landing
   through the native client) is the tier's one landing on the owned
   Forgejo, against real git and CI (tools.md 11.4). The tools' `adopt
   --landing` does not join it: serve's printed link is spent by its
   first sign-in, so `temper ctl` and the native person cannot share that
   run, and the native client would lose its only landing. The tools'
   landing is the real loop's (temper 13); the process tier's cut act
   (temper 10.2) stops at the adoption, with no CI.

8. **A story without the forge gives the engine an unserved forge.**
   temper's startup takes at least one forge, and testing.md 2.4 gives
   the forge peer only to the stories that need it. Until temper 13, a
   real-loop story's configuration names a loopback listener the world
   binds on port 0 and never serves; temper 13's forge stories name the
   forge peer instead. If the engine's start or drain waits on that
   forge, temper 08.2 raises it (its Questions) before any other choice.

9. **Another party's kept request waits for area 3.** tools.md 11.3 has
   a story in which a DIRECTORY signed in as another party leaves the
   first party's kept request unsent, and sends it once the first party
   signs in again. Signing a second party in through the tools takes area
   3's `sign-in --service` (tools.md 5.5, 9), and the engine's printed
   link is spent by its first sign-in, so area 1 has no way to make the
   second party in the real loop. temper 05 proves the line
   (`kept: N for party P, unsent while it is not signed in here`) and the
   exit by units on its own fixtures; the real-loop story comes with
   area 3's tools, a `later.md` row.

10. **A wait across an engine restart waits for the real loop's faults.**
    tools.md 5.4 has a running wait outlive the engine's restart. The
    real loop gives each start a kernel-assigned port (testing.md 2.4),
    so a restarted engine listens elsewhere and DIRECTORY's address names
    the old one; the restart story (temper 09.1) keeps the work, not a
    call. The real-loop story comes with the faults row of `later.md`,
    which drops and restores the loopback links.

11. **The stories set `hinted` short too.** testing.md 2.4 has the real
    loop's polling run "at paces the stories' configuration sets short
    (`poll`, `poll_max` and `slow`)". `forge.client.hinted_ms`, 1 s
    shipped, re-arms a read after each delivery, so a landing would wait
    on it past its 1 s aim. temper 13's stories set it short with the
    other three, through the profile's builder and `admit`; testing.md
    2.4 names it at the designer's next pass.


12. **The fuzzy walking story's cost, fixed at its cause (temper 14).**
    The tools plan's run missed one aim (its ruling 139): temper's fuzzy
    suite's median is 9.2 s against 8 s, and the suite's wall time is
    one test's, `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`
    (`tests/engine/domain`), 7.8 s alone, the median of three
    (`findings/fuzzy-walking.md`). At the request, one step,
    which needs nothing, profiles it and fixes the cause in the code or
    the test (`findings/fuzzy-walking.md`), never by a budget, fewer
    cases than its claim needs, or optimized builds (testing-strategy.md,
    8). The profile, run in planning, found the cause upstream: 84% of the
    story is the `Debug` walk of both worlds at each iteration, and a
    structural compare needs `PartialEq` or `Hash` on skein-lib's
    collections and jig's `Core`, the state digest of testing-strategy.md
    6 (`later.md`). temper 14 therefore shards the story into eight tests
    of eight seeds, each under the fuzzy ceiling, with the same seeds and
    the same compare at every iteration, and names the cause in its doc;
    the CPU it costs is unchanged. Asked of the review (its inbox's
    question 7, 2026-10-19 12:55Z). Answer: (c), 2026-10-19: temper
    14 shards the story, and each shard's doc names the cause (the `{:?}`
    of both worlds at every iteration, 493 MB of text, 84% of the time);
    the digest is its own small plan, next, right after this one, starting
    from `findings/fuzzy-walking.md` (`later.md`; For the user, K).

13. **The tools plan's retrospective, followed where it costs little.**
    Its adopted proposals 1, 2, 5 and 6 (temper e139ccaf4): the steps
    were split into sessions before any file was refreshed; each printed
    field, cursor, bound and act temper 05 adds is mapped to the type
    that carries it (`findings/code-checks.md`); the longest chain is
    projected at about 24 minutes a step; no new test is named with
    `_and_`. Where tools.md lags the code on the tools plan's decisions
    M to Q, the sessions follow the code and those decisions.

14. **Sign-out leaves a zero-byte `token`, read as signed out.** jig's
    native `Shell::keep(None)` writes it so; temper 05.5 reads a
    zero-byte `token` as none, and changes no jig code (3).

## Rulings

101. **The code graph for temper, re-indexed** (steer of 2026-10-19T13:51:25Z).
     The code graph indexes temper again: re-indexed at 2026-10-19T13:51:25Z, at temper `ed725ce57`, past the claim. The README's rule "**The code graph** is not used for temper until the supervisor has re-indexed it past its `origin/main` (AGENTS.md) and a steer says so. Until then, search with `rg` in the worktree." is removed; AGENTS.md's rule stands in its place: "Check each name the graph gives with `rg` in the worktree before relying on it." The graph indexes the primary checkout's `main`, never a session's worktree, and lags behind what lands; the hourly check re-indexes it.

102. **temper 05.1: DIRECTORY opened before its private open; a zero-byte token** (plan defect, temper 05's block of 2026-10-19T13:52:58Z).
     Plan defect: 05.1's Shape prescribes `OpenPrivate`, which makes an absent root, for every call. tools.md 6.3: "`temper ctl: DIRECTORY does not exist`, for any other call on a DIRECTORY that is not there"; decisions.md 5: "Only `sign-in` creates DIRECTORY, so an act on a mistyped path never leaves a directory behind."

     - Every call but `sign-in` first sends `file::Request::OpenDirectory { root: parent, name: leaf, no_follow: true }`, which opens and never makes. Its `NotFound` exits 1 with `temper ctl: DIRECTORY does not exist`, before any `OpenPrivate`. Its `Opened` token is closed (`Close`), then `OpenPrivate`, `Lock` and `Load` go on as the Shape says, keeping io's private-root checks. `sign-in` sends `OpenPrivate` alone, as now.
     - The window between the two opens, in which another process removes DIRECTORY and `OpenPrivate` makes it again empty, is accepted: that call then finds no `token` and exits 1, "DIRECTORY is not signed in". An existing-only private open in skein's io would close it: `later.md`'s new row, not planned. No upstream change and no pin moves ("temper alone").
     - Also from temper 116d0a645 (tools.md 8.1, after the plan's base): `sign-out` "leaves `token` a zero-byte file, which a call reads as no token". 05.1: a zero-byte `token` exits 1 as an absent one, `temper ctl: DIRECTORY is not signed in`. New 05.1 tests: `an_absent_directory_is_never_made` (no `MakeDirectory` or `OpenPrivate` submitted; exit 1) and `a_zero_byte_token_is_not_signed_in`.
     - 05.5: `temper-client-tools`' `src/jig/rows.rs` joins its Files: `SIGN_OUT`'s `line`, "End sign-in and erase the token", becomes "End sign-in and leave the token empty" (tools.md 5.6: "leaves `token` empty, zero bytes, read as signed out"), with `tests/golden/help.txt`, which repeats it; its test `sign_out_waits_for_the_token_erase` is named `sign_out_waits_for_the_empty_token_stored`. Refresh any further copy of the old line, and list it in the commit body.

103. **temper 06.1: the presentation allowance at the person's preset** (plan defect, temper 06's block of 2026-10-19T13:58:19Z).
     Plan defect: Choice 1 keeps "`worst_case` stays one function of the limits", and Choice 2 gives the agent "the tools' preset", but the Shape did not foresee that `view::Limits::of` refuses that preset, so `Temper::worst_case(&limits::client())` panics at `fixtures.rs:76` before any heap peak exists. The proposal is adopted.

     - `Temper::worst_case(limits)` is `domain::worst_case(limits)`, of the limits it takes, plus `2 × view::worst_case` computed wholly at `fixtures::domain_limits()`, the person's preset (its `phrasebooks`, `view_bounds` and limits). Every person world takes `domain_limits()`, so the person's value is unchanged; the agent's root needs no presentation, and this allowance is its upper bound.
     - `agent::new`'s `View::Agent { limits }` is `fixtures::view_bounds(&fixtures::domain_limits(), ..).tree`; its tree stays empty.
     - The agent's root, its `Joint` limits and its codec stay the tools' preset (Choice 2); the Don't's "the person's limits for the agent" means those. No tools-root `worst_case`, no upstream change.
     - If `domain::worst_case` refuses the tools' preset, or the heap check fails, raise it with the error or the peak (06's Questions). Refresh any further copy of the presentation allowance computed at the supplied limits, and list it in the commit body.

104. **temper 08.2: ServiceHost admitted with preparation's envelope** (plan defect, temper 08's block of 2026-10-19T14:01:41Z).
     Plan defect: the Choice admits `ServiceHost` before the caller and prepares it once the referee hands it the peer's address, but the checked world records a root's `worst_case` right after its spawn closure and collects its `operations` before the first turn, and no public builder gives the service's envelope before preparation (`startup_worst_case` is `pub(crate)`, `startup_operations` private). The envelope is preparation's own, taken before the address, since the documents' limits do not name it.

     - `crates/temper` gains `pub fn envelope_in(directory: &Path, config: &[u8]) -> Result<(u64, u32), String>`: `prepare` of `config` in `directory`, as `prepare_in` runs it, then the service's `worst_case()` and `operations()`, the service dropped and every descriptor `prepare` opened closed. Its unit test in `src/tests.rs`: equal to a prepared service's on the same documents, nothing left open. This is the one edit to `crates/temper` (the Rules' exception), beside 05.6's in the Hotspots.
     - `ServiceHost`'s spawn closure writes the documents with the held forge listener's address standing in for the peer's, takes `envelope_in`, and its `Host::worst_case` and `operations` return those two throughout. Once handed the peer's address, it writes the documents again and calls `prepare_in`, as the Shape says, and asserts the prepared service's `worst_case()` and `operations()` equal the admitted ones.
     - Nothing typed, no check switched off, no upstream edit, no copy of startup. If the two differ, if `prepare` refuses the stand-in address, or if dropping a prepared service leaves a descriptor open, raise it with the values or the descriptor. Refresh any further copy of the admission envelope, and list it in the commit body.

105. **temper 10's Shapes against main** (ready check of temper-10.1).
     Ready check of temper 10 against temper 60e76617d (none of 10.1's or 10.2's Files moved since the claim, ed725ce57). Its corrections:

     - 10.1's Shape: `Op::Wait { reap: false }`, then `Op::Wait { reap: true }`, then `Close` of the pidfd (skein-io `kernel::Op::Wait`, "Observes a child exit, or reaps it after observation"); `Spawn.root` the scratch's root, declared among `Calls`' descriptors, with `dir` empty, since `dir` is relative to `root`; `OUTPUT` is a `const` of `ctl.rs`.
     - 10.2's reference test: the kept line is `{"operation": "adopt", "outcome": adopted}`, the act's whole JSON (`crates/temper-client-tools/src/forge.rs`, `print`), not `adopted["outcome"]`, which is null.
     - 05.5's Files: the goldens and the help test are `crates/temper-client-tools/src/tests/golden/{help,help_pending}.txt` and `src/tests/help.rs`.

     The check compiled the corrected `ctl.rs` fixture and the five tests under clippy (clean) and ran `rdeps(temper)`: 59 passed. None of them ran against the shipped binary: `Start`, `Until` and `Cut` need temper 05. 10.1 and forge kit's 02.1 both edit `Process`, `ChatJudge::observe` and `process.rs`: the second to land rebases. Refresh any further copy of these lines, and list it in the commit body.

106. **temper 09's Shapes against main** (ready check of temper-09.1).
     Ready check of temper 09 against temper 28aef8f70 (08.1 landed as 2065dd47f; rulings 102 and 103 touch only 05 and 06): no name of 09 mismatches landed code. Its corrections:

     - 09.4's Shape: `Unread`'s `Event` match names every variant (`wildcard_enum_match_arm` is denied), and its `Debug` is written by hand, since `jig_client_shell_native::Shell` has none (`missing_debug_implementations` is denied); the key's name is `temper_client_tools::jig::key::Name`.
     - 09.4's Files: `temper-client-tools` comes with 09.2, `skein-io` and `skein-lib` with 08.2.
     - 09.2's Files gain `tests/real-loop/src/fixtures.rs`: D2's parent is `fixtures::write`'s, as D1's; if 08.2's `write` makes D1's alone, 09.2 makes D2's there.

     The check compiled 09.4's `unread::record()` and `Unread` sketch and 09.1's landed names under clippy (clean) and ran `rdeps(temper-real-loop-world)`: 3 passed. 09's stories cannot compile before 08.2 to 08.4 and temper 05. Refresh any further copy of these names, and list it in the commit body.

107. **temper 06.1: the tools root built unlinked** (plan defect, temper 06's block of 2026-10-19T14:05:56Z).
     Plan defect: 06.1's Shape builds the agent by `root::Root::new(&agent::limits(), seed)`, and Choice 1 steps `StartUnlinked { kept }`, then `SignIn { with: With::Transport }`; but `Root::new` is `with_client(Client::new(..))`, whose client holds a link, and jig-client refuses `StartUnlinked` on it ("Up and StartUnlinked require an unlinked client"). The proposal is adopted.

     - `crates/temper-client-tools/src/root.rs` gains `pub fn unlinked(limits: &Limits, seed: u64) -> Result<Self, jig_client::Unusable>`, `Root::new` on `Client::unlinked(&limits.domain.client, seed)`, with `new`'s `# Errors`. `with_client` stays `pub(crate)`; the root gains nothing else (06.1's Don't: no `worst_case`).
     - `agent::new` calls `Root::unlinked`. The tools' unit fixture (`src/tests/fixtures.rs`, `Root::with_client(Client::unlinked(..))`) may take it; refresh any further copy of that construction, and list it in the commit body.
     - No other session edits `root.rs`; no upstream change and no pin moves.

108. **temper 06.1: the agent's tree shows its link** (plan defect, temper 06's block of 2026-10-19T14:17:55Z).
     Plan defect, the proposal adopted. The Rules say "jig's check and Made once hold through `Verdict::Passed`", and jig's referee rearms `Online` for every client and meets it, and `Offline`, only from `Seen::Link`, which `Temper::seen` reads from a `Status` named `Live` or `Offline` in the region at `view::frame::link_region()`. Choice 1's empty tree could never meet it. No design text speaks of the agent's tree: it is the harness's.

     - **The agent's tree is the link alone.** `View::Agent { render, limits }`: `render` is `skein_ui::View::new(&limits)`, `limits` the person preset's tree bound (ruling 103). Its tree: `Page` > `Banner` > `Region` at `view::frame::link_region()` > one `Status` named `Live` for `Link::Live`, `Offline` for `Link::Offline`, none for the other states; then an empty `Main`, so that `skein_ui::outline::check` holds (a `Page` root and one `Main`). `build` rebuilds it when that state changes (`render.stale` of a count per state, `render.finish` emitting the patches), from `root.client().link().state`; `tree` is `render.tree()`. The two names are consts of `app.rs` that `Temper::seen` matches too. No bindings; `act` returns `None`; `take` does nothing.
     - **`Until::Steps(2)`, not `Steps(1)`,** in `fixtures::answered`, `fixtures::give` and 06.5's `fixtures::cut`: a `Steps(1)` run stops before world time moves (jig's `World::run`), so repeated runs never reach a callback. `ANSWER_STEPS` stays the loop's guard.
     - **Size:** ~285 lines of `tests/client/src`, the link tree's ~25 added.
     - If the link tree breaks the heap check, or `Online` is still unmet with it, raise it with the peak or the referee's line. Refresh any further copy of the empty agent tree or of `Steps(1)` in these fixtures, and list it in the commit body.

109. **temper 08.2: lands while the 4 s aggregate goes to the designer** (design gap, interim, temper 08's block of 2026-10-19T14:19:53Z).
     Interim, a design gap: the question has gone to the designer (`question.md`), whether testing.md 6.1's "the worlds below the domain … together at most 4 s of the default suite" binds this plan, now that `main` misses it before 08.2, and which steps aim at it.

     - **08.2 lands now.** Its story is 0.072 s (median of three), within its 1 s guide ("a one-process or real-loop story at most 1 s"), and the aggregate is missed without it (4.973 s of 5.059 s), so no answer would move or cut it. 08's Question ("raised … before any merge") is met by this raise.
     - **Every later step goes on** within its own guide and Proof bound, and gives its measures as its session file says. A step that would move a test to the fuzzy suite, or make an existing one cheaper, for the aggregate's sake, waits for the answer: that is the answer's to name.
     - Size: 08.2's 458 lines against ~300 land without asking (development.md, 8.6's rule: within its Files and ruling 104's edit).

110. **temper 06.1: a budget to find why Online is unmet** (plan defect, temper 06's block of 2026-10-19T14:25:21Z).
     Plan defect: ruling 108 told 06.1 to raise an unmet `Online` rather than find its cause, and the implementation rules allows a failing test "at most three times for it, or within the budget a step to find a cause names": 06.1's three runs went to the `Steps(1)` stall. The same block twice (`Online(ClientId(0))` unmet, 14:17:55Z and 14:25:21Z) becomes the README's rule "A world failure's cause, within a budget": eight more runs and 40 minutes per unnamed symptom, in its world diagnostics removed before the commit; a fix within the step's Files and rulings needs no question.

     - **06.1 finds why the agent's link is not shown `Live` by 0.5 s,** within that budget, from now. Where to look first, in this order: whether `root.client().link().state` ever leaves `SignedOut`/`Starting` after `StartUnlinked` and `SignIn { with: With::Transport }` (the person's arm also steps `domain::Event::Start` after them, the agent's `Event::Start` only those two); whether the agent's requests reach the `Joint` as the person's do (`duty` of its `Request::Client`, the front's bind for its `ClientId` and party); whether `build` emits patches when the state changes (the world shows a tree only then, after `take`, in each callback).
     - A fix within 06.1's Files and rulings 103, 107 and 108 lands without a question. Past the budget, or for a fix outside them (jig, the tools' crate beyond `Root::unlinked`), raise it with the cause found, or each run's finding.

111. **temper 08.3: sign-in reads FILE as given** (plan defect, temper 08's block of 2026-10-19T14:35:28Z).
     Plan defect: tools.md 8.1 says `sign-in` reads the link "from FILE", and a FILE given to a command is the path as given, absolute or relative to the working directory; 05.3's Shape read it y's `FileIo`, whose only root is DIRECTORY's parent, and the kernel resolves every path beneath its root, refusing an absolute one with `Escape`. The fix is session 05's, as the block proposes.

     - **A new step, `temper-05.s1`,** `temper/05.s1-file-root.md`: `arguments::file_root` names the directory of the FILE a call reads; the entry point opens it, as `Startup.file_parent`; the host adopts it as a second root of the directory's `FileIo` and loads FILE's leaf beneath it. It runs in session 05 after 05.4; its proof is a unit test, and an absolute FILE on the real kernel is 08.3's sign-in story.
     - **08.3 waits for it,** its Shape kept: `sign-in <scratch>/link`, absolute. Its world gives a call that names a FILE a second startup root, `file_parent`, from `arguments::file_root`; its README Needs and order name the step.
     - **05.6 reads `--call FILE` the same way,** beneath `file_parent`, and `ctl::run` opens it by `skein_shell::open_root`.
     - Any further place that builds `Startup` or loads a FILE argument beneath DIRECTORY's parent takes `file_parent` the same way: list it in the commit body.

112. **temper 08.2: the worlds' sum is the worlds' guide, and the one-process world's cost a step** (design answer, temper 08's block of 2026-10-19T14:19:53Z).
     Design answer, temper 2f97ab8bb (testing.md 6.1; the plan's README, Rules and Done when 5; 06, 08, 09 and 13's Questions; later.md; decisions.md, For the user L). Point 5's step is `temper-13.s1` (`temper/13.s1-one-process-cost.md`), its Files the README's Hotspots exception; it measures by instrumentation, since `perf` is not installed and `kernel.perf_event_paranoid` is 3.


     1. **`main` does not miss the 4 s.** 6.1's guides sit "within" the suites' rules, which are "wall time on the development machine, run idle" (testing-strategy.md 8). The 5.059 s was measured in the build lane, beside another task. I measured idle at `94df159d3` (08.2 landed), with four runs after one build. The runs took 4.189, 3.972, 3.845 and 3.908 s; the median of the last three is **3.91 s**. Per world: client 0.92–0.99 s (not 2.0), one-process 1.54–1.67, forge protocol 0.61–0.67, reconcile 0.26–0.30, terminal 0.16, git 0.15–0.18, real loop 0.09–0.10. No test is past its own guide.
        `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3 4; do cargo nextest run -p temper-client-shell-terminal -p temper-client-world -p temper-engine-protocol-forge-world -p temper-forge-setup-reconcile-world -p temper-one-process-world -p temper-real-loop-world -p temper-worker-protocol-git-world --profile measure -j 1 >below-$i.log 2>&1; done'`
     2. **testing.md 6.1 now says how the sum is measured and whose it is:**
        - "each test's own time, run alone on the idle development machine … with `--profile measure -j 1` the median of three runs. Beside another task, it measures that task."
        - "The sum is the worlds' guide, never a landing's. A test within its own guide lands. A plan that adds to these worlds gives the sum at its base and at its end, with the real loop's part apart. When the worlds together pass the sum, their slowest tests are profiled, then made cheaper at the cause the profile shows or moved to the fuzzy suite, by steps a plan names."
        - Reconcile's world joins the list (section 1 places it below the domain, and 08.2 counted it).
     3. **The sum binds this plan as a guide, and the plan's stories will pass it.** The headroom is 0.09 s, and the plan adds sixteen real-loop and fourteen client stories (README, Pulse). The real loop joined the 4 s on 2026-10-18 (`a1c9f50b8`), and the 4 s stayed. Counting the real loop apart would raise a guide, and "Raising a budget is an explicit decision, not a fix" (testing-strategy.md 8). So it is the user's: decisions.md, For the user L, now with the review. I recommend (b): the real loop is counted apart, its sum is each plan's Pulse aim as the user accepts it (for this plan, C), and the other worlds keep 4 s.
     4. **Until L is decided** (README, Rules, "The worlds' sum"; 06, 08, 09 and 13's Questions):
        - Each step lands within its own tests' guides.
        - No test is moved, cut or merged for the sum.
        - Done when 5 measures the sum at the end, with the real loop apart and the three slowest tests named.
        - later.md has the row.
        - Ruling 109's hold on "a step that would … make an existing one cheaper, for the aggregate's sake" is lifted for point 5.
     5. **The step that aims at the sum: a supplement step, which the supervisor names, before 13.4.**
        - Why the one-process world: it is 40% of the sum. `an_adopted_repository_gets_its_change_pushed_through_git` takes 0.87 s and two stories take 0.24 s each. On skein's simulator that time is CPU, so its cause is in the code or the test. Its 0.95 s came from larger chunks (eb8abb6aa: 3.3 s → 0.95 s), which cut iterations; no profile was ever run.
        - Why before 13.4: 13.4 and 13.5 drive the same adoption through fake git on the real loop ("near the real-loop story's 1 s aim", findings/base-times.md).
        - Scope: profile the three stories (frame pointers are on; `perf record -g` on the unoptimized test binary). Fix the cause in the code or the test, never in a build setting. Give each story's time before and after, idle.
        - Where to look first, from a reading (no profile yet):
          - `Outcome.iterations` and `procs.len()` per story: finished git, sh and agent processes stay in `procs`, and `Referee::observe` (`tests/one-process/src/referee.rs:310`) runs after each turn, O(processes).
          - The referee's per-iteration scans and per-commit `rows.clone_from` and `snapshot`.
          - Fake git's per-invocation `Checkout` rebuild and tree clones (`testing/temper-fake-git/src/answer.rs`, `program.rs`).
        - Its Files cross 08's "nothing of `tests/one-process`" and 13's "No edit to … `testing/temper-fake-git`". That exception is the supervisor's ruling.
        - No step for the client world: 0.95 s for 37 tests is about 26 ms each.

113. **temper 06.2 and 06.3: a chat's run parks for its reply, and LOAD_BYTES is public** (plan defect, temper 06's block of 2026-10-19T14:45:26Z).
     Plan defect: 06.2's Shape answered each run as `answer_runs` does, `End::Finished` with a report, which ends the chat and drops it from the party's reads, so its first `wait --until replied` finds no task and no message can follow. tools.md 5.4's `replied` is "a chat's run done for now", which jig's facts read as a parked run (`RunStage::Idle`). A park needs no turn and no codec, so the proposal's report-then-Parked mode, on 06.3's dependencies, is not needed.

     - **`Joint.park_runs`** (`tests/client/src/front.rs`), beside `answer_runs`: it answers each assignment as `answer_runs` does, but with `End::Parked` and `Parts::none()`. 06.2's tests set it in place of `answer_runs`; `answer_runs` and the stories that use it are unchanged. 06.2's Files gain `front.rs`, within its ~20 lines of `tests/client/src`.
     - **The close story** ends as the engine ends a closed chat. A further fixture answer it needs, within 06.2's Files, needs no question.
     - **08.3's two headings** are answered here: ruling 111 answered the first, but its `Answer to` line left out the heading's time, so the block stayed. Nothing changes for 08.3: it goes on as ruling 111 says.
     - **06.3's `LOAD_BYTES`** is reached as `temper_engine_domain::LOAD_BYTES`: it joins `crates/temper-engine-domain/src/lib.rs`'s `pub use limits::{..}`, beside `VIEWS` and `WATCHES_PER_STREAM`, its sibling bounds; 06.3's Files gain that line, and its sketch's two asserts use the public path. 06.3's Files already named `limits.rs` (`LOAD_BYTES`); no running or waiting step changes `lib.rs`, and no session file names the private path again. A further copy of `limits::LOAD_BYTES` outside the crate takes the public path; list it in the commit body.

114. **temper 06.4: the agent shows the held task it was offered** (plan defect, temper 06's block of 2026-10-19T14:51:28Z).
     Plan defect, the proposal adopted with its shape fixed. jig's referee checks each fresh release against a card the client shows: `visible.contains_key` and the revision shown, which a card without a printed revision takes from the documents delivered to that client. Choice 1 gave the agent the link alone (ruling 108), so no agent's release can pass. For the agent, the card it saw is the read the tools printed for it: `Temper::receipts` already records a `task D` read that holds a hold as `Receipt::Card` of `Escalation(D)` at the hold's revision.

     - **`Agent::build`'s `Main`** (`tests/client/src/agent.rs`) shows one keyed `Item` for each task D that a printed call's `acts:` line offers `release D` for: `Key { owner: frame::owner(), kind: 2, a: D, b: 0 }`, cards.rs's escalation, nested as the inbox nests its entries so that `outline::check` holds. It names no `Hold revision`: the referee compares the hold sent with the revision the agent's read delivered. `build` rebuilds when the link or the shown tasks change. Choice 1 says so; ruling 108's link `Status` is unchanged.
     - **`--key NAME`** is the agent's own key (tools.md: "The agent names the key"): the test's chosen key stands.
     - **06.4's Files** gain `tests/client/src/agent.rs`, and its Size is ~45 lines of `tests/client/src`. Its Don't: no revision printed in the agent's tree, and no `Sent` card or revision set apart for the agent; jig's check runs unchanged.

115. **temper 06.2: a run reads its messages in one turn before it parks** (plan defect, temper 06's block of 2026-10-19T15:01:58Z).
     Plan defect: ruling 113's premise that a park needs no turn holds only for an assignment with no messages. A message stays unread until a committed turn's `read` covers it, and jig refuses a park whose read passes the last committed turn's, so the run is reassigned or fails Invalid. A real run reads its messages in a turn; the fixture does the same.

     - **`park_runs`**: for an assignment that delivers messages (the `More` message, the close's), the Joint first commits one turn whose `read` is the assignment's last message, through `temper_engine_domain_world::protocol::turn` with an opaque body, as `fixtures::replied_chat` already commits one without a codec; then it parks as ruling 113 says. An assignment with no messages parks with no turn. No wait for 06.3: no codec is needed, since nothing decodes the body.
     - **Don't**, now: commit a turn other than the one that reads an assignment's messages, or decode its body.
     - **The `request limit` fix** in `crates/temper-client-tools/src/root.rs` is on `main` already: 05.4's 4445f2d9a adds `client.reclaim` after each routing drain, the same two lines. Rebase 06.2 onto `origin/main` and drop its own copy.

117. **One behaviour per test: act once, assert after it, bounds named** (steer of 2026-10-19T14:50:52Z).
     Steer of the check of 2026-10-19T14:50:52Z, point 1: the README's rule "One behaviour per test" now also says that a test sets up, acts once, then asserts that behaviour; that nothing is asserted on the way to the step under test; and that a bound has both sides, from named constants. Seen in 05.4's `crates/temper-client-shell-cli/src/tests.rs` (4445f2d9a): `two_writes_get_two_ordered_terminals` asserts `!f.act_sent()` before the step it waits for and then `f.store_count() >= first + 2`, a bound with one side computed by the test; `host_bounds_cover_its_holders` asserts `* 2 + 5` with both numbers unnamed. It applies to the tests each session adds or edits from its next increment. Landed tests are unchanged.

118. **08.3 waits on what frees an ended call's stream; its chat script keeps the chat open** (plan defect and design question, temper 08's block of 2026-10-19T15:07:13Z).
     An interim. What frees an ended one-shot call's stream is a design question, and it is now with the designer (question.md of 2026-10-19T15:11:41Z):
     - tools.md, 8.3, says that a read and a wait "hold their client's store in memory, empty, never DIRECTORY's file". So jig's `protocol/client.md`, section 4, "The predecessor's stream is sent gone", does not free their streams.
     - skein's `lib.md`, 7, lets a stream "still send to a peer that only half-closed". So the engine sees the connection's end only at a failed write.
     - What ends the stream, and whose code ends it, is the designer's to decide.

     Until the answer:
     - **08.3's chat story waits.** 08.3 keeps its WIP (d2aeab866) and its target. Nothing is landed for the 409s: no stream bound raised in the world's profile, no heartbeat shortened, no wait or retry in the caller, and no edit to skein, jig or `temper-service` for them.
     - **08.3's chat script stays as the session wrote it:** a turn of `Text(REPLY)`, then `wait({})`, as one-process's `read_then_wait_provider` answers it.
       - Why: the copied `provider` reports, which finishes the chat ("`Finished` ends the chat and drops it from the reads", tools.md, 5.4), so the read ended "watch ended".
       - 08.1's `chat_scripts` line now says so (ruling 118).
       - `tests/real-loop/tests/lifecycle.rs` and `tests/peers.rs` call `chat_scripts(1)`: run both with the changed script. Refresh any further user, and list it in the commit body.
     - **Meanwhile, the session may do work that does not rest on the answer:** the FILE root's story, the scaffolding, and those two suites.

119. **05.6 lands without a FILE named inside a --call call, pending the designer** (design question, temper 05's block of 2026-10-19T15:08:06Z).
     An interim. Whether a call that `--call` reads may name a FILE of its own, and from where it is read, is a design question. It is now with the designer, in question.md's second heading, of 2026-10-19T15:15:12Z:
     - tools.md, 8.1, gives "`--call FILE` reads one call as JSON" and "`sign-in FILE|-` … read from FILE or stdin", and says nothing of the two together.
     - tools.md, 5.6, limits only nested stdin: "A call takes at most one `-`, and none with `--call -`".

     Until the answer:
     - 05.6 goes on with all else of its Shape: waits, `help`, TEXT `-`, `--call` FILE or stdin decoded by `jig::json::call`, and `ctl::run` with `file_parent` by `arguments::file_root` of the argv's call. It may land with that.
     - A `--call` whose decoded call names a FILE gets no code path and no test of its own: no new `Startup` field, no decoding before the loop, no root opened in the host.
     - The answer's unblock says what the case does, and names the step that adds it.

120. **The coordinator checks no tests; the next check follows up the test rule** (process, the coordinator's block of 2026-10-19T15:16:33Z).
     The coordinator does not check any landing's tests. That sentence of the steer's point 1 is withdrawn: the coordinator reads no code. The supervisor's next hourly check verifies the rule against the tests that landed since (development.md, 10.3: "The supervisor follows up. The next check verifies each point."). The rest of point 1 stands: the rule passed on to the sessions that write tests.

121. **temper 06.2: an ended task reads as its historical task** (plan defect, temper 06's block of 2026-10-19T15:09:06Z).
     Plan defect, the proposal adopted. tools.md 5.4 answers it: "`ended` is met by a delivery that shows the subject ended, or by its watch's end because it ended. For `task N`, the answer is then the task as a read of it gives it once ended, so it says how it ended." jig gives an ended task's read as `ReadValue::Historical`, whose `OwnerHistoricalValue::Task { document, result }` holds the task's `contract::Task` and its `Result`; the tools read only `ReadValue::Task`, so the ended wait, like `task N` on an ended task, prints the tools' bug. 06.2 is the first step to read an ended task.

     - **`crates/temper-client-tools/src/jig/wait.rs`** (the finishing arm that reads `ReadValue::Task`) and **`jig/render.rs`** (`render::read`, and `Part::read`, which lists `Historical` as unsupported) take a `ReadValue::Historical` whose value is `OwnerHistoricalValue::Task { document, result }` and that `matches` the task read: the task `document`, printed as a `ReadValue::Task` is, and how it ended from `result`, through the printer's existing result and ending encoders (`render/document/outcomes.rs`, `endings.rs`). A historical of another kind stays the tools' bug.
     - **06.2's Files** gain those two files; its Size, ~30 lines by part 1's count. Its proof is the close story, `a_chat_the_agent_closes_ends`, met with `done: task T, close` and the ended task printed; a tools-crate test is not required.
     - **The close story** begins its wait before the fixture's `Finished` report, as the report says: a wait begun after it reads an ended chat's watch as unknown. That needs no question.
     - A further `ReadValue::Task` arm in the tools that an ended task reaches takes the same historical task; list each in the commit body.

122. **temper 06.5: the cut agent is reloaded before another directory asks again** (plan defect, temper 06's block of 2026-10-19T15:27:04Z).
     Plan defect, the proposal adopted. 06.5's third story takes the cut agent's process as ended, but jig's world ends no client: `reload` is its one way to end a client's process, and it starts the process again at once. The cut agent stays signed in, so once the engine is back the referee asks it to be online within `link_within`, and its first retry is at 1 s (`limits::client()`'s `backoff.first`). Raising either bound breaks the guide, and an upstream close is outside this plan.

     - **The third story,** `a_chat_cut_by_an_engine_restart_asked_again_from_another_directory_is_made_once`: `cut`, then `FrontFault::Restart { offline }` before any rerun, then `world.reload(agent)` of the cut agent, then a second `fixtures::agent` gives the same call: same T, `chats == 1`, and nothing asserted on the way (ruling 117). The reload resends the cut agent's kept chat under its key. The second agent has its own records, gives the same call under the same name and gets the same T, so the chat is still made once across directories. Its name stays.
     - **The runner's `Me` fix** (its `Me` read opened again after a `SignedOut` end, which a reload's `SignedIn` after `Start` needs) lands in 06.5's commit, named in its body. 06.5's Files gain it, and its Size is ~5 lines by part 1's count.
     - **Restart's re-hello of the hosted assignments** stays within `front.rs`, which 06.5's Files name. It needs no question.
     - The four stories that pass land with the fifth; there is no split.

123. **temper 08.3 and 05.6: a peer's end ends its call, and a --call call names no FILE** (design answer, temper 08's block of 2026-10-19T15:07:13Z and temper 05's of 2026-10-19T15:08:06Z).
     Design answer, landed in jig bfc3a82fc and temper 200916de5; the steps it asks for are temper-08.s1 (`08.s1-peer-end.md`) and temper-05.s2 (`05.s2-call-file-usage.md`).


     ### temper 08.3, what frees an ended one-shot call's stream

     (a), at the connection's owner: temper-service. Landed: jig bfc3a82fc
     (`protocol/client.md`, 17.4; STATUS.md), temper 200916de5
     (`client/README.md`, 8; `tools.md`, 8.2; 08-real-loop.md, 08.3;
     later.md; STATUS.md).
     - jig 17.4, "A peer's end is the call's end": "the application's
       server, which reads each connection's events below skein's server,
       ends a connection whose peer ends while a call it handed the engine
       end is unfinished, and tells the engine end the call's connection
       closed, as it does when a connection fails. The engine end takes that
       as the call's end, and a stream's as its connection's end,
       `StreamGone` (4.2)".
     - temper's `client/README.md`, 8: "When the peer ends a connection
       before its call is finished, it aborts the connection, as it does one
       that fails, and the call's owner hears the connection closed".
     - `tools.md`, 8.2: "a call holds one of the engine's streams, which its
       profile bounds …, only while its process runs, and calls that run one
       after another never wait on their predecessors' streams. A read or a
       wait sends no stream gone itself (8.3)."
     - The code: in `crates/temper-service/src/server.rs`'s
       `io::Event::Stream` arm, an `Up::End` on a connection whose `call` is
       `Some` goes to `http::up` as now, then `end(connection, false, out)`,
       the `io::Event::Failed` arm's abort. The `io::Event::Closed` arm then
       sends `Routed::Http { call, event: http::Event::Closed }`, from which
       jig 149970b's mount sends `StreamGone`. No skein or jig code, no pin.
     - The step: temper-service is outside 08.3's Files, and ruling 118's
       interim forbids editing it for the 409s; this answer lifts that for
       this change alone. A supervisor's step before 08.3 lands, about 15
       lines. Its Files: `server.rs` and one temper-service test file. Its
       test: on the small profile's two streams, three clients, one after
       another, each open the stream, read the hello and end their side; the
       third's stream opens with no virtual time past a heartbeat. Its proof:
       08.3's story as written, aiming at 1 s.
     - Unchanged from ruling 118: no bound raised (`CALLS_AT_ONCE`, the
       profile's streams), no heartbeat shortened, no wait or retry in the
       caller, nothing in skein or jig. 08.3's Don't now says so.
     - Deferred: jig-test-service's same gap. It is a row in later.md and
       jig's STATUS.md, not planned.

     ### temper 05.6, where a FILE named inside a `--call` call is read from

     (a). Landed in temper 200916de5 (`tools.md`, 8.1; 05-ctl.md, 05.6's
     Shape).
     - `tools.md`, 8.1: "**A call `--call` reads gives no argument of the
       type a file** (5.1), a path or `-`: one that does is a usage error,
       exit 1, once the call is decoded and before DIRECTORY opens. So
       `sign-in`, and each tool of areas 3 to 5 that reads or writes a FILE,
       is called from its arguments. A call opens a FILE only beneath a root
       its start opened before the loop, the directory of the FILE its
       arguments name, and `--call`'s call is decoded on the loop, from FILE
       or from stdin alike."
     - It extends 5.6's "A call takes at most one `-`, and none with `--call
       -`" to every file argument and both sources. Ruling 111's
       `Startup.file_parent` stands as it is.
     - The code: in `crates/temper-client-shell-cli/src/host/input.rs`'s
       `prepare_progress`, once `jig::json::call` returns `Ok(call)`, a call
       holding a `Kind::File` argument (its `Value::File`, or `Value::Stdin`
       in a file slot) fails as the decode's `Err` arm does, `usage: temper
       ctl DIRECTORY TOOL [ARGUMENT...]`, which `fail` exits 1, before
       `directory.start`. There is no new `Startup` field, no decoding
       before the loop, and no new root.
     - On main (d6c01573f), `sign-in FILE` under `--call PATH` reads FILE's
       last name from PATH's directory (`directory.rs`'s `load_link`). Under
       `--call -` it fails "link file has no parent". Neither is the design.
     - The step: 05.6 landed without the check (ruling 119). A supervisor's
       step after 05.6 adds it, with `a_call_that_names_a_file_is_usage`:
       `sign-in FILE` by `--call FILE` and by `--call -`, each exiting 1 and
       leaving no DIRECTORY. 05.6's Shape now names that step. Nothing is
       deferred, so there is no later.md row.

124. **temper 10.1 waits for 08.s1's freed streams** (plan defect, temper 10's block of 2026-10-19T15:30:28Z).
     Not a design gap: the design question has been answered (ruling 123). tools.md 8.2 now says the engine ends an ended call's stream "then, not at its next heartbeat. So a call holds one of the engine's streams, which its profile bounds …, only while its process runs, and calls that run one after another never wait on their predecessors' streams." The story's `sign-in` and `me` run before the wait, and the wait arms its timer only after about 20 s of connection deadlines: the predecessors' streams. temper-08.s1 is the change that frees them.

     - **10.1 waits for temper-08.s1** to land: its Needs gain it. 10.1 keeps its WIP commit and its four tests as written, then rebases onto `origin/main` and runs its Proof again. The ceiling stands.
     - **A fresh allowance of three diagnostic runs** begins after 08.s1 lands. If the wait still misses its ceiling then, the cause is a new one: report it as a block with the run's log.
     - **Until then,** nothing in 10.1 goes round the cause: no wait or retry in `support::ctl`, no bound or heartbeat changed, no test moved (ruling 118's Don't).

125. **08.s1 lands on its unit proof; 08.3 diagnoses the missing reply** (process, temper 08's block of 2026-10-19T15:44:01Z).
     Ruling 125: 08.s1 lands on its unit proof; the missing reply is
     08.3's, with a diagnosis budget.

     08.s1's Scope is "a call's stream is free once its process exits", and
     its test proves it. 08.3's proof run shows it too: by 1 s, sign-in and
     `chat` exited 0 and read attempt 23 was running, so the reads no longer
     wait behind two ended streams. A reply that the finished reads never
     show is another cause, inside 08.3's story, which 08.s1 does not own.

     - 08.s1's Proof is now "unit, its test, and the land gate; 08.3's story
       is 08.3's proof". It lands as it is (57334ab86, with the land gate).
     - 08.3 then goes on from its saved branch, rebased on that landing, and
       diagnoses why the finished reads never hold `scripts::REPLY`: at most
       3 world runs and 30 minutes. Each run captures each finished read's
       stdout (the last few is enough), what the scripted LLM peer received
       and answered, and the chat task's state in the engine: whether its
       run was assigned, how that run ended, and whether a turn was
       committed. Past the budget, ask, with what the runs showed. Remove the
       diagnostics before landing.
     - A cause found in another owner's files (temper-service beyond
       08.s1's arm, temper 05's client, jig, skein) is raised as a block
       before those files are changed. 08.3's Don't stands: no retry, no
       wait in the caller, no raised bound.
     - 10.1 needs only 08.s1 (its Needs): it may start once 08.s1 lands.

126. **Test-shape rules from the check of 15:46:32Z** (steer of 2026-10-19T15:46:32Z).
     Test-shape rules refined from the check of 2026-10-19T15:46:32Z
     (testing-strategy.md, 10). The README's "One behaviour per test" now says
     the expected value is a literal, a golden file or a named size, never
     computed by the code under test (05.6's `help_opens_nothing` builds its
     bytes with `help::text`; 06.2's attempt test asserts `first_attempt + 1`).
     "A setup written once, and rows carry data" now says: a helper two test
     files share is one helper of the crate's fixtures, found first with `rg
     'fn <name>' <crate>/tests <crate>/src/tests` (06.2 and 06.5 each define
     `call`); a story's parties, tasks, projects and tokens are named fixtures,
     never bare numbers; a table's body checks every row alike, and rows that
     expect different outcomes are tests of their own. Landed tests stay; the
     rules apply to tests added or edited from now.

127. **The real-loop world keeps its own name** (plan defect, temper 08's block of 2026-10-19T16:00:54Z).
     Ruling 127: `temper-real-loop-world` joins the naming check's allowed
     list, beside `temper-one-process-world`; 08.3 then lands.

     jig's design, README.md section 15, "What a client crate is", answers in
     its own words: "A crate that runs in the engine's process keeps the
     engine's name, even when it serves clients; a crate shared by both ends
     for another purpose keeps its own." `temper-real-loop-world` is "Temper's
     standalone service and hosted tools on one real kernel": it joins
     `temper-service` and `temper-client-shell-cli`, both ends, as
     `temper-one-process-world` does, so it keeps its own name. The check's
     `allowed` argument is how a caller says so.

     - 08.3's Files gain `tests/client/tests/names.rs`: its call becomes
       `breaches(&root, "temper", &["temper", "temper-one-process-world",
       "temper-real-loop-world"])`. The test's name and assertion stay.
     - No package is renamed.
     - Any further world of both ends that a later step adds joins the same
       list in that step, named in its commit body; a crate that exists for
       the client alone takes the `temper-client` prefix instead.
     - 08.3 lands past its Size without asking (the implementation rules, "Past the
       Size"); its report gives the count by file.

128. **Session 10's signal check counts matches** (plan defect, temper 10's block of 2026-10-19T16:06:45Z).
     Ruling 128: session 10's follow-up ed5325404 is not landed; the
     session's check counts each signal instead.

     The follow-up only splits `b"-STOP"` and `b"-CONT"` onto two lines so that
     10-process.md's third "Done when" command, `git grep -n -F -e '"-STOP"'
     -e '"-CONT"' … ctl.rs`, "prints two lines": on d26c5cfea both literals
     are on one line, and `-n` prints matching lines. The check meant both
     signals are there, and both are. The command becomes `git grep -o -n -F
     -e '"-STOP"' -e '"-CONT"' origin/main --
     crates/temper/tests/support/ctl.rs`, which prints one line per match:
     two on `main` as it is.

     - No step is added and no landing follows: 10.1 and 10.2 are landed, and
       session 10 is done, with no successor.
     - ed5325404 stays on `real-loop/temper-10-2-signals`, unlanded; its
       worktree and target go with the coordinator's cleanup of ended
       sessions.
     - A further check that counts lines where it means occurrences takes
       `-o` in the same way, named in the ruling that meets it.

129. **09.2 diagnoses the unanswered Me watch within ruling 110's budget** (process, temper 09's block of 2026-10-19T16:35:37Z).
     Ruling 129: 09.2 diagnoses the unanswered Me watch itself, under the
     README's rule "A world failure's cause, within a budget" (ruling 110).

     That rule reads: "A focused world test that fails with a symptom its
     three diagnostic runs did not name gets eight more runs and 40 minutes
     for that symptom, reproduced in its world diagnostics
     removed before the commit. A fix within the step's Files and rulings
     needs no question; past the budget, or for a fix outside them, raise it
     with the cause found, or each run's finding." The four runs so far
     narrowed the symptom (the new stream's seed is sent; no Me delivery and
     no watch acknowledgement follow) but did not name its cause. No other
     session owns the service now, and 09.2's world is the only reproduction.

     - Session 09 has eight more world runs and 40 minutes for this symptom.
       Its diagnostics may go into any of temper's crates (`temper-service`,
       `temper-client-shell-cli`, `temper-client-tools`), and are removed
       before any commit. jig and skein are read at temper's pins, never
       edited.
     - Keep the Choice's same-pass kill: the plan's order is what the story
       proves. A kill moved later is no fix.
     - A cause whose fix lies outside 09.2's Files (the service, the client
       shell, jig, skein) is raised as a block with the cause found, before
       any such file is changed; the ruling then adds the step that fixes it.
     - 09.4's independent work may go on while a diagnostic run waits in
       the queue, and never instead of the budget.

130. **Real-loop stories kept to smoke** (ruling of 2026-10-19).
     Reverses: ruling 129 (2026-10-19T16:37Z), its last point: 09.4 has no work left to go on with.
     Amends: `09-loop-stories.md` (intro, Read first, Choices, Rules, 09.2, 09.3, 09.4, the sketches, Done when); `13-landing.md` 13.4 and Done when; `code-organization.md`; README's hotspot row; `08-real-loop.md`, `06-client-stories.md`, `05-ctl.md` where they name 09.4.

     The user's rule: the real-loop stories "should follow the testing pyramid principles and they are mostly smoke, it's a way to slowly climb up confidence, but extensive behaviour and corner cases should always be testable and tested near where it belongs". The real loop keeps one end-to-end path per feature: S1 drain, S2 chat, S4 close and wait, S5 restart (landed); 09.2's lost answer made once; 09.3's two acts taking the lock in turn; 13.2's forge delivery; 13.4's adopt through ctl; 13.5's chat landing its change.

     Dropped, each step keeping its code and its lower tests (on temper 58a712d69):
     - 09.2 `a_killed_calls_request_is_answered_by_the_next_act_on_its_directory`, with `calls::{kept, kept_lines, nameless}` and `Arg::Kept`: `tests/client`'s `agent_recovery.rs` (`a_chat_cut_by_a_lost_answer_is_resent_with_its_kept_line`, `..._asked_again_under_its_name_is_made_once`).
     - 09.3 `calls_that_send_nothing_run_while_the_lock_is_held`: `temper-client-shell-cli`'s `calls_that_send_nothing_take_no_lock`.
     - 09.3 `a_killed_holders_lock_passes_to_the_next_act`, with `Step::Locked`: the kernel's `flock`; not folded into the two-acts story.
     - 09.4 whole, its two stories, `unread.rs`, `fixtures::UNREAD` and its `jig-client` lines: `temper-client-shell-cli`'s `pending_counts_unread_records_in_its_line`, `pending_lists_each_kept_request` and `a_dropped_record_prints_its_line`. The coordinator sets `temper-09.4` dropped.
     - 13.4 `adopt_of_an_unseen_repository_exits_2_in_its_words`: the tools' `each_forge_refusal_exits_2_in_its_words`, `tests/client`'s `adopting_a_repository_the_forge_lacks_says_not_found`, the engine domain's `an_adoption_of_a_repository_the_forge_lacks_is_refused_as_not_found`.
     A case found not yet proved lower is proved in the lower world that owns it, in the same step, never in the real loop.

     Kept: the landed `a_signed_in_directory_is_private_to_its_user` (`tests/real-loop/tests/tools.rs:30`): skein-io's `src/tests/private.rs` (skein 92e177c) proves the modes against scripted `Stat` records (`mode: 0o700`), so this story is the real kernel's only proof of them.

     Done when: 09's names sum to 3, its kill and lock lines to at least two; 13's names sum to 8. 09.2's diagnosis (ruling 129) goes on for the story kept. The designer amends testing.md 2.4 to match, as a design question.

131. **Owner work resumes behind a refused input** (cross-session scope, temper 09's block of 2026-10-19T16:45:40Z).
     Cross-session scope, as ruling 129 foresaw: the cause lies in
     temper-service, outside 09.2's Files, and the design already answers
     it: "Owner progress is a step of its own. `Resume` admits one"
     (`docs/design/domain/root.md`, 4). The host's `release` withholds that
     step while a refused input waits, so a refusal behind owner work never
     clears. Not a design gap: the fix makes the host follow root.md 4.

     Ruling 131 adds `temper-09.s1` (`temper/09.s1-resume-behind-refusal.md`),
     in session 09, before 09.2, which waits for it: `release` steps `Resume`
     while an input waits refused, and the input is retried once the work
     drains; its unit test in `crates/temper-service/src/tests/` is the
     lower-world proof. 09.2 then rebases onto it and runs its Proof again,
     the same-pass kill unchanged. Ruling 129's budget is spent on the cause;
     09.s1 gets none of its own: past its Size or a second failing gate, a
     block with what was seen.

132. **testing.md 2.4: the real loop is smoke** (designer's answer, the supervisor's question of 2026-10-19T16:43:09Z).
     The designer's answer, landed in temper 2f71813ad, decides the user's ruling into the design:

     Landed in temper `2f71813ad` ("The real loop is smoke; its corner cases
     are tested below"),. real-loop's run takes it; no step's
     Scope changes beyond ruling 130's.

     testing.md 2.4's intro now says: "It is smoke, the testing pyramid's top,
     a way to climb in confidence: one end-to-end path per feature, and each
     behaviour and corner case tested in the lower world that owns it, never
     here." "The stories" now read "one end-to-end path each", as ruling 130
     kept them. The LLM accounts' stories keep "one path per source": "a
     sign-in" and "a program's source". The refused and late sign-ins, the
     limited, refused and unauthorized turns, the fenced notice, and the
     program's limit and lost login move to a new bullet, "The accounts' cases
     are tested below":
     - a sign-in refused or past its deadline: `tests/client` (tools.md 11.2),
       skein's OAuth worlds (oauth.md 3, 6.8), jig's protocol world (llm.md 8);
     - a turn limited, refused or unauthorized: jig's protocol world ("failures
       of each class", "a source limited and resumed"), accounts world and
       core world (authority.md 8.3); `tests/client` for the limited notice,
       `task N` and `retry`;
     - a fenced attempt's notice dropped: jig's fleet world ("Attempts are
       fenced", engine.md 8);
     - a program's usage limit and lost login: smith's world over its fake
       `claude` (programs.md 8), and jig's channel world ("a usage limit
       relayed as limited … a login lost mid-run relayed as unauthorized").
     The closing paragraph becomes "What only the real kernel shows is this
     tier's", and adds testing-strategy.md 2.8's rule: a failure found here is
     rerun lower down.

     Also amended: tools.md 11.3's unread-record story moves to 11.1
     (`temper-client-shell-cli`'s tests), its "A limit" goes (11.2 has it),
     and its closing says smoke; STATUS.md's two LLM accounts' rows; the
     plan's README rule "The real loop is smoke" (a real-loop test outside
     2.4's stories and the steps' Scopes is a question, asked before it is
     written); 09 and 13 cite 2.4; 13's Read first leaves not found to the
     tools and `tests/client`; later.md's real-loop rows become one path per
     feature. The moved accounts cases stay "not planned" in their existing
     rows (STATUS.md's "The LLM accounts' tests"; later.md's area 5 row).

133. **Named positions, and the gate's check of them** (hourly check of 2026-10-19T16:52:04Z).
     Reverses: ruling 131 (2026-10-19T16:48Z), its instruction 1's "give it to session 09's author, who found the cause": development.md 7.3, step 4 holds.
     Amends: the README's rule "A setup written once, and rows carry data"; `temper/09-loop-stories.md`'s Steps; new `temper/09.s2-named-positions.md`; `temper/09.s1-resume-behind-refusal.md` line 3 (its number, 131).

     From the hourly check of 2026-10-19T16:52:04Z, bare numbers in a story's body, sent a second time (08.4's `tools.rs`, 8 hits at temper f4f5e43b3: `grep -rnE '\b(task|watching|join|number)\([0-9]+\)|calls\(\)\[[0-9]+\]' tests/real-loop/tests | wc -l` → 8), become, as development.md 14.4 says:
     - the README rule's text: a story's calls' positions are named consts as its parties, tasks, projects and tokens are, as `restart.rs`'s `CHAT`, `READ` and `CHATS`; no integer literal is the argument of `task(`, `watching(`, `join(` or `number(`, nor an index of `calls()`, in a test body;
     - a check in temper's gate, step `temper-09.s2` (`temper/09.s2-named-positions.md`), in session 09 after temper-09.s1: `scripts/check.sh` refuses such a line in `tests/real-loop/tests`, and the step names tools.rs's eight. The gate today is `scripts/check.sh`, cargo alone (18 lines at f4f5e43b3).
     09's and 13's sketches that type such positions (`task(3)`, `ran.number(3)`) are written with named consts when they are made. temper is the most downstream repository: no repin.

134. **Two calls' exits after one commit have no order** (plan defect, temper 09's block of 2026-10-19T17:02:57Z).
     Plan defect, 08.4's sketch: `assert!(wait.ended > close.ended, ..)`
     orders two processes' exits that follow one commit. The engine answers
     the close's call and delivers `ended` to the wait's watch from the same
     commit; which client process exits first is the kernel's, not the
     story's. The story's claim, "a wait satisfied by another client"
     (08.4), is causal: the wait ends only once the close has been asked.

     Ruling 134: the assertion becomes `wait.ended > close.spawned`, in
     08.4's sketch and in `tests/real-loop/tests/tools.rs`'s
     `another_clients_close_meets_a_running_wait`, and 09.s1 makes that
     one-line change with its fix, its Files gaining `tools.rs`. The exits
     assertion stays. Then 09.s1 runs its land gate again. A further
     assertion that orders two calls' `ended` after one commit is refreshed
     the same way, and listed in the commit body.

135. **temper 09.3: the lock ends at its Close** (design answer, temper 09's block of 2026-10-19T17:13:58Z).
     Ruling 135, the designer's answer, landed in temper 6a287983a ("End DIRECTORY's lock at its Close, before the call exits"); its decision follows.


     Yes. A call ends DIRECTORY's lock at the `Close` of the descriptor it locked, once its
     reads and writes of DIRECTORY have completed, before its output drains and it exits.
     temper 05's `OneShot` already does this and stays unchanged. tools.md 8.3 now says:
     > **The lock ends when the call is done with DIRECTORY,** before its process exits: once
     > the call has its status and its reads and writes of DIRECTORY have completed, a store
     > write in flight at a signal included, it closes the descriptor it locked, and the lock
     > ends with that `Close` (skein's `io.md`, section 5: "The lock lasts as long as the
     > root's descriptor: until its `Close`"). [...] A process that ends sooner, however it
     > ends, ends the lock with its descriptor.

     This replaces "for its run" and "The lock ends with the process, however it ends."
     09.3 now (`temper/09-loop-stories.md`, as landed):
     - `calls::Recorded` also notes the completed `Op::Close` of the descriptor it locked, as
       `Called.released: Option<Time>`. The sketch asserts
       `assert!(x <= b || y <= a, "both held DIRECTORY's lock at once");` on `one.released`
       and `two.released`, keeps both exit equalities, and Don't adds "assert the turns
       against `Called.ended`; change temper 05's `OneShot`."
     - A unit test `the_lock_outlasts_a_store_write_in_flight` goes in
       `crates/temper-client-shell-cli/src/tests.rs`. The `Disk` fixture notes
       `released_after: Option<u32>`, its `store_count()` at each `Op::Close` of `DIRECTORY`
       after a `Lock` it answered taken. The test:
       - holds the act's erase (`answer_new_act()`), signals `Terminate`, and asserts
         `released_after == None`;
       - then runs `store_erase()`, and asserts exit 143, a store write, and
         `released_after == Some(store_count())`.
     - Files add that test file and `src/tests/fixtures.rs`. Size adds at most ~20 lines
       there. Proof adds "unit for the `Close` after a store write in flight". Done when adds
       the test's `git grep -c`, which prints a count of 1.
     It rests on `host.rs` `link_progress`: `if self.status.is_some() &&
     self.signing.as_ref().is_some_and(Signing::is_empty) { self.directory.close(); }`, and
     on jig's `Files::close`, which only sets `closing`. Also amended: 05-ctl.md's "held to
     the end", and 09's 8.3 line. STATUS.md is unchanged, and there is no `later.md` row.

136. **For the user L in testing.md 6.1** (design answer, the user's L of temper 08e91e24a).
     Ruling 136, the designer's landing of the user's L (temper 08e91e24a) in temper bab11df83 ("Count the real loop apart in testing.md 6.1, with a standing 6 s"); its decision follows.


     Landed: temper `bab11df83` (testing.md 6.1; README, Rules and Done when
     5; the Pulse's real-loop bullet; 06, 08, 09, 13 and 13.s1; later.md's
     6.1 row). It takes L as decided: "the real loop is counted apart, with
     a standing ceiling of 6 s for its sum, in place of an aim per plan, and
     each story at most 1 s. The other worlds below the domain keep 4 s."

     - **testing.md 6.1** now reads: "the worlds below the domain other than
       the real loop (the protocol, machine, client, one-process and
       reconcile worlds) together at most 4 s of the default suite; the
       real-loop world's tests, its stories included, together at most 6 s of
       the default suite: a standing ceiling, the same for each plan, sized
       to the smoke that 2.4 keeps the real loop to …, which the user revisits
       when the sum reaches it". The fuzzy 25 s keeps the real loop in, and
       the 0.5 s and 1 s per test stay. Past either sum, the rule stands:
       "their slowest tests are profiled, then made cheaper at the cause the profile
       shows or moved to the fuzzy suite, by steps a plan names." Added: "A
       plan whose end finds the real loop's sum at 6 s or past it puts the
       ceiling to the user … until the user moves it, the 6 s stands."
     - **README, Rules, "The worlds' sums below the domain hold no step":**
       the six packages against 4 s, `temper-real-loop-world` against 6 s,
       each story at 1 s. "No test is moved, cut or merged for the sum until
       the user decides" is replaced by: "A test is made cheaper or moved to
       the fuzzy suite for a sum only by a step the supervisor names (6.1: "by
       steps a plan names"), such as temper-13.s1, never by a session's own
       move, cut or merge." The base names the real loop's part: 0.09 to
       0.10 s of the 3.91 s (ruling 112's answer, point 1).
     - **Done when 5, second part:** the medians of the last three runs
       against 4 s and 6 s; a story the third command names past 1 s in two
       of the three runs is past it. Each run's line fails, and says so, when
       it reads no test of either part. A miss is recorded for the user, and
       the run is done either way, as before.
     - **Unchanged:** the Pulse aims (`aims: temper 37 22 …`), as C set
       them; the bullet now says 6.1 guides the real loop's idle sum to 6 s.
     - **No step waits on this.** later.md's row keeps what is left: the
       other worlds' 4 s, if temper 06's client stories pass it.

137. **Counts and call bindings by named consts** (steer of 2026-10-19T17:57:46Z).
     Ruling 137, from the hourly check of 2026-10-19T17:57:46Z: the README's "A setup written once, and rows carry data" now says:
     - a size or a count an assertion reads (a call's `tries`, a number of chats) is a named const, with a line on what a smaller value would miss;
     - a story's calls are read by their named consts (`ran.calls()[AGAIN]`, as `tests/lock.rs` reads `ran.calls()[FIRST]`), never bound by a slice pattern over `calls()` with `_` for the calls it skips.

     Seen on origin/main: `tests/real-loop/tests/lost.rs:30: let [_, lost, signed, again, _, tasks] = ran.calls() else { panic!("six calls") };` and `lost.rs:36: assert_eq!(signed.tries, 2, …)`. The first slips past temper-09.s2's gate grep, which matches only `calls()[<integer>]`. A pattern that binds every call (`tools.rs:28`, `let [signed, chat, read]`) skips none, and stays.

     It applies from each session's next increment. Landed tests change only when a step touches them: refresh any further copy you meet in a file your step changes, and list it in the commit body. If the `_` form is seen once more, a step will add it to the gate.

138. **temper 13's Shapes against main** (ready check of temper-13.1).
     Ruling 138, from the ready check of temper-13.1 (on Codex, temper 584607692, skein c9b82bbd): 13-landing.md now says
     - under "What exists": the hint's alarm is skein's `skein-forge-client/src/keep/poll.rs` (`hint`); `temper-engine-domain-forge-client/src/keep.rs` is not on main;
     - 13.3's Files: `tests/real-loop/Cargo.toml` (`jig-protocol-config`, `jig-protocol-store`, `temper-protocol-store`, `temper-engine-domain`), which `admit` names once it moves from `crates/temper/tests/support/forgejo.rs` to `landing`;
     - 13.5's Shape: `calls::Called` derives `Debug`, which the story's `{:?}` of `ran.calls()` needs; the story names its positions `CHAT` and `TREE` (README, "A setup written once, and rows carry data"; `scripts/check.sh` refuses `task(2)`); and `scripts/check.sh`'s pattern of literal call positions names `delegate`, in 13.5's Files.
     A further copy of these (another `admit` user, another literal position) is refreshed the same way and listed in the commit body.

139. **temper 13.3's proof after the sign-in fix** (process, temper 13's block of 2026-10-20T11:29:24Z).
     Ruling 139: the process proof's failure, `native signing failed: Again` at `crates/temper/tests/support/person.rs` in `a_change_lands_by_deliveries_with_every_poll_raised` before any landing turn, has its cause fixed on `origin/main` after 13.3's branch was cut: e9bb42960 "Send a link sign-in answered Again after a backoff" (11:20:10Z, owner work) makes the native process person resend the same link after `Signing::Again`, at 100 ms doubling to 2 s within `START_WITHIN`; 13.3's branch starts at 787649ae4 (11:14:40Z) and lacks it. So no retry without a change: rebase 13.3 onto `origin/main`, which is that change aimed at the cause its log names (author 8.8), and run the full process proof once, as 13.3's Proof says. If it fails again on `Again`, or on anything else, block with both logs.

140. **13.5 diagnoses the landing's stall within ruling 110's budget** (process, temper 13's block of 2026-10-20T12:25:22Z).
     Ruling 140: 13.5 diagnoses the stall itself, under the README's rule "A world failure's cause, within a budget" (ruling 110), which answers the question: "A focused world test that fails with a symptom its three diagnostic runs did not name gets eight more runs and 40 minutes for that symptom, reproduced in its world diagnostics removed before the commit. A fix within the step's Files and rulings needs no question; past the budget, or for a fix outside them, raise it with the cause found, or each run's finding." 13-landing.md's "raise it before changing a fixture" is met: the stall was raised, and the fixture stays as it is until the cause is named.

     - The symptom: in `a_chat_lands_its_change_on_the_forge`, the fifth call, `task N --tree --json` for the chat's task, has no exit at the world's deadline, while the delegated task stays `Active(Due)` with its forge `PendingStep` at `Branch` (branch None, CI Unknown) and no coding-agent LLM request comes; adoption printed "Branch protection could not be read."
     - Session 13 has eight more world runs and 40 minutes for it. Its diagnostics may go into any of temper's crates and `tests/real-loop`, and are removed before any commit; skein and jig are read at temper's pins, never edited. Print the peer's observations before `Seen` consumes them, as the session's "keep the trace, the peer's observations and the last stdout" asks.
     - Name which of the three is the first cause: the tree read unanswered, the delegate held at `Branch`, or the protection read the fake forge does not answer; one run's finding for each is enough to order them.
     - A cause whose fix lies inside 13.5's Files (the world's `landing`, `forge`, `calls`) is fixed there and named in the commit body. One outside them (temper-client-tools, temper-service, temper-engine-*, temper-fake-git, skein's fake forge) is raised as a block with the cause found, before any such file is changed; the ruling then adds the step that fixes it.

142. **The git stories' shapes, and the gate's check of them** (steer of 2026-10-20T12:27:37Z).
     Amends: new `temper/13.s2-git-test-shapes.md`; `temper/13-landing.md`'s Steps; the README's order of steps.

     From the hourly check of 2026-10-20T12:27:37Z: "One behaviour per test" sent a third time for assertions on the way (ruling 117 at 14:50:52Z; the steer of 18:10:00Z, lost.rs), and the named sizes of ruling 137, are both seen again on origin/main in `tests/real-loop/tests/git.rs`:
     - `git.rs:55: assert_eq!((added.exit, tree.exit, result.exit), (0, 0, 0));` and `git.rs:78: assert_eq!((checked.exit, cleaned.exit), (0, 0));`;
     - `git.rs:91-92`: the commits' numbers `3` and `4`; `git.rs:101: assert_eq!(object.tree.len(), 1, …)`; `git.rs:35`: `body_bytes = 8192`;
     - `let commit = |number| invocation::object(&temper_fake_git::id::id(number))…` at `git.rs:41`, `:71` and `:86`.

     So, as development.md 14.4 says for a test-shape rule a script can test, sent a second time, step `temper-13.s2` (`temper/13.s2-git-test-shapes.md`), in session 13 after temper-13.5 (both change `scripts/check.sh`), fixes these and adds a check to temper's gate for the two forms a script finds: a tuple of exits asserted together, and a length asserted against a bare integer, in `tests/real-loop/tests` (`grep -rnE 'assert_eq!\(\([a-z_]+\.exit, [a-z_]+\.exit|\.len\(\), [0-9]+[,)]' tests/real-loop/tests` on origin/main: the three lines above). An assertion of a setup's exit alone, before the act, is no form a script can tell from the act's own; it stays the rule's, for the checks to read. temper is the most downstream repository: no repin.

143. **13.5's agent adapter takes the service's roots** (plan defect, temper 13's block of 2026-10-20T12:47:48Z).
     Ruling 143: the cause 13.5 found is a defect of the real-loop world's agent adapter, which 13.5's story is the first to meet (the first agent that reads a repository's checkout), so a plan defect: 13.5's Files miss `tests/real-loop/src/agent.rs`. The README's rule "A world failure's cause, within a budget" asks for a fix "within the step's Files and rulings": this ruling adds the file to them.

     - `agent::roots` hands every agent one descriptor, `<config's parent>/worker-root`, while the service names each Start directory `<worker root>/workspaces/<directory.root>/<directory.name>` (`temper-service/src/worker.rs`, `start_values`), and the shipped agent adopts supplied descriptors "in Start order" (smith 6747e66, `smith-agent-shell/src/agent_shell.rs`, `Resources::roots`). Without supplied roots it opens the host's path itself (`skein_shell::open_root`). So the world hands the agent a root above the checkout, and the coder's `README.md` is "path not found".
     - 13.5 fixes `agent::{agent, roots}` so that each agent adopts, in Start order, the directory the service names for it, as the shipped agent would open it, and names the fix and the other real-loop stories it touches in the commit body. `temper-service`, smith and skein stay unchanged; a fix that needs one of them, or `skein_world`, is raised as a block with the cause, before it is changed.
     - The fixes inside 13.5's Files made under ruling 140 (the tools' row bound, the delegate decoder, the builder window) stay, each named in the commit body with the run that found it.
     - The budget starts afresh for what comes after this fix: a further symptom gets the README rule's three diagnostic runs, then eight runs and 40 minutes, diagnostics removed before the commit. Then 13.5's Proof, clippy, the dependent tests and the three measurements, as its session file says.

145. **temper-13.s3 keeps a wake that overtakes a fresh-read chain** (plan defect, temper 13's block of 2026-10-20T13:14:27Z).
     Ruling 145: the cause 13.5 found lies in `temper-engine-domain-forge`, outside its Files, so, as ruling 140 says, this ruling adds the step that fixes it: `temper-13.s3`, "A wake that overtakes a fresh-read chain is kept" (`temper/13.s3-wake-across-fresh-chain.md`), in session 13, before 13.5 goes on.

     - The defect: `domain.rs`'s `step_change` returns with no trace when a step for the same task is pending, so a wake during a fresh-read chain (OpenPull's `Made` outcome, or its item's hint) is lost, and the chain, which read the pull `Missing` before the pull was opened, decides on `pull = None`. domain/forge.md, 5.3: "Before a decision depends on a fact, the change child domain asks for it afresh". The fix keeps one such wake and steps again from a fresh read once the chain has finished; it is proved by a unit test in the domain's `src/tests.rs` and its counterpart.
     - 13.5's fixes in its Files stay on its branch (the adapter of ruling 143, the title within the fixture's 16 bytes, the peer's clock skew, the domain's four calls and the client's matching connections, all through admitted overrides); each is named in its commit body, with the run that found it.
     - Once 13.s3 has landed, 13.5 rebases onto it and its budget starts afresh, as ruling 143's did: a further symptom gets the README rule's three diagnostic runs, then eight runs and 40 minutes. Then its Proof, clippy, the dependent tests and the three measurements.

146. **13.s3's tests in a module of their own** (steer of 2026-10-20T13:33:20Z).
     Reverses: nothing.
     Amends: temper/13.s3-wake-across-fresh-chain.md's Files.

     From the hourly check of 2026-10-20T13:33:20Z: 13.s3's commit cf16adacb grows `crates/temper-engine-domain-forge/src/tests.rs` from 785 lines on `main` (`git show origin/main:crates/temper-engine-domain-forge/src/tests.rs | wc -l`) to 828, past the README's `file-lines: 800`. 13.s3's Files gain `src/tests/change.rs`, a module beside `tests/adoption.rs`, `tests/fixtures.rs` and `tests/purpose.rs` (`git ls-tree --name-only origin/main crates/temper-engine-domain-forge/src/tests/`), for this step's tests; any further test the step adds goes there too. Its two tests also take testing-strategy.md, section 10's "A setup written once" (they repeat `fixtures::opening()`, `fixtures::missing_pull` and the Created answer) and "Named fixtures and sizes" (the expected `Wall::from_nanos(130_000_000_000)`), as the steer of this check says.

148. **An overtaken chain sends no decision** (plan defect, temper 13's block of 2026-10-20T14:17:29Z).
     Ruling 148: a plan defect in ruling 145's 13.s3 Shape, which let the overtaken chain decide before it stepped again. On `main` at 3d677cddc, `finish_step` calls `decide_step` first and only then, when `pending.woken`, `step_change`: the chain that read the pull Missing sends a Wait, the root's `change_decision` takes it as the due procedure step (jig's `procedure_due` is Some only while a step is Due, and the step's apply sets it Idle), and the fresh chain's Ready finds `procedure_due` None and returns at forge_route.rs's `let Some((connector, code, step)) = domain.core.tasks.procedure_due(task) else { return };`.

     The design answers which side is wrong. domain/forge.md 5.3: "Before a decision depends on a fact, the change child domain asks for it afresh"; domain/root.md 6: "The change procedure's decisions become the core's procedure steps", one per step ("a wait; ready steps it again"). A chain a wake overtook rests on a fact read before the wake: it decides nothing, and only the fresh chain after it does. The root stays as it is.

     New step `temper-13.s4`, "An overtaken chain sends no decision" (`temper/13.s4-overtaken-chain-sends-nothing.md`): in `crates/temper-engine-domain-forge/src/domain.rs`'s `finish_step`, a woken pending step calls no `decide_step` and starts the next step from Branch; the test `an_overtaking_open_outcome_restarts_the_fresh_chain` in `src/tests/change.rs` asserts no `ChangeDecision` from the overtaken chain and one Ready from the fresh one. Proof: unit; budget three runs and 20 minutes. 13.5 needs it, rebases onto it, and its budget starts afresh. The diagnosis also saw "frozen record count fits" and a 30 s retry, both during the stall the stale Wait made; if either remains after 13.s4, 13.5 reports it with its run, as ruling 140 says.

149. **13.5 diagnoses the review-stage stall within the README's budget** (process, temper 13's block of 2026-10-20T15:12:31Z).
     Ruling 149: 13.5 diagnoses the review-stage stall itself, under the README's rule "A world failure's cause, within a budget" (ruling 110), which answers the question in its own words: "A focused world test that fails with a symptom its three diagnostic runs did not name gets eight more runs and 40 minutes for that symptom, reproduced in its world diagnostics removed before the commit. A fix within the step's Files and rulings needs no question; past the budget, or for a fix outside them, raise it with the cause found, or each run's finding." Ruling 148's "13.5 reports it with its run" is met by this block; the symptom is a new one, after 13.s4, with the review rule that 13.5's Scope ("CI, review and land") and "What it waits on" ("three agent spawns (chat, coding, review)") call for.

     - The symptom: in `a_chat_lands_its_change_on_the_forge`, with the review rule in `landing::fixture`, the fake forge's observation ceiling panics, "frozen record count fits" (skein-fake-peers `forge/observe.rs`, `Peer::reserve`, at temper's skein pin 9ab88f69), at 9.697 s (logs/20261020T151008-163.log). That assert fires when the peer has recorded `face.limits.observations` requests: it marks a stall in which the client keeps reading the forge, not its cause. The ceiling stays as it is.
     - Session 13 has eight more world runs and 40 minutes for it, from now. Its diagnostics may go into any of temper's crates and `tests/real-loop`, and are removed before any commit; skein and jig are read at temper's pins, never edited. Print the peer's observations before the ceiling stops the run: what the client keeps reading names the stage it waits on.
     - Name which stage is pending first: the review delegate not started, its scripted turn unanswered, its review not reaching the forge, the gate not reading it, or the parent chat's reply wait; one run's finding for each is enough to order them.
     - A cause whose fix lies inside 13.5's Files is fixed there and named in the commit body. One outside them (temper-client-tools, temper-service, temper-engine-*, temper-fake-git, skein's fake forge) is raised as a block with the cause found, before any such file is changed, as ruling 140 says. Keep 13.5 within its Size, "at most ~120 lines of `tests/real-loop/src`"; past it, say by how much in the report.

150. **The Done when's misses, and the worlds' sums without reconcile's** (Done when, run of 2026-10-20T15:47:40Z).
     Reverses: nothing.
     Amends: the README's Rules, "The worlds' sums below the domain hold no step", and Done when 5's `P`: `temper-forge-setup-reconcile-world` leaves both; later.md gains a row for testing.md 6.1.

     The Done when run of 2026-10-20T15:47:40Z, at temper 9f444b30c (done.md, "Done when, run 2026-10-20T15:47:40Z"), passed items 1, 4 (24 passed, Summary 107.784 s) and 5's suites (focused 14.44 s and fuzzy 19.65 s medians against the Pulse's aims of 37 s and 22 s), and failed five commands. Four are misses whose intent holds on main, recorded here with their reasons; the fifth is fixed by this ruling and measured again.

     1. **Session 13, its second command:** `cue: Some(` and `@coding Small fix` print no line in tests/real-loop/src. The content cue is there as an assignment, `tests/real-loop/src/landing.rs:18: setup.checks.cue = Some(api::Cue { path: b"README.md"…`; 13.5 (fa7db32f6) renamed the coding brief to `cue: b"@coding Fix"` (landing.rs:176) "to keep the coding brief within the title bound", in its commit body. `git log -S'cue: Some(' origin/main -- tests/real-loop` lists no commit: the command's literal never matched a shape any increment wrote.
     2. **Session 14, its seventh and eighth commands:** each prints two lines, not one. The second `global-timeout = "60s"` (.config/nextest.toml:63) is `[profile.browser]`'s, from browser-1's temper-04.1 (00e77f2cd); `[profile.fuzzy]`'s (line 21) is unchanged. The second `opt-level = 3` (Cargo.toml:204) is `[profile.dev.package.skein-ui-bundle]`'s, from browser-1's temper-02.3 (7719ba16a). Neither is this plan's.
     3. **Item 3:** ten `_and_` names print, from 2a343d21e, 2a529b4f2, 3ceceb8ff and a438acc2c (accounts and program-login work, without this plan's `Plan:` trailer, none in its ledger), in directories this plan's diff shares. No test this plan's 36 ledger commits added is named with `_and_`.
     4. **Item 5's worlds below the domain could not run:** `error: package ID specification `temper-forge-setup-reconcile-world` did not match any packages` (logs/20261020T155618-77.log). forge-kit's temper-07.1, "Remove forge worlds now owned by skein" (4b074713d), removed it during this run; temper's testing.md table of tiers says "reconcile's is skein's (2.2)". The six worlds left are temper's: the sum of the other worlds is theirs, against 4 s, and Done when 5's measurement runs again with the amended `P`, its results appended to done.md. testing.md 6.1's own list still names "reconcile worlds": a later.md row asks temper's design owner to reword it.

     A further Done when command whose literal fails only on another plan's lines is recorded so, with the commit that wrote them, rather than amended after the run.

151. **The other worlds' sum is 0.04 s past 4 s** (Done when 5, run of 2026-10-20T16:03:56Z).
     Reverses: nothing.
     Amends: nothing.

     Done when 5's worlds below the domain, run with ruling 150's six packages at temper 8f2228ce8, four runs after one build (task 93, ran alone for 33 s, logs/20261020T160401-93.log), the last three counted: the other worlds 4.04, 4.09 and 4.02 s, median **4.04 s, past testing.md 6.1's 4 s by 0.04 s**; the real loop 3.23, 3.29 and 3.21 s, median 3.23 s, within its 6 s, so the ceiling does not go to the user; no real-loop story past its 1 s in any run. Each run passed 125 of 125 tests.

     The miss is recorded for the user, as the README's Rules ("The worlds' sums below the domain hold no step") and ruling 112 have it: each test within its own guide landed, and only a step the supervisor names makes a test cheaper for a sum. The three slowest of the other worlds' tests, by their median of the three runs:
     1. `temper-one-process-world::one_process an_adopted_repository_gets_its_change_pushed_through_git`, 0.899 s (0.902, 0.899, 0.888), which was 0.87 s at the plan's base (ruling 112's answer, point 1);
     2. `… a_person_signs_in_by_the_printed_link_and_a_chat_is_answered`, 0.264 s;
     3. `… a_signal_mid_run_drains_and_the_next_start_settles_the_claims`, 0.253 s.

     The adoption story alone is 22% of the sum. Work on it belongs to temper's owner, past this run (the message of 2026-10-20T10:03Z: new work goes to owners). The run is done either way (Done when 5).

152. **The supervisor's scratch worktrees are removed** (process, the cleanup's block of 2026-10-20T16:06:53Z).
     The supervisor's two ready-check scratch worktrees, wt/temper-09r and wt/temper-10r, held only the checks' sketches (09's at 28aef8f70, 10's at 60e76617d); the steps they checked landed long since (09.1 to 09.3, 10.1 and 10.2). The supervisor saved each diff to tmp/scratch-worktrees/ and removed both with `git worktree remove --force`. wt/ is empty: the release goes on.

## For the user

The choices below have long-term consequences; each is the planner's,
and the user, may veto it before the run:

- **A. The process tier's five proofs, the exceptions to the real loop**
  (temper 10; tools.md 11.4; design question 3). The real loop hosts
  `temper ctl`'s library as a program per call, never the executable,
  so five proofs stay on the process tier, each for what only the
  executable shows: `temper ctl help` (argv in, stdout out); a usage
  error's exit 1 (`main`'s conversion of the host's exit to the OS's
  status); `sign-in -`, then `me --json` (serve's printed link through
  process pipes into ctl's stdin); a wait's timeout, exit 3 with its
  digest (the OS's status beside the printed document); an adopt cut by
  `SIGTERM`, then made once (the OS's signal, the host's exit 143). The
  tier goes from 152.9 s (`findings/base-times.md`) to about 165 s for its 3 min aim
  (testing.md 2.6). No proof of this plan is on the live tier. A veto
  proves the five by units of `crates/temper` and leaves the executable
  to the tier's existing stories.
  Decision: accepted, 2026-10-19 (each proof is what only
  the executable shows).
- **B. The forge stories run unchecked** (temper 13; design question
  4). Hosted fake git writes into skein's forge peer's shared handle, so
  temper 13's stories run on skein-world's `World::new_controlled`:
  the binaries `tests/git.rs` (13.1's four hosted-git tests) and
  `tests/forge.rs` (13.2's delivery, 13.4's adoption and its refusal,
  13.5's landing), eight tests. Every other real-loop story is checked. They become
  checked once skein's checked world knows a shared handle's owner
  (`later.md`; skein's STATUS.md, "Not planned").
  Decision: accepted, 2026-10-19 (design question 4).
- **C. The real loop's cost** (testing.md 6.1). The plan adds sixteen
  real-loop stories to temper's focused suite, each aiming at 1 s,
  unoptimized; each increment that adds one runs it alone three times
  and gives the median in its commit body. A story over 1 s is made
  cheaper or moved to the fuzzy profile (`fuzzy_restart.rs`,
  `fuzzy_landing.rs`), never excepted. The plan's packages go from 9.90 s summed at the base
  to at most 37 s focused, the sum of the Proof bounds; about 20 s is
  expected (README, Pulse).
  Decision: accepted, 2026-10-19 (the testing vision: the
  real loop over all of temper; a story's cost is cut or moved, never
  excepted).
- **D. Skein's forge peer, awaited from forge-kit** (1). temper 13
  waits for forge-kit's temper-05.2 (skein-05.5 has landed), run on
  another machine; the rest of the plan waits for no other plan. A veto builds a
  forge host in temper (the tools plan's draft 07), a second fake forge
  beside skein's.
  Decision: accepted, 2026-10-19.
- **E. The person on Forgejo's web is the box's** (6). The process
  tier's landing keeps the person temper's support makes today; signing
  in on Forgejo's web, changing the password and making a read-only
  token come with the QA loop (`later.md`). A veto needs skein's
  throwaway Forgejo to sign in on the web (a session cookie, the form
  and its CSRF token: a skein plan and a repin) before temper 10.
  Decision: accepted, 2026-10-19 (the web sign-in comes
  with the QA loop).
- **F. One landing in the process tier** (7). The native client's
  landing stays the tier's one landing on real git and CI; the tools'
  landing is the real loop's (temper 13), and the tier's cut act stops
  at the adoption. A veto adds a second landing to the tier, about 86 s
  alone, past its 3 min aim.
  Decision: accepted, 2026-10-19.
- **G. Another party's kept request waits for area 3** (9).
  Decision: accepted, 2026-10-19.
- **H. A wait across an engine restart waits for the faults row** (10).
  Decision: accepted, 2026-10-19.
- **I. `hinted` set short in the stories** (11).
  Decision: accepted, 2026-10-19.
- **J. Sign-out's zero-byte `token`** (14). A veto makes sign-out erase
  the file, a change to jig's native shell and a repin.
  Decision: accepted, 2026-10-19.
- **K. The fuzzy walking story sharded until the state digest lands** (12):
  temper 14 spreads its cost over eight shards, each naming the cause,
  and the digest, the next plan, cures it.
  Decision: accepted, 2026-10-19 (decision 12).
- **L. How testing.md 6.1 counts the real loop** (the designer's answer
  of 2026-10-19). 6.1 counts the real-loop world among the worlds below
  the domain, together at most 4 s (the real loop joined that sum on
  2026-10-18, and the 4 s stayed); they take 3.91 s at the base,
  measured idle, and this plan adds
  sixteen real-loop and fourteen client stories (C). The choices: (a)
  keep the 4 s with the real loop in, and make cheaper or move to fuzzy
  what passes it; (b) count the real loop apart: the other worlds keep
  4 s, and the real loop's sum is each plan's Pulse aim, accepted by the
  user (for this plan C's 16 s, about 8 s expected), each story at most
  1 s; (c) no sum: the per-test guides and the 15 s rule alone. The
  designer recommends (b). Until it is decided: measured, never moved,
  cut or merged for the sum (README, Rules, "The worlds' sum").
  Decision: (b), by user, 2026-10-19, as amended : the
  real loop is counted apart, with a standing ceiling of 6 s for its
  sum, in place of an aim per plan, and each story at most 1 s. The
  other worlds below the domain keep 4 s. The ceiling is sized to smoke
  (ruling 130 keeps the real loop to smoke, the testing pyramid), and
  the user revisits it when it is reached. testing.md 6.1 takes it.
