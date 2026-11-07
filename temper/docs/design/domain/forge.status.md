# forge: status

2026-11-07. Where temper's code lags or departs from [forge.md](forge.md).

Sections 4, 8.7 and 8.9 have no known implementation gap in
forge-scoped party keys and the shared lookup. Adoption seeds the forge
and user pair; reviews use the pull request's forge, and eligibility uses
that same lookup. The step tables cover equal
user numbers on different forges, unknown accounts and each forge's writer.

- **Not yet:** 4: `forge_person` answers adoption's seeds alone, never a
  forge's link.
- **Not yet:** 4, 4.6: adoption seeds the link's user, and nothing keeps,
  replaces or clears a link from the root's sole owner, a `Reconcile`'s person
  or an App's account.
- **Not yet:** 8.9: eligibility counts an account on any forge of the project,
  not on the forge of each repository the rule covers; the refusal names no
  forge, and the playground's fake forge has no words of its own.
- **Not yet:** 8.7: a change's review facts are removed only when it ends, not
  when its merge is admitted, and keep no time they were first kept at their
  head (`temper-engine-domain-forge/src/domain.rs`, `keep_review_facts`).
- **Not yet:** 8.12: no subject `approvals`, no read of the changes waiting on a
  person's approval, no mark of it, and the read of a proposal's items says
  nothing of an approval on an owned Forgejo.
Section 12.1's questions on the forge remain unbuilt, as listed below;
their designed account mapping also requires the repository's forge.

Sections 8.1, 8.5, 11 and 11.1 have no known implementation gap in a
goal's delivery through r. The six engine domain stories in
`tests/engine/domain/tests/suite/forge_routes/goal_delivery.rs` cover
merges into g while the goal runs, an early dependent, a second run,
work r holds that g lacks, a new delivery under an open change, the merge
job and resolution after the goal frees r's slot, and the goal's close.
A change's close leaves r held by its goal; the goal's close deletes it.
Coverage uses the fake forge. Section 20 keeps the default for deleting
unlanded run work and the deferred saved-start case overtaken by a repair.

Section 4.5's `DismissMaking` is present: an owner can erase a failed row
and free its place, absence is kept, and running rows or another scope
are refused. The request carries `into`, so its keyed scope survives the
row's erasure. The connector's making tables cover these decisions.

Section 8.1 has no known implementation gap in bounding a pull request's
body at 65,536 bytes independently of `forge.client.op_bytes`, with a
marked cut and the change's stored words kept whole. The engine's batch
table admits 32 changes and refuses 33 at an authority batch bound of 32.

- **Not yet:**
  - 10, 18: no supplied result shows the three-thread real-process trial
    passing after update recovery. Both styles' lost answers, retries,
    exhausted counts, release and late-copy holds have world stories;
    the real-Forgejo restart trial remains the after-landing proof.
    The fake reads a rebase copy's head at landing, so the dropped-push
    story uses an explicit branch rewrite and does not establish the
    real forge's concurrent-push behaviour (skein's `forge.md`, section 9).
  - 8.2, step 3: a head already behind taking its base first stands on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/update_first.rs`). The request
    `landing-fixes` of 2026-11-06 has its proof in the next landing plan on a real
    deployment, and no record shows one, so how much of a change's wait it saves,
    and how soon a moving base reaches the bound (section 20), are unmeasured.
  - 8.4, 11: the `landing-fixes-2` request of 2026-11-06 takes the next
    landing plan on a real deployment as its proof, and no result is supplied.
    The fake-forge stories cover a clean update's named failed check, its log
    before the base's contents, an unnamed check and a resolution's conflict
    words (`tests/engine/domain/tests/suite/forge_routes/repair_cause.rs`), and
    a retried repair's saved branch deleted as it closes (`saved_close.rs`).
    They do not establish that deployment's branch cleanup or that the new
    brief would avoid the reported failed `read_job`.
  - 8.11, *Which target*: the coding charter's sentence and the refusal's words
    stand on a script alone
    (`tests/engine/domain/tests/suite/forge_routes/question_target.rs`). No real
    model has read them, and the request `landing-fixes` of 2026-11-06 has the same
    proof as 8.2's, so whether a producer's first question now reaches its change
    is unobserved.
  - 8.11, *The next producer merges the landing tip first*: a tip its start
    already contains, a clean merge committed before its run and the advice
    for a conflict have scripted coverage alone. No deployment replay on
    record proves the `resume-told` request's resumed producer and its answerers
    read the tree rightly (`worker.status.md`, sections 5 and 10).
  - 8.9: a refusal's "adopt the repository again" has no story from the browser. The engine takes an adopted repository's adoption (`readopt`, `temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`), and by reading alone the form sends one (`../client/README.md`, 4.4); it opens at its defaults, so the person types the repository, its role, prefix and landing branch again.
  - 4.1: a removal without the box, its group's act answered in its
    decision: an owned or fake forge's removal asks the box link for
    `remove N` whether or not a pipe is there, and without one waits for
    an answer that never comes, so it is never whole and the
    playground's reset stops at its forge
    (`temper-engine-domain-forge/src/removal.rs`, `begin`, sets
    `box_removed` by the forge's mode alone;
    `temper-service/src/route/box_link.rs`, `remove`).
  - 4.4, 8.3: on GitHub, statuses and dispatches by temper's App for a
    project that lands by turn, and verdicts posted where an installation
    grants `statuses: write`; `status` is false there.
  - 4.4, 9.1, 18: the forge's own merge queue beyond a queued merge kept
    pending: `Pull.queued` and `dequeued` are not read, a queued change
    keeps temper's turn, an ejection makes no repair, the panel shows no
    place in it, and adoption's words leave out the queue, signatures,
    linear history, code owners and the App's bypass
    (`temper-service/src/family.rs`).
  - 7.1, 12: the findings topic and findings (`findings.md`).
  - 7.3: a hold parked until a landing. An explicit landing subscription
    with no paths receives a known landing as kept news; it does not wake
    a parked hold.
  - 8.1: per-change bounds and gates are not caller overrides: `change` takes parameters 1
    to 6, 9 to 12, 14 and 16, its bounds, the questions it takes among
    them, are the deployment's `change_policy`, and its gates its landing
    branch's rules' (`temper-engine-domain/src/route/forge_route.rs`,
    `start_change`); its page's line that a change takes no words.
  - 8.1: a goal's issue can still reach its forge with an empty title. A spec
    whose first line is blank or a lone full stop leaves no sentence
    (`temper-engine-domain/src/route/forge_route.rs`, the projection's `title`;
    `temper-engine-domain-forge-issues/src/text.rs`, `title`), and
    `temper-engine-domain-forge/src/domain.rs`, `project_write`, writes it as it
    is. The request `producer-failure` of 2026-11-05 has no empty title reach a
    forge by any route; its plan's cut took the pull request's alone, an issue
    making its own from a goal's spec. This is from reading: no story writes one.
  - 8.2: the pull request's body naming its goal's issue.
  - 8.3: a person's approval as a gate; a review's lens; gates run at once
    when the change asks; a verdict posted as a commit status.
  - 8.4: a no-change on a change under another hold stays on the row, but the
    hold that stands does not say its words: `hold_change_decision`
    (`temper-engine-domain/src/route/forge_route.rs`) reads them for
    `Hold::Failed` alone, and they are in no panel. A release reads the result
    as a failed producer's, so its words first reach a person in the `Failed`
    hold that follows, and the change does not end. Seen by reading: the step
    test stops at the row
    (`a_questions_hold_keeps_the_result_until_release_and_says_no_change_words`).
  - 8.4: a result ending a stalled hold has no world: a live delegate does not
    stall, so `temper-engine-domain-forge/src/tests/change.rs` gives the held
    row by `Event::Change`, and only a green head's worlds run the release its
    `EndHold` asks for
    (`temper/tests/engine/domain/tests/suite/forge_routes/repair_green.rs`).
  - 8.4: the request `parallel-faults` of 2026-11-06 has its proof in a rerun of
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
  - 8.4: widening exhausted spend with ctl before release: `task amend`'s
    `--budget` is not implemented (`../client/tools.md`, 5.5). A spend
    hold offers cancellation or asking afresh instead.
  - 8.4: `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`), and the plan
    `first-goal-fixes` edits neither, the ui drafts holding them, so a cancel
    is said by `task show` and not by this line.
  - 8.4: the walk the request `brief-failure` of 2026-11-05 ends with has not
    passed: on a real Forgejo, a change whose CI goes red gets a repair that
    starts, and a repair whose brief can't be read ends held with the reason in
    `temper ctl`. Its QA rung is not drafted yet, so both stand on the fake
    forge's stories alone
    (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`).
  - 8.4: `[the reviews could not be read: REASON]`, a gate's repair standing on
    its pull, is unobserved: it is built (`temper-engine-domain-forge/src/brief.rs`,
    `brief_answer`), but those stories run a CI repair's pair alone and no test
    holds its words.
  - 8.4: a released resolution or merge job that failed holds again, where the
    section has a resolution asked afresh: `released_state`
    (`temper-engine-domain-forge-change/src/domain.rs`) keeps `Resolving {
    requested: true }`, the row keeps the failed status
    (`temper-engine-domain-forge/src/domain.rs`, `change_standing`), and
    `delegated` reads it, so no fresh task is made. A failed producer and a failed
    repair take `requested: false`. This is from reading: no test or story
    releases a failed resolution. The `producer-failure` plan, whose request asks
    after a failure's shape and not its release, pins only its hold
    (`failed_repairs_and_resolutions_do_not_take_a_pushed_head`).
  - 8.4: a green head ending a repair has no story for a repair waiting for an
    answer to its question (none asks from a repair), a repair finding nothing
    to do (`NoChange`, read as failed), or a task closing or held with its
    attempt running, taken as a run in progress
    (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate_running`):
    the policy's tests hold a failed status and `delegate_running` alone
    (`temper-engine-domain-forge-change/src/tests/green.rs`).
  - 8.4, 8.5: a goal's landing held past its repairs, then released after
    a change into its goal branch (`../draft/reference-landing.md`,
    section 9's second story), is built, but its story is ignored: after
    the released landing, the restart oracle finds the deployment's
    `messages` number changed across a restart, its cause not yet found
    (`tests/engine/domain/tests/suite/forge_routes/goal_landing_held.rs`).
  - 8.4, 10, 11.1: a goal's landing whose update into main conflicts,
    settled by the merge job and a resolution on its goal's branch
    (`../draft/reference-landing.md`, section 9's first story), is built
    by every change's path, the job writing the branch under the goal's
    hold as the landing's resolution does
    (`temper-engine-domain-forge/src/items.rs`, `change_branch_holder`),
    but no story runs it: the merge job's stories are changes into a
    goal's branch and into main, none a goal's landing
    (`tests/engine/domain/tests/suite/forge_routes/merge_job.rs`).
  - 8.5: a retargeted pull request retargeted back on release; the change
    is held as a failed effect.
  - 8.7: the explicit choice to adopt a GitHub repository without CI.
  - 8.8: the rule and criterion of a change's last wait or refusal, and
    its charter, on its panel.
  - 8.10: a change drained.
  - 8.11: a dependent's landing holding its dependency's pushed head in
    g's ancestry is unobserved in stories, the fake forge squashing
    merges (skein's `fake-forge.md`, 2.2).
  - 8.11: what a step short of message room leaves for the next. A
    question's `Stands` is taken, foreman, asked or held
    (`temper-engine-domain-forge-change/src/boundary.rs`, and the store's
    `Stands`), so nothing holds reasons awaiting passing up, whose stored
    ceiling is not in the schema, or a foreman's words awaiting passing
    down. With no room `relay`
    (`temper-engine-domain-forge-change/src/relay.rs`) leaves the question
    as it stood and wakes the change, while the step's commit takes the
    foreman's verdict from the inbox (jig's `domain/tasks.md`, 8.2), so the
    question waits on an ended foreman. A question released past the bound
    at such a step meets the spent allowance again at the next: the release
    is that step's alone, and `next_act` holds the change.
  - 8.11: a repair's or a resolution's question, relayed as a producer's
    is (`temper-engine-domain/src/route/forge_route/relay.rs`,
    `asking_delegate`), and its words, refused as a producer's are, are
    unobserved in stories: every relay story runs the change's producing
    task only
    (`tests/engine/domain/tests/suite/forge_routes/questions.rs`,
    `foremen.rs` and `goal_questions.rs`).
  - 9: a branch a tier follows still pauses its queue on its own CI's
    red and makes a repair, where its tiers judge it instead
    (`temper-engine-domain-forge-change/src/domain.rs`, `base_ci`).
  - 12: the issue's milestones but its lifecycle and revisions: the plan
    accepted, a change landed or held, a report kept.
  - 12.1: questions on the forge.
  - 13: participating objects beyond their news; a landing waiting for
    someone else's merge where temper may not merge (also 4.4 and 9).
  - 14: upstream contributions.
  - 18: the rules as settings (8.9) in the forge's world: `tests/engine/forge`
    tells none of them. The engine's domain world tells a rule changed
    between two merges, a rule refused in its change's decision, a
    change kept on the charters it was made with and one held when its
    charter is no longer offered, and a restart
    (`tests/engine/domain/tests/suite/settings/kinds.rs`,
    `forge_routes/kind_charters.rs`); no world tells a gate a rule adds
    asked for on a queued change, a gate a rule drops no longer waited
    on.
- **Departs:** 5.7: a project's first run's words count their span from the
  epoch: `temper-engine-domain-forge/src/status/words.rs`, `specification`,
  takes `Wall::EPOCH` with no brief, so the agent reads "Since the last brief:"
  and some half a million hours, where a brief has never been written. This
  section names no span for a first brief, the plan's test gives its row a brief
  (`its_words_say_what_landed_since_the_brief`), and no reason is recorded.
- **Design departs:** 5.6, 5.7, from the request `ui-home-status` of 2026-11-05,
  each decided by its plan's cut, for the user to see:
  - 5.6: `Looked` keeps `next_seen`, a field the request doesn't name, where the
    request has a new sitting set `seen` to the current `landed`. That would read
    as nothing landed at the moment the person comes back; the request's own
    definition of `seen`, the count at `previous`, is what holds.
  - 5.7: `ProjectFacts.running`, a field the request doesn't name, so that "none
    running" survives a restart and a run held past an hour never stops the next.
  - 5.7: a run is a procedure of the project's period that delegates charter 9,
    where the request has an agent's task under the project's pool, because jig's
    connector task takes the project's whole ceiling as its authority and can't
    narrow it, and an agent reading change titles should not hold merge
    authority. So a project whose policy's delegation doesn't name charter 9 has
    no brief, and a period its people's pools reserve whole has none either,
    which a stock project's room holds off (`../protocol/configuration.md`, 13.4).
  - 5.7: a run's task is kept, where the request has it forgotten once read,
    because jig forgets a result only for a person's request (section 20).
  - 5.7: a live goal's "landed and total changes" are its delegates done and
    total, because the forge keeps no count a goal, and "failures the forge holds
    since" are what each tier holds now, because it keeps no times on them.
- **Design departs:** 5.7, from the request `ui-walk-large` of 2026-11-06, decided by its
  plan's cut: the room that request has a stock project keep for its brief is no
  brief's own, so a member's first spend or the forge's repairs can take it and the
  brief falls back to the computed sentence, because a shared project's owners can
  fund their period (section 20).
- **Design departs:** 5.8, 5.9, from the request `ui-away` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - 5.8: `Away` keeps `off` in place of the request's `on`: "23:10 to 08:03" needs
    the time it went off, and a row holding both could disagree with itself. On is
    `off` none; the wire's `SetAway` keeps `on`.
  - 5.9: `Decided` keeps `party`, a field the request doesn't name, and its document
    is the asking party's alone: "decided for you" would be false on a colleague's
    Home, and Overturn sends the requester's message.
  - 5.9: the caller's person is found through its chat, where the request has
    `domain.core.tasks.delegation`: a chat's goal has the chat as its requester, so
    the root takes jig's `served_chat`, then the chat's person.
  - 5.9: "Only goals call it" is held by the root, where the request leaves it to
    charter 1's words: goals and chats share that charter, so a chat's call answers
    ask (`agent.status.md`, 4.3).
- **Design departs:** 8.4, from the request `repair-green` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - The request's rule 1 ends a held repair when CI has passed at the head. A
    hold at gating, past the bound or short of authority, stays held
    (`temper-engine-domain-forge-change/src/domain.rs`, `held_on_ci`), because
    its state can't say whether it is a gate's or CI's, and the plan kept what a
    hold stores.
  - Rule 1's hold ends on a wake alone, and a poll wakes a change only when what
    it read of the pull request moved (`temper-engine-domain-forge/src/domain.rs`,
    the pull read's `wakes`). Under required contexts a check they don't name can
    keep the pull request's CI as it was, so that hold waits for a hint or a
    person's release, and where nothing delivers hints, for the release. It
    costs time and makes no fresh repair.
  - Rule 1 ends a retrying repair with no agent's work. A run ending into a retry
    wakes no change, and the change reads again within `poll`
    (`temper-engine-domain-forge/src/domain.rs`, `repair_poll`), so a backoff
    shorter than that starts another run, which rule 2 leaves to finish.
  - The request has its stories in the forge world or lower. The held and
    retrying ones run in the engine's domain world, above it
    (`tests/engine/domain/tests/suite/forge_routes/repair_green.rs`), where a
    task's phase, its cancel and the deployment's release meet; the forge world
    (`tests/engine/forge/tests/suite/green.rs`) tells the policy's.
- **Design departs:** 8.4, from the request `brief-failure` of 2026-11-05, each
  decided by its plan's cut, for the user to see:
  - The request's true capacity is a place, an account or a brief slot. 8.4 also
    pauses, with no bound, for a forge client `Busy` at the brief's deadline, a
    forge not connected and a connector with no room, so a repair waits out a
    disconnected forge unheld, though saying why, rather than making a person
    release each.
  - The request has a CI repair's stories in the forge world. They run in the
    engine's domain world (`tests/engine/domain/tests/suite/forge_routes/brief_failure.rs`),
    where the core and the connector meet; the forge world
    (`tests/engine/forge/tests/suite/brief.rs`) tells a lone section's failure.
- **Design departs:** 8.4, 8.11 and jig's `domain/tasks.md`, 5.7, from the request
  `producer-failure` of 2026-11-05, each decided by its plan's cut, for the user
  to see:
  - 8.11, and jig's 5.7: the request recommends refusing an asker's `finish` at
    the call, with the words `Your question is still open: call wait; the answer
    reaches this run.` The run parks instead, as a done finish does: smith judges
    a `finish` with no call to its host, and the tasks child decides at the
    answer's commit, the run and its host knowing nothing of it (jig's
    `domain/engine.md`, 7.4). A refusal at the call would need a new crossing from
    jig through the worker to smith, and costs the run a turn that parking does
    not. So those words reach no run; the stock coding charter says the same
    before it finishes, and a resumed run's transcript shows its earlier `finish`
    as accepted, then its waking instruction and the answer.
  - 8.11: the request has a question left by its asker's end say so to whoever
    was waiting. One passed up says so at its answer, not at the end, because
    saying it once at the end would need a stored mark and the request names no
    store change. So a requester that never answers hears nothing of it.
  - 8.4: the request has its failing test in the forge world. Its stories run in
    the engine's domain world
    (`tests/engine/domain/tests/suite/forge_routes/producer_failure.rs`), where the
    core and the connector meet, with the policy's in
    `temper-engine-domain-forge-change/src/tests.rs`; `tests/engine/forge` tells
    none of it.
- **Design departs:** 8.11, from the request `report-in-finish` of 2026-11-05,
  decided by its plan's cut, for the user to see: the call's refusal and the
  words at a step take a change's repair and resolution beside its producing
  task, where the request names the producing task alone, because all three run
  under the one charter and each would pay the same turn
  (`temper-engine-domain/src/route/forge_route/relay.rs`, `asking_delegate`).
- **Design departs:** 8.2, step 3, from the request `landing-fixes` of 2026-11-06,
  item 1, decided by its plan's cut: the update first saves the change's wait on
  CI, where the request counts its 5:26 as CI's, because temper has no effect that
  stops a run the forge has started and a pull request comes before its update. The
  replaced head's run goes by the repository's own CI settings (section 20).
- **Departs:** 9: the first keeps its turn only until the next ready change
  steps: `queue_first` (`temper-engine-domain-forge/src/domain.rs`) chooses
  among `queue_ready`'s changes, which leave out one at `Updating`, `Checking`,
  `Gating` or `Landing`, so a queued change stepped while the first is there
  takes `First` too. Nothing wakes the queue at the first's update or merge, so
  it wants a step of that change's own. `changes_document_keeps_the_first_while_it_updates`
  pins the choice and the document's two owning rows; no story runs it, and no
  reason is recorded.
- **Departs:** 4.5: no step writes `Made`: a run's end goes to
  `Adopting` in one decision, or erases a row with no `into`
  (`temper-engine-domain-forge/src/making.rs`, `made`), so the variant is
  read and carried by the store and the wire, and never written; no
  reason is recorded.
- **Departs:** 8.11: a change taking `foreman` gives its producing, each
  repair and each resolution half the spend the change may give it,
  keeping the rest for the foremen of that agent's questions, since a
  foreman runs while the agent that asked waits
  (`temper-engine-domain/src/route/forge_route.rs`, `change_delegate`);
  8.4 and 8.11 make a change's delegates under its own authority and
  name no such split.
- **Departs:** 8.11: every answer relayed down is cut from its end to
  `tasks.message_bytes` and marked with `…`, as a question passed up is.
  This keeps the message within its admission bound, where the design says
  the answer's words are whole.
