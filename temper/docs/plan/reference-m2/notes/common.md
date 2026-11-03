# reference-m2: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md` (and `jig/AGENTS.md` if the task changes jig).
  jig never names temper.
- **The design is written.** Each task's `## Inputs` names the design sections it builds,
  by path and section; read those, never a whole design. The one `docs/design/` edit a task
  makes is the removal of its own **Not yet** text from the status file its spec names (a
  stated departure from the implementer's text, whose "Not yours" otherwise holds); no
  other design, draft or `docs/help/` edit unless the spec says so. Where the code teaches
  that the design is wrong, say so in the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/reference-m2/notes/contracts.md`:
  binding names, fields, words and rules. A task that needs a shape the note does not give
  asks.
- **Test worlds:** temper's own: the crates' `src/tests/` (the forge crate's pure domain
  tests, `temper-engine-domain-forge/src/tests/`), the engine worlds
  (`temper/tests/engine/domain/tests/suite/forge_routes/` for the root's stories,
  `temper/tests/engine/forge/tests/suite/` for the forge's). Fakes, never real services:
  skein's fake forge for anything a forge answers, smith's fakes for agents. No test
  reaches a network, a held login or a real repository. Each test is one thread and one
  loop, and a seed replays (`AGENTS.md`, 2).
- **Never run a live story:** no test of a `live*` or `proof_live` binary, with or without
  `--run-ignored`.
- **Other plans beside this one** change some of the same files on their own trunks: the
  wave-2 milestones (reference-m3, -m5, -m6, -m7b) add their own members to `change`
  (`ChangeCall`, the `change` decoder and schema in `temper-engine-smith/src/calls.rs`) and
  their own steps to the change procedure (`temper-engine-domain-forge`'s `domain.rs` and
  `boundary.rs`, the root's `route/forge_route.rs`); forgejo-connect changes the forge
  crate's `domain.rs`, `boundary.rs`, `connect.rs` and the root's `forge_route.rs`. Keep each
  hunk to what the task needs: no reformatting, renaming or moving of code the task does
  not change; a new member or arm goes beside the others, never reordering them.
- No new crates, workspace members or dependencies beyond the spec's.
- **No store format change:** this plan adds no stored field or record (temper's
  `protocol/store.md`). A task that finds it needs one stops and asks.
- Old-form counts only fall (`temper/scripts/old-forms.txt`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check
  the count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or test that runs it gives it `</dev/null`.
