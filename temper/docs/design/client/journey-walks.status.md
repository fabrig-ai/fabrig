# journey-walks: status

2026-11-21. Where temper's code lags or departs from [journey-walks.md](journey-walks.md).

- **Not yet:**
  - 5.2: `pages` leaves directories without a version-2 `walk`
    untouched, including their numbered pages, rather than removing
    pages from a directory with no `walk`. The selection is
    `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 6: main's pages await the writer: `chat-draft` has none; seven
    journeys keep leftover recordings; the TUI's `launchpad-drawn`
    and `design-session` keep seven and fifteen pages rather than
    their walks' five and ten. Each successful print replaces its
    journey's pages and removes its recording and surplus pages.
