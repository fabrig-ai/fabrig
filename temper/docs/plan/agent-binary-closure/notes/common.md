# agent-binary-closure: rules every task keeps

- **The request** is `temper/docs/plan/agent-binary-closure/inputs/request.md`:
  deployment-binaries M4, the agent's binary, is carried onto main in one
  commit, and meeting M2's code it turns two boundary tests red,
  `tests/boundary/tests/binaries.rs` (the engine's row) and
  `tests/boundary/tests/closure.rs` (the agent's). Read it for the why;
  your spec and the design sections it names are what you build.
- **Move code; never loosen a row.** The dependency rule is the user's
  (`temper/docs/design/draft/deployment-binaries.md`, section 4). No task
  adds an exception to a row of either test, removes an excluded crate or
  a tree, or adds a crate to a row's allowed temper crates beyond what its
  spec names (`notes/contracts.md`).
- **No new Cargo feature,** in any manifest (the request's "Not in this
  plan"). smith's existing `fixture` features stay as they are.
- **The boundary tests run here:** they are focused tests of the package
  `fabrig-boundary` and read `cargo tree --locked --offline`, so
  `Cargo.lock` must be committed as your build leaves it:
  `heavy cargo nextest run -p fabrig-boundary --test binaries` and
  `heavy cargo nextest run -p fabrig-boundary --test closure`.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a process, live or
  browser story, a walk, a render check or a throwaway driver.
- **Tests stay cheap:** no new test file and no new test binary; the
  boundary tests' rows are the oracle. Budgets and timeouts are the
  user's, never raised.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none`
  (the machines' cargo config sets it). `cargo fmt`, `cargo tree` and
  `cargo metadata` build nothing and run directly.
- **Your status entries:** each spec names the **Not yet** entries in
  `temper/docs/design/**/*.status.md` that are its own. Once your
  acceptance holds, delete those entries' lines, and nothing else under
  `docs/design/`. That departs from the implementer's text, which closes
  `docs/design/` to you: the entries are your task's, and the cut wrote
  the design they lag. Leave the blank lines around them as they are.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base fails too, it's not
  yours: list it under `## Left`.
- **Decide, don't ask.** A small edit just outside your file list, of the
  kinds your spec pre-approves, is made and named in the report. A
  `Question:` is only for what truly blocks your acceptance.
