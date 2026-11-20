# reconciling-start: status

2026-11-20. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design lags:**
  - 4: `../domain/root.md`, 8.2, hears a stop only once the root is ready; `../../help/server.md` sets no unit timeout from the engine's bounds.
  - 5: no design names a start's waiters, or a catching-up forge's past a report bound by its oldest; `../domain/forge/catching-up.md`, 5, shows its counts alone.
- **Design departs:**
  - 3, 5: the forge's work after the opening is "catching up", its report "caught up" and its pages' words "Catching up", where the draft says "reconciling" and "reconciled", as the goal that built section 3 was asked (2026-11-20; its designer's choice on the request's words).
  - 3: a procedure that waits on the forge waits for its fresh reads, the first half of catching up, not for the whole: one uncertain write whose find keeps failing would otherwise hold every procedure, which the draft's own test 4 rules out (`../domain/forge/catching-up.md`, 2 and 4).
  - 3: a write waits behind every earlier entry on its lane, the outbox's order in steady state, not only until each restored one has had its first look: the lane is the fence, and no second order is kept (`../domain/forge/catching-up.md`, 3; skein's `forge/client.md`, 7.2).
- **Open:** none.

Section 2's check of every term that ends reading afresh (S1, 2026-11-20;
`temper/crates/temper-engine-domain-forge/src/domain.rs`, `read_afresh_complete`):

- **Reads alone:** the forge client's fresh pass, the landings' startup
  gathers, the pending CI reads, the goal files' reads and the lost
  writers' reads. None waits for an outbox place, an entry number, the
  outbox's readiness, a write's outcome or a task's release. While the
  outbox is paused no entry is sent or holds a call slot, so a read
  answered `Busy` waits on other reads alone (skein's
  `forge/client.md`, 8).
- **The one that waited:** the goal branches' makings. A held branch
  found missing with no entry asked the root for an entry number during
  the read, then waited in the outbox's line, which asks no one before
  the settle. A making now counts only while it reads, and its write
  waits in the line (`../domain/forge/branches.md`, 1).
- **No term of their own:** jig's start script and temper's root add
  nothing to the phase but the forge's report that it read afresh.
- **Left to section 3:** each of these reads is asked again, with no
  bound, while its forge stays away. That is a wait on a read, not on a
  write, and section 3 takes it out of the start.

Section 3's audit (S3, 2026-11-20, before the start changed): every
procedure that decided on a forge fact without its own read, relying on
the start's barrier, and what it waits on now
(`../domain/forge/catching-up.md`, 4):

- **On the forge's fresh reads:** `step_verified`; `change_facts`
  outside a gather; a read session's stall clock (`read_session`,
  `hold_read_session`); `goal_branch::base_wait`; `pull_seen_closed`
  and `closing_pull` on a change's release; `standing::judge_held_landing`;
  `continue_release_inner` with `release_pull`, `settle_prior_effects`,
  `goal_branch::landed` and `design::unpublished_branch`;
  `design::design_claim_permitted` and a run's write claim;
  `items::workspace_item`, `items::task_workspace_item` and
  `items::design_workspace_waiting`; `project`, a goal's issue
  projection; `landing::judge`; the root's `form_landings`, where
  forming reads a forge fact; the root's `limit_holds::reconcile` and
  `room_freed`.
- **On the fresh reads or their own read:** `tiers::restart` and
  `tiers::start_due`; `follows::resume`, `follows::next` and
  `follows::finish_range` (`follows::observed`); `written::defer`,
  `record_written_head` and `confirm_written_head` (`written::restart`'s
  comparison).
- **On their own read:** `rebase::read_afresh`, which reads the branch
  rather than the client's cached tip.
- **On their entries:** a tier's step after a restart (`tiers::step`,
  its `finish`), with the fresh reads too; `follow_check::advance` with
  a restored flight.
- **On the line and the lane:** `goal_branch::make` and `room::check`,
  whose write the paused outbox alone held; `resolve`, a person's choice
  on a held entry.
- **Shown, deciding nothing:** the landing queues, a change's panel and
  the views, with the catching-up line until the forge has caught up.
- **Already on their own reads:** a change's step (`step_change`,
  `step_read`, `topics::refresh`, `ci_refresh`), `closer::check`,
  `topics::classify`, `drafts`, `held`, `adoption::adopt`,
  `making::making_due`, the landings' gathers, the goal-branch makings'
  reads, `removal::restart`, `reds`, `ci_observed`, and the pure crates.
