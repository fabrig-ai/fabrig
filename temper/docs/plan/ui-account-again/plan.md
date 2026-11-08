# ui-account-again: plan

A person forgot a Claude Code program account. When they declared it
again under the same name, temper refused it: "This act is not offered",
exit 2 (`inputs/request.md`). The cause is in jig's accounts child. A
removed program source's name stays a durable reservation, and
`programs::declare` refused every name that matched one
(`DeclarationRefusal::Issued`, then `Refusal::Uses`, which the core maps to
*not offered*). The plan lets the reservation's own holder declare the
name again, refuses a name that is really taken as *name taken*, and
words the account-act refusals that have one reason a person can act on,
in ctl and on the web.

## Parts and tasks

- **redeclare** (jig): task `redeclare`, high, about 75 minutes, no needs.
  The accounts child revives a retired reservation for its own holder:
  it moves the reservation to the new source, erases the old row, adds
  no lifetime name and pushes the hosts' check. A live name, or another
  holder's, is refused `Refusal::NameTaken`, and the core maps that to
  `people::Refusal::NameTaken`. Its first row reproduces the finding on
  the accounts child's world: declare, forget, declare again.
- **ctl-words** (temper-client-tools): task `ctl-words`, high, about 50
  minutes, no needs. Three refusals get their reason and next step in
  text and JSON: a declaration refused *name taken*, a declaration refused
  *not offered*, and a retry refused *not offered*. They are rows of one
  new table test in `account_acts.rs`.
- **web-words** (temper-client-view, temper-client-domain): task
  `web-words`, high, about 60 minutes, no needs. The web gets the same
  three cases in its words, as rows of one table test.
  `ConnectModel`'s recovery of the person's own name moves from *not
  offered* to *name taken*.
- **story** (temper/tests/client): task `again-story`, high, about 50
  minutes, needs `redeclare` and `ctl-words`. This is the meeting point:
  through the real engine, declare, forget, declare again (exit 0, host
  checked), then declare the live name (exit 2, ctl's *name taken* words).

One slice, no walk.

## Critical path

`redeclare` (75) and `ctl-words` (50) start at the first push, beside
`web-words` (60). `again-story` (50) follows them. The path is about 125
implementer-minutes.

## What was decided

- **Four tasks, not one.** The request gives one task, but the fix is in
  jig and the words are in temper. A branch stays in its tree, and ctl
  and the web are separate crates with separate writers. The contracts
  note (`notes/contracts.md`) fixes every shape and word that crosses
  tasks, so three tasks start at once. The story is the one place where
  both halves meet.
- **The reservation stays, and its holder may take it back.** The
  reservation exists so that a directory a person signed in on a host
  keeps meaning their source (jig's `domain/accounts.md`, 3.4). That
  holds as long as the name is never another holder's, so the rule
  changes from "never reused" to "never another holder's". A revival
  moves the reservation to the new source's number, since restore links
  a live program source to the reservation at its own number and panics
  on a name reserved twice. It counts no further lifetime name. It pushes
  `CheckProgram` because the hosts' list of names does not change, so no
  host would check on its own. That check is the request's "its 'it
  works' check runs as for a first declare".
- **A revival during the old source's sign-out is refused,** as a
  concurrent check is (`Limit { Sources }`). The window closes before the
  forget is answered.
- **No new refusal on the wire.** `NameTaken` already crosses every layer
  (store, engine-clients, client wire, client contract). jig's design now
  gives it a second meaning, a program source's name, in
  `client/domain/contract.md`, section 4. No codec, schema or generated
  file changes, and no stored format changes: erasing the old
  reservation's row is an existing record operation. jig-client's generic
  word for `NameTaken` ("Another project has that name") is left alone,
  since changing it would churn goldens across both trees. temper words
  the declaration's case itself, and jig's own client view offers no
  declaration.
- **Item 3's scope: three cases.** jig's `domain/accounts.md`, section 5,
  now says what *not offered* means for a declaration (the provider
  offers no program login) and for a resume (the source is not limited).
  The clients word those, and *name taken*. A key's add or replace, and a
  sign-in's start or finish, can be refused *not offered* for several
  reasons in the core's key routes. Those keep today's word, because no
  client can name one reason for them.
- **The finding's other facts** (the worker reporting `claude-6` usable,
  with `cleanup true`, and provider 2 unavailable) follow from the
  refusal. Once revived, the name's cleanup is false, the hosts check the
  login, and readiness follows the hosts' answers.

## Design written by the cut

- `jig/docs/design/domain/accounts.md`: 3.4 (*A program source's name*)
  and section 5 (the *declare a program's login* and *order, uses,
  resume* rows); `accounts.status.md`, one **Not yet**.
- `jig/docs/design/client/domain/contract.md`: section 4 (*name taken*,
  *not offered*); `contract.status.md`, one **Not yet**.
- `temper/docs/design/client/tools.md`: 5.5 (*A refused account act says
  why and what next*); `tools.status.md`, one **Not yet**.
- `temper/docs/design/client/README.md`: 5.5 (*A refused account act*);
  `README.status.md`, one **Not yet**.

Each **Not yet** is deleted by the task that closes it, as its spec says.

## Overlaps

- `ui-ask-m1` (integration) changed `temper/docs/design/client/README.status.md`,
  `temper/crates/temper-client-view/src/tests/fixtures.rs` and the gallery.
  This plan adds and later deletes one entry in that status file, and
  `web-words` may add a helper to that fixture. Both are small additions;
  neither can be moved off the file.
- `survey-answer` (smith) and `temper-fuzzy-gate` (`tools/tiers/budgets.toml`)
  share no file with this plan. The queued drafts' files are not known yet.
