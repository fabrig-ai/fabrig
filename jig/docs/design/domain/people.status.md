# people: status

2026-11-02. Where jig's code lags or departs from [people.md](people.md).

- **Not yet:** 3: a service's credential reissued or removed. 5.1:
  `Hold`; `Move` under another task; `Succeed`; a party's `Propose` of a
  release; a widening funded from the amender's pool. 6: retention; a
  delegate's question to a conversation kept with the conversation. 7: a
  chat answering its delegates' questions and asking its person by a
  person task.
- **Departs:** 5.1, 5.2, 5.3: a project's owners, and the deployment by
  its act, remove a project (`Ask::RemoveProject`, `ActKind::RemoveProject`;
  `jig-domain-core/src/admission/deletion.rs`), which this design does not
  have, because the playground is reset
  (`temper/docs/design/draft/playground-reset.md`, 5 and 6). 5.1: `Seen`
  answers a store kept from before `Seen` saved none still restore,
  replay and hold their places until their expiry
  (`kept_seen_answers_restore_replay_and_occupy_room_until_their_original_expiry`,
  `jig-domain-people/src/tests.rs`); a code task removes them. 6, 8: the
  person's own answer is off by default, behind `Core::person_answers()`
  (`jig-domain-core/src/inbox.rs`), because each application opts in
  before the first step, as temper does. With it off, the asked task
  answers and relays. With it on, credits stored before it was set,
  naming a task answerer, route by the question's current inbox owner,
  and moving a recipient re-points its incoming person-answerer credits
  to its new person.
