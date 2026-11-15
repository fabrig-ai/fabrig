# checkout-failure-words: status

2026-11-15. Where the design lags or departs from [checkout-failure-words.md](checkout-failure-words.md).

The design carries this draft. Section 1's detail, its bound, the worker's
host and line and serve's line are `../domain/worker/checkouts.md`, 1; check
out and clean told apart are `../protocol/git.md`, 4.5 and 6; serve's line
and the preparation's failure are `../domain/worker/README.md`, 4.3. No wire
or stored format changes: the command a failure ended in is the checkout's
in-process `PushDiagnostic`, and the rest is bytes of the failure's detail.

- **Design lags:** none.
- **Design departs:** 1: the worker's host is its configured name
  (`workers[].name`), the name the engine and its operator know it by: the
  worker reads no machine's host name, and each worker of a deployment has
  its own name (the goal's designer, 2026-11-15). The detail's fields come
  first and its git words last, and it is bounded at 2,048 bytes, so every
  cut on the way, the worker's and serve's, falls on the words; jig's hub
  keeps a detail's last `hub.detail_bytes`, 4,096 by default, which that
  bound stays under (`../domain/worker/checkouts.md`, 1.2).
- **Open:** none.
