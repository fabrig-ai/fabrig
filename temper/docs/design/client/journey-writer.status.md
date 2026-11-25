# journey-writer: status

2026-11-25. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 4, step 5, and 6.3: the writer's run lists no candidate to retire;
    `looks.html` has no foot. [in flight: change 18159]
  - 5: the delivery check refuses a journey's directory deleted whole
    as written files beside its sources, and lets a `walk` deleted
    alone through while its pages stay. [in flight: change 18160]
