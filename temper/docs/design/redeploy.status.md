# redeploy: status

2026-11-19. Where the code lags or departs from [redeploy.md](redeploy.md).

- **Not yet:** 4.1, 6.3 and 8: the tool notes the count from the last line anywhere holding `upgrade: rehearsal linked `, not the first that starts with `temper-engine upgrade: rehearsal linked `, and the fake's unscripted verdict prints no count line (`temper-redeploy-tool/src/redeploy.rs`, `fake.rs`) [in flight: change 9351]
