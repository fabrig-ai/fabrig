# limits: status

2026-11-12. Where the tree's code lags or departs from [limits.md](limits.md).

- **Not yet:** "A project's dependencies": `forge.dependencies` and its
  profiles' values [in flight: change 1042].

- **Not yet:** "Two profiles, as jig's", a change's policy's `reruns`: no profile or override names `forge.change_policy.reruns` (`temper-service/src/limits.rs`; `limits/names.rs`) [in flight: change 601].

- **Not yet:** "Limits checked against each other at start", the checkout's capacity, whole: the start checks git's stdout rule, the agent's conflicts and the channel's parts each in its own phase, returning at the first that refuses, and jig's `saved` ceiling, 65,536, refuses `checkout.conflicts` past about 120 at 255 path bytes, named `engine.limits.work_saved_some` with no checkout setting (`temper-service/src/startup.rs`, `checked_start_with`; `startup/checks.rs`, `channel`) [in flight: change 41].

- **Not yet:** `forge.changes` bounds every change ever made, not the changes kept at once: the forge's change table never drops one (`../../domain/forge/change/README.md`, 7) [in flight: change 391].

- **Not yet:** `forge.changes`, the landed commits: the forge's `landed` table is never pruned but by a dropped change, and a merge finding it full moves no branch head (`../../domain/forge/change/README.md`, 7) [in flight: change 476].

- **Not yet:** "A limit's refusal names it", a change's registration: past `forge.changes` or `forge.name_bytes` it is refused with the bare `Refused`, held as "forge resource request refused; check resource access and configured bounds", naming no limit [in flight: change 391].

- **Not yet:** "A limit's refusal names it", every other step of the forge's: past its holds, entries, subscriptions, paths, names, gates, steps, watches or landing keys it refuses with the bare `Refused`, naming no limit [in flight: change 392].

- **Not yet:** "Limits checked against each other at start", the store's tier rows': nothing checks `forge.changes` against them, so a value past 128 is refused as the store's map sizing past its codec or arithmetic bounds, naming no limit [in flight: change 393].

- **Not yet:** "`forge.changes` bounds the changes kept at once": its bound is 128, not 1024, while the store schema's tier-row ceilings stay 257 (`../store.md`, 4) [in flight: change 470].
