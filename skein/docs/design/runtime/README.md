# skein's runtime

Provisional, 2026-11-11.

The runtime documents cover the shared building blocks, the kernel
boundary and its backends, the shell that runs a service's loop, and io's
step layer. Read them from the shared vocabulary to the service-facing io:

- [`lib.md`](lib.md), `skein-lib`: handles, slabs, deadlines and time; every other part stands on it.
  - [`lib-containers.md`](lib-containers.md): bounded containers, state digests, map joins and sorted search.
  - [`lib-bytes.md`](lib-bytes.md): byte values, Base64 and hashes.
  - [`lib-streams.md`](lib-streams.md): cursors, streams and output reservations.
- [`kernel.md`](kernel.md), `skein-io` (`kernel`): the records io submits to a backend, its completions and their contract.
  - [`kernel-files.md`](kernel-files.md): files beneath a root, tree metadata and filesystem operations.
  - [`kernel-cgroups.md`](kernel-cgroups.md): cgroups and the helper.
- [`readiness.md`](readiness.md), `skein-shell`: the completion-emulating backend over epoll or kqueue for hosts without io_uring.
- [`shell.md`](shell.md), `skein-shell`, `skein-fail-stop`: the kernel backends, clock, seed and loop over a service's host.
  - [`shell-startup.md`](shell-startup.md): startup, the panic hook, settings, listeners and durable files.
  - [`shell-table.md`](shell-table.md): command tables, help and reference documents.
- [`io/README.md`](io/README.md), `skein-io`: sockets, pipes, files, processes and signals as a service's step layer.
