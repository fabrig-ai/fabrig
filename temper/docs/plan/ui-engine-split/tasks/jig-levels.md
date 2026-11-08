+++
id = "jig-levels"
part = "engine-versions"
kind = "interface"
needs = []
files = ["jig/crates/jig-protocol-store/src/lib.rs", "jig/crates/jig-protocol-store/src/tests/patching.rs", "jig/docs/design/protocol/store.status.md"]
effort = "medium"
minutes = 25
+++

## Goal

jig's store gives a binary its patch levels, `levels`, as
`jig/docs/design/protocol/store.md`, 10.5 sketches it, so that
`temper-engine versions` can print them (the request; temper's
`temper/docs/design/protocol/README.md`, 4.2, `versions`). The function
exists inside the crate today as `patching::plan::binary`, crate-private.

Acceptance:

- `jig_protocol_store::levels(sets: &PatchSets) -> codec::component::Levels`,
  public at the crate's root, with a doc comment citing
  `protocol/store.md`, 10.5 and 5.3: the binary's patch level of each
  owner, `None` before its first patch. It returns what
  `patching::plan::binary` returns; `binary` stays for the crate's own
  callers.
- The table test `the_binary_is_before_baseline_only_without_any_kept_text`
  (`src/tests/patching.rs`) calls `crate::levels` in place of
  `plan::binary`, its rows unchanged, so the public function is the one
  held. Break `levels` once (return all `None`), see the `BASE` row fail,
  restore.
- In `jig/docs/design/protocol/store.status.md`, the **Not yet** 10.5, 14
  entry no longer names `levels` ("… `BackRefusal` or `levels`" becomes
  "… or `BackRefusal`"), its other words unchanged. This departs from the
  implementer's text, which closes `docs/design/` to you: the entry is
  this task's.

## Inputs

- `jig/docs/design/protocol/store.md`, 10.5 (the sketch's `levels` line)
  and 5.3 (the levels).
- `jig/AGENTS.md`.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 3.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`. jig names no temper crate: nothing outside
  `jig/` changes.
- Not yours: rehearsing and going back (10.5's other lines), which stay
  **Not yet**.

## Steps

1. The function and the test's call. `heavy cargo nextest run -p
   jig-protocol-store --lib -E
   'test(=tests::patching::the_binary_is_before_baseline_only_without_any_kept_text)'`;
   read the count (find the path with `--list` if it selects nothing).
   Break and restore as above.
2. The status entry.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`;
   `heavy cargo clippy -p
   jig-protocol-store --all-targets -- -D warnings`; `heavy cargo nextest
   run -p jig-protocol-store`.
4. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the failure the broken `levels`
showed.
