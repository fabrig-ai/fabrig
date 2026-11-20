# builds: status

2026-11-20. Where temper's code lags or departs from [builds.md](builds.md).

- **Not yet:**
  - 2: no deployment builds for landed QA yet; its person sets it up once `temper-qa build keep` lands.
  - 3, step 6, and 4: the checksums are still GNU's `sha256sum` process, not the tool's own SHA-256 with the `manifest` module's one writer and reader. [in flight: change 10611]
