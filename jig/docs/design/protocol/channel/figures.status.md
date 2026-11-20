# figures: status

2026-11-20. Where jig's code lags or departs from [figures.md](figures.md).

- **Not yet:** 3, 4, 7: neither the engine's workers host nor the worker's link takes new timers while it runs: there is no `Timers` and no `retime` on either side, and so no test of them [in flight: change 10944]
