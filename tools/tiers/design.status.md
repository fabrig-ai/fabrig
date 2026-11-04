# design: status

2026-11-04. Where the slow tier runner's code lags or departs from [design.md](design.md).

- **Not yet:** 7.3, 10.2: `execute` cancels through its command's own cancel call, parsing a token from its output, and `summary` counts neither a nextest global timeout nor a recorded cancellation abort as starved. 7.4: `test_cpu.py` covers neither of those, nor cancellation of a command's descendants. 10.3: the command's output is written under the tool's name.
