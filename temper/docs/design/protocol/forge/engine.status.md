# engine: status

2026-11-19. Where the tree's code lags or departs from [engine.md](engine.md).

- **Not yet:** 1: a first set up's intent keeps no `sample`
  [in flight: change 10131].

- **Not yet:**  a forge with a secret and no listener is handed none and polls, but nothing names it in the log, where the section has it named once: `temper-protocol-records/src/forge.rs`, `access`, leaves the secret out, and `temper-service/src/lib.rs`'s `Note` has no entry for it. 1: a GitHub `Connect` on a clients' listener at port 0 is not refused (`temper-engine-domain-forge/src/connect.rs`, `make`); rows of a forge with no record are not named in the log (`temper-service/src/forge/records.rs`). 3: a removed forge's git identity keeps its grant in jig's accounts: `temper-service/src/forge/gone.rs` takes its values out of temper's identities' table alone, so a Forgejo identity's unbounded grant holds its room in jig's table until a restart. 4: `Fact::Gap` is dropped, neither logged nor shown (`temper-service/src/engine.rs`).  the idle cost check, an idle span's calls held to a bound computed from the keep's `poll`, backoff and `slow`; the worlds compare history sizes alone.
