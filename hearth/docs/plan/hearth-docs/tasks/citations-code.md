+++
id = "citations-code"
part = "citations"
kind = "migration"
needs = []
files = ["hearth/crates/**/*.rs", "hearth/tests/**/*.rs", "hearth/testing/**/*.rs", "hearth/testing/mirror/fetch", "hearth/testing/mirror/pin", "hearth/scripts/fail_stop.py", "hearth/scripts/old_forms.py"]
effort = "medium"
minutes = 50
+++

# hearth's code cites by the path from the root

## Goal

hearth's code cites its documents by their path from `hearth/docs/design/` and skein's by bare name; the repository's rule is the
path from the root (`AGENTS.md`, 1). A script rewrites about 480 citations in comments and `reason` strings and about 80 in test
messages, in about 330 `.rs` files, and what hearth prints stays.

Acceptance:

- Outside `hearth/docs/`, this prints only what `notes/contracts.md`, 2.2 keeps:

  ```sh
  git grep -n -E '(^|[^/[:alnum:]_.-])((domain|protocol|client|draft)/[[:alnum:]_.-]+\.md|programming-model\.md|testing-strategy\.md|(io|lib|kernel|shell)\.md|Format\.md)' -- hearth ':!hearth/docs'
  ```

- `git diff --stat` against your base shows no change to `hearth/tests/render/errors/**`, or to any `.json`, and every changed line
  of `hearth/crates/hearth-client-protocol-records/src/errors/reserved.rs` and `errors/repository.rs` is a comment.
- A comment line is at most 120 columns, and `cargo fmt --check` is clean.
- hearth's 30 packages pass clippy and nextest; `python3 -I -B tools/citations/citations.py --all` reads 0; `outward` passes;
  `python3 -I -B hearth/scripts/old_forms.py` raises no count.

## Inputs

- `hearth/docs/plan/hearth-docs/notes/common.md`
- `hearth/docs/plan/hearth-docs/notes/contracts.md`, 2
- `hearth/docs/plan/hearth-docs/inputs/request.md`, 3
- `tools/citations/README.md`: what the checker reads

## Owns and may change

- **Owns:** the citations in the files under `files`, and nothing else of them.
- **Not to touch:** `hearth/docs/**`, `hearth/AGENTS.md`, `hearth/README.md` (other tasks own them); `clippy.toml`, `Cargo.toml`
  and every JSON and golden (section 2.2 keeps what they hold).
- **Options:**
  - A string outside section 2.2's list that a test fails on is a printed string the list missed: restore it, and add it to your
    report.
  - `decisions.md, 13` in `hearth/tests/conformance/src/fake_git.rs` names no document in the repository: leave both, and report them.
  - If `shell.md, 6` in `hearth/crates/hearth/src/lib.rs` isn't skein's "Startup" on reading it, report it and leave it.
  - A trailing comment that passes 120 columns moves to its own line above.

## Steps

1. Write an untracked script over `git ls-files hearth`, leaving out `hearth/docs/`, Markdown, TOML and JSON. It maps each token by
   `notes/contracts.md`, 2.1 and rewrites:
   - in `.rs`, comments and every string, but for the lines section 2.2 keeps. In the kept files only comment lines change, and
     bare `README.md` in `.rs` is always a file name;
   - in `hearth/testing/mirror/fetch` and `pin`, the `#` comment lines;
   - in `hearth/scripts/fail_stop.py` and `old_forms.py`, every "skein's `programming-model.md`" or "skein's
     docs/foundation/programming-model.md" becomes `skein/docs/foundation/programming-model.md`.

   Strings in `.rs` are assertion messages, `.expect(...)`, `panic!`, `#[expect(reason = ...)]` and the `reason:` of
   `hearth/tests/conformance/src/*.rs` and `hearth/tests/render/tests/suite/oracle/engine.rs`: test text, never printed by hearth.
2. Have the script report every token it left, by file, and read that list against section 2.2.
3. Rewrap a comment line past 120 columns, that paragraph alone; `cargo fmt`.
4. Read `git diff --stat`, then a sample of each kind: comment, `#[expect]`, assertion message, `reason:`, wrapped line.
5. Delete the script, commit (`hearth: cite documents by their path from the root`), and run clippy and nextest over the 30
   packages, then the checks in `notes/common.md`, then `old_forms.py`. Push.

## Report

Tokens rewritten by kind and file count; every token left, grouped by why; comments rewrapped; test counts.
