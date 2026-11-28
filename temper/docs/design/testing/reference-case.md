# The reference case

## 1. The case library

The case library: a fixed case in three acts, its stories numbered by
its steps (`4.1` to `6.8`). Surviving stories run in the real loop
(`real-loop.md`), on the case's cast, scripts and oracle,
written once for the real loop's cluster (../protocol/cluster.md, 6).

- **The cast is a fixture.** The person is a client's directory, signed
  in; the director is the person's chat in project 1, on a
  coordinator's charter; the owners (A, B, C) and their delegates are
  the agent and procedure tasks the scripts delegate; kit, and app when
  a step needs it, are repositories on the fake forge, kit's tree
  holding a draft and the process's documents; the workers are the
  standalone service's one, and ws, s1 and s2 once the case runs on the
  cluster. The tasks' numbers are a fresh deployment's, named once and
  checked against the documents the story reads.
- **Every judgement is a script.** Each choice the case gives an agent
  (the director's, an owner's, a delegate's) is a fake LLM script played
  word for word, cued by the task's words (`scenarios.md`, 1.1). Each act of the person
  is a `temper ctl` call. Nothing on the fakes judges, so the case's
  judgements are its scripts, and its oracle checks what temper did with
  them.
- **The oracle checks effects,** one checkpoint for each step, over
  what the step's Musts and Adds promise: authority and spend, a message
  or an answer delivered, work held and resumed, what landed. It reads
  only what every world that runs the case observes: the person's
  documents (`temper ctl`'s JSON, by the act that read them), the forge
  (its main and its checks at each merged head), and what each agent was
  told (the fake LLM's queries, by their cue). It never judges an
  agent's words, only their effects, so the same oracle checks a live
  model's run of the case. It reports every miss at once, each naming
  its step and its Must or Add.
- **Time keeps the case's order, not its ratio.** A story waits on
  conditions, `temper ctl`'s waits, never on time. A timer, deadline,
  cap, reset or grace that a step runs on is armed where the case arms
  it, at the shortest length the loop keeps in order, and the story
  waits for its effect. The case's hours are not mapped to a ratio.
- **Smoke, as the real loop is** (`real-loop.md`): a step's story runs its path
  end to end, once; the cases each Must rests on are tested in the
  lower worlds that own them. Each story is held to `checks.md`, 1's guide and
  follows skein/docs/foundation/testing-strategy.md, 8.1 when it misses it. What a story checks
  decides its suite; cost never does.
- **A process story belongs only where processes are the point**
  (the user's rule, 10-23): a step gets a story on the process tier's
  cluster (`process-tier.md`) only where its Musts depend on processes, that is
  workers, placement, a run moving between workers, a restart, a kill or
  a stop, since a behaviour is tested in the lowest world that owns it
  and the tiers above hold smokes. No case story is in
  `bin/temper-box/tests/`: the cluster's own stories keep the kill, the
  stop and the exit of its processes (../protocol/cluster.md, 6).
- **What a gap holds is named.** A step whose Must needs what temper
  lacks (the gaps below) runs the part that works; its story's name says
  what it shows, and the table below what it leaves out and what holds
  it. Behaviour that conflicts with the case is not changed here, and
  not counted as the case's.
- **A library of the real-loop world:** `tests/real-loop/src/case.rs`,
  the cast and its fixture, and `case/`: `scripts.rs`, the judgements
  for each step; `acts.rs`, the person's acts for each step; and
  `oracle.rs`, the checkpoints over one type of observations, which
  each world fills from what it observes. Named scenarios belong among
  the focused stories in `tests/real-loop/tests/suite/`; fuzzy sweeps
  belong in its `fuzzy/` modules (`checks.md`, 1).

## 2. Opening the work

Act 1:

| Step | Its story shows | It leaves out, and what holds it |
|---|---|---|
| `fakes.md`, 1 | `the_opener_and_the_two_hour_check_run_on_three_workers` (`suite::fuzzy::case_4_1_4_3`), its opener: ws alone starts; s1 joins after the director is placed on ws. A delegates a Markdown-only design on kit, merged through the change procedure with green CI and its design words on main; the draft stays unchanged. The harness checks that ws ran the director, and s1 ran A, its coder and reviewer. Placement uses cached workspaces, free slots and channel order, not names (../protocol/cluster.md, 7). The diagnostics story uses the opener too. | Engine-domain focused holders, under `temper-engine-domain-world::suite`: A's priority, model, effort and brief naming the draft's commit, `root_routes::a_delegate_batch_and_its_named_answer_commit_together`; admitted subtree authority, `root_routes::opener::a_subtree_within_its_owners_authority_is_admitted` (refusal remains `root_routes::a_batch_beyond_authority_is_refused_whole`); spend through the subtree to the goal, `root_routes::opener::a_subtrees_spend_rolls_up_to_its_goal`; question in the person's inbox, answer reaching A and both reports, `root_routes::a_person_answers_a_question_its_task_has_already_read`. The story has no question-and-answer leg. Still out: A's survey, an extra delegate beyond the guide; instructions and pool, which `delegate` doesn't decode (../domain/agent/tools.md); host kind, absent from delegation and amendment; which workers have room; s2 in the opener, whose work is serial behind A's workspace. The director waits in its run on the fixture's chat charter (scenarios.md, 1.1), its threshold past patience, never waited out; parking is covered by `fakes.md`, 4 and 5. Markdown's CI fast path is the repository's, not temper's: fake CI passes at once. Kit has no home or goal issue: an uncertain or refused issue write holds a goal from closing (../domain/forge/issues.md). The director is not a standing chat; it reports after A. No fifteen-line design report is judged. |
| `fakes.md`, 2 | none | CI at the tip and the queue's priorities are the forge's worlds' (`../domain/forge/change/README.md` and `../domain/forge/queues.md`); a failure at the tip that matches an open finding leaving the queue running is undesigned, and today's pause and repair at a failed tip is the conflict the case names, so the step does not run as the case's |
| `fakes.md`, 3 | `the_opener_and_the_two_hour_check_run_on_three_workers` (`suite::fuzzy::case_4_1_4_3`), after the opener: the second director schedules B's benchmark. Both wait in their runs, holding ws and s1; the benchmark is due without a free slot. The person adds s2 late; its benchmark waits there. Once all workers hold runs, the director arms a timer due at `at` 0, the shortest check preserving the case's order. Its real-clock wake is read after the subscription result; the director cooperatively cancels the benchmark, whose run and task end cancelled. The harness checks ws ran the directors alone, s1 the opener and B, s2 the benchmark alone; ws and s1 were assigned before s2 started; every worker drained and no agent was killed. | Engine-domain focused holders under `temper-engine-domain-world::suite`: the subscription armed, `root_routes::timer_cancellation::an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation`; the introduced ancestor's cancel and reason, B hearing it before reporting, both reports and nothing left running, `root_routes::opener::an_introduced_ancestor_cancels_a_run_before_its_owner_reports`. The real-loop checkpoint does not repeat these policy checks. A run ignoring cancel belongs to the hung worker's 4.6. Pools resized or slots lent to A have no control in `ctl` or agent tools; each worker has one slot (../protocol/cluster.md, 7). Host kind is not in amendments. Load, queue and disk reports await worker health design (`checks.md`, 4); the timer wake stands in. B stays on s1, not moved to s2 by host kind; its benchmark stalls until s2 joins. Non-git/non-note data, quiet benchmark machines, disk floors, build lanes, caches and memory caps are outside temper. No two hours elapse: the arm turn is held until placement. B and the benchmark use the chat charter's in-run wait (scenarios.md, 1.1); parking would free a slot. |
| `fakes.md`, 4 | none | The question row, its single inbox entry, the director told before the answer, the numbered acknowledgement and A hearing the answer directly: `temper-engine-domain-world::suite::root_routes::a_person_answers_a_question_its_task_has_already_read`. The director writes and recalls the answer: `temper-engine-domain-world::suite::root_routes::a_note_written_corrected_by_a_person_and_recalled_is_durable`. Its agent amendment selects the default model without effort at revision 1: `temper-engine-domain-world::suite::root_routes::pace::an_agent_amends_its_delegate_to_the_default_model_without_effort`. The first owner-model query and the default-model query after that amendment: `temper-system-world::suite::next_turn::an_agents_amendment_changes_its_delegates_next_provider_query`. Holder order, A keeping its account and the survey selecting the newly first account: `temper-engine-domain-world::suite::root_routes::pace::an_account_rank_keeps_the_owner_and_changes_its_next_delegate`. Every survey request's selected grant, with two usable sources: `temper-system-world::suite::accounts::every_survey_completion_uses_the_first_ranked_accounts_grant`; the old peer's credential index 1 names that second source. The director as asker stays out: only a delegate can ask its person (jig's `domain/tasks/messages.md`, 1). Plan windows, resets and sustainable pace as account facts, B's model and s2's sources stay out; holder order stands for s2. Recall stands for not asking again. |
| `fakes.md`, 5 | none | Provider limit/reset through Smith, the unchanged task, revision and account, due wait, matching source/account reset, automatic resumption and retained spec on credential 0: `temper-system-world::suite::accounts::a_provider_limit_resumes_the_same_work_after_its_reset_without_an_act`. Provider refusal, refused wait and sign-in-needed account, replacement through engine input, retained spec on credential 1, unchanged task/revision/account and director report: `temper-system-world::suite::accounts::a_provider_refusal_resumes_the_same_work_on_the_replaced_key`. Both forbid another agent. Account policy: `jig-accounts-world::suite::simulation::a_limit_with_a_reset_ends_at_it_and_asks_nothing`, `jig-accounts-world::suite::simulation::a_refused_account_holds_and_asks_nothing`; persistence and admission: `jig-core-world::suite::restart::a_limited_source_keeps_its_reset_across_a_restart`, `jig-core-world::suite::credentials::a_terminal_source_limit_waits_without_a_try_until_resume`, `jig-core-world::suite::runs::a_run_waits_while_its_account_must_sign_in_again`. Sign-in on s2 and copying no credentials between machines stay outside: a key replacement stands in for sign-in; B adds no assertion. Five hours become a controlled reset; the failed attempt ends and the same task resumes. |
| 4.6 | `a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` (`suite::fuzzy::case_4_6`) keeps the real worker-lifecycle seam on ws, s1 and s2. The director starts on ws and delegates A's next phase. A asks its question; a completed recall turn commits the director's read, and the script waits. The person stops ws with `stop_worker`; the chat retries on s1 or s2 and the person answers A directly once. After the chat ends, `start_worker` starts ws again on its own directories. The oracle checks one answer reaching A with no director relay; the harness checks the first attempt on ws, a retry elsewhere without counting zero-spend rechecks, ws's two lives each linked once and drained, its second life running nothing, every worker drained and no agent killed. The committed-read activity poll and the replied watch are setup barriers, not journal or question-retention checks; there are no post-run activity pages or report reads. Nothing orders these acts against a product deadline. The fixture keeps the product's worker grace and its zero restart window; ws stays absent until the chat ends, so it cannot receive the retry. | Focused holders in `temper-engine-domain-world::suite`: `root_routes::stopped_worker::a_stopped_chat_retries_elsewhere_with_a_fenced_journal` holds the complete, ordered journal with no open attempt, stopped suspension without a spent try, no replacement before the restart window, retry on the other worker, committed transcript and fencing of late turns. `root_routes::stopped_worker::a_stopped_director_keeps_its_read_unanswered_question_once` holds the committed read, waiting observation and one unanswered question across that retry. `root_routes::a_person_answers_a_question_its_task_has_already_read` holds the direct answer, repeated-command deduplication and both reports; `root_routes::a_restart_restores_an_unanswered_question_beside_results` holds engine restart restoration. The hang past grace and its four hours are not this stop: loss is held by `root_routes::a_lost_workers_run_is_placed_on_the_other_worker_past_its_grace`, `root_routes::a_worker_lost_mid_run_resumes_at_the_last_committed_turn`, `root_routes::a_worker_frozen_past_its_grace_gets_no_next_attempt_until_its_sum_has_passed` and `forge_routes::a_worker_frozen_past_its_grace_resumes_with_a_push_in_hand_and_lands_nothing_twice`, plus the process tier's killed and stopped workers (../protocol/cluster.md, 6). A fenced original still running beside its replacement belongs to jig's fleet world and `root_routes::a_fenced_attempts_late_traffic_changes_no_turn_and_its_worker_takes_new_work`. Taking new work after restart belongs to `root_routes::a_worker_back_at_its_position_after_its_run_ended_takes_new_work_there` (../protocol/cluster.md, 4); here ws returns after the work ends. The stalled-agent bound belongs to smith's no-progress watchdog, translated into agent failure and jig's bounded retry. Three questions remain out: their ring provision exceeds one ring across three workers. No single point remains undesigned: one engine's ending ends the cluster (../protocol/cluster.md, 4). Orphaned completion checkers and Codex stdin hangs belong to smith/jig, and the build queue's environment is outside temper. Forge credentials absent from a worker cannot be asserted by the real loop, whose fake forge stages its token there; the documents given to workers holding no forge admin token are their writer's tests (`../../development/slow-tests-moves-2026-11-24-temper-cluster.md`, 1.6), not per-call git credentials. |
