# Catching up

Provisional, 2026-11-20, from `../../draft/reconciling-start.md`, section 3.
What the forge connector does after a restart once the engine has
opened: it reads afresh and settles its outbox while the engine serves,
and reports when it has caught up. The start that opens before it is
jig's (`domain/engine/restart.md`; `domain/connectors.md`, 14); the
client's part is skein's `forge/client.md`, section 8; the steps the
forge runs on its records are `README.md`, section 9.

## 1. In one page

- **It begins at the opening.** The engine opens once its own records
  are in and its runs adopted; the core then asks each connector to
  catch up, and the forge begins.
- **Two halves, together:** its fresh reads, the restart's reads of
  what changed while the engine was down, and its settle, each restored
  entry looked at once. Neither waits for the other (section 2).
- **It reports each half,** and it has **caught up** once both have
  ended. Nothing bounds it: a forge that stays down keeps catching up
  while the engine serves.
- **A write waits for the restored entries on its lane** and nothing
  else: writes on other lanes go ahead (section 3).
- **A decision on a forge fact** is made on its own read, or waits for
  the forge's fresh reads, never on what the store held alone. Each
  procedure's wait is named in section 4.
- **A person sees it:** the forge's row and the project's landing
  queues say "Catching up" and what it waits for (section 5).

## 2. The two halves

- **Before it begins,** the forge is restored and does nothing outward:
  no read is asked, no entry handed to the client, no timer of its own
  fires (`README.md`, section 9).
- **It begins** when the root gives the forge `Event::CatchUp`, which
  the core asks of each connector at the opening (jig's
  `domain/engine/restart.md`). Both halves start in that event.
- **Its fresh reads** are the restart's read terms, each a read and
  none a write (`../../draft/reconciling-start.status.md`, the check of
  section 2): the client's fresh pass, the landings' startup gathers,
  the pending CI reads, the goal files' reads, the lost writers' reads
  and the goal-branch makings' reads. They end when every term is
  empty; the forge then reports `RestartDone { stage: ReadAfresh }`,
  and from then on its **fresh reads are in**. Outside a restart they
  are in too. In code, `fresh_reads_in(d)`:

  ```rust
  /// The restart's fresh reads are in (temper/docs/design/domain/forge/catching-up.md, 2).
  pub(crate) fn fresh_reads_in(d: &Domain) -> bool {
      d.restore_phase != RestorePhase::Restoring && d.restart_fresh.is_none()
  }
  ```

  in `temper-engine-domain-forge`'s `domain.rs`, beside
  `read_afresh_complete`. As the fresh reads come in, every procedure
  that waits for them (section 4) is woken there, as its own wake
  does: a change by its procedure's wake, a release by
  `ContinueRelease`, a making by `ContinueGoalBranch`.
- **Its settle:** at the beginning, every restored entry not held for a
  person, nor a held landing's, is handed to the client, in number
  order and before any entry kept since the restart; then
  `SettleOutbox`, with no pause first. The client looks for each entry
  with an attempt by its find, and sends each without one, as at any
  time (skein's `forge/client.md`, 8). Once each restored entry has been
  looked at once, the client answers `OutboxDone`, and the forge reports
  `RestartDone { stage: Settled }`. An entry's later outcome, made or
  not found and sent again, comes as any entry's does.
- **Caught up:** both reports made. The core records them by connector
  (jig's `domain/connectors.md`, 14), and the forge clears its
  catching-up state (section 5).
- **No bound.** A forge down keeps its reads waiting and its finds
  retrying, each as its owner's rule says (skein's `forge/client.md`,
  4.2 and 7.5); a read rate-limited at every attempt keeps its pass
  open. None of it holds the engine: only the procedures and writes
  that wait on it, as sections 3 and 4 say.

## 3. Writes while it catches up

- **The line is ready at the beginning.** The outbox's room line
  (`room.md`, 3) admits and orders writes from the beginning of
  catching up, not from the settle's end: a waiter is asked as room
  frees, its place's order kept. Restored entries count against the
  room as any entry does.
- **A write is sent behind its lane.** The forge hands entries to the
  client in number order, the restored ones first; the client sends an
  entry only once no earlier entry on its lane is left (skein's
  `forge/client.md`, 7.2). So a write on a lane that holds a restored
  entry waits for that entry, an uncertain one's find included, and a
  write on any other lane goes ahead. The lane is the fence: no second
  order is kept.
- **A making at restart** (`branches.md`, 1) takes its place in the
  line at its read, and is asked in its place's order from the
  beginning, its write behind its lane like any other.
- **A held entry** stays out of the client until a person resolves it;
  a resolved one is handed in number order, as any.

## 4. What each procedure waits on

A decision that judges a forge fact after a restart is made on one of:

- **its own read,** as in steady state: the procedure reads the
  resource and decides on the answer;
- **the fresh reads,** `fresh_reads_in`: it decides nothing on a
  restored fact before them, and is woken as they come in (section 2);
- **its entries:** a procedure whose restored state names entries of
  its own waits for each one's first look, its outcome or its find's
  answer, as it waits for any entry's outcome.

A procedure may take its own read where the list below says the fresh
reads, when one is at hand. Nothing is judged from the store alone.

| Procedure | The fact it judges | It waits on |
|---|---|---|
| a change's step: `step_change`, `step_read`, `topics::refresh`, `ci_refresh` | its branch, pull, CI, reviews | its own read (kept) |
| `step_verified` | tier verdicts, the change's last head | the fresh reads |
| `change_facts` outside a gather | heads (a moved branch), drift, verdicts, verification | the fresh reads |
| a read session's stall clock (`read_session`, `hold_read_session`) | how long its reads have gone unanswered | the fresh reads: the session is exempt until they are in, and counts from them |
| `goal_branch::base_wait` | the goal branch's head as a change's base | the fresh reads |
| `pull_seen_closed`, `closing_pull` on a change's release | the pull's state | the fresh reads |
| `standing::judge_held_landing` | the last head, verdicts | the fresh reads |
| `tiers::restart`, `tiers::start_due` | the followed branch's landings | the fresh reads, or the branch's own read |
| a tier's step after a restart (`tiers::step`, its `finish`) | its result, its flight's writes | its entries, and the fresh reads |
| `follow_check::advance` with a restored flight | its pending writes | its entries |
| `follows::resume`, `follows::next`, `follows::finish_range` | the followed branch's tip | the branch's own read (`follows::observed`), or the fresh reads |
| `continue_release_inner`, with `release_pull`, `pull_seen_closed`, `settle_prior_effects`, `goal_branch::landed`, `design::unpublished_branch` | heads, pulls, an issue's state, lost writers | the fresh reads |
| `design::design_claim_permitted` and a run's write claim | owned heads, goal-branch and design waits, the writer | the fresh reads |
| `items::workspace_item`, `items::task_workspace_item`, `items::design_workspace_waiting` | a branch's existence, a base head, an owned goal branch | the fresh reads |
| `written::defer`, `record_written_head`, `confirm_written_head` | a writer's push against the head | `written::restart`'s comparison, else the fresh reads |
| `project` (a goal's issue projection) | the issue's state | the fresh reads |
| `rebase::read_afresh` | a branch's drift | its own read of the branch |
| `landing::judge` | the receipts `landing::rebuild` fills | the fresh reads |
| the root's `form_landings` | the trains' landing rows | the fresh reads, where forming reads a forge fact |
| the root's `limit_holds::reconcile`, `room_freed` | the forge's room and usage | the fresh reads |
| `goal_branch::make`, `room::check` | a making's room | the line (section 3) |
| `resolve` (a person's choice on a held entry) | the held entry | handed in number order (section 3) |
| the landing queues, a change's panel, the views (`public`, `standing`, `views`) | restored heads, pulls, verdicts, CI, shown | nothing decided; shown with the catching-up line (section 5) |

Already on their own reads, and kept: `closer::check` and its
selection's read, `topics::classify` (file lists are not restored),
`drafts`, `held` and the brief's reads, `adoption::adopt`,
`making::making_due` and its adoption's read, the landings' gathers,
the goal-branch makings' reads, `removal::restart` and
`removal::restart_releases` (engine records alone), `reds` (requests
alone), `ci_observed` (a fresh status, the restored one only for news),
and the pure crates (`temper-engine-domain-forge-change`, `-tiers`,
`-landing`, `-closer`, `-issues`).

**What goes on at once:** people signing in and out, chats, settings,
task and census reads, worker links and adopted runs, due tasks whose
work touches no forge fact. A run's call that needs the forge waits in
that call, within the run's own deadlines, as when the forge is slow
(`room.md`, 5).

## 5. What a person sees

- **The state:** while the forge catches up, `Domain::catching_up(forge)`
  answers, per forge, what it still waits for; `None` once caught up,
  and outside a restart:

  ```rust
  pub struct CatchingUp {
      pub fresh: u32,         // the client's fresh pass: resources and listings still owed
      pub gathers: u32,       // landings' startup gathers
      pub ci: u32,            // CI reads
      pub goal_files: u32,    // goal files' reads
      pub lost_writers: u32,  // lost writers' reads
      pub goal_branches: u32, // goal-branch makings' reads
      pub uncertain: u32,     // restored entries with an attempt not yet looked at
  }
  ```

  in `temper-engine-domain-forge`, each count that forge's alone. The
  client's two counts are its own (skein's `forge/client.md`, 8). It is
  the one record of what a catching-up forge waits for: the root, the
  pages and the rehearsal (`../../protocol/binaries.md`, 2) read it.
- **The words,** in this order, each count past zero, singular at one:
  "Catching up: 3 fresh reads, 2 landing reads, 1 CI read, 1 goal-file
  read, 1 lost-writer read, 6 goal-branch reads, 2 uncertain writes";
  "Catching up" alone where every count is zero.
- **Where:** the forge's row (`../../client/forge/deployment.md`) and the
  head of a project's landing queues for the forge of its repository
  (`../../client/forge/pages.md`). Neither shows anything once the forge
  has caught up.

## 6. Open questions

- How long a catching-up forge may go before it is named in the logs,
  the census and the pages by its oldest waiters
  (`../../draft/reconciling-start.md`, section 5).
