+++
id = "has-goals"
part = "has-goals"
kind = "feature"
needs = ["goals-domain"]
files = [
  "temper/crates/temper-client-domain/src/pages/goals.rs",
  "temper/crates/temper-client-domain/src/pages.rs",
  "temper/crates/temper-client-domain/src/domain.rs",
  "temper/crates/temper-client-domain/src/tests/pages/goals.rs",
  "temper/crates/temper-client-domain/src/tests/fixtures.rs",
  "temper/crates/temper-client-domain/src/pages/delegation.rs",
  "temper/crates/temper-client-shell-web/src/limits.rs",
  "temper/crates/temper-client-tools/src/limits.rs",
  "temper/crates/temper-client-shell-terminal/src/limits.rs",
]
effort = "high"
minutes = 80
+++

# Whether a project has a goal

## Goal

`Domain::has_goals(project)` says whether the open project has a goal, whatever page is open. The rail, `ui-home-status` and `ui-away` read it.

Acceptance:

- As `temper/docs/plan/ui-goals/notes/contracts.md`, The fact.
- From a live watch of `goals::query` with window 1 and one ended read of it (`QueryOrder::Ended`, `phase: Ended`), for the open project (`Pages.project`), from two handles
  fixed for the session as `accounts`' is (`u64::MAX` is theirs; these are `u64::MAX - 1` and `- 2`), so that a page's `pages.next` never meets them.
- Moving to another project, or to none, unwatches and starts again; sign-out and out-of-date release them. A successful sign-in asks the read again, since reads aren't
  replayed (`pages.rs`, `awaits_first_read`'s comment).
- A live goal, or an ended one: true. No template, no project, not delivered, or both empty: false.
- The fact stays true across pages: a test walks Home, Goals, a chat and back, one watch throughout.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/client/README.md`, 5.3, the rail's "Goals".
- `temper/crates/temper-client-domain/src/pages/accounts.rs`, `configure`; `pages/chats.rs`, `configure`; `pages.rs`, `step`.

## Owns and may change

Owns, as `files` lists them: a small struct in `pages/goals.rs`, private to `pages`, held on `Pages`, reconciled in `step` beside `chats.configure`.

May also change, each only for this fact's Watch and Read among a test's expected requests, or its answer to them:

- `temper-client-domain/src/tests/**`, `temper-client-view/src/tests/**`, `temper/tests/client/**`, `temper/tests/one-process/**`, `temper/tests/playground/**`, `temper/crates/temper/tests/**`.

Options, each pre-approved:

- **Limits:** this adds 2 handles and 1 watch to every page. Raise `handles` and `watches` in web, tools and terminal together by exactly those, and reserve one more in
  `delegation::bound` (its pin in `tests/pages/delegation.rs` moves with it, and its comment names this fact). These are bounds of the client's tables, not budgets. The tests' `fixtures.rs` limits may rise likewise.
- If a test counts requests in a way that one more breaks in many files, give the fixture's start its answer once in the fixture; say which.
- A template-less `me` sends nothing: fixtures without a goal template see no change, so expect few to move.

## Steps

1. **The fact,** then `Domain::has_goals`.
2. **Tests,** `tests/pages/goals.rs`: `has_goals_stands_across_pages_on_one_watch`; `an_ended_goal_counts`; `no_template_sends_nothing`;
   `another_project_starts_again`; `a_sign_in_asks_the_read_again`.
3. **A change of meaning:** every page holds two more handles. Run the readers: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-world -p temper-client-shell-terminal -p temper-client-tools`.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report, with the limits as they stand and how many tests moved.
