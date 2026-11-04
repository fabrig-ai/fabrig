+++
id = "record-landing"
part = "record-landing"
kind = "feature"
needs = ["home-page", "rail-readers"]
files = [
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys/welcome-playground/02.html",
  "temper/docs/design/client/journeys.md",
]
effort = "high"
minutes = 100
+++

# Record project-landing again, on Home

## Goal

`project-landing` is walked anew on a fresh `temper box --fake-backends`, through Ready's adoption
and naming, so that its `01.html` is Home in "Hello", with `work/hello` in it, and the reprint can
be set against the request's page. If something stops it, say what, exactly.

Acceptance:

- `project-landing/recording` is a fresh recording at `/projects/1`, added with `add`, its first
  replay proving it (`temper/docs/design/client/journeys.md`, 4.3).
- `01.html` reprinted; `git show 0197eaea31:temper/docs/design/client/journeys/project-landing/01.html
  | diff - 01.html` holds only what `account` will explain. List each line.
- `project-landing/README.md` and `journeys.md`, section 9, say what the journey now is.
- `the_committed_journeys_are_their_print` passes for every journey.
- Every commit carries the request's trailer (`notes/common.md`).

## Inputs

- `temper/docs/plan/look-round-3/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 3.5, 4.3, 8 and 8.1, 9.
- `temper/docs/design/client/journeys/project-landing/README.md`, and the commit that recorded it,
  `68cddad304`.

## Owns and may change

Owns, as `files` lists them.

May also change: `temper/tools/journeys/**`, only if `add` refuses what the design allows, and
then say so.

Departures, pre-approved:

- **`again` can't walk this:** the recording holds no act, since it starts on a reload after Ready.
  Walk it as a person would, in headless Chromium, by a throwaway driver kept outside any worktree
  under `~/src/fabrig/look-round-3/project-home/walk/`, untracked and never committed: the box
  printing its sign-in link, the recording bundle (`scripts/bundle.sh --loopback --record`), the
  welcome walked to Ready, Forgejo run on this machine, an empty repository, the ChatGPT plan
  through the fake sign-in, "Create your project"; then the wait until Home says "work/hello is in
  Hello"; then the page reloaded, and the save (Alt+Shift+S). The driver may use skein's browser
  kit, or Chromium's debugging port directly.
- **Forgejo is real on a fake box** (`temper/docs/design/protocol/setup.md`, 7.5): the walk needs
  the builder's Forgejo and runner, `TEMPER_FORGEJO` and `TEMPER_FORGEJO_RUNNER`, and the callback
  port 1455 free. Never a second box on it at once.
- **What stops it is a result, not a failure:** if the walk can't finish (no Forgejo here, the
  sign-in's departure, a page never still), keep the recording as it stands, commit nothing that
  fails `the_committed_journeys_are_their_print`, and report `Done:` with what stops it, step by
  step, and the last page reached. `project-landing` then prints its stale state: if the test can't
  hold, that's a `Question:`, with options and your recommendation.

## Steps

1. Read the commit `68cddad304` and the recording's head: `path "/chats"`, `start`, `open`, `end`.
2. Walk, save, `heavy cargo run -p temper-client-journeys -- add project-landing FILE`.
3. Reprint every journey; diff `01.html` against the request's, by line.
4. README and section 9.
5. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-journeys`. Remove
   every box and scratch directory you made, but the kept walk logs.

## Report

As `notes/common.md`, Your report: the page's diff against the request's, line by line, with the
reason you can see for each; the builder's `hostname`.
