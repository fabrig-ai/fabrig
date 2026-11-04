+++
id = "hearth-profiles"
part = "hearth"
kind = "migration"
needs = ["lint-check"]
files = ["hearth/clippy.toml", "hearth/tests/**", "hearth/testing/**", "hearth/crates/hearth/**", "hearth/AGENTS.md"]
effort = "medium"
minutes = 30
+++

# hearth: every package on a profile, every `clippy.toml` a variant

## Goal

The first tree onto the profiles, and the smallest: 14 manifests and 8 `clippy.toml` files. It proves the check
and the definitions on real crates, with hearth's binary on `ring`. hearth's item stays until `lint-gate`
deletes it, with the others'.

Acceptance:

- `python3 -I -B tools/lints/lints.py hearth` reads 0 findings.
- `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` passes for every hearth package whose manifest or
  `clippy.toml` you changed, and for the crates each inherits for.
- `rg -n '^disallowed_\w+ = "(allow|warn)"' -g Cargo.toml hearth` finds nothing.

## Inputs

- `temper/docs/plan/debt-11/notes/common.md`
- `temper/docs/plan/debt-11/notes/contracts.md`, 2, 3 and 4
- `tools/lints/design.md`, 3, 4 and 5
- `hearth/docs/development/debt/D010-lint-config-copied.md`

## Owns and may change

- **Owns:** hearth's 14 manifests with their own block (the contracts' section 2) and its `clippy.toml` files.
- **May change:** code in those packages, as far as `notes/common.md`'s list says. `hearth/AGENTS.md`: line 7's
  "declare their own lints" becomes "take a lint profile (`tools/lints/design.md`, 3)", and nothing else.
- **Options:**
  - `hearth-test-mirror`'s `clippy.toml` differs from `hearth/tests/`'s by leaving `thread::spawn` out, because
    its loopback proxy runs on a thread of its own and conformance does not replay. It takes `ordinary`, and its
    one `spawn` takes a scoped `#[expect(clippy::disallowed_methods, reason = "...")]` saying that.
  - If `--set` deleted a comment that spoke of something other than lints, put it back.

## Steps

1. ```sh
   python3 -I -B tools/lints/lints.py --set harness hearth/tests/*/Cargo.toml hearth/testing/hearth-fake-git/Cargo.toml \
       hearth/testing/hearth-fake-mmdebstrap/Cargo.toml hearth/testing/hearth-recordings/Cargo.toml \
       hearth/testing/hearth-test-mirror/Cargo.toml
   python3 -I -B tools/lints/lints.py --set ring hearth/crates/hearth/Cargo.toml
   ```

   `hearth-fake-machine` is on `step` and stays there.
2. Copy the variants by the contracts' section 4: `cp tools/lints/clippy/NAME.toml DIR/clippy.toml`.
3. Clippy, then what it turns red, by `notes/common.md`. Cargo tracks a `clippy.toml` a crate read; if a result
   looks stale, `heavy cargo clean -p PKG` in your slot.
4. `hearth/AGENTS.md`. Fmt, old forms, the check, citations.
5. Tests: `heavy cargo nextest run -p PKG` for each package whose code changed beyond a deleted or added attribute.
   A lint block changes no behaviour.
6. Commit and push.

## Report

The report of your implementer text, and: each level or ban you saw move, against the contracts' sections 3 and 4;
the exceptions you added, by crate; the branch's net lines. Say anything the definitions made hard: later trees
stand on them, and a later task can't change them.
