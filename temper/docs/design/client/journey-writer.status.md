# journey-writer: status

2026-11-23. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 4, 7: `scripts/journey-pages.sh` holds the run in shell, and its
    tests run it as a process on real git; no `writer` command asks
    for its effects as values, and no step test holds its decisions
    [in flight: change 14131].
  - 6, 7: `scripts/look-refresh.sh` holds the refresh in shell, and its
    tests run it as a process on real git; no `look-refresh` command
    holds it [in flight: change 14133].
  - 2, 3: main holds seven leftover recordings, and the TUI's
    `launchpad-drawn` and `design-session` hold seven and fifteen pages
    rather than their walks' five and ten. `chat-draft` has no pages.
    The writer's next successful print of each journey removes its
    recording and surplus pages, and writes the pages it lacks.
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
