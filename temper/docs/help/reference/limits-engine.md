# Engine limits

Kind: reference

Every limit engine.limits.overrides and a worker's limits.overrides take: its unit, the small profile's value and what it bounds.

The table uses the small profile, not the standard profile selected by temper-box init. Standard tasks.tree_tasks is 384; small is 32 as listed below. engine.limits.profile selects the profile; engine.limits.overrides replaces named values. Check the box's configuration for its effective value; these are profile defaults.

The tree limit counts all tasks made over the requester's tree's life, including its root and ended tasks. Ending a task frees live slots, not lifetime capacity; a restart does not reset the count. A task's delegated authority may leave less room than this deployment limit.

A change batch reserves one task per member and shares the descendant allowance across members within each change policy's cap. These are prospective allowances for that call, not retained reservations across calls. A task-tree refusal names available and minimum required capacity and says whether fewer changes per call can fit (temper help change).

temper sizes the engine's watch rooms itself: a stream holds 31 watches, and the engine holds clients.streams times that, in all and for one party. Raise clients.streams for more.

## tasks

- `tasks.placed_holds`: count, 4. Bounds placed holds in the deployment, including lifted holds kept for their tasks.
- `tasks.project_placed_holds`: count, 4. Bounds placed holds retained in one project.
- `tasks.party_placed_holds`: count, 2. Bounds placed holds retained for one party owner.
- `tasks.exemptions`: count, 8. Bounds tasks one standing placed hold may exempt.
- `tasks.tasks`: count, 8. Bounds retained live task slots, name-index entries and retry alarms.
- `tasks.funders`: count, 8. Bounds retained finite period, pool and recurring allotment identities.
- `tasks.project_tasks`: count, 8. Bounds live tasks per project, including every member of an admitted batch.
- `tasks.project_goals`: count, 8. Bounds live tracked tasks per project.
- `tasks.tree_tasks`: count, 32. Bounds lifetime tasks made in a requester tree, including its root.
- `tasks.depth`: count, 8. Bounds structural depth below the requester-tree root, which has depth zero.
- `tasks.delegates`: count, 4. Bounds live direct delegates of one requester task.
- `tasks.references`: count, 8. Bounds introduced peer references per live task.
- `tasks.subscriptions`: count, 4. Bounds standing interests per task.
- `tasks.batch`: count, 2. Bounds directly created tasks in one atomic nonempty `Make` batch.
- `tasks.batch_bytes`: bytes, 131,072. Bounds the complete owned bytes of one task batch.
- `tasks.dependencies`: count, 8. Bounds immutable dependency identities per task, including its remaining live waiting subset.
- `tasks.circle`: count, 8. Bounds tasks a circle of waits names in a refusal or a hold.
- `tasks.holdings`: count, 4. Bounds resources required by one task.
- `tasks.hold_kinds`: count, 8. Bounds connector resource kinds installed in this deployment.
- `tasks.resource_reports`: count, 128. Bounds cached named connector admission reports.
- `tasks.pools`: count, 8. Bounds distinct pools whose latest connector slot counts are retained.
- `tasks.hold_segments`: count, 4. Bounds literal segments in one resource name.
- `tasks.hold_bytes`: bytes, 1,024. Bounds aggregate literal bytes in one resource name.
- `tasks.hold_waiters`: count, 8. Bounds tasks waiting on any one resource.
- `tasks.hold_wait_ms`: milliseconds, 120,000. Bounds the wait for holds before a task is held for review.
- `tasks.grace_ms`: milliseconds, 600,000. Bounds the span a cancel may give its task to finish.
- `tasks.inputs`: count, 8. Bounds retained input identities checked by the root before delegation.
- `tasks.spec_bytes`: bytes, 4,096. Bounds combined specification words and byte-valued parameter bytes per task.
- `tasks.parameters`: count, 4. Bounds parameters per specification.
- `tasks.result_bytes`: bytes, 512. Bounds result or reason bytes, with one bounded reason and one bounded partial result retained on cancellation.
- `tasks.run_inbox`: count, 8. Bounds messages a run starts with and holds relayed and unread in its inbox.
- `tasks.inbox_messages`: count, 8. Bounds whole unread word messages kept by one live task.
- `tasks.inbox_bytes`: bytes, 4,096. Bounds total unread word bytes kept by one live task.
- `tasks.message_bytes`: bytes, 4,096. Bounds bytes in one admitted word message.
- `tasks.person_stall_ms`: milliseconds, 3,600,000. Bounds time an active person task waits before becoming news for its requester.
- `tasks.proposal_stall_ms`: milliseconds, 3,600,000. Bounds time an undecided proposal waits at a nonfinal holder before passing upward.
- `tasks.escalation_stall_ms`: milliseconds, 3,600,000. Bounds time an undecided escalation waits at a nonfinal holder before passing upward.
- `tasks.saved_resources`: count, 4. Bounds connector resources on which a task has saved state.
- `tasks.contract_choices`: count, 1. Bounds distinct-code choices in a nonempty verdict contract.
- `tasks.charters`: count, 4. Bounds retained configured agent-charter numbers.
- `tasks.charter_parts`: count, 8. Bounds parts per charter.
- `tasks.charter_documents`: count, 2. Bounds documents per charter.
- `tasks.authority_grants`: count, 4. Bounds resource grants per task authority carrier.
- `tasks.authority_segments`: count, 4. Bounds base segments per carried grant pattern.
- `tasks.authority_bytes`: bytes, 2,048. Bounds total bytes across every grant's base and terminal segments in one authority carrier.
- `tasks.executor_kinds`: count, 4. Bounds carried authority executor-permission entries.
- `tasks.facts`: count, 32. Bounds content-free observation capacity.
- `tasks.room_refusals`: count, 8. Bounds recent owner-proven room refusals.
- `tasks.retries.transient.retries`: count, 3. Bounds failures allowed before holding a task for transient failures.
- `tasks.retries.transient.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for transient failures.
- `tasks.retries.transient.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for transient failures.
- `tasks.retries.permanent.retries`: count, 3. Bounds failures allowed before holding a task for permanent failures.
- `tasks.retries.permanent.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for permanent failures.
- `tasks.retries.permanent.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for permanent failures.
- `tasks.retries.run.retries`: count, 3. Bounds failures allowed before holding a task for run failures.
- `tasks.retries.run.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for run failures.
- `tasks.retries.run.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for run failures.
- `tasks.retries.agent.retries`: count, 3. Bounds failures allowed before holding a task for agent failures.
- `tasks.retries.agent.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for agent failures.
- `tasks.retries.agent.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for agent failures.
- `tasks.retries.lost.retries`: count, 3. Bounds failures allowed before holding a task for lost failures.
- `tasks.retries.lost.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for lost failures.
- `tasks.retries.lost.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for lost failures.
- `tasks.retries.invalid.retries`: count, 3. Bounds failures allowed before holding a task for invalid failures.
- `tasks.retries.invalid.base_ms`: milliseconds, 1,000. Bounds the initial exponential-backoff delay for invalid failures.
- `tasks.retries.invalid.max_ms`: milliseconds, 30,000. Bounds the exponential-backoff delay for invalid failures.
- `tasks.retries.infrastructure.retries`: count, 12. Bounds failures allowed before holding a task for infrastructure failures.
- `tasks.retries.infrastructure.base_ms`: milliseconds, 10,000. Bounds the initial exponential-backoff delay for infrastructure failures.
- `tasks.retries.infrastructure.max_ms`: milliseconds, 300,000. Bounds the exponential-backoff delay for infrastructure failures.

## people

- `people.program_hosts`: count, 9. Bounds configured workers plus the engine host in one retained program outcome.
- `people.program_host_name_bytes`: bytes, 32. Bounds whole configured worker name bytes in retained operation answers.
- `people.operations`: count, 8. Bounds task operations in one keyed compound request.
- `people.batch`: count, 2. Bounds members of one requested or proposed batch.
- `people.people`: count, 4. Bounds retained parties and provider/subject index entries.
- `people.inbox_entries`: count, 8. Bounds visible cached entries and unread count.
- `people.inbox_tasks`: count, 16. Bounds live task and person-proposal census rows, including per-conversation positions until forgotten.
- `people.inbox_recipients`: count, 32. Bounds recipient references in one task projection, independent of cached entries.
- `people.sign_ins`: count, 8. Bounds live secret-free sign-ins and expiry alarms.
- `people.projects`: count, 3. Bounds retained project role sets.
- `people.holdings`: count, 8. Bounds unique-person holdings per project role set.
- `people.initial_owners`: count, 2. Bounds retained configured first-owner entries.
- `people.requests`: count, 2,000. Bounds retained completed keys plus ordinary flights' answer reservations across the engine.
- `people.room_refusals`: count, 8. Bounds unsaved entrance-pressure records independently of diagnostics.
- `people.pending`: count, 8. Bounds simultaneous keyed flights, including answered moves waiting for Written.
- `people.waiters`: count, 4. Bounds reply destinations per flight, including its first caller.
- `people.identity_bytes`: bytes, 1,024. Bounds combined subject, login and display-name bytes per party.
- `people.words`: bytes, 2,048. Bounds opening words, escalation rejection and settings-proposal reason bytes, including pending and completed copies.
- `people.amendment_bytes`: bytes, 8,192. Bounds owned bytes carried by one keyed task amendment.
- `people.sign_in_lifetime_ms`: milliseconds, 86,400,000. Bounds the lifetime of a person sign-in.
- `people.request_retention_ms`: milliseconds, 86,400,000. Bounds time for which completed person keys replay their durable answers.
- `people.facts`: count, 32. Bounds capacity of optional content-free observations.

## fleet

- `fleet.configured_workers`: count, 8. Bounds configured workers, including those not in contact.
- `fleet.worker_name_bytes`: bytes, 32. Bounds bytes in each configured worker name.
- `fleet.program_sources`: count, 12. Bounds program source names retained per connected worker, independently of run slots.
- `fleet.source_detail_bytes`: bytes, 4,096. Bounds diagnostic detail retained for each login or sign-out failure.
- `fleet.workers`: count, 2. Bounds workers in contact at once.
- `fleet.needs_bytes`: bytes, 65,536. Bounds owned charter/model requirements retained for admission and placement.
- `fleet.engine_slots`: count, 0. Bounds slots hosted inside the engine.
- `fleet.slots`: count, 2. Bounds runs a worker hosts at once.
- `fleet.workstreams`: count, 8. Bounds the workstreams kept per worker, for placement to prefer.
- `fleet.attempts`: count, 8. Bounds the parent's attempts awaiting placement, acknowledgement or adoption, including replaced attempts.
- `fleet.grants`: count, 8. Bounds credential names held by each attempt, across both protocol tables.
- `fleet.calls`: count, 32. Bounds relayed calls the parent serves at once.
- `fleet.call_name_bytes`: bytes, 64. Bounds bytes of a run's opaque call name retained for each relay.
- `fleet.progress_bytes`: bytes, 1,024. Bounds whole tool-name bytes in one best-effort observation.
- `fleet.turns`: count, 32. Bounds turns awaiting adoption or the parent's commitment, across all attempts.
- `fleet.grace_ms`: milliseconds, 360,000. Bounds the wait for a lost worker to return or an unclaimed attempt to be adopted.
- `fleet.facts`: count, 32. Bounds facts kept until the parent drains them.
- `fleet.history`: count, 8. Bounds recent lost channels and refused hellos retained separately from diagnostics, at most this many of each.

## authority

- `authority.projects`: count, 2. Bounds live project policies.
- `authority.roles`: count, 4. Bounds roles per project policy.
- `authority.requirements`: count, 4. Bounds generic requirements per deployment or project policy.
- `authority.facts`: count, 16. Bounds supplied verdicts and guards per effect question.
- `authority.grants`: count, 4. Bounds grants per admitted authority value.
- `authority.executors`: count, 4. Bounds permitted executor entries per admitted authority value.
- `authority.segments`: count, 4. Bounds base or resource-name segment count.
- `authority.segment_bytes`: bytes, 256. Bounds bytes per base segment or terminal value.
- `authority.implications`: count, 8. Bounds retained connector-kind implication pairs.
- `authority.batch`: count, 2. Bounds directly delegated tasks per batch question.
- `authority.accounts`: count, 4. Bounds model-account usability reports per run question.
- `authority.writes`: count, 4. Bounds written resources per run question.

## brief

- `brief.briefs`: count, 4. Bounds briefs gathering at once.
- `brief.sections`: count, 10. Bounds sections in one brief.
- `brief.read_bytes`: bytes, 8,192. Bounds size a connector may report for one section.
- `brief.brief_bytes`: bytes, 16,384. Bounds total size of one completed brief.

## accounts

- `accounts.token`: bytes, 7,934. Bounds the bytes of an account credential token.
- `accounts.sign_in_deadline_ms`: milliseconds, 300,000. Bounds how long a sign-in waits for its redirect.
- `accounts.program_hosts`: count, 9. Bounds hosts a program sign-out can await.
- `accounts.accounts`: count, 4. Bounds accounts in the deployment.
- `accounts.holder_accounts`: count, 4. Bounds accounts one holder keeps, the deployment or a party.
- `accounts.program_names`: count, 12. Bounds lifetime program names, retired names included.
- `accounts.sources`: count, 3. Bounds sources one account keeps, in its holder's order.
- `accounts.sign_ins`: count, 2. Bounds sign-ins, LLM-key keeps, moves out and moves in under way at once.
- `accounts.credentials`: count, 3. Bounds retained credentials and their expiry alarms.
- `accounts.backoff_base_ms`: milliseconds, 2,000. Sets the initial backoff after a temporary source failure.
- `accounts.backoff_max_ms`: milliseconds, 60,000. Bounds the backoff after a temporary source failure.
- `accounts.retry_ms`: milliseconds, 30,000. Bounds the wait before retrying a source that needs sign-in or loading.
- `accounts.move_deadline_ms`: milliseconds, 60,000. Bounds how long a move out waits for Written after its answer.
- `accounts.spent_attention_ms`: milliseconds, 300,000. Sets the remaining spend-limit wait at which a source needs attention.
- `accounts.facts`: count, 32. Bounds account observations waiting to be drained.

## views

- `views.runs`: count, 4. Bounds live runs whose provisional text is retained, watched or not.
- `views.subjects_per_change`: count, 32. Bounds subjects named by one change event.
- `views.text_bytes`: bytes, 65,536. Bounds retained provisional bytes per live run.
- `views.facts`: count, 32. Bounds best effort observations waiting to be drained.

## notes

- `notes.scopes`: count, 8. Bounds scope indexes kept in memory at once.
- `notes.entries_per_scope`: count, 16. Bounds entries admitted in one scope.
- `notes.party_entries`: count, 12. Bounds entries admitted in one party's scope.
- `notes.party_bytes`: bytes, 120. Bounds description bytes in one party's scope.
- `notes.pattern_bytes`: bytes, 256. Bounds bytes in one connector pattern.
- `notes.description_bytes`: bytes, 256. Bounds bytes in one note description line.
- `notes.body_bytes`: bytes, 32,768. Bounds bytes in one note body.
- `notes.references`: count, 8. Bounds task references in one entry.
- `notes.load_rows`: count, 4. Bounds records in one store load page.
- `notes.lines`: count, 4. Bounds index lines in one answer.
- `notes.recalled`: count, 4. Bounds entries in one recall page.

## hub

- `hub.program_sources`: count, 12. Bounds local program sources.
- `hub.slots`: count, 2. Bounds runs and jobs hosted at once: the shared slots.
- `hub.accounts`: count, 8. Bounds distinct grants retained for a hosted attempt.
- `hub.charter_bytes`: bytes, 65,536. Bounds bytes of a charter.
- `hub.transcript_bytes`: bytes, 262,144. Bounds opaque transcript bytes, including the committed call tail.
- `hub.delivery_evidence_bytes`: bytes, 4,096. Bounds opaque evidence bytes in one settled delivery call.
- `hub.turn_bytes`: bytes, 65,536. Bounds bytes of one completed turn body.
- `hub.outcome_bytes`: bytes, 4,096. Bounds bytes of a run's declared outcome.
- `hub.detail_bytes`: bytes, 4,096. Bounds bytes of detail an answer carries for operators: the tail of what it is given.
- `hub.held`: count, 8. Bounds inbound events a run holds until it is live.
- `hub.inbox`: count, 8. Bounds messages a run starts with and refusals a turn or answer carries.
- `hub.event_bytes`: bytes, 69,632. Bounds bytes of an inbound event.
- `hub.run_calls`: count, 8. Bounds host calls a run has in flight at once, including delivery.
- `hub.facts`: count, 64. Bounds facts kept until the parent drains them.
- `hub.told`: count, 64. Bounds activity and live pieces kept for the core while the link cannot take it.
- `hub.turns`: count, 8. Bounds retained turns per attempt, with an additional window reserved for stopping.
- `hub.turn_queue_bytes`: bytes, 524,288. Bounds retained body bytes per attempt, with an additional window reserved for stopping.
- `hub.piece_bytes`: bytes, 4,096. Bounds bytes of one piece of live content.

## documents

- `documents.rows`: count, 4. Bounds rows in one constructed document.
- `documents.text_bytes`: bytes, 4,096. Bounds text bytes in one constructed document.
- `documents.chips`: count, 4. Bounds chips in one constructed document.
- `documents.lines`: count, 4. Bounds lines in one constructed document.
- `documents.versions.minimum`: count, 1. Sets the oldest served document contract version.
- `documents.versions.maximum`: count, 1. Bounds the served document contract version.

## core

- `core.settings.proposals`: count, 4. Bounds pending settings proposals.
- `core.settings.revisions`: count, 4. Bounds settings revisions held besides the newest.
- `core.settings.versions`: count, 8. Bounds charter versions held besides the newest settings view.
- `core.settings.history`: count, 64. Bounds revision records kept in the store beyond those held.
- `core.settings.alternatives`: count, 63. Bounds model alternatives in a charter policy.
- `core.settings.permissions`: count, 32. Bounds permissions in one settings policy value.
- `core.templates`: count, 8. Bounds configured templates and overrides per project policy.
- `core.fill_mark`: bytes, 268,435,456. Bounds store map bytes at or above which new party work is refused.
- `core.connectors`: count, 2. Bounds numbered connectors retained by the core.
- `core.resume_bytes`: bytes, 65,536. Bounds bytes retained for one task's resumable transcript tail.
- `core.run_bytes`: bytes, 16,384. Bounds owned bytes of the configured run policy and its model alternatives.
- `core.policy_bytes`: bytes, 16,384. Bounds bytes in one encoded mutable policy row the application journal accepts.
- `core.escalation_reason_bytes`: bytes, 512. Bounds bytes in an ended person escalation rejection reason.
- `core.load_slots`: count, 4. Bounds concurrent authenticated read loads, including retained result reads.
- `core.load_bytes`: bytes, 65,536. Bounds owned bytes brought by one authenticated store read.
- `core.call_records`: count, 32. Bounds named call replay and in-flight route slots.
- `core.call_answer_bytes`: bytes, 69,632. Bounds total bytes of a retained opaque call name, tool name and settled answer.
- `core.brief_parts`: count, 8. Bounds fragments when rendering a core-owned brief section.
- `core.brief_core_budgets.tail`: bytes, 2,048. Bounds transcript tail text bytes in a fresh run's brief.
- `core.brief_core_budgets.task`: bytes, 4,096. Bounds task section text bytes in one brief.
- `core.brief_core_budgets.dependencies`: bytes, 2,048. Bounds dependency section text bytes in one brief.
- `core.brief_core_budgets.attempts`: bytes, 1,024. Bounds attempt section text bytes in one brief.
- `core.brief_core_budgets.plan`: bytes, 2,048. Bounds plan section text bytes in one brief.
- `core.brief_core_budgets.notes`: bytes, 2,048. Bounds note index text bytes in one brief.
- `core.brief_core_budgets.preferences`: bytes, 2,048. Bounds whole requester preferences in one brief.
- `core.brief_core_budgets.now`: bytes, 4,096. Bounds a standing task's Now body in one brief.
- `core.public.goals`: count, 0. Bounds live public goals per board.
- `core.public.ended`: count, 0. Bounds ended public goals kept per project.
- `core.public.lines`: count, 0. Bounds public lines per goal and delegates per card.
- `core.public.depth`: count, 0. Bounds the lineage of a public card.

## link

- `link.chunk`: bytes, 16,384. Bounds the channel bytes read at once.
- `link.name`: bytes, 32. Bounds bytes in a worker name.
- `link.secret`: bytes, 256. Bounds bytes in a worker secret.
- `link.names`: count, 8. Bounds configured worker names.
- `link.opening`: count, 4. Bounds connections opening at once.
- `link.handshake_ms`: milliseconds, 10,000. Bounds the time to complete a channel opening.
- `link.hello_ms`: milliseconds, 10,000. Bounds the time to receive a worker hello.
- `link.ping_ms`: milliseconds, 5,000. Bounds the interval between channel pings.
- `link.silence_ms`: milliseconds, 15,000. Bounds channel silence before declaring the link lost.
- `link.stall_ms`: milliseconds, 30,000. Bounds how long a channel may stall.
- `link.backoff_ms`: milliseconds, 250. Sets the first redial backoff span.
- `link.ceiling_ms`: milliseconds, 30,000. Bounds the redial backoff span.
- `link.proved_ms`: milliseconds, 60,000. Sets the open-channel span that resets the redial backoff.
- `link.busy_ms`: milliseconds, 2,000. Bounds the wait before resending a turn answered busy.
- `link.facts`: count, 4,470. Bounds channel observations waiting to be drained.

## clients

- `clients.streams`: count, 2. Bounds client streams admitted at once.
- `clients.exchanges`: count, 8. Bounds client exchanges admitted at once.
- `clients.queued_bytes`: bytes, 65,536. Bounds encoded records held by one client stream, including its in-flight record.
- `clients.exchanges_per_client`: count, 2. Bounds exchanges in flight per sign-in, shared by its clients.
- `clients.provider_codes`: count, 16. Bounds volatile authorization codes.
- `clients.provider_tokens`: count, 64. Bounds volatile provider access tokens.
- `clients.public.reads`: count, 0. Bounds public reads admitted at once; zero turns public reading off.
- `clients.public.rate`: count, 0. Bounds public reads per second; zero turns public reading off.
- `clients.public.rate_burst`: count, 0. Bounds the public reads bucket at its fullest.
- `clients.public.bytes`: bytes, 0. Bounds public answer bytes per second.
- `clients.public.bytes_burst`: bytes, 0. Bounds the public answer bytes bucket at its fullest.
- `clients.public.answer_bytes`: bytes, 0. Bounds one public answer and every declared public kind.
- `clients.public.max_age`: count, 0. Bounds public answer freshness in seconds, at most sixty.

## clients_http

- `clients_http.streams`: count, 2. Bounds HTTP client streams admitted at once.
- `clients_http.calls`: count, 6. Bounds HTTP client calls admitted at once.
- `clients_http.public_calls`: count, 0. Bounds public HTTP calls in flight apart from signed-in calls.

## io

- `io.output`: bytes, 1,048,576. Bounds queued output per stream and the largest whole frame the stream can send.
