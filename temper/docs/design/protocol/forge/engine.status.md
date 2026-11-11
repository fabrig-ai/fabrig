# engine: status

2026-11-11. Where the tree's code lags or departs from [engine.md](engine.md).

- **Not yet:**  a forge with a secret and no listener is handed none and polls, but nothing names it in the log, where the section has it named once: `temper-protocol-records/src/forge.rs`, `access`, leaves the secret out, and `temper-service/src/lib.rs`'s `Note` has no entry for it. 1: a GitHub `Connect` on a clients' listener at port 0 is not refused (`temper-engine-domain-forge/src/connect.rs`, `make`); rows of a forge with no record are not named in the log (`temper-service/src/forge/records.rs`). 3: a removed forge's git identity keeps its grant in jig's accounts: `temper-service/src/forge/gone.rs` takes its values out of temper's identities' table alone, so a Forgejo identity's unbounded grant holds its room in jig's table until a restart. 4: `Fact::Gap` is dropped, neither logged nor shown (`temper-service/src/engine.rs`).  the idle cost check, an idle span's calls held to a bound computed from the keep's `poll`, backoff and `slow`; the worlds compare history sizes alone.
- **Departs:** 2, 8: `temper-box serve` writes no group file: the link sends `start FORGE owned|fake`, and the box writes `<data>/box/groups/N` holding the kind alone, composing the setup entry and port from `application.machine` at each use (`temper-service/src/route/box_link.rs`, `start`; `temper-box-machine/src/pipe.rs`, `StartKind`; `temper-forge-setup/src/machine.rs`, `entry`). The code records no reason; at most one owned and one fake forge are kept at once (`temper-engine-domain-forge/src/make.rs`, `owned`; `playground.rs`), so each kind has the machine's one entry.

The group-file departure (8) also names the box's `Command::StartKind`
(`temper-box-machine/src/pipe.rs`) and `temper-box-machine/src/group_records.rs`;
`temper-box forge remove` composes the setup entry too.
