# ui-authorize-red: plan

red-71's repair. `temper-client-browser-world`'s story `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` has failed at step 7 since just after midnight UTC on 11-08. After the server's restart, the Authorize page reads "temper couldn't sign you in just now" instead of handing the forge its code. The plan finds the cause by running that one story alone, and fixes it where it sits (`temper/docs/plan/ui-authorize-red/inputs/request.md`).

## Parts and tasks

- **repair** (direct, slice 1, no walk): task `repair` (`tasks/repair.md`), high effort, about 120 minutes, needing nothing. It runs the story by name, alone, through heavy, and reads the restarted server's log with `secret=` filtered out. It fixes the cause at its lowest place:
  - a fixture's clock made relative to the run's clock;
  - or product code under a failing focused test first, in skein, jig or temper, wherever the cause sits.

  The story's checks stay as they are.

## Critical path

`repair` alone. It starts at the first push, and the plan lands with it.

## What was decided

- **One task, as the request asks.** The cause is unknown until the story runs, and this call runs on a machine that builds nothing. Splitting the work would only guess.
- **The lead's reading narrows the search and settles nothing.** The page is `State::Busy`, the client domain's state for jig's *not ready*, *busy* and *lost* answers alike. The spec lists where each can come from: jig's provider (`jig-protocol-engine-clients/src/provider.rs`), jig people's `Whose` and its start's readiness (`jig-domain-people/src/domain.rs`), and the client's link (`jig-client-protocol-link/src/authorize.rs`). It points out that an expired sign-in reads *signed out*, not *busy*. The browser world reads neither of finding-438's files.
- **The spec allows one slow-tier run, by name and alone:** `TIERS_WRAPPER=heavy tools/tiers/run temper browser -- -E 'test(=…)'`. A red's repair may do this under the lab's rules. The spec states it as a departure.
- **The fix may leave temper.** If the cause is in jig or skein, it is fixed and tested there (`AGENTS.md`, 5). The spec owns the likely jig and temper paths and pre-approves skein and the client's link.
- **Not reproduced is an outcome.** The cause may hold only part of the UTC day. If the story passes alone on the base, the task still looks for a day-bound value and pins one if found. If it finds none, it reports `Done: not reproduced` with two logs and their times, and after-landing runs the story again.
- **No design is written.** The design (`temper/docs/design/protocol/sign-in.md`, 9.3; `temper/docs/design/client/browser.md`, 4.5) already says what the story checks. A status line the fix makes stale is the task's one-line edit.
- **No `notes/common.md`.** With one task, the request's `For every task of this plan` section serves as the common note, and the spec cites it.

## Overlaps

Of the drafts beside this one, `ui-account-again` changes `jig/crates/jig-protocol-engine-clients/src/translate/accounts.rs`, `jig/crates/jig-domain-accounts/**` and `jig/crates/jig-domain-core/src/routing/party/account_requests.rs`. These are neighbours of the provider and of people, not their files. The spec allows the smallest edit there if the cause needs one, and asks for the overlap to be named in the report. `ui-ask-m1`'s branches are merged, and none touches the files this task owns. The queued drafts' files are not known yet.
