# forge: status

2026-11-15. Where temper's code lags or departs from [README.md](README.md).

- **Not yet:** 2.6: serve's start looks up GitHub's API host and its twenty results hosts, one at a time, before it listens (`temper-engine-shell/src/serve.rs`, `github_runtime`; `serve/github_logs.rs`), and no forge waits for GitHub's API host while the engine serves [in flight: change 3453].
- **Not yet:** 2.6: nothing hands a forge already added a results host resolved later (skein's `LogOrigin`), and a results host not resolved keeps its `Fact::Origin` as a refused origin (`temper-engine-service-component/src/forge/origins.rs`) [in flight: change 3455].
- **Not yet:** 9: the protocol world's Forgejo change whose Actions CI stays pending ten minutes, with and without deliveries, counted a minute at a time.

`engine.md`, section 1 has no known implementation gap in adoption's users among
jig's parties: their subjects name the repository's forge and the user.
The forge record also holds its optional nonzero user/person link, and
setup evidence reaches the connector as `PersonOf`; the root logs its
gone, writer, replaced and cleared notes. The retained-link scope gap is
recorded in `../../domain/forge/adoption.status.md`, 6. Links for several members
and pasted Apps remain deferred (`../setup/README.md`, section 9).

- **Not yet:** 4: a forge with a secret and no listener is handed none and polls, but nothing names it in the log, where the section has it named once: `temper-protocol-records/src/forge.rs`, `access`, leaves the secret out, and `temper-service/src/lib.rs`'s `Note` has no entry for it.  a GitHub `Connect` on a clients' listener at port 0 is not refused (`temper-engine-domain-forge/src/connect.rs`, `make`); rows of a forge with no record are not named in the log (`temper-service/src/forge/records.rs`).  a removed forge's git identity keeps its grant in jig's accounts: `temper-service/src/forge/gone.rs` takes its values out of temper's identities' table alone, so a Forgejo identity's unbounded grant holds its room in jig's table until a restart.  `Fact::Gap` is dropped, neither logged nor shown (`temper-service/src/engine.rs`). 9: the idle cost check, an idle span's calls held to a bound computed from the keep's `poll`, backoff and `slow`; the worlds compare history sizes alone.
