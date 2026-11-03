# look-welcome-chats: the look requests on `look-requests`

At the user's request (through their laptop session, 2026-11-03), implement the look requests on the branch `origin/look-requests`, as `temper/docs/design/client/journeys.md`, section 7 has "Done". Done is a landing that:
- carries each request's `Look-request:` trailer in commits of its own, never the request's commit itself;
- moves the overlay's rules into `temper.css` or `task-pages.css`, and the markup's changes into the view;
- reprints, so that the reprint equals the requests' pages, compared as text. Where it does not, the landing says why.

## The requests

There are 11 commits on `origin/look-requests`, over an older main (merge base 5f3749c50c). All of them touch the welcome-playground journey: `temper/docs/design/client/journeys/welcome-playground/02.html` and `journeys/overlay.css`.

They stack: implement the state at the branch's tip, since later commits supersede earlier ones. Read them with `git log -p origin/main..origin/look-requests`. Their trailers, oldest first:

playground-without-account-banner, terminal-chats-experiment, chats-left-pane-experiment, chats-pane-head-experiment, banner-project-switcher-experiment, switcher-new-project-experiment, pane-playground-note-experiment, pane-note-after-chats-experiment, pane-repository-only-experiment, banner-leave-playground-experiment, switcher-rename-project-experiment.

What they ask, in short:
- a terminal-like chats page;
- a left pane (`aside.temper-sidebar`) holding the new chat's status and repository, the chats' heading, filters and list, and the playground's note;
- the prompt alone on the right, its hints and model choice below it;
- a quieter one-line banner with darker inactive words;
- the banner naming the open project alone, with a switcher (`details.temper-projects`) that lists the others, renames the open one in place (jig's Rename, owners only) and makes a new project (MakeProject, no repository);
- "Set up temper ->" in the banner while the playground runs (the existing EndPlayground act).

## Bounds

- **No engine change:** the user expects none. The acts named are existing ones. If one turns out to be missing, ask rather than build it.
- **The playground's invitation** ("What shall we change?" and its suggested requests) is gone from these pages. If tests break because of that, the user says it is fine to put the suggested requests back to keep tests green; they will revisit it.
- **The design follows the look:** `client/README.md` and its pages' sections, `journeys.md`, and the mockups as `mockups/README.md`, 1 says. What the edits imply for states no journey reaches is the view's and the stylesheet's for every state (`journeys.md`, 7).
- **Out of scope:** `temper/docs/design/draft/first-run-projects.md`, whose execution is asked for separately.

## What runs beside it

Wave 2 of the reference case (plans `reference-m2`, `-m3`, `-m5`, `-m6`, `-m7b`) runs at the same time. It works on temper's engine, forge connector and ctl, not on the web client's pages. Leave its files alone.

## Walk

None of its own. The journeys' test is the check: the reprint equals the requests' pages.
