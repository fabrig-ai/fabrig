# channel: status

2026-11-07. Where jig's code lags or departs from [channel.md](channel.md).

- **Not yet:** 5.1: a `logins` row names no email. 1, 4.1, 8, 10.1, 10.2: TLS between hosts: the workers' listener and the worker's dial are plaintext on loopback or in-process (`jig/crates/jig-protocol-config/src/hosts.rs`, `address.rs`). 3, 6.4, 8: a worker whose root composes `smith-inline-agent`: none does, so no worker gives a tool begun with its call.
- **Departs:** 12: the `jig-world-cost` request says no story loses what it
  checks. Root checks at every step cover the six drawn fuzzy stories and
  the replaced-key and application-generation stories, but not the other
  stories, including `Facts` and the refusals. Those keep arrival, next-step,
  pre-replacement and surviving-host end checks, which can miss a transient
  value or one held by a killed host. The plan chooses this limit because
  those grants arrive through the same `Assignment` and `Granted` frames as
  the covered stories; extending coverage remains section 14's question.
