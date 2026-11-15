# bridges: status

2026-11-15. Where jig's code lags or departs from [bridges.md](bridges.md).

- **Not yet:** 2: `Spoken::kept`, and `spoken()` and `wire::layout()` read from the generated `LAYOUTS`; they parse the kept texts at each call. [in flight: change 3879]
- **Not yet:** 4: the table parsed from the floor's texts alone, and the link's `Component::with_schema`; a worker builds the table at each check and again in its link. [in flight: change 3880]
- **Not yet:** 9: `spoken()` against the parsed schemas' ranges, and `wire::layout()` against the current text's. [in flight: change 3879]
- **Not yet:** 9: the table from the floor's texts against every text's, and the link from a prepared table. [in flight: change 3880]
