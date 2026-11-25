# shell-table: status

2026-11-23. Where the tree's code lags or departs from [shell-table.md](shell-table.md).

- **Not yet:** 1.4: `Table::reach`, `Reached` and `Table::unknown`.
- **Departs:** 1.2: the page's `environment:` part, and a command page's, also list
  the host's variables a setting's `Fallback::Base` reads, after the
  twins and the table's variables, in the settings' order, so `HOME`
  comes after the first XDG variable (`table/help.rs`,
  `help_environment`; `table/help/tests.rs`, `HOST_ENVIRONMENT`). That
  reads 1.1's "a table that holds such a setting reads them" as putting
  them on the page; 1.1 has the documents say so, and
  `../../foundation/cli.md`, 7's example, the draft's
  (`temper/docs/design/draft/cli.md`, 3.6), lists the directories' twins
  alone: `temper-box`'s and `temper-engine`'s pages are the first to
  list them.
  1.3: a binary's documents gain a `## Test` part after `## Internal`
  when its table holds a test entry, where 1.3 has the two alone and
  `../../foundation/cli.md`, 4 puts the test set in the test set's document.
  No table holds one.
