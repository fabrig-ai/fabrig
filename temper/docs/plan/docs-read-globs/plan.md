# docs-read-globs: the web journeys' files are test inputs

The request: `inputs/request.md`. fab's queue takes any file under a `docs/` directory as docs unless `.config/gate/read-by-tests.txt` lists it; `temper/docs/design/client/journeys/` holds non-Markdown test inputs the list doesn't name.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `globs` | `tasks/journeys-glob.md` | | `temper/docs/design/client/journeys/**` listed beside `temper/docs/design/client/tui/journeys/**`; the list's header says any format under `docs/`, every non-Markdown input there listed | 5 |

**Critical path:** `journeys-glob` alone, about 5 implementer-minutes.

## What was decided

- **One direct task, no survey,** as the request says. The cut checked its facts on the trunk: the directory holds 2 CSS, 1 JS, 31 HTML, 4 Markdown files and the journeys' `recording` and `setup` files, and `temper/tools/journeys/src/tests.rs` reads them (`the_committed_journeys_are_their_print`, line 13).
- **The header's words are pinned** in the spec, as a `#` comment like the line it replaces, with `docs/` written plainly (the file isn't Markdown).
- **The new glob goes directly above `temper/docs/design/client/tui/journeys/**`,** the request's "beside"; the other lines keep their order.
- **No build or test in the task:** it changes no package, and the request runs nothing beyond the plan's focused and fuzzy runs on the trunk. The spec says so as its departure from the implementer's text.
- **No notes:** one task, and its spec carries its rules.
- **No design written:** the request changes nothing else. `.config/gate/README.md`, *The gate's lists* ("globs of Markdown compiled into or read by tests"), and `AGENTS.md`, 2 ("Some Markdown is read by tests") still say Markdown after this plan. The request leaves them as they are, so they're left for a later draft, and the task names them under `## Left`.

No stored format, protocol or public interface changes.

## Overlaps with drafts beside this one

None. No draft beside this one names `.config/gate/read-by-tests.txt`.
