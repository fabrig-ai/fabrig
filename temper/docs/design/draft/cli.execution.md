# cli: execution

2026-11-11. How [cli.md](cli.md) gets built. It designs nothing: each milestone writes its own design from the draft as it runs, and lands whole. Plans use the focused and fuzzy suites alone, and each walk comes after its milestone lands.

## 1. The milestones

| Milestone | Draft parts | Owner | Needs | Its walk | State |
|---|---|---|---|---|---|
| C1, the conventions and their mechanism: the command line's page in `skein/docs/foundation/`, `skein-shell`'s table, sets and rendering, help's format, and the checks, landing green with an exception for each thing temper hasn't brought into line yet, named for the milestone that removes it | 3, 8.1 to 8.3 | temper/ui | — | none | done |
| C2, `temper-box` and `temper-engine` in `temper/bin/`: the libraries holding their names renamed, their tables, sets, help, `--version`, pre-table modes as rows, generated documents, their internal variables renamed, and the dropped twins' tests reworked | 4.2, 4.3, 4.6, 8.4, 8.5, 8.7, 8.10 | temper/ui | C1 | none | done |
| C3, `temper` in `temper/bin/`, from `temper-client`: its table, sets, help, and `temper box`: the TUI in front of `temper-box --front`, the local link's pipe, the identity check, the cache and the download, and `temper-box` without `--no-tui` and `internal tui` | 4.1, 5, 8.4 to 8.6 | temper/tui | C1; C2 for `temper-box`'s name | none; the download's walk is C7's | done |
| C4, the test set: the 29 renamed to `TEMPER_TEST_*`, with their setters, and the test set's document | 4.6, 8.7 | temper/tiers | C1 | none | done |
| C5, features out of temper, each reporting the binaries' sizes before and after: `forge` and `worker` as crates; the recorders' shared journey and redaction code in a crate of its own, then the TUI's recorder; the web's `record`, `small-window`, `loopback-navigation` (with `skein-ui-web`'s) and `playground` | 6, 8.8 | temper/migration for `forge` and `worker`; temper/tui for the shared code and the TUI's recorder, telling temper/ui first; temper/ui for the web's | C1 | none | not started |
| C6, `temper-worker` and `temper-agent` in `temper/bin/`, with their tables | 4.4, 4.5 | temper/migration, with `deployment-binaries.execution.md`'s M3 and M4 | C1 | none | done |
| C7, the site: `temper` at the short link, gzipped, and `temper-box` in each commit's directory, with the download line | 5, 8.9 | whoever runs the site's release job, with `deployment-binaries.execution.md`'s M6 | C2, C3 | on a fresh machine, the download line starts the box | not started |
| C8, temper's tools and fixtures in `temper/bin/`, as internal binaries | 3.1 | temper/ui | C1 | none | not started |

## 2. Order

- **Low priority** (the user, 2026-11-09): its plans carry no priority mark, and start when places are free and marked work isn't about to arrive.
- **C1 first,** once this draft lands: it adds, and moves nothing, so it runs beside other work.
- **C2 to C6 and C8 after C1,** and after the work in the same crates has landed: `deployment-binaries.execution.md`'s M3 and M4, and the whole story's plans then in flight in the client and the service. Their moves and renames reach most of temper's crates and tests, so each lands as one change, out of the way of plans in flight. Marked work goes first, as everywhere.
- **C7 after C2 and C3,** and with `deployment-binaries.execution.md`'s M6.
- **Each later milestone removes its exceptions:** `until: "cli CN"` in `tests/boundary/tests/cli.rs` and `tests/boundary/tests/cli_names.rs`, and each `expect` whose reason starts `cli CN`. C5 also removes check 3's `temper/crates/temper-worker-shell/Cargo.toml: temper-service` exception: that dependency switches on the service's `worker` feature until C5 makes the worker's half crates of its own (`../protocol/binaries.md`, 1). One left matching nothing fails its check (`skein/docs/foundation/cli.md`, 9). C7 has none.
- **A milestone of `deployment-binaries.execution.md` not yet started when C1 lands** starts in `temper/bin/`, as C6 says.
