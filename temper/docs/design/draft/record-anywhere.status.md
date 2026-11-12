# record-anywhere: status

2026-11-12. Where the design lags or departs from [record-anywhere.md](record-anywhere.md).

- **Design lags:** none: sections 2 to 5 are designed in `../client/web/README.md`, 2.1 and 2.2; `../client/web/journeys.md`, 3.1, 3.5, 3.6 and 3.8; `../protocol/sign-in.md`, 7; and `../protocol/binaries.md`, the forms and `temper-box serve`'s `--record`.
- **Design departs:**
  - 3: "every page the deployment serves" records but the public pages (`../client/web/public.md`): they start no client host, whose seam the recorder keeps (`../client/web/journeys.md`, 3.1), as they record nothing by the address's setting today. The goal's designer's reading, 2026-11-12.
  - 3: `--record` is also taken by `temper-box`'s internal forms, the fake backends' and `--front`, so that `temper box --record` passes it, and by `temper-box internal serve --box`, which the box gives it to; a worker alone, serving no page, takes it and changes nothing (`../protocol/binaries.md`). The goal's designer's reading, 2026-11-12.
  - 4: a recording whose streams don't decode is saved with every `heard piece` all `x`s, where the draft names only the two originals, since the stream's credential can't be sought in a stream that doesn't decode; no such recording can be added (`../client/web/journeys.md`, 3.1). The goal's designer's reading, 2026-11-12.
  - 5: the bound is `RECORDING_BYTES`, 64 MiB, two of the client's whole-body bound, and the start's facts are kept whole (`../client/web/README.md`, 2.1; `../client/web/journeys.md`, 3.1). The goal's designer's reading, 2026-11-12.
  - 6: `cli.md`, 6's line on the internal settings is left as the draft wrote it, since a goal edits no draft's text; `cli.status.md` records the design's departure from it.
- **Open:** none.
