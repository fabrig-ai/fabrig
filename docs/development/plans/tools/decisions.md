# tools: decisions

The questions this plan raised and their answers first; then the
decisions this plan rests on, numbered; the supervisor's rulings during
the run follow them, from 101; the choices the user may veto close the
file. The designer answered the design questions (rounds 1 and 2, and
questions 33 and 34); the user answered 18a and 20. The planner took the
rest, the most reversible option where the choice would have been the
user's (development.md, 10.4).

temper's `docs/design/client.md` is `docs/design/client/README.md`
since temper `6f342f624`, its sections numbered as before: below,
temper's "client.md N" is that file's section N; jig's `client.md` is
jig's `docs/design/client/domain/client.md`.

## Design questions

These were raised while planning and answered by the designer (rounds 1 and 2) or by the user (18a, 20).

1. **A given key in skein's request table.** lib.md 12 draws keys from the seed. Open: when a given
   key is admitted, how long it is held, a typed refusal apart from "limit", `restore` against a
   live key, and a stale `NotKept` retiring a reused name's new holder.
   Answer: skein lib.md 12: a given key is admitted while unheld and held until `reclaim`. `ask` and
   `restore` refuse `Full` or `Held`. Each admission is numbered and returned with its key, and
   `not_kept` names the key and the number, so a stale one is dropped. skein 52d6750.
2. **jig's named send.** client.md 4, 6, 20: a client-side name conflict is an `assert!` panic in
   `send`. Open: the same name and ask under another application key, the same name and another ask,
   a name held by a restored record, the outcome for "name held", and family asks. The planner
   recommended that the first case join.
   Answer: not a join. A `Send` may carry a name, which is the engine key (contract.md 3.1;
   client.md 4: `Send { key, name, request }`). When the table already holds the key (another
   application key, another ask, a restored record), the client answers a new `Outcome::Held` at
   once, keeping and sending nothing, until the holder's record is erased. The assert stays for the
   same application key with another request. Family asks are covered. jig f50c3f63.
3. **A rerun in a fresh process.** client/tools.md 7, 8.2, 13. Open: the kept request settling first, or
   still parked and colliding with the call, and a repeat past the retention.
   Answer: client/tools.md 7, 8.2: step 4 ends only once each kept request is answered and erased, so a
   call never meets its own name held, and a rerun under a kept name gets the engine's first answer;
   there is no "kept: pending". A link down past 30 s at step 3, 4 or 5 exits 1 and the records stay
   kept. Past the retention `me` gives, the name is free. temper a7a67b05b.
4. **The turn read.** contract.md 5.1, 6, 7.1; store.md 7.3, 8.3. Open: its row, subject and wire
   shape; its answers (decoded, undecodable, not found); a turn too large for one read, which lost
   its size; client/tools.md 5.2's "412 KB".
   Answer: jig contract.md 6: `turn{n,attempt,turn}` is a read, never a subject, with
   `activity{n}`'s access. Its answers are Unknown, Gone (past the transcript horizon), Busy,
   Malformed and `TooLarge{bytes}` (the engine's, past one read's bound; the client's, past
   `document_bytes`). Pages keep wire bytes: `TurnProblem::TooLarge{bytes}`. The 412 KB example
   needs a profile whose turn bound passes the page bound (client/tools.md 11.2). jig f50c3f63.
5. **jig STATUS.md on the one-shot mode.** It lists client.md 20 as "not planned" while client/tools.md 13
   treats it as designed.
   Answer: jig STATUS.md: the one-shot mode is designed and `temper ctl` is its first host, in this
   plan. The service mode and the one-shot client's story stay unplanned. jig f50c3f63.
6. **The repin takes smith's main.** Moving skein to main forces smith's main (transcript v5, jobs)
   on jig and temper. Open: how they follow `Outcome::{Running,Job}`,
   `CallProblem::{JobsFull,UnknownJob}`, `NotRun`, `Turn.jobs`, and temper's `Request::Job`.
   Answer: jobs never reach temper's engine; temper-engine-smith's `turn` answers non-turn requests
   `Problem::Type` and stays (temper agent.md 4.3). jig client.md 10 words jobs as call outcomes,
   "started as job N", never "is running". jig-protocol-transcript follows v5; `Turn.jobs` is not
   shown. Both STATUS files say this plan moves the pins. jig f50c3f63, temper a7a67b05b.
7. **`adopt` cannot build its record.** client.md 4.4, 4.6 and forge.md 4 need landing, prefix, CI
   or checks, and home; client/tools.md 5.3 has no flags.
   Answer: client/tools.md 5.3: `adopt --project P [--forge F] --owner O --name N --landing B`, then
   `[--role owned|fork|context] [--prefix X] [--no-home] [--no-ci] [--check G]...`, with the form's
   defaults. `--role context` turns home off; `--no-ci` without `--check` is a usage error. temper
   a7a67b05b.
8. **Marks.** client/tools.md 5.4. Open: which bytes are digested, which reads print a mark, and what a
   composite read digests.
   Answer: client/tools.md 5.4: "mark" is renamed DIGEST (`--since DIGEST`, a `digest:` line): SHA-256 over
   the subject's document as `--json` prints it, without the digest field, 16 hex. Only watch
   subjects print one: `me`; `inbox` and `tasks` without a cursor; `task N` (one per subject with
   `--tree` or `--run`); `run N`. `chat`, `activity`, `turn` and `--activity` lines print none. With
   `--since` and `--until`, the wait is met at the first delivery that differs and meets the
   condition. temper a7a67b05b.
9. **Waiting for a chat's reply.** No condition names it, a chat never ends, and the `phase:` words
   are unlisted.
   Answer: client/tools.md 5.4: `--until replied` (on `task N`, a chat) is met when the chat's status is
   parked, waiting for you, waiting for them, held, closing or ended. A chat wakes on its person's
   words (jig tasks.md 8.3), so a wait begun after `message` is not met by an earlier reply.
   `phase:` takes the client's 14 phases, lower-case and hyphenated. temper a7a67b05b.
10. **Provisional text.** Agent processes give none (llm.md 7), but 5.2's `chat` and `run` rows do
    not say so.
    Answer: client/tools.md 5.2: the rows give provisional text only where the run's host gives it; an
    agent process gives none and nothing is printed. temper a7a67b05b.
11. **The three turn bounds.** client/tools.md 5.2, 6.4, 6.3: the page bound, the client's limit and the
    tools' window. Open: their values, which one wins, and the exit status past the client's limit.
    Answer: the new client/tools.md section 12, "The bounds", names the page bound (engine), one read's
    bound (engine), the client's `document_bytes` and the tools' window: 50 rows; 32 KiB result
    text; 16 KiB turn part; 1 MiB document; 30 s link deadline; 60 s lock wait at 100 ms. A turn
    past the read bound exits 0 with its size and "past the limit; report it". The step test keeps
    each tools' ceiling within the client limit. temper a7a67b05b.
12. **Missing rows and formats.** Rows for `sign-in`, `sign-out`, `pending` and `help`; the JSON
    field names; the two `decide` forms in `--call`; the `kept:` line; TEXT from stdin.
    Answer: client/tools.md 5.6: `sign-in FILE|-` (the link is never an argument), `sign-out`, `pending`
    (reads the store file only: no lock, no client, nothing sent) and `help [TOOL]`. client/tools.md 6.2:
    JSON is one object in snake case; `help TOOL --json` prints the table's record; `decide` takes
    the proposal, or the task and hold, then outcome and reason; `kept:` lines are one JSON object
    per line on stderr. TEXT `-` reads stdin, once per call, not with `--call -`. temper a7a67b05b
    (29 and 33 changed `kept:` and `pending`).
13. **Open exit statuses.** An act whose link stays down, a client-side key conflict, a watch that
    ends mid-wait, and `adopt`'s refusals.
    Answer: client/tools.md 6.3: 1 is a link down past the deadline (record kept), `Held`, an address off
    loopback, a lock busy past 60 s, a watch ended by sign-out. 2 is an engine key conflict, or a
    wait's subject no longer visible. 3 is a timeout, or a watch ended with its condition unmet.
    `adopt`: adopted 0; unknown past the retention 1; every other 4.7 outcome 2. Questions 24 and 30
    amended two rows. temper a7a67b05b.
14. **The lock.** client/tools.md 8.3; client.md 11. Open: the mechanism, the wait and exit status under
    contention, and which tools take it. The planner recommended `Op::Lock` on the store.
    Answer: client/tools.md 8.3: skein's `Lock` on DIRECTORY itself, not the store (renamed on write),
    tried every 100 ms; past 60 s exit 1, "DIRECTORY is busy with another call". Acts, `sign-in` and
    `sign-out` take it; `pending`, `help`, reads and waits do not. Reads and waits use an empty
    in-memory store, never DIRECTORY's. temper a7a67b05b.
15. **The address.** client/tools.md 8.2 keeps an address file in DIRECTORY; client.md 6.1 passes it as an
    argument. Open: name, format, loopback check.
    Answer: client/tools.md 8.1: `address` is temper-client-shell-cli's: one line, `http://HOST:PORT`, mode
    0o600, written by `sign-in`. Every call checks 127.0.0.0/8 or [::1] before sending the token,
    else exit 1, "the engine's address must be on loopback". temper a7a67b05b.
16. **Which jig-client limits `temper ctl` sets.** client.md 5.4 against client/tools.md 6.4.
    Answer: client/tools.md 12: `ctl` has its own `Limits` preset in temper-client-tools' limits, not the
    terminal's: rows 50, document 1 MiB, segments 9, authority 12, the rest as the terminal's.
    temper a7a67b05b.
17. **The agent's credentials.** How `ctl` signs in on the box (link and password, setup.md 6.5, or
    a script token, setup.md 16), and the forge token the agent needs to read the landed change.
    Answer: `ctl` signs in by the box's printed link on stdin (`sign-in -`). The harness plays the
    person (setup.md 6.5): it signs in on Forgejo's web, changes the password, makes the repository
    to adopt in an org, and makes a person token with read-only scopes (setup.md 16). The agent
    reads the forge with that token, never `admin_token` (setup.md 9; domain/testing.md 2.6).
    `tests/support/owned.rs` still reads admin_token as the proofs' read handle: a plan that touches
    it moves it. temper a7a67b05b.
18. **The real loop to a landed change.** domain/testing.md 2.4, 10; client/tools.md 11.3. No tier lands a change
    through the HTTP fake forge, the real engine and worker, and an agent on a real kernel. Open:
    (a) the bridge, fake git on the shared in-memory forge or real bare repositories, (b) authority
    and merging if the latter, (c) CI, (d) timing, (e) the tools' root, (f) a lost answer on a real
    ring, (g) the chat's reply, (h) scope.
    Answer: (a) decided by the user: option 1, the fake git program on the shared
    in-memory fake forge, so (b) is moot. (c) CI from a content marker; the sh subset later. (d) The
    forge host delivers signed webhooks to the engine's listener, with a polling backstop at short
    paces; a story past the 1 s guide is made cheaper or moved to fuzzy, never excepted. (e) The
    tools' root is hosted over the native transport; the executable stays with 11.4. (f) Drop the
    hosted root once the engine holds the request, before the answer; a fresh root on another
    DIRECTORY, same party, reruns under the name and gets the first answer, one chat. (g) A hosted
    smith agent on the fake LLM's scripted turns. (h) In this plan: `tests/real-loop`, its fixtures,
    and the hosted service, worker, agent, fake LLM, forge host, fake git and tools' root; later:
    real git, sh CI, faults, several workers. The forge host moves to
    `testing/temper-fake-forge-host` (domain/testing.md 7). temper aeff31a (client/tools.md 11.3, 11.4; domain/testing.md
    2.4, 4.2, 9, 10).
19. **client/tools.md has no section 12.** Cosmetic.
    Answer: answered with question 11: client/tools.md section 12, "The bounds", now exists. temper
    a7a67b05b.
20. **jig's test builds at opt-level 2.** Can they take their dependencies at
    `[profile.dev.package."*"] opt-level = 2`, given testing-strategy.md 8? The planner recommended
    allowing it, with an amendment.
    Answer: first allowed (skein e9cc93b), then replaced by the user's ruling of 10-19: not allowed.
    skein b4c6241 reverts e9cc93b; testing-strategy.md 8 says "A slow test is understood, never
    hidden": test builds stay unoptimized, dependencies included; a slow test's cause is found with
    a profile and fixed; a per-crate override is for one hot leaf crate only, with a profile and
    sign-off cited beside it. jig 04 becomes a profiling pass, then restores 15 s.
21. **Digests of `inbox` and `tasks`.** client/tools.md 5.4: jig's watches deliver a document unlike the
    read's (no cursor), so a read's digest cannot equal the wait's and `--since DIGEST` breaks.
    Answer: accepted. client/tools.md 5.4: a read and a wait digest one form, the watch's document: for
    `inbox` and `tasks`, the rows of the first window, in its order, plus whether more follow,
    within the tools' page of rows. The read prints its cursor outside the digest. A subject takes
    only what its watch takes (`inbox` with no flag; `tasks` without `--after` or `--ended`); else a
    usage error, exit 1. temper 4e14c2b5.
22. **`phase:` conditions never met.** client/tools.md 5.4: the engine ends a task watch when the task
    ends, and a chip's phase is never done, failed or cancelled.
    Answer: accepted. client/tools.md 5.4: `phase:ended|done|failed|cancelled` are usage errors, exit 1,
    naming `--until ended`. `ended` is met by a delivery showing the subject ended, or by its
    watch's end; for `task N` the answer is the task as a read gives it once ended, so it says how
    it ended. temper 4e14c2b5.
23. **`tree N --until waiting-on-me`.** client/tools.md 5.4: tree nodes carry no "waits on". The planner
    recommended reading task N's own watch, a second watch kept beside the tree's.
    Answer: the designer differs from the planner. `tree N` takes `ended` only; `waiting-on-me` is
    dropped there, with no second watch. A task's waits-on is computed over its tree (jig
    domain/tasks.md 3), so `wait task N --until waiting-on-me` covers the tree. client/tools.md 5.4. temper
    4e14c2b5.
24. **Exit 1 for "watch ended by the engine's stop".** client/tools.md 6.3: jig reopens a watch the engine
    dropped, so no `Ended` arrives.
    Answer: accepted. An engine that stops ends no watch: jig's client reopens what it dropped
    (client.md 8.1). The wait goes on and fails as a link down past its deadline, exit 1; "the
    engine's stop" leaves the exit table. client/tools.md 5.4, 6.3. temper 4e14c2b5.
25. **A call cut by a signal.** client/tools.md 6.3 gives no status.
    Answer: accepted. The tools catch no signal: a cut call's status is the OS's, never 0, and a
    rerun under its name gets its first answer. The process tier asserts "not 0", then makes the
    rerun once. client/tools.md 6.3, 8.2, 11.4. temper 4e14c2b5.
26. **The runner's asker.** jig client.md 4.6 gives the tools' runner none, and temper root.md 15
    reads "one asker per owner, all in one enum, never reused". The planner recommended one asker
    for the runner, each call's number in the application key's high bits.
    Answer: accepted, except on the owner, where the designer differs in part from the planner
    ("Owner gains no variant" in the draft): temper's `Owner` gains `Runner` = 3 (jig client.md
    4.6). The call's number goes in the high 32 bits of each key's and handle's number; calls are
    numbered from one past the highest among the keys kept at `Start`. client/tools.md 4. temper 4e14c2b5.
27. **A named rerun and the wall clock.** client/tools.md 7: `shapes::chat` takes the clock, so a rerun in
    a fresh process builds other bytes and jig answers a key conflict, not the first answer.
    Answer: accepted, in jig's domain (jig 62bcf71e, design only; its STATUS rows say "the
    downstream repository's tools plan"). A deadline given as a span means "this long after the task
    is made": the engine makes it a wall from the wall time of the decision that makes the task (a
    `Create`'s admission, or the acceptance of the proposal that asks for it); a proposal keeps the
    span until accepted (jig contract.md 3.1, 3.2, 3.6; client.md 19, where `shapes::chat` takes no
    clock; authority.md 7; tasks.md 4; engine.md 4). temper client/tools.md 7: a named request is built
    from the call and `me` alone; a rerun after `me` changed what it is built from is another ask, a
    key conflict. temper 4e14c2b5.
28. **`close` takes no REASON.** client/tools.md 5.3, but `shapes::close_chat` needs one.
    Answer: accepted. client/tools.md 5.3: `close N [--move D]... [--reason TEXT]`, the reason empty unless
    given, as jig's reference UI sends it. temper 4e14c2b5.
29. **What a `kept:` line names.** client/tools.md 6.2: jig-client exposes no engine key. The planner
    recommended the name when the call has one, and the application key always.
    Answer: the designer differs from the planner, and corrected a first answer (temper fe0ec316). A
    kept line prints no name and no key: a request's record keeps its engine key, for a named
    request the name's digest (first 16 bytes of SHA-256), never the name (jig protocol/client.md
    10; client.md 11). Text: `kept: ` + the operation (jig's, or the forge's stop or adopt) + the
    outcome as an act prints. JSON: `operation`, `outcome`, no `name`. The runner keeps no records
    of its own. client/tools.md 6.1, 6.2.
30. **A refusal by jig's client itself.** client/tools.md 6.3: Unconfirmed, a client-side key conflict, a
    limit. The planner recommended exit 2, as the engine's key conflict.
    Answer: the designer differs from the planner. Exit 2 is the engine's answer alone. The client's
    own answers exit 1: not kept, held, unknown past the retention, a read past its limits.
    Unconfirmed and the client-side key conflict never reach a call (the runner sends nothing before
    `me` is live; each call's keys are its own): either is the tools' bug, exit 1, to be reported as
    a finding (11.5). The plan's draft 03 ("a client limit exits 2") becomes exit 1, except the
    sized turn, which stays 0. client/tools.md 6.3. temper 4e14c2b5.
31. **`decide P/K`.** client/tools.md 5.3: jig's `ProposalKey.proposer` is `Task(u64)` or `Party(u64)`.
    Answer: accepted. `decide P/K`, P as `tN` or `pN`: `t41/3`, `p3/17`. 6.1's example reads
    `proposed: p3/17, to maintainers`. client/tools.md 5.3, 6.1. temper 4e14c2b5.
32. **The forge's words, and skein-ui in the crate list.** Cosmetic, client/tools.md 4.
    Answer: skein-ui is added to client/tools.md 4's crate list. The forge's words moving to
    temper-client-domain is temper 11; the design already says so. temper 4e14c2b5.
33. **What `pending` prints, and records the tools cannot read.** client/tools.md 5.6: with 29, a pending
    row has no key or name; and a kept record of another store version, or of a family or owner the
    tools do not declare, had no defined output.
    Answer: client/tools.md 5.6, 6.1, 6.2, 6.3, 7, 8.2, 11.3. A row prints the operation, `party N`, and
    the first sending or `not sent`; JSON `{"pending": [...], "unread": N}`, a row having
    `operation`, `party`, `first_sent`. A record the tools cannot read is counted, never listed
    (jig's client drops it at start and gives only `Dropped { count }`): `pending` prints after the
    rows "N kept records these tools cannot read; a call that sends drops them", exit 0; a file that
    is not a small store's exits 1. A sending call prints `dropped: N ...` on stderr; step 4 sends
    only `me`'s party's requests and says "kept: N for party P, unsent while it is not signed in
    here". 11.3 adds two real-loop checks. temper fe0ec316.
34. **A deadline span against an authority's wall bounds.** The engine's checks (a `Create`,
    `propose::allowed`, `goal_accept_allowed`, the acts a document offers) and the client's
    `shapes::set_goal`, which uses `now`.
    Answer: engine side, the planner's (a), stated generally: each check judges a span as its own
    decision's wall plus the span (admission, each re-decision of where a proposal waits,
    acceptance). A waiting proposal's span ages: a later acceptance beyond the accepter's authority
    is answered as any (authority.md 8.1), and past the ceiling it is refused. No new commit kind,
    no time-driven re-routing. Client side differs from all three options: `set_goal` loses `now`, a
    span is within both of a draft's bounds as `AsAllowed` is, and the engine judges it (contract.md
    5.3; client.md 19; authority.md 7). jig 6ad8bb40.

## The plan's own

1. **jig and temper move to smith's main as well as skein's.** Forced:
   smith `29fa97c` does not build on skein's main, and the first smith
   that does, `b415f737`, carries transcript v5 and jobs
   (`findings/base-repin.md`). This plan is "the plan that moves jig's
   smith" of jig's and temper's `STATUS.md`.
2. **The first pins are fixed commits:** skein `b4c6241` and smith
   `a277e922`, the mains at the plan's start (skein's later commits are
   the forge kit's design and STATUS marks), so jig 01 does not wait
   for skein 01. jig 02.1 moves jig's skein to skein 01's landing;
   temper 01.1 pins jig 01's landing; temper 03.1 moves temper's jig to
   the newest landing of jig 02, 03 and 05. The closing repins (17)
   take every repository to the mains the run ends with.
3. **The turn read moves jig's wire to version 9,** nothing translated
   (protocol/README.md 7, contract.md 8; the design names no number).
   jig 03.2, 03.3, 05.2 and 05.3 change version 9 in place; there is no
   version 10 in this plan.
4. **The named send changes no wire:** the name is the engine key the
   wire carries already. jig 02 changes no wire version.
5. **For the next plan:** `temper ctl`'s one-shot host answers the
   client's Save and Erase in memory for reads and waits (client/tools.md 8.3)
   and drops the loaded records; jig's native Shell is not changed (it
   has no memory store; client/tools.md 15 keeps the host temper's). This
   plan's library therefore leaves the store to its host: the root and
   the runner take the kept records they are given and touch no file.
6. **For the next plan:** temper-client-shell-cli's host is a skein host
   with no main of its own, and the real loop hosts it as a program per
   call, so that DIRECTORY's lock and files and a call's exit run on the
   real kernel (`findings/next/05-ctl.md`, `08-real-loop.md`).
7. **Proofs by tier.** The user's direction of 10-18 22Z puts area 1's
   stories in the real loop (domain/testing.md 2.4) wherever it can carry them.
   The real loop is the next plan's (16), so this plan proves the tools
   as a library, in `temper-client-tools`' unit tests, and jig's changes
   in jig's worlds and joint stories; the process tier carries only the
   repins' own checks (jig 01.3, temper 01.1). The tools' stories
   through a hosted call are the next plan's, in `tests/client` and
   `tests/real-loop` ("For the user", B).
8. **For the next plan:** the real loop needs no skein addition:
   `real::World` kills a hosted program (`Op::Signal{pidfd, Kill}`) and
   spawns it again after `Op::Wait`.
9. **For the next plan:** the fake forge's host moves to
   `testing/temper-fake-forge-host` in a session of its own, extending
   the existing fake forge (`ForgeHost`, temper-fake-forge-protocol's
   delivery) on a kernel-assigned port; no new fake.
10. **Forge- and LLM-facing work in increments of its own** (the user's
    addendum 2 of 10-18 22Z): temper 11 holds the forge's words and
    `adopt`; temper 03.1's forge arm is a repin's compile repair. The
    README's Hotspots list the files the forge kit and the LLM accounts
    work would also touch.
11. **jig 04, the test cost, is in this plan,** as the review asked
    (10-19, after the user's ruling on question 20): a profiling pass on
    jig's focused suite (`findings/jig-test-profile.md`). It starts at
    once, beside jig 01; each increment fixes one cause, in the code or
    the test, named in its commit; 04.4 reverts ruling 275's commit (the
    focused budget back to 15 s) once jig 01 has landed, so the time is
    measured at the pins the plan keeps, and the 15 s binds jig 02, 03
    and 05.
12. **Areas 2 to 4, the MCP endpoint and the tools in jig** (client/tools.md 14)
    are in `later.md`; the QA loop is not in this plan.
13. **A named send under the same application key with another name is
    a key conflict,** loud as today's (the assert kept and extended): one
    application key names one request (client.md 4); the design does not
    say. A named send before `me` is answered Unconfirmed first
    (client.md 11).
14. **jig 05, deadline spans, is in this plan:** jig `62bcf71e`'s design
    (a deadline given as a span, made a wall by the engine; shapes read
    no clock) names "the downstream repository's tools plan" in its
    STATUS rows, and client/tools.md 7's named requests need it (a name sent
    again makes the same ask; question 27). Question 34's answer fixes
    the engine's checks (the decision's wall, the span at each check).
15. **Sessions over section 5's 400 lines are split,** the new halves
    numbered after the last, with no renumbering: temper 11, the tools'
    forge parts and `adopt` (from 03); temper 12, help and a call's JSON
    (from 02); temper 13, the real loop's landing (from 09, drafted for
    the next plan).
16. **Area 1 runs as two consecutive plans** (the rust-d1,
    Proposal 1: a longest chain past about 20 increments, or more than
    about 80 steps, runs as consecutive plans). Area 1 whole had 68
    steps and a longest chain of about 31. This plan, the first, has
    38 steps and 4 closing repins, with a longest chain of 21 (jig 01,
    03, 05.2 to 05.5, temper 03, 04, temper-repin): section 13, and the
    tools as a library. The next plan starts from this one's `main`:
    `temper ctl`'s host (client/tools.md 5.5, 8), the client stories, the fake
    forge's host, the real-loop world and its stories, and the process
    tier; `later.md`'s first row names it, and its drafts are in
    `findings/next/`, with their numbers (temper 05 to 10, 13), as its
    planner's input. Section 13 first, as the brief asks, and 5.5 next:
    the split falls between them.
17. **Closing repins** (development.md 5.7): `smith-repin` once skein 01
    has landed; `jig-repin` once smith-repin and every jig step have;
    `hearth-repin` once jig-repin has and hearth has a checkout on the
    run's machine; `temper-repin` once hearth-repin and every temper step
    have. Neither smith nor hearth uses skein's request table, so their
    repins change pins and absorb only what skein's main changed.
18. **The Pulse's aims are this machine's:** the run's machine
    (debian-16gb-hel1-1) ran each suite three times at the plan's start
    (`findings/base-times.md`): jig's focused 5.23 s, fuzzy 6.72 s;
    temper's focused 2.74 s, fuzzy 7.04 s. The aims are jig 7 8 and
    temper 5 8: each suite's median plus what the run may add (the
    README's Pulse), rounded up to the second.

## Rulings


101. **jig 01.1's skein pin after smith's update** (plan defect, jig 01's block of 2026-10-19 03:31:33 UTC).
     Ruling 101: a plan defect. The Pins rule fixes the result, "prints two lines: `skein.git#b4c6241…`; `smith.git#a277e922…`" (jig/01-repin.md, Rules that bind it), and the Shape's order cannot reach it once skein's `main` has moved past `b4c6241`: smith's update moves the skein packages that only smith's crates use to skein's `main`. temper 01.1's Shape already says "If a source still holds two revisions, run its command once more." jig 01.1 does the same: after smith's command, run `cargo update -p skein-io --precise b4c6241aefea0e434b6e1128754168d02bae0706` once more, then check the Pins rule's two lines; a second revision still there after that is a question. The README's Pins rule now says it for every step: a source left with two revisions by a later command gets its own command once more, last.

102. **jig 05's Shapes against main** (ready check of jig-05.1).
     jig 05's ready check (skein 7ee1ac86, smith a277e922, jig dca2bbfb, temper 3119ae3a) found users its Files missed; jig/05-deadline-spans.md now names them. 05.1: `crates/jig-protocol-store/src/tests/{format,commits}.rs` join its Files, and its Test shape names the version cases that follow (want 4; the people case of `another_family_is_refused_naming_the_first_difference` finds 5, an arm per family; the other-version people row writes 5; `memory.rs` and `fuzzy_decode.rs` say core 3, tasks and people 4). 05.2: `tests/client/tests/stories.rs` and `tests/client-wire/tests/fuzzy_client_wire.rs`. 05.3: `acts/requests.rs` and `routing/people.rs` of jig-domain-core; its protocol sketch takes `.expect("first task")` (clippy denies `unwrap_used`), and its core sketch's `use` is in rustfmt's form. 05.4 and the Other jig users: the client world's `src/{stories,stories/goals,harness/referee/offers}.rs` and the ops world's `src/client/stories.rs` and `tests/client_stories/creating.rs`, which call `shapes::{chat,set_goal,fall}`. With 05.1's corrections, the store suites passed, 310 of 310 and 14 of 14 under fuzzy, and the workspace's 4272. A further user found: change it as its increment's Shape says, and list it in the commit body.

103. **temper 01's Shapes against main** (ready check of temper-01.1).
     temper 01's ready check (skein b3146c74, smith a277e922, jig b2aded80, temper 8f09dc12) compiled and ran only the crates that do not depend on jig: 120 passed, 3 under fuzzy, clippy clean, with T1 to T6 applied. temper/01-repin.md now says: `smith-transcript` is a dependency of `tests/client`, not a dev-dependency, and `tests/client/Cargo.toml` and `Cargo.lock` join T6's Files; `Limits::new(bytes, turn)` is on jig's `main` already (`pub const fn new(bytes: u32, turn: TurnLimits)`), so Shape 6 is Choices' form, and a JIG01 without it is a question; `turn` joins the `use crate::{…}` of `crates/temper-engine-smith/src/tests.rs`; step 7's memory, if reached, changes `docs/examples/owned/agent.json`'s `"memory"` (2147483648) too, which the owned deployment tests load through `SHIPPED`. The check's worry that temper's jig pin trails jig's `main` does not hold: `git diff --name-only bbea9408 origin/main` in jig, outside `docs/`, names only `crates/jig-client-view/src/{owner_tree,sections/project/owner,sizes/pages}.rs`, jig 04.1's. The jig-dependent crates are checked again when 01.1 starts, against JIG01. A further copy of a changed value: change it as its step says, and list it in the commit body.

104. **jig 02's Shapes against main** (ready check of jig-02.1).
     jig 02's ready check (skein b3146c7 with 7ee1ac86 pinned and a local `ask_given` stand-in, smith a277e922, jig b2aded80, temper 8f09dc122) built jig-client-domain with 02.1 and 02.2 applied: clippy and fmt clean, nextest 44/44 after 02.1 and 53/53 after 02.2. jig/02-named-send.md now says: 02.1's `restore_record` takes the admission as `self.table.restore(..).ok()`, as clippy's `manual_ok_err` wants, the exhaustive-match rule covering a `match` only; 02.2's assert is `held.request == request && named`, with `let named = match name { None => true, Some(name) => name == held.admission.key };`, since jig's clippy.toml disallows `Option::is_none_or` ("no closures: match"); `a_name_is_admitted_again_once_its_holder_is_answered` drains `c.downs()` as well as `c.told()` after the `Final` answer; jig-client's `Event::Send` takes no doc line in the sketch; `cargo fmt` runs after `send`'s signature is pasted. Not built by the check, as jig 01 was not in it: jig-client, jig-client-view, ops, the test domain and the protocol world, the fuzzy and browser profiles, `named.rs` and the Done-when greps; they are checked again when 02.1 starts. A further site the compiler or clippy names: fix it as the step's Shape says, and list it in the commit body.

105. **jig 03's Shapes against main** (ready check of jig-03.1).
     jig 03's ready check (skein 93052c6b, smith a277e922, jig dca2bbfb, temper 3119ae3a) applied 03.1 in a scratch worktree and, with the further sites its diagnosis found, ran jig's gate at version 9: drift with and without `SKEIN_CODEGEN_WRITE=1`, the workspace's 4270 tests, fuzzy's 111, clippy (native and wasm32) and fmt clean, the browser tier's 13; the proof `an_engine_outside_jig_s_version_leaves_the_client_out_of_date_sending_nothing` passed in 0.019 s. jig/03-turn-read.md now says: the focused budget is `.config/nextest.toml`'s 30 s, not 15 s; 03.1's jig-version literals are those a wider `rg` finds, 32 files at jig `1899792f`, named in its Files: among them the scripted engines' `versions().contains(8)` in `tests/client/src/scripted.rs` and `tests/ops/src/client/transport.rs` (left at 8, about 68 client stories read "Client out of date"), three `version(), 8` asserts, `SERVICE_ANSWER`'s version byte and `Owner::Jig, 8, 8`; family numbers stay; the `a_v8_*` tests become `a_v9_*`, `v8` leaves three test files' panic texts, and tech-debt.md names v9; `client-v9.schema`'s first line is `family jig-client-wire 9`; the out-of-date story serves `serve_jig(8)` and expects `VersionRange { first: 8, last: 8 }`, the restart story serves 9 to 9, and `cargo fmt` runs after; 03.2's held answer is `Held::Read` (`Held::ReadAnswer` does not exist), and the client `ReadNone` matches in `examples/ops/client-domain/src/family.rs`, `tests/client-protocol/src/family.rs` and `tests/client/src/scripted.rs` join its Files; `ReadValue`'s matches in `tests/client/src/{observe,scripted,harness/referee,harness/referee/offers}.rs` join 03.3's; `tests/client/src/joint/wire/pages.rs` joins 03.4's; the last Done-when command, which passes on `main` already, now greps `tests/client-protocol/tests/turn_read.rs` for `a_turn_read_after_restart_keeps_its_archive`. temper's users of v9, `TurnProblem`, `ReadValue` and `ReadNone` are in temper 03.1's Files already. The browser story `under_a_prefix_the_inbox_s_links_carry_the_prefix` failed once under load and passed three reruns, unrelated to the version: if the gate fails it, report the log as an infrastructure block, without a retry loop. A further jig-version literal: change it to 9, and list it in the commit body.

106. **Joint stories share the fixture** (steer of 2026-10-19T04:40:39Z).
     From the hourly check of 2026-10-19T04:40:39Z. The README's "Rules of this plan" gains "Joint stories share the fixture" (testing-strategy.md, 10: "A setup written once", "Named fixtures and sizes"): a joint story in jig's `tests/client/src/joint/` builds its turn with `fixture.rs`'s `turn`, given a parameter there if it needs one, and writes no turn, call or result of its own; a Shape that asks for one gets it once, in `fixture.rs`, said in the commit body; each size and job id is a named const with a line on how its number is derived; the first increment that edits `stories/job_stories.rs` moves its own `turn`, `call` and `result` (jig-01.2) there too. Checked at jig `aa3e661c`: `tests/client/src/joint/stories/job_stories.rs` defines `fn call(id: &[u8], name: &[u8]) -> source::Block`, `fn result(id, value: source::Outcome) -> source::Block` and `fn turn() -> Box<[u8]>`, with the literal `256` three times; `tests/client/src/joint/fixture.rs` has `pub(super) fn turn() -> Box<[u8]>`. Prerequisite 4 now says TEMPER_FORGEJO is set on this machine (Forgejo 17.0.0, the pinned build, decisions.md, C), so temper 01.1's process proof runs here: no Proof lands "unavailable", and a layer that cannot run blocks as infrastructure. A further copy of a story's own turn, call or result found later: move it to `fixture.rs` the same way, and list it in the commit body.

107. **jig 05's Shapes against main** (re-check of jig-05.1).
     jig 05's re-check of jig-05.1 (skein 93052c6b, smith d56bbe3c, jig aa3e661c, temper 6f8ee6665; read only, nothing compiled, the queue paused behind jig-04's exclusive gate): 05.2's Files now name the unit, world and scripted fixtures that build `people::Member`, `tasks::New` or `ProposedTask` literals outside the reviewies it named, found by `git grep -l -E '(\b|::)(Member|New|NewTask|ProposedTask) \{' origin/main -- '*.rs'` at jig `aa3e661c`: `crates/jig-domain-core/src/{tests,tests/fixtures}.rs`, `crates/jig-domain-people/src/tests/fixtures.rs`, `crates/jig-domain-tasks/src/tests/{last_batch,revisions,revisions/text}.rs`, `crates/jig-protocol-store/src/tests/fixtures.rs`, `testing/jig-client-scripted-engine/src/{owner_history/tests,owner_proposal/tests,tests/fixtures,tests/recurring}.rs`, `tests/client/src/{stories,joint/engine_stories,joint/wire/history,joint/wire/historical/tests,joint/wire/projects/tests}.rs` and `tests/ops/src/conformance/mod.rs`; a listed file with no such literal is left unchanged. 05.2 and 05.3 also name `crates/jig-protocol-store/src/sizes.rs` and `src/tests/format.rs`: `largest` takes `tasks_v4::Row::worst_case_bytes` and `people_v4::Row::worst_case_bytes` (jig-05.1, `f392967c`), so `format.rs`'s 81_241, 81_353, 17_499 and 34_998 follow the grown rows, refreshed by the 5.7 rule. The check's worry that the Done-when grep for `Deadline::Span` in `codec/people` is vacuous does not hold: with `-w`, `TemplateDeadline::Span` does not match, and `git grep -n -w -F 'Deadline::Span' origin/main -- crates/jig-protocol-store/src/codec/people crates/jig-protocol-store/src/codec/tasks` prints nothing at `aa3e661c`. temper's own users of these types (`temper-client-domain` `pages/chats.rs`, `temper-service` `tests/fixtures.rs`, `temper-client-view` `cards/proposal.rs`, `temper-protocol-store` `records/{application,application/refusal}.rs` and `tests/records.rs`) are temper's repin's. A further literal the compiler names outside these Files: give it the selector as its step says, and list it in the commit body.

108. **temper 02's Shapes against main** (ready check of temper-02.1).
     temper 02's ready check (skein 93052c6b, smith d56bbe3c, jig aa3e661c, temper 482d5fefe; read only, nothing compiled, the queue paused behind jig-04's exclusive gate). temper/02-tools-table.md now says: `client()` writes `text: TEXT` beside `rows: ROWS` and `document_bytes: DOCUMENT`, and the preset test sets `rows` and `text` aside, because client/tools.md, 12, gives "a result's text | 32 KiB" among the bounds of "the tools' own preset of `jig-client`'s `Limits`, not the terminal's", and jig bounds a received document's text by it (`crates/jig-client-protocol-link/src/limits.rs` `wire`: `wire.document_text_bytes = client.text`), where native's `text: 128` would refuse a result of 129 bytes or more; `each_ceiling_is_within_the_client_limit_it_reads_from` also asserts `limits::TEXT == client.text`; if `the_tools_preset_has_a_worst_case` refuses the preset at 50 rows and 32 KiB, that is a block for the designer, not a smaller bound. `jig::table::Limits`'s doc comment says "from the host's preset", not temper's `limits::tools()`, which `no_file_under_jig_names_temper_or_an_item_outside_jig` and the Done-when's grep under `src/jig` would refuse. The Done-when's tool-name grep matches `name: b"<tool>"`, since `b"me"`, `b"task"` and `b"run"` also occur as words of the rows' sets. temper's `clippy.toml` bans `alloc::vec::Vec`, `alloc::string::String`, `core::matches` and `core::iter::Iterator::all` in tests too: `fixtures::sources()`, `Name::new` and the boundary test's body are written within it, as `temper-box/src/tests/boundary.rs` reads its sources. Interim, on `--order`: jig's `QueryOrder` has five variants (`Number`, `Changed`, `Activity`, `Priority`, `Ended`, `crates/jig-client-domain/src/contract/queries.rs`) and `TaskQuery.order` is no `Option`, while client/tools.md, 4, gives `tasks … [--ended] [--order activity|priority]` and names no order for a call without `--order`; that goes to the designer (question.md). 02.1 goes on; in 02.2, `--order`'s default and `back`'s words for `Number`, `Changed` and `Ended` wait for the answer, and the rest of 02.2 does not. A further copy of a bound or a tool name found later: change it as its step says, and list it in the commit body.

109. **temper 01.1's owned example bounds and runner** (plan defect and infrastructure, temper 01's block of 2026-10-19T04:58:22Z).
     Ruling 109. 1. The owned example's bounds. setup.md, 8.1, says it in its own words: "Its `memory_bytes` hold its worst cases wherever a person lays it and whatever its forge's files hold". `docs/examples/owned.json`'s `engine.memory_bytes` and `workers[0].memory_bytes` are those worst cases, measured by `serve_prepares_once_on_the_owned_example` at the bounds of what a copy changes, as forge-setup's `temper-14.s2` raised them; they are no input the Don't list guards. So each refused number rises by the N of its refusal, "the worst case passes memory_bytes by N bytes": the engine's by 264, the worker's by 366,843,064,659, or by what the test says on the final tree. Nothing else in the example, no worst case and no other test changes. The commit body gives each refusal before the raise, and the test passes after it. temper/01-repin.md's Files, Shape step 7 and Don't now say this. setup.md's figures for them are the designer's: later.md has the row. A further copy of either number: refresh it, and list it in the commit body.
     2. The runner. forgejo-runner 13.2.0, the pinned build (sha256 fadaec89…fcad1), is at `/home/free/.local/opt/forgejo-runner-13.2.0/forgejo-runner`, and `~/.profile`, which each `bash -lc` reads, exports `TEMPER_FORGEJO_RUNNER` with that path beside `TEMPER_FORGEJO`. Run 01.1's process proof as the plan gives it, with both set; it lands no Proof "unavailable". A command whose environment lacks either is a shell started before 05:13Z: start it again from a login shell.

110. **temper 02.2's order of tasks** (steer of 2026-10-19T05:25:37Z).
     temper 02.2's order of `tasks`, from decisions.md, J, decided  for the user (temper d3704b9d6): "without `--order`, `tasks` reads by `QueryOrder::Number`, ascending, and `--ended` changes no order. The tools never send `Changed` or `Ended`, so `jig::argv::back` answers that a call with either order has no argv form, a refusal and never a guess, and `parse(back(call)) == call` holds over every call `parse` makes." temper/02-tools-table.md now says so: Choices' argv forms, 02.2's Shape on `--order` and on `back`, with its Test shape's three cases, and the sketch's `back`, which returns `Result<(), Unwritten>`, `enum Unwritten { Overflow(Overflow), NoForm }` (`Overflow`, skein-lib's `writer.rs`). This ends ruling 108's interim wait on `--order`; the rest of ruling 108 stands. client/tools.md, 5.2, says it at the designer's next pass, after this plan: later.md has the row. A further user of `back`'s error that the compiler names: give it `Unwritten`, and list it in the commit body.

111. **temper 02.1's worktree** (process, temper 02.1's block of 2026-10-19 05:26:05 UTC).
     `wt/temper-02` was the supervisor's: ruling 111's premise is that temper 02's ready check, behind ruling 108, added it as its scratch worktree at `origin/main` 482d5fefe, at 04:40Z, and left its sketch of `crates/temper-client-tools/` there. It held no author's work. The supervisor removed it, and `wt/temper-01r`, a re-check's that was not restarted. Run `plan worktree temper 02` again and start 02.1 from temper/02-tools-table.md as landed: it includes ruling 110, J's order of `tasks` without `--order`, which ends ruling 108's interim wait on `--order`; the rest of ruling 108 stands. The supervisor's scratch worktrees are named `<repository>-<NN>r` from now, as `jig-02r`, `jig-03r` and `jig-05r` are: a worktree under `wt/` with a session's own name that the ledger gives no author is a block, as this one was.

112. **jig 03's Shapes against main** (re-check of jig-03.3).
     jig 03's re-check (jig f67d51c7, after 03.1 and 03.2 landed; 03.3 running) corrected jig/03-turn-read.md: 03.3's Files add `testing/jig-client-scripted-engine/src/payload.rs` (`impl Payload for Read`, `impl Payload for ReadValue`), `crates/jig-client-view/src/tests/domain/objects.rs` (its `ReadValue` match) and `tests/client/src/joint/read_ledger_pages.rs`, whose `match what` lists every `client::Read` with no wildcard: `client::Read::Turn { .. }` joins its `panic!("one of the two original histories")` arm; 03.4's `pages.rs` has two `client::TurnProblem::TooLarge`, not three; 03.4's `Undecodable::TooLarge` is the arm of `turn_fields`'s match on `jig_protocol_transcript::decode`, not the link's `translate::Undecodable`; the budgets are `.config/nextest.toml`'s `global-timeout`s, 15 s and 60 s; `read.rs`'s base is 752 lines. A further exhaustive consumer the compiler names: add its arm, and list it in the commit body.

     Checked (jig origin/main f67d51c7):
     - `git grep -n 'impl Payload for Read\b\|impl Payload for ReadValue' -- testing/jig-client-scripted-engine/src/payload.rs`: 888 `impl Payload for Read {`, 919 `impl Payload for ReadValue {`.
     - `git grep -n 'match what\|one of the two original' -- tests/client/src/joint/read_ledger_pages.rs`: 35 `let expected = match what {`, 51 `| client::Read::Deployment => panic!("one of the two original histories"),`.
     - `git grep -c 'TurnProblem::TooLarge' -- tests/client/src/joint/wire/pages.rs`: 2.
     - translate/pages.rs `turn_fields`: `Err(jig_protocol_transcript::Undecodable::TooLarge) => c::TurnProblem::TooLarge`; translate.rs:11 `pub enum Undecodable { Malformed, TooLarge, Version }`.
     - `.config/nextest.toml`: `[profile.default]` `global-timeout = "15s"`, `[profile.fuzzy]` `global-timeout = "60s"`.
     - `git show origin/main:crates/jig-protocol-store/src/transcripts/read.rs | wc -l`: 752.

113. **temper 02's Shapes against main** (re-check of temper-02.1).
     temper 02's re-check (skein 0db7b1fc, smith 538ef49a, jig 31aae420, temper a2fd8e86; scratch with temper's pins, jig 719bd70d) corrected temper/02-tools-table.md: the `takes` doc comment of the Interface sketches backticks `Ended`, `WaitingOnMe`, `Task` and `Phase`, which clippy's `doc_markdown` refuses bare under `-D warnings`; 02.1's Test shape reads `src/jig.rs` and `src/jig/**` under `env!("CARGO_MANIFEST_DIR")`, since temper-box's boundary test walks a relative `src`. The rows (`limits::ROWS`, 50 against jig's wire ceiling of 32) are decisions.md, For the user, K, and stay as they are until it is decided: with them at 32, the check passed 6 of 6 in `temper-client-tools` and 94 of 94 across it, `temper-client-domain` and `temper-client-shell-terminal`, clippy clean. jig main's `Held` and named send are temper 03.1's, at its repin, as temper/03 says. A further sketch comment that clippy refuses: backtick its names, and list it in the commit body.

     Checked:
     - temper/02-tools-table.md, before: `/// tree/run: Ended only; WaitingOnMe: Task only; Phase: ten live phases only.`
     - `git -C ~/src/rust/temper grep -n '"src' origin/main -- crates/temper-box/src/tests/boundary.rs`: 53 `pending.push(std::path::PathBuf::from("src"))`.
     - temper/03-*.md:3 "temper repins onto jig's named send, turn read and deadline spans (03.1)"; :17 names `Held`.

114. **temper 02.1's rows** (design gap, temper 02.1's block of 2026-10-19 05:31:30 UTC).
     decisions.md, For the user, K, decided  (temper 724865e25): the tools' rows are 32, jig's client wire ceiling, wherever client/tools.md, 12, and the plan say 50; jig's wire stays as it is. `limits::ROWS: u32 = 32` in temper 02.1; the plan now says 32 for every page, window and `--limit` that read it: temper 02.2's `READS` row `activity 41 --limit 32` and `a_value_outside_its_kind_or_rows_is_a_usage_error` with `--limit 33`; temper 03's pages (`at most 32 rows`, `Subject::Tree { task, window: 32 }`, `--activity K` and `--limit K` at most 32); temper 04's windows (`fixtures::inbox_first_window(33)`, `read.rows()` 32, the 31/32/33-row cases); the Done-when grep, `ROWS: u32 = 32`. `the_tools_preset_has_a_worst_case` stays as the plan gives it: with `ROWS = 32`, temper 02's re-check passed it, 6 of 6 in `temper-client-tools`. client/tools.md, 12, says so at the designer's next pass (later.md). Any further 50 that is the tools' rows, or 49 or 51 beside it: make it 32, 31 or 33, and list it in the commit body.

115. **temper 12's Shapes against main** (ready check of temper-12.1).
     temper 12's ready check (skein 0db7b1fc7, smith 538ef49a5, jig f67d51c78, temper 1ceb21697; temper's pins skein b4c6241, jig 719bd70) corrected temper/12-tools-help.md: `jig::json::Refusal` derives `Clone, Copy, PartialEq, Eq, Debug`, which the reference test's comparison and the workspace's `missing_debug_implementations` need, and gains `Refusal::Text`: `skein_json::writer::Encoder::measured` refuses with `TooLong`, `TooDeep`, `Text` or `Number`, and argv's `Value::Text` and `File` carry bytes that need not be UTF-8, so `write_call` and `write_tool` match `TooLong | TooDeep` to `Long` and `Text | Number` to `Text`, with a match, since clippy.toml bans `map_err`; help measures into `skein_lib::Writer::new(text)` (`usize::try_from(limits::TEXT)`) and writes into a `Writer` of the measured `written()` length, since a `Writer` is exact-length and `finish` asserts it full, and `the_help_fits_the_tools_window` asserts on the measured length; 12.2 needs temper 03.2, whose `fixtures::golden(name)` its goldens read; `a_document_past_ask_bytes_is_a_usage_error` expects `Usage::Json`, the length checked against the u64 `ask_bytes` before the u32-length tokenizer. A further refusal or writer the compiler or clippy names: handle it the same way, and list it in the commit body.

     Checked:
     - `git -C ~/src/rust/skein grep -n 'pub fn measured\|TooLong,\|TooDeep,\|Text,\|Number,' b4c6241 -- crates/skein-json/src/writer.rs`: 395 `pub fn measured(self) -> Result<u32, Refusal>`; `Refusal` 70-78: `TooLong`, `TooDeep`, `Text`, `Number`.
     - `git -C ~/src/rust/skein grep -n 'pub fn new\|pub fn written\|assert' b4c6241 -- crates/skein-lib/src/writer.rs`: 32 `pub fn new(len: usize) -> Writer`, 38 `pub fn written(&self) -> usize`, 61 `assert!(self.bytes.len() == self.len, "a writer is finished full")`.
     - `git -C ~/src/rust/temper grep -n map_err origin/main -- clippy.toml`: 120 `{ path = "core::result::Result::map_err", reason = "no closures: match" }`.
     - temper/03-tools-root.md:62: `golden(name)` (reads `src/tests/golden/NAME.txt`), in 03.2's `src/tests/fixtures.rs`; temper/02-tools-table.md:369: `Usage` has `Json`.

116. **The check of 05:45: setups, rows and lint exceptions** (steer of 2026-10-19T05:45:56Z).
     The check of 2026-10-19T05:45:56Z adds two rules to the README's "Rules of this plan": "A setup written once, and rows carry data", for each test an increment adds or edits, with jig's `read_activity.rs` and `requests.rs` mended by the first increment that edits them; and "No lint exception for a plan's words". temper/02-tools-table.md, 02.1's Test shape, now reads the sources by relative paths, as temper-box's boundary test does: ruling 113's `env!("CARGO_MANIFEST_DIR")` is a call clippy.toml bans, and temper 02.1 landed it under `#[expect(clippy::disallowed_macros)]`; the next temper 02 increment removes both. Reverses: ruling 113 of 2026-10-19T05:41:43Z, its `CARGO_MANIFEST_DIR` sentence.

     Checked:
     - `git -C ~/src/rust/temper grep -n core::env origin/main -- clippy.toml`: 94 `{ path = "core::env", reason = "behaviour does not depend on the build environment" }`.
     - `git -C ~/src/rust/temper show 72a96c022:crates/temper-client-tools/src/tests/fixtures.rs`: `#[expect(clippy::disallowed_macros, reason = "the boundary fixture reads this crate's source tree")]` above `pub(super) fn sources()`, `Path::new(env!("CARGO_MANIFEST_DIR"))`.
     - `git -C ~/src/rust/temper grep -n '"src' origin/main -- crates/temper-box/src/tests/boundary.rs`: 53 `pending.push(std::path::PathBuf::from("src"))`.
     - testing-strategy.md, 10: "Rows carry data" (546), "A setup written once" (555), "Named fixtures and sizes" (561), "A bound's two sides" (573).

117. **temper 12.2's Shapes against main** (re-check of temper-12.1).
     temper 12's re-check after its start (temper 1e089afb5; 12.1 landed there, so only 12.2 is corrected) found 12.2's usage rule short of the rows temper 02 landed. temper/12-tools-help.md, 12.2, now says: a switch is `[--name]`, read by its place before its kind; a repeated flag is `[--name SHOWN]...`; a positional's `Kind::Word` set shows its words joined by `|`; `Kind::Order` shows `activity|priority`; `src/jig/rows.rs` joins the Files and takes client/tools.md 5.3's order (`CHAT_CREATE`'s `project` before `text`, `CANCEL`'s `grace` before `reason`, `DECIDE`'s escalation form `hold` after `task`); help gives every row named `chat`, `help::text(Some(i))` row `i` and each later row of its name; `subject::live` becomes `pub(crate)` (`src/jig/subject.rs` joins the Files) with one const of every `Phase`; the guide's reference test and `fixtures::parse_line` are `for` loops over `GUIDE.get(at)`, since clippy.toml bans `Vec`, `slice::split`, `filter_map`, `collect` and `Option::is_none_or`; the help sketches import `skein_lib::{Overflow, Writer}`. Where a row's `shown` or order still differs from 5.2's to 5.6's line, the row changes to the line's, with each test whose expected text follows the rows' order: list each in the commit body.

     Checked:
     - `git show origin/main:crates/temper-client-tools/src/jig/rows.rs`: `CHAT_CREATE` lists `positional(b"text", …)` then `flag(b"project", …)`; `CANCEL` `reason` then `flag(b"grace", …)`; `DECIDE`'s second form ends `flag(b"hold", b"R", …)`; `TASKS` has `flag(b"order", b"O", Kind::Order, …)`.
     - `git grep -n -E 'chat --project|cancel N|--hold R|--order' origin/main -- docs/design/client/tools.md`: 197 `chat --project P TEXT`; 206 `cancel N [--grace D] REASON`; 203 `decide N --hold R release|leave|pass-up [REASON]`; 162 `[--order activity|priority]`.
     - `jig/table.rs:91`: `switch` builds `shown: b"", kind: Kind::Word { set: &[] }, … place: Place::Switch`.
     - `jig/argv.rs:92`: `fn tool_index` returns the first row whose `name == word`; `table.rs:13,17`: `&rows::CHAT`, `&rows::CHAT_CREATE`.
     - `jig/subject.rs:97`: `fn live(phase: Phase) -> bool` (private); `:79` `pub fn phase_word`.
     - `git grep -n -E 'env!|disallowed_macros' origin/main -- crates/temper-client-tools/src`: nothing; ruling 116's mend of `sources()` is on main.
     - The sub-agent's run at f0743c0e2 with the corrections: `cargo clippy -p temper-client-tools --all-targets -- -D warnings` exit 0 (exit 101 as written); default profile 36 passed with the rows reordered.

118. **jig 05.2's shipped store bounds** (plan defect, jig 05's block of 2026-10-19 06:47:37 UTC).
     Take the accounting repair the report proposes: in `crates/jig-protocol-config/src/profile/small.rs`, `STORE_KV`'s `value` 1_303_207, `chunk` 1_305_306 and `page.bytes` 1_305_286 (each +8); in `profile/standard.rs`, `value` 10_396_367, `chunk` 10_398_466 and `page.bytes` 10_398_446 (each +24); and the rows of `testing/jig-test-service/src/tests.rs`'s `both_jig_profiles_have_a_bounded_service` follow them (`value` only: `commit`, `ops` and `queued` stay as they are). These bounds are accounting that follows the encoded rows: no design text names them, and 05.2's Don't, "change … template configuration", is about templates, "the application's definition of a shape of work, numbered data in its configuration" (jig's `client/domain/README.md`), not a store profile's bounds. The session's question rule was kept: the report gave needs and has (Small needs 1_303_207, has 1_303_199; Standard 10_396_367 against 10_396_343). Both files join 05.2's Files, and 05.3's, whose Span carries a `Duration` and grows the rows again: there, too, a profile whose `value` a grown row passes takes the growth, needs minus has, in `value`, `chunk` and `page.bytes` alike, and the service test's row follows, listed in the commit body. A need in `commit`, `ops`, `queued` or memory is still a question. Nothing downstream copies these values: temper's own sums, as `docs/examples/owned.json`'s `engine.memory_bytes`, follow the jig that temper 03.1 pins, in that step. A further copy of a changed bound: change it by the same growth, and list it in the commit body.

119. **temper 03's Shapes against main** (ready check of temper-03.1).
     temper 03's ready check (skein c551a66, smith c241a7a4, jig 590015b4, temper a6f34f6af; scratch with the corrections, : default 1857 passed and 1 skipped, fuzzy 50 passed, clippy and fmt clean) corrected temper/03-tools-root.md. 03.1: the skein pin stays where temper's own descends from SKEIN01, and moves to it where not; smith's packages share one revision, the newest temper's `Cargo.lock` holds at the step's start; the store families are at v4 on main already; no match of `ReadNone` needs an arm, and the terminal fixtures' `read_value` gains `Read::Turn {.. } => None`; `docs/examples/owned.json`'s `engine.memory_bytes` rises by needs minus has, as `serve_prepares_once_on_the_owned_example` reports it (Shape 6). 03.2: `task N` prints `Admission::Bounded` as client/tools.md, 4 and 6.4, say; `a_request_under_the_pages_owner_is_answered_undecodable` takes `Owner::Runner` too; the Don't is `src/tests/boundary.rs`'s rule, any case, comments and strings included; the sketches take `#[must_use]` and `call` takes a `Called`. 03.3: `tasks` reads one group per call, the default of decisions.md, For the user, L, and builds the rest first until L is decided; `rows.rs` joins its Files. 03.4: the runner asks `Client::pending(key)` before each named send, since jig's client panics on a key held for another request. A further user of a value these change: change it as its step says, and list it in the commit body.

     Checked:
     - `git -C temper show a6f34f6af:Cargo.lock | grep -o 'git+…forgetic/[a-z]*.git#[0-9a-f]*' | sort | uniq -c`: skein `0db7b1fc` (30); smith `538ef49a` (19) and `a277e922` (9).
     - `git -C jig show 590015b4:Cargo.lock`: skein `93052c6b`; `git -C skein merge-base --is-ancestor 93052c6b 0db7b1fc`: yes. `git -C smith diff --stat a277e922 538ef49a -- crates`: nothing.
     - `git -C temper grep -n -E 'people_v3|tasks_v3' a6f34f6af`: only `docs/development/plans/client-api/`.
     - `git grep 'ReadNone::'`: `terminal/native.rs:17`, one `ReadNone::SignedOut` arm in a non-exhaustive match; `tests/fixtures.rs:661`: `fn read_value(what: &contract::Read) -> Option<contract::ReadValue>`.
     - jig 590015b4 `crates/jig-client-domain/src/requests.rs:59`: `assert!(held.request == request && named, "request key conflict")`; `contract/documents.rs:240`: `Bounded { within: Bounds, beyond: Beyond }`, `:265` `pub enum Beyond`.
     - temper `crates/temper-client-tools/src/tests/boundary.rs:5`: `assert!(!text.to_ascii_lowercase().contains("temper"), …)`; `docs/examples/owned.json:192`: `"memory_bytes": 78669036813790`, +370824 is 78669037184614; `temper-service/src/tests/family.rs:276`: `fn a_request_under_the_pages_owner_is_answered_undecodable()`.
     - jig `crates/jig-client-view/src/page/conversations.rs:405`: `query.phase = Some(contract::PhaseClass::Ended);` `jig-client-domain/src/contract/reads.rs:47`: `Tasks { query: super::TaskQuery, live: Option<Cursor>, ended: Option<Cursor> }`.

120. **Test shape: fixtures built on, sizes named, one behaviour per test** (steer of 07:05).
     From the hourly check of 06:51:27Z: the README's "Rules of this plan" now hold, for each test an increment touches in a file it edits, that a case extending a fixture is built from it (`[fixtures::ADOPTION, &["--no-home"]].concat()`), never retyped; that each size a test uses (a depth, a capacity, a chunk) is a named const of its fixtures, with a line on what a smaller value would miss; and, as a rule of its own, "One behaviour per test" (testing-strategy.md, 10): a test checks one behaviour and is named for it, never with "and", and a two-behaviour test in a file an increment edits is split there, said in its commit body. 12.1's plan-given test names stay.

     Checked:
     - `git -C ~/src/rust/temper grep -n 'fn adopts_flags_parse_and_its_absent_flags_stay_absent\|fn json_arguments_use_snake_case\|List::with_capacity(32)\|depth: 6\|depth: 2' origin/main -- crates/temper-client-tools/src/tests`: `table.rs:237` the adopt test, `table.rs:295` `List::with_capacity(32)`, `json.rs:168` `depth: 6`, `json.rs:180` `fn json_arguments_use_snake_case()`, `fixtures.rs:114` `writer::Limits { depth: 2, … }`.
     - The kit's testing-strategy.md, 10: line 540 "One behaviour per test", line 561 "Named fixtures and sizes".

121. **L into temper 03.3** (For the user, L, decided ).
     decisions.md, For the user, L, decided  (temper 09cfaca18): the default. temper/03-tools-root.md, 03.3's Lists, now names that decision: `tasks` reads one group per call, the live group without `--ended` and the ended group alone with it, as jig's view reads it; the call that reads on keeps `--ended`, its help reads "Only ended tasks", and an agent that wants both makes two calls, the printed cursor jig's, with no group byte. 03.3 no longer waits on L. later.md has its row for client/tools.md, 5.2, at the designer's next pass. Any further text that reads `--ended` as "include": make it "only", and list it in the commit body.

     Checked:
     - `git -C ~/src/rust/temper show 09cfaca18 --stat`: `Decide the tools plan's item L: one group of tasks per call`; decisions.md's L ends "the printed cursor stays jig's, with no group byte. client/tools.md, 5.2, says so at the designer's next pass, after this plan."
     - `awk -F'\t' '$1 ~ /^temper-03/' ledger.tsv`: temper-03.1 ready, 03.2 to 03.4 waiting; no temper 03 session has started.

122. **deadline_spans.rs mended where next edited** (steer of 08:18).
     From the hourly check of 07:56:50Z: the README's "A setup written once, and rows carry data" now names jig's `tests/core/tests/deadline_spans.rs` (jig 05.3) among the files the first increment that edits them mends: its aged-span test of three behaviours, its row loop that branches on the expected value, its signed-in opener in three tests, and its bare seeds, request keys and walls, beside `fixtures/deadlines.rs`'s `SPAN` and `CREATED`. No step is added for it.

     Checked:
     - `git -C ~/src/rust/jig grep -n -E 'fn an_aged_span_is_beyond_its_accepter_and_passed_up|fn a_resolved_span_beyond_the_creator_is_refused|const SPAN|const CREATED' origin/main -- tests/core` (jig c0142dc2): `tests/deadline_spans.rs:66` and `:79`, the two tests; `src/fixtures/deadlines.rs:9` `pub const SPAN`, `:42` `pub const CREATED: u64 = 2`.

123. **temper-03.1's re-check against main** (re-check of temper-03.1, started 08:21:53Z).
     temper 03.1, started at 08:21:53Z, re-checked against what moved since its check (skein c551a66, smith c241a7a4, jig 590015b4, temper a6f34f6af): its Shape stands but for step 1's skein hash. forge-kit's temper-05.1 moved temper's skein pin from `0db7b1fc` to `378fd8cfdeb2af90261b8710b60fc91d70131792` (33 packages), a descendant of SKEIN01 (`93052c6b`, the one skein of jig 872bfb7c's `Cargo.lock`, the landing of jig-05.5). Step 1's rule, keep temper's own pin where it descends from SKEIN01, now keeps `378fd8cf`: where the jig update moves it, `cargo update -p skein-lib --precise 378fd8cfdeb2af90261b8710b60fc91d70131792`, never `0db7b1fc`, the pin before forge-kit's repin. temper/03-tools-root.md, 03.1, step 1, names it. smith's `538ef49a` stands. jig 05.4 took `now` from `shapes::chat` and `shapes::set_goal` and gave `shapes::fall` a `deadline`; temper calls only `chat`, in `Chats::start`, as step 5 has it. A further copy of the old hash: replace it with `378fd8cf`, and list it in the commit body.

     ## Checked

     - `git show <temper>:Cargo.lock | grep -o 'git+…forgetic/[a-z]*.git#[0-9a-f]*' | sort | uniq -c`: at a6f34f6af skein `0db7b1fc…` ×30; at origin/main (e10aa299f) skein `378fd8cf…` ×33; smith `538ef49a…` ×19 and `a277e922…` ×9 at both.
     - `git merge-base --is-ancestor 93052c6b 378fd8cf` (skein): true. jig origin/main 872bfb7c's lock: skein `93052c6b…` ×34 only.
     - `git log a6f34f6af..origin/main -- Cargo.lock` (temper): 9c8e5a2f8 "Repin skein before moving temper's fake forge users", Plan: forge-kit, Step: temper-05.1.
     - `git diff 590015b4 origin/main -- crates/jig-client/src/shapes.rs` (jig): `chat(me, project, words)`, `set_goal(me, project, draft, reason)`, `fall(creating, draft, deadline)`; `git grep -nE 'set_goal\(|shapes::fall|\bfall\('` in temper's crates, tests, testing: none; `shapes::chat` at `crates/temper-client-domain/src/pages/chats.rs` only.

124. **temper 11's Shapes against main** (ready check of temper-11.1).
     temper 11's ready check (skein 6e2907921, smith 137a1cb48, jig 872bfb7c, temper e10aa299f) compiled the corrected 11.1 and 11.2 sketches under clippy and passed nextest 1689/1689 and fuzzy 50/50. temper/11-tools-forge.md now says: the domain's `Resource` derives `Clone, Copy, Debug, PartialEq, Eq`, as the view's does, and `resource`, `owner`, `joined` and `full_name` are `#[must_use]`; `forge::answer` gives `Some(Answer::Unknown)` for jig's `Done` and each outcome `told` takes as unknown, and `None` only when the family record or the limits cannot be decoded, as `told` does today; 11.1 also moves the pure mappings to `Word` (from `page.rs`'s `broken`, `kinds` and `outcome`'s role, and `path_text`'s resource, with `most` passed in) into domain `forge::words`, which the view's functions call, keeping their names and output, and which 11.2's Parts, marks and outcomes call, so that the tools copy none of them: the sketches' preamble already gives the domain the pure conversions; `Unshowable` (id 24) stays in the view's words; the tools' `skein-ui` dependency is temper 03.2's, in its Files, so 11.2 adds none. The 11.2 fixtures the check did not find on `main` (`signed_in`, `PARTY`, `golden` and the others) are temper 03's: its Fixtures rule names the harness, and its tests' sketches use `fixtures::PARTY`. A further copy of a moved mapping: call the domain's, and list it in the commit body.

     ## Checked

     - `git grep -n "Unshowable" origin/main -- crates/temper-client-view/src`:
       crates/temper-client-view/src/words.rs:123:    Unshowable,
       crates/temper-client-view/src/words.rs:234:    Word::Unshowable,
       crates/temper-client-view/src/words.rs:350:            Self::Unshowable => 24,
       crates/temper-client-view/src/words.rs:586:        Word::Unshowable => (&[b"Cannot show this part."], &[]),
     - `git show origin/main:crates/temper-client-view/src/forge.rs | grep -n -B1 "pub enum Resource\|fn joined\|fn full_name\|fn path_text"`:
       10-#[derive(Clone, Copy, Debug, PartialEq, Eq)]
       11:pub enum Resource<'a> {
       --
       40-}
       41:fn joined(segments: &[Box<[u8]>], most: u32) -> Box<[u8]> {
       --
       71-}
       72:fn full_name(owner: &[u8], name: &[u8]) -> Box<[u8]> {
       --
       80-}
       81:fn path_text(builder: &mut Builder, path: &[Box<[u8]>], words: &Words) {
     - `git show origin/main:crates/temper-client-view/src/forge/page.rs | grep -n "^fn broken\|^fn kinds\|^fn outcome\|forge::Role::"`:
       35:        (forge::Role::Owned, Word::OwnedChoice),
       36:        (forge::Role::Fork, Word::ForkChoice),
       37:        (forge::Role::Context, Word::ContextChoice),
       124:fn broken(builder: &mut Builder, broken: Option<forge::Broken>, words: &Words) {
       139:fn kinds(kinds: forge::Kinds, words: &Words) -> Box<[u8]> {
       173:fn outcome(builder: &mut Builder, form: &forge::Form, answer: &forge::Answer, words: &Words, text_limit: u32) {
       177:                forge::Role::Owned => Word::Owned,
       178:                forge::Role::Adopted => Word::AdoptedRole,
       179:                forge::Role::Fork => Word::Fork,
       180:                forge::Role::Context => Word::Context,
     - `git show origin/main:crates/temper-client-domain/src/forge.rs | grep -n "Answer::Done\|Answer::Unknown\|family::answer"`:
       98:                        draft.outcome = Some(Answer::Unknown);
       121:                    Answer::CoreRefused(_) | Answer::ForgeRefused(_) | Answer::EntranceRefused(_) | Answer::Unknown => {
       258:                    family::answer(&record, &wire_limits(&limits.forge)?, &link)?
       264:                Outcome::Answered(jig_client::Answered::Jig(jig_client::contract::Answer::Done(_)))
       269:                | Outcome::Unconfirmed => Answer::Unknown,
     - `grep -n "skein-ui" docs/development/plans/tools/temper/03-tools-root.md (docs worktree)`:
       59:  Use jig's phrases through `skein-ui`; decimal KB rounded up, text times UTC HH:MM, dated off the step's day.
       139:Files: `crates/temper-client-tools/src/root.rs` (new), `src/lib.rs`, `Cargo.toml` (`skein-ui`);
     - `git show origin/main:crates/temper-client-tools/Cargo.toml | grep -n "skein-ui"`:
       (none)
     - `grep -n "PARTY\|signed_in" 03-tools-root.md` (docs worktree):
       61:- **Fixtures:** one module, temper 02's `src/tests/fixtures.rs`, extended here with the root harness `signed_in`, `submit`, `serve`,
       170:      let mut root = fixtures::signed_in(fixtures::PARTY);
       192:      let mut root = fixtures::signed_in(fixtures::PARTY);

125. **temper 04's Shapes against main** (ready check of temper-04.1).
     temper 04's ready check (skein 6e2907921, smith 137a1cb48, jig 872bfb7c, temper e10aa299f) built the sketches into `temper-client-tools`: clippy clean once corrected, nextest 1690/1690, fuzzy 50/50; the boundary test (`no_file_under_jig_names_temper_or_an_item_outside_jig`) failed with the sketch's path comments copied as text. temper/04-waits.md now says: the fixtures' limits are `fixtures::limits()`, a function returning `limits::tools()`, which is no `const fn`, and a fixture neither 02 nor 03 adds (`PARTY`, `me`) joins the module in 04; 04's chat builder is `fixtures::chat_at(task, stage, work)`, returning a `Latest`, beside 03's `fixtures::chat(task, acts)`; `Facts` is built as a struct literal, `Facts { subject, me }`; `holds` reads a task condition from `Latest::Jig(Delivery::Task(task))`, its `waits_on`, `project` and `chip`, and gives None for another form; `waits_on_me` copies jig's private `reader` (owner 0, maintainer 1, member 2, observer 3, a policy role its own), since `chat_status`'s `WaitingForYou` comes only under an active chip, after held, closing and ended, while client/tools.md's `waiting-on-me` reads what the task waits on, and the commit body names the copy; `deliver`'s subject is temper's table `Subject::Task(41)`, or 03's form if its harness takes another; a run's delivery brings both `Told::Changed` and `Told::Text`, and each compares the subject's digest of its current document, so the second answers nothing; `Part` has a variant for each form a subject's read or delivery brings (`Inbox`, `OwnerInbox`, `InboxPage`, `OwnerInboxPage`, `Tasks`, `TaskPage`, `Tree`, `OwnerTree`, `RunState` beside the others; jig reads no tree page); each function returning a value is `#[must_use]`; the sketches' path comments name files and are not their text; "(temper 02)" in the digest's doc becomes "the table's"; `answers` takes a `match`, not `is_some_and`, which clippy.toml disallows. A further copy of a name or form this ruling changes: change it as the Shape now says, and list it in the commit body.

     ## Checked

     - `git -C temper grep -n 'pub fn tools()' origin/main -- crates/temper-client-tools/src/limits.rs`:
         origin/main:crates/temper-client-tools/src/limits.rs:57:pub fn tools() -> crate::jig::Limits {
     - `git -C temper grep -n -E 'LIMITS|PARTY|fn chat|fn limits' origin/main -- crates/temper-client-tools/src/tests/fixtures.rs`:
         (nothing)
     - `git -C temper grep -n 'is_some_and' origin/main -- clippy.toml`:
         origin/main:clippy.toml:114:    { path = "core::option::Option::is_some_and", reason = "no closures: match" },
     - `git -C temper grep -n -E '^    Task\(u64\)' origin/main -- crates/temper-client-tools/src/jig/subject.rs`:
         origin/main:crates/temper-client-tools/src/jig/subject.rs:9:    Task(u64),
     - `git -C temper grep -n 'fn no_file_under_jig' origin/main -- crates/temper-client-tools/src/tests/boundary.rs`:
         origin/main:crates/temper-client-tools/src/tests/boundary.rs:3:fn no_file_under_jig_names_temper_or_an_item_outside_jig() {
     - `git -C jig grep -n -E '^pub fn chat_status|^fn reader' origin/main -- crates/jig-client/src/facts.rs`:
         origin/main:crates/jig-client/src/facts.rs:28:pub fn chat_status(chip: &Chip, waits_on: &WaitsOn, project: u32, me: &Me) -> ChatStatus {
         origin/main:crates/jig-client/src/facts.rs:93:fn reader(whom: Whom, project: u32, me: &Me) -> bool {
     - `git -C jig show origin/main:crates/jig-client/src/facts.rs | sed -n 28,40p`:
         pub fn chat_status(chip: &Chip, waits_on: &WaitsOn, project: u32, me: &Me) -> ChatStatus {
             match chip.details.as_ref() {
                 ChipPhase::Owner { phase, details } => match phase {
                     OwnerChipPhase::Held { reason, .. } => ChatStatus::Held(*reason),
                     OwnerChipPhase::Closing(_) => ChatStatus::Closing,
                     OwnerChipPhase::Ended => ChatStatus::Ended { at: details.ended_at.expect("ended chip has its end time") },
                     OwnerChipPhase::Active(stage) => {
                         if let Some(waiting) = waiting(waits_on, project, me) {
                             return waiting;
                         }
                         active(*stage, details)
                     }
                     OwnerChipPhase::Waiting { .. } | OwnerChipPhase::Person { .. } => ChatStatus::Starting,
     - `git -C jig show origin/main:crates/jig-client-domain/src/contract/watches.rs | sed -n '/^pub enum Delivery/,/^}/p' | grep -E '^    [A-Z]'`:
             OwnerProject(Box<super::OwnerProject>),
             Me(Box<Me>),
             Task(Box<Task>),
             OwnerProposal(Box<super::OwnerProposal>),
             Tree(Box<Tree>),
             OwnerTree(Box<super::OwnerTree>),
             Chips(List<Chip>),
             Inbox(Box<Inbox>),
             OwnerInbox(Box<super::OwnerInbox>),
             Tasks(Box<super::Tasks>),
             Run(Box<RunText>),
             RunState(Box<RunState>),
             Deployment(Box<Deployment>),
             OwnerDeployment(Box<super::OwnerDeployment>),
     - `git -C jig show origin/main:crates/jig-client-domain/src/contract/reads.rs | sed -n '/^pub enum ReadValue/,/^}/p' | grep -E '^    [A-Z]'`:
             Tasks(Box<super::TaskPage>),
             OwnerProject(Box<super::OwnerProject>),
             Historical(Box<super::OwnerHistorical>),
             Me(Box<Me>),
             Turn(Box<super::ArchivedTurn>),
             Task(Box<Task>),
             Proposal(Box<Proposal>),
             OwnerProposal(Box<super::OwnerProposal>),
             Escalation(Box<Escalation>),
             OwnerEscalation(Box<super::OwnerEscalation>),
             Question(Box<Question>),
             PersonTask(Box<PersonTask>),
             OwnerPersonTask(Box<super::OwnerPersonTask>),
             Result(Box<Result>),
             Ended(Box<ObjectPage>),
             OwnerEnded(Box<super::OwnerEndedPage>),
             History(Box<HistoryPage>),
             Activity(Box<ActivityPage>),
             Inbox(Box<InboxPage>),
             OwnerInbox(Box<super::OwnerInboxPage>),
             Spend(Box<SpendPage>),
             OwnerSpend(Box<super::OwnerSpendPage>),
             Project(Box<Project>),
             Notes(Box<NotePage>),
             Deployment(Box<Deployment>),
             OwnerDeployment(Box<super::OwnerDeployment>),
     - `git -C jig show origin/main:crates/jig-client-domain/src/boundary.rs | sed -n '/^pub enum Told/,/^}/p' | grep -E '^    [A-Z]'`:
             Link { link: Link },
             Pending { key: Key },
             Answered { key: Key, outcome: Outcome },
             Kept { key: Key, request: Ask },
             Dropped { count: u32 },
             Changed { handle: Handle },
             Text { handle: Handle },
             Ended { handle: Handle, why: Why },
             Page { handle: Handle, page: Readout },
     - `git -C temper show origin/main:docs/design/client/tools.md | sed -n 309,311p`:
         - **`waiting-on-me`** covers task N's tree: what a task waits on is
           computed over its tree (jig's `domain/tasks.md`, section 3), so a
           question or a proposal below for the party meets it on `task N`, and

126. **temper 03.2: offered acts with no tool, interim** (design, temper 03.2's block of 2026-10-19 08:58:21 UTC).
     Interim, until decisions.md, For the user, M, is decided: what `task N`'s text prints for an offered act with no tool in client/tools.md, 5.3 (Take, HandBack, Amend, Move, Reorder, Seen, EditNote, DeleteNote, ChangePolicy, SetRoles, MakeService, SetPool, Withdraw) is a design gap, and waits; it goes to the user as M, as no designer is joined. temper 03.2 goes on with everything else: the root, the runner, routing, `me`, JSON, the exits, and every offered act that has a tool, printed as its Shape says (Message, Answer, Accept, Reject, PassUp, LeaveHeld, Release, Stop, Cancel, Adopt). Its tests and goldens use documents whose offered acts all have tools; the arm for an act with no tool is the one part that waits, and no call names a tool that 5.3 lacks. 03.2 does not land before the ruling on M reaches it. A further act kind with no tool: treat it as these, and list it in the commit body.

127. **temper 03.2: offered acts with no tool** (decision M, for temper 03.2's block of 2026-10-19 08:58:21 UTC).
     decisions.md, For the user, M, decided  for the user (temper 545b65a8a), the default, ruled into temper 03.2: `task N`'s `acts:` prints an offered act with no tool in client/tools.md, 5.3 (Take, HandBack, Amend, Move, Reorder, Seen, EditNote, DeleteNote, ChangePolicy, SetRoles, MakeService, SetPool, Withdraw) by its name in the tools' words, then ` (no tool)`, in the document's order, with no call and no admission; JSON prints the document's offers as 03.2's Shape says. temper/03-tools-root.md, 03.2's Shape, gains that bullet beside Refused, and its tests `an_act_with_no_tool_prints_its_name_and_no_call`; later.md gains client/tools.md, 6.1's sentence, for the designer's next pass. This ends ruling 126's wait: 03.2 lands once its arm does this. A further act kind with no tool prints the same way; list it in the commit body.

     ## Checked

     - `git -C ~/src/rust/temper show 545b65a8a:docs/development/plans/tools/decisions.md | grep -n -A3 'Decision: the default, 2026-10-19: an offered act'`:
         795:  Decision: the default, 2026-10-19: an offered act with no
         796-  tool prints its name in the tools' words and ` (no tool)`, in the
         797-  document's order, with no call and no admission. JSON prints the
         798-  document's offers as 03.2 says. The agent learns the act exists and can
     - `git -C jig show origin/main:crates/jig-client-domain/src/contract/documents.rs | sed -n '/^pub enum ActKind/,/^}/p' | grep -E '^    [A-Z]' | tr -d ' \n'`:
         Message,Answer,Take,HandBack,Accept,Reject,PassUp,Release,LeaveHeld,Amend,Stop,Cancel,Move,Reorder,Seen,EditNote,DeleteNote,Adopt,ChangePolicy,SetRoles,MakeService,SetPool,Withdraw,
     - `git -C temper show origin/main:docs/design/client/tools.md | sed -n '/^### 5.3/,/^### 5.4/p' | grep '^| `' | grep -o '`[A-Z][a-zA-Z]*`' | sort -u | tr '\n' ' '`:
         `Answer` `Cancel` `Create` `Decide` `Message` `Release` `Stop` 

128. **jig's deadline_spans.rs mends to later** (steer of 2026-10-19T09:14:01Z).
     The steer of 2026-10-19T08:18:50Z, point 1, which had jig's `tests/core/tests/deadline_spans.rs` mended at its next edit with no step of its own (ruling 122 and the README's "A setup written once, and rows carry data"), lapses: every jig step of this plan has landed, and neither that file nor `crates/jig-domain-core/src/tests/read_activity.rs`, which the same rule names, was edited after the rule. Their mends, and that of jig 05.5's `tests/client-protocol/tests/deadline_spans.rs`, whose `a_named_proposal_keeps_its_span_across_client_and_engine_restart` holds three behaviours with an "and" in its name (the hourly check of 09:02:04Z), go to `later.md`, for jig's next increment that edits either file, and to this plan's retrospective. The README's rule is unchanged: it holds for each test that temper's increments add or edit.

     ## Checked

     - `awk -F'\t' '$2=="jig" && $4!="landed"' ../ledger.tsv | wc -l`:
         0
     - `git -C ~/src/rust/jig log --oneline --since=2026-10-19T08:18Z origin/main -- tests/core/tests/deadline_spans.rs | wc -l`:
         0
     - `git -C ~/src/rust/jig log --oneline --since=2026-10-19T05:53Z origin/main -- crates/jig-domain-core/src/tests/read_activity.rs | wc -l`:
         0
     - `git -C ~/src/rust/jig grep -n 'fn a_named_proposal_keeps_its_span_across_client_and_engine_restart' origin/main -- tests/client-protocol/tests/deadline_spans.rs`:
         origin/main:tests/client-protocol/tests/deadline_spans.rs:173:fn a_named_proposal_keeps_its_span_across_client_and_engine_restart() {
     - `grep -n 'tests/core/tests/deadline_spans.rs' docs/development/plans/tools/README.md`:
         210:  destructuring), and `tests/core/tests/deadline_spans.rs` (jig 05.3:
     - `grep -n -- '--text' ../kit/scripts/dev/plan | head -2`:
         21:#   plan ruling [--text FILE] TITLE SOURCE  land a ruling in decisions.md, once
         490:# Decision of unblock.draft.md, or with --text a file's. It becomes the next

129. **temper 03.3: the inbox's call that reads on, interim** (design, temper 03.3's block of 2026-10-19 09:18:06 UTC).
     Interim, until decisions.md, For the user, N, is decided: how the call that reads on after an `inbox` selects its group is a design gap, and waits (jig's `Read::Inbox` reads one `InboxReadGroup` a call, each with its own opaque cursor, and client/tools.md 5.2's table has no group selector); it goes to the user as N, as no designer is joined. temper 03.3 goes on with everything else: `tasks`, `task N --tree --activity K --run`, `activity N`, `run N`, `chat N`, `turn N A.T`, and `inbox`'s first answer, needs then updates, each `Read::Inbox` with `after: None`, as its Shape says. Its inbox tests and goldens use groups that end on their first page, printed with `no more`; the parse of `inbox --after C` and the printed call that reads on after an inbox are the one part that waits, and no inbox cursor is printed or parsed meanwhile. 03.3 does not land before the ruling on N reaches it. A further read whose cursor names no group: treat it as this one, and say so in the report.

130. **temper 03.3: the inbox's call that reads on** (decision N, for temper 03.3's block of 2026-10-19 09:18:06 UTC).
     decisions.md, For the user, N, decided  for the user (temper ad1b1f80f), the default, ruled into temper 03.3: an inbox's printed cursor is one group byte (0 needs, 1 updates), then jig's cursor, whole; the call that reads on stays in needs while needs has more, then moves to updates; `--after C` reads C's group, and with `--kind K` a C of the other group (`InboxReadGroup::of`) is a usage error, exit 1. temper/03-tools-root.md, 03.3's Shape, gains that bullet under Lists, its tests `an_inbox_reads_on_in_needs_while_needs_has_more`, `an_inbox_reads_on_in_updates_once_needs_ends` and `an_inbox_cursor_of_another_group_than_its_kind_exits_1`, and 40 lines of Size; later.md gains client/tools.md, 5.2's sentence, for the designer's next pass. This ends ruling 129's wait: 03.3 lands once its inbox call does this. A further read whose cursor names no group: treat it as this one, and list it in the commit body.

     ## Checked

     - `git -C ~/src/rust/temper show ad1b1f80f:docs/development/plans/tools/decisions.md | grep -n -A2 'Decision: the default, 2026-10-19: the printed cursor is'`:
         858:  Decision: the default, 2026-10-19: the printed cursor is
         859-  one group byte (0 needs, 1 updates) and then jig's cursor, whole. The
         860-  call that reads on stays in needs while needs has more, then moves to
     - `git -C ~/src/rust/jig show origin/main:crates/jig-client-domain/src/contract/reads.rs | grep -n 'pub const fn of'`:
         33:    pub const fn of(kind: InboxGroup) -> Self {
     - `grep -n -- '--text' ../kit/scripts/dev/plan | head -1`:
         21:#   plan ruling [--text FILE] TITLE SOURCE  land a ruling in decisions.md, once

131. **temper 03.3: the inbox's first page, interim** (design, temper 03.3's block of 2026-10-19 09:25:20 UTC).
     Interim, until decisions.md, For the user, O, is decided: which rows the first `inbox` call prints, and where its call that reads on goes, within client/tools.md section 12's 32 rows for "a list's page", is a design gap, and waits (jig's `Read::Inbox` takes no row bound, and under N the call that reads on stays in needs while needs has more); it goes to the user as O, as no designer is joined. It replaces ruling 129's "`inbox`'s first answer, needs then updates, each `Read::Inbox` with `after: None`": until O is decided, temper 03.3 builds the inbox's group byte, its `--kind` and `--after` checks and the call that reads on in needs, as ruling 130 says, with `an_inbox_reads_on_in_needs_while_needs_has_more` and `an_inbox_cursor_of_another_group_than_its_kind_exits_1`; whether a page prints updates once needs ends on it, `inbox_prints_what_needs_the_party_then_updates` and `an_inbox_reads_on_in_updates_once_needs_ends` wait on O. The rest of 03.3 goes on, and 03.3 lands once its inbox follows O.

132. **temper 03.3: a limited activity's call that reads on, interim** (design, temper 03.3's block of 2026-10-19 09:33:47 UTC).
     Interim, until decisions.md, For the user, P, is decided: where the call that reads on goes after a page that `activity N --limit K` or `task N --activity K` cut is a design gap, and waits (jig's `Read::Activity { task, before }` takes no row count, and its page's `before` is older than the whole page; client/tools.md section 12 gives the rest of a window's cut "by its cursor", so no row is skipped); it goes to the user as P, as no designer is joined. temper 03.3 goes on with everything else, `activity N` without `--limit` and with `--limit` at or above the page's rows included; the call that reads on after a cut page, and its tests, wait on P, and 03.3 lands once they follow P and O.

133. **temper 03.3: the inbox's first page** (decision O, for temper 03.3's block of 2026-10-19 09:25:20 UTC).
     decisions.md, For the user, O, decided  for the user (temper 0ecb23bf6), the default, ruled into temper 03.3: `inbox` reads needs first; while needs has more, a page is needs alone; once needs ends on a page, updates' first page follows it when both fit in 32 rows, and when they do not, the call that reads on starts updates (group byte 1, no jig cursor), so no page passes 32 rows and no update is read twice. This ends ruling 131's wait; ruling 132's stands. temper/03-tools-root.md, 03.3's Shape, has it under Lists in place of "once per `InboxReadGroup`, needs first", with the test `an_inbox_page_whose_updates_do_not_fit_reads_on_at_the_start_of_updates` and a Size of at most 360 lines; later.md's row for N adds O for client/tools.md, 5.2. Any further copy of the first page's old shape (a fixture, a golden, a help line): refresh it as this says, and list it in the commit body.

     ## Checked

     - `git -C ~/src/rust/temper show 0ecb23bf6:docs/development/plans/tools/decisions.md | grep -n -A3 'Decision: the default, 2026-10-19: needs come first'`:
         901:  Decision: the default, 2026-10-19: needs come first, and
         902-  while needs has more, a page is needs alone. Once needs ends on a page,
         903-  updates' first page follows it when both fit in 32 rows. When they do
         904-  not, the call that reads on starts updates (group byte 1, no jig
     - `grep -n -F -e 'reads needs first' -e 'whose_updates_do_not_fit' -e 'at most 360' docs/docs/development/plans/tools/temper/03-tools-root.md | cut -c1-120`:
         218:  - `inbox` reads needs first (decisions.md, For the user, O, decided , temper 0ecb23bf6): while need
         248:`an_inbox_page_whose_updates_do_not_fit_reads_on_at_the_start_of_updates`,
         275:Size: at most 360 lines, with the inbox's cursor and first page (decisions.md, N and O).

134. **temper 03.3: a limited activity's call that reads on** (decision P, for temper 03.3's block of 2026-10-19 09:33:47 UTC).
     The decision was P (decisions.md, For the user, P, temper 37c08b325), the default: a page that `activity N --limit K` cuts reads on with the same read without `--limit`, which prints the page whole, its K rows again, then reads on at jig's `before`; `task N --activity K` that cuts a page prints `read with: activity N`; the printed cursor stays jig's. temper 03.3's Shape says so under "A task's parts" and "The conversation", its Test shape adds `a_limited_activity_reads_on_with_its_page_whole`, its Size is at most 380 lines, and later.md's row for client/tools.md 5.2 carries P. temper 03.3 lands once its reads-on do this; ruling 132's wait ends.

     ## Checked
     - `git -C ~/src/rust/temper show 37c08b325:docs/development/plans/tools/decisions.md | sed -n 929,932p`:
           Decision: the default, 2026-10-19: a page cut by `--limit
           K` reads on with the same read without `--limit`, which prints the page
           whole, its K rows again, then reads on at jig's `before`. `task N
           --activity K` that cuts a page prints `read with: activity N`. The
     - `grep -n 'read with: activity N\|without `--limit`\|a_limited_activity' docs/development/plans/tools/temper/03-tools-root.md`:
         234:    `read with: activity N` (decisions.md, For the user, P, decided , temper 37c08b325). `run N` is `--run`
         239:    without `--limit` (P): the page whole, its K rows again, then jig's `before`; the printed cursor stays jig's.
         255:`a_limited_activity_reads_on_with_its_page_whole`,

135. **temper 03.4: a restored proposal's recipient, interim** (design, temper 03.4's block of 2026-10-19 10:01:22 UTC).
     Interim, until decisions.md, For the user, Q, is decided: how a kept line of a restored proposal prints, when jig's `Outcome::Proposed { proposal }` keeps no recipient and client/tools.md, 6.1, has a kept line print "its outcome as an act's prints" (`proposed: p3/17, to maintainers`), is a design gap, and waits; it goes to the user as Q, as no designer is joined. temper 03.4 goes on with everything else, a new call's `Proposed` outcome with its offered recipient included; the kept line of a restored proposal, and its test, wait on Q, and never print a recipient the kept request does not hold. 03.4 lands once they follow Q.

136. **temper 03.4: a restored proposal's recipient** (decision Q, for temper 03.4's block of 2026-10-19 10:01:22 UTC).
     The decision was Q (decisions.md, For the user, Q, temper 37849f0b0), the default: a kept line of a restored proposal prints its identity alone (`kept: chat, proposed: t3/17`), with no `to`, as jig's `Outcome::Proposed { proposal }` keeps no recipient; a new call's outcome keeps `, to <recipient>`. temper 03.4's Shape says so under its outcomes, its Test shape adds `a_restored_proposals_kept_line_prints_its_identity_without_a_recipient`, its Size is at most 310 lines, and later.md has a row for client/tools.md 6.1 and 8.2. This ends ruling 135's wait; temper 03.4 lands once its kept line does this. Any further copy of a restored proposal's kept line (a fixture, a golden): refresh it as this says, and list it in the commit body.

     ## Checked

     - `git -C ~/src/rust/temper show 37849f0b0:docs/development/plans/tools/decisions.md | sed -n 983,985p`:
           Decision: the default, 2026-10-19: a kept line of a
           restored proposal prints its identity alone (`kept: chat, proposed:
           t3/17`), with no `to`. A new call's outcome keeps `, to <recipient>`.
     - `grep -n 'kept: chat, proposed\|a_restored_proposals' docs/development/plans/tools/temper/03-tools-root.md`:
         298:    identity alone, `kept: chat, proposed: t3/17`, with no `to` (decisions.md, For the user, Q, decided ,
         332:  `a_restored_proposals_kept_line_prints_its_identity_without_a_recipient` (Q),

137. **temper 04.1: an inbox read's digest when updates do not fit** (design, temper 04.1's block of 2026-10-19 10:32:17 UTC).
     Not a design gap: client/tools.md 5.4 answers it, and ruling 133 (decision O) stands. A digest is of "the subject's document as `--json` prints it", and "a read and a wait digest one form, the watch's document: for `inbox` and `tasks` ... the rows the watch's first window holds, in its order, and whether more follow, within the tools' page of rows. The read makes that form from its own rows and prints its cursor beside it, outside the digest"; 04's Choices has the read assemble "the watch's first window: needs before updates, each in its supplied order, at most 32 rows across both". So an inbox read that O's first page cuts before updates (31 needs and 2 updates) prints O's page, 31 needs, and digests the watch's first window all the same: its needs, then the first rows of the updates' page it read (`updates_fit` reads both pages), to 32 rows, and whether more follow, through `digest::part` and the one JSON renderer. The bytes digested are what a wait's first delivery prints, so a wait `--since` that digest does not answer at once; the read's printed rows are then fewer than the digested, which is not a second renderer or a hidden document. temper/04-waits.md, 04.1, says so in its Shape and adds `an_inbox_read_cut_before_updates_digests_the_watchs_first_window`; `the_printed_part_without_digest_is_the_bytes_digested` holds for each delivery and for a read whose page is the window. temper 04.1 goes on whole, inbox included.

138. **temper repin: jig repinned again past live-text's smith** (design, temper-repin's block of 2026-10-19 11:25:37 UTC).
     Not a question for the user: smith's live-text plan settles it (its README, "Repin": "jig and temper take this plan's skein and smith at the tools plan's closing jig-repin and temper-repin, with `live: None` in the start, an ignored `OpenEvent::Live`, and jig's four `smith_channel::v3` paths", from its decisions.md, answer 4, and "For the user" 9). Our `jig-repin` landed (jig 2f9c5f00, 08:28:45Z) before live-text's smith did (smith f925b180, 09:34:54Z, "Carry the host's live ask over channel version three", to cd76fb64), so jig's `main` pins smith 137a1cb4 and does not build at smith's `origin/main`. The supervisor adds a step, `jig-repin.s1` (development.md, 5.7, "The supervisor may add one earlier"): in a jig worktree at `origin/main`, it moves jig's skein and smith, together, to their `origin/main` (`cargo update -p smith-channel`, then the skein package), runs `cargo check --workspace --all-targets` and repairs what breaks the way live-text's own commits repaired smith's: `smith_channel::v2` becomes `smith_channel::v3` in `testing/jig-test-service/src/tests/fixtures.rs`, `tests/core/src/agents_tools.rs`, `tests/core/src/protocol.rs` and `tests/ops/src/conformance/scripts.rs`; each start takes `live: None`; `OpenEvent::Live` is ignored in `testing/jig-test-service/src/worker_route.rs`'s two matches, as `Opened` is. It lands with jig's gate, within 5.7's 200 lines and no change of design; a break past that is a question. `temper-repin` then starts again from temper's `origin/main`, moves skein, smith and jig to their `origin/main`, and makes the same two adaptations in temper (`crates/temper-service/src/worker_route.rs`, and each start). Refresh any further copy that the check names, and list it in the commit body.

139. **Done when 13: temper's fuzzy aim missed** (Done when run of 2026-10-19T11:52:47Z).
     Done when 13 misses one aim, for the user; the run is done either way, as
     the item says. Measured once at `origin/main` on debian-16gb-hel1-1,
     2026-10-19T11:52:47Z (done.md, "Done when, run 2026-10-19T11:52:47Z"):
     temper's fuzzy suite, 50 tests, took 8.790 s, 9.210 s and 9.681 s, a median
     of 9.210 s against the Pulse's 8 s. The other three aims are met: jig
     focused 5.404 s (7 s), jig fuzzy 6.993 s (8 s), temper focused 3.501 s
     (5 s).

     Its three slowest tests, from one more fuzzy run (7.860 s) at
     2026-10-19T11:57:17Z:
     - `temper-engine-domain-world::fuzzy_walking
       fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`,
       7.859 s;
     - `temper-conformance-world::fuzzy_shared
       a_changed_judge_is_crashed_at_every_commit`, 2.935 s;
     - `temper-conformance-world::fuzzy_stories
       a_ci_repair_is_crashed_at_every_commit`, 2.448 s.

     The reason: the suite runs no longer than its slowest test, and the first
     alone nearly fills the aim, so the suite is over it whenever that test runs
     slow. No step is added: the item records the miss rather than fixing it,
     and the retrospective weighs that test against the aim.

## For the user

The choices below have long-term consequences; each is the planner's,
and the user, may veto it before the run:

- **A. jig's 15 s budget, and what jig 04 can win** (11). The budget is
  a wall-clock time and depends on the machine: the run's machine
  (debian-16gb-hel1-1, 8 vCPU) runs jig's focused suite in 5.23 s
  already, carbon-forge (4 cores) in 14.4 s. So jig 04's proofs also
  read the slowest tests' own times before and after each fix
  (nextest's per-test times), which carry from one machine to the next.
  The profile (`findings/jig-test-profile.md`: 4,270 tests, 59.4 s
  summed) gives three fixes, about 2.5 to 3.6 s summed (page bounds
  reused, the vocabulary's capacity kept by its book, the drain's bound
  reused), then the restore to 15 s. The largest cause, the phrasebook's
  first load in each test process (about 2.7 s summed), is not fixed
  here: a cache across tests would cross the process boundary, and data
  prepared at build time needs a skein form for it (`later.md`). A veto
  keeps the wall-clock alone, or drops jig 04 and leaves the budget at
  30 s.
  Decision: accepted, 2026-10-19.
- **B. Two plans, and the real loop in the second** (7, 16). This plan
  builds section 13 and the tools as a library, proven by unit tests in
  temper and by jig's worlds and joint stories; `temper ctl`, the client
  stories, the fake forge's host, the real-loop world (domain/testing.md 2.4,
  client/tools.md 11.3) and its stories, and the process tier are the next
  plan's, drafted in `findings/next/`. The real loop, as the general
  layer the user asked for, is built there whole: its world, the hosted
  service, worker and agent, the scripted fake LLM, the fake forge's
  host over the existing fake forge, signed deliveries, and the cli host
  hosted per call. `later.md` lists what it still lacks to cover the
  rest of temper. Exceptions to "the real loop wherever it can": none
  of this plan's tools' proofs, because no real loop exists before the
  next plan; jig 01.3 and temper 01.1 (C) prove repins on the process
  tier, which only the executables show. A veto runs area 1 as one plan
  of about 68 steps with a longest chain of about 31.
  Decision: accepted, 2026-10-19. Two plans, with the real loop built whole in the second.
- **C. The process proofs, and where they run** (7). jig 01.3 drops a
  declared memory in jig's process tests; temper 01.1's process proof
  runs `a_native_chat_reaches_the_model`, which needs `TEMPER_FORGEJO`
  (the Forgejo binary's path). It is unset on the run's machine. Until
  the machine has it, temper 01.1's process proof runs where Forgejo is
  installed, or the supervisor rules it one layer down. A veto installs Forgejo on the run's machine before the run.
  Decision: vetoed, 2026-10-19. Forgejo 17.0.0, the pinned build (sha256 5a3b07e1…42ca), is installed on the run's machine, debian-16gb-hel1-1, with TEMPER_FORGEJO set in its login profile and Claude's environment, so temper 01.1's process proof runs there (the client-api retrospective's proposal 7: no Proof lands "unavailable").
- **D. The user's drafts** (temper `draft/llm-setup.md`, jig
  `draft/llm-accounts.md`): area 1 chooses nothing they must undo. The
  points that touch them:
  - tool names: area 1's are plain verbs; the accounts tools may want
    `remove`, `order`, `check` or `resume`, and area 1 takes none;
  - a secret never rides in a kept request: the token is a file of its
    own, never in the store (client/tools.md 8.1, the next plan's);
  - a call prints one result; a sign-in that waits for a browser (the
    drafts' OAuth on port 1455) would be a wait tool, not a change to
    that rule;
  - no config file (client/tools.md 8.1);
  - the named key (SHA-256 of `"temper-tools"`, a zero byte, the name)
    is generic: keyed account acts can reuse it.
  Decision: accepted, 2026-10-19.
- **E. jig's jobs read as call outcomes** (jig 01.2): "Started as job
  N", "Job N ended: exit C", "Not run: after a failed write"; jobs never
  reach temper's engine (`Request::Job` answered `Problem::Type`).
  Decision: accepted, 2026-10-19.
- **F. The repin absorbs smith's parity and wall-clock changes** (1):
  transcript v5, jobs, event vocabulary v3 and the larger shipped
  memory, into jig and temper, in jig 01 and temper 01.
  Decision: accepted, 2026-10-19.
- **G. The forge kit** (skein `draft/forge.md`, temper
  `draft/forge-kit.md`) (10): temper 11 moves the forge's words from
  temper-client-view to temper-client-domain and builds `adopt` on them;
  the kit, which would move the forge client, will rebase them. The
  README's Hotspots list each file and step. The fake forge's host is
  the next plan's.
  Decision: accepted, 2026-10-19. The forge kit's run follows this one's temper 11, on another machine, by its own hotspot list.
- **H. hearth's repin** (17): hearth has no checkout on the run's
  machine, and its own run on hel8 is paused. `hearth-repin` waits for
  a checkout and runs after jig-repin; if hearth's run resumes first,
  the two runs' pins meet in hearth's lock. A veto drops hearth-repin
  from this plan and leaves hearth on its own pins.
  Decision: accepted, 2026-10-19. hearth is cloned on debian-16gb-hel1-1; hearth's own run stays paused.
- **I. The aims are this machine's** (18): jig 7 8 and temper 5 8, from
  three runs each on debian-16gb-hel1-1. A run on another machine
  measures new base times first.
  Decision: accepted, 2026-10-19. The run goes on debian-16gb-hel1-1.
- **J. The order of `tasks` without `--order`** (question.md in the run's
  directory, from temper 02.1's ready check; no designer is joined to
  answer it): client/tools.md, 5.2, gives `tasks … [--ended] [--order
  activity|priority]` and no default, and jig's task query has no
  unordered form (`TaskQuery.order` is a `QueryOrder`: `Number`,
  `Changed`, `Activity`, `Priority` or `Ended`). Default: without
  `--order`, `tasks` reads by number, ascending (`QueryOrder::Number`),
  `--ended` changes no order, and the tools never send `Changed` or
  `Ended`. Or: a designer joins the run (`plan join designer`, in a
  design session) and answers question.md. temper 02.2 waits on it;
  02.1 goes on.
  Decision: the default, 2026-10-19: without `--order`,
  `tasks` reads by `QueryOrder::Number`, ascending, and `--ended`
  changes no order. The tools never send `Changed` or `Ended`, so
  `jig::argv::back` answers that a call with either order has no argv
  form, a refusal and never a guess, and `parse(back(call)) == call`
  holds over every call `parse` makes. client/tools.md, 5.2, says so at the
  designer's next pass over client/tools.md, after this plan.
- **K. The tools' rows pass jig's wire ceiling** (question.md in the run's
  directory, from temper 02.1's block of 05:31:30Z; no designer is
  joined to answer it): client/tools.md, 12, gives 50 rows, "a list's page, the
  activity lines `task` adds, a conversation's page; the client's rows
  per window", in the tools' own preset of `jig-client`'s `Limits`.
  jig's client wire (v8 and v9, `CEILINGS`) admits at most 32 in each of
  the 33 limits that take the client's rows, so the preset has no worst
  case (`temper_client_domain::worst_case` is `None`, and
  `jig_client::max_out` panics on it); at 32 rows it is
  `Some(91854289240)`. Default: the tools' rows are 32, the wire's
  ceiling, wherever client/tools.md, 12, and the plan say 50 (`limits::ROWS`,
  and the sessions that read it: temper 02, 03, 04 and 11), and client/tools.md
  says so at the designer's next pass, after this plan. Or: jig's wire
  admits 50 rows, a jig step and wire version this plan does not hold;
  or a designer joins the run (`plan join designer`, in a design
  session) and answers question.md. temper 02.1 waits on it, and 02.2,
  02.3 and temper 03 behind it.
  Decision: the default, 2026-10-19: the tools' rows are 32,
  jig's client wire ceiling, wherever client/tools.md, 12, and the plan say 50
  (`limits::ROWS` and the sessions that read it: temper 02, 03, 04 and
  11). A page of 32 rows serves the tools as well as 50, and changes
  later without a wire version; jig's wire stays as it is. client/tools.md, 12,
  says so at the designer's next pass, after this plan.
- **L. One group of tasks per call** (question.md in the run's directory,
  from temper 03.1's ready check; no designer is joined to answer it):
  client/tools.md, 5.2, gives `tasks … [--ended]`, and "a read answers with a
  document, or a page and its cursor" (client/tools.md, 5); jig's tasks are
  "live first, then ended, in two groups, each with its own cursor"
  (jig's client/domain/contract.md, 6.1), read as `Read::Tasks { query,
  live: Option<Cursor>, ended: Option<Cursor> }`, and temper 02's help
  says `--ended` is "Include ended tasks". One printed cursor cannot
  carry both groups' cursors. Default: one group per call: without
  `--ended`, the live group, `--after C` its `live`; with `--ended`, the
  ended group alone, as jig's view reads it (`query.phase =
  Some(PhaseClass::Ended)`, `live: None`, `--after C` its `ended`); the
  call that reads on keeps `--ended`, and its help reads "Only ended
  tasks"; client/tools.md, 5.2, says so at the designer's next pass, after this
  plan. Or: `--ended` keeps "include", and the printed cursor carries a
  group byte beside jig's; or a designer joins the run (`plan join
  designer`, in a design session) and answers question.md. temper 03.3's
  `tasks` waits on it; the rest of 03 goes on.
  Decision: the default, 2026-10-19: one group per call.
  Without `--ended`, `tasks` reads the live group, `--after C` its `live`
  cursor; with `--ended`, the ended group alone, as jig's view reads it,
  `--after C` its `ended` cursor, and the call that reads on keeps
  `--ended`. Its help reads "Only ended tasks". An agent that wants both
  makes two calls; the printed cursor stays jig's, with no group byte.
  client/tools.md, 5.2, says so at the designer's next pass, after this plan.
- **M. Offered acts with no tool** (question.md in the run's directory,
  from temper 03.2's block of 08:58:21Z; no designer is joined to answer
  it): client/tools.md, 6.1, prints "Acts as calls", and an agent "copies them
  from the acts a result printed" (5.3); temper 03.2 prints "one call per
  offered act in the document's order". jig's task documents offer 23
  kinds of act (`ActKind`), and 5.3's table has tools for Message,
  Answer, Accept, Reject, PassUp, LeaveHeld (`decide`), Release, Stop,
  Cancel and Adopt; Take, HandBack, Amend, Move, Reorder, Seen,
  EditNote, DeleteNote, ChangePolicy, SetRoles, MakeService, SetPool and
  Withdraw have none, so a call for them would name a tool that does not
  exist. Default: an offered act with no tool prints, in the document's
  order, its name in the tools' words and ` (no tool)`, and no call, as a
  refused act prints no call; its admission is not printed; JSON prints
  the document's offers as 03.2 says; client/tools.md, 6.1, says so at the
  designer's next pass, after this plan. Or: `acts:` leaves out an act
  with no tool; or the tools gain those acts (5.3 rows and table forms,
  a plan of their own); or a designer joins the run (`plan join
  designer`, in a design session) and answers question.md. temper 03.2's
  landing waits on it; the rest of 03.2 goes on.
  Decision: the default, 2026-10-19: an offered act with no
  tool prints its name in the tools' words and ` (no tool)`, in the
  document's order, with no call and no admission. JSON prints the
  document's offers as 03.2 says. The agent learns the act exists and can
  hand it to a person; no call names a missing tool. client/tools.md, 6.1, says
  so at the designer's next pass, after this plan.
- **N. The inbox's call that reads on** (question.md in the run's
  directory, from temper 03.3's block of 09:18:06Z; no designer is joined
  to answer it): client/tools.md, 5.2, has `inbox [--project P] [--kind K]
  [--root N] [--after C]` answer "what needs the party, then updates",
  and lists print "the cursor for the next page" (6.1). jig reads the
  inbox one group a call (`Read::Inbox { group: InboxReadGroup, .. }`):
  "the two groups carry independent opaque cursors", and "a kind belongs
  to exactly one of the two query groups" (`InboxReadGroup::of`). The
  table has no group selector, so the cursor printed after an inbox
  without `--kind` does not say which group it reads on. Default: an
  inbox's printed cursor is one leading byte naming its group (0 needs, 1
  updates), then jig's cursor bytes, kept whole; the call that reads on
  goes on where the page stopped, in needs while needs has more, then in
  updates; `--after C` reads C's group, and with `--kind K` a C of the
  other group is a usage error, exit 1; client/tools.md, 5.2, says so at the
  designer's next pass, after this plan. Or: `inbox` reads one group a
  call, as `tasks` does (L), with a flag for updates (a table change); or
  a designer joins the run (`plan join designer`, in a design session)
  and answers question.md. temper 03.3's inbox call that reads on waits
  on it; the rest of 03.3 goes on.
  Decision: the default, 2026-10-19: the printed cursor is
  one group byte (0 needs, 1 updates) and then jig's cursor, whole. The
  call that reads on stays in needs while needs has more, then moves to
  updates. `--after C` reads C's group, and with `--kind K` a C of the
  other group is a usage error, exit 1. The inbox stays the one call
  client/tools.md, 5.2, gives it, "what needs the party, then updates"; the
  cursor stays opaque to the agent. client/tools.md, 5.2, says so at the
  designer's next pass, after this plan.
- **O. The inbox's first page** (question.md in the run's directory,
  from temper 03.3's block of 09:25:20Z; no designer is joined to answer
  it): client/tools.md, 5.2, has `inbox` answer "what needs the party, then
  updates", and section 12 bounds "a list's page" at 32 rows. jig's
  `Read::Inbox` takes no row bound and answers one `InboxReadGroup` a
  call, each page up to the client's 32 rows; under N, the call that
  reads on stays in needs while needs has more. A first call that prints
  both groups' pages may print 64 rows, and while needs has more, the
  updates it printed are read again later. Default: a call reads needs
  first; while needs has more, its page is needs alone and reads on in
  needs; once needs ends on the page, the same call reads updates' first
  page and prints it after needs' rows when the two fit in 32 rows, and
  when they do not, prints needs' rows alone, its call that reads on
  starting updates (the group byte 1, with no jig cursor after it). Or:
  the call reads needs alone, and updates start at the next call (an
  empty needs prints no rows and that call); or a designer joins the run
  (`plan join designer`, in a design session) and answers question.md.
  temper 03.3's first inbox page waits on it; the rest of 03.3 goes on.
  Decision: the default, 2026-10-19: needs come first, and
  while needs has more, a page is needs alone. Once needs ends on a page,
  updates' first page follows it when both fit in 32 rows. When they do
  not, the call that reads on starts updates (group byte 1, no jig
  cursor). No page passes 32 rows, and no update is read twice. client/tools.md,
  5.2, says so at the designer's next pass, beside N.
- **P. A limited activity's call that reads on** (question.md in the run's
  directory, from temper 03.3's block of 09:33:47Z; no designer is joined
  to answer it): client/tools.md, 5.2, has `activity N [--before C] [--limit K]`
  answer "a page from its end", and `task N --activity K` its "K latest
  activity lines"; section 12 gives the rest of what the tools' window
  cuts "by its cursor". jig's `Read::Activity { task, before }` takes no
  row count: a page is up to 32 rows, with one `before` cursor older than
  the whole page, so printing K latest of its rows and reading on at
  `before` skips the rest of the page. Default: a page cut by `--limit
  K` reads on with the same read without `--limit` (`activity N`, or
  `activity N --before C` with the C it was given), which prints the
  page whole, its K rows again, then reads on at jig's `before`; `task N
  --activity K` that cuts a page prints `read with: activity N`; the
  printed cursor stays jig's. Or: the printed cursor wraps jig's with
  the count of rows printed, and reads on exactly where the page stopped
  (a cursor of the tools' own); or a designer joins the run (`plan join
  designer`, in a design session) and answers question.md. temper 03.3's
  `--limit` and `--activity K` calls that read on wait on it; the rest of
  03.3 goes on.
  Decision: the default, 2026-10-19: a page cut by `--limit
  K` reads on with the same read without `--limit`, which prints the page
  whole, its K rows again, then reads on at jig's `before`. `task N
  --activity K` that cuts a page prints `read with: activity N`. The
  printed cursor stays jig's. At most a page's rows repeat, and no row is
  skipped. client/tools.md, 5.2, says so at the designer's next pass.
- **Q. A restored proposal's recipient** (question.md in the run's
  directory, from temper 03.4's block of 10:01:22Z; no designer is joined
  to answer it): client/tools.md, 6.1, has an act's outcome print `proposed:
  p3/17, to maintainers`, and a kept line (8.2, step 4) print `kept: `,
  the operation, "then its outcome as an act's prints". jig's
  `Outcome::Proposed { proposal: ProposalKey }` keeps no recipient, and
  `OwnerOutcome::Proposed` only its number; the recipient comes with the
  live offer (`Admission::Proposed { to }`), which a restored request no
  longer has, and may have changed since. Default: a kept line of a
  restored proposal prints its identity alone (`kept: chat, proposed:
  t3/17`), with no `to`; a new call's outcome keeps `, to <recipient>`.
  Or: the kept line reads the proposal and prints its present recipient,
  labelled as such (`, now waiting on <recipient>`); or a designer joins
  the run (`plan join designer`, in a design session) and answers
  question.md. temper 03.4's kept line of a restored proposal, and its
  test, wait on it; the rest of 03.4 goes on.
  Decision: the default, 2026-10-19: a kept line of a
  restored proposal prints its identity alone (`kept: chat, proposed:
  t3/17`), with no `to`. A new call's outcome keeps `, to <recipient>`.
  The kept line prints only what jig keeps, so it is never stale, and an
  agent reads the proposal for its present recipient. client/tools.md, 6.1 and
  8.2, says so at the designer's next pass.
