# debt-13: skein D013 and jig D004

Two independent parts (`inputs/request.md`). skein D013 is paid here. jig D004 is size L and is cut to what fits;
its last task rewrites the item to say what's left. This file comes up to date with each push.

## Parts and tasks so far

| Part | Task | Needs | Effort | Min | What it does |
|---|---|---|---|---:|---|
| `forge-tail` | `forge-tail` | | medium | 30 | `grow::LogTail` over lib's `Tail`, with lib's `into_boxed` |
| `forge-entry` | `forge-entry` | | high | 75 | `List<ForgeEntry>`, `Owner`, `swap_remove`; methods still take a position |
| `root-admission` | `root-admission` | | medium | 35 | `Class` in `jig-domain-link`, `LinkUp::new_work` |
| `root-timers` | `root-timers` | | medium | 30 | `Env::wall_due`, `decision_header` |
| `root-sender` | `root-sender` | | medium | 20 | `Party::sender` |

The rest of both parts follows in the next push: `notes/contracts.md` already holds their shapes.

## Critical path as it stands

`forge-entry`, then the chain that makes every step take an entry, then a call's dialect, then a connection with no
dialect, which closes D013.

## Overlaps with other drafts

None so far: no listed draft changes `skein-forge-connection`, `skein-lib`'s `list.rs`, `tail.rs` or `env.rs`, or
anything under `jig/`. `repository-copies-1` and `fetch-progress-1` are in other crates.
