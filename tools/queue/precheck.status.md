# Branch pre-ready check status

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
and absent ownership marker were verified afterward. The tool is ready for
merge; installation on builders follows landing. Heavy completion after a lost client requires reconciliation;
unsupported sparse/submodule/hidden-index or ignored-collision inputs are
refused. Unit observation cannot replace exclusive ownership of an idle
claimed slot.
