# tasks: status

2026-11-02. Where jig's code lags or departs from [tasks.md](tasks.md).

- **Not yet:** 5.3.1: job tasks. 5.9: a drain; a project's scope; a
  party's proposal of a release; a hold parked until an event. 7: a
  widening funded by whoever gives it (the task's existing funder is
  charged); moving a task under another task; successors; questions that
  outlive their asker. 8.3: markers. 8.5: a conversation's questions.
  11: a verdict's fields counted apart from a result's bytes: there is no
  `VERDICT_FIELDS`, and the settings codec sizes a verdict's fields by
  `result_bytes` (`jig-protocol-store/src/codec/settings.rs`, `limits`), so a
  result's bytes past the schema's 512 fail at store open.
  12: the stories of these.
- **Departs:** 7: a deleted project's conversations are cancelled with
  no grace (`jig-domain-core/src/admission/deletion.rs`, `cancel`),
  because `RemoveProject` names no grace and the engine reads no
  template (`people.md`, 5.2, and its status).
