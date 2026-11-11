# README: status

2026-11-11. Where the tree's code lags or departs from [README.md](README.md).

- **Not yet:** 1: parameter 18, the source landings a follow change
  follows, its route `up` and no early start [in flight: change 99].

- **Not yet:** 3, *Its own writes are its own* and *Its branch moved*'s words: an update's made head is not recorded as its branch's head, so a goal's landing on its held branch takes it for drift and holds `BranchMoved`; a let-go update is judged against the base's tip read now, not the base it asked for; a close-out's or catch-up's merge into the goal's branch takes no writer slot the landing sees; and a `BranchMoved` hold names neither head (`temper-engine-domain-forge/src/domain.rs`, `change_outcome`, `change_facts`; `temper-engine-domain/src/route/forge_route.rs`, `hold_change_decision`) [in flight: change 596].

- **Not yet:** 1, `reruns`, and 2, step 3, *The runner's failures run again*, with step 7's attempt in the landed result: a change's CI failed by the runner is repaired like any other, and no configuration names `forge.change_policy.reruns` (`temper-engine-domain-forge-change/src/domain.rs`, `checking`, `gating`) [in flight: change 601].

- **Not yet:** 7, and 5's panel kept until its room is needed: no change's record ever leaves the table, so once `forge.changes` changes have been made every new one is refused, before it registers, with words naming no bound; nothing drops a settled change, nor what the forge keeps by its task [in flight: change 391].

- **Not yet:** 7, "Its landed commits are bounded alike": the forge's `landed` table leaves only with a dropped change, and a merge finding it full skips its `Landed` row, its landing branch's head, `Pushed` and its tiers' observation, so a goal is held on its own merge [in flight: change 476].

- **Not yet:** 7, "Dropped last": an `Uncleared`, `Landed` or `Red` change whose task has ended is never dropped, so a tier runner down for long fills the table [in flight: change 471].

- **Not yet:** 2, a change whose pushed head its landing branch already contains: it lives on, and only a contained catch-up ends with no change; nothing reads whether the base's tip contains a change's head [in flight: change 264].

- **Not yet:** 2, step 6: a resolution and a merge job wait for no producer or repair to end, and are cut at the last head the change knew, not its latest push (`repairs.md`) [in flight: change 37].

- **Not yet:** 1, 3: a goal's runs push to its run
  branch r, and a `"branch": "run"` change lands r into g; g records no
  design writers, so a push to g by the goal's run would hold the goal as
  drift and a landing change on g as "branch moved"; a goal's run prepared
  before g is made and confirmed starts on the base, its run unable to
  deliver (`temper-engine-domain-forge/src/items.rs`,
  `task_workspace_item`) [in flight: change 98].

`repairs.md` has no known implementation gap in a resolution's files:
its pull section carries a file whole, both sides, where it fits the
room the section has left, and names one that doesn't, with its sides'
bytes or `absent before` and `absent after`, for its checkout, the files
after it carried as they fit (`temper-engine-domain-forge/src/brief.rs`,
`name_conflict_if_past_room`; `src/tests/brief.rs`,
`a_resolutions_files_are_carried_whole_or_named`). The charter's fit
that can still cut the section's end is `../../agent/README.status.md`'s `../adoption.md`, 3; no
story takes a named file through its charter's start.

`../adoption.md`, section 5's `DismissMaking` is present: an owner can erase a failed row
and free its place, absence is kept, and running rows or another scope
are refused. The request carries `into`, so its keyed scope survives the
row's erasure. The connector's making tables cover these decisions.

Section 1 has no known implementation gap in bounding a pull request's
body at 65,536 bytes independently of `forge.client.op_bytes`, with a
marked cut and the change's stored words kept whole. The engine's batch
table admits 32 changes and refuses 33 at an authority batch bound of 32.

`../README.md`, section 5.2 has no known gap in a goal's topics from the core: they are
its landing branches' alone, and a goal's CI subscription held at
restart is dropped with its jig row (`temper-engine-domain-forge/src/domain.rs`,
`restored`, by `Request::GoalUntopic`), while a task's own stays.

`repairs.md` has no known gap in a merge job's failures: each is a line on
the worker that met it and on the engine, naming the job, its kind and why,
and the way out of its hold, out of tries, points at the engine's log,
which names each (`../../worker.status.md`, 4.4). The composed system world runs
a change's merge job from the engine through its worker to the fake forge
(`../../../testing/README.status.md`, 2.1).

- **Not yet:**
  -  the user's N4 request of 2026-11-08 asks for focused and
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
  - 2, step 3: a head already behind taking its base first stands on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/update_first.rs`). The request
    `landing-fixes` of 2026-11-06 has its proof in the next landing plan on a real
    deployment, and no record shows one, so how much of a change's wait it saves,
    and how soon a moving base reaches the bound (`../README.md`, section 11), are unmeasured.
  -  the `landing-fixes-2` request of 2026-11-06 takes the next
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
    proof as 2's, so whether a producer's first question now reaches its change
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
  - 1: per-change bounds and gates are not caller overrides: `change` takes parameters 1
    to 6, 9 to 12, 14 and 16, its bounds, the questions it takes among
    them, are the deployment's `change_policy`, and its gates its landing
    branch's rules' (`temper-engine-domain/src/route/forge_route.rs`,
    `start_change`); its page's line that a change takes no words.
  - 1: a goal's issue can still reach its forge with an empty title. A spec
    whose first line is blank or a lone full stop leaves no sentence
    (`temper-engine-domain/src/route/forge_route.rs`, the projection's `title`;
    `temper-engine-domain-forge-issues/src/text.rs`, `title`), and
    `temper-engine-domain-forge/src/domain.rs`, `project_write`, writes it as it
    is. The request `producer-failure` of 2026-11-05 has no empty title reach a
    forge by any route; its plan's cut took the pull request's alone, an issue
    making its own from a goal's spec. This is from reading: no story writes one.
  - 2: the pull request's body naming its goal's issue.
  -  a person's approval as a gate; a review's lens; gates run at once
    when the change asks; a verdict posted as a commit status.
  -  a no-change on a change under another hold stays on the row, but the
    hold that stands does not say its words: `hold_change_decision`
    (`temper-engine-domain/src/route/forge_route.rs`) reads them for
    `Hold::Failed` alone, and they are in no panel. A release reads the result
    as a failed producer's, so its words first reach a person in the `Failed`
    hold that follows, and the change does not end. Seen by reading: the step
    test stops at the row
    (`a_questions_hold_keeps_the_result_until_release_and_says_no_change_words`).
  -  a result ending a stalled hold has no world: a live delegate does not
    stall, so `temper-engine-domain-forge/src/tests/change.rs` gives the held
    row by `Event::Change`, and only a green head's worlds run the release its
    `EndHold` asks for
    (`temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs`).
  -  the request `parallel-faults` of 2026-11-06 has its proof in a rerun of
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
  -  widening exhausted spend with ctl before release: `task amend`'s
    `--budget` is not implemented (`../../../client/ctl/areas.md`). A spend
    hold offers cancellation or asking afresh instead.
  -  `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`), and the plan
    `first-goal-fixes` edits neither, the ui drafts holding them, so a cancel
    is said by `task show` and not by this line.
  -  the walk the request `brief-failure` of 2026-11-05 ends with has not
    passed: on a real Forgejo, a change whose CI goes red gets a repair that
    starts, and a repair whose brief can't be read ends held with the reason in
    `temper ctl`. Its QA rung is not drafted yet, so both stand on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`).
  -  `[the reviews could not be read: REASON]`, a gate's repair standing on
    its pull, is unobserved: it is built (`temper-engine-domain-forge/src/brief.rs`,
    `brief_answer`), but those stories run a CI repair's pair alone and no test
    holds its words.
  -  a released resolution or merge job that failed holds again, where the
    section has a resolution asked afresh: `released_state`
    (`temper-engine-domain-forge-change/src/domain.rs`) keeps `Resolving {
    requested: true }`, the row keeps the failed status
    (`temper-engine-domain-forge/src/domain.rs`, `change_standing`), and
    `delegated` reads it, so no fresh task is made. A failed producer and a failed
    repair take `requested: false`. This is from reading: no test or story
    releases a failed resolution. The `producer-failure` plan, whose request asks
    after a failure's shape and not its release, pins only its hold
    (`failed_repairs_and_resolutions_do_not_take_a_pushed_head`).
  -  a green head ending a repair has no story for a repair waiting for an
    answer to its question (none asks from a repair), a repair finding nothing
    to do (`NoChange`, read as failed), or a task closing or held with its
    attempt running, taken as a run in progress
    (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate_running`):
    the policy's tests hold a failed status and `delegate_running` alone
    (`temper-engine-domain-forge-change/src/tests/green.rs`).
  -  a goal's landing whose update into main conflicts,
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
  - 3: a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
  -  the explicit choice to adopt a GitHub repository without CI.
  - 5: the rule and criterion of a change's last wait or refusal, and
    its charter, on its panel.
  - 6: a change drained.
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
- **Not yet:** 3: containment is read as a commit list or a
  comparison, not a `Range`: the change step's up-to-date and catch-up
  ranges (`Read::Commits`, `temper-engine-domain-forge/src/domain.rs`,
  `ancestry_status`), a landing member's main (`landing.rs`) and a lost
  run's observed head (`domain.rs`, `lost_read`) (skein's `forge/README.md`,
  3.2).
