# limits: status

2026-11-26. Where the tree's code lags or departs from [limits.md](limits.md).

- **Not yet:** 4: `setup.fetch_bytes` bounding a packed binary's
  unpacked bytes; the fetch still bounds a bottle's extraction with it
  [in flight: change 18878].






- **Not yet:** "One stock", a change's policy's `reruns`: no profile or override names `forge.change_policy.reruns` (`temper-service/src/limits.rs`; `limits/names.rs`).





- **Not yet:** `forge.changes` bounds every change ever made, not the changes kept at once: the forge's change table never drops one (`../../domain/forge/change/records.md`, 1).

- **Not yet:** `forge.changes`, the landed commits: the forge's `landed` table is never pruned but by a dropped change, and a merge finding it full moves no branch head (`../../domain/forge/change/records.md`, 1).

- **Not yet:** "A limit's refusal names it", a change's registration: past `forge.changes` or `forge.name_bytes` it is refused with the bare `Refused`, held as "forge resource request refused; check resource access and configured bounds", naming no limit.

- **Not yet:** "A limit's refusal names it", every other step of the forge's: past its holds, entries, subscriptions, paths, names, gates, steps, watches or landing keys it refuses with the bare `Refused`, naming no limit.
