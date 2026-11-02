# reference-m7a: standing rules

Every task of this plan names this note and `notes/contracts.md` beside it, whose shapes are
binding: names, fields and types as written there.

- **The trees are jig and temper.** Read `AGENTS.md`, then `jig/AGENTS.md` (every jig crate
  is step code under the strict subset: no traits and no generic items of its own; the core
  never links smith; no system is named in jig's code) and, for a temper task,
  `temper/AGENTS.md`.
- **Not yours:** any `docs/design/`, `docs/help/`, draft or plan file. What the code taught,
  and any place where the design and the code disagree, goes in the report.
- **Schemas change in place.** Pre-release, every family keeps its version number
  (`jig/docs/design/protocol/README.md, 7`; `jig/docs/design/protocol/store.md, 5`): change the
  `.schema` file, then regenerate its generated code, `layouts.rs` where the family has one, and
  its golden bytes with `SKEIN_CODEGEN_WRITE=1` on the family's drift test (the store's and
  the client's: `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p
  jig-store-codecs-world -E 'test(drift::)'`; the worker wire's: `-p jig-worker-wire-world -E
  'test(drift::)'`), never by hand. Then run the drift tests again without the variable, and
  read `git diff --stat`: only the family's schema, generated files and goldens move, and a
  new arm's goldens are new files (`*_full.bin`, `*_smallest.bin`, as `no_change`'s in
  `golden/tasks/`). No version constant (`VERSION`, `PAYLOAD_VERSION`, a family's number)
  changes.
- **New arms are appended** after a schema enum's last, so existing tags keep their numbers.
- **Forced arms.** A new variant forces arms in every exhaustive match; the implementer's
  text allows them in its tree. Where a spec lets a task change another tree's files for its
  forced arms, it names them. Where an arm's behaviour belongs to a later task, write the
  total arm the spec gives; never `panic!`, `unreachable!`, `todo!` or a wildcard `_` arm
  that would swallow the next variant. A translation stays exhaustive both ways, with no
  wildcard (`jig/docs/design/protocol/store.md, 5.1`).
- **No new public items** beyond the spec's and the contracts', no new crates but the one
  `tooling-capability` makes, no new external dependencies.
- **Tests** are ordinary Rust under the tree's `tests/clippy.toml` (one thread, one loop, a
  seed replays); step crates' unit tests follow the crate's own idiom. Every new story cites
  the design section it checks in its doc comment.
- **Old-form counts only fall** (`jig/scripts/old-forms.txt`, `temper/scripts/old-forms.txt`):
  no new comment says "actual" or "genuine".
- **Citations:** code cites a design section at module or type level, by path from the root
  (`jig/docs/design/domain/tasks.md, 5.3.1`). Before pushing, run
  `python3 -I -B tools/citations/citations.py --base origin/main`; it must report no new
  broken citation.
- **Every build and test through `heavy`,** one at a time; the narrowest test first, and
  check the count it selected. No live service, no real repository.
- **Docs read by tests:** `jig/docs/design/protocol/store.md` is compiled into
  `jig-protocol-store`'s words tests (`src/tests/words.rs`); a task that changes the store
  family runs them (`heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/words/)'`).
