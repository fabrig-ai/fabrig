# testing: status

2026-11-11. Where the tree's code lags or departs from [testing.md](testing.md).

- **Not yet:** 2: the user's `ctl-waits` request asks for the
  long-chat wait with a fake model refusing its resume, concurrent reads and
  a same-key retry, with the outcome captured at both sides. The joined
  `tests/client` story `kept_message.rs` extends a conversation and fills its
  inbox, captures Busy and the client's retry, then frees room and checks
  the answer and same-key replay. It disables run answers and drives no
  model's resume refusal. The bounded retry fix covers that full-inbox
  cause, but the reported 23.2 s success and repeated 30 s waits have no
  reproduced cause on record; the settings-effort tests likewise guard an
  existing final refusal, without reproducing that walk's silence. The next
  QA walk remains the requested proof (`README.md`, section 11).
- **Not yet:**  the absent command groups remain `tools.status.md`, 1.
   the absent read commands remain `tools.status.md`, 2.  the absent deciding commands remain `tools.status.md`, 3.
   the missing area operations remain `areas.status.md` (Not yet, the whole document).
   the hello stop-bound units remain `results.status.md`, 1.
   the trailing-help command syntax remains `README.status.md`, 6.1. 2:
  the joined story of a model's amendment (`tests/client`'s
  `amend_tools.rs` parses `--model` alone); the joined stories of
  `playground end`, `forge remove`, and a set up or a connect without
  FORGE, which `temper-client-tools`' own step tests hold alone; the joint component correlates
  forge setting reads, including historical revision values. 3:
  the real loop's connected Forgejo, its tokens connect, a replace of
  its git token and another user's tokens stopped.
