# design: status

2026-11-04. Where the slow tier runner's code lags or departs from [design.md](design.md).

- **Not yet:** 1, 2.1, 3, 6, 7, 7.5: `run` and `cpu-run` name one build tool before every command and wait for a fixed disk figure; neither `TIERS_WRAPPER` nor `TIERS_DISK_BYTES` is read, and `run`'s listing sets that tool's tail variable. 2: `run` still bundles for the retired `qa-temper`. 5: `test_run.py` covers neither plain commands nor the settings. 7.3, 10.2: `execute` cancels through its command's own cancel call, parsing a token from its output, and `summary` counts neither a nextest global timeout nor a recorded cancellation abort as starved. 7.4: `test_cpu.py` covers neither of those, nor cancellation of a command's descendants. 10.3: the command's output is written under the tool's name.
