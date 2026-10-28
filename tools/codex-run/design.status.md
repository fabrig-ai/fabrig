# Codex runs status

Provisional, 2026-10-28. `tools/codex-run/design.md` is implemented by
`codex-run`, `test-codex-run`, `tools/install`, `tools/installed` and
`tools/test_install.py`. No known implementation departures.

Fleet installation is pending coordinator review. The agents repository's
copies remain in place. Run and owner storage remain under `~/src/rust/tmp`
by default, preserving resume access and accounting; migration is deferred.
