# forge-client: status

2026-11-09. Where skein's code lags or departs from [forge-client.md](forge-client.md).

- **Not yet:** 7.3, 10: no real-Forgejo conformance record shows a merge
  update of a head already holding its base, or a rebase's push overtaking
  a later push (`forge.md`, section 9). The client world's late copies read
  the head at landing, so its stories for both styles do not establish the
  rebase's concurrent-push behaviour. The connector world checks a dropped
  push by an explicit branch rewrite; the design takes the forced push at
  its worst until the real forge's behaviour is recorded.
- **Not yet:** 3, 4: every watched repository is listed each `poll`, one
  watched only for branches or as a `Repository` included, and a hint
  starts its listing.
- **Not yet:** 2.2, 4.1, 7.7: `Limits` has no `fallback` and the client
  keeps no hearing: every rest, repair and queued merge's look keeps
  section 4's pace while deliveries arrive, and `Domain::poll_at` is
  missing.
- **Not yet:** 4.2: every keeping failure but `Missing` is read again
  after `backoff`, unchanging; a `TooLarge` comparison or inbox scan is
  read again and reports `Changed` each time.
