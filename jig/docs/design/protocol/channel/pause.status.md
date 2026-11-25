# pause: status

2026-11-25. Where jig's code lags or departs from [pause.md](pause.md).

- **Not yet:** 3: `Unpaused` indexes the settings list by worker number
  minus one instead of resolving the authenticated name as `Opened`
  does. It can name the wrong worker or panic for sparse worker numbers.
- **Not yet:** 4: the link queues retained calls with the hello, before
  `paused`. The application's worker also routes retained turns and
  answers as part of routing the hello, ahead of the pause notice.
- **Not yet:** 5: no focused wire test checks the conversion plan's
  per-value refusal for `pause` down to a worker layout without it or
  `paused` up to an engine layout without it. The component tests
  exercise skipping the absent kind, not the plan's refusal.
