# debt-14: hearth's small items, smith's two

Two independent parts (`inputs/request.md`). This file grows with each push.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `ratchet` | `ratchet` | | medium | 30 | D011: six forms and the exemption in hearth's ratchet; step code's `while` goes; D011 deleted |
| `paths` | `paths` | `ratchet` | medium | 50 | D008: 22 of 25 `#[path]` go; `hearth-test-fixtures` created; D008 narrowed to 3, stays open |
| `digits` | `digits` | `paths` | high | 90 | D005: step code calls lib's digits, one mode function each way, `hex` in the kit; D005 deleted |
| `shapes` | `shapes` | `paths` | medium | 45 | D012: `SixLayers`, `ENTRY_RANGE_*`, `Outcome::text`, one cut-pipe table of six tests; D012 deleted |

Still to come: D017 and D018 (smith), once their surveys report.

## Critical path

`ratchet`, `paths`, `digits`: 170 minutes. `digits` and `shapes` run side by side after `paths`.

## Decisions

- **D008 stays open, narrowed to three.** The other three `#[path]`s are in the two `memory.rs` suites the request
  leaves alone, so its **Paid when** can't hold here. `paths` takes the 22 and rewrites the item. A `[[question]]` says
  so; nothing waits on it.
- **A kit, not a third copy.** The goldens' reader crosses crate roots, and the worlds can't reach each other without a
  new world edge, which D009 counts against. `hex` must reach `hearth-lib`'s tests, so it can't live in `hearth-lib`
  without shipping a fixture. Both go in `hearth/testing/hearth-test-fixtures`, which depends on no hearth crate, so a
  step crate's dev-dependency on it makes no cycle. `paths` creates it, so `digits` needs `paths`.
- **The six cut-pipe tests stay six.** `CUT_BATCHES_PER_CASE` splits them on purpose. D012's "one table" is a macro's
  table of the same six names, not one long test.
- **D005's 8.** `Lines::decimal`, `Oid::hex`, fake git's `hex`, the kit's `hex` and four references stay. That needs
  `hearth-fake-machine`'s two and `cache.rs`'s `hex` to go or be renamed, which D005's text doesn't spell out but its
  "about 8" requires.
- **`while` to `for`.** `document()` takes no limits and changing its signature isn't needed: the decoder's length is a
  u32, which bounds the walk.

## Overlaps

- `hearth-memory-1`, whose cut runs beside this: `hearth/scripts/old-forms.txt` (each lowers its own rows; fab recounts
  where merges meet), and `hearth/docs/design/domain/testing.md` and its status, in other sections.
- `shapes` and `paths` both touch `hearth/tests/render/tests/suite/oracle.rs`, so `shapes` needs `paths`.
- `digits` and `shapes` both touch `hearth-domain-render`'s tests, in different files.
- `Cargo.lock` and the root `Cargo.toml`: `paths` and `digits` add the kit, one after the other.
