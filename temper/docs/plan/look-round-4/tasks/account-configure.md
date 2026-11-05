+++
id = "account-configure"
part = "account-configure"
kind = "docs"
needs = ["configuring-chat"]
files = [
  "temper/docs/design/client/mockups/STATUS.md",
]
effort = "high"
minutes = 100
+++

# `configure-chat-in-settings`: the render check and the account

## Goal

Prove the landing of `configure-chat-in-settings` by rendering, and account for each item, by
`temper/docs/plan/look-round-4/notes/account.md`. Every commit carries `Look-request:
configure-chat-in-settings`; the last holds the table.

Acceptance:

- **Pages compared,** from `53ba7876f8`: `configure-chat/01.html` against the gallery's `configure`,
  at 1440, 1366, 1000 and 420.
- **States in words:** typing, sent (`configure-sending`, then `configure-chat`), refused, in Chats
  (`configure-chats-tag`), a configuring chat opened again.
- **Screenshots:** `~/src/fabrig/look-round-4/configure-chat-in-settings/`.
- **Rows that differ or defer,** at the least, each with its home:
  - the address, settled: `/projects/<project>/settings/configure`, `browser.md`, 2.3;
  - the Settings page the path names isn't designed: `README.md`, 3.5, *Settings*;
  - How work runs' offer and "Change something": `HowWorkRuns` isn't built, `README.status.md`;
  - "gpt-6-luna", "the configuring role's first model": the first model `me` serves for the project,
    since a template's `llm` may be its charter's default, which no client is given;
  - this page docked beside How work runs: Later, the request's;
  - focus at opening, if `configure-page` left it.
- `mockups/STATUS.md` holds a row for the page, under *The project's frame and Home*.
- **The held requests stayed out:** `git log --format='%(trailers:key=Look-request,valueonly)'
  origin/main..HEAD | sort -u` names neither `new-menu` nor `configure-chat-draft`.

## Inputs

- `temper/docs/plan/look-round-4/notes/common.md`, `notes/account.md`.
- `temper/docs/design/draft/look-requests.md`, 3 to 5.
- `git show --format=%B -s 53ba7876f8`; the commits on the trunk carrying its trailer.

## Owns and may change

Owns, as `files` lists them. May also change, for a difference the check finds, under the request's
trailer: `temper/crates/temper-client-shell-web/temper.css` and
`temper/crates/temper-client-view/src/pages/{configure.rs,chat.rs}`.

## Steps

1. The check, the fixes, the screenshots.
2. The status lines.
3. **Check,** `notes/common.md`, then the citations; a fix runs `heavy cargo nextest run -p
   temper-client-view -p temper-client-gallery`.
4. The last commit, its table in its message, then push.

## Report

The table, the render check's result, the directory and the builder.
