# playground: status

2026-11-09. Where temper's code lags or departs from [playground.md](playground.md).


Section 9's walk of story A in the one-process world, by the terminal
client, from the welcome to the CSV change landed and the three replies
seen, runs (`temper/tests/one-process/tests/suite/story_a.rs`): the
world lays the story's disk and handovers through the playground world's
`temper_playground::story::prepare` (section 4), and walks it in 44.8
simulated seconds, its person waiting up to 60 s for each line it
looks for.

**Departure, 2026-11-12 (the user):** story A's kept log is no longer
checked at every landing, since every change to the engine's or the
client's wire made each landing record it again. The walk keeps its own
checks and writes the log only when asked to record; the three replays
of the kept log (`walk.rs`'s `story_a_kept_log_replays_natively`,
`replay.rs`'s `story_a_kept_log_replays_under_the_referee` and
`temper-playground`'s `story_a_start_makes_its_project_and_adopts_widget`)
are ignored, to be run by hand after the log is recorded again.

Section 9's replay clear of its byte bounds, when run by hand:
`walk.rs`'s `story_a_kept_log_replays_natively` replays the kept log
over story A's deployment as kept and with each `*_bytes` override of
its engine, its worker and its application doubled, and both reach the
log's end. It leaves out, by name in `doubled_byte_bounds`,
`checkout.name_bytes`, `forge.name_bytes` and `hub.event_bytes`, whose
doubling startup refuses, and `documents.text_bytes`,
`tasks.spec_bytes` and `tasks.result_bytes`, counted by payload alone.

- **Not yet:**
  - 4: the reset without the box: its fake forge's removal waits for the
    box (`domain/forge/adoption.md`, 1, and its status).
  - 6: the playground forge's links shown as text.
  - 7: released with temper: no release's build calls
    `scripts/bundle.sh --playground`, `scripts/release.sh` itself being
    `release.status.md`'s 5; the site and the help link no playground,
    and no private preview of `main` is served.
  - 8: the documents' capacities sized to story A: its deployment keeps
    the `small` profile with a few capacities raised, and `memory_bytes`
    of 2^60 for the engine and its worker
    (`temper/crates/temper-playground/stories/a/deployment.json`).
    `temper/docs/development/playground-memory.md` measures the world's
    native heap, by process, at its first step and its start alone, and
    the playground world's test holds them within a quarter (9);
    through the story there is only the browser tier's
    high water, which does not say what reserves it. The download:
    6,882,055 B gzipped, within its 8 MB
    (`temper/docs/development/playground-figures.md`).
  - 8, *Clear of the bounds that count slots*: story A's deployment
    keeps its byte bounds tight to the story (`core.load_bytes` 8192,
    `core.run_bytes` 16384 and others), none given room: the kept log's
    doubled replay (9) agrees, so the kept walk reaches none of them
    natively, but nothing shows the browser's walk, at its own pace,
    reaches none. The browser story's native replay of its downloaded
    log differed 6 times in about 180 runs on 11-07 and 11-08
    (`Differs { line: 524, .., why: Said }`); three runs of the story
    alone passed on 11-09, and the cause, a bound or another, is
    unknown. No test in a lower world fails for it. The next failure's
    `Differs` names the line's input, both digests and the replay's
    last lines (skein's `playground.md`, 6.2), and comes back as a
    repair.
  - 9: story A's high water in the one-process world:
    `temper/tests/one-process/tests/memory.rs` remains ignored: the
    story walks, but a service world's observer has no way to keep its
    storage outside the process's span (`skein/docs/design/testing/README.md`,
    5), so process 0's exact release check fails
    (`temper/docs/development/playground-memory.md`).
  - 10: the bar: story A by hand on Firefox and Safari, of which nothing
    records a walk; within its bars, the memory's and the slice's unset
    (section 11); and published.
    The page offers a failure's log
    (`temper/crates/temper-client-shell-web/src/playground/web.rs`), but
    no test fails a world in a browser: the tier downloads a whole
    walk's log from the frame, and that log replays natively.
- **Departs:**
  - 9: the kept log replays under the one-process world's forge referee
    alone (`temper/tests/one-process/src/replay.rs`, `Refereed`),
    because the referee's parts that watch the terminal client's person
    or keep the links have no place in a visit, and the log's digests
    prove the visitor's answers.
