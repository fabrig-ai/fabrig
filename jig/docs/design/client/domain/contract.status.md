# contract: status

2026-11-07. Where jig's code lags or departs from [contract.md](contract.md).

Section 5.1 has no known gap in `Task.run_wait`'s hello and bounds
refusals, their optional required/offered figures or charter-bearing
turns, time and waiting shortfalls. These remain derived document facts,
with no new hold or stored state.

Sections 3.2 and 5.1 have no known gap in `ChangeModel`, numbered model
settings, source and estimated-price facts, the project's omitted count
(apart from the delivery gap below),
per-account provider-list narrowing, list history or new-label deadlines.
`ReadLists` carries the provider and returns the number of reads; account
times reach the accounts watch without marking an unchanged offer.

- **Not yet:** 3.2, 5.1: the client's charter carries no parts and its LLM
  choice no `later` (`jig/crates/jig-client-protocol-wire/schema/client-v19.schema`
  and `client-v19-part1.schema`, `jig/crates/jig-client-domain/src/contract/settings.rs`
  and `llm.rs`).
- **Not yet:** 5.1: a task's document shows its first choice whatever its reasons
  (`jig/crates/jig-domain-core/src/runs/llm.rs`, `document_llm`).

- **Not yet:** 5.1: a provider read changing only the visible accounts'
  largest `left_out` updates the accounts watch but does not mark `me`,
  leaving its omitted count stale until another Frame mark
  (`../../domain/engine.status.md`, 17.12). The field is computed correctly
  on a fresh read; no reason for the missing mark is recorded.

Section 5.1 has no known gap in either account row's bounded facts,
registered-kind filtering or application of each kind's audience. The
client wire and contract carry their values and reject oversized shares.
Unlisted sections, including 5.1's proposal-making lines and retained
proposal reasons and settings projects, and public documents (12), have no
known gap.

Sections 3.2 and 4 have no known gap in routing an acceptance with a
running-work choice to its retained decision, or answering `Unknown`
when no such record exists. The core world's settings-edge table checks
accepted and rejected task proposals, a party's withdrawn proposal,
task replacements with Stays and Moves, and unchanged settings, task,
decision record and pending replacement. Missing-proposal rows cover
both proposers with and without a running-work choice.

- **Departs:** 3.2, 4, from `ui-goal-asks-2`'s request for a replaced
  proposal to answer decided first: automatic task replacement retains
  no decision and answers `Unknown`. The answer of 2026-11-07 retained
  that refusal and declined an archive or a new decider because this is
  a ctl edge outside the happy path and spend binds. Section 11 keeps
  the retention choice open.

- **Departs:** 5.1: an omitted template's preview and its new task's next-run
  model can differ while no supporting worker is ready. `Core::preview_llm`
  (`jig-domain-core/src/runs/llm.rs`) takes the first of `Core::offer`, which
  needs no host for a direct source; `task_llm` uses `model_readiness`, which
  needs a host even for that source, then falls back to the first serving
  account. With a Claude program account and an OpenAI direct account, no
  ready worker leaves Sol in the preview and Opus in the task document.
  The plan deliberately kept direct sources offered across worker restarts
  and left the run's readiness check unchanged; that does not establish
  the promised agreement. Seen by reading; no runtime result is supplied.

- **Not yet:** 5.1: `me` gives `words_bytes` from `people.words` for a party's
  spec, message and answer, but says no bound on an amendment's: the long-words
  plan carries the words bound alone.
- **Departs:** 4: answers give typed lacking facts, how an object ended,
  and who decided it and how; the client wire carries these fields, but
  `jig-protocol-engine-clients/src/translate/refusals.rs`, `lack` and
  `ending`, always fill `LackingTaskKind.executor`, `LackingDepth.needed`,
  `LackingTasks.needed`, `LackingDeadline.asked` and `latest`, failed and
  cancelled endings' `reason`, and a decided ending's `party` and `how`
  with `None`, so a client always reads them empty. Either the engine
  fills them or a client wire version drops them; no reason is recorded.
- **Not yet:** 5.3: `me` represents a refused creating offer by absence,
  without carrying its refusal reason, including an authority-limit
  refusal when an executor list exceeds `documents.lines`.
- **Not yet:** 3.2: `Hold`; `Move` of a live task as its new requester; `Succeed` (neither `contract::Ask` nor the client wire's v19 schema has them). 5.1: the accounts document's aggregate `settled` counts only the party's pending sign-ins, not the program checks they asked, and lists no host whose answer is pending (`jig/crates/jig-domain-core/src/documents/accounts.rs`). 5.3: creating's executors list charters and procedures, not persons or roles (`CreateBounds`); a result's texts are not checked as UTF-8 where the tasks child admits a result; a project's document offers `RemoveProject` to no one, its owners included (`jig/crates/jig-domain-core/src/acts/objects.rs`, `admit`, refuses `Act::RemoveProject` not offered; `documents/offers.rs` leaves it out of a project's acts). 5.6: the `holds{project}` read. 6: a turn read past its transcript's horizon: the client reads `Gone`, and no engine path answers it. 7.4: an application's streams.
