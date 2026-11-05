+++
id = "record-landing"
part = "record-landing"
kind = "feature"
needs = ["signin-model"]
files = [
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys/welcome-playground/02.html",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 75
+++

# Record project-landing again, on Home

## Goal

Walk `project-landing` anew on a fresh `temper box --fake-backends`, through the ChatGPT sign-in and
Ready's Create, to Home at `/projects/1` in "Hello" with `work/hello` adopted, and reprint it. If
something stops it, say what, exactly.

Acceptance:

- `project-landing/recording` is a fresh recording at `/projects/1`, added with `add`, its first
  replay proving it (`temper/docs/design/client/journeys.md`, 4.3).
- `01.html` is reprinted and draws Home in Hello, with `work/hello` adopted.
- `project-landing/README.md`, `journeys.md`, section 9's row and `journeys.status.md` say what is
  true now: the walk is whole, and the 9 **Not yet** entry is gone.
- `the_committed_journeys_are_their_print` passes for every journey.
- Every commit of this task carries the trailer `Look-request: project-home` in its last paragraph,
  directly above any `Co-Authored-By:` line, with nothing after that paragraph.

## Inputs

- `temper/docs/plan/fake-signin-model/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 3.5, 4.3, 8 and 8.1, 9.
- `temper/docs/design/client/journeys/project-landing/README.md`, and the commit that recorded it,
  `68cddad304`.
- `signin-model`'s commits, for what changed in the walk.

## Owns and may change

Owns, as `files` lists them. These are stated departures from "Not yours" in your text: the journey's
pages, its README, section 9's row and the status entry are this task's.

May also change: `temper/tools/journeys/**`, only if `add` refuses what the design allows, and then
say so.

Pre-approved:

- **`again` can't walk this** (`journeys.md`, 8.1): the recording holds no act, since it starts on a
  reload after Ready. Walk it as a person would, in headless Chromium, by a throwaway driver kept
  outside any worktree under `~/src/fabrig/fake-signin-model/project-home/walk/`, untracked and never
  committed: the box printing its sign-in link, the recording bundle
  (`scripts/bundle.sh --loopback --record`), the welcome walked to Ready, Forgejo run on this machine
  with an empty `work/hello`, the ChatGPT plan through the fake sign-in, "Create your project"; then
  the wait until Home says "work/hello is in Hello"; then the page reloaded at `/projects/1`, and the
  save (Alt+Shift+S). The driver may use skein's browser kit, or Chromium's debugging port directly.
  It sends a person's input only, and injects no engine fact.
- **What stops it is a result, not a failure:** if the walk can't finish, keep the recording as it
  stands, commit nothing that fails `the_committed_journeys_are_their_print`, and report `Done:` with
  what stops it, step by step, and the last page reached.
- **Another journey's print changes on reprint:** commit only `welcome-playground/02.html`, if that;
  any other changed page outside `project-landing` is a `Question:`.

## Steps

1. Walk, save, `heavy cargo run -p temper-client-journeys -- add project-landing FILE`.
2. Reprint every journey: `heavy cargo run -p temper-client-journeys`.
3. README, section 9's row and the status entry.
4. **Check:** `heavy cargo nextest run -p temper-client-journeys`, `cargo fmt`,
   `python3 -I -B tools/citations/citations.py --base origin/main` from the root. Remove every box and
   scratch directory you made, but the kept walk logs. Commit and push.

## Report

As your text says, with: the walk step by step, the last page reached, and the builder's `hostname`.

## Answers

2026-11-04. From `signin-model`'s question: what if the walk stops at Ready with no model again?

`signin-model` did not reproduce that stop through a native client, so it lands with no sign-in fix,
and this Chromium walk is the first to repeat the path that stopped. If Ready, after the sign-in,
has no model and Create stays disabled, that one stop is a `Question:`, not the `Done:` the spec
allows for other stops, since it is the fault this draft is for. Before removing the box, keep in the
walk's logs `box.stderr`, the page Ready drew with any ending's words, and from `temper account list
--json` only `usable`, `last_end` and `signing`. Say which of `signin-model`'s three states it is.
Fix nothing in the sign-in; the answer to that question widens the work.
