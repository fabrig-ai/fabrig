+++
id = "args-people-views"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-people/src/domain.rs", "jig/crates/jig-domain-people/src/domain/inbox.rs", "jig/crates/jig-domain-people/src/domain/start.rs", "jig/crates/jig-domain-people/src/domain/parties.rs", "jig/crates/jig-domain-people/src/domain/requests.rs", "jig/crates/jig-domain-views/src/text.rs", "jig/crates/jig-domain-views/src/watch.rs", "jig/crates/jig-domain-views/src/domain.rs"]
effort = "medium"
minutes = 45
+++

# jig-domain-people and jig-domain-views: each helper takes its arguments folded

## Goal

Part of jig D016: people's four sites (`domain/inbox.rs`'s `chat_state_event`, `project_chat`; `domain/parties.rs`'s `signin`; `domain/requests.rs`'s `admit_ask`) and views' three (`text.rs`'s `written`, `input`; `watch.rs`'s `open`) lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds.

Acceptance:

- `rg -o 'too_many_arguments' jig/crates/jig-domain-people/src jig/crates/jig-domain-views/src -g '!**/tests/**' -g '!**/tests.rs' | wc -l` reads 0.
- `heavy cargo nextest run -p jig-domain-people --lib`, `-p jig-people-world`, `-p jig-domain-views --lib` and `-p jig-client-views-world` pass, with no assertion changed.
- About +16 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header.
- **`inbox::State`** becomes `pub(super)`, built by the two callers. It isn't a stored form or a public one. A field the body reads after moving another is read first.
- **`signin`** holds `too_many_lines` in the same `#[expect]`: see the note.
- **Stays off:** `jig-domain-people/src/batch.rs`, `limits.rs`: other drafts hold them, and no site here needs them.

## Steps

1. People; its tests.
2. Views; its tests.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-people -p jig-domain-views --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
