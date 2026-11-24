# agent: status

2026-11-24. Where smith's code lags or departs from [agent.md](agent.md).

Section 4 has no known gap in endpoint `limits`, refusing endpoint model
lists or taking model and effort names from the charter within their bounds.

Section 6.1 has no known implementation gap in `--list`, its bounded
stdin/stdout records, endpoint request or typed failure, without a run.
The fake-provider child stories establish no real-provider result.

- **Not yet:**
  - 1, 3, 6, 8: the view. Commands, searches and checks run as plain
    children in process groups (`smith-protocol-machine/src/process.rs`),
    with no read-only git directory or machine and no writable scope;
    startup checks no machine for contained trees, and the agent says
    `containment: groups`.
  - 6, 8: a live run ended by its channel's end within `orphan_exit`,
    and the process world's stories of a host gone at each point.
    [in flight: change 16344]
  - 6, 8: a live run ended by its host's silence, and the stories of a
    silent host. [in flight: change 16346]
- **Departs:** 6: a host may have the shell write each model request and what
  a program was given to a private directory (`smith-agent-shell/src/dump.rs`),
  with its lines also in `dump.log` there, because a development deployment
  must read what a run was sent; it comes out when that need ends. A write
  blocks the shell's pass, and never fails a run. Unasked, nothing is kept.
  A name holds its Start's identity, or `none` where the host gave none, never
  the identity the run mints, and a rename replaces what has that name, so a
  host that gives no identity keeps two agents' files apart by their
  activations alone.
