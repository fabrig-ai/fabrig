# Branch pre-ready check status

The 15:52 review repairs scope SIGCHLD ownership and restore exact caller
handlers after known API/Executor completion or preflight refusal, retaining
waitable handling during unresolved custody. Standalone readiness applies the
same boundary before each Git launch. Raw launch requirements distinguish
direct executable files from Python-readable inputs. Handler, real failed/
signaled diff-tree and coordinated permission fixtures cover these boundaries.

Repair of the 15:18 source review establishes waitable SIGCHLD handling before
Git/checker execution and binds discovered suites to present prepared raw
sources, including symlink closure. Real subprocess fixtures inherit ignored
SIGCHLD across exec and preserve nonzero/abnormal results; coordinated missing
source receipts refuse even with updated manifests and raw trees. Production
cold proof and deployment remain pending source review.

Repair of the 14:17 source review adds child-subreaper descendant closure,
private descriptor-bound quarantine cleanup, scope-derived receipt requirement
and index-derived discovery, complete static ancestor/construction admission,
and common-directory graft refusal before preparation. Adversarial tiny
fixtures reproduce the detached child, root replacement, coordinated erasure/
substitution, deep packed-history layout and linked-worktree graft boundaries.
The original ddc2 implementation remains preserved in history. These are author
checks under decision0022, not a production cold certificate or deployment.

Cold-tools preparation and version-1 receipt validation are implemented by
precheck, eagerly loaded cold_tools.py and ready-check (precheck.md, 1.1, 3–5.1).
Only tools cwd/environment move to an independent private prepared copy with
logical-index/raw-tree bindings and historical objects. Static-copy headroom
admission is transitional, not decision0012 hard containment. Unknown tools
completion or cleanup failure retains source, aux, marker and lifetime hold.
Older tools receipts require re-precheck after compatible deployment. Tiny
offline fixtures establish source behavior; source-CLI production cold proof
awaits Ops source review. Installed old-policy precheck evidence is separate.

Decision0016 stage A supplies the author `mq.tools_required` path predicate.
Decision0022 gate skipping is separate: precheck still checks selected tools;
this repair changes neither mq nor worker gate policy. Full selection without changed `tools/` paths skips
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
