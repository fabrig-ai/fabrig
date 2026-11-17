# deployment: status

2026-11-17. Where temper's code lags or departs from [deployment.md](deployment.md).

- **Not yet:** 2's four documents with no `agent.json`, 3's pinned agent configuration under `data/worker/workspaces/agent/`, and 6.2's `engine` or `worker` alone: the driver still requires and renders `config/worker/agent.json` [in flight: change 6894].
- **Not yet:** 5's order with the worker's setting, `worker add` after ctl's sign-in and before the worker starts: the driver starts the worker before ctl and waits for its `linked`, which an engine holding no worker setting refuses `not-admitted` [in flight: change 6894].
