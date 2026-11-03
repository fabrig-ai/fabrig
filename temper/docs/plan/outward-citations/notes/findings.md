# The stricter checker's findings, by owner

Measured at `4c830578a7` (the plan's base) with the `strict-checker` task's C1 to C3 applied
and `allow.txt`'s "Evidence outside the repository" block removed: `--all` reads 29. Each is
listed as the checker prints it (`file:line: kind target (detail)`).

## strict-checker (13): the README's own text

Its examples of the forms the checker reads, now that `tools/**/*.md` is read, and its
example of another repository's document:

- `tools/citations/README.md:4: path x.md (rel-dead)`, twice
- `tools/citations/README.md:4: section-self tools/citations/README.md|AGENTS.md 5.3 (no-numbered-headings)`
- `tools/citations/README.md:19: path ../../a.md (rel-dead)`
- `tools/citations/README.md:22: path x.md (rel-dead)`, three times
- `tools/citations/README.md:22: section-self tools/citations/README.md 10 (no-numbered-headings)`
- `tools/citations/README.md:24: path x.md (rel-dead)`, twice
- `tools/citations/README.md:29: path docs/design/fab.md (outside)`
- `tools/citations/README.md:29: path x.md (rel-dead)`
- `tools/citations/README.md:30: path x.md (rel-dead)`

The line numbers move as the README is rewritten; the task's acceptance is by kind, file and
target.

## outward-wording (16): citations of documents kept outside the repository

Shown once the lab exception goes (C1, C2):

- `AGENTS.md:71: path docs/design/fab.md (outside)`
- `skein/scripts/browser.sh:2: path docs/design/landing-flow.md (code-dead)`
- `jig/scripts/browser.sh:3: path docs/design/landing-flow.md (code-dead)`
- `jig/scripts/live-sign-in.sh:8: path docs/design/build-infrastructure.md (code-dead)`
- `temper/scripts/proof-live.sh:7: path docs/design/build-infrastructure.md (code-dead)`
- `temper/scripts/proof-live-github.sh:7: path docs/design/build-infrastructure.md (code-dead)`
- `temper/scripts/proof-live-github-managed.sh:9: path docs/design/build-infrastructure.md (code-dead)`
- `temper/docs/design/draft/dashboards.md:10: path docs/design/status-page.md (rel-dead)`
- `temper/docs/design/draft/dashboards.md:198: path docs/design/status-page.md (rel-dead)`
- `temper/docs/design/draft/reference-case.execution.md:45: path fab/roles/cut-run.md (outside)`

Shown once `allow.txt` drops its "Evidence outside the repository" block:

- `skein/crates/skein-fake-forge-protocol/src/answers.rs:368: path ladder-b01/g2/report-2.md (code-dead)`
- `temper/crates/temper/tests/support/csv_repair.rs:21: path ladder-b01/g1/report.md (code-dead)`
- `temper/crates/temper/tests/support/csv_repair.rs:22: path qa-focused/rung-b1-brief.md (code-dead)`
- `temper/docs/design/draft/archive/first-run.md:7: path /src/rust/tmp/temper-first-time-ux-2026-10-20.md (root)`
- `temper/docs/design/draft/macos.md:6: path /src/rust/tmp/macos-survey-2026-10-22.md (root)`
- `temper/docs/design/draft/upgrades.md:7: path /src/rust/tmp/roadmap-3/store-upgrades.md (root)`

Also `outward-wording`'s, though no finding: `dashboards.md` lines 58 and 370 call the
status page's document "agents' `status-page.md`"; `allow.txt` excuses `status-page.md` there
as the test's data, and the line keeps that name without the possessive.

## What the measurement also showed

- Reading the root `DESIGN.md`, `tools/merge/README.md`, `tools/old_forms/README.md`,
  `tools/tiers/design.md` and `tools/tiers/design.status.md` finds nothing.
- Leaving every file under `docs/plan/` out finds nothing new and loses no finding (no
  `plan.toml` comment is broken today).
- With the lab's phrases checked, "its agents' questions (`domain/forge.md`, 8.11)" and the
  like, which the old exception skipped, resolve: they add no finding.
- C2's form fires on exactly `AGENTS.md:71`, `reference-case.execution.md:45` and
  `README.md:29`, and on none of the repository's 40-odd "a repository's", "the repository's",
  "its repository's" phrases before a path.
