# limits: status

2026-11-04. Where smith's code lags or departs from [limits.md](limits.md).

- **Departs:** 4: limits are refused before the loop starts, but
  `smith-local-service/src/host_io.rs`'s `native_io_valid` and
  `smith-agent-service/src/service.rs`'s `io_valid` rebuild
  `skein_io::valid` by hand, without its ceiling on the operation table
  (skein's `io.md`, 5): from 2^24 sockets they pass, `Io::new` panics, and
  `native_io_valid`'s test pins `u32::MAX / 8` as valid; no reason is
  recorded (the code's side is
  `smith/docs/development/debt/D003-host-io-route-table.md`).
