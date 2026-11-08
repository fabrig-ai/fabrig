# docs-read-globs: the web journeys' files are test inputs

Since agents b5fd5b8c (the user's word, 11-08: "mockups (all of them, any format, including JS code) are treated as docs, as long as they live under docs/"), fab's queue selects any file under a `docs/` directory as docs unless `.config/gate/read-by-tests.txt` lists it. Before, every non-Markdown file under `docs/` ran the full tests, so the list names mostly Markdown. One directory of non-Markdown test inputs under `docs/` is missing from it: `temper/docs/design/client/journeys/`. Its CSS, JS, recordings, setups, pages and READMEs are read by `temper/tools/journeys/src/tests.rs` (lines 13, 219, 227) and by `temper-client-shell-web`'s journey tests (`src/journey/redact/tests.rs:351`, `scan/tests.rs:334`, `stream/tests.rs:122`, `tests/suite/host/redacted.rs:239,241`).

Two changes to `.config/gate/read-by-tests.txt`, nothing else:
1. Add `temper/docs/design/client/journeys/**`, beside the existing `temper/docs/design/client/tui/journeys/**`.
2. The header line becomes: inputs under `docs/` (any format) or elsewhere that tests compile in or read; every non-Markdown input under `docs/` must be listed, since the queue takes the rest as docs.

One direct task, no survey. No test is added or run for it beyond the plan's focused and fuzzy runs.
