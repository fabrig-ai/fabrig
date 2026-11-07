# account-email

temper names an account by its email: `jig/docs/design/draft/account-facts.md`, section 6, steps 1 and 2, the look request `ready-account-email`, and a Claude subscription's email. The request: `inputs/request.md`. Step 3, usage windows, plan, renewal and next reset, is left out: the user is still deciding, and the format holds them.

## Parts and tasks

- **format** (`jig/docs/design/domain/accounts.md`, 3.7): `facts-record`, about 150 minutes. The record, its key, codec and registry.
- **claims** (`skein/docs/design/oauth.md`, 2.2): `oauth-profile`, about 120 minutes. A registration asks for `email` and `name`.
- **program-email** (`smith/docs/design/protocol/programs.md`, 3): `status-email`, about 60; `login-wire`, about 90; `program-report`, about 60. Claude Code's email, from its status to the account's fact.
- **line** (`jig/docs/design/client/domain/contract.md`, 5.1): `facts-line`, about 170. Both documents' rows, the audiences, the wire, the contract, and the journeys recorded again.
- **email** (`jig/docs/design/domain/accounts.md`, 3.7): `keep-facts`, about 130. Kinds 1 and 2 at sign-in and refresh.
- **look** (`temper/docs/design/client/README.md`, 3.5): `names-by-email`, about 110; `fake-says-email`, about 80; `email-close`, about 20. The look request `ready-account-email`, the story that runs both real sides, and the status files.

About 890 implementer-minutes in all.

## Critical path

`facts-record` (150), `facts-line` (170), `names-by-email` (110), `fake-says-email` (80), `email-close` (20): about 530 minutes. Beside it, `oauth-profile` (120) and `status-email` (60) start at once; `keep-facts` needs the record and the claims, `login-wire` needs the status, and `program-report` needs both of those.

**The meeting point** is `fake-says-email`: `a_terminal_sign_in_returns_to_the_real_listener` and `a_trial_runs_through_the_worker_and_the_fake_model` already run the real listener, engine and client, so each takes the email by a line or two, and no new story. No task but that one and the close waits for the whole.

## What the cut decided

- **Claude Code's status names an email.** Read on 2026-11-07, version 2.1.312: its `email` is there, by field names and shapes alone, and no value was printed. So kind 1, origin 5, and the Claude line reads it (`jig/docs/design/domain/providers.md`, 3.3). The draft's status says so.
- **A refresh's ID token has no signature check.** Its claims are read when `iss`, `aud`, `exp` and `sub` hold, and the token endpoint's TLS stands in (OpenID Connect Core, 3.1.3.7): the alternative fetches the issuer's keys at every refresh, for words to show.
- **`seen` is when a value was first said,** so that an hourly refresh rewrites no record and marks no document.
- **A fact's map stands apart from `Account`,** so that restore takes rows in any order, as its tests ask.
- **The email goes on a `Login` row, not in `LoginState`.** `LoginState::Usable` has sixty-odd readers, and none of them cares; `ProgramCheck::Usable`, with nine, takes the field.
- **The fleet says a program's email to the child by a request of its own,** `ProgramEmail`, so that only logins the fleet took are reported.
- **Who an account is reads one function,** `who`: the email, otherwise what it was named. An email beats a display: the request says temper names an account by its email, and a holder has no act that gives a signed-in account a display today.
- **The Claude line keeps the mock's words:** "Claude subscription · EMAIL, signed in on this machine", the email in the source's name's place.
- **`temper-service`'s Google sign-in keeps its own reader** of the same claims, off the kept record. Replacing it with skein's is another plan's; `oauth-profile` leaves it.
- **Formats change in place:** the accounts' family 2, the client wire 19 and the worker wire 9 keep their numbers (the user, 2026-11-04 and 11-06). No compatibility, no migration.

## Overlaps

- **`model-catalogue-3`** reads each provider's list per account, and writes `jig-domain-accounts`, the accounts' family, a worker's check and `temper-client-view`'s Settings. Whichever lands second catches up; a key tag moves to the next free one. `facts-record`, `keep-facts` and `program-report` are where it meets this plan; I couldn't keep them off those crates, since the record lives there.
- **`call-room`** moves `tool_payload` and the charter's bounds, and meets this plan nowhere that I can see but `temper-service`'s tests.
- **Journey prints** conflict with whichever plan re-records next: regenerate on a catch-up, never hand-merge.

## After landing

- The render check of `ready-account-email`, on a real sign-in and a real Claude Code login, files its findings back to fab.
- The browser and process stories whose words change run there, `temper/crates/temper/tests/fake_backends/walk/program_here.rs` among them.
- A real provider's ID token isn't proved here: that OpenAI's says `email_verified` as the draft reads it is for that check.
