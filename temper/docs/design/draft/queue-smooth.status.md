# queue-smooth: status

2026-11-11. Where the design lags or departs from [queue-smooth.md](queue-smooth.md).

No gap: the design carries all four of its wants, and nothing was dropped.
A landing's own update is its own in `../domain/forge/change/README.md`,
section 3, *Its own writes are its own*, `../domain/forge/README.md`,
section 6, and `../domain/forge/change/repairs.md`, *Stalls* and *A stall's
hold says where*, the bound being the existing stall,
`forge.change_policy.stall_ms`. The first keeps the queue in
`../domain/forge/queues.md`, *The first owns the queue*. A runner's fault
runs again in `../domain/forge/change/README.md`, section 2, step 3, *The
runner's failures run again*, `../domain/forge/README.md`, sections 2 and
4, kind 17, and `../protocol/configuration/limits.md`,
`forge.change_policy.reruns`, with `ci-on-forgejo-17.status.md`'s
departures from that draft's section 2.
