# look-welcome-chats: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md`. Your task stays in the files its spec names; a
  reader file outside them is edited only as the spec says.
- **The requests** are the 11 commits on `origin/look-requests` over `5f3749c50c`
  (`git log -p origin/main..origin/look-requests`; never check the branch out, never push it).
  They stack: the state at its tip, `7fba5d0fe3`, is what is asked. Read the tip's pages with
  `git show 7fba5d0fe3:temper/docs/design/client/journeys/overlay.css` and
  `git show 7fba5d0fe3:temper/docs/design/client/journeys/welcome-playground/02.html`, and the
  ask against the trunk's print with `git diff HEAD 7fba5d0fe3 --
  temper/docs/design/client/journeys/welcome-playground/02.html`. The requests' base print equals
  the plan's first trunk's, so that diff is what is still asked, less what the spec says the
  print keeps otherwise.
- **The design is written by this plan's cut,** on the trunk with your spec: the sections your
  spec cites say what the code does. Design text, drafts and `docs/help/` are not yours.
- **Status files close with the code.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"): a task edits exactly the status entries its spec lists, and no other. A
  status file's form is `DESIGN.md`, 2: its date line today's (`date -u +%F`), only the gaps as
  they stand after your change. Remove the words your change closes and leave the rest word for
  word; a status left with no gap says `No gap.` under its date line.
- **The behaviour behind the switcher is another plan's** (`first-run-projects`, its step 3:
  `temper/docs/design/draft/first-run-projects.md`, 7): Rename in place and "+ New project" are
  not drawn by this plan. Never add a `temper-client-domain` act for them, never send jig's
  `Rename` or `MakeProject` from the pages.
- **No engine change,** and none in `temper-client-domain` unless the spec sketches it. The acts
  the pages bind are existing ones (`Act::Project`, `EndPlayground`, the composer's).
- **Generated files are regenerated, never edited by hand:** the journeys' pages, `skein-ui.css`
  and `journey.js` by `heavy cargo run -p temper-client-journeys` (reprint every journey;
  `temper/docs/design/client/journeys.md`, 5.1); goldens with `SKEIN_CODEGEN_WRITE=1` on the
  test that owns them. Read each regenerated diff by `git diff --stat`, then its hunks, and keep
  only what your change moves. A journey reported stale is a question: no recording changes here.
- **`overlay.css` stays empty** and the recordings and the journeys' `README.md` stay as they
  are, unless the spec says otherwise (`journeys.md`, 6).
- **The trailers:** the commit that carries a request's change to the trunk's print ends its
  message with that request's `Look-request: NAME` line, as the spec lists them, one line each,
  before the `Co-Authored-By` line if any. Never cherry-pick or merge a request's commit.
- **No budget, timeout or limit moves** (skein's `docs/foundation/testing-strategy.md`, 8.1).
- **Leave the reference case's files alone** (plans `reference-m2`, `-m3`, `-m5`, `-m6`,
  `-m7b`: the engine, the forge connector, ctl).
- **Old-form counts only fall:** `python3 -I -B temper/scripts/old_forms.py`; `--write` only to
  lower a count for what you removed. A rising count is yours to fix.
- **Tests:** after each change the narrowest test (`AGENTS.md`, 6); before the push, the focused
  tests of every package you changed and of its direct dependents (`heavy cargo nextest run -p
  ...`). Check each run's count: a run that selected nothing proves nothing.
- **Before the push,** when your branch changed any Markdown: `python3 -I -B
  tools/citations/citations.py --base origin/main` from the repository root, and fix what it
  lists in your files.
- One commit per coherent step, each saying why. Push your branch.
