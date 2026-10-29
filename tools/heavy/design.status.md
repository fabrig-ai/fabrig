# Heavy status

Provisional, 2026-10-29. `tools/heavy/design.md` is implemented by the six
runtime files named in section 7, the four `test-*` suites, `tools/install`,
`tools/installed` and `tools/test_install.py`. Landing/check scripts in this
directory support tests and are not installed tools.

Fleet installation and reconciler repointing remain operator steps. The
agents copy and default workspace/state paths are retained. No stored
format or queue identity is changed by relocating the executable.

Recovery-suite rollback covers the current queue across installation
paths, without agents Git history. Runtime/cache fixtures use private
version launchers and fonts. A historical daemon rollback and real browser
package validation are deferred in the design's Later section.
