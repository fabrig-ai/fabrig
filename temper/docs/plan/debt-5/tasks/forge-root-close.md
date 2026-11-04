+++
id = "forge-root-close"
part = "skein-forge-root"
kind = "docs"
needs = ["forge-users", "connection-twins"]
files = [
  "skein/docs/design/forge.status.md",
  "skein/docs/development/debt/**",
]
effort = "medium"
minutes = 15
+++

# skein D010: the item goes

## Goal

With the moves merged, the status file drops its entries for them, the item's file and row go, and the items that cite it say what is left. This task closes D010.

Acceptance:

- `temper/docs/plan/debt-5/notes/forge-root.md`'s measure prints 0.
- `skein/docs/design/forge.status.md` has no **Not yet** entry for 4.4 and no clause on 4.1's borrowing; its other clauses stand, its date is today's.
- `skein/docs/development/debt/D010-github-on-forgejo-modules.md` and its row are gone.
- `rg -n 'D010' skein/docs/development/debt` reads nothing, but in `reviews/`.
- `python3 -I -B tools/citations/citations.py --all` reads 0.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/forge-root.md`, `AGENTS.md`, `skein/AGENTS.md`.
- `skein/docs/design/forge.md`, 4.4.

## Owns and may change

- `skein/docs/design/forge.status.md`: as the acceptance says.
- The item's file and its row in `skein/docs/development/debt/README.md`.
- `D005-civil-calendars.md`: its `Where` and lines naming `forgejo/time.rs` say `time.rs`; the line "GitHub's dialect reads times through `forgejo::time` (see D010)." goes, and so does "It helps D010."
- `D006-lru-tables.md`: the user table lives once at the root, so its **Where** gains `skein-forge/src/users.rs`, its Paid-when's last clause goes, and "It is one of the neutral pieces D010 moves." goes. Its measure stays 2.
- `D011`, `D013`, `D014`, `D015`: each sentence that says "after D010" says what holds: the neutral pieces sit at skein-forge's root (`skein/docs/design/forge.md`, 4.4). Their **Paid when** lines keep their sense.
- Nothing else. No Rust.

## Steps

1. The edits above. Run the measure and the citations; commit (`D010: GitHub's dialect stands on skein-forge's root`, saying why) and push.

## Report

The measure, and each item you edited with its changed line.
