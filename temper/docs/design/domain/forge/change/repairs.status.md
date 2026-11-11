# repairs: status

2026-11-11. Where the tree's code lags or departs from [repairs.md](repairs.md).

- **Not yet:** "Stalls" and "A stall's hold says where": a `Stalled` hold names neither the step nor what it waits on, and a goal's landing can sit at updating with no hold (`temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`; `temper-engine-domain-forge-change/src/domain.rs`, `updating`) [in flight: change 596].
- **Not yet:** "A repair", a runner's failure: the brief does not say a failure was the runner's, nor how many times its job ran again at the head, and such a failure is repaired, not run again (`temper-engine-domain-forge/src/brief.rs`) [in flight: change 601].

- **Not yet:** "A pull section is pinned at the change's head" and "A brief section's read that fails": a pull past the pin fails the brief, transient, whoever pushed it, the task's own earlier attempt included, until the task is held; a lost reread superseded by a later attempt of the same task records no head; the `brief read refused` line names no reason or heads, `error=None` for a refusal the forge answered (`temper-engine-domain-forge/src/brief.rs`, `brief_answer`) [in flight: change 262].
- **Not yet:** "A release", released from a resolution or a merge job that failed or was cancelled: the change keeps `Resolving { requested: true }` and holds again on the ended delegate's status (`temper-engine-domain-forge-change/src/domain.rs`, `released_state`) [in flight: change 263].

- **Not yet:** "A resolution", its "It waits for its producer": a conflict met while a task that pushes the change's branch still lives is resolved at once, its pull pinned at the last head the change knew, so a push by that task fails every attempt of the resolution's brief, `the change's head moved` (`temper-engine-domain-forge-change/src/domain.rs`, `conflict_due`; `temper-engine-domain/src/assemble.rs`, `finish_brief_plan`) [in flight: change 37].

- **Not yet:**
  - the whole document: the user's N4 request of 2026-11-08 asks for focused and
    fuzzy tests. The focused story
    `a_persons_request_for_changes_repairs_with_its_remarks`
    (`tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`)
    has all four requested rows and checks the repaired head's new review
    wait, but no fuzzy story exercising this person's request-to-repair
    path is identified in main or the supplied reports. The existing
    fuzzy stories do not establish that coverage.
  -  no supplied result shows the three-thread real-process trial
    passing after update recovery. Both styles' lost answers, retries,
    exhausted counts, release and late-copy holds have world stories;
    the real-Forgejo restart trial remains the after-landing proof.
    The fake reads a rebase copy's head at landing, so the dropped-push
    story uses an explicit branch rewrite and does not establish the
    real forge's concurrent-push behaviour (skein's `forge/forgejo.md`, section 2).
  - `README.md`, 2, step 3: a head already behind taking its base first stands on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/update_first.rs`). The request
    `landing-fixes` of 2026-11-06 has its proof in the next landing plan on a real
    deployment, and no record shows one, so how much of a change's wait it saves,
    and how soon a moving base reaches the bound (`../README.md`, section 11), are unmeasured.
  - the whole document: the `landing-fixes-2` request of 2026-11-06 takes the next
    landing plan on a real deployment as its proof, and no result is supplied.
    The fake-forge stories cover a clean update's named failed check, its log
    before the base's contents, an unnamed check and a resolution's conflict
    words (`tests/engine/domain/tests/suite/forge_routes/repair_cause.rs`), and
    a retried repair's saved branch deleted as it closes (`saved_close.rs`).
    They do not establish that deployment's branch cleanup or that the new
    brief would avoid the reported failed `read_job`.
  - `questions.md`, 1, *Which target*: the coding charter's sentence and the refusal's words
    stand on a script alone
    (`tests/engine/domain/tests/suite/forge_routes/question_target.rs`). No real
    model has read them, and the request `landing-fixes` of 2026-11-06 has the same
    proof as `README.md`, 2's, so whether a producer's first question now reaches its change
    is unobserved.
  - `questions.md`, 1, *The next producer merges the landing tip first*: a tip its start
    already contains, a clean merge committed before its run and the advice
    for a conflict have scripted coverage alone. No deployment replay on
    record proves the `resume-told` request's resumed producer and its answerers
    read the tree rightly (`../../worker.status.md`, sections 5 and 10).
  -  a refusal's "adopt the repository again" has no story from the browser. The engine takes an adopted repository's adoption (`readopt`, `temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`), and by reading alone the form sends one (`../../../client/forge/family.md`, 4); it opens at its defaults, so the person types the repository, its role, prefix and landing branch again.
  -  a removal without the box, its group's act answered in its
    decision: an owned or fake forge's removal asks the box link for
    `remove N` whether or not a pipe is there, and without one waits for
    an answer that never comes, so it is never whole and the
    playground's reset stops at its forge
    (`temper-engine-domain-forge/src/removal.rs`, `begin`, sets
    `box_removed` by the forge's mode alone;
    `temper-service/src/route/box_link.rs`, `remove`).
  -  on GitHub, statuses and dispatches by temper's App for a
    project that lands by turn, and verdicts posted where an installation
    grants `statuses: write`; `status` is false there.
  -  the forge's own merge queue beyond a queued merge kept
    pending: `Pull.queued` and `dequeued` are not read, a queued change
    keeps temper's turn, an ejection makes no repair, the panel shows no
    place in it, and adoption's words leave out the queue, signatures,
    linear history, code owners and the App's bypass
    (`temper-service/src/family.rs`).
  -  the findings topic and findings (`../../findings.md`).
  -  a hold parked until a landing. An explicit landing subscription
    with no paths receives a known landing as kept news; it does not wake
    a parked hold.
  -  per-change bounds and gates are not caller overrides: `change` takes parameters 1
    to 6, 9 to 12, 14 and 16, its bounds, the questions it takes among
    them, are the deployment's `change_policy`, and its gates its landing
    branch's rules' (`temper-engine-domain/src/route/forge_route.rs`,
    `start_change`); its page's line that a change takes no words.
  -  a goal's issue can still reach its forge with an empty title. A spec
    whose first line is blank or a lone full stop leaves no sentence
    (`temper-engine-domain/src/route/forge_route.rs`, the projection's `title`;
    `temper-engine-domain-forge-issues/src/text.rs`, `title`), and
    `temper-engine-domain-forge/src/domain.rs`, `project_write`, writes it as it
    is. The request `producer-failure` of 2026-11-05 has no empty title reach a
    forge by any route; its plan's cut took the pull request's alone, an issue
    making its own from a goal's spec. This is from reading: no story writes one.
  -  the pull request's body naming its goal's issue.
  -  a person's approval as a gate; a review's lens; gates run at once
    when the change asks; a verdict posted as a commit status.
  - the whole document: a no-change on a change under another hold stays on the row, but the
    hold that stands does not say its words: `hold_change_decision`
    (`temper-engine-domain/src/route/forge_route.rs`) reads them for
    `Hold::Failed` alone, and they are in no panel. A release reads the result
    as a failed producer's, so its words first reach a person in the `Failed`
    hold that follows, and the change does not end. Seen by reading: the step
    test stops at the row
    (`a_questions_hold_keeps_the_result_until_release_and_says_no_change_words`).
  - the whole document: a result ending a stalled hold has no world: a live delegate does not
    stall, so `temper-engine-domain-forge/src/tests/change.rs` gives the held
    row by `Event::Change`, and only a green head's worlds run the release its
    `EndHold` asks for
    (`temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs`).
  - the whole document: the request `parallel-faults` of 2026-11-06 has its proof in a rerun of
    its parallel replay, and no record shows one, so no real run has shown a
    producer working past the stall or a stall's hold reaching anyone. It also
    asks why the replay's hold came about 57 minutes in, where the stall is 30,
    and that stays unknown: in a world the clock starts at the producer's
    request, after any wait for a dependency, and a hold shows only when a step
    runs, but no deployment's times were read. So is whether that row held with
    its task not, by the path `change_decision` now closes: a world shows that
    drop only by delivering the decision out of step, though the empty inbox,
    and `task show`'s "Waiting for news", which jig answers of a task not held
    (`jig/crates/jig-domain-core/src/runs/waits.rs`), fit it.
  - the whole document: widening exhausted spend with ctl before release: `task amend`'s
    `--budget` is not implemented (`../../../client/ctl/areas.md`). A spend
    hold offers cancellation or asking afresh instead.
  - the whole document: `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`), and the plan
    `first-goal-fixes` edits neither, the ui drafts holding them, so a cancel
    is said by `task show` and not by this line.
  - the whole document: the walk the request `brief-failure` of 2026-11-05 ends with has not
    passed: on a real Forgejo, a change whose CI goes red gets a repair that
    starts, and a repair whose brief can't be read ends held with the reason in
    `temper ctl`. Its QA rung is not drafted yet, so both stand on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`).
  - the whole document: `[the reviews could not be read: REASON]`, a gate's repair standing on
    its pull, is unobserved: it is built (`temper-engine-domain-forge/src/brief.rs`,
    `brief_answer`), but those stories run a CI repair's pair alone and no test
    holds its words.
  - the whole document: a released resolution or merge job that failed holds again, where the
    section has a resolution asked afresh: `released_state`
    (`temper-engine-domain-forge-change/src/domain.rs`) keeps `Resolving {
    requested: true }`, the row keeps the failed status
    (`temper-engine-domain-forge/src/domain.rs`, `change_standing`), and
    `delegated` reads it, so no fresh task is made. A failed producer and a failed
    repair take `requested: false`. This is from reading: no test or story
    releases a failed resolution. The `producer-failure` plan, whose request asks
    after a failure's shape and not its release, pins only its hold
    (`failed_repairs_and_resolutions_do_not_take_a_pushed_head`).
  - the whole document: a green head ending a repair has no story for a repair waiting for an
    answer to its question (none asks from a repair), a repair finding nothing
    to do (`NoChange`, read as failed), or a task closing or held with its
    attempt running, taken as a run in progress
    (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate_running`):
    the policy's tests hold a failed status and `delegate_running` alone
    (`temper-engine-domain-forge-change/src/tests/green.rs`).
  - the whole document: a goal's landing whose update into main conflicts,
    settled by the merge job and a resolution on its goal's branch
    (`../../../draft/reference-landing.md`, section 9's first story), is built
    by every change's path, the job writing the branch under the goal's
    hold as the landing's resolution does
    (`temper-engine-domain-forge/src/items.rs`, `change_branch_holder`),
    but no story runs it: the merge job's stories are changes into a
    goal's branch and into main, none a goal's landing
    (`tests/engine/domain/tests/suite/forge_routes/merge_job.rs`; the
    composed system world's `suite::merge_job`). The
    forge world steps such a landing to its merge job and a resolution
    on scripted delegations alone, with no job or resolution run
    (`a_first_change_behind_its_base_takes_it_or_goes_to_its_merge_job`).
  -  a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
  -  the explicit choice to adopt a GitHub repository without CI.
  -  the rule and criterion of a change's last wait or refusal, and
    its charter, on its panel.
  -  a change drained.
  -  a dependent's landing holding its dependency's pushed head in
    g's ancestry is unobserved in stories, the fake forge squashing
    merges (skein's `testing/fakes/forge.md`, 2.2).
  -  what a step short of message room leaves for the next. A
    question's `Stands` is taken, foreman, asked or held
    (`temper-engine-domain-forge-change/src/boundary.rs`, and the store's
    `Stands`), so nothing holds reasons awaiting passing up, whose stored
    ceiling is not in the schema, or a foreman's words awaiting passing
    down. With no room `relay`
    (`temper-engine-domain-forge-change/src/relay.rs`) leaves the question
    as it stood and wakes the change, while the step's commit takes the
    foreman's verdict from the inbox (jig's `domain/tasks/messages.md`, 2), so the
    question waits on an ended foreman. A question released past the bound
    at such a step meets the spent allowance again at the next: the release
    is that step's alone, and `next_act` holds the change.
  -  a repair's or a resolution's question, relayed as a producer's
    is (`temper-engine-domain/src/route/forge_route/relay.rs`,
    `asking_delegate`), and its words, refused as a producer's are, are
    unobserved in stories: every relay story runs the change's producing
    task only
    (`tests/engine/domain/tests/suite/forge_routes/questions.rs`,
    `foremen.rs` and `goal_questions.rs`).
  -  a branch a tier follows still pauses its queue on its own CI's
    red and makes a repair, where its tiers judge it instead
    (`temper-engine-domain-forge-change/src/domain.rs`, `base_ci`).
  -  the issue's milestones but its lifecycle and revisions: the plan
    accepted, a change landed or held, a report kept.
  -  questions on the forge.
  -  participating objects beyond their news; a landing waiting for
    someone else's merge where temper may not merge (also `../adoption.md`, section 4 and `../queues.md`).
  -  upstream contributions.
  -  the rules as settings (`gates.md`, 3) in the forge's world: `tests/engine/forge`
    tells none of them. The engine's domain world tells a rule changed
    between two merges, a rule refused in its change's decision, a
    change kept on the charters it was made with and one held when its
    charter is no longer offered, and a restart
    (`tests/engine/domain/tests/suite/settings/kinds.rs`,
    `forge_routes/kind_charters.rs`); no world tells a gate a rule adds
    asked for on a queued change, a gate a rule drops no longer waited
    on.
- **Design departs:** `repairs.md`, `questions.md`, 1 and jig's `domain/tasks/lifecycle.md`, 8, from the request
  `producer-failure` of 2026-11-05, each decided by its plan's cut, for the user
  to see:
  - `questions.md`, 1, and jig's `../attention.md`, 3: the request recommends refusing an asker's `finish` at
    the call, with the words `Your question is still open: call wait; the answer
    reaches this run.` The run parks instead, as a done finish does: smith judges
    a `finish` with no call to its host, and the tasks child decides at the
    answer's commit, the run and its host knowing nothing of it (jig's
    `domain/engine/executors.md`, 4). A refusal at the call would need a new crossing from
    jig through the worker to smith, and costs the run a turn that parking does
    not. So those words reach no run; the stock coding charter says the same
    before it finishes, and a resumed run's transcript shows its earlier `finish`
    as accepted, then its waking instruction and the answer.
  -  the request has a question left by its asker's end say so to whoever
    was waiting. One passed up says so at its answer, not at the end, because
    saying it once at the end would need a stored mark and the request names no
    store change. So a requester that never answers hears nothing of it.
  - the whole document: the request has its failing test in the forge world. Its stories run in
    the engine's domain world
    (`tests/engine/domain/tests/suite/forge_routes/producer_failure.rs`), where the
    core and the connector meet, with the policy's in
    `temper-engine-domain-forge-change/src/tests.rs`; `tests/engine/forge` tells
    none of it.
