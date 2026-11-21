# catching-up: status

2026-11-20. Where temper's code lags or departs from [catching-up.md](catching-up.md).

- **Not yet:** 2: the forge reads afresh and settles before the engine opens; no `Event::CatchUp` [in flight: change 11034].
- **Not yet:** 2 and 3: the halves run one after the other, the outbox paused through the first, and the line asks no one before the settle [in flight: change 11036].
- **Not yet:** 4: `step_verified`, `change_facts`, read sessions, `goal_branch::base_wait`, `standing::judge_held_landing`, `tiers`, `follow_check` and `follows` decide on restored facts once they step [in flight: change 11022].
- **Not yet:** 4: `continue_release_inner` and its parts, `design::design_claim_permitted`, `items`, `written`, `project`, `rebase::read_afresh` and `landing::judge` decide on restored facts once they step [in flight: change 11024].
- **Not yet:** 4: the root's `form_landings` and `limit_holds` wait for the start's phases, not the forge's fresh reads [in flight: change 11034].
- **Not yet:** 2 and 4: procedures wait for the whole forge's fresh reads (`fresh_reads_in`), not their repositories'; no `repository_fresh`, and they are woken only at the forge's report [in flight: change 11537].
- **Not yet:** 5: no `CatchingUp`; the forge's row and the landing queues say nothing of a forge catching up [in flight: change 11026].
