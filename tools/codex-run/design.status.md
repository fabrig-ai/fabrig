# Codex runs status

Provisional, 2026-10-28. `tools/codex-run/design.md` is implemented by
`codex-run`, `test-codex-run`, `tools/install`, `tools/installed` and
`tools/test_install.py`. No known implementation departures.

Fleet installation is pending coordinator review. The agents repository's
copies remain in place. Run and owner storage remain under `~/src/rust/tmp`
by default, preserving resume access and accounting; migration is deferred.

Task run reaper producer hooks and isolated adoption tests are implemented
(`tools/codex-run/design.md`, 3.1). They tolerate an absent CLI and unit;
deployment, discovery, keeps and deletion remain the reaper owner's work.
Persistent owner lifecycle records remain MC's responsibility.
