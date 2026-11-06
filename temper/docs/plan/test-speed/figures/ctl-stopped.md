# ctl-stopped: measured refusal and process tier

Measured 2026-11-06 on `debian-16gb-hel1-8`, in one builder slot.
Base: `70c981f961377cc5373dc068683b32c87e8d3c42`.
Implementation: `8a22e56aa89116d71b57593ebe9eb3b8778cbbd6`.
The following commit adds these figures alone.

## Cause

A temporary probe initialized a fresh local deployment, disabled external
provider sign-in, gave its listeners free loopback ports, started its box,
waited for `listening clients`, and signed a client directory in through the
printed link on stdin. It stopped and reaped the box, then timed
`temper ctl DIR me </dev/null`. It never read the directory's token file.
Both versions used this procedure and the shipped dev binary.

| Stopped-box call | Base wall s | Base CPU s | Implementation wall s | Implementation CPU s |
|---|---:|---:|---:|---:|
| `temper ctl DIR me` | 30.012393 | 0.011509 | 0.021931 | 0.008380 |

Both calls exited 1. The base printed `the engine at ADDRESS did not answer
within 30 s`; the implementation printed `nothing listens at ADDRESS; is
temper running?`, each prefixed by `temper ctl: ` and ending with a newline.
CPU is the command's waited user plus system time; wall uses a monotonic
clock around the command.

The story's 30 seconds was the give-back hook's `ctl account list --json`
against its stopped box (`temper/crates/temper/tests/support/held_login.rs`,
`write_hook`). The read transport turned the kernel's refused connect into
a reset, and the host waited out `limits::DEADLINE`. Distinguishing that
initial refusal removes the wait without changing the hook, the story, its
assertions, or the 30-second deadline.

## Process tier

Both full runs used
`heavy python3 -I -B tools/tiers/run --cpu DIR temper process`, with fresh
private directories and `TEMPER_BUNDLE=none`. Summaries used
`tools/tiers/cpu-summary DIR --budgets tools/tiers/budgets.toml --pair process-temper`.
CPU includes waited and reaped descendants. Tier wall spans test execution
and settlement; compilation and prerequisites are excluded.

| Measurement | Base wall s | Base CPU s | Implementation wall s | Implementation CPU s |
|---|---:|---:|---:|---:|
| Full `process-temper`, 51 tests | 721.857293 | 305.911324 | 679.174372 | 287.060219 |
| `a_story_panicking_after_its_box_stopped_leaves_the_held_record_at_its_path` | 32.191338 | 0.975884 | 1.570655 | 0.783848 |

The implementation's focused process-tier story run also passed: 2.591002 s
wall, 0.861111 s CPU. Both observations are below 5 seconds.

The base had 50 passes and one failure:
`a_turn_landing_is_found_after_a_runner_cancellation_and_restarts` stalled
after a rebase update (91.535 s wall). Its one isolated base run passed
(42.224328 s wall, 23.721533 s CPU); it passed in the implementation's full
run too. The implementation had 51 passes. Each full run skipped the same
two existing ignored tests. Both stayed within the 360 CPU-second suite
and 60 CPU-second per-test budgets, with no starvation, unobserved tests,
leaks, or incomplete accounting.

The machine was loaded. During the base, load averages (1/5/15 minutes)
were 18.19/11.51/7.65 at 10:36:01 UTC; its command probe observed
14.13/15.18/11.56. The implementation tier began at 11:03:54 UTC with
14.43/13.42/12.09; at 11:09:37 UTC they were 16.56/15.39/13.42.
These are single tier runs, and the base includes the flake's stalled
minute. Their totals do not isolate this change's effect. The stopped-box
command and unchanged story directly show removal of the deadline wait.

## Scope

Six new CLI tests cover initial refused reads, acts and sign-in; keeping
an act's existing records and releasing its lock; a heard wait retaining
its deadline and recovering across a refused reconnect; and a sign-in
retry refused after the engine answered busy. Existing quiet-connection
deadline tests pass unchanged. A reply or streamed engine message latches
that the call has heard the engine, including a reply before `me`.

`temper open` and the terminal client still use the native shell's reset
and retry path; their stopped-box timing was not measured here. This
change is confined to the one-shot CLI host and its transport.
