# design: status

2026-11-12. Where the repository lags or departs from [design.md](design.md).

- **Not yet:** 3.2: no shared-test-code check, `tests/boundary/tests/shared_test_code.rs`, and no reader in `fabrig-boundary`'s library: `sizes.rs` holds its own [in flight: change 2020].
- **Not yet:** 3.2: no `skein-browser-rig`: `skein-browser-world`'s test binaries each compile its real rig and its pages [in flight: change 2049].
- **Not yet:** 3.2: no `jig-test-kit`: `jig-test`'s two test binaries each compile its support and the accounts kit's visitor [in flight: change 2050].
- **Not yet:** 3.2: `temper-real-loop-world`'s rail steps are a module each of its two test binaries compiles, not its library's [in flight: change 2051].
- **Not yet:** 3.2: no `temper-box-kit`: `temper-box`'s test binaries each compile `tests/support/` and `tests/fake_backends/box_link.rs` [in flight: change 2089].
- **Not yet:** 3.1: `jig-protocol-store`'s `people` and `tasks` are still the size check's exceptions, sharing the store with its other families [in flight: change 2040].
- **Not yet:** 3.2: `tests/boundary/` has no codec check [in flight: change 2042].
