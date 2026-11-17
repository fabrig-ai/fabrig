# limits: status

2026-11-17. Where the tree's code lags or departs from [limits.md](limits.md).





- **Not yet:** "One stock", a change's policy's `reruns`: no profile or override names `forge.change_policy.reruns` (`temper-service/src/limits.rs`; `limits/names.rs`).


- **Not yet:** "Limits checked against each other at start", the checkout's capacity, whole: the start checks git's stdout rule, the agent's conflicts and the channel's parts each in its own phase, returning at the first that refuses, and jig's `saved` ceiling, 65,536, refuses `checkout.conflicts` past about 120 at 255 path bytes, named `engine.limits.work_saved_some` with no checkout setting (`temper-service/src/startup.rs`, `checked_start_with`; `startup/checks.rs`, `channel`).


- **Not yet:** `forge.changes` bounds every change ever made, not the changes kept at once: the forge's change table never drops one (`../../domain/forge/change/README.md`, 7).

- **Not yet:** `forge.changes`, the landed commits: the forge's `landed` table is never pruned but by a dropped change, and a merge finding it full moves no branch head (`../../domain/forge/change/README.md`, 7).

- **Not yet:** "A limit's refusal names it", a change's registration: past `forge.changes` or `forge.name_bytes` it is refused with the bare `Refused`, held as "forge resource request refused; check resource access and configured bounds", naming no limit.

- **Not yet:** "A limit's refusal names it", every other step of the forge's: past its holds, entries, subscriptions, paths, names, gates, steps, watches or landing keys it refuses with the bare `Refused`, naming no limit.

- **Not yet:** `forge.changes`, its bound of 768 within the tier rows' 1,537: the store schema's ceilings stay 759, so the start's check bounds it at 379 (`../store.md`, 4) [in flight: change 7113].

- **Not yet:** the stock's `forge.changes` 768 and `forge.tiers.test_id_bytes` 512: the stock holds 379 and 256 [in flight: change 6959].

- **Not yet:** "Limits checked against each other at start", the store's failing tests' ids: nothing checks `forge.tiers.test_id_bytes` against the store's ceiling before its sizing, which refuses one past it naming a schema field [in flight: change 6959].


