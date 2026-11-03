# playground-browser: the plan

From `inputs/request.md`: the playground in the browser, part three, on skein's kit (plan
`playground-skein`) and temper's composition (plan `playground-temper`), both on main:
`temper/docs/design/playground.md`, 3 (the visitor in the composition, the shell's feature), 6
(the page), 7 (served), 9 (the native walk through the web client's host, a log replayed, the
wasm32 builds, the browser tier's walk) and 10 (its bar). The design is complete for this work;
the shapes that cross tasks are `notes/contracts.md`. Every task names `notes/common.md`.

## Parts and tasks (first push)

1. **shell-clippy** (`temper/docs/design/client/browser.md, 4.3`), slice 1.
   - `tasks/shell-turn-split.md`, feature, medium, ~45 min, needs nothing: `Shell::turn` split
     with no lint exception, its platform touched in one method; `temper-client-shell-web` in
     `.config/gate/wasm32.txt`; browser.status.md's 4.3.
2. **visitor** (`temper/docs/design/playground.md, 3`), slice 2.
   - `tasks/playground-visitor.md`, interface, high, ~90 min, needs nothing: `Process::Visitor`,
     the visitor's config from the deployment's document, the link printed to the world's
     printer, `story::{reach, parameters, slices}`, `stories::named` (C1, C2).

Later pushes add the native walk and its log (tests/playground), the log replayed under the
one-process world's referee, the worker's entry, the shell's feature `playground` (the page, its
starter, frame and failure page), the served files with `headers` and `bundle.sh --playground`,
and the browser tier's walk.

## Critical path (as it stands)

`playground-visitor` (90) → the native walk of story A → the worker's entry and the page → the
served bundle → the browser walk.

## Overlaps with drafts beside this one

- `.config/gate/wasm32.txt`: plan `wasm32-temper` (not in the brief's list; its inputs landed at
  14:41Z) adds `temper-playground` after `skein-playground`; `shell-turn-split` adds
  `temper-client-shell-web` after `jig-client-shell-web`, a line apart.
- `temper/docs/design/playground.status.md`: `wasm32-temper` may close words of its 9; this
  plan's tasks remove words in place, never reflowing (`notes/common.md`).
- `Cargo.lock`: `playground-visitor` changes `temper-playground`'s dependency lines.
