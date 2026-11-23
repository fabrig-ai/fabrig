# drift: status

2026-11-22. Where the tree's code lags or departs from [drift.md](drift.md).

- **Not yet:** 1: a goal's runs push to its run
  branch r, and a `"branch": "run"` change lands r into g; g records no
  design writers, so a push to g by the goal's run would hold the goal as
  drift and a landing change on g as "branch moved"; a goal's run prepared
  before g is made and confirmed starts on the base, its run unable to
  deliver (`temper-engine-domain-forge/src/items.rs`,
  `task_workspace_item`).

- **Not yet:**
  - 1: a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
- **Not yet:** 1: containment is read as a commit list or a
  comparison, not a `Range`: the change step's up-to-date and catch-up
  ranges (`Read::Commits`, `temper-engine-domain-forge/src/domain.rs`,
  `ancestry_status`), a landing member's main (`landing.rs`) and a lost
  run's observed head (`domain.rs`, `lost_read`) (skein's `forge/vocabulary.md`,
  1.2).

- **Not yet:** 1, *Its own writes are its own*: an update's made head is not recorded as its branch's head, so a goal's landing on its held branch takes it for drift and holds `BranchMoved`; a let-go update is judged against the base's tip read now, not the base it asked for; and a close-out's or catch-up's merge into the goal's branch takes no writer slot the landing sees (`temper-engine-domain-forge/src/domain.rs`, `change_outcome`, `change_facts`; `temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`).
