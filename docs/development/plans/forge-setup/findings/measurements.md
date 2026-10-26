# Measurements at the base, on x1

Base: temper `origin/main` at `0ff45022e` ("Design the forge's setup, with
webhooks on loopback"), in a detached scratch worktree with its own `target/`
(cold for the gate). x1, 8 cores, 2026-10-16. `waited` and `ran` are the command's figures; `alone` is the command's
"ran alone" line (the command itself). `ran` also holds the gate lane's
handover, which pauses the build group and waits for its running task.

## Commands, exit status, time

1. `scripts/check.sh` (needs no env): exit 0, waited 0s, ran
   120s, alone 116s. A cold build (197 crates compiled) is inside it. Its
   nextest summaries: default 5.514s (1297 run, 1 skipped), fuzzy 24.999s
   (46 run).
2. `sh -c 'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do
   cargo nextest run --workspace 2>&1 | grep Summary; cargo nextest run
   --workspace --profile fuzzy 2>&1 | grep Summary; done'`: exit 0, waited 1s,
   ran 223s, alone 89s (134s of `ran` was the handover behind another task).
3. `TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
   TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
   scripts/proof-one-process.sh`: exit 0, waited 126s (queued
   behind another gate task), ran 51s, alone 50s. 6 tests passed, summary
   45.731s; the build inside the script took 3.1s. Slowest:
   `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart` 30.974s,
   `temper_open_adopts_a_repository_by_commands` 13.102s,
   `temper_open_signs_in_prints_the_inbox_and_exits_on_q` 1.176s; the other
   three under 0.3s.
4. The same two env vars, then `cargo run -p forgejo-conformance
   -- --scratch target/forgejo-conformance --out tools/forgejo-conformance`
   (tools/forgejo-conformance/README.md): exit 0, waited 0s, ran 44s, alone
   43s. It wrote `facts.txt` with 69 `holds`, 4 `fails`, 1 `unchecked`:
   `pull-for-names-the-newest-of-several`, `open-pull-no-changes-is-422`,
   `delete-missing-branch-is-404`, `effective-protection-name-for-a-writer`
   fail; `webhook-deliveries` is unchecked. The committed `facts.txt` has 109
   `holds`, 4 `fails`, 1 `unchecked`: the 40 missing `holds` are the
   `component-` facts, which only `--component` writes, so a plain recording
   deletes them. It also changed 34 transcripts (35 files in all). The
   worktree was restored (`git checkout -- . && git clean -fd`), then removed.

## The two suites, three samples each (nextest's own Summary time, step 2)

| Suite | Sample 1 | Sample 2 | Sample 3 | Median | Swing (max - min) |
|---|---|---|---|---|---|
| default (1297 tests) | 4.472s | 4.434s | 4.386s | 4.434s | 0.086s |
| fuzzy (46 tests) | 24.581s | 24.173s | 24.218s | 24.218s | 0.408s |

Budgets (.config/nextest.toml): default 15s, fuzzy 60s.
