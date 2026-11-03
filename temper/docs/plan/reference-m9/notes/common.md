# reference-m9: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it.

- **The trees:** each task stays in the tree its spec names; read `AGENTS.md`, then that
  tree's `AGENTS.md`. A change another tree's code needs beyond the compiler-forced lines the
  spec lists is a question.
- **The design is binding:** `temper/docs/design/domain/after-landing.md`, sections 1 to 3,
  5.3, 6, 7 and 12, and `temper/docs/design/domain/agent.md`, 4.3.1 (`repairs`), as this plan
  wrote them. A tier following goal branches judges no landing; a red's news goes to its goal;
  a change naming a red landing repairs it. A red of no goal's (findings), trailers (section 8)
  and the forge's statuses and comments (section 9) are not this plan's: build none of them,
  and leave no hook for them beyond what a spec names.
- **Other plans write the same files at once** (reference-m3, reference-m5, reference-m6 and
  reference-m7b in `forge_route.rs`, `calls.rs`, the store's `store-v11.schema` and
  `temper-engine-domain-forge/src/domain.rs`): add your part beside theirs, never reorder or
  rewrite theirs, and keep each edit to the lines your spec names. A conflict at fab's merge is
  fab's, not yours.
- **Old code, a reference never a plan:** where a spec quotes the old verdicts line (temper
  `verdicts/t9-bound`), take what fits today's code and write the rest afresh; say in the
  report what you took. The old code follows the landing branch alone and blames on main
  only.
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
