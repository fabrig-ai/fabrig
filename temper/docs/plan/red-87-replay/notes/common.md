# red-87-replay: rules every task keeps

- **The request** is `temper/docs/plan/red-87-replay/inputs/request.md`:
  an after-landing red, the browser story
  `browser_playground::story_a_walks_on_the_built_playground`
  (`temper/tests/client-browser/tests/browser_all/browser_playground.rs`),
  whose native replay of the downloaded session log now and then fails
  with `Differs { line: 524, .., why: Said }`. Read it for the why; your
  spec and the design sections it names are what you build.
- **The story keeps its replay check.** It changes only where a spec
  says so; its walk, its figures and its `panic!("{differs:#?}")` stay.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a process or live tier
  story, a tier's CPU figure, a walk, a render check or a throwaway
  driver. The one exception is a spec that names it: the story above,
  by name and alone, a few times, reading no CPU figure.
- **No budget, tier or timeout changes.** Tests stay cheap: a new case of
  one behaviour is a row of an existing test or table, not a new world.
- **Every build and test through `heavy`,** with `TEMPER_BUNDLE=none` (the
  machines' cargo config sets it). `cargo fmt`, `cargo tree` and `cargo
  metadata` build nothing and run directly.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once alone on the pristine base. If the base passes and the branch
  fails, it's the branch's.
- **Decide, don't ask.** A small edit just outside the file list is made
  and named in the report. A `Question:` is only for what truly blocks the
  task.
- **Effort medium,** as the request asks: the shapes below are settled;
  spend the run on the code and its tests.
