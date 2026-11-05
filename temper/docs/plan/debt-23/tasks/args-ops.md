+++
id = "args-ops"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/examples/ops/domain-infrastructure/src/application.rs", "jig/examples/ops/domain-infrastructure/src/domain.rs", "jig/examples/ops/domain/src/client_documents.rs", "jig/examples/ops/domain/src/route.rs"]
effort = "medium"
minutes = 25
+++

# ops: three helpers take their arguments folded

## Goal

Part of jig D016: `jig/examples/ops`' three sites lose their `too_many_arguments` exception, by `notes/arguments.md`: `domain-infrastructure/src/application.rs`'s `decide`, which drops `_person` and `_project`, as nothing reads them (`person: _, project: _` at its caller), and folds (token, owner); `domain/src/client_documents.rs`'s `document` and `read`.

Acceptance:

- `rg -o 'too_many_arguments' jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` reads 0.
- `heavy cargo nextest run -p jig-ops-domain --lib`, `-p jig-ops-domain-infrastructure --lib` and `-p jig-ops-world` pass, with no assertion changed.
- About +10 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the files in the header.
- **`client_documents::read`'s `too_many_lines`** exception is adjacent: see the note.

## Steps

1. `decide`; then the two in `client_documents.rs`.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-ops-domain -p jig-ops-domain-infrastructure --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items.
