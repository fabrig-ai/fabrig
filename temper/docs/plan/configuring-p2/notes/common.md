# configuring-p2: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`jig/AGENTS.md`,
  `temper/AGENTS.md`). jig's core never links smith; jig never names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds, by
  path and section; read those, never a whole design. The one `docs/design/` edit a task makes
  is the removal of its own **Not yet** text from the status file its spec names (a stated
  departure from the implementer's text, whose "Not yours" otherwise holds); no other design,
  draft or `docs/help/` edit unless the spec names the file. Where the code teaches that the
  design is wrong, say so in the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/configuring-p2/notes/contracts.md`,
  and, for what this plan builds on, `temper/docs/plan/reference-m3/notes/contracts.md`.
  A task that needs a shape neither gives asks.
- **On M3's code, never a second copy of it:** temper's kinds as settings, the settings form's
  items of them, `settings show landing-rule` and `forge-charter`, and the forge's
  `ProposalItems` read are the reference's M3's (`temper/docs/plan/reference-m3/plan.md`).
  A task extends them; it never writes its own.
- **Test worlds:** each tree's own: jig's `jig/tests/<area>/` worlds and its testing crates
  (`jig/testing/`, `jig-test-connector` for an application's kinds), temper's
  `temper/tests/engine/...` worlds and the crates' `src/tests/`. Fakes, never real services. No
  test reaches a network, a held login or a real repository. Each test is one thread and one
  loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** change some of the same crates on their own trunks
  (reference-m5, -m6, -m7b: `temper-engine-domain`'s `route/forge_route.rs`, the forge crates,
  temper's store; first-run-projects: `temper-init`, `temper-service`'s stock). Keep each hunk
  to what the task needs: no reformatting, renaming or moving of code the task does not
  change. A conflict with what such a plan landed on main reaches this trunk by catch-up, not
  by you.
- No new crates, workspace members or dependencies beyond the spec's.
- Store schemas and goldens: a task that adds a stored field follows the store's own rules for
  a format change (jig's `protocol/store.md`; temper's `protocol/store.md`) and names the
  schema version it writes; a format change no spec names is a question.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
- Before pushing a branch that touches Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` and fix what it lists in the
  files you own.
