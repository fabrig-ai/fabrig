# reference-m1: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`jig/AGENTS.md`,
  `temper/AGENTS.md`). jig's core never links smith; jig never names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds,
  by path and section; read those, never a whole design. The one `docs/design/` edit a task
  makes is the removal of its own **Not yet** text from the status file its spec names (a
  stated departure from the implementer's text, whose "Not yours" otherwise holds); no
  other design, draft or `docs/help/` edit. Where the code teaches that the design is
  wrong, say so in the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/reference-m1/notes/contracts.md`:
  binding names, fields and rules. A task that needs a shape the note does not give asks.
- **Test worlds:** each tree's own: jig's `jig/tests/<area>/` worlds and its testing crates
  (`jig/testing/`), temper's `temper/tests/engine/...` worlds and the crates' `src/tests/`.
  Fakes, never real services: skein's fake forge (`skein-fake-forge-*`) for anything a forge
  answers, smith's fakes for agents. No test reaches a network, a held login or a real
  repository. Each test is one thread and one loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** change some of the same crates on their own trunks
  (reference-m7a in `jig-domain-core`, `jig-domain-tasks`, `jig-protocol-store`;
  reference-m8 in `temper-engine-domain-forge`, `temper-engine-domain-forge-change`,
  `temper-protocol-store`, `temper-engine-domain`'s forge routes). Keep each hunk to what the
  task needs: no reformatting, renaming or moving of code the task does not change.
- No new crates, workspace members or dependencies beyond the spec's.
- Store schemas and goldens: a task that adds a stored field follows the store's own rules
  for a format change (jig's `protocol/store.md`; temper's `protocol/store.md`) and names the
  schema version it writes; a format change no spec names is a question.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check
  the count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
