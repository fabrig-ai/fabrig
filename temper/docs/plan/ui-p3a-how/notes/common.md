# ui-p3a-how: what every task follows

- **The request binds:** `temper/docs/plan/ui-p3a-how/inputs/request.md`, all of
  it, *For every task of this plan* first. It wins over a spec where they differ.
- **The design** this plan follows was written by its cut, on the trunk before any
  task starts: `temper/docs/design/client/README.md`, 5.13 (*The page*, *Its
  parts*, *Its rows*, *A way's acts*) and section 11 (*How work runs' later
  parts*); `temper/docs/design/client/browser.md`, 2.5 (*How work runs' look*) and
  7.2 (the states `how` to `night-how`). Its look is the mockups
  `temper/docs/design/client/web/whole-story/how.html`, `pair/how.html` and
  `whole-story.css`: their words, parts, order and look bind, their figures don't.
- **Draw what exists** (`temper/docs/design/draft/whole-story.execution.md`,
  section 2): What reaches you, Rules by path, What follows a landing and a role's
  budget are later milestones'. No task draws them, an empty section for them, a
  placeholder or a made-up value.
- **Status:** each task deletes the **Not yet** entry its spec names, in
  `temper/docs/design/client/README.status.md` or `browser.status.md`, and edits
  no other design text: that departs from the implementer's text, which closes
  `docs/design/`. What the code taught goes in the report.
- **Keep off the drafts beside this one:** `ui-p1-rail` owns the rail
  (`temper-client-view`'s `frame.rs`, `temper-client-domain`'s `pages/rail.rs`),
  Work, Deployment's pages and `settings.rs`'s frame and tabs; `ui-p2-home` owns
  Home and Needs you. This plan meets the rail at its "How work runs" row, which
  opens `Settings { project }`: no task changes the row, `Address` or a route.
  Where a spec allows an edit in a shared file (`pages.rs`, `settings.rs`), it is
  that edit alone.
- **Words and styles** (`temper-client-view`): new phrase keys come from 4900 to
  4949, each variant added in the four places `words.rs` keeps it, beside the
  ways' words (`Word::WaysTitle`'s), never at a list's end. New style tokens are
  appended to `look.rs`'s `NAMES` with their constants beside the ways' (`WAYS_*`),
  named as `notes/contracts.md`, C3, names them; a catch-up that meets another
  plan's appended tokens renumbers this plan's, never theirs.
- **`temper.css`:** this plan's rules stand under `/* ==== ui: how ==== */`,
  which the first task to write one adds directly after the last
  `temper-ways-` rule under `/* ==== ui: settings ==== */`; never at the file's
  end. Rules a change replaces are removed where they stand.
- **Tests stay cheap:** a new case is a row of an existing table test, or a row of
  the one new table test a spec names; no other new test function. The gallery is
  over its budget (`browser.status.md`, 7.3): add only the states a spec names.
- **Generated files** a change makes stale are that task's to regenerate by their
  own tool, never by hand: the text shell's goldens
  (`temper-client-shell-terminal/src/tests/golden/`), the journeys' prints
  (`heavy cargo run -p temper-client-journeys`, then `again NAME` for a recording
  it reports stale, retried as the request says), the gallery's `names.txt`.
- **Runs:** focused and fuzzy alone, each build and test through `heavy`, one
  command at a time. No `tools/tiers/run`, no `<tier>-<tree>` profile, no
  `scripts/browser.sh`, no walk, render check, screenshot or throwaway driver. A
  slow-tier crate a task touches is compiled alone (`heavy cargo check -p PKG
  --tests`).
- **Before pushing:** `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
  `python3 -I -B tools/citations/citations.py --base origin/main` (it reads
  committed work: commit first); clippy as the implementer's text says, naming
  every package changed.
