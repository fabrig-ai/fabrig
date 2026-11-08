# tui-launchpad-2: rules every task keeps

- **The request** is `temper/docs/plan/tui-launchpad-2/inputs/request.md`;
  the look request's message and diff are beside it
  (`request-commit-chat-title-below.txt`, `chat-title-below-04.diff`).
  Read them for the why; your spec and the design sections it names are
  what you build.
- **The contracts** that cross tasks are
  `temper/docs/plan/tui-launchpad-2/notes/contracts.md`: names, types and
  signatures there are binding. A change one of them needs is a
  `Question:`, never a quiet departure.
- **Only the focused and fuzzy suites run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or live
  tier story, nor a walk, render check or throwaway driver. A slow-tier
  test whose expected output your change makes stale is updated,
  compiled (`heavy cargo check -p PKG --tests`) and named in the report.
  Only the task `journeys` runs `temper-client-journeys again`.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test where one fits. Budgets and timeouts are the
  user's, never raised.
- **Generated files your change makes stale are yours to regenerate,**
  by their own tool, never by hand:
  - the TUI's goldens, `temper/crates/temper-client-tui/src/tests/golden/*.txt`:
    each is `Frame::print` of its screen; a mismatch prints the frame
    rendered, which you check against the design before writing it as
    the golden;
  - the TUI journeys' prints under
    `temper/docs/design/client/tui/journeys/` (`NN.txt`, `NN.html`,
    `theme.css`, `tui-journey.js`): `heavy cargo run -p
    temper-client-journeys` reprints both directories; then `heavy cargo
    nextest run -p temper-client-journeys` must pass. A recording, a
    README, a walk, a setup and `overlay.css` are sources: never yours.
  A diff in a generated file you cannot explain by your change is a red,
  not staleness.
- **The look request's trailer:** every commit of the tasks `rule-title`,
  `seam`, `chat-rule` and `journeys` ends its message's trailers with
  `Look-request: tui-chat-title-below`, directly above any other trailer
  (`temper/docs/design/draft/look-requests.md`, section 3, last
  paragraph). Other tasks' commits carry none.
- **Not yours:** `docs/design/` (but the generated prints above), the
  drafts, `docs/help/`, and `temper-client-view`'s `shared` module, which
  the web client shares: none of this plan's tasks changes it.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If main passes and the branch fails, it is
  the branch's.
- **The TUI's crates:** `temper-client-tui` (the TUI's view, state,
  events, theme: step code under the strict subset), `temper-client-shell-tui`
  (the host, the run, the journey module) and the binary's
  `temper/crates/temper/src/tui.rs`. A task's spec says which files of a
  crate it owns where two tasks share one crate in a wave; edit another
  task's files only as your spec's `Owns and may change` allows.
