# Common rules for every component-kit task

Read this first, then your family's note (`stage-common.md` or
`lib-common.md`) when your spec names one, then your spec. The draft is
`skein/docs/design/draft/component-kit.md`; the choices it leaves open are
settled in `decisions.md` beside this file.

## Scope

- Change only the files your spec's `files` lists. A reader in another
  file that must change with yours (a call site, a `mod` line) is named in
  the spec; anything beyond that is a `Question:`.
- The run is skein's: no file outside `skein/` changes, except where a
  spec names one. A skein public item other trees use changes only
  additively (a new item, a provided method); if a smith, jig or temper
  package stops compiling because of your change, stop with a `Question:`.
- No design document, draft or `docs/help/` changes. What the code taught
  goes in your report.
- Behaviour stays as it is, except where your spec says otherwise: these
  are moves onto shared items. A test that pinned the old private form is
  moved onto the shared one, never weakened.

## Code

- skein's rules hold (`skein/AGENTS.md`, `programming-model.md` 10 for
  step crates): no `_` arms, no `matches!`, no `unwrap`, no `as`, no
  indexing, checked arithmetic. A lint exception is a scoped
  `#[expect(..., reason = "...")]`.
- Comments say what is true now, in the design's words; no history.
- A new public item gets a short `///` contract: what it returns or
  does, and what it refuses.

## The counts

- The counts of `skein/scripts/old_forms.py` may only fall. Your task
  lowers the ones it moves: run `python3 -I -B scripts/old_forms.py
  --write` from `skein/` and commit `skein/scripts/old-forms.txt` with
  your change. A count your change would raise is a `Question:`.
- Your report gives each count you touched, before and after.

## Checks, in your slot, through `heavy`, one at a time

1. `cargo fmt --all` (directly).
2. `python3 -I -B scripts/old_forms.py` from `skein/`: exit 0.
3. `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`
   naming every package you changed.
4. `heavy cargo nextest run -p PKG [...]` for every package you changed
   and its direct dependents inside skein; check the count is not zero.
5. After a public change in skein-lib, skein-io or skein-shell:
   `heavy cargo check -p <dependent> --tests` for the dependents your
   spec names in other trees.

Read a failure's log before any rerun. Never run the whole workspace:
the plan's end check runs the queue's full set.

## Commits and the report

- Small commits that say why, on your branch; push your branch only.
- The report's first line is `Done: <what landed>` or `Question:` with
  the options and your recommendation. Then: the sites moved (path:line),
  any site left with the reason your spec's defaults gave, the counts
  before and after, the tests run with their counts, and the calls you
  made inside the spec.
