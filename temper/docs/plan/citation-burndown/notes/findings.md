# The 67 findings and their owners

`python3 -I -B tools/citations/citations.py --all` at 17273de9e2, 2026-11-02: 67 findings, as
`file:line: kind target`. Each belongs to one task. A line number may have moved on your base;
the file, kind and target identify a finding.

## jig-store-items: a real break, store.md's items have no headings (6)

- `jig/crates/jig-domain-tasks/src/positions.rs:1: section jig/docs/design/protocol/store.md 15.7`
- `jig/crates/jig-domain-tasks/src/retire.rs:1: section jig/docs/design/protocol/store.md 15.1`
- `jig/docs/design/domain/people.md:645: section jig/docs/design/protocol/store.md 15.7`
- `jig/docs/design/domain/tasks.md:1243: section jig/docs/design/protocol/store.md 15.7`
- `jig/tests/store/tests/suite/upgrades/generations.rs:81: section jig/docs/design/protocol/store.md 10.1.2`
- `jig/tests/store/tests/suite/upgrades/generations.rs:118: section jig/docs/design/protocol/store.md 10.1.5`

## skein-forge-readme: a real break (1)

- `skein/docs/design/forge-github.md:927: path tests/forge/github/README.md`

## temper-citations: real breaks (6)

- `temper/docs/design/client/mockups/STATUS.md:76: section-self temper/docs/design/client/mockups/STATUS.md 9`
- `temper/docs/design/client/mockups/STATUS.md:134: section-self temper/docs/design/client/mockups/STATUS.md 11`
- `temper/docs/design/draft/archive/first-run.md:393: section-self temper/docs/design/draft/archive/first-run.md 7.2`
- `temper/docs/design/draft/reference-code-production.status.md:50: section-self temper/docs/design/draft/reference-code-production.md 8.2`
- `temper/docs/design/protocol/configuration.md:81: section-self temper/docs/design/protocol/configuration.md 15`
- `temper/docs/design/protocol/credentials.md:694: section-self temper/docs/design/protocol/credentials.md 14`

## checker-readings: misreadings (37), by the rule that reads them right

- **R1 and R2,** paths up two directories, and the possessive and parenthesised forms:
  - `jig/docs/design/client/domain/README.md:8: section-self jig/docs/design/client/domain/README.md 10`
  - `jig/docs/design/client/domain/contract.md:1368: section-self jig/docs/design/client/domain/contract.md 14`
  - `jig/docs/design/client/domain/testing.md:96: section-self jig/docs/design/client/domain/testing.md 15`
  - `jig/docs/design/client/domain/testing.md:421: section-self jig/docs/design/client/domain/testing.md 15`
- **R3,** "Its" with a capital:
  - `jig/docs/development/quality-review.md:21: section-self jig/docs/development/quality-review.md 10`, and the
    same line's `11` and `12` (3 findings)
- **R4,** "its" after another tree's path:
  - `temper/docs/design/domain/testing.md:2183: section temper/docs/design/domain/testing.md 9`
- **R5,** an RFC's parenthesised section:
  - `skein/docs/design/oauth-issuer.md:421: section-self skein/docs/design/oauth-issuer.md 2.1`
- **R6,** a count after a path:
  - `temper/docs/design/draft/upgrades.status.md:7: section jig/docs/design/protocol/store.md 21`
- **R7,** a status file's step labels:
  - `temper/docs/design/draft/reference-case.status.md`, lines 15 to 29: 21 `status-label
    temper/docs/design/draft/reference-case.md N` findings, N in 11 to 28
- **R8,** a list item headed by a path:
  - `jig/docs/design/client/domain/README.md:38: section-self jig/docs/design/client/domain/README.md 18`
  - `temper/docs/design/draft/ci-on-forgejo-17.md:140: section-self temper/docs/design/draft/ci-on-forgejo-17.md 9`
  - `temper/docs/design/draft/ci-on-forgejo-17.md:142: section-self temper/docs/design/draft/ci-on-forgejo-17.md 13`
  - `temper/docs/design/draft/ci-on-forgejo-17.md:145: section-self temper/docs/design/draft/ci-on-forgejo-17.md 18`
- **R9,** links to tracked pages that are not Markdown, with a query:
  - `temper/docs/design/client/mockups/STATUS.md:84: path first-run.html`
  - `temper/docs/design/client/mockups/STATUS.md:119: path connect-model.html?world=first-run&step=5`

## allow-and-zero: not citations, and they stay (17)

- `jig/docs/development/quality-review.md:377: path agents/backlog.md`
- `skein/crates/skein-fake-forge-protocol/src/answers.rs:362: path ladder-b01/g2/report-2.md`
- `smith/docs/design/benchmarks.md:455: path CODEX_HOME/AGENTS.md`
- `temper/crates/temper-engine-domain-forge-landing/src/tests/key.rs:101: path crates.md`
- `temper/crates/temper/tests/support/csv_repair.rs:21: path ladder-b01/g1/report.md`
- `temper/crates/temper/tests/support/csv_repair.rs:22: path qa-focused/rung-b1-brief.md`
- `temper/docs/design/client/README.md:3133: path process.md`
- `temper/docs/design/domain/landing.md:304: path crates/x/README.md`
- `temper/docs/design/domain/testing.md:371: path changes/1.md`
- `temper/docs/design/draft/archive/first-run.md:7: path /src/rust/tmp/temper-first-time-ux-2026-10-20.md`
- `temper/docs/design/draft/configuring.md:329: path process.md`
- `temper/docs/design/draft/dashboards.md:271: path status-page.md`
- `temper/docs/design/draft/macos.md:6: path /src/rust/tmp/macos-survey-2026-10-22.md`
- `temper/docs/design/draft/notes.md:168: path status.md`
- `temper/docs/design/draft/notes.md:512: path landing-model.md`
- `temper/docs/design/draft/upgrades.md:7: path /src/rust/tmp/roadmap-3/store-upgrades.md`
- `temper/docs/help/landing.md:115: path crates/x/README.md`
