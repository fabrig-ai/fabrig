# forge: status

2026-11-05. Where temper's code lags or departs from [forge.md](forge.md).

- **Not yet:**
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
  - 8.2: the pull request's body naming its goal's issue.
  - 8.3: a person's approval as a gate; a review's lens; gates run at once
    when the change asks; a verdict posted as a commit status.
  - 8.4: widening exhausted spend with ctl before release: `task amend`'s
    `--budget` is not implemented (`../client/tools.md`, 5.5). A spend
    hold offers cancellation or asking afresh instead.
  - 8.4: `change show`'s `held:` line for a cancelled delegate reads `a
    delegate or a write failed for good`: the panel's hold is the client
    wire's `failed` (`temper-client-protocol-wire/schema/client-wire-v1.schema`,
    `Hold`), and its words are the client's
    (`temper-client-domain/src/forge/words.rs`, `FailedHold`).
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
  - 8.11: an agent's own words: the `message` call does not refuse them,
    and at a step they take the fallback feedback for a requester
    (`temper-engine-domain/src/route/forge_route/relay.rs`, `take`); the
    stock coding charter does not say where its report goes.
  - 8.11: a repair's or a resolution's question, relayed as a producer's
    is (`temper-engine-domain/src/route/forge_route/relay.rs`,
    `asking_delegate`), is unobserved in stories: every relay story's
    asker is the change's producing task
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
    no brief, and a period its people's pools reserve whole has none either
    (`../client/README.status.md`, 3.5).
  - 5.7: a run's task is kept, where the request has it forgotten once read,
    because jig forgets a result only for a person's request (section 20).
  - 5.7: a live goal's "landed and total changes" are its delegates done and
    total, because the forge keeps no count a goal, and "failures the forge holds
    since" are what each tier holds now, because it keeps no times on them.
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
