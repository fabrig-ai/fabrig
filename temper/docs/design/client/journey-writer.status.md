# journey-writer: status

2026-11-26. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 6.3: the writer's reader takes only `NN.html` page paths, so a
    render report naming a page that is no journey's stops the run;
    no step test reads `look-render`'s own lines back (7).
    [in flight: change 18783]
  - 6.2: step 4's Chromium starts without file access, so a page's
    own masks and images are refused. [in flight: change 18784]
  - 7: the look commands' git test is one of the library's step
    tests, running every look step test again; no list file.
    [in flight: change 18785]
