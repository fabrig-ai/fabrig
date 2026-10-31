# Rules every component-kit-2 task follows

Read this first, then the family note your spec names (`stage.md` or
`lib.md`, beside this file), then your spec. The draft is
`skein/docs/design/draft/component-kit.md`. `decisions.md` beside this
file holds the cut's choices; your spec quotes what binds you, so read it
only for the why.

## Scope

- Change only the files your spec lists under `Owns and may change`,
  and only in the way it says for reader files. Anything beyond that is
  a `Question:`.
- The run is skein's. No file outside `skein/` changes unless your spec
  names it. skein's public items that other trees use change only
  additively: a new item, a provided method, a new enum variant where
  your spec says so. If a smith, jig or temper package stops compiling
  for a reason your spec does not pre-approve, stop with a `Question:`.
- No design document, draft or `docs/help/` changes. What the code
  taught you goes in your report.
- Behaviour stays as it is, except where your spec says otherwise: these
  are moves onto shared items. A test that pinned the old private form
  moves onto the shared item; it is never weakened or deleted unless your
  spec says so.

## Code

- skein's rules hold (`skein/AGENTS.md`; `skein/docs/foundation/programming-model.md`,
  10, for step crates): no `_` arms, no `matches!`, no `unwrap`, no
  `as`, no indexing, checked arithmetic, no closures in step code. A lint
  exception is a scoped `#[expect(..., reason = "...")]`.
- Comments say what is true now, in the design's words; no history, no
  "now uses the shared ...".
- A new public item gets a short `///` contract: what it returns or does,
  and what it refuses or treats as a bug.
- Use the shared items by their paths in your spec; never add a private
  wrapper around one.

## The counts

- `skein/scripts/old_forms.py` counts the old forms of `decisions.md`'s
  table. Counts may only fall. Your task lowers the ones it removes: from
  `skein/`, run `python3 -I -B scripts/old_forms.py --write` and commit
  `skein/scripts/old-forms.txt` with your change. A count your change would
  raise is a `Question:`.
- If `scripts/old-forms.txt` on your base has no line for a form your spec
  names, the counts task has not merged: stop with a `Question:`.
- Your report gives each count you touched, before and after, and every
  site you left with the default that let you leave it.

## Checks, in your slot, through `heavy`, one at a time

1. `cargo fmt --all` (directly, from the repository root).
2. `python3 -I -B scripts/old_forms.py` from `skein/`: exit 0.
3. `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings`,
   naming every package you changed.
4. `heavy cargo nextest run -p PKG [-p PKG ...]` for every package you
   changed and the readers your spec names; check the count is not zero.
5. Your spec's other trees, if it names any: `heavy cargo check -p PKG
   --tests` for each.

Read a failure's log before any rerun (`AGENTS.md`, 6). Never the whole
workspace: the trunk's end check runs the queue's full set.

## Commits and the report

- Small commits that say why, on your branch; push your branch only.
- The report's first line is `Done: ...`, `Failed: ...` or `Question:
  ...` with the options and your recommendation. Then: the sites moved
  (path:line), the sites left and why, the counts before and after, the
  tests run with their counts, the calls you made inside the spec, and
  your implementation and check minutes apart.
