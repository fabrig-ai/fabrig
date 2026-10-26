# The suites' times at the base

2026-10-19, on debian-16gb-hel1-1 (8 vCPU), the run's machine: jig at
`6ad8bb40`, temper at `1c918a77c`, each in a worktree of its
`origin/main`, through the gate lane, alone (load 1.1 and 6.8 at the
start: the code graph was indexing). Three runs of each suite:

```
$ cd ~/src/rust/tmp/jig/tools-base && sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run --workspace 2>&1 | grep Summary; cargo nextest run --workspace --profile fuzzy 2>&1 | grep Summary; done'
     Summary [   6.364s] 4270 tests run: 4270 passed (1 slow), 0 skipped
     Summary [   7.329s] 111 tests run: 111 passed, 0 skipped
     Summary [   5.044s] 4270 tests run: 4270 passed, 0 skipped
     Summary [   6.724s] 111 tests run: 111 passed, 0 skipped
     Summary [   5.232s] 4270 tests run: 4270 passed, 0 skipped
     Summary [   6.379s] 111 tests run: 111 passed, 0 skipped
$ cd ~/src/rust/tmp/temper/tools-base && sh -c '(the same)'
     Summary [   2.738s] 1879 tests run: 1879 passed, 1 skipped
     Summary [   7.038s] 50 tests run: 50 passed, 0 skipped
     Summary [   2.892s] 1879 tests run: 1879 passed, 1 skipped
     Summary [   7.163s] 50 tests run: 50 passed, 0 skipped
     Summary [   2.333s] 1879 tests run: 1879 passed, 1 skipped
     Summary [   6.934s] 50 tests run: 50 passed, 0 skipped
```

| Suite | Median | Swing |
|---|---|---|
| jig focused | 5.23 s | 1.32 s |
| jig fuzzy | 6.72 s | 0.95 s |
| temper focused | 2.74 s | 0.56 s |
| temper fuzzy | 7.04 s | 0.23 s |

The same suites on carbon-forge (4 cores), from the Gate-Result trailers
of the last code landings: jig `bbea9408` 13.19 s and 24.07 s, temper
`3a3f9bb9d` 7.56 s and 24.69 s. The README's aims are this machine's;
measured on carbon-forge they would miss.
