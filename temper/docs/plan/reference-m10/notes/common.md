# reference-m10: standing rules

Every task of this plan names this note.

- **The trees:** each task stays in the tree its spec names; read `AGENTS.md`, then that
  tree's `AGENTS.md`. A change another tree's code needs beyond the compiler-forced lines the
  spec lists is a question.
- **The design is binding:** `temper/docs/design/protocol/configuration.md`, 13.9 (the
  reference's configuration) and the sections each spec names, as this plan wrote them. The
  reference is settings over the stock, never a stock of its own: change no stock default
  (`temper/docs/design/protocol/configuration.md`, 13.4) unless a spec says so.
- **The reference's files** are `temper/docs/examples/reference/` (its settings file, its
  landing member) and the repository's `.temper/`. Other plans and walks read them; keep their
  forms (`temper ctl settings apply`'s settings file; the document's `application.landing`
  member) and change only what a spec names. Text a spec gives whole is placed as given.
- **Charter instructions** are a JSON string whose paragraphs are joined by `\n\n`; check the
  file with `python3 -I -c 'import json,sys; json.load(open(sys.argv[1]))' <file>` after each
  edit.
- **A budget or timeout is the user's** (`skein/docs/foundation/testing-strategy.md`, 8.1): no
  task raises a spend, a period, a run's `time` or a test budget. Profile limits are sizes.
- **Not yours:** any `docs/design/`, drafts, `docs/help/` (a spec that gives help text gives
  it whole, and the task places it as given), `temper/docs/plan/`.
- **Never run a live story** (`live`, `live_github*` binaries, `--run-ignored`); the worlds
  are the fake forges.
- **Old-form counts only fall:** new text adds no old form (no "actual", "genuine",
  "originally"); run each touched tree's `scripts/old_forms.py`.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`, then
  `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package you
  changed, then their focused tests (`AGENTS.md`, 6).
- **No new store version, no new public item, no new limit name** beyond the spec's; a
  compiler-forced arm or field in your tree is always allowed and listed in the report.
