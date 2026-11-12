# requests: status

2026-11-11. Where the tree's code lags or departs from [requests.md](requests.md).

- **Not yet:** 2: `RemoveProject` of a project with a live task is admitted, not refused *live tasks* [in flight: change 628].
- **Not yet:** 2: `RemoveProject` sent again under its key after the deletion ended is answered unknown, not removed, and the deleted project's read unknown, not gone [in flight: change 629].

- **Departs:** 2: from `ui-goal-asks-2`'s request for a replaced
  proposal to answer decided first: automatic task replacement retains
  no decision and answers `Unknown`. The answer of 2026-11-07 retained
  that refusal and declined an archive or a new decider because this is
  a ctl edge outside the happy path and spend binds. `README.md`, section 9 keeps
  the retention choice open.

- **Not yet:** 2: `Hold`; `Move` of a live task as its new requester; `Succeed` (neither `contract::Ask` nor the client wire's v19 schema has them).  the account settlement facts remain `documents.status.md`, 1.  the creating executor list remains `documents.status.md`, 3; a result's texts are not checked as UTF-8 where the tasks child admits a result; a project's document offers `RemoveProject` to no one, its owners included (`jig/crates/jig-domain-core/src/acts/objects.rs`, `admit`, refuses `Act::RemoveProject` not offered; `documents/offers.rs` leaves it out of a project's acts).  the holds read remains `documents.status.md`, 6.  the transcript-horizon read remains `README.status.md`, 4.  application streams remain `README.status.md`, 5.4.

`README.md`, section 2's browser-approved sign-in is present on the client wire and
in the clients' component. Only a take with the secret makes the ordinary
sign-in of the browser's party; a bearer's decision is refused.
