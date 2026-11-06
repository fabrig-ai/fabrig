# contract: status

2026-11-06. Where jig's code lags or departs from [contract.md](contract.md).
Unlisted sections have no known gap.

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
  3.4: `jig-protocol-engine-clients/src/translate/operations.rs` answers an
  operation of another project `Malformed::ProjectMismatch` itself, where this
  section has people admit a compound, because people's `Operation` holds no
  project, and the `debt-28` plan of 2026-11-06 found the move past the size its
  request allows (section 11; the code's side is
  `jig/docs/development/debt/D026-engine-clients-project-mismatch.md`).
- **Not yet:** 5.3: `me` represents a refused creating offer by absence,
  without carrying its refusal reason, including an authority-limit
  refusal when an executor list exceeds `documents.lines`.
- **Not yet:** 3.2: `Hold`; `Move` of a live task as its new requester; `Succeed` (neither `contract::Ask` nor the client wire's v19 schema has them). 5.1: `me`'s templates carry no `resolved_default` preview; the accounts document's aggregate `settled` counts only the party's pending sign-ins, not the program checks they asked, and lists no host whose answer is pending (`jig/crates/jig-domain-core/src/documents/accounts.rs`). 5.3: creating's executors list charters and procedures, not persons or roles (`CreateBounds`); a result's texts are not checked as UTF-8 where the tasks child admits a result; a project's document offers `RemoveProject` to no one, its owners included (`jig/crates/jig-domain-core/src/acts/objects.rs`, `admit`, refuses `Act::RemoveProject` not offered; `documents/offers.rs` leaves it out of a project's acts). 5.6: the `holds{project}` read. 6: a turn read past its transcript's horizon: the client reads `Gone`, and no engine path answers it. 7.4: an application's streams.
