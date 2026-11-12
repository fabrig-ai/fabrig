# skein's runtime

Provisional, 2026-11-11.

The runtime documents cover the shared building blocks, the kernel
boundary and its backends, the shell that runs a service's loop, and io's
step layer. Read them from the shared vocabulary to the service-facing io:

- [`lib.md`](lib.md), `skein-lib`: handles, slabs, bounded containers, cursors, streams, deadlines and time; every other part stands on it.
- [`kernel.md`](kernel.md), `skein-io` (`kernel`): the records io submits to a backend, its completions and their contract.
- [`readiness.md`](readiness.md), `skein-shell`: the completion-emulating backend over epoll or kqueue for hosts without io_uring.
- [`shell.md`](shell.md), `skein-shell`, `skein-fail-stop`: the kernel backends, clock, seed, startup, panic hook and loop over a service's host.
- [`io/README.md`](io/README.md), `skein-io`: sockets, pipes, files, processes and signals as a service's step layer.
