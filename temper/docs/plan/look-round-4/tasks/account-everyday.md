+++
id = "account-everyday"
part = "account-everyday"
kind = "docs"
needs = ["home-everyday"]
files = [
  "temper/docs/design/client/mockups/STATUS.md",
  "temper/docs/design/client/README.status.md",
]
effort = "high"
minutes = 90
+++

# `project-home-everyday`: the render check and the account

## Goal

Prove the landing of `project-home-everyday` by rendering, and account for each item, by
`temper/docs/plan/look-round-4/notes/account.md`. Every commit carries `Look-request:
project-home-everyday`; the last holds the table.

Acceptance:

- **Pages compared,** from `8bbc1bc964`: `project-at-work/01.html` against the gallery's
  `home-at-work`, `project-landing/01.html` against `home-nothing-needs`, each to its reason.
- **States in words:** more than one need (`home-needs-several`); an answer given, Goals and
  "No goals yet…": deferred, not rendered.
- **Screenshots:** `~/src/fabrig/look-round-4/project-home-everyday/`.
- **Rows that differ or defer,** at the least, each with its home:
  - "you last looked", what landed in the headline, the lede and the byline: `README.md`, 11, *What
    changed since the person last looked*, the plan's question `home-what-changed`;
  - context, choices, "recommended", "Later", the answer's green line and "Undo", "Reply" going to
    the inbox: `README.md`, 11, *A question answered on Home*, the question `home-answers`;
  - "landed" on a chat's line: one read per row, `README.status.md`;
  - Goals: `README.md`, 11, *Goals on Home*, the question `home-goals`;
  - "Landing: …": `README.md`, 11, *The landing's line on Home*, the question `home-landing-line`;
  - `temper-count` drawn as `temper-nav-count`; `Address::Task` where the request drew `/chats/3`;
  - "All 3" with its rows cut reads "All 3+"; the rail's count is the person's across projects
    (`me`), where the section's is the project's.
- `README.status.md` gains **Not yet** 3.5: "landed" on a chat's line, and `mockups/STATUS.md`'s *The
  project's frame and Home* says what Home now draws.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/account.md`.
- `temper/docs/design/draft/look-requests.md`, 3 to 5.
- `git show --format=%B -s 8bbc1bc964`; the commits on the trunk carrying its trailer.

## Owns and may change

Owns, as `files` lists them. May also change, for a difference the check finds, under the request's
trailer: `temper/crates/temper-client-shell-web/temper.css` and
`temper/crates/temper-client-view/src/pages/home.rs`, the lines this request landed.
`to-begin-card` may be writing beside you: keep it small.

## Steps

1. The check, the fixes, the screenshots.
2. The status lines.
3. **Check,** `notes/common.md`, then `python3 -I -B tools/citations/citations.py --base origin/main`;
   a fix runs `heavy cargo nextest run -p temper-client-view -p temper-client-gallery`.
4. The last commit, its table in its message, then push.

## Report

The table, the render check's result, the directory and the builder.
