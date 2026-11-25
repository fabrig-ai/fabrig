# journey-writer: status

2026-11-25. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 2, 4: the run prints at every start, whatever the timer's rule
    says, and keeps no finished run; it checks no landed request,
    `looks.html` is not among its written files, its commit's dates
    are the clock's, and its key names the tip, not the offered
    commit. [in flight: change 17985]
  - 6: `look-refresh` drops a request by `Printed:` alone and reads no
    `looks.html`. [in flight: change 17985]
  - 6.2: the journeys tool has no `look-render` command.
    [in flight: change 17984]
  - 6.3: nothing writes or reads `looks.html`.
    [in flight: change 17985]
