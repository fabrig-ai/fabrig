# reconciling-start: status

2026-11-22. Where the design lags or departs from [reconciling-start.md](reconciling-start.md).

- **Design departs:**
  - 2: a making's read belongs to read-afresh only while it is out: one the forge client refuses for room, or one waiting out its forge, waits in the connector's read line and is asked in its turn, the start's fresh reads ending without it, and the branch's own readers wait for its making instead (`../domain/forge/branches.md`, 1; `../domain/forge/keeping-up.md`, 3). draft-7 asked for a start that goes on with the making retried after it (2026-11-22): a read held for room held its repository's fresh reads, and every procedure on the goal's home with them.
  - 3, 5: the forge's work after the opening is "catching up", its report "caught up" and its pages' words "Catching up", where the draft says "reconciling" and "reconciled", as the goal that built section 3 was asked (2026-11-20; its designer's choice on the request's words).
  - 3: a procedure that waits on the forge waits for its fresh reads, the first half of catching up, not for the whole: one uncertain write whose find keeps failing would otherwise hold every procedure, which the draft's own test 4 rules out (`../domain/forge/catching-up.md`, 2 and 4).
  - 3: a decision waits for its repository's fresh reads, not its resource's: resources within one repository share its wait, so one resource still unread holds every procedure on that repository and none on another. The goal's requester chose it (2026-11-21) over waiting for the whole forge; a wait per resource is Later (`../domain/forge/catching-up.md`, 2, 4 and 6).
  - 3: a write waits behind every earlier entry on its lane, the outbox's order in steady state, not only until each restored one has had its first look: the lane is the fence, and no second order is kept (`../domain/forge/catching-up.md`, 3; skein's `forge/client-effects.md`, 1.2).
  - 7, test 2: a making found missing with room is made in the room line's order from the beginning of catching up, its write behind its lane, not after the settle: section 3 has the line order writes as soon as the outbox's table is restored, which the test's "after the settle" predates; what the test guards, that no read waits on the making's write, holds (`../domain/forge/branches.md`, 1).
  - 6: of the revisions the draft asks for, `../protocol/binaries.md`, 2 alone is made: the redeploy tool's design it names beside it left the repository with the tool, a deployment's redeploy being its operator's, outside fabrig, which reads the rehearsal's pass from the check's opening line (the user, 2026-11-20).
  - 6: the check has no bound of its own past which it fails: the start's work up to the opening is local and finite, and a bound would be a timeout the user has not set (`../protocol/binaries.md`, 2).
  - 4: the bounds are printed by `temper-engine versions`, not `--version`: `--version` stays the one line, `NAME VERSION (COMMIT)`, that the binary's test and an operator's staging read whole, and `versions` is the engine's version details, reading no configuration, store or settings, so it prints the bounds at the stock limits (`../protocol/binaries.md`, 2). The goal's designer, 2026-11-20.
  - 5: the report bound is one fixed figure, 30 s, for every step of a start and every full table, not one derived from each step's work on the deployment's store: `versions` reads no store, and dev.fabrig.ai's whole start took ten seconds once its cycle was cut (`../protocol/configuration/derived.md`, 5.6). The goal's designer, 2026-11-20.
  - 5: "the health census" and "the deployment census" are the deployment's forges, the forges' document on the client's Deployment page and `temper ctl forge list`: no census of waits exists, and the outbox and read admission are the forge connector's, one for every forge (`../protocol/service-process.md`, 1.3; `../client/forge/deployment.md`, *A full table*). The goal's designer, 2026-11-20.
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
