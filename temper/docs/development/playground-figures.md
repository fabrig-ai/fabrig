# Playground figures

2026-11-04, Chromium 147.0.7727.101, at 10×.
Measured by `story_a_walks_on_the_built_playground` in the browser tier
(temper/docs/design/playground.md, 8 and 9).

```text
playground figures: download raw 19854021 B gzip 7026541 B; memory high water 6186598 B; longest slice 117.799 ms
```

| Figure | Measured | Bar | Result |
| --- | ---: | --- | --- |
| Download, raw | 19,854,021 B | No separate raw bar | Reported |
| Download, gzip | 7,026,541 B | 8 MB (8,000,000 B) gzip | Under by 973,459 B |
| Memory high water | 6,186,598 B (frame: 5.9 MiB) | the user's, Open | Reported |
| Longest slice | 117.799 ms | the user's, Open | Reported |

The download sums every file under the built playground's output, including
`index.html` and `headers`; gzip encodes each file separately with
`skein_http::gzip::encode` and its bound. Memory is converted from the frame's
displayed tenths of MiB, so the byte figure has that display's precision.
The slice is the frame's displayed maximum. The test enforces no figure bar.

The CPU-accounted tier ran 12 tests, all passed: 63.835377 CPU seconds
(51.624100 user, 12.211277 system) and 61.022827 seconds wall, within the
85-second CPU budget and the design's 120-second cap
(temper/docs/design/client/browser.md, 4.3). No test was starved or unobserved;
accounting reported no leaked or incomplete process trees. The playground
walk used 14.806375 CPU seconds and 13.573744 seconds wall; its downloaded
session log replayed natively with `Ok`.

The first tier run through `temper/scripts/browser.sh` passed all 12 tests
in 61.222 seconds. Its playground bundle setup took 120.398 seconds; the
CPU-accounted run's warm setup took 14.861 seconds. Setup is outside the tests'
time. The first run measured the same download and memory, and a longest
slice of 117.900 ms.

## The compressor, 2026-11-06

The test now compresses each file with the machine's `gzip` at its default
level, with no name or timestamp, not `skein_http::gzip::encode`. That encoder
took 4.85 CPU seconds of the story in the dev profile, and compression here is a
tool, not the code under test; the machine's took 1.046. Figures from the two
don't compare: the same 29 files, 21,241,423 B raw, read 6,882,055 B by `gzip`
1.13 and 7,593,581 B by the encoder, each under the 8 MB bar.
