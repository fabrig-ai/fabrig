# reference-m7b: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it where it builds a
shape that crosses tasks: names, fields and types there are binding.

- **The trees are jig and temper.** Read `AGENTS.md`, then `jig/AGENTS.md` for a jig task
  (every jig crate is step code under the strict subset; the core never links smith; no system
  is named in jig's code) and `temper/AGENTS.md` for a temper task.
- **Not yours:** any `docs/design/`, `docs/help/`, draft or plan file, unless the spec names
  one. What the code taught, and any place where the design and the code disagree, goes in the
  report.
- **Schemas change in place.** Pre-release, every family keeps its version number
  (`jig/docs/design/protocol/README.md, 7`; `temper/docs/design/protocol/store.md, 2`): change
  the `.schema` file, then regenerate its generated code and its golden bytes with
  `SKEIN_CODEGEN_WRITE=1` on the family's drift test, never by hand; run the drift tests again
  without the variable, and read `git diff --stat`: only the family's schema, generated files
  and goldens move. New arms are appended after a schema enum's last. No version constant
  changes.
- **Forced arms.** A new variant forces arms in every exhaustive match; the implementer's text
  allows them in its tree. Where an arm's behaviour belongs to a later task, write the total
  arm the spec gives; never `panic!`, `unreachable!`, `todo!` or a wildcard `_` arm that would
  swallow the next variant.
- **No new public items** beyond the spec's and the contracts', no new crates, no new external
  dependencies.
- **Tests** are ordinary Rust under the tree's `tests/clippy.toml` (one thread, one loop, a
  seed replays); step crates' unit tests follow the crate's own idiom. Every new story cites
  the design section it checks in its doc comment.
- **Old-form counts only fall** (`jig/scripts/old-forms.txt`, `temper/scripts/old-forms.txt`):
  no new comment says "actual" or "genuine".
- **Citations:** code cites a design section at module or type level, by path from the root
  (`temper/docs/design/domain/forge.md, 8.4`). Before pushing, run
  `python3 -I -B tools/citations/citations.py --base origin/main`; it must report no new
  broken citation.
- **Every build and test through `heavy`,** one at a time; the narrowest test first, and check
  the count it selected. No live service, no real repository.
- **Docs read by tests** (`.config/gate/read-by-tests.txt`): `temper/docs/help/**` is compiled
  into temper's help tests; a task that changes a help page runs them.

## Corrections found after release (revision R2 of `notes/contracts.md`)

- **The store's drift test** is in `temper-codecs-world`, not `temper-protocol-store`:
  `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper-codecs-world --test suite -E
  'test(=drift::store_schema_matches_generated_code_and_goldens)'`, then the same without the
  variable (`change-merge-surface`).
- **`merge-kind`'s names:** the second place the service builds the core's configuration is
  `temper/crates/temper-service/src/startup.rs` (`let core = config::core(..)`, which
  `checks::admission` then reads), not `startup/checks.rs`; `tooling::Kind::verb` is a `Box<[u8]>` (`Box::from(&b"merge"[..])`);
  `whole/jobs.rs` fixes its kind at `JobKind(9)`, only `delivers` a parameter, so the merge
  stories take the kind as a parameter of their own or a world of their own. Its bounds are
  R2's.
- **`merge-settings`:** a project's way is kept in the forge's `landing.rs`
  (`landing::State::projects`), so `merge_job` is kept there beside it; `ProjectLanding`
  literals are three (the forge's `landing.rs`, `temper/tests/engine/domain/src/forge_routes.rs`
  and `temper-engine-domain/src/route/forge_route/tests.rs`), each `merge_job: None`.
- **`merge-help`'s text does not fit:** `temper/docs/help/change.md` is 8,177 bytes against
  temper-docs' 8,192-byte bound for a concept page. In place of its three replacements, apply
  these two, verbatim (8,182 bytes):
  1. replace
     ```
       it waits for, by task number or a `name` from the same call) and
       `changes` (several at once, all or none).
     ```
     with
     ```
       it waits for, by task number or a `name` from the same call),
       `changes` (several at once, all or none) and `merge` (a catch-up's
       commit).
     ```
  2. replace
     ```
       forge, which merges the base in, where its rules ask up to date;
       else a conflicting change is held, saying so;
     - `resolving`: an agent resolves a conflict with the base and pushes
       the merge commit;
     ```
     with
     ```
       forge, which merges the base in, where its rules ask up to date;
     - `resolving`: the merge job, then an agent, settles a conflict with
       the base and pushes the merge commit;
     ```

