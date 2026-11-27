# Limits

Kind: reference

Every row belongs to the temper-limits setting: its name, unit, stock value, when a change applies and what it bounds.

## live_goals

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `live_goals` | count | 40 | next start | Bounds goals live at once; work-in-flight counts scale with it. |

## tool_payload

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `tool_payload` | bytes | 63,740 | next start | Bounds the largest piece a tool writes; payload carriers follow it. |

## transcript_memory

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `transcript_memory` | bytes | 1,073,741,824 | at once | Bounds transcripts and turns held while work prepares, not the process's memory. |

## sign_in

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `sign_in.members` | count | 64 | next start | Bounds configured members and the forge connector's retained member bindings. |
| `sign_in.pending` | count | 64 | next start | Bounds simultaneous sign-ins; a start past this count gets the busy page. |
| `sign_in.deadline_ms` | milliseconds | 600,000 | next start | Bounds a sign-in's life from start through its return's answer and sets its cookie's lifetime. |
| `sign_in.fetches` | count | 8 | next start | Bounds simultaneous exchanges with Google for sign-ins. |
| `sign_in.body_bytes` | bytes | 65,536 | next start | Bounds an answer body from Google, for a key set or a token. |
| `sign_in.url_bytes` | bytes | 2,048 | next start | Bounds a configured endpoint and the authorization URL given to the browser. |
| `sign_in.secret_bytes` | bytes | 512 | next start | Bounds the Google client secret. |

## picker

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `picker.page` | count | 30 | next start | Bounds repositories taken by one picker read, within the forge's page cap of at most 100. |

## routes

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `routes.query_bytes` | bytes | 2,048 | next start | Bounds a forge route's query. |

## root

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `root.journal.commits` | count | 3 | next start | Bounds commits awaiting the store's answer. |
| `root.journal.held` | count | 1,536 | next start | Sets the retained held-delivery count; the uncapped journal does not use it for admission. |
| `root.journal.writes` | count | 12,288 | next start | Bounds unique writes per commit, including the saved deployment header; a decision has one fewer slot. |
| `root.journal.deliveries` | count | 512 | next start | Sets the door output count used in the default admission watermark, without capping a decision. |
| `root.journal.transcript_bytes` | bytes | 18,431,846 | next start | Bounds transcript bytes or a wrapped child row's owned bytes admitted to the journal. |
| `root.journal.result_bytes` | bytes | 317,786 | next start | Bounds owned result text or assignment section text admitted to a held delivery. |
| `root.journal.run_bytes` | bytes | 65,536 | next start | Bounds a run charter's owned bytes in a held assignment. |
| `root.loads.loads` | count | 2 | next start | Bounds issued store loads, including abandoned reads awaiting their terminal answer. |
| `root.loads.rows` | count | 2 | next start | Bounds rows requested and accepted in one incoming store page. |
| `root.loads.bytes` | bytes | 18,432,106 | next start | Bounds a kept page prefix, counting record slots and their owned bytes. |
| `root.loads.read_bytes` | bytes | 36,863,684 | next start | Bounds owned bytes one authenticated store read brings for a person. |
| `root.loads.reply_bytes` | bytes | 18,432,106 | next start | Bounds a decoded store answer before its kept prefix is cut. |
| `root.loads.transcript_bytes` | bytes | 18,431,842 | next start | Bounds a row's owned bytes, including transcripts and wrapped child rows, before the prefix is cut. |
| `root.brief.briefs` | count | 2 | next start | Bounds briefs the root gathers at once. |
| `root.brief.sections` | count | 10 | next start | Bounds brief sections the root gathers at once. |
| `root.brief.parts` | count | 2 | next start | Bounds parts in one gathered brief section. |
| `root.brief.read_bytes` | bytes | 131,584 | next start | Bounds bytes read for one gathered brief section. |
| `root.brief.brief_bytes` | bytes | 317,786 | next start | Bounds the body of a gathered brief. |
| `root.brief.gather_ms` | milliseconds | 300,000 | next start | Bounds the time spent gathering a brief. |
| `root.brief.budgets.task` | bytes | 131,584 | next start | Bounds task section bytes in a gathered brief. |
| `root.brief.budgets.dependencies` | bytes | 128 | next start | Bounds dependency section bytes in a gathered brief. |
| `root.brief.budgets.document` | bytes | 4,096 | next start | Bounds document section bytes in a gathered brief. |
| `root.brief.budgets.ci` | bytes | 4,096 | next start | Bounds CI section bytes in a gathered brief. |
| `root.brief.budgets.reviews` | bytes | 131,584 | next start | Bounds review section bytes in a gathered brief. |
| `root.brief.budgets.pull` | bytes | 131,584 | next start | Bounds pull request section bytes in a gathered brief. |
| `root.brief.budgets.attempts` | bytes | 128 | next start | Bounds attempt section bytes in a gathered brief. |
| `root.brief.budgets.plan` | bytes | 128 | next start | Bounds plan section bytes in a gathered brief. |
| `root.brief.budgets.notes` | bytes | 64 | next start | Bounds note section bytes in a gathered brief. |
| `root.brief.budgets.preferences` | bytes | 2,048 | next start | Bounds whole requester preferences in a gathered brief. |
| `root.brief.budgets.now` | bytes | 4,096 | next start | Bounds a standing task's Now body in a gathered brief. |

## forge

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `forge.forges` | count | 3 | next start | Bounds retained forges of every kind and mode, counting an ended forge until its removal is whole. |
| `forge.repositories` | count | 6 | next start | Bounds repositories retained by the forge connector, shared by every retained forge. |
| `forge.tasks` | count | 128 | at once | Bounds tasks participating in the forge connector. |
| `forge.holds` | count | 160 | at once | Bounds held task resources in the forge connector. |
| `forge.subscriptions` | count | 128 | next start | Bounds subscriptions to forge topics. |
| `forge.entries` | count | 8 | at once | Bounds durable outbox entries in the forge connector. |
| `forge.named_calls` | count | 128 | next start | Bounds retained named calls and their connector payloads, independently of outbox entries. |
| `forge.changes` | count | 768 | next start | Bounds live changes retained by the forge connector. |
| `forge.issues` | count | 8 | next start | Bounds live issue projections retained by the forge connector. |
| `forge.landing_members` | count | 5 | next start | Bounds changes and held member branches in one landing; at least each project's batch. |
| `forge.resources_per_task` | count | 10 | next start | Bounds distinct forge names watched by one task, including held resources and subscribed branches. |
| `forge.paths_per_subscription` | count | 4 | next start | Bounds paths retained by one forge subscription. |
| `forge.name_bytes` | bytes | 64 | next start | Bounds names retained by the forge connector. |
| `forge.output` | count | 192 | next start | Bounds output records reserved for one forge step. |
| `forge.facts` | count | 16 | next start | Bounds facts awaiting the forge connector's drain. |
| `forge.makings` | count | 32 | next start | Bounds repositories being made or failed, from 1 to 64. |
| `forge.adoptions` | count | 2 | next start | Bounds repository adoptions in flight. |
| `forge.applications` | count | 32 | next start | Bounds the forge's kept answers to keyed requests; the oldest answered makes room for a new one. |
| `forge.collaborators` | count | 8 | next start | Bounds collaborators retained by the forge connector. |
| `forge.changes_rows` | count | 32 | next start | Bounds queued rows across the changes document's queues. |
| `forge.changes_landed` | count | 8 | next start | Bounds recent landed changes in the changes document. |
| `forge.landings_kept` | count | 16 | next start | Bounds retained landings per repository. |
| `forge.landings_pending` | count | 64 | next start | Bounds landings kept per repository until every tier following their branch has judged them. |
| `forge.landings` | count | 32 | next start | Bounds live landing procedure rows. |
| `forge.key_entries` | count | 256 | next start | Bounds the entries of one tree a landing's gate key reads. |
| `forge.branch_reds` | count | 64 | next start | Bounds the reds held by a goal. |
| `forge.red_key_bytes` | bytes | 256 | next start | Bounds a goal red's key. |
| `forge.history_days` | count | 14 | next start | Days a change's history stays after its last event, from 1 to 3,650. |
| `forge.history_events` | count | 131,072 | next start | Events retained per project, at least 1; oldest last-event changes swept whole past it. |
| `forge.findings_per_project` | count | 256 | next start | Bounds retained findings per project. |
| `forge.decided_per_project` | count | 1,024 | next start | Bounds retained decided and asked rows per project. |
| `forge.judge_projects` | count | 16 | next start | Bounds project judge tables retained beside committed policy. |
| `forge.judge_criteria` | count | 8 | next start | Bounds criteria in each deployment or project judge table. |
| `forge.dependencies` | count | 16 | next start | Dependencies per project. |
| `forge.follows` | count | 8 | next start | Follows per project. |
| `forge.follow_paths` | count | 8 | next start | Path prefixes per follow. |
| `forge.follow_queue` | count | 64 | next start | Queued landings per follow. |
| `forge.follow_merge` | count | 8 | next start | Landings per follow change. |
| `forge.follow_commits` | count | 8 | next start | Commits retained per follow entry. |
| `forge.follow_files` | count | 32 | next start | Files retained per follow entry. |
| `forge.follow_message_bytes` | bytes | 512 | next start | Message bytes retained per follow commit. |
| `forge.follow_skips` | count | 8 | next start | Skips retained per follow. |
| `forge.follow_reason_bytes` | bytes | 256 | next start | Bytes per follow skip reason. |
| `forge.landing_rules` | count | 4 | next start | Bounds landing rules per scope. |
| `forge.brief_sections` | count | 40 | next start | Bounds forge brief sections gathered at once. |
| `forge.brief_bytes` | bytes | 131,584 | next start | Bounds bytes retained for one gathered forge brief section. |
| `forge.tiers.tiers` | count | 2 | next start | Bounds tiers retained per repository. |
| `forge.tiers.failing_tests` | count | 64 | next start | Bounds failing tests reported per tier run. |
| `forge.tiers.test_id_bytes` | bytes | 512 | next start | Bounds a failing test's identifier in a tier's report. |
| `forge.sitting_ms` | milliseconds | 1,800,000 | next start | Sets the gap between a person's sittings on Home. |
| `forge.queue_window_ms` | milliseconds | 30,000 | next start | Sets the time allowed to gather a landing queue. |
| `forge.client.pending` | count | 4 | next start | Bounds logical forge calls retained, queued and in flight. |
| `forge.client.calls` | count | 2 | next start | Bounds forge calls in flight. |
| `forge.client.rate` | count | 100 | next start | Bounds requests allowed in one forge rate window. |
| `forge.client.reserve` | count | 0 | next start | Reserves requests in the forge rate window for writes. |
| `forge.client.window_ms` | milliseconds | 10,000 | next start | Sets the forge request rate window. |
| `forge.client.op_bytes` | bytes | 63,738 | next start | Bounds owned bytes of one forge operation. |
| `forge.client.answer_bytes` | bytes | 1,048,576 | next start | Bounds decoded bytes of one forge answer. |
| `forge.client.rows` | count | 8 | next start | Bounds rows in one forge answer page. |
| `forge.client.inbox` | count | 16 | next start | Bounds comment and review delivery stamps retained per resource. |
| `forge.client.read_attempts` | count | 3 | next start | Bounds attempts of one forge read. |
| `forge.client.backoff_ms` | milliseconds | 1,000 | next start | Sets the first delay before a forge read is tried again. |
| `forge.client.backoff_max_ms` | milliseconds | 4,000 | next start | Bounds the delay before a forge read is tried again. |
| `forge.client.entries` | count | 8 | at once | Bounds durable outbox entries in the forge client. |
| `forge.client.write_attempts` | count | 3 | next start | Bounds attempts of one forge write. |
| `forge.client.lifetime_ms` | milliseconds | 300,000 | next start | Bounds how long a sent forge write may arrive; moves a still-derived call deadline with it. |
| `forge.client.clock_margin_ms` | milliseconds | 1,000 | next start | Sets the allowance for clock movement when a forge write's lifetime is checked. |
| `forge.client.resources` | count | 2,048 | next start | Bounds resources retained for forge polling. |
| `forge.client.repositories` | count | 6 | next start | Bounds repositories retained by the forge client and read by a setup plan. |
| `forge.client.poll_ms` | milliseconds | 5,000 | next start | Sets the normal interval between forge resource polls. |
| `forge.client.poll_max_ms` | milliseconds | 30,000 | next start | Bounds the interval between forge resource polls. |
| `forge.client.hinted_ms` | milliseconds | 1,000 | next start | Sets the delay before a hinted forge read. |
| `forge.client.slow_ms` | milliseconds | 40,000 | next start | Sets the delay before a slow forge read. |
| `forge.client.settle_ms` | milliseconds | 30,000 | next start | Bounds settling after a pull request's head or base moves, while hinted CI and mergeability are read again. |
| `forge.client.fallback_ms` | milliseconds | 300,000 | next start | Timed reads fall back while repository deliveries arrive; 0 keeps the polling paces. |
| `forge.client.facts` | count | 16 | next start | Bounds forge client facts awaiting drain. |
| `forge.change_policy.repairs` | count | 80 | next start | Bounds repair attempts in a change. |
| `forge.change_policy.resolutions` | count | 80 | next start | Bounds conflict resolution attempts in a change. |
| `forge.change_policy.reruns` | count | 1 | next start | Bounds re-runs of a runner-failed job at one head. |
| `forge.change_policy.updates` | count | 80 | next start | Bounds update attempts in a change. |
| `forge.change_policy.questions` | count | 80 | next start | Bounds questions raised in a change. |
| `forge.change_policy.tasks` | count | 80 | next start | Bounds the producer, reviews and repairs a change may create. |
| `forge.change_policy.stall_ms` | milliseconds | 1,800,000 | next start | Bounds the wait for a silent forge during a change. |
| `forge.change_policy.unknown_grace_ms` | milliseconds | 1,800,000 | next start | Bounds the grace for CI that says nothing of its execution at checking. |
| `forge.change_policy.runner_wait_ms` | milliseconds | 14,400,000 | next start | Bounds the wait for a runner at checking. |
| `forge.change_policy.gates` | count | 8 | next start | Bounds gates retained by a change. |
| `forge.change_policy.clean_heads` | count | 8 | next start | Bounds clean branch heads retained by a change. |
| `forge.issue_policy.plan_items` | count | 384 | next start | Bounds items in an issue plan. |
| `forge.issue_policy.milestones` | count | 8 | next start | Bounds milestones in an issue projection. |
| `forge.issue_policy.title_bytes` | bytes | 128 | next start | Bounds an issue title. |
| `forge.issue_policy.body_bytes` | bytes | 512 | next start | Bounds an issue body. |
| `forge.issue_policy.comment_bytes` | bytes | 256 | next start | Bounds an issue comment. |
| `forge.issue_policy.interval_ms` | milliseconds | 5,000 | next start | Sets the interval between issue projection checks. |

## checkout

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `checkout.workspaces` | count | 3 | next start | Bounds workspaces cached on disk and held at once; at least the slots; preparation refuses when all are held. |
| `checkout.preparations` | count | 1 | live | Bounds workspace preparations active at once on a worker, runs' and jobs' alike; the rest wait in order of arrival. |
| `checkout.repositories` | count | 2 | next start | Bounds repositories named by one checkout specification. |
| `checkout.name_bytes` | bytes | 256 | next start | Bounds a workstream key, repository name, remote or branch. |
| `checkout.message_bytes` | bytes | 63,740 | live | Bounds a commit message's title and body together. |
| `checkout.conflicts` | count | 512 | next start | Bounds conflicted paths per repository; zero disables merge starts. |
| `checkout.path_bytes` | bytes | 255 | next start | Bounds one full relative conflicted path. |
| `checkout.remote_timeout_ms` | milliseconds | 60,000 | live | Bounds a checkout operation that reaches the forge, including cloning, fetching and pushing. |
| `checkout.local_timeout_ms` | milliseconds | 60,000 | live | Bounds an operation on the worker's disk, including checking out, merging and committing. |
| `checkout.reds_read_ms` | milliseconds | 5,000 | live | Waits for the goal’s reds at each check; 100 to 60,000 milliseconds. |
| `checkout.rerun_processes` | count | 4 | live | Bounds failing test processes run again by a check; 0 to 16. |
| `checkout.facts` | count | 256 | next start | Bounds facts kept until the parent drains them; later facts are dropped and counted. |

## git_component

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `git_component.operations` | count | 3 | next start | Bounds simultaneous git component operations; at least the workspaces. |
| `git_component.stdout_bytes` | bytes | 131,072 | live | Bounds standard output retained from a git operation. |
| `git_component.stderr_bytes` | bytes | 512 | live | Bounds standard error retained from a git operation. |
| `git_component.read_bytes` | bytes | 4,096 | live | Bounds one read from a git operation's output. |
| `git_component.sweep_entries` | count | 16 | live | Bounds entries in one page of the checkout trash sweep's listing. |
| `git_component.sweep_bytes` | bytes | 4,080 | live | Bounds bytes of names in one page of the checkout trash sweep's listing. |

## clients

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `clients.bundle_files` | count | 128 | next start | Bounds bundle files, including the browser's page. |
| `clients.bundle_bytes` | bytes | 33,554,432 | next start | Bounds all bundle file bytes held in memory while the service runs. |

## records

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `records.value_bytes` | bytes | 4,096 | next start | Bounds one forge record or carried value of temper's in the secrets store. |

## forges

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `forges.installations` | count | 8 | next start | Bounds installations accepted by one GitHub forge. |
| `forges.log_origins` | count | 20 | next start | Bounds one forge's job log origins, at most 32. |

## forge_component

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `forge_component.log_origins` | count | 20 | next start | Bounds job log redirect origins and remembered origins per forge. |
| `forge_component.log_poll_ms` | milliseconds | 5,000 | next start | Sets the interval between reads of a GitHub App's delivery log. |
| `forge_component.log_catch_up_ms` | milliseconds | 86,400,000 | next start | Bounds how far back delivery logs are read when a forge restarts within one process, at most a day. |
| `forge_component.log_pages` | count | 10 | next start | Bounds pages taken by one delivery log read. |
| `forge_component.delivery_bytes` | bytes | 1,048,576 | next start | Bounds a delivery read from the log. |
| `forge_component.installations` | count | 8 | next start | Bounds installations for which the component holds tokens, per GitHub forge. |
| `forge_component.key_bytes` | bytes | 1,196 | next start | Bounds a GitHub App's key. |
| `forge_component.renew_before_ms` | milliseconds | 600,000 | next start | Sets how long before expiry an installation token is minted again. |
| `forge_component.forges` | count | 3 | next start | Bounds forges served by the forge component. |
| `forge_component.token_bytes` | bytes | 512 | next start | Bounds a forge's API token, git token or site admin token. |
| `forge_component.connections` | count | 2 | next start | Bounds forge connections, equal to the forge client's calls in flight. |
| `forge_component.head_bytes` | bytes | 16,384 | next start | Bounds a response head read from a forge. |
| `forge_component.headers` | count | 64 | next start | Bounds fields in a forge response head. |
| `forge_component.body_bytes_in` | bytes | 16,777,216 | next start | Bounds a streamed response body read from a forge, counting time and bytes rather than held memory. |
| `forge_component.depth` | count | 16 | next start | Bounds JSON nesting followed in a forge response. |
| `forge_component.chunk` | bytes | 4,096 | next start | Bounds bytes of a JSON string demanded at once. |
| `forge_component.string_bytes` | bytes | 512 | next start | Bounds a retained forge name, title, context, description, link or path. |
| `forge_component.body_bytes` | bytes | 127,497 | next start | Bounds a forge response body retained whole. |
| `forge_component.inner` | count | 32 | next start | Bounds elements retained from a list within one forge document. |
| `forge_component.target_bytes` | bytes | 4,096 | next start | Bounds a forge request target. |
| `forge_component.repositories` | count | 6 | next start | Bounds repositories in the forge component's table. |
| `forge_component.users` | count | 256 | next start | Bounds users in the forge component's table. |
| `forge_component.tls.read` | bytes | 16,384 | next start | Bounds plaintext demanded at once from a forge TLS connection. |
| `forge_component.tls.send` | bytes | 16,384 | next start | Bounds plaintext send room demanded from a forge TLS connection. |
| `forge_component.tls.records` | bytes | 18,437 | next start | Bounds ciphertext retained by a forge TLS connection, including handshake messages spanning records. |
| `forge_component.hooks` | count | 4 | next start | Bounds connections to the webhooks listener. |
| `forge_component.hook_head_bytes` | bytes | 8,192 | next start | Bounds a webhook delivery's request head. |
| `forge_component.hook_body_bytes` | bytes | 1,048,576 | next start | Bounds a streamed webhook body; a delivery past it gets 413 and hints nothing. |
| `forge_component.hook_secret_bytes` | bytes | 128 | next start | Bounds a Forgejo forge's webhook secret, at least 32 bytes. |
| `forge_component.hook_deadline_ms` | milliseconds | 10,000 | next start | Bounds a webhook delivery's read and answer from accept or the previous answer. |
| `forge_component.deadline_ms` | milliseconds | 270,000 | next start | Bounds calls and startup reads; lesser of lifetime and 270 s by default, follows lifetime overrides unless changed. |
| `forge_component.idle_ms` | milliseconds | 4,000 | next start | Bounds how long an idle forge connection is kept. |
| `forge_component.start_backoff_ms` | milliseconds | 1,000 | next start | Sets the first delay before a forge's start is tried again. |
| `forge_component.start_backoff_max_ms` | milliseconds | 60,000 | next start | Bounds the doubling delay before a forge's start is tried again. |

## setup

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `setup.deadline_ms` | milliseconds | 120,000 | next start | Bounds a setup plan's run, including its wait for the forge. |
| `setup.backoff_ms` | milliseconds | 100 | next start | Sets the first delay before a setup request is tried again. |
| `setup.backoff_max_ms` | milliseconds | 5,000 | next start | Bounds the doubling delay before a setup request is tried again. |
| `setup.runs` | count | 8 | next start | Bounds simultaneous setup host runs, counting a writing run and a reachable read separately. |
| `setup.states` | count | 8 | next start | Bounds states held for manifest and installation returns. |
| `setup.state_life_ms` | milliseconds | 3,600,000 | next start | Bounds a return state's life, at most the manifest conversion code's hour. |
| `setup.fetch_bytes` | bytes | 268,435,456 | next start | Bounds a fetched binary's streamed body, written to its part file without holding it in memory. |

## box

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `box.backoff_ms` | milliseconds | 500 | next start | Sets the first delay before restarting a long step. |
| `box.backoff_max_ms` | milliseconds | 30,000 | next start | Bounds the doubling delay before restarting a long step. |
| `box.restarts` | count | 5 | next start | Bounds a long step's exits within the restart window before the box gives up. |
| `box.window_ms` | milliseconds | 300,000 | next start | Sets the window in which a long step's exits count toward its restart limit. |
| `box.grace_ms` | milliseconds | 10,000 | next start | Bounds the grace after each of the two stop signals sent to a long step. |
| `box.line_bytes` | bytes | 256 | next start | Bounds a box pipe line in either direction, including its line feed. |
| `box.log_bytes` | bytes | 4,194,304 | next start | Bounds a step's log file before it moves aside and a fresh log starts. |
| `box.tail_lines` | count | 10 | next start | Bounds retained log lines printed after a step's failure. |
| `box.groups` | count | 8 | next start | Bounds group files the box lists and runs at a later start. |
