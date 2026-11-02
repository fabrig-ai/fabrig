# live-temper-stories: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`.
- **Never run a live story.** The tests of the `live`, `live_github`, `live_github_managed`
  and `proof_live` binaries reach real services or need held logins: no task runs them, with
  or without `--run-ignored`. The landing stories' harness is checked by `temper`'s `suite`
  binary (`crates/temper/tests/suite/forge_land.rs`), whose tests run offline. fab's
  after-landing pair is the live proof, after the merge.
- Never read, copy or name a login: no `~/.claude`, no `auth.json`, no shelf path, nothing
  under `~/src/fabrig/fab/`. Test fixtures are synthetic files and directories in a test's
  scratch (`support::Scratch`), written by `support::write_private`.
- **The design is written:** `temper/docs/design/client/tools.md, 8.1` and
  `temper/docs/design/domain/testing.md, 2.7` already say what each task builds. The one
  `docs/design/` edit a task makes is the removal of its own **Not yet** text from the status
  file its spec names (a stated departure from the implementer's text); no other design,
  draft or `docs/help/` edit.
- No new crates, no dependency changes, no new public items beyond the spec's.
- Old-form counts only fall (`temper/scripts/old-forms.txt`): new comments carry no
  "actual" or "genuine".
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing).
- `temper ctl` reads stdin: a script or stub that runs it gives it `</dev/null`.
