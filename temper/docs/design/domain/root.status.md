# root: status

2026-11-07. Where temper's code lags or departs from [root.md](root.md).

Section 9.1's owner handoff is present: after both domains load, the root
hands the forge `people.sole_owner()` before its restore decision. The
connector's retained-link scope gap remains in `forge.status.md`, 4.6.

Section 5.2's host-answer room is shared across the box's runs, as
`fleet.calls` bounds the fleet rather than each run. No concurrency gap is
known in that room; the settings-answer bound gap remains below.

Section 10 has no known gap in deriving both watch rooms from the admitted
streams. Its settings-answer bound gap remains below.

- **Not yet:** 10: a change of settings refused for a charter's stored
  size names neither the size nor the bound: jig's refusal carries no facts
  for it (`jig-domain-core/src/settings/check.rs`, `parts_fault`), and ctl
  prints `charter 5: shape`.
- **Not yet:** 5.2: what the room saves is unobserved: no goal on record has
  counted its `read_task` calls since the room landed, where the request
  `read-task-room` of 2026-11-06 takes a real goal's next count as its proof.
  The engine world holds the bound on a script alone
  (`temper/tests/engine/domain/tests/suite/host_calls.rs`).
- **Design departs:** 5.2, from the same request, decided by its plan's cut: a
  process with no local agent's metadata names `documents.text_bytes`, where the
  request has a run read up to what its reply holds, because an engine alone does
  not know its remote agents' reply bound, and an answer past it reaches the run
  as smith's `TooLarge` at every offset, so a long spec could not be read at all
  (`agent.md`, section 10 keeps it open).
- **Not yet:** 7.2: a section for each
  document a charter names (the forge's brief sources are CI, reviews, the
  pull request and the home: `temper-engine-domain-forge/src/boundary.rs`,
  `BriefSource`), and a merge-queue ejection's reason. 9.1: the
  rehearsing root, `temper upgrade --check`.
- **Not yet:** 10: the settings-answer bound must include temper's declared
  application kinds, because `temper-engine-domain/src/limits.rs`, `valid`,
  passes an empty kinds slice to the core's `reads::answer_bound`, while
  `decision.rs`, `host_within`, relies on that bound for settings output
  held before its commit. The application settings' contribution is omitted;
  no reason is recorded.
- **Departs:** 12: a duplicate person's identity panics at its link,
  naming its row (`jig-domain-people/src/domain/start.rs`, `fits`;
  `tests/engine/domain/tests/suite/root_routes.rs`,
  `a_duplicate_person_identity_panics_at_its_link_naming_its_row`),
  where the user's request of 11-04 counts that panic a bug, because
  `jig/docs/design/domain/engine.md`, 6 has a start judge no record and
  panic at a link it cannot make (the user, 10-28). Which holds is the
  user's, unanswered (that document's section 16).
