# configuring-p3: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`jig/AGENTS.md`,
  `temper/AGENTS.md`, `smith/AGENTS.md`). jig's core never links smith; jig never names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds, by
  path and section; read those, never a whole design. The one `docs/design/` edit a task makes
  is the removal of its own **Not yet** text from the status file its spec names (a stated
  departure from the implementer's text, whose "Not yours" otherwise holds); no other design,
  draft or `docs/help/` edit unless the spec names the file. Where the code teaches that the
  design is wrong, say so in the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/configuring-p3/notes/contracts.md`.
  The design wins on detail, the contracts on names. A task that needs a shape neither gives
  asks.
- **On main's code, never a second copy of it:** jig's settings, revisions, proposals and
  application settings (jig's `domain/engine.md`, section 17), temper's kinds as settings,
  the settings form's items of them, `chat start --template`, `proposal show` and `decide`,
  `settings show`, `revision show|list|advance|restore` are on main (plans `reference-m3`,
  `configuring-p1`, `configuring-p2-build`). A task extends them; it never writes its own.
- **Test worlds:** each tree's own: jig's `jig/tests/<area>/` worlds and its testing crates
  (`jig/testing/`), temper's `temper/tests/...` worlds and the crates' `src/tests/`. Fakes,
  never real services. No test reaches a network, a held login or a real repository. Each test
  is one thread and one loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** (`public-reading-build`, `playground-browser`) change some of
  the same crates on their own trunks: `jig/crates/jig-protocol-config/src/**`,
  `temper/crates/temper-service/src/limits.rs`, `temper/crates/temper-service/src/clients/**`,
  `temper/crates/temper-client-tools/**`, `temper/docs/design/client/tools.md`, the forge crate's
  `public` modules, `temper/tests/one-process/**`, `temper/tests/client-browser/**`. Leave their
  files alone unless your spec names one; where it does, keep each hunk to what the task needs:
  no reformatting, renaming or moving of code the task does not change.
- No new crates, workspace members or dependencies beyond the spec's.
- Store and wire schemas and goldens: a task that adds a stored or wired field follows the
  format's own rules (jig's `protocol/store.md`; the wire's schema README and generator) and
  changes the schema version its spec names, in place (no store or wire before the baseline
  is upgraded: `temper/docs/design/draft/upgrades.md`, 4.1); a format change no spec names is a
  question.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
- Before pushing a branch that touches Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` and fix what it lists in the
  files you own.
