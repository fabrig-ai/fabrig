# Engine limits

Kind: reference

Every limit engine.limits.overrides and a worker's limits.overrides take: its unit, the stock's value and what it bounds.

Every row belongs to the limits setting. The table uses the stock, including temper's stock-agent carriers; overrides replace named values. The deployment's Limits page shows effective values.

The tree limit counts all tasks made over the requester's tree's life, including its root and ended tasks. Ending a task frees live slots, not lifetime capacity; a restart does not reset the count. A task's delegated authority may leave less room than this deployment limit.

A change batch reserves one task per member and shares the descendant allowance across members within each change policy's cap. These are prospective allowances for that call, not retained reservations across calls. A task-tree refusal names available and minimum required capacity and says whether fewer changes per call can fit (temper help change).

temper sizes the engine's watch rooms itself: a stream holds 31 watches, and the engine holds clients.streams times that, in all and for one party. Raise clients.streams for more.

## store

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `store.checkpoints.keep` | count | 2 | next start | Bounds the newest completed checkpoints kept by the store. |

## tasks

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `tasks.placed_holds` | count | 512 | next start | Bounds placed holds in the deployment, including lifted holds kept for their tasks. |
| `tasks.project_placed_holds` | count | 256 | next start | Bounds placed holds retained in one project. |
| `tasks.party_placed_holds` | count | 128 | next start | Bounds placed holds retained for one party owner. |
| `tasks.exemptions` | count | 768 | next start | Bounds tasks one standing placed hold may exempt. |
| `tasks.tasks` | count | 768 | next start | Bounds retained live task slots, name-index entries and retry alarms. |
| `tasks.funders` | count | 768 | next start | Bounds retained finite period, pool and recurring allotment identities. |
| `tasks.project_tasks` | count | 768 | next start | Bounds live tasks per project, including every member of an admitted batch. |
| `tasks.project_goals` | count | 80 | next start | Bounds live tracked tasks per project. |
| `tasks.tree_tasks` | count | 2,048 | next start | Bounds lifetime tasks made in a requester tree, including its root. |
| `tasks.depth` | count | 32 | next start | Bounds structural depth below the requester-tree root, which has depth zero. |
| `tasks.delegates` | count | 80 | next start | Bounds live direct delegates of one requester task. |
| `tasks.references` | count | 16 | next start | Bounds introduced peer references per live task. |
| `tasks.subscriptions` | count | 4 | next start | Bounds standing interests per task. |
| `tasks.batch` | count | 160 | next start | Bounds directly created tasks in one atomic nonempty `Make` batch. |
| `tasks.batch_bytes` | bytes | 4,194,304 | next start | Bounds the complete owned bytes of one task batch. |
| `tasks.dependencies` | count | 16 | next start | Bounds immutable dependency identities per task, including its remaining live waiting subset. |
| `tasks.circle` | count | 32 | next start | Bounds tasks a circle of waits names in a refusal or a hold. |
| `tasks.holdings` | count | 8 | next start | Bounds resources required by one task. |
| `tasks.hold_kinds` | count | 8 | next start | Bounds connector resource kinds installed in this deployment. |
| `tasks.resource_reports` | count | 256 | next start | Bounds cached named connector admission reports. |
| `tasks.pools` | count | 768 | next start | Bounds distinct pools whose latest connector slot counts are retained. |
| `tasks.hold_segments` | count | 8 | next start | Bounds literal segments in one resource name. |
| `tasks.hold_bytes` | bytes | 1,024 | next start | Bounds aggregate literal bytes in one resource name. |
| `tasks.hold_waiters` | count | 768 | next start | Bounds tasks waiting on any one resource. |
| `tasks.hold_wait_ms` | milliseconds | 120,000 | next start | Bounds the wait for holds before a task is held for review. |
| `tasks.grace_ms` | milliseconds | 600,000 | next start | Bounds the span a cancel may give its task to finish. |
| `tasks.inputs` | count | 16 | next start | Bounds retained input identities checked by the root before delegation. |
| `tasks.spec_bytes` | bytes | 131,072 | next start | Bounds combined specification words and byte-valued parameter bytes per task. |
| `tasks.parameters` | count | 8 | next start | Bounds parameters per specification. |
| `tasks.result_bytes` | bytes | 63,738 | next start | Bounds result or reason bytes, with one bounded reason and one bounded partial result retained on cancellation. |
| `tasks.run_inbox` | count | 82 | next start | Bounds messages a run starts with and holds relayed and unread in its inbox. |
| `tasks.inbox_messages` | count | 82 | next start | Bounds whole unread word messages kept by one live task. |
| `tasks.inbox_bytes` | bytes | 5,226,516 | next start | Bounds total unread word bytes kept by one live task. |
| `tasks.message_bytes` | bytes | 63,738 | next start | Bounds bytes in one admitted word message. |
| `tasks.person_stall_ms` | milliseconds | 3,600,000 | next start | Bounds time an active person task waits before becoming news for its requester. |
| `tasks.proposal_stall_ms` | milliseconds | 3,600,000 | next start | Bounds time an undecided proposal waits at a nonfinal holder before passing upward. |
| `tasks.escalation_stall_ms` | milliseconds | 3,600,000 | next start | Bounds time an undecided escalation waits at a nonfinal holder before passing upward. |
| `tasks.saved_resources` | count | 8 | next start | Bounds connector resources on which a task has saved state. |
| `tasks.contract_choices` | count | 4 | next start | Bounds distinct-code choices in a nonempty verdict contract. |
| `tasks.charters` | count | 48 | next start | Bounds retained configured agent-charter numbers. |
| `tasks.charter_parts` | count | 8 | next start | Bounds parts per charter. |
| `tasks.charter_documents` | count | 2 | next start | Bounds documents per charter. |
| `tasks.authority_grants` | count | 16 | next start | Bounds resource grants per task authority carrier. |
| `tasks.authority_segments` | count | 8 | next start | Bounds base segments per carried grant pattern. |
| `tasks.authority_bytes` | bytes | 4,096 | next start | Bounds total bytes across every grant's base and terminal segments in one authority carrier. |
| `tasks.executor_kinds` | count | 64 | next start | Bounds carried authority executor-permission entries. |
| `tasks.facts` | count | 128 | next start | Bounds content-free observation capacity. |
| `tasks.room_refusals` | count | 32 | next start | Bounds recent owner-proven room refusals. |
| `tasks.retries.transient.retries` | count | 3 | next start | Bounds failures allowed before holding a task for transient failures. |
| `tasks.retries.transient.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for transient failures. |
| `tasks.retries.transient.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for transient failures. |
| `tasks.retries.permanent.retries` | count | 3 | next start | Bounds failures allowed before holding a task for permanent failures. |
| `tasks.retries.permanent.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for permanent failures. |
| `tasks.retries.permanent.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for permanent failures. |
| `tasks.retries.run.retries` | count | 3 | next start | Bounds failures allowed before holding a task for run failures. |
| `tasks.retries.run.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for run failures. |
| `tasks.retries.run.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for run failures. |
| `tasks.retries.agent.retries` | count | 3 | next start | Bounds failures allowed before holding a task for agent failures. |
| `tasks.retries.agent.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for agent failures. |
| `tasks.retries.agent.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for agent failures. |
| `tasks.retries.lost.retries` | count | 3 | next start | Bounds failures allowed before holding a task for lost failures. |
| `tasks.retries.lost.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for lost failures. |
| `tasks.retries.lost.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for lost failures. |
| `tasks.retries.invalid.retries` | count | 3 | next start | Bounds failures allowed before holding a task for invalid failures. |
| `tasks.retries.invalid.base_ms` | milliseconds | 1,000 | next start | Bounds the initial exponential-backoff delay for invalid failures. |
| `tasks.retries.invalid.max_ms` | milliseconds | 30,000 | next start | Bounds the exponential-backoff delay for invalid failures. |
| `tasks.retries.infrastructure.retries` | count | 12 | next start | Bounds failures allowed before holding a task for infrastructure failures. |
| `tasks.retries.infrastructure.base_ms` | milliseconds | 10,000 | next start | Bounds the initial exponential-backoff delay for infrastructure failures. |
| `tasks.retries.infrastructure.max_ms` | milliseconds | 300,000 | next start | Bounds the exponential-backoff delay for infrastructure failures. |

## people

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `people.program_hosts` | count | 65 | next start | Bounds configured workers plus the engine host in one retained program outcome. |
| `people.program_host_name_bytes` | bytes | 32 | next start | Bounds whole configured worker name bytes in retained operation answers. |
| `people.operations` | count | 81 | next start | Bounds task operations in one keyed compound request. |
| `people.batch` | count | 4 | next start | Bounds members of one requested or proposed batch. |
| `people.people` | count | 16 | next start | Bounds retained parties and provider/subject index entries. |
| `people.inbox_entries` | count | 8 | next start | Bounds visible cached entries and unread count. |
| `people.inbox_tasks` | count | 1,536 | next start | Bounds live task and person-proposal census rows, including per-conversation positions until forgotten. |
| `people.inbox_recipients` | count | 176 | next start | Bounds recipient references in one task projection, independent of cached entries. |
| `people.sign_ins` | count | 32 | next start | Bounds live secret-free sign-ins and expiry alarms. |
| `people.projects` | count | 9 | next start | Bounds retained project role sets. |
| `people.holdings` | count | 32 | next start | Bounds unique-person holdings per project role set. |
| `people.initial_owners` | count | 4 | next start | Bounds retained configured first-owner entries. |
| `people.requests` | count | 8,000 | next start | Bounds retained completed keys plus ordinary flights' answer reservations across the engine. |
| `people.room_refusals` | count | 32 | next start | Bounds unsaved entrance-pressure records independently of diagnostics. |
| `people.pending` | count | 32 | next start | Bounds simultaneous keyed flights, including answered moves waiting for Written. |
| `people.waiters` | count | 4 | next start | Bounds reply destinations per flight, including its first caller. |
| `people.identity_bytes` | bytes | 1,024 | next start | Bounds combined subject, login and display-name bytes per party. |
| `people.words` | bytes | 8,192 | next start | Bounds opening words, escalation rejection and settings-proposal reason bytes, including pending and completed copies. |
| `people.amendment_bytes` | bytes | 135,168 | next start | Bounds owned bytes carried by one keyed task amendment. |
| `people.sign_in_lifetime_ms` | milliseconds | 86,400,000 | next start | Bounds the lifetime of a person sign-in. |
| `people.request_retention_ms` | milliseconds | 86,400,000 | next start | Bounds time for which completed person keys replay their durable answers. |
| `people.facts` | count | 128 | next start | Bounds capacity of optional content-free observations. |

## fleet

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `fleet.configured_workers` | count | 64 | next start | Bounds configured workers, including those not in contact. |
| `fleet.worker_name_bytes` | bytes | 32 | next start | Bounds bytes in each configured worker name. |
| `fleet.program_sources` | count | 12 | next start | Bounds program source names retained per connected worker, independently of run slots. |
| `fleet.source_detail_bytes` | bytes | 4,096 | next start | Bounds diagnostic detail retained for each login or sign-out failure. |
| `fleet.workers` | count | 8 | next start | Bounds workers in contact at once. |
| `fleet.needs_bytes` | bytes | 262,144 | next start | Bounds owned charter/model requirements retained for admission and placement. |
| `fleet.engine_slots` | count | 0 | next start | Bounds slots hosted inside the engine. |
| `fleet.slots` | count | 32 | next start | Bounds runs a worker hosts at once. |
| `fleet.workstreams` | count | 64 | next start | Bounds the workstreams kept per worker, for placement to prefer. |
| `fleet.attempts` | count | 64 | next start | Bounds the parent's attempts awaiting placement, acknowledgement or adoption, including replaced attempts. |
| `fleet.grants` | count | 32 | next start | Bounds credential names held by each attempt, across both protocol tables. |
| `fleet.calls` | count | 256 | next start | Bounds relayed calls the parent serves at once. |
| `fleet.call_name_bytes` | bytes | 64 | next start | Bounds bytes of a run's opaque call name retained for each relay. |
| `fleet.progress_bytes` | bytes | 1,024 | next start | Bounds whole tool-name bytes in one best-effort observation. |
| `fleet.turns` | count | 128 | next start | Bounds turns awaiting adoption or the parent's commitment, across all attempts. |
| `fleet.grace_ms` | milliseconds | 360,000 | next start | Bounds the wait for a lost worker to return or an unclaimed attempt to be adopted. |
| `fleet.restart_wait_ms` | milliseconds | 1,200,000 | next start | Sets the restart wait said to workers at a settled stop, from 0 to 86400000 milliseconds. |
| `fleet.facts` | count | 256 | next start | Bounds facts kept until the parent drains them. |
| `fleet.history` | count | 64 | next start | Bounds recent lost channels and refused hellos retained separately from diagnostics, at most this many of each. |

## authority

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `authority.projects` | count | 8 | next start | Bounds live project policies. |
| `authority.roles` | count | 8 | next start | Bounds roles per project policy. |
| `authority.requirements` | count | 8 | next start | Bounds generic requirements per deployment or project policy. |
| `authority.facts` | count | 16 | next start | Bounds supplied verdicts and guards per effect question. |
| `authority.grants` | count | 16 | next start | Bounds grants per admitted authority value. |
| `authority.executors` | count | 64 | next start | Bounds permitted executor entries per admitted authority value. |
| `authority.segments` | count | 8 | next start | Bounds base or resource-name segment count. |
| `authority.segment_bytes` | bytes | 256 | next start | Bounds bytes per base segment or terminal value. |
| `authority.implications` | count | 8 | next start | Bounds retained connector-kind implication pairs. |
| `authority.batch` | count | 32 | next start | Bounds directly delegated tasks per batch question. |
| `authority.accounts` | count | 16 | next start | Bounds model-account usability reports per run question. |
| `authority.writes` | count | 8 | next start | Bounds written resources per run question. |

## brief

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `brief.briefs` | count | 32 | next start | Bounds briefs gathering at once. |
| `brief.sections` | count | 10 | next start | Bounds sections in one brief. |
| `brief.read_bytes` | bytes | 8,192 | next start | Bounds size a connector may report for one section. |
| `brief.brief_bytes` | bytes | 16,384 | next start | Bounds total size of one completed brief. |

## accounts

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `accounts.token` | bytes | 7,934 | next start | Bounds the bytes of an account credential token. |
| `accounts.sign_in_deadline_ms` | milliseconds | 300,000 | next start | Bounds how long a sign-in waits for its redirect. |
| `accounts.program_hosts` | count | 65 | next start | Bounds hosts a program sign-out can await. |
| `accounts.accounts` | count | 16 | next start | Bounds accounts in the deployment. |
| `accounts.holder_accounts` | count | 4 | next start | Bounds accounts one holder keeps, the deployment or a party. |
| `accounts.program_names` | count | 12 | next start | Bounds lifetime program names, retired names included. |
| `accounts.sources` | count | 3 | next start | Bounds sources one account keeps, in its holder's order. |
| `accounts.sign_ins` | count | 4 | next start | Bounds sign-ins, LLM-key keeps, moves out and moves in under way at once. |
| `accounts.credentials` | count | 15 | next start | Bounds retained credentials and their expiry alarms. |
| `accounts.backoff_base_ms` | milliseconds | 2,000 | next start | Sets the initial backoff after a temporary source failure. |
| `accounts.backoff_max_ms` | milliseconds | 60,000 | next start | Bounds the backoff after a temporary source failure. |
| `accounts.retry_ms` | milliseconds | 30,000 | next start | Bounds the wait before retrying a source that needs sign-in or loading. |
| `accounts.move_deadline_ms` | milliseconds | 60,000 | next start | Bounds how long a move out waits for Written after its answer. |
| `accounts.spent_attention_ms` | milliseconds | 300,000 | next start | Sets the remaining spend-limit wait at which a source needs attention. |
| `accounts.facts` | count | 128 | next start | Bounds account observations waiting to be drained. |

## views

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `views.runs` | count | 32 | next start | Bounds live runs whose provisional text is retained, watched or not. |
| `views.subjects_per_change` | count | 32 | next start | Bounds subjects named by one change event. |
| `views.text_bytes` | bytes | 65,536 | next start | Bounds retained provisional bytes per live run. |
| `views.facts` | count | 256 | next start | Bounds best effort observations waiting to be drained. |

## notes

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `notes.scopes` | count | 19 | next start | Bounds scope indexes kept in memory at once. |
| `notes.entries_per_scope` | count | 64 | next start | Bounds entries admitted in one scope. |
| `notes.party_entries` | count | 12 | next start | Bounds entries admitted in one party's scope. |
| `notes.party_bytes` | bytes | 120 | next start | Bounds description bytes in one party's scope. |
| `notes.pattern_bytes` | bytes | 256 | next start | Bounds bytes in one connector pattern. |
| `notes.description_bytes` | bytes | 256 | next start | Bounds bytes in one note description line. |
| `notes.body_bytes` | bytes | 65,536 | next start | Bounds bytes in one note body. |
| `notes.references` | count | 8 | next start | Bounds task references in one entry. |
| `notes.load_rows` | count | 4 | next start | Bounds records in one store load page. |
| `notes.lines` | count | 8 | next start | Bounds index lines in one answer. |
| `notes.recalled` | count | 8 | next start | Bounds entries in one recall page. |

## hub

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `hub.program_sources` | count | 12 | next start | Bounds local program sources. |
| `hub.slots` | count | 80 | next start | Bounds runs and jobs hosted at once: the shared slots. |
| `hub.accounts` | count | 4 | next start | Bounds distinct grants retained for a hosted attempt. |
| `hub.charter_bytes` | bytes | 262,144 | next start | Bounds bytes of a charter. |
| `hub.transcript_bytes` | bytes | 73,727,378 | next start | Bounds opaque transcript bytes, including the committed call tail. |
| `hub.delivery_evidence_bytes` | bytes | 4,096 | next start | Bounds opaque evidence bytes in one settled delivery call. |
| `hub.turn_bytes` | bytes | 18,431,842 | next start | Bounds bytes of one completed turn body. |
| `hub.outcome_bytes` | bytes | 131,072 | next start | Bounds bytes of a run's declared outcome. |
| `hub.detail_bytes` | bytes | 4,096 | next start | Bounds bytes of detail an answer carries for operators: the tail of what it is given. |
| `hub.held` | count | 8 | next start | Bounds inbound events a run holds until it is live. |
| `hub.inbox` | count | 82 | next start | Bounds messages a run starts with and refusals a turn or answer carries. |
| `hub.event_bytes` | bytes | 63,738 | next start | Bounds bytes of an inbound event. |
| `hub.run_calls` | count | 48 | next start | Bounds host calls a run has in flight at once, including delivery. |
| `hub.facts` | count | 256 | next start | Bounds facts kept until the parent drains them. |
| `hub.told` | count | 256 | next start | Bounds activity and live pieces kept for the core while the link cannot take it. |
| `hub.turns` | count | 8 | next start | Bounds retained turns per attempt, with an additional window reserved for stopping. |
| `hub.turn_queue_bytes` | bytes | 147,454,736 | next start | Bounds retained body bytes per attempt, with an additional window reserved for stopping. |
| `hub.piece_bytes` | bytes | 4,096 | next start | Bounds bytes of one piece of live content. |

## documents

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `documents.rows` | count | 8 | next start | Bounds rows in one constructed document. |
| `documents.text_bytes` | bytes | 8,192 | next start | Bounds text bytes in one constructed document. |
| `documents.chips` | count | 8 | next start | Bounds chips in one constructed document. |
| `documents.lines` | count | 16 | next start | Bounds lines in one constructed document. |
| `documents.versions.minimum` | count | 1 | next start | Sets the oldest served document contract version. |
| `documents.versions.maximum` | count | 1 | next start | Bounds the served document contract version. |

## core

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `core.settings.proposals` | count | 8 | next start | Bounds pending settings proposals. |
| `core.settings.revisions` | count | 8 | next start | Bounds settings revisions held besides the newest. |
| `core.settings.versions` | count | 16 | next start | Bounds charter versions held besides the newest settings view. |
| `core.settings.history` | count | 64 | next start | Bounds revision records kept in the store beyond those held. |
| `core.settings.alternatives` | count | 63 | next start | Bounds model alternatives in a charter policy. |
| `core.settings.permissions` | count | 32 | next start | Bounds permissions in one settings policy value. |
| `core.templates` | count | 16 | next start | Bounds configured templates and overrides per project policy. |
| `core.fill_mark` | bytes | 1,073,741,824 | next start | Bounds store map bytes at or above which new party work is refused. |
| `core.connectors` | count | 4 | next start | Bounds numbered connectors retained by the core. |
| `core.resume_bytes` | bytes | 18,431,846 | next start | Bounds bytes retained for one task's resumable transcript tail. |
| `core.run_bytes` | bytes | 65,536 | next start | Bounds owned bytes of the configured run policy and its model alternatives. |
| `core.policy_bytes` | bytes | 16,384 | next start | Bounds bytes in one encoded mutable policy row the application journal accepts. |
| `core.escalation_reason_bytes` | bytes | 512 | next start | Bounds bytes in an ended person escalation rejection reason. |
| `core.load_slots` | count | 16 | next start | Bounds concurrent authenticated read loads, including retained result reads. |
| `core.load_bytes` | bytes | 131,072 | next start | Bounds owned bytes brought by one authenticated store read. |
| `core.call_records` | count | 128 | next start | Bounds named call replay and in-flight route slots. |
| `core.call_answer_bytes` | bytes | 135,168 | next start | Bounds total bytes of a retained opaque call name, tool name and settled answer. |
| `core.brief_parts` | count | 8 | next start | Bounds fragments when rendering a core-owned brief section. |
| `core.brief_core_budgets.tail` | bytes | 2,048 | next start | Bounds transcript tail text bytes in a fresh run's brief. |
| `core.brief_core_budgets.task` | bytes | 4,096 | next start | Bounds task section text bytes in one brief. |
| `core.brief_core_budgets.dependencies` | bytes | 2,048 | next start | Bounds dependency section text bytes in one brief. |
| `core.brief_core_budgets.attempts` | bytes | 1,024 | next start | Bounds attempt section text bytes in one brief. |
| `core.brief_core_budgets.plan` | bytes | 2,048 | next start | Bounds plan section text bytes in one brief. |
| `core.brief_core_budgets.notes` | bytes | 2,048 | next start | Bounds note index text bytes in one brief. |
| `core.brief_core_budgets.preferences` | bytes | 2,048 | next start | Bounds whole requester preferences in one brief. |
| `core.brief_core_budgets.now` | bytes | 4,096 | next start | Bounds a standing task's Now body in one brief. |
| `core.public.goals` | count | 0 | next start | Bounds live public goals per board. |
| `core.public.ended` | count | 0 | next start | Bounds ended public goals kept per project. |
| `core.public.lines` | count | 0 | next start | Bounds public lines per goal and delegates per card. |
| `core.public.depth` | count | 0 | next start | Bounds the lineage of a public card. |

## retention

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `retention.turns_ms` | milliseconds | 1,209,600,000 | next start | Bounds how long ended tasks keep their transcripts. |
| `retention.records_ms` | milliseconds | 2,592,000,000 | next start | Bounds how long ended tasks keep their full records and decided person proposals. |
| `retention.summaries_ms` | milliseconds | 31,536,000,000 | next start | Bounds how long ended tasks keep their summaries. |

## link

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `link.chunk` | bytes | 16,384 | next start | Bounds the channel bytes read at once. |
| `link.name` | bytes | 32 | next start | Bounds bytes in a worker name. |
| `link.secret` | bytes | 256 | next start | Bounds bytes in a worker secret. |
| `link.names` | count | 64 | next start | Bounds configured worker names. |
| `link.opening` | count | 16 | next start | Bounds connections opening at once. |
| `link.handshake_ms` | milliseconds | 10,000 | next start | Bounds the time to complete a channel opening. |
| `link.hello_ms` | milliseconds | 10,000 | next start | Bounds the time to receive a worker hello. |
| `link.ping_ms` | milliseconds | 5,000 | next start | Bounds the interval between channel pings. |
| `link.silence_ms` | milliseconds | 15,000 | next start | Bounds channel silence before declaring the link lost. |
| `link.stall_ms` | milliseconds | 30,000 | next start | Bounds how long a channel may stall. |
| `link.backoff_ms` | milliseconds | 250 | next start | Sets the first redial backoff span. |
| `link.ceiling_ms` | milliseconds | 30,000 | next start | Bounds the redial backoff span. |
| `link.proved_ms` | milliseconds | 60,000 | next start | Sets the open-channel span that resets the redial backoff. |
| `link.busy_ms` | milliseconds | 2,000 | next start | Bounds the wait before resending a turn answered busy. |
| `link.facts` | count | 4,470 | next start | Bounds channel observations waiting to be drained. |

## clients

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `clients.streams` | count | 64 | next start | Bounds client streams admitted at once. |
| `clients.exchanges` | count | 68 | next start | Bounds client exchanges admitted at once. |
| `clients.queued_bytes` | bytes | 520,094,521 | next start | Bounds encoded records held by one client stream, including its in-flight record. |
| `clients.exchanges_per_client` | count | 2 | next start | Bounds exchanges in flight per sign-in, shared by its clients. |
| `clients.reads_per_client` | count | 32 | next start | Bounds reads in flight per sign-in, alone or in a batch. |
| `clients.openings_per_client` | count | 31 | next start | Bounds watch openings in flight per sign-in, alone or in a batch. |
| `clients.provider_codes` | count | 256 | next start | Bounds volatile authorization codes. |
| `clients.provider_tokens` | count | 1,024 | next start | Bounds volatile provider access tokens. |
| `clients.public.reads` | count | 0 | next start | Bounds public reads admitted at once; zero turns public reading off. |
| `clients.public.rate` | count | 0 | next start | Bounds public reads per second; zero turns public reading off. |
| `clients.public.rate_burst` | count | 0 | next start | Bounds the public reads bucket at its fullest. |
| `clients.public.bytes` | bytes | 0 | next start | Bounds public answer bytes per second. |
| `clients.public.bytes_burst` | bytes | 0 | next start | Bounds the public answer bytes bucket at its fullest. |
| `clients.public.answer_bytes` | bytes | 0 | next start | Bounds one public answer and every declared public kind. |
| `clients.public.max_age` | count | 0 | next start | Bounds public answer freshness in seconds, at most sixty. |

## clients_http

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `clients_http.streams` | count | 64 | next start | Bounds HTTP client streams admitted at once. |
| `clients_http.calls` | count | 68 | next start | Bounds HTTP client calls admitted at once. |
| `clients_http.public_calls` | count | 0 | next start | Bounds public HTTP calls in flight apart from signed-in calls. |

## io

| Name | Unit | Stock | Applies | Bounds |
|---|---|---|---|---|
| `io.output` | bytes | 18,432,866 | next start | Bounds queued output per stream and the largest whole frame the stream can send. |
