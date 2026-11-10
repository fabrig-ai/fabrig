# forge-client: status

2026-11-10. Where skein's code lags or departs from [forge-client.md](forge-client.md).

- **Not yet:** 7.3, 10: no real-Forgejo conformance record shows a merge
  update of a head already holding its base, or a rebase's push overtaking
  a later push (`forge.md`, section 9). The client world's late copies read
  the head at landing, so its stories for both styles do not establish the
  rebase's concurrent-push behaviour. The connector world checks a dropped
  push by an explicit branch rewrite; the design takes the forced push at
  its worst until the real forge's behaviour is recorded.
