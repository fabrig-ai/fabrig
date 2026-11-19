# macos: status

2026-11-19. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:** 2: temper's one story on macOS has not run green
  on the runner, so `../draft/macos.md`'s acceptance stands open.
- **Not yet:** 3: `bin/temper-box/tests/suite/support_tests.rs`'s
  `the_elsewhere_deployment_keeps_the_whole_fixture_provider_in_its_data_directory`
  reaches `forgejo_elsewhere` ungated, so temper-box's `suite` does not
  build on macOS; it and its listed neighbours in `support_tests.rs` and
  `fake_backends/mod.rs` carry no reason beside their gates [in flight: change 10148]
