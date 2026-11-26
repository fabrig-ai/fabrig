# journey-writer: status

2026-11-26. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 6.2, steps 2 and 5 to 8, and 7: `look-render` renders each page a
    gallery-drawn directory changes, never `not the requested look`;
    touches a gallery-drawn page's every element and compares its
    whole style and box, a selected element with no pair a `pair`
    difference; and on a journey's page lists `width`, `height`,
    `inline-size`, `block-size`, `transform-origin` and
    `perspective-origin` beside the box. [in flight: change 18837]
  - 6.2, step 4, and 7: its Chromium starts without
    `--hide-scrollbars`. [in flight: change 18835]
