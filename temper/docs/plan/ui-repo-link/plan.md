# ui-repo-link: the plan

The request: a project's repository is one click to its page on the forge (`inputs/request.md`). Part 1 is worth having alone and goes first.

## Parts

- **link** (released): the link everywhere a repository is named with its forge known. Task `repo-link`.
- **box-sign-in** (being cut): one click, signed in, on a box's owned Forgejo.

## Critical path, as it stands

`repo-link` alone so far.

## Overlaps

`repo-link` touches `temper-client-view/src/words.rs`, `frame.rs` and `temper.css` (the switcher's block), which `ui-pause-close`, `model-catalogue-1` and `chat-reading-c` also edit: unavoidable, and each edit is local. Goldens and the gallery are regenerated on a catch-up.

## Decisions

- A folded Forges row's summary stays words; the link is in the row's body. A disclosure's description is text inside `<summary>`, and a link there is the trap the request names for a button. Moving one out would redraw the row.
- Encoding: a name's bytes are percent-encoded as a path's segment, then again in `redirect_to`, so a hostile name ends neither.
