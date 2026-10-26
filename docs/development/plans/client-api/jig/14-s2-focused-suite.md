# Supplement jig-14.s2: jig's focused suite cheaper, beside jig 14

Added by ruling 269 (the steer of 2026-10-17T16:16Z). Read first: jig's
`AGENTS.md`, `docs/development/workflow.md`; this plan's README, "Rules of
this plan" ("Test time") and "The run's steers".

Scope: README "Test time": "the session's next increment first makes the
slowest tests cheaper, found once by the measure run of the suite that
missed (...), until its own gate is back under the aim". jig's focused
gate ran 12.204, 13.653, 13.945 and 14.248 s at 14.1, 14.2, 14.3 and 14.s1
(aim 13.3 s; stop 15 s, a rule). This step makes that repair for jig, in a
slot of its own, so that jig 14's increments do not.
Needs: nothing; it runs beside jig 14.4.

Files: the tests, fixtures and test helpers of the slowest tests its
measure run names; no production code. Until jig 14.4 lands, not the five
files of 14.4's repair (`5e4afc0c8`: `tests/channel/src/fixtures.rs`,
`tests/channel/src/processes.rs`, `tests/client-protocol/src/native/config.rs`,
`tests/ops/src/client/app.rs`, `tests/ops/tests/client_stories.rs`); after
it lands and you rebase, they are in scope too.

Shape:
1. Once, at the start, on your worktree at `origin/main`: the focused
   measure run, `cargo nextest run --workspace --profile
   measure -j 1`, its log in `logs/jig-14-s2/measure-start.log`. 14.3's
   (`logs/20261017T151241-664.log`) led with 0.312, 0.274, 0.266, 0.260
   and 0.248 s; its 52 tests of 0.1 s or more summed 8.6 s of 37.8 s.
2. Slowest first, make each test cheaper where its own time goes: a world
   or fixture built more than once, more virtual-clock steps or rounds than
   its state needs, limits larger than its story needs (the protocol
   binary's `journal()`, `tests/client-protocol/src/fixtures.rs:28`, raises
   the journal to at least 2,000 writes and held; the native world takes
   the small profile with overrides, `native/config.rs:3`). A test that
   needs a limit other than its profile's names it beside the profile's,
   with the reason (development.md, 5.6).
3. Stop when the tests you changed sum at least 4 s less in a second
   measure run (`logs/jig-14-s2/measure-end.log`), or at 90 minutes,
   whichever comes first. In the gate, 1 s of serial test time is about
   0.37 s of wall: 4 s is about 1.5 s.

Proof: the gate. Aim for its focused suite under 13.3 s, a guide: if it
is missed, the landing stands and the report names the gate's time and its
three slowest tests. The report names each test changed with its two
measure times, and its gate's suite times from `Gate-Result`; nothing else
is measured.
Size: tests and fixtures only, aim for at most 300 changed lines.

Don't: drop or weaken an assertion, case, seed or story; merge or rename
tests; touch production code, an `opt-level` or `.config/nextest.toml`
(README "Test time", Never: no `global-timeout`, slow period or per-test
ceiling raised; no focused test moved to the fuzzy suite; no thread count
changed).
