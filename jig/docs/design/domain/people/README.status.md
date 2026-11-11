# people: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).

Section 4's `Domain::sole_owner` is present: it reads the deployment's
roster without allocation or mutation, including during restore, and
answers none with no owner, several owners or an unreadable roster.
The project-members table covers those cases.

Unlisted sections, including public reading (13) and `acts.md`, 1's refusal of a
live task's move to its current requester as not offered, have no known gap.
Loading goals ended before publication or across a restart remains
deferred in section 11; a connector's retained commits do not widen the
tasks handed from the public index (`public-reading.md`, 3, 9).
`public-reading.md`, section 3's title decision is carried by temper's publication help and
completed ctl answer. No acknowledgement precedes `SetVisibility`; that
client step remains deferred in temper's `client/ctl/README.md`, section 11.

- **Departs:** 5, 7: the person's own answer is off by default, behind
  `Core::person_answers()` (`jig-domain-core/src/inbox.rs`), because
  each application opts in before the first step, as temper does. With
  it off, the asked task answers and relays. With it on, the inbox holds
  unanswered questions to a person's conversations alone, for that person
  to answer; a question to any other task its person asked for is the
  task's to answer. Credits stored
  before it was set, naming a task answerer, route by the question's
  current inbox owner, and moving a recipient re-points its incoming
  person-answerer credits to its new person.

Section 3's browser-approved sign-in uses the clients' component's
`Whose` and an ordinary sign-in with no held credential. Nothing reaches
people before the take; the joint-client stories cover the approving
party and an approval never taken making no sign-in.
- **Not yet:** 3: a service's credential reissued or removed. 
  the unbuilt party acts remain `acts.status.md`, 1. 5: retention; a
  delegate's question to a conversation kept with the conversation. 6:
  a chat answering its delegates' questions and asking its person by a
  person task.  orphan note entries remain `acts.status.md`, 2.
