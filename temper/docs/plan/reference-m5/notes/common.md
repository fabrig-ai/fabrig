# reference-m5: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree the task changes (`temper/AGENTS.md`,
  and `jig/AGENTS.md` where a task names a jig file). jig's core never links smith; jig never
  names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds,
  by path and section; read those, never a whole design. The one `docs/design/` edit a task
  makes is the removal of its own **Not yet** text from the status file its spec names (a
  stated departure from the implementer's text, whose "Not yours" otherwise holds); no
  other design, draft or `docs/help/` edit unless the spec names the file. Where the code
  teaches that the design is wrong, say so in the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/reference-m5/notes/contracts.md`:
  binding names, fields and rules. A task that needs a shape the note does not give asks.
- **Test worlds:** temper's `temper/tests/engine/...` worlds and the crates' `src/tests/`.
  Fakes, never real services: skein's fake forge for anything a forge answers, scripted
  agents for every agent. No test reaches a network, a held login or a real repository.
  Each test is one thread and one loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** change some of the same files on their own trunks:
  reference-m2 (`change`'s `mine`, a change on its requester's held branch) and reference-m3
  (a change's producer charter and LLM choices, the forge's charters as settings) in
  `temper-engine-smith/src/calls.rs`, `temper-engine-domain/src/boundary.rs`,
  `route/forge_route.rs`, `temper-engine-domain-forge-change` and the design sections
  `domain/agent.md` 4.3.1, `domain/forge.md` 8.1 and `protocol/configuration.md` section 13;
  every wave-2 plan in the reference's coordinator's charter. Keep each hunk to what the
  task needs: no reformatting, renaming or moving of code the task does not change; a new
  member goes beside the others, never in place of one.
- No new crates, workspace members or dependencies beyond the spec's.
- Store schemas and goldens: a task that adds a stored field follows temper's
  `protocol/store.md` rules for a format change and names the schema version it writes; a
  format change no spec names is a question.
- Old-form counts only fall (`<tree>/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check
  the count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
