# design: status

2026-11-17. Where the outward scan's code lags or departs from [design.md](design.md).

- **Not yet:** 3 and 5: the Rust test's prefilter holds where the text holds an entry's first three bytes, not its whole name, and is a key folded again at every byte and sought in a sorted list; its matching tries every entry at each byte of a line; and no test holds the prefilter against a naive one.
