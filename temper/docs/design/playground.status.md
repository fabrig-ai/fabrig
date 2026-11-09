# playground: status

2026-11-09. Where temper's code lags or departs from [playground.md](playground.md).

Section 9's walk of story A in the one-process world, by the terminal
client, from the welcome to the CSV change landed and the three replies
seen, runs (`temper/tests/one-process/tests/suite/story_a.rs`): the
world lays the story's disk and handovers through the playground world's
`temper_playground::story::prepare` (section 4), and walks it in 44.8
simulated seconds, its person waiting up to 60 s for each line it
looks for.

- **Not yet:**
  - 4: the reset without the box: its fake forge's removal waits for the
    box (`domain/forge.md`, 4.1, and its status).
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
    keeps byte bounds tight to the story (`core.load_bytes` 8192,
    `core.run_bytes` 16384, `documents.text_bytes` 128 and others),
    some counted with `size_of` of their slots, and nothing shows
    whether the story reaches one natively; the browser story's native
    replay of its downloaded log differs now and then (red-87).
  - 9: story A's high water in the one-process world:
    `temper/tests/one-process/tests/memory.rs` remains ignored: the
    story walks, but a service world's observer has no way to keep its
    storage outside the process's span (`skein/docs/design/testing.md`,
    5), so process 0's exact release check fails
    (`temper/docs/development/playground-memory.md`).
  - 9, *Clear of its byte bounds*: no test replays the kept log over
    story A's deployment with its byte bounds doubled.
  - 10: the bar: story A by hand on Firefox and Safari, of which nothing
    records a walk; within its bars, the memory's and the slice's unset
    (section 11); and published.
    The page offers a failure's log
    (`temper/crates/temper-client-shell-web/src/playground/web.rs`), but
    no test fails a world in a browser: the tier downloads a whole
    walk's log from the frame, and that log replays natively.
- **Departs:**
  - 6, 9: the page's code builds on wasm32 with or without the feature
    `playground`, which chooses the module's entry alone
    (`temper-client-shell-web`'s `src/lib.rs` and
    `src/playground/web.rs`), because the gate's list holds packages, not
    features: so the gate lints the page's code, but not the shell with
    its feature, which `scripts/bundle.sh --playground` alone builds.
  - 9: the kept log replays under the one-process world's forge referee
    alone (`temper/tests/one-process/src/replay.rs`, `Refereed`),
    because the referee's parts that watch the terminal client's person
    or keep the links have no place in a visit, and the log's digests
    prove the visitor's answers.
