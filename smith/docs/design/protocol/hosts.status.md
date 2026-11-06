# hosts: status

2026-11-06. Where smith's code lags or departs from [hosts.md](hosts.md).

- **Not yet:** 1, 3: the agent spawned in a contained tree with its view, its own
  trees inside it, and proved empty: `smith-host-protocol/src/process.rs`
  spawns a plain child with three pipes.
- **Departs:** 5: `smith-local-service/src/host_io.rs`'s `PROGRAM_ROOT` sets tag 1
  by hand, `Token::new(1 << 60)`, where the `debt-27` request of 2026-11-06 has
  smith's token tags use `Token::tagged` (skein's `lib.md`, 4), because that plan's
  cut held that it names one operation, with nothing nested under it, and no
  namespace.
