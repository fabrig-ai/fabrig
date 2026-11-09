# ui-n3a-only: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, `temper/AGENTS.md`, your spec, and
  `temper/docs/plan/ui-n3a-only/notes/contracts.md`, which binds the shapes
  that cross tasks. Where a spec and the note differ, the note wins; say
  so in the report.
- **The request:** `temper/docs/plan/ui-n3a-only/inputs/request.md`, read
  whole (it is short). Its `For every task of this plan` list binds every
  task; the lines below repeat what bites here.
- **The design is written:** `temper/docs/design/client/README.md`, 3.5,
  **Only you**, and the rows and lines this plan's cut added beside it
  (3.5, *Needs you*'s *Only you* line and *A page's parts*' row; 5.5,
  *Each state*'s two expiring rows and the paragraph under the table),
  and `temper/docs/design/client/browser.md`, 7.2's four new states. Draw
  what they say and the engine holds, with no placeholder.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, a browser, process or live tier, or
  `scripts/browser.sh`. A test of those tiers your change must touch is
  edited and compiled (`heavy cargo check -p PKG --tests`), never run;
  say so in the report. No walk, render check, screenshot or throwaway
  driver: the plan is complete when it lands.
- **Tests stay cheap.** A new case of a behaviour is a row of an existing
  table test where one fits; a new test only where none does, beside its
  siblings' fixture. temper's focused and fuzzy suites have 30 and 120
  CPU seconds in the gate; the density fuzz keeps its seeds and steps.
- **Fakes, never the real thing:** the client domain's own tables
  (`temper/crates/temper-client-domain/src/tests/`), the view's
  (`temper/crates/temper-client-view/src/tests/`), and the client's joint
  world (`temper/tests/client`, package `temper-client-world`), whose
  `front::account_acts` fixtures drive temper's engine to a signed-in plan
  (`tests/suite/accounts_page.rs`, `signed_in`) with only the issuer
  faked.
- **Design files are the cut's,** with one departure each spec states: a
  task deletes the **Not yet** entry it closes from the status file it
  names, and touches no other design text.
- **Words and styles:** every new word is its own `Word` in its words
  table with a stable identifier, as its neighbours are; a style is a
  `look.rs` token and its rule in
  `temper/crates/temper-client-shell-web/temper.css` under
  `/* ==== ui: needs ==== */`, after the rules already there and before
  `/* ==== ui: work ==== */`, never at the file's end.
- **Generated files your change makes stale** (`temper/docs/help/reference/*`,
  goldens, prints, journeys, the gallery's `names.txt`, counted baselines)
  are yours to regenerate with their own tool in the same task, and to
  name in the report. On a catch-up, regenerate rather than hand-merge.
  `temper-client-journeys again` is generation, not a walk: run it for a
  journey your change leaves stale, retried as the request says on "a key
  mark caught a moving page".
- **A test's oracle or helper that the correct new behaviour trips** is
  yours to repair in the same task, keeping what it checks for everything
  else, named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **Decide, don't ask.** A public interface may change freely: no
  compatibility. A small edit just outside your files (a re-export, a
  `mod` line, a `pub(crate)` on a helper you reuse, a golden) is made and
  named in the report. A `Question:` is only for what truly blocks your
  acceptance.
- **Not to touch:** `temper/docs/design/**` but the status entry your spec
  names; `temper/crates/temper-client-shell-tui/**` and the terminal's
  inbox (`temper-client-view`'s `pages/inbox.rs`), which are temper/tui's;
  jig's crates; and every other task's owned files below.

| Task | Owns |
|---|---|
| `only-read` | `temper-client-domain`'s `pages/only_you.rs`, the Only you fields of `pages/parts.rs`, their tests |
| `accounts-expiring` | `temper-client-view`'s `pages/accounts/rows.rs` and `accounts/words.rs` for an expiring source, and their view tests |
| `only-region` | `temper-client-view`'s `pages/only_you.rs`, its call in `pages/needs.rs`, its words, styles and view tests |
| `only-story` | the joint world's `tests/suite/only_you.rs` and the density fuzz's Only you facts (`tests/suite/fuzzy/density.rs`) |
| `only-gallery` | `temper/tools/gallery`'s four states of `browser.md`, 7.2, and `names.txt` |
