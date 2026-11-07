# ui-drafts-default: what every task holds to

The request is `temper/docs/plan/ui-drafts-default/inputs/request.md`. Its section `For every task of this plan` holds in
full and is not repeated here. What it comes to for these two tasks:

- **Focused and fuzzy alone.** No `tools/tiers/run`, `<tier>-<tree>` profile, `scripts/browser.sh`, walk, render check or
  throwaway driver. A slow-tier story that reads the old default changes with it and is compiled (`--no-run`), not run; the
  report says so. `temper-client-journeys again` is the one thing here that starts a browser, and the request asks for it.
- **New cases are rows of an existing table test,** beside their siblings' fixture. A new test only where the spec names one.
- **Decide, don't ask.** A stored format, a protocol or a public interface may change freely. A small edit just outside your
  list (a re-export, a `mod` line, a golden) is made and named in the report.
- **Generated files your change makes stale are yours,** by their own tool and never by hand: goldens, prints,
  `docs/help/reference/*`.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If the base passes
  and your branch fails, it's yours.
- **A departure from the implementer's text, for both tasks:** `docs/design/` is otherwise closed, but each task removes
  from `temper/docs/design/client/README.status.md` and `temper/docs/design/protocol/configuration.status.md` the **Not yet**
  entries its spec names, once its code makes them true. It changes no other design line; what the code taught goes in the
  report.
- **Beside this plan:** `ui-proposal-2`, `ui-small-1`, `ui-model-line` and `ui-box-slots` change the same chat, words and
  status files. Whichever lands second catches up; a golden or print is regenerated rather than merged.
