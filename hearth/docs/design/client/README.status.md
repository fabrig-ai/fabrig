# The client: status

2026-11-04. Where hearth's code lags or departs from [README.md](README.md).

- **Not yet:** 2: `hearth-client-domain-dev`, `hearth-client-protocol-incus`,
  `hearth-client-fake-incus`, `hearth-client-world` and
  `hearth-client-dev-world`.
- **Not yet:** 3.1: commands other than in-process `render`, `diff` and
  `build`. `build` renders and resolves, plans the layers and reports
  their keys, but its steps have no runner: the protocol layer answers
  them as unavailable, and the client exits 69. The remaining commands
  include capture, revision and slot operations, retarget and the
  development loop; the milestone marks defer their later variants.
- **Not yet:** 3.2, 3.3: the remaining commands' records and outcomes,
  including slot tables, identity rows, journal entries and the
  permission outcome (77). Render, diff, store progress, startup errors
  and build planning have records and terminal codes.
- **Not yet:** 3.4, 3.5: the steward's socket, peer credentials, admission
  and requests; the socket-first path and daemon fallback. The client
  always hosts the builder in-process.
- **Not yet:** 3.5: the `--proxy <url>` option. The service accepts a proxy
  in its configuration, but the binary's argument parser refuses the
  option and supplies no proxy.
- **Not yet:** 3.6, 3.7, 3.8: root-only commands, the read-only shell and
  bootstrap's arguments.
- **Not yet:** 4: the client world against the steward, including
  credential and conditional-command checks and the comparison between
  socket and in-process records.
