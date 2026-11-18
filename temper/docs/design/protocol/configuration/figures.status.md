# figures: status

2026-11-18. Where temper's code lags or departs from [figures.md](figures.md).

- **Not yet:** 6: the agents' environment is empty at every start, the worker alone's and the box's (`temper-worker-service`'s `startup::worker::application::fixed` and `temper-service`'s `startup::application::parse_over` give none), so an agent's programs run with no `HOME` and no `PATH` but what smith's own spawn gives. [in flight: change 7446]
- **Not yet:** 9: no test holds the agents' environment, and no story runs an agent's command that needs the worker's `PATH`. [in flight: change 7446]

