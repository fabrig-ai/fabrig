+++
id = "account-onboarding"
part = "account-onboarding"
kind = "docs"
needs = ["to-begin-card", "to-begin-story"]
files = [
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/journeys.status.md",
]
effort = "high"
minutes = 90
+++

# `project-home-onboarding`: the render check and the account

## Goal

Prove the landing of `project-home-onboarding` by rendering, and account for each item, by
`temper/docs/plan/look-round-4/notes/account.md`. Every commit carries `Look-request:
project-home-onboarding`; the last holds the table.

Acceptance:

- **Pages compared,** from `9fcbec1d32`: `project-landing/01.html` against the gallery's
  `home-two-left`, `project-after-skip/01.html` against `home-after-skip`, each to its reason.
  `project-landing/01.html` also at 1366 by 630.
- **States in words:** no repository, the adoption out, refused, another model, a step done (each
  note's wording), something needs the person under To begin: the gallery's states. Landing by turn:
  deferred.
- **Screenshots:** `~/src/fabrig/look-round-4/project-home-onboarding/`.
- **Rows that differ or defer,** at the least, each with its home:
  - landing by turn: `README.md`, 11, *How a project lands*, the question `to-begin-landing-by`;
  - "Landing: nothing has landed yet" on the page after "Skip": the question `home-landing-line`;
  - `/chats/new` as drawn, `/chats?at=start` landed;
  - `project-landing`'s reprint still prints Welcome until `record-landing`.
- `mockups/STATUS.md`'s *The project's frame and Home* says what To begin draws.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/account.md`.
- `temper/docs/design/draft/look-requests.md`, 3 to 5.
- `git show --format=%B -s 9fcbec1d32`; the commits on the trunk carrying its trailer.

## Owns and may change

Owns, as `files` lists them. May also change, for a difference the check finds, under the request's
trailer: `temper/crates/temper-client-shell-web/temper.css` and
`temper/crates/temper-client-view/src/pages/home.rs`, the lines this request landed.

## Steps

1. The check, the fixes, the screenshots.
2. The status lines.
3. **Check,** `notes/common.md`, then the citations; a fix runs `heavy cargo nextest run -p
   temper-client-view -p temper-client-gallery`.
4. The last commit, its table in its message, then push.

## Report

The table, the render check's result, the directory and the builder.
