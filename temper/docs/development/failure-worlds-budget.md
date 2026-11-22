# Failure worlds: the budget before the checks and sweeps

2026-11-20. **Report-only fallback: current suite figures are unmeasured.** The requester authorized this incomplete measurement account after the build/disk failure; it is not an affordability or gate-pass claim. This is the CPU plan for `temper/docs/design/draft/failure-worlds.md`, sections 3, 5, 7 and 8, and F0, F7 and F9 of `temper/docs/design/draft/failure-worlds.execution.md`, section 1. It proposes no change to a budget, reference, reason, seed or patience. A proposal below is for the user to decide, not permission for an implementer to spend another layer's room.

## Method and limits

The intended measurement source was the focused (`default`) and `fuzzy` selections of **all 459 packages in the five trees**, at **349ba05e9e2f0decf73109f01bd7b0ee870c7c74**, through `tools/tiers/one-process`. Package membership comes from `cargo metadata --no-deps --format-version 1`: skein 115, smith 63, jig 116, temper 130 and hearth 35. The repository's boundary package is outside this selection; its checks are not a sixth measured tree. Both `TEMPER_INTERNAL_BUNDLE` and `TEMPER_BUNDLE` are `none`.

The first attempt, on producer machine **w3**, acquired no build hold. From 21:02:29 UTC until its shell's 3600-second timeout it printed only `one-process: hold: waiting for the active delivery check` and `smith-check: waiting`. No suite started and no CPU figure comes from that attempt. The subsequently authorized producer attempt started on producer machine **w4** at `e94f14631389f7dcd4b3eb8094c1004379d6b2c0` (the report draft and boundary-only changes above the same tree code); its first observed output at 22:50:36 UTC was the same hold wait. The requester subsequently reported that the producer attempt's build wrapper exited **75, Infra: disk**, after its **1200 s wait for 20 GB**. That later failure was relayed, not verified from a locally readable run log.

Operations' **w1** attempt is recorded **as ops (person 1) reported it at 23:21Z**, through the requester: commit `349ba05e`, the same 459-package selection, bundles off and normal build hold, acquired at 22:31 UTC. All packages compiled; a subsequent metadata command encountered the build wrapper's 20 GB disk minimum, waited 1200 s and exited 75. The outer `default` command exited **1 with no measurement report**; **fuzzy did not complete**. Its logs, results.json, run.json and collector summary weren't readable from this checkout, so its lanes, machine figures and per-binary CPU are unknown. “w1” is the supplied machine identifier, not a verified hostname. Its source directory's reported basename was `budget-11096-dAT9LyEx`; the reported copy `target/w1-budget-11096-Ng5Kp34d` was absent here. Machine-private absolute paths are omitted to keep the report within the repository's outward boundary. These are evidence locators supplied by ops, not files this author read.

No standalone-binary completion ran, since there was no suite result identifying a partial binary. No further build or disk workaround was attempted. No hold was bypassed. No slow profile, browser, walk, render check, instrumented prototype or throwaway driver ran. Every layer and every affected binary below is **unmeasured**, not zero and not a completed partial observation.

The intended method measures CPU as **user plus system seconds of suite processes, including waited-for descendants**, from `results.json`, placed by its own `tree` and `layer` fields, checked against the collector's summary. No such current figure was available here. Sum only `tests`, `fresh` and `alone` processes; exclude `reference` and `empty`. A long binary may have several processes. See `tools/tiers/design.md`, sections 11.4 and 11.8, and `tools/tiers/budgets.md`, section 4. A test's shared-process wall time is **not its CPU**, and is never summed or apportioned into a measured test CPU.

Main's ledgers were not supplied. Neither the failed attempts nor the historical observations below establish main's held budgets, medians, noise allowances or a gate verdict. The ceiling is `tools/tiers/budgets.toml` with **all existing moves applied once**, not the bare `cpu_s` entry; the held budget may be lower (`tools/tiers/budgets.md`, sections 2 and 7 to 10). A negative room is reported, not repaired by changing its figure. No CPU-to-instructions conversion is attempted: `temper/docs/design/draft/test-budgets.md`, section 3, describes the instructions gate that is not yet built here.

Earlier evidence is deliberately separate:

- **H17:** `temper/docs/development/test-budgets-1117.md`, dated **2026-11-17**, commit `7abbe33109ec2ffc8d69f4b9295cbb2c666cc9da`, hel1-8, eight lanes for `cpu-1117-focused` and `cpu-1117-fuzzy`. Its *Method and limits* supplies the method and its binary tables supply historical envelopes, not today's figures. Its package selection was not all five trees.
- **HW7:** `temper/docs/development/limits-worlds.md`, *What stays, for the user's placement decision*, supplies the eighteen survivors' **historical alone CPU** and unique coverage. Its 2026-11-07 runs are not today's binary CPU or guaranteed savings.
- **HF7:** `temper/docs/development/limits-fuzzy.md`, dated **2026-11-07**, combined trunk `182ef80c19a1`, figure `close`, identifies existing sweeps and their retained draws, restart/readback work and historical instruction profiles. Its serial native ranges include the driver; instruction counts are not native CPU in this plan. HW7 and HF7 used serial commands, not H17's one-process selection; their ranges are not same-code confidence intervals.
- `temper/docs/development/failures-1113-1120.md` supplies the incident-to-lowest-world mapping. Few attributed catches means **none attributed**, not none caught.
- `temper/docs/design/testing/checks.status.md` records the placement departure: eighteen named real-loop scenarios remain in fuzzy because the earlier request kept them there, not because their CPU chooses their suite. The departed contract is `temper/docs/design/testing/checks.md`, section 1.

A failed, starved, cancelled or unobserved test makes its binary's figure **partial**. A complete standalone-binary follow-up replaces that binary's first figure; it is not added to it. A follow-up still incomplete establishes only a lower bound, not a passing layer. Zero selected tests means zero contribution to this suite, not proof of the excluded tests. Unplaced binaries retain the runner's warning rather than acquiring a guessed layer.

## Layers now

### Focused (`default`)

| Tree | Layer | Figure, CPU s | Effective ceiling, CPU s | Room, CPU s |
|---|---|---|---:|---|
| skein | step | unmeasured | 7 | unmeasured |
| skein | domain | unmeasured | 2 | unmeasured |
| skein | machine | unmeasured | 5 | unmeasured |
| skein | protocol | unmeasured | 7 | unmeasured |
| skein | io | unmeasured | 2 | unmeasured |
| skein | simulated | unmeasured | 3 | unmeasured |
| skein | real-loop | unmeasured | 7 | unmeasured |
| skein | end-to-end | unmeasured | 4 | unmeasured |
| smith | step | unmeasured | 7 | unmeasured |
| smith | domain | unmeasured | 3 | unmeasured |
| smith | system | unmeasured | 2 | unmeasured |
| smith | protocol | unmeasured | 3 | unmeasured |
| smith | simulated | unmeasured | 11 | unmeasured |
| smith | real-loop | unmeasured | 2 | unmeasured |
| smith | end-to-end | unmeasured | 21 | unmeasured |
| jig | step | unmeasured | 47.5 | unmeasured |
| jig | domain | unmeasured | 60.6 | unmeasured |
| jig | machine | unmeasured | 2 | unmeasured |
| jig | protocol | unmeasured | 6.2 | unmeasured |
| jig | simulated | unmeasured | 59.7 | unmeasured |
| jig | real-loop | unmeasured | 2 | unmeasured |
| jig | end-to-end | unmeasured | 2 | unmeasured |
| temper | step | unmeasured | 253 | unmeasured |
| temper | domain | unmeasured | 137.5 | unmeasured |
| temper | system | unmeasured | 111 | unmeasured |
| temper | machine | unmeasured | 4 | unmeasured |
| temper | protocol | unmeasured | 9 | unmeasured |
| temper | simulated | unmeasured | 470 | unmeasured |
| temper | real-loop | unmeasured | 81.5 | unmeasured |
| temper | end-to-end | unmeasured | 39 | unmeasured |
| hearth | step | unmeasured | 2 | unmeasured |
| hearth | domain | unmeasured | 4 | unmeasured |
| hearth | machine | unmeasured | 13 | unmeasured |
| hearth | real-loop | unmeasured | 11 | unmeasured |
| hearth | end-to-end | unmeasured | 2 | unmeasured |

H17's **historical**, complete temper/domain figure was 161.271829 CPU s and protocol 11.391424; compared with these file ceilings they would have room −23.771829 and −2.391424 s. Its temper/real-loop 66.407326 CPU s was partial, so even then 15.092674 s was only an upper bound on room. These are historical comparisons, not current rows.

### Fuzzy

| Tree | Layer | Figure, CPU s | Effective ceiling, CPU s | Room, CPU s |
|---|---|---|---:|---|
| skein | step | unmeasured | 32 | unmeasured |
| skein | domain | unmeasured | 2 | unmeasured |
| skein | machine | unmeasured | 43 | unmeasured |
| skein | protocol | unmeasured | 23 | unmeasured |
| skein | io | unmeasured | 21 | unmeasured |
| skein | simulated | unmeasured | 2 | unmeasured |
| skein | real-loop | unmeasured | 20 | unmeasured |
| skein | end-to-end | unmeasured | 1 | unmeasured |
| smith | step | unmeasured | 3 | unmeasured |
| smith | domain | unmeasured | 16 | unmeasured |
| smith | system | unmeasured | 2 | unmeasured |
| smith | protocol | unmeasured | 3 | unmeasured |
| smith | simulated | unmeasured | 13 | unmeasured |
| smith | real-loop | unmeasured | 2 | unmeasured |
| smith | end-to-end | unmeasured | 3 | unmeasured |
| jig | step | unmeasured | 4.1 | unmeasured |
| jig | domain | unmeasured | 68 | unmeasured |
| jig | machine | unmeasured | 2 | unmeasured |
| jig | protocol | unmeasured | 30.4 | unmeasured |
| jig | simulated | unmeasured | 37.5 | unmeasured |
| jig | real-loop | unmeasured | 2 | unmeasured |
| jig | end-to-end | unmeasured | 1 | unmeasured |
| temper | step | unmeasured | 13 | unmeasured |
| temper | domain | unmeasured | 80.8 | unmeasured |
| temper | system | unmeasured | 13 | unmeasured |
| temper | machine | unmeasured | 2 | unmeasured |
| temper | protocol | unmeasured | 2 | unmeasured |
| temper | simulated | unmeasured | 196 | unmeasured |
| temper | real-loop | unmeasured | 195.2 | unmeasured |
| temper | end-to-end | unmeasured | 1 | unmeasured |
| hearth | step | unmeasured | 1 | unmeasured |
| hearth | domain | unmeasured | 32 | unmeasured |
| hearth | machine | unmeasured | 50 | unmeasured |
| hearth | real-loop | unmeasured | 9 | unmeasured |
| hearth | end-to-end | unmeasured | 1 | unmeasured |

H17's **historical**, complete temper/domain figure was 100.521904 CPU s, which would leave −19.721904 s against today's file ceiling. There is no historical complete all-five-tree breakdown in the cited reports to fill the other rows. Layers absent from a tree's tables are not assigned a zero budget here; the repository/checks group (default ceiling 72) was outside the 459-package selection and is unmeasured too.

### Affected binaries

These use the runner's identifiers, not package names standing in for binary IDs. Current figures belong here, separately from any historical alone observations below.

| Binary ID | Tree / layer | Current focused / fuzzy CPU s | Earlier CPU s, source and limits |
|---|---|---|---|
| `temper-engine-domain-world::suite` | temper/domain | unmeasured / unmeasured | H17 144.807676 / 45.044258; HF7 fuzzy serial range 10.303352–10.659738, different code/selection. |
| `temper-engine-forge-world::suite` | temper/domain | unmeasured / unmeasured | H17 8.305353 / 8.557658. |
| `temper-worker-domain-world::suite` | temper/domain | unmeasured / unmeasured | H17 0.218231 / 0.280485. |
| `temper-worker-checkout-world::fakes` | temper/domain | unmeasured / unmeasured | H17 1.495047 / 2.827622. |
| `temper-conformance-world::suite` | temper/domain | unmeasured / unmeasured | H17 5.621187 / 43.811881; HW7 medians 5.576544 / 30.586688; HF7 fuzzy serial range 27.876891–30.403677. |
| `temper-system-world::suite` | temper/system | unmeasured / unmeasured | HW7 focused median 10.806115; no same-world fuzzy CPU in these reports. |
| `temper-one-process-world::suite` | temper/simulated | unmeasured / unmeasured | HW7 medians 27.899207 / 11.275533; HF7 fuzzy range 11.800430–12.403466. These precede added upgrade cases. |
| `temper-real-loop-world::suite` | temper/real-loop | unmeasured / unmeasured | H17 focused 50.806274 partial; HW7 fuzzy median 60.139001; HF7 fuzzy range 61.052666–66.214588. |
| `temper-engine-protocol-forge-world::suite` | temper/protocol | unmeasured / unmeasured | H17 focused 11.391424; no fuzzy figure. |
| `jig-core-world::suite` | jig/domain | unmeasured / unmeasured | No same-world binary CPU in the cited reports; unknown, not temper's CPU. |
| `jig-engine-clients-world::suite` | jig/real-loop | unmeasured / unmeasured | Unknown in the cited reports. |
| `jig-client-protocol-world::suite` | jig/simulated | unmeasured / unmeasured | Unknown in the cited reports. |
| `jig-people-world::suite` | jig/domain | unmeasured / unmeasured | Unknown in the cited reports. |
| `smith-session-world::suite` | smith/domain | unmeasured / unmeasured | Unknown in the cited reports. |
| `smith-host-world::suite` | smith/domain | unmeasured / unmeasured | Unknown in the cited reports. |
| `smith-run-world::suite` | smith/domain | unmeasured / unmeasured | Unknown in the cited reports. |
| `smith-protocol-llm-world::suite` | smith/protocol | unmeasured / unmeasured | Unknown in the cited reports. |
| `temper-engine-service-startup` | temper/step | unmeasured / unmeasured | Unknown in the cited reports. |
| `temper-engine-service-vocabulary::figures` | temper/step | unmeasured / unmeasured | Unknown in the cited reports. |
| `temper-engine-service-stock` | temper/step | unmeasured / unmeasured | Unknown in the cited reports. |
| `temper-client-tools` | temper/step | unmeasured / unmeasured | Unknown in the cited reports. |
| `skein-fake-forge-tests::suite` | skein/domain | unmeasured / unmeasured | Unknown in the cited reports. |
| `skein-llm-world::suite` | skein/protocol | unmeasured / unmeasured | Unknown in the cited reports. |

Placement here is from the unchanged layers file, **not a claimed placement read from nonexistent current results.json**. Hearth has no failure-worlds addition in this draft, but all of its suite layers remain in the unmeasured baseline. No unplaced warning can be quoted without a runner report.

## What the checks add

### Where the work runs

This is a placement plan, not new code. A check inherited through a shared driver is charged once to the binary that executes it, not again to the adapter's kit. Library and binary unit tests are `step` by rule; integration binaries take `tools/tiers/layers.toml`'s placement (`tools/tiers/layers.md`, sections 3 to 6).

| Binary ID | Tree / layer | Additions to existing runs and nearest existing operation |
|---|---|---|
| `jig-core-world::suite` | jig / domain | 3.1/3.2 over tasks, waits, held answers, creation reports and subscriptions, at the root driver's drained-immediate-work fence; 3.3/3.4 over emitted and durable observations; 3.7 on isolated committed snapshots. Existing root checks, cold restarts and commit-cut scenarios are the proxies. |
| `jig-engine-clients-world::suite` | jig / real-loop | 3.1/3.2 on the sign-in's pending exchanges at the component driver's fence, not on global inactivity. P117's parked/resend case belongs here today. Existing request/answer and close fences are the proxy. Its binary also contains real children; calling it protocol without splitting it would undercharge real-loop. |
| `jig-client-protocol-world::suite` | jig / simulated | P77's client read/decide proposal path, complementing the core's authority assertion. Existing settings carriage and its settling checks are the proxy. |
| `jig-people-world::suite` | jig / domain | P76's account-use admission assertion. Existing people-world request/answer fence is the proxy. |
| `temper-engine-domain-world::suite` | temper / domain | 3.1/3.2 on the engine and jig inspections together; 3.3's run/try accounting, 3.4's explicit time and step bounds and unaffected-goal progress, and 3.7 on committed snapshots. Existing `direct::Driver::pump`/`drain`/`idle_since`, replay digests and public cold-readback oracle are the proxies. |
| `temper-engine-forge-world::suite` | temper / domain | 3.1/3.2 on landing/change waits, write/read admissions, holds and kept rows; 3.3's gate/candidate/base classification, both simultaneous-fault controls, 3.4 and 3.7. Existing forge-settling, restart replay and keyed-write checks are the proxies. |
| `temper-worker-domain-world::suite` | temper / domain | 3.1/3.2 on build places, workspaces and scratch at the worker driver's fence; 3.3's orderly stop versus lost-run accounting. Existing retained-turn and whole-worker replay checks are the proxies. No engine-store check is charged to a worker with no engine store. |
| `temper-worker-checkout-world::fakes` | temper / domain | Fetch-interruption, full fake disk and unremovable-trash checks with the workspace adapter; existing fake-file cleanup and seeded merge replay are the proxies. Actual git locking remains outside this proof. |
| `smith-session-world::suite` | smith / domain | 3.3 over provider/session/task counters and account/model identities, including the synthetic billing envelope, quota and unserved model. Existing session-ended/referee sweep is the proxy. |
| `smith-host-world::suite` | smith / domain | 3.3 for untooled programs and P71's request-derived silent-run bound; existing exactly-once host-ending ledger is the proxy. |
| `smith-run-world::suite` | smith / domain | Delivery-check queue versus execution clock and run blame; existing run-bound and cancelled-call checks are the proxy. |
| `smith-protocol-llm-world::suite` | smith / protocol | Shared rate window without retry-after and endpoint-bound windows, with 3.3's retry expectations. Existing pool's completion-order sweep is the proxy. |
| `temper-system-world::suite` | temper / system | 3.1/3.2 at each engine/worker driver's own quiet point, 3.3/3.4 from the joined observations, 3.5's normalized two-run oracle, and sampled 3.7. Existing `fuzzy::composed::seeded_system_runs_replay_turn_credit_calls_and_prepared_mounts` already runs two joined worlds per seed. |
| `temper-conformance-world::suite` | temper / domain | 3.7 every committed snapshot in its verified commit-cut cases; existing cold-start/public-readback branches are the proxy. Do not count these branches again as newly added restart sweeps. |
| `temper-one-process-world::suite` | temper / simulated | Keep the true whole-service/upgrade seams; call the inherited engine/worker checks at their local quiet points and sample 3.7. Existing restart, kill/power-loss and upgrade stories are the proxies. Do not install a second referee that reads product state. |
| `temper-engine-service-startup` and `temper-engine-service-vocabulary::figures` | temper / step | 3.6's shipped profile derivations and capacity relations, with valid override/refusal exceptions and generated mutations. Existing sizing and typed stock-limit tests are the proxies. |
| `temper-engine-service-stock` | temper / step | 3.6's live producer/model/charter references and provider-capability fixtures; existing stock-set/provider fixture tests are the proxies. P128 lives here, not in a live-provider test. |
| `jig-core-world::suite` | jig / domain | 3.6's settings-apply refusal when a template still names a retired producer (P59), plus amendment-admission relations only once S15's widening rule is built. Existing settings mutation/admission tests are the proxies. |
| `temper-client-tools` | temper / step | P81's proposed JSON command retains ctl's amount unit. This is serialization/admission, not an extra joined workload. Existing proposed-command rendering is the proxy. |
| `skein-fake-forge-tests::suite` and `skein-llm-world::suite` | skein / domain and protocol | The new fake behaviours' own transitions, intervals and causal identities; existing fake refusal and provider-wire tests are the proxies. Fakes in production crates do not depend on the world's ledger; the world copies their own reports. |

3.1 and 3.2 run at **every driver's own quiet point between iterations**, outside heap meters, as `skein/docs/design/testing/inspection.md`, section 5, requires. An unrelated task's steady work must not postpone the check. `direct::Driver::idle_since` deliberately excludes the fleet alarm from its immediate-work fence; replacing it with whole-root quiescence would miss the rule. Wait-cycle fixed points and table release scans cost more as the number of waits/entries grows; the workload must still reach the shipped small profile's bounds.

3.3's ledger grows per injected fault and records closed intervals, account/runner/gate/candidate/base/request identities and **six distinct counters**. 3.4 uses both simulated time and a nonzero step bound. 3.5 needs independent workload, faults, parties and scheduling streams and logical party-event replay; existing deterministic lockstep replay does **not** already prove equivalence with and without a restart. Deadline-race exclusions are named, not silently compared. The ledger, streams, fold and configured-fault-fell assertion are `skein/docs/design/testing/sweeps.md`, sections 2 to 5.

### How to price an operation not yet built

The suite measures processes, not quiet-point microseconds. For binary b let B(b) be its complete suite CPU. Let P(b,t) be the measured CPU of the process(es) containing existing test t. Where a process also ran other tests, P is an **inclusive upper envelope**, never an attributed CPU for t. For an existing sweep with N seeds, P/N is an upper estimate of its **average CPU per seed**, not a worst-seed guarantee. It includes setup, checks and any lockstep/restart companions the sweep already executes. New checks and new sweeps are estimates, not benchmark results.

For quiet-point checks the nearest existing operations are the same driver's fence and frozen-state digest/observation passes. For 3.3/3.4 they are its observation ledger/referee pass. For 3.5 use the same world's existing paired replay, retaining both runs in the price. For 3.7 use its existing restart and cold readback. A whole-process envelope is conservative about unrelated test work, but it cannot bound the new algorithm's growth at saturation. Any narrower planning fraction must be an explicit assumption beside the figure, not an invented measurement.

Price the **combined** 3.1/3.2 adapter pass once; do not charge a second traversal merely because there are two check numbers. Likewise normalized outcomes and the fault ledger are shared by 3.3 to 3.5. Existing heap, replay, encoding and teardown checks stay. A new operation may not be paid by deleting these checks or by weakening the load that reaches a table's bound.

### Historical price envelopes for the additions

**Current marginal CPU is unmeasured in every row.** These are deliberately broad *planning estimates resting on earlier reports*, not current measurements or guaranteed upper bounds. Let B be the earlier whole-binary CPU from the affected-binaries table. For the combined quiet-point adapter/invariants plus ledger/referee changes, use an exploratory **0..B CPU-s envelope per suite**: the same world's existing fences, frozen-state passes and referee checks are inside B. The explicit assumption is that the new passes together cost no more than one additional whole existing binary execution. Saturated wait graphs can invalidate that assumption. No nonzero lower marginal bound is known, and zero in an interval is not a prediction of free work.

| Existing-test additions, binary ID | Current focused / fuzzy increment | Historical exploratory focused / fuzzy CPU-s envelope | Proxy and uncertainty |
|---|---|---|---|
| `temper-engine-domain-world::suite`, 3.1–3.4 | unmeasured / unmeasured | 0..144.807676 / 0..45.044258 (H17) | Existing immediate-work fence and per-iteration digests, including oracle work; large unrelated scale/follows work makes this broad. |
| `temper-engine-forge-world::suite`, 3.1–3.4 | unmeasured / unmeasured | 0..8.305353 / 0..8.557658 (H17) | Existing forge fence, restart replay and history referee, not isolated phase CPU. |
| `temper-worker-domain-world::suite`, 3.1–3.3 | unmeasured / unmeasured | 0..0.218231 / 0..0.280485 (H17) | Existing worker/retained-turn checks and replay; new disk/workspace faults can enlarge the world. |
| `temper-worker-checkout-world::fakes`, 3.1/3.2 | unmeasured / unmeasured | 0..1.495047 / 0..2.827622 (H17) | Existing fake-disk/merge/cleanup passes; no actual git cost. |
| `temper-conformance-world::suite`, additional snapshot checks | unmeasured / unmeasured | 0..5.621187 / 0..43.811881 (H17) | Existing cold restarts/readbacks at cuts; reuse is possible, but a start's extra link/connector checks are not measured. |
| `temper-system-world::suite`, 3.1–3.4 | unmeasured / unmeasured | 0..10.806115 / unknown (HW7) | Existing composed fence and observations; only focused historical CPU exists. |
| `temper-one-process-world::suite`, inherited checks | unmeasured / unmeasured | 0..27.899207 / 0..12.403466 (HW7/HF7) | Existing full-service runs/restarts; upper endpoint is the earlier serial range's largest fuzzy sample, not a new upgrade-case price. |
| Jig's four affected worlds; smith's four affected worlds | unmeasured / unmeasured | unknown / unknown | The placements above name the nearest existing fences/referees. None of the permitted earlier reports isolates these worlds' CPU. A finite range would be invented; temper's store or provider is not a substitute. |
| 3.6 settings: stock/startup/vocabulary/client step tests, jig settings apply, and skein fake transitions | unmeasured / unmeasured | unknown / unknown | Same-tree stock mutation, admission or fake transition is the correct operation, but its CPU is absent from the earlier reports. Full schema/stock CPU cannot be inferred from a reference or a ceiling. |
| 3.5 normalized restart/no-restart comparison in system/world sweeps | unmeasured / unmeasured | unknown / unknown | An existing paired replay includes two worlds but has no historical isolated system-sweep CPU here. Add one comparable companion where absent, plus normalization, rather than claiming current replay already pays it. |

These envelopes overlap existing operations and **are not additive microprofiles**. In particular the conformance cold-check envelope and the restore price below describe the same work, not two purchases. New focused positive/finding cases and 3.3's two controls need their own same-world run prices; until those reports exist, their cost is unknown rather than a guessed per-test guide. The ceilings/reference values are limits, not measurements.

### Restore at every commit, or sample

For a run with C commits and a cold-check cost R at its store size, exhaustive 3.7 costs approximately C × R; more exactly sum R(size at commit i). A final restart's R is a proxy, not proof that early commits cost the same. No readable current suite report counts the runs' commits or isolates R, so a finite exhaustive CPU promise cannot be derived just by dividing binary CPU by tests. As a historical estimate only, one additional cold-check per existing run is enveloped by one additional full B: engine-domain 0..144.807676 focused / 0..45.044258 fuzzy, forge 0..8.305353 / 0..8.557658, conformance 0..5.621187 / 0..43.811881 (H17), system 0..10.806115 focused / unknown fuzzy (HW7), one-process 0..27.899207 / 0..12.403466 (HW7/HF7). These are coarse same-world restart/readback proxies, not isolated R, and are not added again to the overlapping table above. For every-commit checking multiply the appropriate run's cold-check proxy by its **unmeasured C**, so the exhaustive CPU interval has **no established finite endpoint**. Jig's R and C are both unknown in the cited earlier reports.

Propose **every-commit checking in the verified commit-cut domain cases** (`jig-core-world::suite` and `temper-conformance-world::suite`), with existing restart branches reused where they perform the full check. Elsewhere propose a seeded sampled committed snapshot per run initially, plus the ending cold check where already present. The committed snapshot is restored apart from the live world; the sample is selected independently of the fault/workload streams and the run reports checked/total commits. This makes no exhaustive claim. Every-commit checking in all composed and simulated runs is a separately priced option, not a default hidden in the sweep estimate. A product's full start checks, jig link pass/derived state and temper connector checks are not replaced by checking only the snapshot's decoder.

## What the sweeps cost

The seed choices below are **proposed new workloads**, not changes to existing seeds. Each fault family gets a cheapest reachable world; subfamilies crossing a wire or the simulator have an explicitly additional row. A family that spans roots cannot be covered by pretending its cheapest component's fake is the other real root. Each sweep must demonstrate that every configured fault fell. A seed count that fails that assertion is not a completed sweep, and is not silently raised past its allowance.

| Family / reach | Planned owner binary and layer | Existing same-world sweep used for its seed price | Proposed seeds |
|---|---|---|---:|
| 4.1 forge: busy/late/lagging/refused/oversized answers and CI blame | `temper-engine-forge-world::suite`, temper/domain | `fuzzy::landing_batches::seeded_batches_keep_the_fast_forward_prefix_and_cleanup_invariants` (64 seeds, 1..65) | 8 |
| 4.2 provider outcomes visible at the domain boundary | `smith-session-world::suite`, smith/domain | `fuzzy::simulation::random_worlds_settle_with_every_session_ended` (1,000 seeds) | 16 |
| 4.2 reset headers and endpoint-bound window, which need the byte boundary | `smith-protocol-llm-world::suite`, smith/protocol | `fuzzy::pool::random_pool_sizes_complete_in_order_without_capacity_failures` (4 seeds × 32 pool draws × 2 pool modes) | 4 |
| 4.3 worker resources, fetch/disk/scratch and placement | `temper-worker-domain-world::suite`, temper/domain; fake-checkout subcases in `temper-worker-checkout-world::fakes`, temper/domain | `fuzzy::whole::seeded_worker_delivery_and_contact_stories_replay` (16 seeds × 4 cases × 2 runs); checkout's seeded fake merges (128 seeds) | 8 + 8 |
| 4.3 untooled program, silent request and delivery-check clock | `smith-host-world::suite` and `smith-run-world::suite`, smith/domain | Host `fuzzy::simulation::random_worlds_settle_and_reach_every_ending` (240 seeds); run world's existing noisy sweep | 8 each |
| 4.4 people/parties: cancel/hold/release/amend/late or repeated answers | `jig-core-world::suite`, jig/domain | `fuzzy::faults::random_host_worlds_draw_link_store_and_cold_restart_faults` (64 seeds) | 8 |
| 4.4 pending sign-in resend | `jig-engine-clients-world::suite`, jig/real-loop today | Existing fuzzy cursor/source byte sweeps do not drive a pending exchange: **no same-world workload-sweep proxy**. Price its focused P117-style request fence separately; do not invent a per-seed cost from codec bytes. | 0 until priced |
| 4.5 stale/reordered/rewritten forge state | `temper-engine-forge-world::suite`, temper/domain | `fuzzy::opened::seeded_opens_keep_sources_untouched_and_land_only_proved_prefixes` (32 seeds) | 8 |
| 4.6 park/resume/move, including waiting coordinators | `jig-core-world::suite`, jig/domain; endpoint-window subcase in smith's protocol row above | Core's link/store/cold-restart sweep (64 seeds), preserving its root and fake workers | 8 |
| 4.7 saturated engine bounds with funds at stock policy | `temper-engine-domain-world::suite`, temper/domain | `fuzzy::walking::fuzzy_walking_shard_0` through `fuzzy_walking_shard_7` (64 seeds, two worlds each) | 8 |
| 4.8 engine restart/stop/forge unavailable/full tables | `temper-engine-domain-world::suite`, temper/domain | `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` (4 seeds, two worlds each) | 4 |
| 4.8 worker drain/crash and all simulated unit processes; older-layout fixture | `temper-one-process-world::suite`, temper/simulated | `fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` and `drawn_power_losses_keep_the_conformance_rules` (4 seeds each); `fuzzy::upgrade::drawn_power_losses_during_an_upgrade_keep_the_history_and_continue_the_story` (4 seeds) | 4 |
| All families mixed: several goals, two parties/accounts, dependent milestone, landing train and following tier | `temper-system-world::suite`, temper/system | `fuzzy::composed::seeded_system_runs_replay_turn_credit_calls_and_prepared_mounts` (24 seeds, two worlds each) | 4 |

### Prices at the proposed seeds

Every current per-seed and total figure is **unmeasured**. Where an earlier whole-binary figure B exists, B/N is used only as an **inclusive historical average-seed envelope**, since the cited report did not isolate that sweep's process CPU. It is not an observed sweep CPU or a maximum seed. The prospective upper planning price below is **2 × proposed seeds × B/N**: one proxy workload plus at most another proxy workload for faults, inherited checks and a restart/no-restart companion. This factor of two is an explicit planning assumption, not a measured ratio, and fails if the new workload is more than twice as large. There is no proven positive lower bound on the isolated sweep's share, so intervals start at zero. No affordability follows from their endpoints.

| Planned row above | Historical same-world B/N CPU s | Current price | Estimated total at proposed seeds, CPU s, earlier-report basis |
|---|---:|---|---|
| Forge 4.1, 8 seeds | at most 8.557658/64 = 0.133713 (H17) | unmeasured | 0..2.139415, inclusive forge binary proxy |
| Provider session 4.2, 16 seeds | unknown | unmeasured | unknown; no smith-session CPU in earlier reports |
| Provider byte-boundary 4.2, 4 seeds | unknown | unmeasured | unknown; do not substitute session-domain CPU for pool byte processing |
| Worker 4.3, 8 seeds | at most 0.280485/16 = 0.017530 (H17), already four cases and two runs per seed | unmeasured | 0..0.280485 |
| Fake checkout 4.3, 8 seeds | at most 2.827622/128 = 0.022091 (H17), including unrelated simulation sweep work | unmeasured | 0..0.353453 |
| Smith host/run 4.3, 8 seeds each | unknown | unmeasured | unknown; no same-world measured CPU in earlier reports |
| People 4.4 and park/move 4.6, 8 seeds each | unknown | unmeasured | unknown; jig core's old per-seed CPU is absent |
| Pending exchange 4.4 | no workload-sweep proxy | unmeasured | deferred, no execution credit and no price claimed from hostile cursor bytes |
| Stale/reordered forge 4.5, 8 seeds | at most 8.557658/32 = 0.267427 (H17) | unmeasured | 0..4.278829 |
| Saturated engine 4.7, 8 seeds | at most 45.044258/64 = 0.703817 (H17) | unmeasured | 0..11.261065, broad whole-binary envelope includes follows/removal/etc. |
| Engine restart 4.8, 4 seeds | at most 45.044258/4 = 11.261065 (H17) | unmeasured | 0..90.088516; cannot be added as if this envelope isolated terminal cuts |
| Simulated worker/upgrade 4.8, 4 seeds | at most 12.403466/4 = 3.100867 (HF7) for its older four-seed crash sweep | unmeasured | 0..24.806932; older-layout/upgrade additions are not measured by that earlier sweep |
| Composed mixed workload, 4 seeds | unknown: HW7 gives focused whole-binary 10.806115, not the 24-seed fuzzy sweep's CPU | unmeasured | unknown; dividing the focused figure by 24 would invent a fuzzy per-seed measurement |

The known envelopes are deliberately not summed: several reuse the **same whole-binary CPU** as an upper proxy for different operations. The missing jig/smith/composed prices preclude a total. Restore at every commit is not inside the factor of two when its commit count is unknown; the sampled option is the initial costing assumption. No new sweep is authorized by these estimates without the fresh figures.

The composed proxy is **not already the proposed workload**. Its random completion/commit delays and replay price the same world, but several goals, a train, two accounts and fault intervals add work. Charge the additional roots/goals and one restart-free companion for 3.5, not merely four copies of a single-chat run. Unknown scaling is shown as uncertainty, not “free because it uses the same harness.” Keep the workload and all fault assertions when choosing a smaller initial seed set.

The family sweeps belong to fuzzy. Focused adds each new behaviour's passing/finding cases, 3.3's two controls and the catalog's named regressions in the placement table, not a second large sweep. Ignored open-problem scenarios compile and cost no execution CPU until their fixing work runs them. Each exclusion names just its ignored fault/workload/state combination; an open scenario is not permission to suppress any finding of the same category. Section 7 of the failure-worlds draft names the contracts.

## What retiring and moving down frees

### Eighteen real-loop fuzzy survivors

Every path below is under `temper-real-loop-world::suite`, temper/**real-loop**, and starts with `fuzzy::`. The CPU column is **historical alone CPU**, from `limits-worlds.md`'s 2026-11-07 closing observations at `bcddfe1ff1`, not a current cost or a current saving. That report explicitly distinguishes native ordinary-patience runs from its two profiled-patience stories. Today's per-story CPU cannot be reconstructed from shared-process wall times; the current binary envelope above bounds the group.

A lower candidate is a place to move the **behavioural assertion**, not evidence that it already holds the whole path. Preserve unique assembled, kernel or wire coverage until a lower holder proves it and a minimal upper seam still holds what only that seam shows. In particular “named story in fuzzy” is a suite departure; it is not by itself proof that the test may be deleted or moved to a lower layer.

| Test path after `fuzzy::` | Historical CPU s | Unique assertion / closest lower candidate / disposition |
|---|---:|---|
| `case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | 4.254333 | Late third-worker placement, stall/timer cancel, subtree authority and Markdown-only change. Split domain placement/control assertions into engine/fleet worlds; keep three-worker whole-path evidence until replaced. No retirement credit. |
| `case_4_4::the_pace_question_reaches_the_person_and_its_answer_is_kept` | 2.247000 | Amendment reaches the next wire query and delegate uses first-ranked account. Engine domain can show choice; system/protocol must show the actual query. Unique wire coverage stays. |
| `case_4_5_limited::a_limited_account_holds_its_work_until_its_reset` | 0.772398 | Provider limit/reset holds and resumes without a person. Smith protocol plus composed system is the lower candidate; existing domain rate counters alone do not prove the wire. Stays until replacement. |
| `case_4_5_refused::a_refused_key_holds_its_work_until_it_is_replaced` | 0.862726 | Refused credential replaced and used by resumed work. Smith protocol and system candidates; unique replacement-to-next-request path stays. |
| `case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | 1.596838 | Committed unanswered question, retry elsewhere, one answer across two worker lives. Jig core plus simulated stop/restart candidate. The ignored process story is not a holder. Stays. |
| `case_5_1::a_held_director_keeps_the_owners_question_and_its_fresh_run_reads_the_new_rules` | 1.983960 | Changed guide reaches fresh system text through checkout preparation and rendered brief. Worker/engine domain and system candidates must preserve those bytes. Stays. |
| `case_5_2::a_stopped_owner_holds_its_delegates_and_its_release_resumes_them_and_its_news_waits_for_its_release_without_a_park_or_a_retired_charter` | 3.818982 | Reviewed change stops/releases/lands and news appears once after release. Engine domain plus system candidate; keep review-to-news seam. |
| `case_5_4::a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_with_an_overlap_wake_and_a_closing_comment` | 5.310823 | Cancellation against pushed head and held overlap news. Engine forge/domain can assert causal order with fake forge; real pushed-head seam stays until separately held. Not retired merely because it flakes. |
| `case_5_5::an_owners_answered_plan_starts_a_sub_owner_under_a_deadline_that_cannot_delegate_without_the_deadline_firing_or_a_fresh_run_only_at_the_milestones_end` | 1.817697 | Hosted depth refusal makes no task and fresh-run note recall recovers the plan. Engine-domain notes/delegation plus system candidate; current overlap is not a complete duplicate proof. |
| `case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | 5.430119 | Deployment-requested red-tip repair is first in forge order while queue pauses. Engine forge/domain candidate; keep request/merge-order seam until replacement. |
| `case_6_5::a_budget_question_climbs_two_owners_to_the_persons_inbox_and_a_raised_investigation_hands_its_report_to_an_advisor_without_the_director_asking_the_person` | 2.838387 | Inbox before/after answer, next-run effort and advisor report/turn; earlier inventory found no clearly unique assembled path. Candidate to split among engine-domain question/effort/role tests, but no exact complete-holder proof yet, so no credited retirement. |
| `case_6_6::a_kit_change_lands_before_the_apps_early_repin_without_relay_without_the_a3_a4_epilogue` | 4.328254 | Atomic batch across repositories, early producer and pin bytes against merge. Engine domain/forge candidates; retain cross-repository tree evidence. |
| `case_6_7::a_sub_owner_past_its_deadline_is_held_at_its_next_run_and_escalated_through_its_owner_to_the_director_who_splits_it_by_amendment_and_release_without_a_grace_for_its_delegates` | 2.448953 | Real-clock wake of parked work, escalation and amendment/release. Domain shows policy but not the real timer mechanism. Unique real-clock seam stays in real-loop. |
| `case_6_8::m1s_end_message_starts_m2_beside_a_tester_without_forge_effects_whose_test_lands_only_through_a1s_change` | 3.572302 | Refused tester forge call has no effect, message starts concurrent work, report lands only through A1. Engine domain plus system candidate; preserve all observations. |
| `green_light::a_green_light_in_the_inbox_lets_the_change_land` | 5.977125 | B0 independent recorded-model history, advisor goal and delivered stock review brief. Keep unique transcript history; not a scripted-story duplicate. |
| `landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | 2.154811 | Dependency added to report delegate, first run after real Forgejo landing. Engine-domain added-dependency holder covers the policy; actual forge seam stays. |
| `program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | 2.165572 | Claude Code backend through a whole change and Read/Edit activations. Keep unique backend path. |
| `repair::a_change_failing_ci_is_repaired_and_lands` | 7.350897 | B1's six independent recorded-model transcripts, repair wording and bounded outcome. Keep history; not replaced by a scripted repair. |

The older report already retired four stories only after nineteen holders were selected alone and passed. In particular its focused engine-domain holders include `root_routes::timer_cancellation::an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation`, `root_routes::a_note_written_corrected_by_a_person_and_recalled_is_durable`, and the four named `root_routes::added_dependencies` cases. Those are examples of a **demonstrated** transfer, not four further savings available now. Do not count them again.

### Eleven unchanged engine-domain imports

`temper/tests/engine/domain/tests/suite/main.rs` includes `removal.rs` and `reset.rs` a second time under `fuzzy`, without changing their arguments. Every row below is in `temper-engine-domain-world::suite`, temper/**domain**. The matching focused path is the same suffix **without `fuzzy::`**. Retire only the fuzzy registration; the focused case and every assertion/cut/seed stay. **Each test's current CPU and the group's saving are unmeasured.** H17's whole engine-domain fuzzy CPU is 45.044258 s; a historical exploratory interval for all eleven together is 0..45.044258 s, not that interval eleven times. For every row the current CPU is unmeasured and the only earlier CPU envelope is that shared group interval; no alone CPU exists in these reports. Credited saving in the balance is **zero pending measurement**, not a claim that retirement saves no work. This frees fuzzy domain CPU and no focused CPU. It is not a layer move and cannot fund any other layer by itself.

| Fuzzy test path | What is unchanged / disposition |
|---|---|
| `fuzzy::removal::removal_cut_after_decision_ends_whole` | Same cut, seeds `[1, 7, 71]`, paired `run`; retire fuzzy duplicate. |
| `fuzzy::removal::removal_cut_after_secrets_erased_ends_whole` | Same cut/seeds/paired run; retire. |
| `fuzzy::removal::removal_cut_after_box_removed_ends_whole` | Same cut/seeds/paired run; retire. |
| `fuzzy::removal::removal_cut_after_task_ended_ends_whole` | Same cut/seeds/paired run; retire. |
| `fuzzy::removal::removal_cut_after_chat_erased_ends_whole` | Same cut/seeds/paired run; retire. |
| `fuzzy::removal::a_removed_forges_number_is_never_given_again` | Identical fixture, crash 91 and number assertion; retire. |
| `fuzzy::removal::an_ended_chat_reads_its_removed_forges_repository_until_its_project_is_deleted` | Identical retained-name/readback and crash path; retire. |
| `fuzzy::reset::the_reset_deletes_the_project_with_the_chat_templates_grace` | Same driver and committed grace assertion; retire. |
| `fuzzy::reset::reset_cut_after_project_ends_whole` | Same cut, seeds `[1, 7, 71]`, paired `run`; retire. |
| `fuzzy::reset::reset_cut_after_forge_ends_whole` | Same cut/seeds/paired run; retire. |
| `fuzzy::reset::reset_cut_after_account_ends_whole` | Same cut/seeds/paired run; retire. |

Three further literal helper calls in `fuzzy::landing_settings` also duplicate focused execution. Each current CPU is unmeasured; each shares the same H17 0..45.044258 s envelope with the eleven above, with **no extra additive credit**: `landing_values_wait_for_the_next_start_then_prove_the_turn` calls `forge_routes::landing::go_live::a_project_with_its_own_roles_goes_live_by_turn_with_running_stays`; `the_operator_holds_queue_and_turn_until_restart` calls `forge_routes::landing::the_operator_hold_waits_by_queue_and_turn_until_restart`; `the_operator_holds_the_goal_closers_land_step` calls `forge_routes::goal_closer::the_operator_hold_stops_the_goal_closers_land_step`. They add no draw. Retire their fuzzy wrappers, preserving all three focused holders. Credit is fuzzy temper/domain only, and bounded together with, not added on top of, the binary's current CPU envelope.

### One-process stories

All paths here are in `temper-one-process-world::suite`, temper/**simulated**. **Each path's current CPU is unmeasured.** HW7 gives the older whole-binary focused/fuzzy medians 27.899207 / 11.275533 s, and HF7 gives fuzzy 11.800430–12.403466 s. Those are group envelopes, not per-story CPU; in particular they predate the later upgrade scenarios, whose individual costs remain unknown. The four duplicate upgrade wrappers have no attributable earlier alone CPU, so their savings are unknown. For the old chat/merge behaviour candidates an exploratory group saving is 0..12.403466 s, before paying for the lower replacement, not a guaranteed debit of the simulated layer. Preserve the real simulated-disk/start/upgrade/whole-service observations; domain assertions alone do not replace those mechanics.

| Path(s) | Disposition and lowest candidate |
|---|---|
| `fuzzy::upgrade::power_lost_at_the_copys_first_write_keeps_a_whole_generation`; `power_lost_at_the_patch_commit_keeps_a_whole_generation`; `power_lost_at_the_switch_rename_keeps_a_whole_generation`; `power_lost_at_the_wires_commit_keeps_a_whole_generation` (all after `fuzzy::upgrade::`) | Exact `power_loss(0..3)` calls already execute under focused `upgrade`: the second focused holder is named `power_lost_at_the_seal_keeps_a_whole_generation` but calls the same `power_loss(1)`. Retire these four fuzzy wrappers; keep the focused simulated proofs. No layer moves. |
| `fuzzy::upgrade::drawn_power_losses_during_an_upgrade_keep_the_history_and_continue_the_story` | Four drawn cuts, seeds 1..=4, unlike focused seed 5. Keep: new cut locations and history/continuation proof, not duplication merely because both call `drawn_power_loss`. |
| `fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` | Four seeded kill cuts with pending adoption and at-most-once answer readback. Keep simulated: core alone does not show disk/service restart. |
| `fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules` | Four seeded power-loss cuts, pending adoption and drained ending. Keep simulated; loss semantics are distinct from kill. |
| `fuzzy::one_process::a_chat_reads_then_replies_and_waits` | One unchanged `fixtures::SEED`, no sweep. Candidate to move behaviour to `temper-system-world::suite` (system), preserving read/reply/wait and sign-in observations; keep a minimal simulated shipping seam if lower translation cannot show it. Not the same scenario as focused `Story::Chat`, so not an unconditional duplicate. |
| `fuzzy::one_process::a_merge_in_progress_is_pushed_with_both_parents` | One unchanged `fixtures::SEED`, no sweep. Compare `temper-system-world::suite`'s `merge_job::a_changes_merge_job_runs_on_its_worker`: it already holds real worker-domain routing and fake-git parents across five rows. Move only after it proves this in-progress merge's two parents; simulated process scheduling is not itself the asserted rule. |
| Focused `across_versions::*`, `github::*`, `golden::*`, `public::*`, `push::*`, `ready_copies::*`, `story_a::*`, `replay::*`, `m15::*`, `held_program::*`, `upgrade::*` and signal/restart cases in `one_process::*` | Keep unique version bridging, hosted git credentials, golden-store startup, public cache/proxy/wire, push, copy order, full story/log, redaction, held program and upgrade/stop seams. Move a pure assertion down only with a named lower holder, not this whole group as a budget shortcut. |
| Focused `one_process::a_secret_in_a_root_fails_the_referee_naming_it` and `one_process::a_restart_showing_part_of_a_commit_fails_the_referee` | Pure supplied-value referee controls: candidate step binary, not a simulated service. Preserve literal token/partial-commit outcomes. This is a real layer move if split out. |

### Existing sweeps that share focused helpers

Sharing helpers is not proof of duplication. **Every sweep's current CPU is unmeasured**; earlier whole-binary envelopes below are shared across its sweeps, not attributed test costs. H17 provides engine-domain 45.044258, forge 8.557658, conformance 43.811881, worker-domain 0.280485 and fake-checkout 2.827622 CPU s. HW7/HF7 give the one-process figures above. For an individual sweep its historical exploratory interval is 0..its whole-binary envelope; the retirement credit is zero because its additional inputs/coverage stay. `limits-fuzzy.md`, *Figures and unchanged draws*, records why restart/readback and lockstep work was retained. The following temper/domain sweeps stay unless F7 demonstrates all their additional assertion inputs elsewhere:

- `temper-engine-domain-world::suite`: `fuzzy::walking::fuzzy_walking_shard_0` through `fuzzy_walking_shard_7` retain 64 seeds, two lockstep worlds, delayed store/pages and both commit cuts; `fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` adds four terminal cuts. Keep. A fixed focused walk is not their timing/trace/state oracle.
- Same binary: `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` crosses five endings with three durable cuts; `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` crosses three real starting states, two owners and a cut. Keep the complete matrices; repeated fixtures are not repeated inputs.
- Same binary: `fuzzy::follows::fuzzy_follows_shard_0` through `fuzzy_follows_shard_3` call focused follow helpers but add sixteen seeds of 24 drawn actions, a red across restart, owner changes and exact state/trace replay. Keep.
- `temper-engine-forge-world::suite`: `fuzzy::branch_patterns::branch_patterns_match_full_patterns_across_repositories`, `fuzzy::rules_by_path::path_rules_agree_with_the_independent_pattern_statement` and `fuzzy::member_message::arbitrary_member_words_preserve_message_bounds_and_trailers` add arbitrary branch/path/message bytes and independent expectations. Keep; potential **step** placement is separate from retirement, since they mostly call pure functions with one fixture setup.
- `temper-conformance-world::suite`: `fuzzy::stories::*` and `fuzzy::shared::*` execute a focused story at **each live durable commit**, asserting the forked commit list. Keep domain: one final restart does not prove each commit cut. `fuzzy::shared::a_shrinking_pool_is_not_applicable` is a fixed applicability control, not a fault sweep; candidate step placement, not credit for all conformance CPU.
- `temper-worker-domain-world::suite`: `fuzzy::whole::seeded_worker_delivery_and_contact_stories_replay` and `fuzzy::runtime::runtime_random_turn_sizes_and_tiny_windows_settle` add seeded whole-worker replay, four delivery/contact cases and tiny retained-turn/body windows. Keep. Checkout's `fuzzy::merges::seeded_merge_worlds_preserve_complete_trees_and_parent_graphs` likewise adds random trees and graph structure, not merely more executions of an identical tree.

No blanket removal of “sweeps that call focused helpers” is proposed. The demonstrated removals above are the unchanged registrations and literal same-argument calls. Every group's current CPU envelope is charged only once, and a range of possible savings is not a guaranteed credit.

## The moves proposed

A move has exactly `from`, `to`, `cpu_s`, `why`, under its suite; it goes strictly down `tools/tiers/layers.toml`'s ordering and preserves the sum of ceilings (`tools/tiers/budgets.md`, section 10). Duplicate retirement **within one layer** is not a move. Nor can a retained unique real-loop story donate its entire historical CPU because part of its policy is expressible below.

For each layer let measured baseline be M, demonstrated retirement credit D, lower replacement work L, existing-test check increment H, new sweeps S and proposed net ceiling transfer T. The after balance is **ceiling + T − (M − D + L + H + S)**. Before is ceiling − M. Show the whole uncertainty interval; a partial baseline gives only an upper bound on room. A proposed ceiling transfer is not CPU saved by an implementation. Main's unknown held budget may make less available still.

### Candidate downward moves, for decision only after fresh measurement

These are **conditional requests**, resting on HW7's survivor/one-process inventory, H17's binary envelopes and the source-level duplicate reading above. They are **not funded moves established by today's run**, and no entry is written to `budgets.toml`. The 12/20 s choices reserve only part of the earlier real-loop group (HW7 60.139001 s fuzzy median, HF7 61.052666–66.214588 s), not all eighteen stories. The 2 s system and 0.1 s step requests are small provisional allocations within the older one-process group, **not measured marginal replacement costs**. A fresh run may show that none is available. No numeric jig/smith/skein transfer is defensible without their missing prices.

| Suite | from | to | cpu_s | why |
|---|---|---|---:|---|
| default | `temper.real-loop` | `temper.domain` | 12 | Move the engine policy/ordering assertions of the scripted reference-case stories to engine domain/forge holders, retaining only their unique upper seams; HW7 identifies the lower candidates and unique observations. |
| default | `temper.simulated` | `temper.step` | 0.1 | Split the two supplied-value referee controls out of the one-process binary, preserving their literal secret/partial-commit outcomes; HW7's older whole-binary 27.899207 s is only a coarse source envelope. |
| fuzzy | `temper.real-loop` | `temper.domain` | 20 | Put the scripted stories' transferable policy assertions in seed-driven engine worlds only after a lower holder proves each; retain transcript/backend/real-timer evidence, as HW7's survivor inventory requires. |
| fuzzy | `temper.simulated` | `temper.system` | 2 | Move chat read/reply/wait and in-progress merge-parent assertions to the composed system world if its holders prove them, preserving simulated restart/upgrade mechanics; HF7's 11.800430–12.403466 s is only the older whole-binary envelope. |

A move is rejected if its upper scenario still needs the same full execution or the lower holder omits an assertion. The eighteen historic alone CPUs are not summed into an available budget. The duplicate imports/upgrade wrappers free their **own** layer only; they are not listed as inter-layer moves. The user may choose smaller moves or none. Without a measured donor balance these entries cannot be said to pay for all checks/sweeps. In particular the unpriced jig/smith/settings/composed work prevents a finite financed total.

### Layer balances before and after those candidate moves

Every current **before and after balance is unmeasured**; this is the outcome for each tree/layer, not a blank or an assumed zero. The numeric columns are only the known ceilings before/after the conditional transfers. For each cell substitute its own measured M, D, L, H and S in the formula above once fresh reports exist. The table makes the ceiling arithmetic auditable without treating capacity as CPU observed or saved.

| Tree / layer | default ceiling before → after, CPU s | fuzzy ceiling before → after, CPU s | Current balance before → after, default; fuzzy |
|---|---:|---:|---|
| skein/step | 7 → 7 | 32 → 32 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/domain | 2 → 2 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/machine | 5 → 5 | 43 → 43 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/protocol | 7 → 7 | 23 → 23 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/io | 2 → 2 | 21 → 21 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/simulated | 3 → 3 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/real-loop | 7 → 7 | 20 → 20 | unmeasured → unmeasured; unmeasured → unmeasured |
| skein/end-to-end | 4 → 4 | 1 → 1 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/step | 7 → 7 | 3 → 3 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/domain | 3 → 3 | 16 → 16 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/system | 2 → 2 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/protocol | 3 → 3 | 3 → 3 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/simulated | 11 → 11 | 13 → 13 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/real-loop | 2 → 2 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| smith/end-to-end | 21 → 21 | 3 → 3 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/step | 47.5 → 47.5 | 4.1 → 4.1 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/domain | 60.6 → 60.6 | 68 → 68 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/machine | 2 → 2 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/protocol | 6.2 → 6.2 | 30.4 → 30.4 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/simulated | 59.7 → 59.7 | 37.5 → 37.5 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/real-loop | 2 → 2 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| jig/end-to-end | 2 → 2 | 1 → 1 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/step | 253 → 253.1 | 13 → 13 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/domain | 137.5 → 149.5 | 80.8 → 100.8 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/system | 111 → 111 | 13 → 15 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/machine | 4 → 4 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/protocol | 9 → 9 | 2 → 2 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/simulated | 470 → 469.9 | 196 → 194 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/real-loop | 81.5 → 69.5 | 195.2 → 175.2 | unmeasured → unmeasured; unmeasured → unmeasured |
| temper/end-to-end | 39 → 39 | 1 → 1 | unmeasured → unmeasured; unmeasured → unmeasured |
| hearth/step | 2 → 2 | 1 → 1 | unmeasured → unmeasured; unmeasured → unmeasured |
| hearth/domain | 4 → 4 | 32 → 32 | unmeasured → unmeasured; unmeasured → unmeasured |
| hearth/machine | 13 → 13 | 50 → 50 | unmeasured → unmeasured; unmeasured → unmeasured |
| hearth/real-loop | 11 → 11 | 9 → 9 | unmeasured → unmeasured; unmeasured → unmeasured |
| hearth/end-to-end | 2 → 2 | 1 → 1 | unmeasured → unmeasured; unmeasured → unmeasured |

As a **historical sensitivity check only**, applying the candidate ceilings to H17's baseline temper/domain would leave default **−11.771829 s** and fuzzy **+0.278096 s**, **before** any new checks, sweeps or replacement costs and with no retirement credit. The broad historical engine/forge/worker/checkouts check envelopes already exceed that fuzzy margin. Thus even the historical sample does **not** establish that these transfers pay for the milestone. Focused protocol's old −2.391424 s has no proposed transfer and still needs its own repair. Main's present held budgets and the donor layers' complete current figures are unknown. No layer is declared within budget.

## For the user

- **The plan's figures need a fresh run of the focused and fuzzy suites before any move is decided.** Current CPU, balances and affordability remain unmeasured after Infra: disk.
- Decide only the explicit per-suite downward moves, not a blanket authority to add or raise figures.
- Authorize retirement of the eleven unchanged imports, three same-argument landing-settings wrappers and four duplicate upgrade-cut wrappers, while keeping their focused holders.
- Keep every unique real-loop/backend/transcript and simulated restart/upgrade seam until its replacement is demonstrated; decide suite placement separately from layer and CPU.
- Choose sampled 3.7 initially outside verified commit-cut cases, with checked/total commits reported; exhaustive checking everywhere needs its own measured allowance.
- Approve initial sweep seeds only with configured-fault-fell assertions and named exclusions; no broad suppression and no silent seed/patience raise.
- Treat unknown per-operation or per-test CPU as unknown; an inclusive process envelope is not an observed marginal cost or a guaranteed saving.
- Recheck this CPU plan when the instruction gate lands; this report neither sets its instruction figures nor forecasts a counter-derived budget.
