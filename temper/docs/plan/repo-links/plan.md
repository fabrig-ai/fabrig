# repo-links: the plan

From `inputs/request.md` (the director, 2026-11-03 ~19:05Z): the old GitHub repositories are
about to be deleted, and temper still links to the old temper repository's copy of "Run temper
on a server". One part, one task, in temper alone; the task names `notes/common.md`.

## The design

Nothing to write. The link is `temper/docs/design/client/forge.md`, 5.4's "Show how" on
Forgejo on this machine's Details; the design names no address, and
`temper/docs/design/client/forge.status.md`'s 5.4 entry ("a link to the deployment's help")
stays true. The one design file the task touches is the mockup line the request names.

## Parts and tasks

1. **repo-links** (temper, `temper/docs/design/client/forge.md, 5.4`), slice 1.
   - `tasks/server-help-link.md`, medium, ~20 minutes, needs nothing: `DEPLOYMENT` in
     `temper-client-view`'s `forge/connect.rs` and the mockup's `href` in
     `temper/docs/design/client/mockups/pages.js` become
     `https://github.com/fabrig-ai/fabrig/blob/main/temper/docs/help/server.md`; a new view test
     in `tests/connect_forge.rs` pins it on the "Show how" link.

Critical path: `server-help-link` alone.

## What was decided, and why

- **Exactly two holders.** `git grep` over every tracked file (hidden ones too) at the cut's
  base finds the old repository's address in `forge/connect.rs` and `mockups/pages.js` alone,
  besides this plan's `inputs/`. `temper/docs/design/domain/forge.md` and the welcome journey's
  `03.html` hold no such link, as the request says. No test, golden or fixture pins the old
  address.
- **A test is added.** No test reads `DEPLOYMENT` or the link's address, so `temper-client-view
  --lib` alone would pass whatever the address. The task adds one test that pins the new
  address on the "Show how" link, so the check the request names checks the link.
- **`fabrig-ai/temper-probe` stays.** The request's search, `fabrig-ai/temper`, also matches
  `fabrig-ai/temper-probe`, the GitHub repository kept for QA (in a draft, and in plan and
  development files). It is another repository, not the old temper one; the task's search
  leaves it out (`fabrig-ai/temper(?![\w-])`).
- **The mockup is the task's.** The request asks for one task for both places; the task's
  common note states the departure from the implementer's text for that one line.
- **Not in scope:** each tree's `README.md` links to `https://github.com/fabrig-ai/hearth`,
  another repository, which the request does not name; no tracked file names
  `fabrig-ai/skein`, `fabrig-ai/smith` or `fabrig-ai/jig`.

## Overlaps with other drafts

None. `public-reading-build/public-web` owns other files of `temper-client-view` (`src/tests.rs`,
`src/words.rs`, `src/lib.rs` and its `public` modules) but not `src/forge/connect.rs` or
`src/tests/connect_forge.rs`; no draft beside this one names `mockups/pages.js`.
