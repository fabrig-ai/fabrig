# Merge queue status

Decision0022 uses shared `mq.gate_tools_required` for gate checks and
worker plans: no pipeline tools suites run for any batch, including tools
changes and full runs. Author precheck still calls `mq.tools_required`
as a compatibility predicate, then `mq.author_tools_commands` selects only
the changed owners' suites before Ready (section 2.3). Unknown ownership or
missing candidate suites refuses explicitly; installer changes select the
installer suite. No unrelated launcher suite is selected for other tools.
The policy remains in the frozen `mq` runtime file; restart and worker
identity already include it, with no additional runtime dependency.
Train preparation and evidence validation consume the worker
plan; exact result, runtime identity and publication replay rules are retained.
Train's independent expectation and recovery's stored-certificate validation
use that same exact plan. Ready-check validates the recorded compile/lint
plan and has no full-implies-tools rule. Publication has no tools selector.
Stage B CPU-per-suite policy awaits Green's shared schema. Launcher CPU15 /
wall60 and each tool suite's wall60 remain unchanged; no deployment is claimed.

Provisional, 2026-10-29. Sections 1–3.7 retain one-gate selection/check/outcome
policy. Explicit parent/candidate inputs, writing ratchets, offline tools
suite discovery and `restart --batch 16` remain covered by `test_mq.py`.
Section 3.7 now uses section 3.10's shared authority, persistent startup guard
and acknowledged same-inode handoff. Nothing is installed or activated here.

Section 3.7.1 is implemented by the opt-in pinned repair triplet and restart's
external runtime selection. `test_repair_priority.py` covers disposable local
origins, full singleton ordering, normal batches of 16, durable tried tips,
pin refusal/advancement, red/conflict/error audit and simulated CLI/restart
launches. This enables no production action and changes no check or timeout.
It is integrated with section 3.10's authority and restart interfaces. External
runtime validation covers all twelve frozen module/list files, refuses changes
before update and launch, and loads its authority dependencies from that copy.
The priority suite retains its 19 cases and adds dependency-drain, persistent
ownership, cross-clone exclusion and real external-child handoff coverage.
Review repairs cover signal/137 heavy uncertainty with durable global and
checkout refusal receipts, no input recovery/pruning and unsupported release;
consumed-pin restart with trusted state revalidation; PATH/plan refusal audit;
and opt-in adjudicated repair hold. Real disposable daemons prove official
restart through held consumed pins, same authority inode, stop drain and a
later restart without hold resuming ordinary work. No production activation
or unknown-heavy reconciliation is supplied.

Section 3.8 remains an opt-in local worker, with no dispatch from mq/restart
and no second-prover activation. It shares checkout exclusion with legacy and
preparation; private request/log/result publication and unresolved ownership
refusal are hardened. The earlier dedicated-gate empty-plan smoke validates
object identity and local evidence only; it proves no heavy supervision.

Section 3.9 remains a disabled library with mandatory authority/transport.
Pinned chained candidates, independent certificates, ordered decisions,
conservative cadence, sequential bisection and provisional partial conflicts
remain intact. INFRA rejects no input, invalidated jobs still physically own
their gates, and speculation stops through ancestor failure and bisection.
Full P/C diffs include generated baselines and may conservatively select full;
preparation ratchet reds conservatively require full bisection checks.

Section 3.10 is implemented in authority.py, durable.py and recovery.py with
minimal mq/restart/worker/train integration. A clone-independent shared lock,
durable train ownership receipt and worker/precheck/preparation markers block
unsafe cross-mode reuse. Restart serializes independently, drains before
acquiring main authority, retains stop on failure and passes one authority
inode to an acknowledged child. Local effect clients inherit authority.

Concrete train effects load recorded P/C intent, read fresh main and push the
explicit C normally; cleanup leases apply only to ready deletion. Frozen area
routes and stable event/evidence bodies use a durable at-least-once outbox.
State version 1 refuses unknown versions and incompatible runtime/config;
there is no silent migration. Structural/artifact validation, cadence and
bisection consistency, preparation/dispatch/evidence/promotion/outbox inventory,
publication poisoning and completed-preparation reconciliation are implemented.
Pins and all artifacts are retained; there is no automatic GC or repair by
replacing immutable evidence.

Part 5 remains required and production train activation is prohibited. Its
supervised SSH/heavy lifetime evidence must cover client/controller/worker
loss, stable same-job identity, complete object/evidence return and physical
drain. The built-in evidence adapter accepts only local empty-plan jobs;
nonempty/heavy/remote completion remains blocked. Fake tests inject a separate
identity-bound completion adapter and prove no real heavy quiescence.

Operational stop/drain/resume/restart, explicit generation retirement/legacy
return, real-tool validation and reviewed rollout remain part 5. A snapshot
or train ownership receipt always refuses legacy; process absence or released
flocks never retire ownership. No real main authority, queue restart, SSH,
post, install, Cargo/build, systemd operation or second-gate activation was
performed by this change. Disposable Git/process tests cover explicit crash
boundaries, not arbitrary machine power loss or remote supervision.

Bisection recovery consumes admitted groups before replaying red splits or
INFRA requeues, including concrete promotion's snapshot load. Applied INFRA
ownership stays in a valid decision phase through late status failures and
verified release; drain requires a loadable matching snapshot. Legacy cleanup
uses expected-tip deletion leases, and local mutating children inherit both
main authority and checkout exclusion through parent death. Restart restores
stop across the unlink/fsync/launch boundary. Error reporting resets after a
completed cycle so a new error episode alerts again. Authority/recovery/worker
and mq fixture roots are outside the checked source. The controller fixture
is shared without test inheritance across preparation (19), decision (22) and
recovery (22) entries; two isolation/discovery cases verify external roots and
exactly-once coverage. The manifest preserves all 63 part4 case bodies, including
its two completion-adapter changes. Part3 split suites passed real h8 validation;
the combined part4 integration still awaits validation. No timeout or completion evidence is
weakened, and part 5's remote lifetime/activation boundaries remain blocked.
