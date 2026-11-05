+++
id = "record-landing"
part = "record-landing"
kind = "feature"
needs = ["account-configure"]
files = [
  "temper/docs/design/client/journeys/project-landing/**",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 70
+++

# `project-landing`, recorded again

## Goal

Record `project-landing` once the welcome leads to Home, so that its reprint can be the request's page
(`temper/docs/design/client/journeys.md`, 8), and say how it compares. Every commit carries
`Look-request: project-home-onboarding` and, under it, `Look-request: to-begin-more-accounts`.

Acceptance:

- **First, the sign-in:** plan `fake-signin-model` is fixing what stopped `look-round-3`: a fake box's
  ChatGPT sign-in left Ready with no model. If `origin/main` holds that fix and your base doesn't,
  merge `origin/main` into your branch. Walk to Ready. **If it still says "Model none yet", stop:
  commit nothing, and end with `Question:`, saying that `fake-signin-model` hasn't landed, or doesn't
  reach this walk, with what you saw. Never work round it.**
- **The walk:** `temper/docs/design/client/journeys.md`, 9, `project-landing`'s row: Forgejo on this
  machine with an empty `work/hello`, the ChatGPT plan signed in, Ready's "Hello" created, then Home
  reloaded at `/projects/1` once "work/hello is in Hello" shows. By `again` if the recording's acts
  still hold, otherwise walked anew and `add`ed (`journeys.md`, 8).
- **Its page,** `project-landing/01.html`, is Home with To begin: reprinted, and compared by line with
  `git show 53ba7876f8:temper/docs/design/client/journeys/project-landing/01.html`.
- **Its render,** beside the request's, at 1440, 1000, 420 and 1366 by 630, in
  `~/src/fabrig/look-round-4/project-home-onboarding/` on your builder, named `-recorded`.
- **`journeys.md`, 9, `journeys.status.md`, `project-landing/README.md`** say what is true: the row's
  stop goes, the **Not yet** entry goes, the README names the page. Hand-made `project-at-work` and
  `project-after-skip` are reached as drawn: they are not committed, and no recording reaches them.
- **The last commit** carries a short table above its trailers: the page's rows that `account-onboarding`
  and `more-accounts` left to this recording, each landed or differing and why.
- **What stops it otherwise is a result:** if the walk can't finish for any other reason (the sign-in's
  departure, a page never still), keep the recording as it stands, commit nothing that fails
  `the_committed_journeys_are_their_print`, and report `Done:` with what stops it, step by step, and
  the last page reached.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`.
- `temper/docs/design/client/journeys.md`, 3 to 5, 8 and 9; `journeys.status.md`.
- `temper/docs/plan/look-round-3/tasks/record-landing.md`: how it was walked, and its driver.
- `temper/docs/design/client/journeys/project-landing/README.md`.

## Owns and may change

Owns, as `files` lists them. `welcome-playground/**` and `welcome-set-up/**`: their reprints, if any,
and nothing else. A throwaway walk driver is allowed: untracked, gone before the push, as
`notes/common.md` says for render scripts.

`fake-signin-model/record-landing` owns the same files on its own trunk: if its commit is in your base,
start from what it recorded and say so.

## Steps

1. Read the recording's head, the sign-in's state.
2. Walk, save, `heavy cargo run -p temper-client-journeys -- again project-landing`, or `-- add
   project-landing FILE`.
3. Reprint every journey; diff `01.html` against the request's.
4. README, `journeys.md`, 9, `journeys.status.md`.
5. **Check,** `notes/common.md`, then `heavy cargo nextest run -p temper-client-journeys`. Remove every
   box and scratch directory you made, but the kept walk logs.

## Report

The page's diff against the request's, line by line, with the reason you can see for each; the
builder's `hostname`.

## Answers

2026-11-05 (fabrig/ui, owner of temper's UI, through fabrig/temper and agents/loop). **Defer the Home recording; don't widen the sign-in investigation.** End the task with status entries naming the stop, and leave `project-landing`'s recording as it is. fabrig/ui takes the Chromium sign-in fault as its own plan and re-records after it.
