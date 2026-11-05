+++
id = "hearth-path"
part = "hearth-path"
kind = "feature"
needs = []
files = ["hearth/tests/config/tests/suite/memory.rs", "hearth/tests/config/tests/suite/memory/**", "hearth/tests/config/src/fixtures/**", "hearth/scripts/old-forms.txt", "hearth/docs/development/debt/D008-path-modules.md"]
effort = "medium"
minutes = 15
+++

# hearth: the config memory suite holds its own fixtures

## Goal

Pay `hearth/docs/development/debt/D008-path-modules.md`: the last two `#[path]`s in hearth become plain modules of their
suite. The item's closing lines say these wait for hearth's memory migration; the request sets that aside. Neither file
names `hearth-service-builder` or a `MAX_OUT` form, so nothing here needs them.

Acceptance:

- `git grep -E '#\[path' -- 'hearth/*.rs' | wc -l` reads 0.
- `hearth/tests/config/src/fixtures/` is gone; `hearth/tests/config/src/fixtures.rs` and its `pub mod fixtures;` stay.
- `hearth/scripts/old-forms.txt` reads `path_attribute 0`.
- Both tests of `suite/memory.rs` pass with the same ceilings, the same document counts and the same assertions.
- `hearth/docs/development/debt/D008-path-modules.md` is deleted.

## Inputs

- `temper/docs/plan/debt-17/notes/common.md`
- `hearth/docs/development/debt/D008-path-modules.md`
- `hearth/docs/design/domain/testing.md`, 2.4

## Owns and may change

- **Owns:** `hearth/tests/config/tests/suite/memory.rs`, the new `hearth/tests/config/tests/suite/memory/`, and the two
  files that move out of `hearth/tests/config/src/fixtures/`.
- **May change:** `hearth/scripts/old-forms.txt`, by `old_forms.py --write`.
- If the suite's `limits` module name is taken or reads badly in `memory.rs`, keep the sketch's shape under a name of
  your choosing and say so in the report.
- If a move would need `hearth-service-builder/src/lib.rs` or a machine's `MAX_OUT` form, it doesn't here. Should that
  change, leave that `#[path]`, say why beside it, and report it; the item stays then with the reason.

## Steps

1. `git mv` so history follows:

   - `tests/config/src/fixtures/metered.rs` to `tests/config/tests/suite/memory/measured.rs`
   - `tests/config/src/fixtures/memory.rs` to `tests/config/tests/suite/memory/limits.rs`

2. `suite/memory.rs` opens:

   ```rust
   use hearth_config_world::fixtures;

   mod limits;
   mod measured;
   ```

   Its calls read `limits::JSON`, `limits::DECLARATION`, `measured::metered` and `measured::metered_declaration`. The
   `#[expect(clippy::module_inception, ...)]` goes with the name that needed it.

3. `measured.rs`'s first line says what the module is now, as a plain module of the suite (its "included by the
   integration test" goes); `skein-heap` stays a dev dependency of `hearth-config-world`.

4. `heavy cargo nextest run -p hearth-config-world --test suite -E 'test(/memory/)'`. Check the count: two tests.

5. `cargo fmt`; from `hearth/`, `python3 -I -B scripts/old_forms.py --write`; `heavy cargo clippy -p hearth-config-world
   --all-targets -- -D warnings`; delete the debt item; the citations check. Commit and push.

## Report

The common note's items. A move: nothing to break by hand.
