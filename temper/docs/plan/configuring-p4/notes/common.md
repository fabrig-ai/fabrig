# configuring-p4: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md`. This plan writes one new tool crate,
  `temper/tools/stage-one` (package `temper-stage-one`); it changes no crate of temper's binary.
- **The design is written.** Each task's `## Inputs` names the design sections it builds, by
  path and section; read those, never a whole design. The one `docs/design/` edit a task makes
  is the removal of its own **Not yet** text from `temper/docs/design/domain/testing.status.md`
  (a stated departure from the implementer's text, whose "Not yours" otherwise holds); no other
  design, draft or `docs/help/` edit. Where the code teaches that the design is wrong, say so in
  the report; never change the design to fit.
- **The shapes that cross tasks** are `temper/docs/plan/configuring-p4/notes/contracts.md`.
  The design wins on detail, the contracts on names. A task that needs a shape neither gives
  asks.
- **Held by the acceptance, never by the binary.** Nothing of the golden, the check or the
  judge goes into `temper`'s binary, its stock, `temper/docs/help/` or any crate the binary
  links (`temper/docs/design/domain/testing.md`, 6.5). `temper-stage-one` may depend on
  temper's crates; no crate of temper's may depend on it.
- **The reference profile is read, never changed:** `temper/docs/examples/reference/*.json` are
  the golden's values. A task that finds the profile disagreeing with section 14 reports it
  under `## Left`; it does not edit the profile.
- **Tests:** the crate's own, in `temper/tools/stage-one/src/tests.rs` and
  `src/tests/<module>.rs`, ordinary Rust under the crate's `clippy.toml` (the tests' rules:
  one thread, no `HashMap`, no clock); milliseconds each, no network, no held login, no real
  repository, no process but what a test names. They run in the gate's focused profile:
  `heavy cargo nextest run -p temper-stage-one`.
- **Never run a live story** and never run `temper` against a real service: no test of a
  `live*` or `proof_live` binary, with or without `--run-ignored`.
- **Other plans beside this one** (`public-reading-build`, `playground-browser`) change files of
  their own on their trunks. This plan's one shared file is the root `Cargo.toml` (a workspace
  member line, which `playground-browser/browser-walk` also edits) and `Cargo.lock`; keep that
  hunk to the one line. Touch nothing under `temper/tests/client-browser/**`,
  `temper/tests/one-process/**`, `temper/crates/temper-playground/**`,
  `temper/crates/temper-client-tools/**` or `temper/docs/design/client/**`.
- No new crates, workspace members or dependencies beyond the spec's.
- Old-form counts only fall (`temper/scripts/old-forms.txt`; run
  `python3 -I -B temper/scripts/old_forms.py`).
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: code or a test that runs it gives it `</dev/null`
  (`std::process::Stdio::null()`).
- Before pushing a branch that touches Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` and fix what it lists in the
  files you own.
