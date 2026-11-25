# macos: status

2026-11-25. Where temper's code lags or departs from [macos.md](macos.md).

- **Not yet:**
  - 2: temper's one story on macOS has not run green
    on the runner, so `../draft/macos.md`'s acceptance stands open.
  - 3 and 8: `temper/scripts/proof-live.sh` still takes GNU's `readlink
    -f` on a store not yet made, `stat -c`, `timeout` and `tail --pid`,
    and finds the processes it stops in `/proc` alone, so on macOS it
    exits 1 before its first build [in flight: change 18205].
