# github-forgejo: status

2026-11-22. Where the design lags or departs from [github-forgejo.md](github-forgejo.md).

- **Design departs:**
  - 1.2: a Forgejo connected as its admin has its hook at adoption and at each start's repair only when its API is on loopback; one on another host polls (`../../protocol/setup/reconcile.md`, 5), where this section has the hook whenever the deployment has a webhooks' listener, because a forge on another host can't reach a loopback listener. The user's word of 2026-11-06, the request `managed-listener`.
