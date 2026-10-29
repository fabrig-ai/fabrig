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

The read-only `--completion-query` API in design 6.1 is implemented for queue
transport. Seven offline isolated-ledger cases cover immutable matching
receipts/logs, unresolved/absent identities, corruption/version/queue refusal,
and byte/mtime invariance on success and failure. This new behavior requires
fresh operator validation; previous identical-runtime migration proofs do not
validate it. No real recovery/systemd operation was run for this addition.
The query's sealed-log read is bounded at 64 MiB; the added oversized-log case
brings its offline entry to eight cases. This does not change stored formats.
