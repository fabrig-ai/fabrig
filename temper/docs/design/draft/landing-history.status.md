# landing-history: status

2026-11-13. Where the design lags or departs from [landing-history.md](landing-history.md).

Nothing of this draft is designed or built yet: its execution file's milestones write the design as they run.

- **Design lags:**
  - 1 and 3: `../domain/forge/change/README.md`, 2 keeps a change's step and the time of its last step, replaced at the next; `../domain/forge/queues.md` derives a change's place in its queue and keeps no time for it; `../domain/landing.md`, 12 keeps a landing's current round alone and no time for its steps or rounds; `../domain/landing-search.md`, 9 keeps the search's bounds, not when it ran; `../domain/after-landing.md`, 10 keeps the recent landings as a ring, not their steps. No step writes an event, nor carries an identity for one.
  - 4: `../domain/landing.md`, 6.2 keeps one time per receipt, temper's; the forge client reads no run's start or end and no pull request's merge time (Later).
  - 5: `../domain/forge/change/README.md`, 7 evicts an ended change's row with all it holds; `../protocol/store.md`, 2, 4 and 5 has no history family, no demand-loaded temper family outside the restored state, and no sweep by age.
  - 6 and 7: `../domain/landing.md`, 14 offers `landing show` and `landing list`, which hold no ended `queue` change and no steps; `../domain/forge/change/README.md`, 5 has no panel for an evicted change; `../client/ctl/README.md`, 9 has neither `landing history` nor `landing times`; `../protocol/wires.md`, 8 has no client read for them.
  - 8: `../protocol/configuration/limits.md` has neither `history_days` nor `history_events`.
- **Design departs:** none yet.
- **Open:** none.
