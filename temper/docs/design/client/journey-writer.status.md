# journey-writer: status

2026-11-26. Where temper's code lags or departs from [journey-writer.md](journey-writer.md).

- **Not yet:**
  - 2: `temper-client-walk pages` selects directories whose `walk`
    starts with `walk 2` and leaves other directories untouched. It
    does not remove a directory's numbered pages when it has no `walk`.
    The selection is `temper/crates/temper-client-walk-tool/src/lib.rs`'s;
    `pages_removes_recording_and_preserves_unmoved_and_stopped_journeys`
    in that crate's `tests/suite/main.rs` holds the preservation.
  - 6.2 and 7: `look-render` asks git for the changed hand-made paths
    as `diff-tree ... -rz`, which git refuses with its usage, so the
    writer's run stops at a landed request with a hand-made directory;
    no test runs the look commands' git argument lists with real git
    [in flight: change 18416].
  - 6.1, step 2, and 6.2, step 1: the request is fetched from `origin`
    alone, even where the checkout holds C
    (`temper/crates/temper-client-journeys-tool/src/look_compare/request.rs`)
    [in flight: change 18417].
  - 6.2, steps 2, 3 and 6: a page drawn on a gallery state is
    `uncompared`, `hand-made`, with no gallery print beside it, and the
    texts the check diffs are not put in the print's form for a diff
    [in flight: change 18418].
