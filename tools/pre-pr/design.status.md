# design: status

2026-11-28. The worker delivery path and its offline acceptance target are
described in [design.md](design.md), sections 1 and 9. Close runs candidate
assertions from the snapshot and selects the published budget baseline in
the source checkout. Sections 2–8 retain historical detail for interpreting
earlier receipts.

- **Not yet:** 2: pre-PR still has its goal and child modes (`--goal`, `--child`), their wrappers `.temper/goal-pre-pr` and `.temper/goal-child-pre-pr`, the gate's scope and pre-PR's hook list naming them, the replay of a failing test's process with its `temper/rerun` read, and the workflows `.forgejo/workflows/goal-pre-pr.yml` and `ci.yml` that ran the goal mode on a CI runner (sections 1, 3 and 4 say they are gone).
