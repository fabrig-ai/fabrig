# live-temper: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it.

- The tree is temper (one task edits the root `.config/nextest.toml`); read `AGENTS.md`,
  then `temper/AGENTS.md`.
- **Never run a live story.** The tests of the `live`, `live_github` and `live_github_managed`
  binaries reach real services and need held logins: no task runs them, with or without
  `--run-ignored`. `cargo nextest list` is how a task checks what a profile selects. The
  launcher's tests in `temper/crates/temper/tests/suite/forge_land.rs` stub `heavy` and run
  offline: those are the tests that check `scripts/proof-live.sh`.
- Never read, copy or name a login: no `~/.claude`, no `auth.json`, no shelf path. Test
  fixtures are synthetic files and directories in a test's scratch.
- Touch no `docs/help/` and no draft. `docs/design/` only where the task's spec gives it.
- No new crates, no dependency changes, no new public items beyond the spec's.
- Old-form counts only fall (`temper/scripts/old-forms.txt`): new comments carry no
  "actual" or "genuine".
- Every build and test through `heavy`, one at a time; the narrowest test first
  (`heavy cargo nextest run -p temper --test suite -E 'test(=forge_land::NAME)'`), check the
  count it selected.
