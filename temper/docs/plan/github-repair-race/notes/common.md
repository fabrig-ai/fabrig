# github-repair-race: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`.
- The change is commit `063edf0d7e` of `reference-m7a/repair-1`, as it is: the same bytes on
  main and on M7a's trunk, so that when M7a's trunk lands the two sides merge cleanly. Nothing
  is reworded, reformatted or improved beside it.
- The only file this plan changes is `temper/tests/real-loop/tests/suite/github_repair.rs`. No
  `docs/design/`, no draft, no `docs/help/`, no status file, no other test, no crate, no
  dependency change.
- The package is `temper-real-loop-world`, its test binary `suite`; the test runs in the
  focused profile (`default`), in nextest's `temper-real-loop` group (two at a time).
