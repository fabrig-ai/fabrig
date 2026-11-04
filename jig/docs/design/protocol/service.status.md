# service: status

2026-11-04. Where jig's code lags or departs from [service.md](service.md).

- **Not yet:** 2, 6, 10: agents in a host's own process (`smith-inline-agent`). 10: a client the process holds, over an in-process transport. 6, 7: contained process trees, and TLS between hosts. 7, 8: `ops`'s service and `ops` standalone: `jig/examples/ops` has no service. 8: the live story fails on a real plan. 11: the user's request of 2026-11-04 has jig ship what temper copies. All seven parts ship and both services compose them, but both still write a bundle's file kinds, digest-path lookup and reply chunking (`jig/testing/jig-test-service/src/server/bundle.rs`, `temper/crates/temper-service/src/clients/bundle.rs`; section 9), and the services' shared distinct lines read 356 against the bound of 200, largely each one's own pass and adapters. The code's side is `jig/docs/development/debt/D005-accounts-host-copied.md` and `temper/docs/development/debt/D001-reference-service-copy.md`.
