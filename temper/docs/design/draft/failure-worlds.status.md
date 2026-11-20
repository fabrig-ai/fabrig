# failure-worlds: status

2026-11-20. Where the design lags or departs from [failure-worlds.md](failure-worlds.md).

- **Design lags:**
  - 3: `../testing/README.md`, 4.2 and `../testing/checks.md` give temper's worlds no inspection adapters, and their referees no checks beyond each scenario's own. jig's referee (`jig/docs/design/domain/testing.md`, section 6) already holds bounded delivery, order, ownership and nothing lost; what lags is 3.1's wake for every wait, 3.2's lifetime rule for every table, and 3.3's ledger.
  - 3.6: the stock profiles, policy and catalogues are checked figure by figure, not against one another.
  - 3.7: the start's link checks run only at a scenario's final cold restart.
  - 4: `skein/docs/design/testing/fakes/llm.md` has no shared rate window without `retry-after`; `skein/docs/design/testing/fakes/forge.md` has no lagging views; the fake machine has no partial fetch or full disk; and nothing models a stop in each phase of a start. The faults marked [script], [composition] or [assertion] need no design change.
  - 5: `../testing/README.md`, 2.1 has no world running several goals at once, and the seed has one stream.
  - 6 and 7: nothing.
- **Design departs:** none.
- **Open:** 8: the moves between layers that F0's plan proposes are the user's decision.
