# Heavy status

Provisional, 2026-10-29. `tools/heavy/design.md` is implemented by the six
runtime files named in section 7, the four `test-*` suites, `tools/install`,
`tools/installed` and `tools/test_install.py`. Landing/check scripts in this
directory support tests and are not installed tools.

The queue discovers the three offline heavy suites through their executable
shebangs and explicitly omits `test-heavy-recovery` from its automatic step.

Fleet installation and reconciler repointing remain operator steps. The
agents copy and default workspace/state paths are retained. No stored
format or queue identity is changed by relocating the executable.

Recovery-suite rollback covers the current queue across installation
paths, without agents Git history. Runtime/cache fixtures use private
version launchers and fonts. A historical daemon rollback and real browser
package validation are deferred in the design's Later section.

Executable suites disable bytecode before importing local helpers. The offline
bytecode regression invokes their real entry points in a disposable clean Git
checkout with global configuration disabled, so generated Python caches cannot
be hidden by a developer’s excludes. The real-systemd recovery suite remains
manual-only.

`HEAVY_MIN_FREE_BYTES` is implemented as an opt-in, per-task exact-byte floor
on the command filesystem. The version 1 ledger and receipt schema retain their
existing fields; optional byte observations and retry state are additive.
The byte fixtures cover independent reconciliation, payload cgroup cleanup and
retry, cancellation, legacy behavior, floor composition and landing priority.
Deployment uses the normal fabrig tools install and remains inert until opted
in; rollback first drains or cancels unfinished opt-ins with the new helper
(`tools/heavy/design.md, 7`). No fleet install is part of this task.
