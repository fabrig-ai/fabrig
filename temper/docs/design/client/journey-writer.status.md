# journey-writer: status

2026-11-27. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 6.2, steps 5 and 8: a page with elements past `Limits::elements`
    is `uncompared`, `its markup doesn't parse as written`, not `too
    large to compare, over N elements`. [in flight: change 20973]
  - 7: the render check's lines, read back: one step test reads two
    hand-written pages' lines; the listed lines and the test that reads
    them all are not built. [in flight: change 20973]
