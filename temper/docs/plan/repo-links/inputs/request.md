# Request: temper's links to the old temper repository point at fabrig

From the director, 2026-11-03 ~19:05Z: the old GitHub repositories are about to be deleted, and
temper still links to `https://github.com/fabrig-ai/temper/blob/main/docs/help/server.md`, which
breaks then. At main (checked 19:05Z) exactly two places hold that URL:

- `temper/crates/temper-client-view/src/forge/connect.rs:25` (`DEPLOYMENT`, the product's
  deployment help link);
- `temper/docs/design/client/mockups/pages.js:1648` (a design mockup).

One task: point both at the same page in the fabrig repository,
`https://github.com/fabrig-ai/fabrig/blob/main/temper/docs/help/server.md`, and fix any test,
golden or fixture that pins the old URL (search the whole tree for `fabrig-ai/temper` after the
change; nothing may remain, other than history in a status or plan file). The old-repos report
also named `temper/docs/design/domain/forge.md` ~514 and the welcome-set-up journey page
`03.html`; at main neither holds the URL, so leave them unless the search finds it there. Check:
the narrowest tests of `temper-client-view` that cover the link (`heavy cargo nextest run -p
temper-client-view --lib`, or the test that names `DEPLOYMENT`).
