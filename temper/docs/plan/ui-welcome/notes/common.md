# ui-welcome: the rules every task names

Read this note, then your spec. Where this note and the implementer's text differ, this note wins.
The request is `temper/docs/plan/ui-welcome/inputs/request.md`; your spec cites its sections.

## Decide, don't ask

The user is asleep and takes the request's judgement. There is no supported deployment: a stored
format, a protocol or a public interface may change in place, with no compatibility and no
`Question:`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design
line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly
blocks the task.

## Departures from the implementer's text

- **Design text:** the cut wrote the design. A task may edit only a `docs/design/` line that names what
  it builds: the **Not yet** entry it closes in the status file its spec names, a table row or a word
  its spec names, and the journeys' pages it reprints. A draft's own text stays as it is.
- **Generated files** your change makes stale (`docs/help/reference/*`, prints, goldens, counted
  baselines) are yours to regenerate in the same task.
- **A throwaway render script** is allowed in any task, untracked and gone before the push.
- **`docs/help/`:** a line naming what your task changes is yours; nothing else.

## The mock

It is the target: `temper/docs/design/client/greenfield/`, `app.js` and `mock.css`. Reach for it, not
for the old page; where an old test walks a part the mock removes, change the test with it.

- Its first run is a mode in localStorage: set `temper-greenfield-mode` to `first-run` and
  `temper-greenfield-first-run` to `{"setup":{"forge":…,"model":…}}` in an inline script, or press
  "Start the first run" on `welcome.html`.
- **Skip,** whatever the mock draws: every `Mockup:` line and `.mock-inline`, GitHub's `.away-note`,
  `notFirstRun`, `mockCorner`, `phone.html`, GitHub's repository count on the forge's line and Home,
  the playground's pages.
- **Its first project's "Default" is superseded:** Ready offers "Hello" or "My project". Copies run at
  Ready, never in the forge's steps: no "Copying…" on Model's pane and no "N repositories copied" on the
  forge's line. "Open Forgejo ↗" stays on Ready's forge line. The forge's pane on Model stays.
  `to-begin.md` overrides `freshHome()`: no count headline, no Connected, "Skip" stays.
- **Kept though the mock doesn't draw them:** failed set up, a forge cut short, declined, held, refused,
  every paste ending, the port taken. Their words stay.
- **Don't restyle what matches:** Welcome, Forge's cards, the sheet and Ready's form match the mock to
  the pixel. What this plan adds is states and acts.

## Rules of the page

- **`temper.css`:** new rules go under `/* ==== ui: welcome ==== */`, never at the end of the file;
  replaced rules are removed where they stand. A rule for Home goes under the same marker, scoped to its
  page. Each class you draw goes in `temper-client-view/src/look.rs` (`look::NAMES`), as its neighbours
  do, its rule scoped to `#app:has(> .temper-welcome)` where the welcome's scale is meant.
- **The mock's class is not the client's:** `.forge-status`, `.away-note`, `.panel`, `.acts`, `.cmd`,
  `.numbered`, `.facts`, `.checklist` name the mock's rules. The client's are `temper-` names carrying the
  same declarations at the welcome's `--gf-*` tokens; the first task that needs one adds it, its name
  in `notes/contracts.md`.
- **The terminal keeps its meaning:** the view is shared. `skein-ui-text` prints a named region as
  `NAME:` with its children indented and a paragraph's text runs joined with no separator, so a part
  stacked or marked by style says it in the terminal's words: the view reads `w.shell`, as
  `forge/connect.rs`'s `stacked` does. A spinner has no terminal form: its words alone print. Goldens move
  with reasons, named in your report.
- **A new word** goes in `words.rs` as its neighbours: the `Word` enum, the list, the next unused number,
  its English. The list's bounds in `limits.rs` are derived: raise a derived count by what you add, with
  its comment, and say so. It is not a budget.
- **Standalone `ConnectForge` and `ConnectModel` keep everything:** a change for the welcome branches on
  `root.pages().address()`'s `Welcome` or the view's `Place::Welcome`, never in their shared parts.
- **No `Look-request:` trailer:** nothing here lands a look request by name.

## Gallery and journeys

- **Goldens, prints, `temper/tools/gallery/names.txt` and `tests.rs` conflict across plans:** on a
  catch-up, regenerate rather than hand-merge, and keep `names.txt` in its order, a new state beside its
  neighbours. Add a state to `temper/docs/design/client/browser.md`'s table of states (7.2) in the same
  commit.
- **A page's markup changing reprints:** `heavy cargo run -p temper-client-journeys` reprints every
  journey in place; commit the pages it changed. A recording goes stale only if the exchange with the
  engine changes (`temper/docs/design/client/journeys.md`, 8). A task that stales one runs `again NAME`;
  if the walk itself changed, it says so in its report and leaves it to `journey-landing`.
- **`welcome-set-up` adds nothing past Model:** it stays within `again`'s 40 seconds.

## Render proofs

For every page and state your spec names, render beside the mock's page at 1440, 1000 and 420 wide,
plus its open states. Screenshots go on your builder in `~/src/fabrig/ui-welcome/`, outside any worktree,
never committed; your report names the builder.

- **Chromium 147's new headless won't go below 500 px by `--window-size`:** a 420 px proof uses CDP's
  `Emulation.setDeviceMetricsOverride`, or an iframe of that width, or the media queries lie.
- **Handle `Page.javascriptDialogOpening`,** or a pending request's `onbeforeunload` hangs a CDP driver.
- **Fonts:** compare with the client's bundled Plex, loading the mock's from a local copy.
- **The gallery's states are the cheapest render:** `heavy cargo run -p temper-client-gallery`.

## Your checks

Besides the implementer's text: a task that touched design text runs
`python3 -I -B tools/citations/citations.py --base origin/main` from the repository root. Markdown read
by tests (`.config/gate/read-by-tests.txt`) runs its tests. A word asserted by an old test moves with
the test; `rg` finds them before you run them.

## Other plans beside you

`ui-home-status` also touches Home; `claude-always/fetching-words` touches
`temper-client-view/src/accounts/words.rs` and `connect_model`'s tests; `ui-signin` owns
`temper-fake/src/oauth.rs` and `walk/sign_in.rs`. Leave what is theirs. A catch-up keeps both sides.

## Your report

Besides what the implementer's text asks: for each state or rule of the request your task landed, one
line, and the goldens you moved and why.
