# wire: status

2026-11-17. Where the tree's code lags or departs from [wire.md](wire.md).

- **Not yet:** 2.1: a batch's records, `Member`, `Batch` and `BatchReply`, at jig's client wire 28 [in flight: change 6338].
- **Not yet:** 2.2: a batch admitted as one exchange, each member taken, reserved and answered as alone, its replies handed out as `Part`s, then `Done`; the per-sign-in reads and openings in flight; `StreamOpen`'s `replaces`, the named stream ended first [in flight: change 6346].
- **Not yet:** 2.3: `Message`'s `Batch`, the heard `Part` and `Done`, and `OpenStream`'s `replaces`, carried by the transports [in flight: change 6354].
- **Not yet:** 2.3, *Sent at the settling*: the link gathering what a step asks and handing it out at the settling [in flight: change 6345].
- **Not yet:** 2 and 2.3: one batch for what a settling gathered, each member its own, and each opening of the stream naming the one it replaces, with no stream gone sent [in flight: change 6355].
