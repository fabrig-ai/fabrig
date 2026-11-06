# github: status

2026-11-06. Where the design lags or departs from [github.md](github.md).

The design carries parts A, B and C (`../domain/forge.md`, `../protocol/setup.md`, section 17, `../protocol/credentials.md`, `../client/forge.md`, skein's `forge-github.md`), and `../../help/github.md` walks it.

- **Design lags:**
  - 9: `../domain/forge.md` leaves open which checks are CI on GitHub without required checks (section 20), where this draft takes the repository's Actions workflows' check runs, pending skein's `Status` carrying each row's App.
- **Design departs:**
  - 12.2: a Forgejo connected as its admin has its hook at adoption and at each start's repair only when its API is on loopback; one elsewhere polls (`../protocol/setup.md`, 5.5), where this section has the hook whenever the deployment has a webhooks' listener, because a forge on another host can't reach a loopback listener. The user's word of 2026-11-06, the request `managed-listener`.
  - 11: this section keeps a probe account beside `fabrig-ai`, with `temper-probe-2` that account's spare, not granted to the App. `../domain/testing.md`, 2.7 and the managed sitting answer to `fabrig-ai` alone, whose `temper-probe-2` the App's installation covers, because the user's request of 11-04 took the personal account out of the live probes.
