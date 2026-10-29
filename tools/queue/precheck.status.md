# Branch pre-ready check status

Provisional, 2026-10-29. `tools/queue/precheck.md`, sections 1–4, is
implemented in `precheck` with disposable Git and fake-command coverage in
`test_precheck.py`. The installed manifest and installer tests include its
executable link. `mq` and its default queue behavior are unchanged.

Ops' initial real builder docs-only smoke exposed a timer being inspected
as a service. Service-only observation and exact refusal of the existing
`mq-worker-unresolved.json` worker marker are implemented with regressions.
Ops must repeat real validation, including ordinary-heavy compile/lint/build
coverage. This task runs no Cargo builds, remote commands, production
installation or service operations. Ops will validate
on a claimed ordinary builder slot before marking ready, and install after
landing. Heavy completion after a lost client requires reconciliation;
unsupported sparse/submodule/hidden-index or ignored-collision inputs are
refused. Unit observation cannot replace exclusive ownership of an idle
claimed slot.
