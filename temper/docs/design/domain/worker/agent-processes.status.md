# agent-processes: status

2026-11-22. Where the tree's code lags or departs from [agent-processes.md](agent-processes.md).

- **Not yet:** 1, *Its build places too, and its bound counts only them*: fabrig's check takes a hold's first line as held, a `waiting` line included, and tells smith of no wait (`tools/tiers/run`, `hold`) [in flight: change 9657].
- **Not yet:** 1, *Its build places too, and its bound counts only them*: smith counts a check's bound from its start, its waits included (`smith-domain-run/src/land.rs`, `next`) [in flight: change 9658].
- **Not yet:** 1, *The base's build has a target of its own*: the check's base runs inherit its `CARGO_TARGET_DIR`, or none (`tools/pre-pr/pre_pr.py`, `judge_base`) [in flight: change 9655].

`agent-processes.md`, section 1's agent program is the worker's named program: the box's
`temper-box internal agent AGENT.json` or a deployment's `temper-agent
AGENT.json`, with the same program used for a model list
(`../../protocol/README.status.md`, 5). Every temper worker names its agent's
`host_reply_text` as the hello's reply bound, the in-process worker among
them (`temper-worker-shell/src/agent.rs`). The service gives the worker's
`Limits::reply_bytes` the same figure (`temper-service/src/worker_settings.rs`).
- **Not yet:** 1: a standalone start past the process's `limits::SOCKETS`, 256, by reading
  21 slots on the owned example with workspaces and operations as many, is
  refused `engine.limits.io.sockets: 258 bytes, past its bound of 256`, naming a
  limit no document raises (`checks::process_io` sets it), and a worker alone
  past the link's `u32` caps, by reading about 31 slots at that example's hub
  (`jig-protocol-worker-wire/src/sizes.rs`, `worker_cap`), is refused
  `workers[0].limits: past its bound`, naming none. The `worker-many-slots`
  request of 2026-11-05 has each start's refusal name what to raise; it asked
  for 8 and 10, which fit, so this is `README.md`, section 10's question. No test runs 8
  slots past `temper_box_shell::prepare_in`: that request takes the next deployment at 8
  and 10 as its proof, with no walk, and no record shows one.

- **Departs:** 1: with `TEMPER_INTERNAL_DUMP_LLM` in the worker's environment at its start, an agent
  process's environment holds that one variable, an absolute directory, and its
  log's lines also go to `dump.log` there; unset, which is the default and the
  only shipped form, section 1 holds exactly. Because the deployment's
  comparison with another pipeline must read what each run was sent
  (`../../draft/dogfood-hacks.md`, 2.13). It carries no credential. No test
  runs the variable from a worker through a real agent process to a file, as
  the request of 2026-11-05 leaves that to the next comparison's run.
