# Branch pre-ready check status

Decision0016 stage A shares `mq.tools_required` with queue execution and
worker expected plans. Full selection without changed `tools/` paths skips
tools suites while preserving the ordinary ratchet/fmt/clippy/build plan.
Stage B CPU-per-suite policy is not implemented; existing budgets remain.

Provisional, 2026-10-29. `tools/queue/precheck.md`, sections 1–4, is
implemented in `precheck` with disposable Git and fake-command coverage in
`test_precheck.py`. The installed manifest and installer tests include its
executable link. `mq` and its default queue behavior are unchanged.

Ops' initial real builder docs-only smoke exposed a timer being inspected
as a service. A repeat smoke also identified the installed reconciler service,
which owns no slot build working directory. Service-only observation skips
that reserved maintenance unit; payload/wrapper services remain checked.
Service-only observation and exact refusal of the existing
`mq-worker-unresolved.json` worker marker are implemented with regressions.
Ops validated the executable on a claimed ordinary builder: a docs-only
check passed, then fmt, ordinary-heavy clippy and nextest test-build checks
passed over the queue selection. The original branch, commit, clean worktree
and absent ownership marker were verified afterward. Corrected candidate,
selection parity, disk admission and caller isolation have real-builder proof;
the corrected precheck is not deployed. Heavy completion after a lost client requires reconciliation;
unsupported sparse/submodule/hidden-index or ignored-collision inputs are
refused. Unit observation cannot replace exclusive ownership of an idle
claimed slot.

Per-command disk admission is implemented with `--min-free-bytes` (default
80000000000), including a fresh observation before the nextest build after
clippy. `test_precheck_admission.py` uses disposable Git, fake checkers and
deterministic free bytes to cover deferral, exact restoration, retained
completed results, unchanged commands and CLI validation. This helper
scheduling threshold leaves heavy and queue floors unchanged. Admission proof
does not install the corrected executable; deployment remains Ops' work.

The first real Jig check exposed a tools-test isolation failure:
`test_mq.py` ran queue ratchet stage/commit logic against its caller's Git
repository. Its effectful cases and scratch now use disposable sibling
fixtures; ratchet commit semantics remain tested with real Git.
`test_precheck_tools_isolation.py` covers the offending case, all 120 mq
cases over a staged candidate plus unstaged lower baseline, and precheck
restoration after running the real mq suite, with no Cargo/heavy step for
its docs/tools candidate. Real corrected-candidate and caller-isolation proof
is complete; the corrected executable is not deployed.

Selection captures merged candidate paths before generated ratchet writes,
matching the queue's selection before checker recounts. The result keeps
`selection_paths` separately from the complete post-recount `paths` diff.
Branch-supplied baseline changes and baseline-only conflicts still select
full checks; conflict paths remain selected even when recounting matches
BASE. `test_precheck_selection.py` covers package/reverse dependency scope,
docs without a graph, supplied baselines, recount-before-lint ordering,
conflict retention and exact restoration. Claimed-builder ordinary-heavy
selection-parity and low-disk admission proof is complete, with deployment
still pending.

Section 5 is implemented by standalone `ready-check`, with disposable bare
origins/candidate clones in `test_ready_check.py`. It validates exact-tip
owner-supplied evidence, checks reported literal paths against freshly
observed main history, and refuses observed remote advancement during the
check. The installed manifest and installer tests include its executable
link. It does not call precheck or modify its ownership/lifetime behavior,
and does not publish readiness. A pass has the point-in-time and exact-file
scope limits in section 5; the queue gate remains necessary. Ready-check has
landed and is installed.

The automatic whole-target guard is not deployed. Production uses the frozen
bcaf external ten-file queue repair lane while the train is off. Source
consolidation and passing fixtures do not establish installation or activate
the train; readiness and deployment remain Root's work.
