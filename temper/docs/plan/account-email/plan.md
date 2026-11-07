# account-email

temper names an account by its email: `jig/docs/design/draft/account-facts.md`, section 6, steps 1 and 2, the look request `ready-account-email`, and a Claude subscription's email. The request: `inputs/request.md`. Step 3, usage windows, plan, renewal and next reset, is left out: the user is still deciding, and the format holds them.

## Parts and tasks

Released:

- **format** (`jig/docs/design/domain/accounts.md`, 3.7): `facts-record`, about 150 minutes.
- **claims** (`skein/docs/design/oauth.md`, 2.2): `oauth-profile`, about 120 minutes.
- **program-email** (`smith/docs/design/protocol/programs.md`, 3): `status-email`, about 60 minutes.

To come: the client's line, jig keeping the facts at sign-in, refresh and a program's report, a host's login carrying its email, temper naming an account by it, the fakes saying one, and the status files' close.

## Critical path

As it stands: `facts-record` (150), then the client's line, temper's naming, the fakes and the close.

## What the cut decided

- **Claude Code's status names an email.** Read on 2026-11-07, version 2.1.312: its `email` is there, by field names and shapes alone, and no value was printed. So kind 1, origin 5, and the Claude line reads it (`jig/docs/design/domain/providers.md`, 3.3). The draft's status says so.
- **A refresh's ID token has no signature check.** Its claims are read when `iss`, `aud`, `exp` and `sub` hold, and the token endpoint's TLS stands in (OpenID Connect Core, 3.1.3.7): the alternative fetches the issuer's keys at every refresh, for words to show.
- **`seen` is when a value was first said,** so that an hourly refresh rewrites no record and marks no document.
- **A fact's map stands apart from `Account`,** so that restore takes rows in any order, as its tests ask.
- **Formats change in place:** the accounts' family 2, the client wire 19 and the worker wire 9 keep their numbers (the user, 2026-11-04 and 11-06). No compatibility, no migration.

## Overlaps

- **`model-catalogue-3`** reads each provider's list per account, and writes `jig-domain-accounts` and the accounts' family. Whichever lands second catches up; a key tag moves to the next free one.
