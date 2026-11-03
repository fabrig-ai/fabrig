# people: status

2026-11-03. Where jig's code lags or departs from [people.md](people.md).

- **Not yet:** 3: a service's credential reissued or removed. 5.1:
  `Hold`; `Move` under another task; `Succeed`; a party's `Propose` of a
  release; a widening funded from the amender's pool. 6: retention; a
  delegate's question to a conversation kept with the conversation. 7: a chat answering its delegates' questions and asking
  its person by a person task.
- **Departs:** 6, 8: the person's own answer is off by default, behind
  `Core::person_answers()` (`jig-domain-core/src/inbox.rs`), because
  each application opts in before the first step, as temper does. With
  it off, the asked task answers and relays. With it on, the inbox holds
  unanswered questions to a person's conversations alone, for that person
  to answer; a question to any other task its person asked for is the
  task's to answer. Credits stored
  before it was set, naming a task answerer, route by the question's
  current inbox owner, and moving a recipient re-points its incoming
  person-answerer credits to its new person.
