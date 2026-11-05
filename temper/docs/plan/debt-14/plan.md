# debt-14: hearth's small items, smith's two

Two independent parts (`inputs/request.md`). Five items are paid and deleted here. D008 stays open at three.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `ratchet` | `ratchet` | | medium | 30 | D011: six forms and the exemption in hearth's ratchet; step code's `while` goes; D011 deleted |
| `paths` | `paths` | `ratchet` | medium | 50 | D008: 22 of 25 `#[path]` go; `hearth-test-fixtures` created; D008 narrowed to 3, stays open |
| `digits` | `digits` | `paths` | high | 90 | D005: step code calls lib's digits, one mode function each way, `hex` in the kit; D005 deleted |
| `shapes` | `shapes` | `paths` | medium | 45 | D012: `SixLayers`, `ENTRY_RANGE_*`, `Outcome::text`, one cut-pipe table of six tests; D012 deleted |
| `worlds` | `worlds` | | high | 120 | D017: four tests move to protocol worlds; fakes of decode, path feedback and listing, held to the code; D017 deleted |
| `words` | `words` | | high | 120 | D018: `Notice::Turn`, `Notice::Answer`, `Failure::EffortRefused`; every word in `terminal.rs`; D018 deleted |

## Critical path

`ratchet`, `paths`, `digits`: 170 minutes. `digits` and `shapes` run side by side after `paths`. smith's two start at
once, 120 minutes each, and share no file with each other or with hearth.

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

- **D017 is paid, not kept.** The survey found no use the item's fallback needs: four tests are in the wrong tier, and
  what's left is about 140 lines of fake and 100 of equivalence test. That is Size M, not the item's S, and it fits one
  task. The equivalence tests aren't optional: the fakes' bytes reach token counts and traces.
- **`tests/agent` and `tests/run` keep their protocol edges.** The item names neither, and the status file records
  neither. `worlds` rewrites the **Departs** entry to name them, so it stays true, with no debt item behind it. So
  `STATUS.md`'s "Two domain worlds call protocol crates" stays true too, which also keeps `worlds` off the sentence `words`
  edits.
- **The design decides D018.** `host.status.md` left "whether this departs" to the designer. `domain/host.md`, 8 now says
  the local domain holds none of the person's words, and the entry moves to **Not yet**, for `words` to close.
- **`Request::Show` and friends aren't a public interface.** `Request::Show`, `Older::words`, `Domain::result_text` and
  `Service::effort_notice` go, and `Domain::tool_verdict` changes its type. They are `pub` for smith's own crates, the
  workspace isn't published, nothing outside smith names them, and D018's **Paid when** can't hold with them. No stored
  or wire format changes. Earlier plans changed such items the same way.
- **Turn lines go through `Notice`, for one route.** That costs a world arm and the service's arms; a typed `Show` would
  be smaller but leave D018's two routes.
- **`Older::words` is in `words`.** D018 doesn't name it; its **Paid when** requires it.
- **The effort is read earlier.** `Failure::EffortRefused` snapshots it when the answer settles; the shell read it when
  it drained. They can differ only if the shell's drain lags past a `Loaded` that applies a pending `/effort`. The sent
  charter's effort is what the words mean, so the snapshot is the truer of the two. No test shows either.

## Overlaps

- `hearth-memory-1`, whose cut runs beside this: `hearth/scripts/old-forms.txt` (each lowers its own rows; fab recounts
  where merges meet), and `hearth/docs/design/domain/testing.md` and its status, in other sections.
- `shapes` and `paths` both touch `hearth/tests/render/tests/suite/oracle.rs`, so `shapes` needs `paths`.
- `digits` and `shapes` both touch `hearth-domain-render`'s tests, in different files.
- `Cargo.lock` and the root `Cargo.toml`: `paths` and `digits` add the kit, one after the other.
