# real-journeys: common rules

Every task names this file. This is the guard between a real account's secrets and a public repository, so its tests carry the plan.

- **A failing test first:** for each kind and place your task names, the test is written and seen failing before the code that passes it. Your report says how it failed.
- **Test data is fake:** a secret's shape is built in the test from pieces, so that the source spells none whole: `["s", "k", "-"].concat()`, `["ey", "J"].concat()`, `["Bear", "er "].concat()`, `["co", "de="].concat()`, `["refresh", "_token"].concat()`. `tests/boundary/tests/outward.rs` and the repository's own checks stay clean. No test reaches a real host or account.
- **No bytes in a message:** no error, panic, assertion or Debug print of the production code holds an original. A test checks by position (`assert!(find(haystack, secret).is_none(), "the kind and place")`), never `assert_eq!` on bytes that may hold one, so that its failure's log is as clean.
- **The committed journeys are read, never written:** `temper/docs/design/client/journeys/**` is another draft's by priority. Reprint nothing; regenerate nothing.
- **Their cost:** no fuzzy test. The journeys' reprint together is at most 1 s of the default suite (`temper/docs/design/client/journeys.md`, 6); a test of yours that replays a committed journey says its time in your report.
- **The design is written:** the sections you read are binding, and what the code teaches goes in your report. `journeys.status.md` is the last task's.
- **Each crate's lints hold:** `temper-client-shell-web` is the `binary` profile with its own `clippy.toml`, and `tools/journeys` the `harness` profile under `temper/tools/clippy.toml`.
