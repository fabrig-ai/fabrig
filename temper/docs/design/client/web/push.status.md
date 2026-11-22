# push: status

2026-11-22. Where the tree's code lags or departs from [push.md](push.md).


- **Not yet:** `push.md`, 1, *Tested*: no browser-tier story turns push on. The
  browser kit can't do it offline: `skein-browser`'s requests and
  `fake-person-browser`'s steps can't grant a notification permission,
  and Chromium's `subscribe` reaches its own push service, a real one.
  It needs a kit change first: a permission grant and a local push
  facility in skein's browser kit, by skein's design. The shell's push
  calls and the service worker are checked by compiling and by the
  service's route tests only (2026-11-11, goal N3b, which stops at six
  changes).
