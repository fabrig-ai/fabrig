# configuring-p1: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it where its spec
says so.

- **The plan:** phase P1 of `temper/docs/design/draft/configuring.execution.md`: a charter's
  documents (jig's `jig/docs/design/domain/engine.md`, 17.9; temper's
  `temper/docs/design/protocol/configuration.md`, 13.7), each run reading its charter at its
  task's revision (jig's `engine.md`, 17.3; temper's `temper/docs/design/domain/agent.md`, 4.1
  and 4.6), and `chat start --template` (`temper/docs/design/client/tools.md`, 5.3). Build them
  generally, for any charter and any template: no charter or template of the reference's or
  of the configuring agent's (charter 3, template 3) is this plan's.
- **The trees:** each task stays in the tree its spec names; read `AGENTS.md`, then that
  tree's `AGENTS.md`. A change another tree's code needs beyond the lines its spec lists
  (a required field's name, type path or value; a compiler-forced arm) is a question.
- **The design is binding:** the sections each spec cites, as they stand on your base.
  Never edit `docs/design/`, drafts or `temper/docs/plan/`, except the status lines a spec
  names, each edited only as it says (an entry the task builds leaves the status).
- **Help pages are the lead's:** `temper/docs/help/` is compiled into `temper-docs` and read
  by its tests. A spec that gives help text gives it whole, and the task places it as given;
  write no other help text. Test goldens under a crate's `src/tests/golden/` are the task's,
  regenerated as that crate's tests say (`SKEIN_CODEGEN_WRITE=1`), then read hunk by hunk.
- **Old code, a reference never a plan:** the old branches the request names are not
  reachable from your slot. What of them a spec takes, it quotes or describes; write the
  rest afresh in today's shapes.
- **Step crates** follow their tree's `clippy.toml` (temper's: no `Vec`, no `String`;
  `Box<[T]>`, `skein_lib::List`/`Queue`, UTF-8 in `Box<[u8]>`) and
  `skein/docs/foundation/programming-model.md`, section 10; jig's step code has no traits and
  no generic items. Tests under the tree's `tests/clippy.toml`: one thread, one loop, a seed
  replays.
- **Never run a live story** (`live`, `live_github*` binaries, `--run-ignored`).
- **Old-form counts only fall:** new code adds no old form (no "actual", "genuine",
  "originally" in comments); run `python3 -I -B <tree>/scripts/old_forms.py` for each tree
  you touch.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`, then
  `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` naming every package you
  changed, then their focused tests and their direct dependents' in every tree
  (`AGENTS.md`, 6), and `python3 -I -B tools/citations/citations.py --base origin/main`.
- **No new store version, no new limit name, no new public item** beyond your spec's; a
  compiler-forced arm or field in your tree is always allowed and listed in the report.
- **Other plans are building beside this one** (`reference-m1`, `playground-decisions`):
  their branches touch some of the same files (jig's settings and client wire, temper's
  forge and ctl). Edit only the lines your task needs, keep your hunks small and local, and
  never reformat or reorder code your task does not change.
