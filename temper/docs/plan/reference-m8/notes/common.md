# reference-m8: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it.

- **The trees:** each task stays in the tree its spec names; read `AGENTS.md`, then that
  tree's `AGENTS.md`. A change another tree's code needs beyond the compiler-forced lines the
  spec lists is a question.
- **The design is binding:** `temper/docs/design/domain/after-landing.md` as this plan wrote
  it (section 2's `branch`, N5: a tier follows the one branch it names, the repository's
  landing branch when it names none; a landing on a branch no tier follows is cleared as it
  lands). Its sections 7 (reds to goals, `repairs`), 8 (trailers), 9 (statuses and comments
  on the forge) and findings are not this plan's: build none of them, and leave no hook for
  them beyond what a spec names.
- **Old code, a reference never a plan:** `temper/docs/plan/reference-m8/notes/old/` holds
  patches of the old repositories' unlanded verdicts line, re-rooted to this repository's
  layout. A spec says which to read. The tree moved since (store version, test binaries
  regrouped under `tests/*/tests/suite/`, the limits regime, the forge domain's shape):
  take what fits today's code, write the rest afresh, and say in the report what you took.
  Never apply a whole patch blind; `git apply --3way` (or `git am --3way` with
  `--directory=<tree>` where the spec says) on a scratch commit, then read every hunk. The old
  code follows the landing branch alone; N5's `branch` replaces that wherever it appears.
  Old memory-figure and price regenerations, land-set pins and `Cargo.lock` hunks are never
  taken.
- **Step crates** follow `temper/clippy.toml` (no `Vec`, no `String`: `Box<[T]>`,
  `skein_lib::List`/`Queue`, UTF-8 in `Box<[u8]>`), skein's
  `skein/docs/foundation/programming-model.md`, section 10; tests under the tree's
  `tests/clippy.toml`, one thread, one loop, a seed replays.
- **Not yours:** any `docs/design/`, drafts, `docs/help/` (a spec that gives help text gives
  it whole, and the task places it as given), `temper/docs/plan/`.
- **Never run a live story** (`live`, `live_github*` binaries, `--run-ignored`); the worlds
  are the fake forges.
- **Old-form counts only fall:** new code adds no old form (no "actual", "genuine",
  "originally" in comments); the queue recounts `temper/scripts/old-forms.txt` and its
  siblings.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`, then
  `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package you
  changed, then their focused tests and their direct dependents' (`AGENTS.md`, 6).
- **No new store version, no new limit name, no new public item** beyond the spec's; a
  compiler-forced arm or field in your tree is always allowed and listed in the report.
