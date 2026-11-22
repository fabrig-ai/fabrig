# rate-limits: status

2026-11-22. Where skein's code lags or departs from [rate-limits.md](rate-limits.md).

- **Not yet:** 1, 2: a message's `try again in` figure and its limit's window read into `RateLimited`'s delay where the head and the error's fields give none; `RateLimit::delay` reads the head and the fields alone.
