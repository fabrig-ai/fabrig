# debt-11: one lint regime across the trees

Pays `jig` D019, `skein` D036, `temper` D011 and `hearth` D010, as one job (`inputs/request.md`). The design is
`tools/lints/design.md`; the literal definitions and this base's map are `notes/contracts.md`.

## Parts and tasks

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `check` | `lint-check` | | high | 60 | the definitions, `tools/lints/lints.py` and its tests |
| `manual-map` | `map-quiet` | | medium | 45 | root expectations in the temper crates no draft is changing |
| `hearth` | `hearth-profiles` | `lint-check` | medium | 30 | 14 manifests, 8 `clippy.toml`: proves the definitions first |
| `jig` | `jig-profiles` | `lint-check` | medium | 60 | 41 manifests, 22 `clippy.toml` |
| `skein` | `skein-worlds` | `lint-check` | medium | 60 | the test crates near `harness` already |
| `skein` | `skein-raised` | `lint-check` | high | 90 | seven crates `harness` raises, two with no pedantic |
| `skein` | `skein-shells` | `lint-check` | high | 75 | `ring` and `binary`, with `skein-ui-web`'s `_` arms |
| `skein` | `skein-clippy` | the three above | high | 60 | 27 `clippy.toml` to three variants |
| `temper` | `temper-profiles` | `lint-check` | medium | 75 | 31 manifests, 40 wildcard expectations go |
| `temper` | `temper-clippy` | `temper-profiles` | high | 60 | 18 `clippy.toml` to two variants, and one for the tools |
| `manual-map` | `map-busy` | `map-quiet`, `temper-clippy` | medium | 45 | the crates other drafts touch; D011's measure reaches 0 |
| `gate` | `lint-gate` | all of the above | high | 45 | the check goes on, the four items go, the level table |

## Critical path

`lint-check` → `temper-profiles` → `temper-clippy` → `map-busy` → `lint-gate`: 285 implementer-minutes. skein's path is
255: `lint-check` → `skein-raised` → `skein-clippy` → `lint-gate`. `hearth-profiles`, the smallest, is the meeting
point: it reads back on the definitions before the larger trees finish.

## Decisions

- **Four profiles, not three.** `binary` held two regimes (`tools/lints/design.md`, 2.3 and 2.4). One block would
  loosen the ring's shell or raise twelve crates by hundreds of sites, and `harness` would allow wildcards in code a
  user runs. Open to the user as question `fourth-profile`; nothing waits on it.
- **Three `clippy.toml` variants** for four profiles: `binary` shares `ordinary` with `harness`. A variant is chosen
  apart from a profile, as `skein-sim` shows, on `step` with `ring`'s bans. 14 semantic variants become 3.
- **`testing/` keeps per-crate copies.** D036 proposed one `testing/clippy.toml`, but step crates sit there beside the
  kits, and clippy would have handed them the looser bans silently. Copies are held to one text by the check instead.
- **The definitions are text, compared byte for byte,** so `--write` makes a change of policy one edit. That answers
  D036's "a change of policy is 48 edits", at about 120 lines of standard-library Python.
- **The check goes on last.** Each tree's task runs it on its own paths; `lint-gate` adds it to the gate once the four
  trees read 0, so no branch is red in between.
- **The block measures stay above 0.** D019's, D036's and D010's measures count manifests with a block of their own.
  The request has each manifest keep its block, so each item's target is `lints.py` at 0 findings.
- **One branch deletes all four items.** Each cites the others, so deleting them apart would break
  `citations.py --all`. `lint-gate` does it, and temper's item stays open until then.
- **A boundary test runs the check,** so the gate holds it from inside fabrig. The queue's precheck is outside this
  repository; adding the line there is open as question `precheck-line`.
- **smith and the root `tests/boundary` stay out:** the request names four trees (`tools/lints/design.md`, 7).
- **`matches!`** needs no change: D2 asked to allow it in test crates, and `ordinary` bans no macro.

## Overlaps with the drafts beside this one

Avoided by order: `map-busy` comes after the tree tasks, so the crates `debt-10`, `serve-without-box` and `look-round-3`
are changing are touched as late as this plan allows, and in attribute lines alone.

Not avoided:

- `map-busy`: `temper-service/src/engine.rs`, `system.rs`, `clients.rs`; `temper-engine-domain/src/domain.rs`,
  `route/forge_route.rs`; `temper-client-domain/src/route.rs`; `temper/tools/gallery/src/worlds.rs`.
- `skein-raised`: `skein/tests/kv/`, with `kv-chunk-target`'s world and suite. A lint's line edits only.
- `skein-worlds` and `skein-clippy`: `skein/testing/skein-throwaway-forgejo/`, with `macos-cheap/pin-move`.
- `temper-clippy`: `temper/crates/temper/tests/support/`, with `macos-cheap/habits`, in other files, attribute lines only.

`map-quiet` and `temper-profiles` run in parallel and meet in two temper files, on different lines: each spec says
which it leaves to the other.
