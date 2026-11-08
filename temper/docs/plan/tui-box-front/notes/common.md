# tui-box-front: what every task follows

- **The request binds:** `temper/docs/plan/tui-box-front/inputs/request.md`,
  *For every task of this plan* first, and its hand-made frames
  (`inputs/box-front-01.txt` to `-05.txt`, `inputs/box-front-README.md`,
  `inputs/request-commit.txt`). It wins over a spec where they differ.
- **The contracts:** `temper/docs/plan/tui-box-front/notes/contracts.md`, the
  names and shapes that cross this plan's tasks. Build to them; a change to
  one is a question.
- **The trailer:** every commit of every task carries
  `Look-request: tui-box-front` directly above its other trailers
  (`temper/docs/design/draft/look-requests.md`, section 3).
- **Design:** each spec's `## Inputs` names the sections it builds. A task
  deletes the **Not yet** bullet that names it (by its task's name and
  `tui-box-front`) in the status file its spec names, and edits no other
  design text: that departs from the implementer's text, which closes
  `docs/design/`. The drafts (`docs/design/draft/*`) and `docs/help/` stay
  closed but where a spec says otherwise.
- **Runs:** focused and fuzzy alone, each build and test through `heavy`, one
  command at a time. No `tools/tiers/run`, no `<tier>-<tree>` profile, no
  `scripts/browser.sh`, no process, browser or live story run, no walk, no
  render check, no throwaway driver. A slow-tier story a task writes or
  changes is compiled (`heavy cargo check -p PKG --tests`) and named in the
  report. The temper crate's `suite` binary
  (`temper/crates/temper/tests/suite/`) is focused, and its box stories run.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing
  table test; a new behaviour may take one new test function, named in its
  spec. A failing test first for each behaviour; the report names the rule
  broken by hand and the failure seen (the implementer's text, *A new test
  must bite*). Budgets and timeouts are the user's, never raised.
- **Never shown:** a link's secret (the printed link's, the local link's)
  is never printed, logged or put in a panic or an assertion's message,
  but where the design draws it (the TUI's welcome draws the printed link).
  A test compares it by equality and `contains`, and names it in words
  ("the printed link"), never by its bytes.
- **`tui-launchpad-2` is in flight** in the same TUI files
  (`temper-client-tui`'s `view.rs`, `state.rs`, `view/launchpad.rs`,
  `view/foot.rs`, `theme.rs`, `look.rs`; `temper-client-shell-tui`'s
  `host.rs`, `run.rs`, `journey/**`; `temper/crates/temper/src/tui.rs`). Keep
  this plan's views in files of their own, and its hunks in shared files
  small and apart. Where `temper_client_tui::place` is on your base, the
  front's place is `place(address, true)`; where it isn't, the place is drawn
  as today and the report says so.
- **Before pushing:** `cargo fmt`; `python3 -I -B <tree>/scripts/old_forms.py`
  for each tree the branch touches (`--write` only to lower a count);
  `python3 -I -B tools/citations/citations.py --base origin/main`; clippy as
  the implementer's text says, naming every package changed.
- **Decide, don't ask** (the request): a stored format, a protocol or a public
  interface may change before v0.1.0, with no compatibility or migration; a
  small edit just outside a spec's files is made and named in the report.
