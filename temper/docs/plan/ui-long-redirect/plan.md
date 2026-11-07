# ui-long-redirect: the plan

A ChatGPT sign-in whose answer is long still signs in (the request,
`inputs/request.md`). The cause is skein's accounts driver bounding the returning
redirect by the client's `url_bytes`, which jig's small profile holds at `ADDRESS`,
2,048, the bound of the address the box sends out. The fix is in skein and jig; temper
changes nothing but takes the new profile.

The design this plan wrote: `skein/docs/design/oauth.md`, 6.3, 6.5 (*an over-long
redirect*), 6.7 (the redirect bound, the listener's head, the application's name) and
6.8 (*Focused rows*); `jig/docs/design/protocol/configuration.md`, 6 (*LLM accounts'
bounds*). Their status files carry a **Not yet** bullet per task, which the task
deletes. The shapes that cross tasks and trees are `notes/contracts.md`'s.

## Parts and tasks

| part | task | tree | needs | what |
|---|---|---|---|---|
| bound | `redirect-bound` | skein | — | `Limits.redirect_bytes`, `redirect_room`, `listener_head`, `BROWSER_FIELDS`; the listener and the paste bound by it; the failing rows of 1,900- and 3,900-byte codes first, then green; a browser's real header set and a cookie at the bound |
| profile | `driver-profile` | jig | `redirect-bound` | small's driver: `redirect_bytes` 16,384 against its room (13,407), `server.head` from `listener_head` (24,593, from 4,096); `driver_fits` and the whole check refuse a driver that holds less |
| long | `long-redirect` | skein | `redirect-bound` | the over-long redirect of the sign-in's own state: the page "This sign-in's answer is longer than temper accepts. Go back to temper and start the sign-in again.", 414, and the sign-in failed at once as `LongRedirect { length, bound }`; the refusal page kept for a redirect that doesn't match |

The request's tasks 1 and 2 are one task, `redirect-bound`: its rows are written and
seen failing first, then turned green in the same branch, so the trunk is never red.
Task 3 is split: the browser's request row and `listener_head` are skein's
(`redirect-bound`), sizing small's head is jig's (`driver-profile`). Task 4 is
`long-redirect`.

## Critical path

`redirect-bound` → (`driver-profile` ∥ `long-redirect`). The person's sign-in works once
`redirect-bound` lands (small's driver already takes `redirect_bytes` 16,384 there, and
a 2,095-byte redirect with a browser's usual fields fits today's 4,096 head);
`driver-profile` makes room for redirects up to the bound with cookies beside them.

## Decisions

- **The bound is the driver's, not the client's.** `redirect_bytes` sits in
  `skein_oauth_accounts::Limits`, beside the listener's limits, since the driver alone
  reads a returning redirect; `skein_oauth::ClientLimits` and its `url_bytes` stay as
  they are. Every other driver keeps today's behaviour: its `redirect_bytes` is its
  client's `url_bytes`.
- **The room counts the issued client id** besides the four parts the request names:
  an issuer that issues its client id sends `client_id` with the code (oauth.md, 2.1).
  Small's 16,384 holds the room, 13,407, with space for codes to grow again.
- **The listener's page names the owner's application.** skein names no application,
  and smith's local service runs the same listener; so `Component::new` takes the
  application's name, which jig passes from its own (`temper`, from temper's
  `ACCOUNT_APPLICATION`) and smith gives as `smith`.
- **Only the sign-in's own over-long redirect ends it:** the listener reads its one
  `state` field and the client compares it in constant time (`awaits_state`). Anything
  else past the bound gets the refusal page and the sign-in still waits, as the design
  asks so that a stray local request cannot end a person's sign-in.
- **A paste past the bound stays refused** as another address: a paste has its own
  terminal, and the person pastes again.
- **The length and the bound stop at jig's boundary.** The sign-in's terminal names
  them (`Failure::LongRedirect`); jig carries it as `SignInAgain`, which temper shows as
  needing a sign-in, as jig already carries an oversized record without its size.
  Showing the numbers on temper's page would change the client wire in place and
  re-record journeys while `tui-approval`'s wire change lands: left for a later request.

## What changes in place

No stored format, client wire or refusal vocabulary changes. Public Rust interfaces
and a profile change in place (`notes/contracts.md`, section 4); a heads-up for
hearth/core and temper/migration on jig's and skein's shared parts:

- `skein_oauth_accounts::Limits` gains `redirect_bytes` (every literal of it must set
  one); the crate exports `BROWSER_FIELDS`, `redirect_room`, `listener_head`,
  `APPLICATION_BYTES`.
- `skein_oauth_accounts::Component::new` takes the application's name.
- `skein_oauth_accounts::Failure` gains `LongRedirect { length, bound }`; an exhaustive
  match of it needs an arm.
- `skein_oauth::Client` gains `awaits_state`.
- jig's small profile, and standard through it: the driver's `redirect_bytes` 16,384,
  its listener's `server.head` 24,593 (from 4,096). temper's fake issuers that copy
  `profile.driver.server` take the larger head with it.

## Overlaps with the drafts beside this one

- `long-redirect` edits two jig files `tui-approval` and `ui-stop-coding-row` don't
  list (`jig-protocol-accounts`' `translate.rs`, `moves.rs`, `component.rs`); none of
  this plan's files is in their lists. `driver-profile` edits `jig-protocol-config`'s
  `overrides.rs`, `domain.rs` and `profile/small.rs`, where `tui-approval/config` edited
  `clients.rs` and `members.rs` alone.
