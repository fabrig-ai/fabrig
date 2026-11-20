# sign-in: status

2026-11-20. Where the tree's code lags or departs from [sign-in.md](sign-in.md).

- **Not yet:** 3: the one click's lines, the "Type a code instead"
  button, `may_type_instead`, `Told::Listener` and their goldens [in
  flight: change 10572].
- **Not yet:** 3: the host's one click: the listener opened before the
  ask, the ask with its port, the tab with its code, "Type a code
  instead" asking a typed code with no tab, and "Ask again" asking the
  one click again [in flight: change 10576].
- **Not yet:** 4: the run's loopback listener, `loopback::grant`'s
  reading of its one path, and its answers [in flight: change 10574].
- **Not yet:** 4: the host's half: the grant given to the root, the
  redirect back, the 404s and `unlisten` at each closing turn [in
  flight: change 10576].

No other gap.
