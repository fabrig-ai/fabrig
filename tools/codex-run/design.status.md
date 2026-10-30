# Codex runs status

Provisional, 2026-10-28. `tools/codex-run/design.md` is implemented by
`codex-run`, `test-codex-run`, `tools/install`, `tools/installed` and
`tools/test_install.py`. No known implementation departures.

Fleet installation is pending coordinator review. The agents repository's
copies remain in place. Run and owner storage remain under `~/src/rust/tmp`
by default, preserving resume access and accounting; migration is deferred.

Owner-bearing task run start records and best-effort service triggers are
implemented (`tools/codex-run/design.md`, 3.1), with isolated fixtures. No run
completion or stop emits an end. The task-finalization interface is pending
separate design; task-final retention adoption is incomplete. Deployment,
discovery, indefinite keeps for resumable/uncertain tasks and deletion remain
the reaper owner's work. Persistent owner lifecycle remains MC's responsibility.
