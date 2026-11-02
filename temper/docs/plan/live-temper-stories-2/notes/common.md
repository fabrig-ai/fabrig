# live-temper-stories-2: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`.
- **Never run a live story.** The tests of the `live`, `live_github`, `live_github_managed`
  and `proof_live` binaries reach real services or need held logins: no task runs them, with
  or without `--run-ignored`. In the slot, run the worlds and the harness's tests: the
  process tier's restart worlds on the owned Forgejo and fake backends
  (`crates/temper/tests/process.rs`, through `tools/tiers/run temper process`), and the
  landing stories' harness in `temper`'s `suite` binary (`crates/temper/tests/suite/forge_land.rs`),
  which runs offline. fab's after-landing pair is the live proof, after the merge.
- Never read, copy or name a login: no `~/.claude`, no `auth.json`, no shelf path, nothing
  under `~/src/fabrig/fab/`.
- **The design is written:** `temper/docs/design/domain/testing.md, 2.7` already says what
  each task builds, from `temper/docs/design/domain/forge.md, 8.8`. The one `docs/design/`
  edit a task makes is the removal of its own **Not yet** text from the status file its spec
  names (a stated departure from the implementer's text); no other design, draft or
  `docs/help/` edit.
- No new crates, no dependency changes, no new public items beyond the spec's.
- Old-form counts only fall (`temper/scripts/old-forms.txt`): new comments carry no history
  and no "actual" or "genuine".
- Every build and test through `heavy`, one at a time; the narrowest test first, and check the
  count it selected (nextest exits 4 when it selects nothing).
