# README: status

2026-11-04. Where jig's code lags or departs from [README.md](README.md).

- **Not yet:** 4, 5: agents in a host's own process: no engine or worker of jig's composes `smith-inline-agent`. 10: TLS's server side between hosts, the workers' and clients' listeners being plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`); contained process trees for workers' agents.
- **Departs:** 8, 10: the printed link's secret is a secret, and credentials' values take the generator's `secret` type, whose `Debug` shows its length only; but `jig/crates/jig-client-protocol-wire/schema/client-v19.schema` declares `record LinkSignIn { secret: bytes 64 }`, so the generated `LinkSignIn` derives `Debug` over the secret's bytes. The fix is the schema's field type and a regenerated codec, which changes the public `LinkSignInParts` at its callers in jig and temper: the bugs-security plan left it for a plan of its own (its question `link-sign-in-secret`).
