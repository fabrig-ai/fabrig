# Heavy status

Provisional, 2026-10-30. `tools/heavy/design.md` is implemented by the six
runtime files named in section 7, the six `test-*` suites, `tools/install`,
`tools/installed` and `tools/test_install.py`. Landing/check scripts in this
directory support tests and are not installed tools.

The queue discovers the five offline heavy suites through their executable
shebangs and explicitly omits `test-heavy-recovery` from its automatic step.

Fleet installation and reconciler repointing remain operator steps. The
agents copy and default workspace/state paths are retained. No stored
format or queue identity is changed by relocating the executable.

The recovery-suite `rollback` case covers relocation of the current helper
across installation paths; it does not establish old-version compatibility. Runtime/cache fixtures use private
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

Byte observation failures are scoped to the affected task and use byte
waiting/normal retry cleanup. Supported decimal minima are normalized and
bounded to 640 significant digits for JSON replay under any supported Python
conversion limit. The offline repair suite covers a failed cwd beside unrelated
cancellation, strict/default/permissive subprocess replay, corrected wait ageing,
and crash/delayed entry with retry cgroups held populated. Its pinned pre-feature
fixture demonstrates why all old writers must be drained and updated or disabled.
Ops deployment proof is required before first opt-in; installed recovery alone
is not that proof. No mixed-version writer deployment is approved.

Optional `HEAVY_MIN_FREE_WAIT_S` and the fresh, narrowly scoped slot
machine policy are implemented. New byte tasks freeze declarations, accumulate
same-boot monotonic low-space episodes across retries, and publish durable
`outcome=disk-wait-cap` exit 2 receipts at the cap. CPU queue waits with enough disk, payload
execution and terminal cleanup are excluded. Additive `heavy-disk-wait-v1`
accounting in state, event, log and v1 receipt records host/slot/declaration,
episode count and elapsed seconds/minutes; old missing accounting and interrupted
clock histories remain unknown. The landing reader accepts validated accounting.
The offline slot/cap suite uses fake time, disk and cgroups, including a full
submission record and machine-file changes. Fleet enablement, writer-lifetime
proof and production installation remain Ops work after source approval.

The pinned cap keys are `HEAVY_MIN_FREE_WAIT_S` and machine-scoped
`HEAVY_SLOT_MIN_FREE_WAIT_S`, with the existing `HEAVY_SLOT_BYTE_ADMISSION`
enable key. New byte receipts expose `observed_free_bytes`, `wait_below_min_s`
and `wait_cap_s` beside the required minimum and existing host/slot/task/episode
evidence. The offline suite runs a local payload that exits 2 and verifies its
`outcome=completed`, distinguishing it from the operational cap outcome.
Exit 2 without a validated receipt remains ambiguous. Pending declarations
and the 80000000000-byte slot minimum are unchanged.
