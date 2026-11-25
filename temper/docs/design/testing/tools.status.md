# tools: status

2026-11-25. Where temper's code lags or departs from [tools.md](tools.md).

- **Not yet:**
  - 2: the journeys tool has no `look-render` command.
    [in flight: change 17984]
  - 4.4: the gallery, journeys, transcripts, stage-one and twin tools
    have no reference documents. The playground host's documents are
    rendered and checked; the tree's common and test documents exist
    (`../cli/README.md`, 8 and 11).
  - 6: `temper-client-gallery`, `temper-client-walk`,
    `temper-git-transcripts`, `temper-playground-serve` and `temper-qa`
    still launch the binary for each row of their tables, rather than
    their crates' unit tests holding the rows beyond one launch per
    path.
