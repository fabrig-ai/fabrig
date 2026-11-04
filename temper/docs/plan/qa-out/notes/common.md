# qa-out: what every task holds

Standing rules for this plan's tasks. Each task names this file.

## Documents only

- **No Rust, no test, no script, no budget changes.** No code comment needs a change either: the
  comment on `deployment` in `temper/crates/temper/tests/support/forge_land/rung.rs` cites
  `temper/docs/development/qa.md`, 3, which stays, so leave that file alone (fabrig-scale is
  changing it).
- **So these steps of the implementer's text do not apply:** `cargo fmt`, `old_forms.py`, clippy,
  and the packages' focused tests. The checks are the two below.
- **Named design files are in scope.** The implementer's text says `docs/design/` and drafts are
  not yours. This plan departs from that because the request (`inputs/request.md`) changes them:
  a task may edit a design or draft file only where its `Owns and may change` names the passage,
  and only as it says. Any other design file is still not yours. Report what you would change.
- **Other plans' directories are records and stay as they are.** `retired-plans-out`,
  `tiers-wrapper`, `withdrawn-forms`, `live-smith-reds-3`, `sign-in-wait` and the rest may name
  a removed path; `docs/plan/` is outside both checks.

## The checks

Run from the repository root, once the work is committed: both read what git tracks.

```sh
python3 -I -B tools/citations/citations.py --all
heavy cargo nextest run -p fabrig-boundary --test outward -E 'test(=no_file_names_the_lab_beyond_the_inventory)'
```

The first must print `citations: 0 broken in HEAD`; the second must run one test and pass. A run
that selected nothing proves nothing.

The second prints the inventory the tree needs when it fails. Take it as a hint for your own
files only: `tests/boundary/outward.txt` changes only where your task says.

## The words

- **What must not appear in a file these tasks write:** a tool, machine, session, role or
  repository of whoever runs fabrig's development; who marks, chooses, starts, gates, triages
  or walks; a lent or borrowed login; a machine or workspace variable (`W`, `HOST`, `REPO`,
  `UNIT`, `REPORT`). The help's own placeholders (`CONFIG`, `DIR`, `PROJECT`, `FORGE`, `KEY`)
  stay.
- **Comments and documents say what is true now:** no history, no "originally", no editions,
  no walk ids. A rule keeps its reason in the present tense.
- **"Pass", "case", "walk" (the help's walks, `temper help box`) and "rung" stay:** they are
  temper's words. A rung is a story and the backends it runs on.
- **Line width:** match the file you edit; a new page wraps at 100 columns.
