# oauth: status

2026-11-02. Where skein's code lags or departs from [oauth.md](oauth.md).

- **6.3, never back to back.** The component sets a just-kept record's
  lead from its expiry alone; with a refresh lead longer than the new
  token's lifetime it refreshes again at once. Task
  `live-smith-reds-2/refresh-twin` closes it.
