+++
id = "store-remaps"
part = "store-rules"
kind = "feature"
needs = ["store-copy"]
files = ["jig/crates/jig-protocol-store/src/codec/settings.rs", "jig/crates/jig-protocol-store/src/codec/tasks.rs", "jig/crates/jig-protocol-store/src/codec/tasks/models.rs", "jig/crates/jig-protocol-store/src/codec/people.rs", "jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/models.rs", "jig/crates/jig-domain-core/src/settings/tests.rs", "jig/crates/jig-domain-tasks/src/llm.rs", "jig/crates/jig-domain-tasks/src/limits.rs", "jig/crates/jig-domain-tasks/src/value.rs", "jig/crates/jig-domain-tasks/src/stored.rs", "jig/crates/jig-domain-tasks/src/stored/models.rs", "jig/crates/jig-domain-tasks/src/questions.rs", "jig/crates/jig-domain-tasks/src/run.rs", "jig/crates/jig-domain-authority/src/value.rs", "jig/docs/design/protocol/store.status.md", "jig/docs/development/debt/D025-store-codec-domain-rules.md"]
effort = "high"
minutes = 80
+++

# jig: the catalogue's remaps are the domains'

## Goal

Pay `jig/docs/development/debt/D025-store-codec-domain-rules.md`. The codec keeps its translating and calls the owning domain for what a model's move or departure means (`temper/docs/plan/debt-20/notes/contracts.md`, 2). A row decodes to the same value and encodes to the same bytes.

Acceptance:

- `rg -n 'u8::MAX|fn remap|fn model_moves' jig/crates/jig-protocol-store/src/codec/settings.rs` finds nothing: `settings::remap` builds its own moves. The loop bodies of `codec/people.rs`'s `model_moves` and `codec/tasks/models.rs`'s `moves` call `settings::model_bit`. `rg -n 'fn remap|fn authority\b|fn members|fn proposal\b|fn amendment' jig/crates/jig-protocol-store/src/codec/tasks/models.rs` finds nothing.
- `rg -n 'b"(Nothing more will come|You owe your requester)' jig/crates/jig-protocol-store/src -g '!**/tests/**'` finds nothing.
- New tests in `jig-domain-core/src/settings/tests.rs` pin `remap`: a template's model that moved follows its name; one that left, or whose chosen effort left, reads `ModelEffort::GONE`; a named mask follows and loses what left; `Models::All` stays. `jig-domain-tasks`' pin `Limits::word_bytes` with a limit below and above each literal, and `Stored::remap_models` on a `Live` row's authority.
- By hand: make the left arm keep the old index (`remap`'s test goes red); use `min` for `max` in `word_bytes` (its test goes red). Restore each. `jig-protocol-store`'s tests pass with what they assert unchanged, and its golden rows don't move.
- The branch's net production lines are at most 0 (estimate: -30, the masks' loops and `moves`' lookups written once). The item is deleted, and the `5.1:` clause leaves `store.status.md`.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/contracts.md`, 2: binding
- `jig/docs/development/debt/D025-store-codec-domain-rules.md`
- `jig/docs/design/protocol/store.md`, 5.1 (its **Retained request identity** and **The codec translates; the domains decide** paragraphs)
- `jig/AGENTS.md`: no traits and no generic items in step code

## Owns and may change

- **Owns:** the files in `files`. `store.status.md`: only the `5.1:` clause.
- **`referenced` and `encode` in `codec/tasks/models.rs`** say which names a row keeps, which is translating, so they stay. If `referenced`'s helpers then read best beside `remap_models` because they walk the same fields, they may move as `Stored::named_models(&self) -> u64`; say so in your report. Either is fine.
- **A private literal becomes visible in its crate as needed:** `OWED_WORDS` and `NOTHING_AWAITED_WORDS` keep their homes and their words.
- **`jig-domain-core/src/runs/model_names.rs`** stays: it has other work (a missing name refuses) and other plans' files beside it.
- **`jig_domain_tasks::Limits` and the store's `wire::Limits`:** only `word_words: word_bytes(domain)`'s call changes.
- **Stay off** `jig-domain-core/src/{lib.rs,stored.rs,vocabulary.rs,effects.rs}`, `routing/**` and `reads/**`: other plans are changing them. `Models` and `ModelEffort` need nothing there.

## Steps

1. The domains' functions with their tests, leaves first: `Models::remap` (authority, tasks), `ModelEffort::GONE`, `Limits::word_bytes`, `Stored::remap_models`, `settings::{model_bit, remap}`.
2. The codec calls them; its private copies go. A `kept` list made once where a decoder remaps several values.
3. Break the rules by hand, as the acceptance says.
4. Delete the item and the status clause. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-store -p jig-domain-core -p jig-domain-tasks -p jig-domain-authority --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-protocol-store -p jig-store-world -p jig-domain-tasks -p jig-domain-authority`; `heavy cargo nextest run -p jig-domain-core --lib -E 'test(/settings/)'`. The citations check. Commit and push.

## Report

The common note's items; whether `referenced` moved; what you broke and what failed.

## Answers

2026-11-05. May `codec/settings.rs`'s `model_moves` go when the codec calls `settings::remap`, or must it stay with a role? It goes: option 1. `temper/docs/plan/debt-20/notes/contracts.md`, 2 binds `settings::remap(value, kept, names)`, which takes no map and builds the moves itself with `settings::model_bit`, so the helper has no caller left, and keeping it would be dead code. The first acceptance bullet above is amended: the settings codec's search finds nothing. `codec/people.rs`'s `model_moves` and `codec/tasks/models.rs`'s `moves` stay, with `settings::model_bit` in their loops; they hand a map to `Models::remap` and `Stored::remap_models`, which do take one. The codec makes one `kept` list per decoder (the `Revision` and `Proposal` decoders' loops call `remap` with the same list). The rest of the task stands: finish the migration from `debt-20/store-remaps`' pushed work, then delete the item and the `5.1:` clause, with the production-lines cap measured at the end.
