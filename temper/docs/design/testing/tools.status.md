# tools: status

2026-11-23. Where temper's code lags or departs from [tools.md](tools.md).

- **Not yet:**
  - 2, 4.2, 6: `temper-client-journeys` has no `writer` command; the
    writer's run is `scripts/journey-pages.sh`'s shell
    [in flight: change 14131].
  - 2, 4.2, 6: `temper-client-journeys` has no `look-refresh` command;
    the refresh is `scripts/look-refresh.sh`'s shell
    [in flight: change 14133].
  - 6: `temper-client-journeys`'s suite launches the binary for each
    row of its table and each refusal of `print`
    [in flight: change 14086].
  - 6: `temper-stage-one`'s and `temper-twin`'s suites launch the
    binary for each row of their tables and each refusal
    [in flight: change 14119].
  - 4.4: the gallery, journeys, transcripts, stage-one and twin tools
    have no reference documents. The playground host's documents are
    rendered and checked; the tree's common and test documents exist
    (`../cli/README.md`, 8 and 11).
  - 6: `temper-client-gallery`, `temper-client-walk`,
    `temper-git-transcripts`, `temper-playground-serve` and `temper-qa`
    still launch the binary for each row of their tables, rather than
    their crates' unit tests holding the rows beyond one launch per
    path.
