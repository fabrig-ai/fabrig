# pause: status

2026-11-20. Where jig's code lags or departs from [pause.md](pause.md).

- **Not yet:** 2: `jig-worker-wire` is at 18, with no `pause` or `paused` kind and no `PauseMode` or `PauseState` in the link's vocabulary [in flight: change 10975].
- **Not yet:** 3: the engine's component sends no `pause` and gives no `Unpaused`, and reads no `paused` [in flight: change 10975].
- **Not yet:** 3: no fleet request for a pause reaches the link, and no `paused` reaches the fleet [in flight: change 10983].
- **Not yet:** 4: the worker's link hands up no `pause` and sends no `paused` [in flight: change 10975].
- **Not yet:** 4: the hub says no pause state, after a hello or at a change [in flight: change 10982].
