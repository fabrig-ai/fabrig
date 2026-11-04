# debt-12: the spot review of skein D016, and a generated debt index

Two independent parts (`inputs/request.md`). No debt item closes here. Finding 7 is left alone, as the request
says, and `skein/docs/design/link.status.md` now records it under section 1.

## Parts and tasks

| Part | Task | Needs | Effort | Min | Findings | What it does |
|---|---|---|---|---:|---|---|
| `debt-index` | `debt-index` | | high | 40 | | `tools/debt/`, the five READMEs lose their tables |
| `forge` | `forge-close` | | high | 75 | 1, 3, 8, 6 | the close bound's test, no spin, the guard, two dead lines |
| `link` | `link-surface` | | high | 40 | 2 | `Debug` by hand with canaries, `due`, `Upload::of` |
| `owners` | `oauth-tidy` | | medium | 10 | 5 | `link_env`, all that's owed there |
| `forge` | `forge-tidy` | `link-surface`, `forge-close` | medium | 15 | 5, 9 | `link_env`, `Upload::of`, `next == Finish` |
| `link` | `link-ending` | `link-surface`, `forge-close` | high | 60 | 4, 10, 6 | three pins first, then one state, then `socket` goes |
| `owners` | `llm-tidy` | `link-surface` | high | 30 | 5, 6, 10 | `due`, `link_env`, two dead calls, the world drops after `Close` |
| `owners` | `setup-tidy` | `link-surface`, `link-ending`, `llm-tidy`, `forge-tidy` | high | 25 | 5, 9, 10 | `due`, `link_env`, `Upload::of`, one `close_timeout`; the production total |

## Decisions

- **`debt-index` is one task:** the tool and the five READMEs land together, so no commit has a README naming a
  command that isn't there. It crosses the trees only in those READMEs, and one citation in hearth's design.
- **The highest ID is a line in each README,** not derived from history: a shallow clone would derive a wrong
  one. Paying an item never touches the line, so it causes none of the conflicts this part removes; two
  branches that add items do meet on it, which is the clash on an ID it should show. The command checks it,
  since it's the one hand-kept line left.
- **Findings checked by reading,** before the specs: `notes/contracts.md`, 3. Three corrections to the review:
  - **5:** oauth's site stays. It already reads `due(web.next_deadline(), now)` through `Web`, which hides its
    link, so `Link::due` would add a wrapper and save nothing.
  - **6:** `Link::socket` is removed, not made test-only: the link world reads it, and an integration test can't
    reach `#[cfg(test)]`. Its two assertions go; `World::deliver` already asserts each request's `SOCKET`.
  - **10:** the llm test keeps its name and gains the power to fail: its world now drops a `Stream` after `Close`,
    as io does, rather than renaming the test to what it can't check.
- **`Moved` and `Event` keep their derived `Debug`,** over `Up`'s bytes: the review asks for `Upload` and `Link`,
  which keep bytes; those two pass through.
- **The surface first and small:** `link-surface` carries what owners build on, so they don't wait on
  `link-ending`, which reshapes the state behind the same face.
- **`socket` goes in `link-ending`, not `link-surface`:** the guard that reads it is the forge's, so `forge-close`
  removes the guard and no task writes in another's crate.
- **Finding 8 changes an order,** so `forge-close` runs the forge's readers across trees, and stops and asks if a
  test's trace moves. It restores the connection's order before it stood on the link, closing aside.
- **The design** moved first, in this batch: `skein/docs/design/link.md`, 3.1, 3.4 and 4, with **Not yet**
  entries in `link.status.md`, which `link-surface` and `link-ending` delete as they build.

## Overlaps with other drafts

- `repository-copies-1/copy-words` changes `skein-forge-setup/src/host.rs` and `host/connection.rs`, and its
  `copy-story` changes `skein/tests/forge/setup/src/host.rs`. `setup-tidy` needs them and can't avoid them, so it
  runs last, with `needs` beyond the code it builds on, on purpose. Its hunks are a few lines each.
- `debt-index` rewrites each tree's table: a branch elsewhere that pays an item before it lands meets it once.
  No draft listed beside this one touches those READMEs.

## Production size

The request expects about 65 lines lower. By task: `link-surface` +30, `link-ending` -35, `forge-close` about 0,
`forge-tidy` -12, `llm-tidy` -12, `oauth-tidy` -4, `setup-tidy` -20: about 55 lower, the review's 80 less `Debug`
and `due`, and oauth's site. `setup-tidy` reports the total.

## Critical path

`forge-close` (75) and `link-surface` (40), then `link-ending` (60), then `setup-tidy` (25): about 160 minutes.
