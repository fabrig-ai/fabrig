+++
id = "builder-current"
part = "builder-lifecycles"
kind = "feature"
needs = ["builder-child", "builder-resolve", "builder-read"]
files = ["hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/crates/hearth-protocol-builder/src/keep.rs", "hearth/crates/hearth-protocol-builder/src/store.rs", "hearth/crates/hearth-protocol-builder/src/resolve.rs", "hearth/crates/hearth-protocol-builder/src/tests/fixtures.rs", "hearth/crates/hearth-protocol-builder/src/tests/fixtures/resolve.rs"]
effort = "high"
minutes = 105
+++

# hearth-protocol-builder: `Protocol` holds one current call's state

## Goal

Hearth D006's largest. `Protocol` holds `child`, `mode`, `keeping`, `resolving`, `ran` and `call`, each `Option`, and asks each in turn whether it's empty: 22 `self.` sites, and 28 `protocol.` lines in `lib.rs`. About -10 to +10 net lines.

The sketch binds; no public signature moves:

```rust
enum Current {
    Call(Call), Idle,
    Step(builder::build::Step),       // only if the check below passes
    Keeping(keep::Keep),
    Resolving(resolve::Resolve),
    Git { child: git::Child, mode: Mode },
}
```

`child` and `mode` already pair: both are set in one `down` or `finish` and taken together, so `spawn` returns the `Child`. `Mode`'s own `answer` and `ended` stay inside their variants.

Acceptance:

- **The check first.** `Request::Render` doesn't assert `resolving`, and `Run` asserts only `ran`. A reading has them exclusive, since render has one request at a time and a build runs after its one target's spec, but nothing proves it. On today's code, add a throwaway assert that no two of these are `Some` at once, wherever one is set, and run the tests. **If `Run` coexists with other work, `ran` stays its own field,** and the report says which case. Then `Current` holds the rest, and that meets **Paid when**, which names `Option`s that hold in turn. Remove the assert.
- `rg '^\s+(child|mode|keeping|resolving|ran|call): Option' hearth/crates/hearth-protocol-builder/src/lib.rs` reads nothing, or only `ran`, by the check.
- The tests in the note pass, with no assertion changed.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/builder-states.md`
- `hearth/docs/development/debt/D006-lifecycles-in-flags.md`
- `skein/docs/foundation/programming-model.md`, 5.4
- `hearth/docs/design/domain/render.status.md`, its 8.5 entries, which name `up`'s `!above.is_empty()` and `work_pending`, and stay true

## Owns and may change

- **Owns:** the files in the header: `keep.rs`, `store.rs` and `resolve.rs`'s tail in their `self.` sites, the fixtures in how they reach `resolving` and `mode`.
- **Names stay:** D001 cites `revision_stdout` and `up`.

## Traps

- **`keeping = None` runs on any store answer,** in `up` under `!above.is_empty()`. A naive `current = Idle` there destroys a live `Git` or `Resolving`. It clears only `Keeping`, and `resume_store` doesn't clear it at all. Keep that read as it is.
- **Resume's priority isn't one match.** Its order is call, progress, client, run, store, resolve, keep, git, so `Call` and `Step` can't be taken in one match ahead of the clients.
- **`work_pending` is effectively unpinned.** D001 records that removing its `told` guard survived 384 cases, so the tests may not catch a wrong truth table. Review it arm by arm against today's, keeping its three early returns, and say in the report that you did.

## Steps

1. The check; remove it.
2. `Current`, moving one `Option` at a time: `keeping`, `resolving`, `child` with `mode`, `call`, `ran`. Run the tests between them.
3. `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p hearth-protocol-builder --all-targets -- -D warnings`; the tests. Commit and push.

## Report

The common note's items; what the check found; whether a variant needed an `#[expect]`.
