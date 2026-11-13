# Limits

Kind: reference

Every limit application.limits.overrides takes: its unit, the small profile's value and what it bounds.

## sign_in

- `sign_in.members`: count, 64. Bounds configured members and the forge connector's retained member bindings.
- `sign_in.pending`: count, 64. Bounds simultaneous sign-ins; a start past this count gets the busy page.
- `sign_in.deadline_ms`: milliseconds, 600,000. Bounds a sign-in's life from start through its return's answer and sets its cookie's lifetime.
- `sign_in.fetches`: count, 8. Bounds simultaneous exchanges with Google for sign-ins.
- `sign_in.body_bytes`: bytes, 65,536. Bounds an answer body from Google, for a key set or a token.
- `sign_in.url_bytes`: bytes, 2,048. Bounds a configured endpoint and the authorization URL given to the browser.
- `sign_in.secret_bytes`: bytes, 512. Bounds the Google client secret.

## picker

- `picker.page`: count, 30. Bounds repositories taken by one picker read, within the forge's page cap of at most 100.

## routes

- `routes.query_bytes`: bytes, 2,048. Bounds a forge route's query.

## root

- `root.journal.commits`: count, 3. Bounds commits awaiting the store's answer.
- `root.journal.held`: count, 1,536. Sets the retained held-delivery count; the uncapped journal does not use it for admission.
- `root.journal.writes`: count, 12,288. Bounds unique writes per commit, including the saved deployment header; a decision has one fewer slot.
- `root.journal.deliveries`: count, 512. Sets the door output count used in the default admission watermark, without capping a decision.
- `root.journal.transcript_bytes`: bytes, 65,536. Bounds transcript bytes or a wrapped child row's owned bytes admitted to the journal.
- `root.journal.result_bytes`: bytes, 14,592. Bounds owned result text or assignment section text admitted to a held delivery.
- `root.journal.run_bytes`: bytes, 65,536. Bounds a run charter's owned bytes in a held assignment.
- `root.loads.loads`: count, 2. Bounds issued store loads, including abandoned reads awaiting their terminal answer.
- `root.loads.rows`: count, 2. Bounds rows requested and accepted in one incoming store page.
- `root.loads.bytes`: bytes, 65,536. Bounds a kept page prefix, counting record slots and their owned bytes.
- `root.loads.read_bytes`: bytes, 65,536. Bounds owned bytes one authenticated store read brings for a person.
- `root.loads.reply_bytes`: bytes, 65,536. Bounds a decoded store answer before its kept prefix is cut.
- `root.loads.transcript_bytes`: bytes, 65,536. Bounds a row's owned bytes, including transcripts and wrapped child rows, before the prefix is cut.
- `root.brief.briefs`: count, 2. Bounds briefs the root gathers at once.
- `root.brief.sections`: count, 10. Bounds brief sections the root gathers at once.
- `root.brief.parts`: count, 2. Bounds parts in one gathered brief section.
- `root.brief.read_bytes`: bytes, 4,608. Bounds bytes read for one gathered brief section.
- `root.brief.brief_bytes`: bytes, 14,592. Bounds the body of a gathered brief.
- `root.brief.gather_ms`: milliseconds, 30,000. Bounds the time spent gathering a brief.
- `root.brief.budgets.task`: bytes, 4,608. Bounds task section bytes in a gathered brief.
- `root.brief.budgets.dependencies`: bytes, 128. Bounds dependency section bytes in a gathered brief.
- `root.brief.budgets.document`: bytes, 4,096. Bounds document section bytes in a gathered brief.
- `root.brief.budgets.ci`: bytes, 4,096. Bounds CI section bytes in a gathered brief.
- `root.brief.budgets.reviews`: bytes, 4,608. Bounds review section bytes in a gathered brief.
- `root.brief.budgets.pull`: bytes, 4,608. Bounds pull request section bytes in a gathered brief.
- `root.brief.budgets.attempts`: bytes, 128. Bounds attempt section bytes in a gathered brief.
- `root.brief.budgets.plan`: bytes, 128. Bounds plan section bytes in a gathered brief.
- `root.brief.budgets.notes`: bytes, 64. Bounds note section bytes in a gathered brief.
- `root.brief.budgets.preferences`: bytes, 2,048. Bounds whole requester preferences in a gathered brief.
- `root.brief.budgets.now`: bytes, 4,096. Bounds a standing task's Now body in a gathered brief.

## forge

- `forge.forges`: count, 3. Bounds retained forges of every kind and mode, counting an ended forge until its removal is whole.
- `forge.repositories`: count, 2. Bounds repositories retained by the forge connector, shared by every retained forge.
- `forge.tasks`: count, 8. Bounds tasks participating in the forge connector.
- `forge.holds`: count, 8. Bounds held task resources in the forge connector.
- `forge.subscriptions`: count, 8. Bounds subscriptions to forge topics.
- `forge.entries`: count, 8. Bounds durable outbox entries in the forge connector.
- `forge.named_calls`: count, 32. Bounds retained named calls and their connector payloads, independently of outbox entries.
- `forge.changes`: count, 8. Bounds live changes retained by the forge connector.
- `forge.issues`: count, 8. Bounds live issue projections retained by the forge connector.
- `forge.landing_members`: count, 2. Bounds changes and held member branches in one landing; at least each project's batch.
- `forge.resources_per_task`: count, 3. Bounds distinct forge names watched by one task, including held resources and subscribed branches.
- `forge.paths_per_subscription`: count, 4. Bounds paths retained by one forge subscription.
- `forge.name_bytes`: bytes, 64. Bounds names retained by the forge connector.
- `forge.output`: count, 80. Bounds output records reserved for one forge step.
- `forge.facts`: count, 16. Bounds facts awaiting the forge connector's drain.
- `forge.makings`: count, 32. Bounds repositories being made or failed, from 1 to 64.
- `forge.adoptions`: count, 2. Bounds repository adoptions in flight.
- `forge.applications`: count, 32. Bounds keyed adoption answers retained for request retention.
- `forge.collaborators`: count, 8. Bounds collaborators retained by the forge connector.
- `forge.changes_rows`: count, 32. Bounds queued rows across the changes document's queues.
- `forge.changes_landed`: count, 8. Bounds recent landed changes in the changes document.
- `forge.landings_kept`: count, 16. Bounds retained landings per repository.
- `forge.landings`: count, 2. Bounds live landing procedure rows.
- `forge.history_days`: count, 14. Days a change's history stays after its last event, from 1 to 3,650.
- `forge.history_events`: count, 131,072. Events retained per project, at least 1; oldest last-event changes swept whole past it.
- `forge.findings_per_project`: count, 256. Bounds retained findings per project.
- `forge.decided_per_project`: count, 1,024. Bounds retained decided and asked rows per project.
- `forge.judge_projects`: count, 2. Bounds project judge tables retained beside committed policy.
- `forge.judge_criteria`: count, 8. Bounds criteria in each deployment or project judge table.
- `forge.dependencies`: count, 4. Dependencies per project.
- `forge.follows`: count, 2. Follows per project.
- `forge.follow_paths`: count, 8. Path prefixes per follow.
- `forge.follow_queue`: count, 16. Queued landings per follow.
- `forge.follow_merge`: count, 8. Landings per follow change.
- `forge.follow_commits`: count, 8. Commits retained per follow entry.
- `forge.follow_files`: count, 32. Files retained per follow entry.
- `forge.follow_message_bytes`: bytes, 512. Message bytes retained per follow commit.
- `forge.follow_skips`: count, 8. Skips retained per follow.
- `forge.follow_reason_bytes`: bytes, 256. Bytes per follow skip reason.
- `forge.landing_rules`: count, 4. Bounds landing rules per scope.
- `forge.brief_sections`: count, 40. Bounds forge brief sections gathered at once.
- `forge.brief_bytes`: bytes, 4,608. Bounds bytes retained for one gathered forge brief section.
- `forge.tiers.tiers`: count, 2. Bounds tiers retained per repository.
- `forge.tiers.failing_tests`: count, 16. Bounds failing tests reported per tier run.
- `forge.tiers.test_id_bytes`: bytes, 256. Bounds a failing test's identifier in a tier's report.
- `forge.sitting_ms`: milliseconds, 1,800,000. Sets the gap between a person's sittings on Home.
- `forge.queue_window_ms`: milliseconds, 30,000. Sets the time allowed to gather a landing queue.
- `forge.client.pending`: count, 4. Bounds logical forge calls retained, queued and in flight.
- `forge.client.calls`: count, 2. Bounds forge calls in flight.
- `forge.client.rate`: count, 100. Bounds requests allowed in one forge rate window.
- `forge.client.reserve`: count, 0. Reserves requests in the forge rate window for writes.
- `forge.client.window_ms`: milliseconds, 10,000. Sets the forge request rate window.
- `forge.client.op_bytes`: bytes, 512. Bounds owned bytes of one forge operation.
- `forge.client.answer_bytes`: bytes, 65,536. Bounds decoded bytes of one forge answer.
- `forge.client.rows`: count, 8. Bounds rows in one forge answer page.
- `forge.client.inbox`: count, 16. Bounds comment and review delivery stamps retained per resource.
- `forge.client.read_attempts`: count, 3. Bounds attempts of one forge read.
- `forge.client.backoff_ms`: milliseconds, 1,000. Sets the first delay before a forge read is tried again.
- `forge.client.backoff_max_ms`: milliseconds, 4,000. Bounds the delay before a forge read is tried again.
- `forge.client.entries`: count, 8. Bounds durable outbox entries in the forge client.
- `forge.client.write_attempts`: count, 3. Bounds attempts of one forge write.
- `forge.client.lifetime_ms`: milliseconds, 10,000. Bounds how long a sent forge write may arrive.
- `forge.client.clock_margin_ms`: milliseconds, 1,000. Sets the allowance for clock movement when a forge write's lifetime is checked.
- `forge.client.resources`: count, 16. Bounds resources retained for forge polling.
- `forge.client.repositories`: count, 2. Bounds repositories retained by the forge client and read by a setup plan.
- `forge.client.poll_ms`: milliseconds, 5,000. Sets the normal interval between forge resource polls.
- `forge.client.poll_max_ms`: milliseconds, 30,000. Bounds the interval between forge resource polls.
- `forge.client.hinted_ms`: milliseconds, 1,000. Sets the delay before a hinted forge read.
- `forge.client.slow_ms`: milliseconds, 40,000. Sets the delay before a slow forge read.
- `forge.client.settle_ms`: milliseconds, 30,000. Bounds settling after a pull request's head or base moves, while hinted CI and mergeability are read again.
- `forge.client.fallback_ms`: milliseconds, 300,000. Timed reads fall back while repository deliveries arrive; 0 keeps the polling paces.
- `forge.client.facts`: count, 16. Bounds forge client facts awaiting drain.
- `forge.change_policy.repairs`: count, 3. Bounds repair attempts in a change.
- `forge.change_policy.resolutions`: count, 3. Bounds conflict resolution attempts in a change.
- `forge.change_policy.reruns`: count, 1. Bounds re-runs of a runner-failed job at one head.
- `forge.change_policy.updates`: count, 3. Bounds update attempts in a change.
- `forge.change_policy.questions`: count, 4. Bounds questions raised in a change.
- `forge.change_policy.tasks`: count, 6. Bounds the producer, reviews and repairs a change may create.
- `forge.change_policy.stall_ms`: milliseconds, 1,800,000. Bounds the wait for a silent forge during a change.
- `forge.change_policy.gates`: count, 8. Bounds gates retained by a change.
- `forge.change_policy.clean_heads`: count, 8. Bounds clean branch heads retained by a change.
- `forge.issue_policy.plan_items`: count, 8. Bounds items in an issue plan.
- `forge.issue_policy.milestones`: count, 8. Bounds milestones in an issue projection.
- `forge.issue_policy.title_bytes`: bytes, 128. Bounds an issue title.
- `forge.issue_policy.body_bytes`: bytes, 512. Bounds an issue body.
- `forge.issue_policy.comment_bytes`: bytes, 256. Bounds an issue comment.
- `forge.issue_policy.interval_ms`: milliseconds, 5,000. Sets the interval between issue projection checks.

## checkout

- `checkout.workspaces`: count, 3. Bounds workspaces cached on disk and held at once; at least the slots; preparation refuses when all are held.
- `checkout.preparations`: count, 1. Bounds workspace preparations active at once on a worker, runs' and jobs' alike; the rest wait in order of arrival.
- `checkout.repositories`: count, 2. Bounds repositories named by one checkout specification.
- `checkout.name_bytes`: bytes, 256. Bounds a workstream key, repository name, remote or branch.
- `checkout.message_bytes`: bytes, 4,096. Bounds a commit message's title and body together.
- `checkout.conflicts`: count, 512. Bounds conflicted paths per repository; zero disables merge starts.
- `checkout.path_bytes`: bytes, 255. Bounds one full relative conflicted path.
- `checkout.remote_timeout_ms`: milliseconds, 60,000. Bounds a checkout operation that reaches the forge, including cloning, fetching and pushing.
- `checkout.local_timeout_ms`: milliseconds, 10,000. Bounds an operation on the worker's disk, including checking out, merging and committing.
- `checkout.facts`: count, 256. Bounds facts kept until the parent drains them; later facts are dropped and counted.

## git_component

- `git_component.operations`: count, 3. Bounds simultaneous git component operations; at least the workspaces.
- `git_component.stdout_bytes`: bytes, 131,072. Bounds standard output retained from a git operation.
- `git_component.stderr_bytes`: bytes, 512. Bounds standard error retained from a git operation.
- `git_component.read_bytes`: bytes, 4,096. Bounds one read from a git operation's output.
- `git_component.sweep_entries`: count, 16. Bounds entries in one page of the checkout trash sweep's listing.
- `git_component.sweep_bytes`: bytes, 4,080. Bounds bytes of names in one page of the checkout trash sweep's listing.

## clients

- `clients.bundle_files`: count, 128. Bounds bundle files, including the browser's page.
- `clients.bundle_bytes`: bytes, 33,554,432. Bounds all bundle file bytes held in memory while the service runs.

## records

- `records.value_bytes`: bytes, 4,096. Bounds one forge record or carried value of temper's in the secrets store.

## forges

- `forges.installations`: count, 8. Bounds installations accepted by one GitHub forge.
- `forges.log_origins`: count, 20. Bounds one forge's job log origins, at most 32.

## forge_component

- `forge_component.log_origins`: count, 20. Bounds job log redirect origins and remembered origins per forge.
- `forge_component.log_poll_ms`: milliseconds, 5,000. Sets the interval between reads of a GitHub App's delivery log.
- `forge_component.log_catch_up_ms`: milliseconds, 86,400,000. Bounds how far back delivery logs are read when a forge restarts within one process, at most a day.
- `forge_component.log_pages`: count, 10. Bounds pages taken by one delivery log read.
- `forge_component.delivery_bytes`: bytes, 1,048,576. Bounds a delivery read from the log.
- `forge_component.installations`: count, 8. Bounds installations for which the component holds tokens, per GitHub forge.
- `forge_component.key_bytes`: bytes, 1,196. Bounds a GitHub App's key.
- `forge_component.renew_before_ms`: milliseconds, 600,000. Sets how long before expiry an installation token is minted again.
- `forge_component.forges`: count, 3. Bounds forges served by the forge component.
- `forge_component.token_bytes`: bytes, 512. Bounds a forge's API token, git token or site admin token.
- `forge_component.connections`: count, 2. Bounds forge connections, equal to the forge client's calls in flight.
- `forge_component.head_bytes`: bytes, 16,384. Bounds a response head read from a forge.
- `forge_component.headers`: count, 64. Bounds fields in a forge response head.
- `forge_component.body_bytes_in`: bytes, 16,777,216. Bounds a streamed response body read from a forge, counting time and bytes rather than held memory.
- `forge_component.depth`: count, 16. Bounds JSON nesting followed in a forge response.
- `forge_component.chunk`: bytes, 4,096. Bounds bytes of a JSON string demanded at once.
- `forge_component.string_bytes`: bytes, 512. Bounds a retained forge name, title, context, description, link or path.
- `forge_component.body_bytes`: bytes, 16,384. Bounds a forge response body retained whole.
- `forge_component.inner`: count, 32. Bounds elements retained from a list within one forge document.
- `forge_component.target_bytes`: bytes, 4,096. Bounds a forge request target.
- `forge_component.repositories`: count, 2. Bounds repositories in the forge component's table.
- `forge_component.users`: count, 256. Bounds users in the forge component's table.
- `forge_component.tls.read`: bytes, 16,384. Bounds plaintext demanded at once from a forge TLS connection.
- `forge_component.tls.send`: bytes, 16,384. Bounds plaintext send room demanded from a forge TLS connection.
- `forge_component.tls.records`: bytes, 18,437. Bounds ciphertext retained by a forge TLS connection, including handshake messages spanning records.
- `forge_component.hooks`: count, 4. Bounds connections to the webhooks listener.
- `forge_component.hook_head_bytes`: bytes, 8,192. Bounds a webhook delivery's request head.
- `forge_component.hook_body_bytes`: bytes, 1,048,576. Bounds a streamed webhook body; a delivery past it gets 413 and hints nothing.
- `forge_component.hook_secret_bytes`: bytes, 128. Bounds a Forgejo forge's webhook secret, at least 32 bytes.
- `forge_component.hook_deadline_ms`: milliseconds, 10,000. Bounds a webhook delivery's read and answer from accept or the previous answer.
- `forge_component.deadline_ms`: milliseconds, 10,000. Bounds a forge call and each startup read, at most the forge client's lifetime.
- `forge_component.idle_ms`: milliseconds, 4,000. Bounds how long an idle forge connection is kept.
- `forge_component.start_backoff_ms`: milliseconds, 1,000. Sets the first delay before a forge's start is tried again.
- `forge_component.start_backoff_max_ms`: milliseconds, 60,000. Bounds the doubling delay before a forge's start is tried again.

## setup

- `setup.deadline_ms`: milliseconds, 120,000. Bounds a setup plan's run, including its wait for the forge.
- `setup.backoff_ms`: milliseconds, 100. Sets the first delay before a setup request is tried again.
- `setup.backoff_max_ms`: milliseconds, 5,000. Bounds the doubling delay before a setup request is tried again.
- `setup.runs`: count, 8. Bounds simultaneous setup host runs, counting a writing run and a reachable read separately.
- `setup.states`: count, 8. Bounds states held for manifest and installation returns.
- `setup.state_life_ms`: milliseconds, 3,600,000. Bounds a return state's life, at most the manifest conversion code's hour.
- `setup.fetch_bytes`: bytes, 268,435,456. Bounds a fetched binary's streamed body, written to its part file without holding it in memory.

## box

- `box.backoff_ms`: milliseconds, 500. Sets the first delay before restarting a long step.
- `box.backoff_max_ms`: milliseconds, 30,000. Bounds the doubling delay before restarting a long step.
- `box.restarts`: count, 5. Bounds a long step's exits within the restart window before the box gives up.
- `box.window_ms`: milliseconds, 300,000. Sets the window in which a long step's exits count toward its restart limit.
- `box.grace_ms`: milliseconds, 10,000. Bounds the grace after each of the two stop signals sent to a long step.
- `box.line_bytes`: bytes, 256. Bounds a box pipe line in either direction, including its line feed.
- `box.log_bytes`: bytes, 4,194,304. Bounds a step's log file before it moves aside and a fresh log starts.
- `box.tail_lines`: count, 10. Bounds retained log lines printed after a step's failure.
- `box.groups`: count, 8. Bounds group files the box lists and runs at a later start.
