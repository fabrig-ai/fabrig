# reference-m6: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`jig/AGENTS.md`,
  `temper/AGENTS.md`). jig's core never links smith; jig never names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds,
  by path and section; read those, never a whole design. The one `docs/design/` edit a task
  makes is the removal of its own **Not yet** text from the status files its spec names (a
  stated departure from the implementer's text, whose "Not yours" otherwise holds), with the
  status's date set to the day you push (`date -u`); no other design, draft or `docs/help/`
  edit. Where the code teaches that the design is wrong, say so in the report; never change
  the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/reference-m6/notes/contracts.md`:
  binding names, fields and rules. A task that needs a shape the note does not give asks.
- **Test worlds:** each tree's own: jig's `jig/tests/<area>/` worlds and its testing crates
  (`jig/testing/`), temper's `temper/tests/engine/...` and `temper/tests/worker/...` worlds
  and the crates' `src/tests/`. Fakes, never real services: skein's fake forge, temper's
  fake git (`temper-fake-git`, `temper-fake-checkout`), smith's fakes for agents. No test
  runs a real git, reaches a network, a held login or a real repository. Each test is one
  thread and one loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** (reference-m2, -m3, -m5, -m7b, configuring-p1) change some
  of the same crates on their own trunks: the change procedure
  (`temper-engine-domain-forge-change`), the forge connector (`temper-engine-domain-forge`),
  the root's `route/forge_route.rs`, temper's store schema, jig's `jig-domain-tasks` and
  `jig-domain-core`. Keep each hunk to what the task needs: no reformatting, renaming or
  moving of code the task does not change.
- No new crates, workspace members or dependencies beyond the spec's.
- Store schemas and goldens: a task that adds a stored field follows the store's own rules
  for a format change (jig's `jig/docs/design/protocol/store.md`, section 5; temper's
  `temper/docs/design/protocol/store.md`, section 4) and names in its report the schema
  version it writes; a format change no spec names is a question.
- Wire schemas: the worker wire (`temper-protocol-worker-wire`) changes in place at its
  payload version before the first release (`temper/docs/design/protocol/channel.md`,
  section 3), its generated code and goldens regenerated as its tests require.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check
  the count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
