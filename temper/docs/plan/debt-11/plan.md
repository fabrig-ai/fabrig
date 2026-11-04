# debt-11: one lint regime across the trees

Pays `jig` D019, `skein` D036, `temper` D011 and `hearth` D010, as one job (`inputs/request.md`).

## Parts and tasks

| Part | Task | Needs | Pays |
|---|---|---|---|
| `check` | `lint-check` | | the definitions and `tools/lints/lints.py` |
| `manual-map` | `map-quiet` | | temper's crates no draft is changing |

Written so far; the rest follows in later pushes: each tree's move onto the profiles, the rest of
`manual_map`, and the gate.

## Critical path

`lint-check` → each tree's move → `lint-gate`, which turns the check on and deletes the four items.

## Decisions

- **Four profiles, not three.** `binary` held two regimes (`tools/lints/design.md`, 2.3 and 2.4), and the
  split moves no level. Open to the user as question `fourth-profile`; nothing waits on it.
- **The check is on last.** Each tree's task runs it on its own paths; `lint-gate` adds it to the gate once
  the four trees read 0, so no branch is red in between.
- **The block measures stay above 0.** D019's, D036's and D010's measures count manifests with a block of
  their own. The request has each manifest keep its block, so the item's target is `lints.py` at 0 findings.
- **One branch deletes all four items.** Each cites the others, so deleting them apart would break
  `citations.py --all`. `lint-gate` does it.
- **smith and the root `tests/boundary` stay out:** the request names four trees.
