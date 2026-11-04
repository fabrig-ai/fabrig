# The wall-clock plan's measurements

October 2026. The evidence behind three rules of `docs/design/benchmarks.md`, moved here from the design text, which keeps the rules. The plan ran smith's wall-clock work against Codex: a `parity` baseline, change runs on the way, and a closing `parity` run.

## Change runs resolve little (16.6, "Measured alone, or with the others")

Four change runs took 9.3 hours of the gate lane, a third of the plan's run. Their RT intervals were 0.34 to 0.78 wide, against effects of 4% to 22% of smith's time, and two of them stopped on noise. Their ratios multiplied to 1.083. Meanwhile the closing run moved RT from 1.397 to 1.033, and the counts the steps targeted moved too: responses on `shell-cwd` fell from 70.1 to 54.2.

So a change whose run cannot tell its expected effect from none proves itself on the counts it targets, and leaves the closing run to measure it.

## A projection counts each change once (16.6, "its projection")

In step 10.1, the RC projection was 1.821, computed as a product over the changes landed before it. A factor of 1.485 of that came from three codes measured twice, once as a change and once as the next change's parent. Each cost more in its own run than as the next step's parent.

So a projection is the baseline's statistic times this change's ratio alone.

## Where the probes' numbers come from (11.1)

- **`slow-completion`, over 60 s:** 60 s is the whole-response cap at which the spike's attempts were cut (findings S5).
- **`large-write`, three exercised passes within forty-eight attempts:** the plan's proving rules.
- **`parity`, ten attempts on `shell-cwd` (16.1):** the plan's decisions record the test's power at five, ten and fifteen attempts.
