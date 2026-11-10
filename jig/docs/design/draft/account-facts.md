# What a provider says about an account

Draft, 2026-11-04, from a session with the user that day. An LLM
account holds what jig decides about it: its provider, holder,
subject, display, sources and uses (`../domain/accounts.md`, 3.1).
What the provider says about it is kept nowhere: not the signed-in
account's email, which temper's Ready cannot name ("ChatGPT plan" and
nothing more), nor its plan, its subscription's renewal, or how much of
its usage limits is spent. This draft fixes the store's format for
those facts, once, so that each provider's facts can be added later
without changing it. Reading and showing them come after (section 6).

## 1. In one page

- **Facts beside the account, not in it:** a record of their own per
  account, apart from the account's record, so that facts change as
  often as a provider reports them while the account's record changes
  only at people's acts (section 2).
- **The email is one of them:** a signed-in account's verified email,
  refreshed at each sign-in and refresh, apart from the display its
  holder gives, and never its identity, which stays the validated
  `sub` (`../domain/providers.md`, 2.2). The user chose a place of its
  own for it over filling the display (2026-11-04).
- **A handful to start:** the email, the person's name, the plan, the
  workspace, the subscription's renewal or end, and each usage window
  with how much of it is used and when it resets (section 3).
- **An open list of typed facts:** each fact a kind the design
  registers, a value of a few generic shapes, when the provider said
  it and how. A new kind is a line of the registry, not a change of the
  format (section 4).
- **Each kind names its audience:** its holder alone, or whoever sees
  the account; the client's line carries what its reader may see. The
  user took section 3's audiences as they stand (2026-11-04).
- **Changed in place:** until v0.1.0 there is no supported deployment,
  so the accounts' store version and the client protocol change in
  place (the user, 2026-11-04; `../protocol/store/records.md`).
- **The format now, the facts later:** nothing writes a fact until a
  provider's adapter learns it; the email is the first (section 6).

## 2. A record of its own

- **Why apart:** the account's record is jig's decisions, rewritten at
  a person's act; facts are the provider's word, and usage changes at
  each limit a call meets. Apart, a usage report rewrites a small
  record, and the account's format never moves for a provider's news.
- **Keyed by the account,** in the accounts' family, the next key tag
  after the issued sign-ins (`../protocol/store/README.md`, 4.4): at most one
  record per account.
- **Gone with the account;** an account moved to another deployment
  (`../domain/accounts.md`, 6) moves without its facts, which its next
  sign-in or call reports again.
- **Not a secret:** the ID token stays in the secrets store
  (`../protocol/store/secrets.md`); a fact keeps only what the client may
  show.

## 3. The first facts

| Kind | Name | Shape | Audience | From |
|---|---|---|---|---|
| 1 | email | text | holder | a signed-in source's ID token, its `email` with `email_verified` true |
| 2 | name | text | holder | the ID token's `name` |
| 3 | plan | text | account | the provider's name for the plan ("Plus", "Max") |
| 4 | workspace | text | account | the organization or workspace the account works in, where it has one |
| 5 | renews | instant | holder | when the subscription renews |
| 6 | ends | instant | holder | when it ends, not renewing |
| 7 | usage | window | account | a usage window, named by the provider ("five_hour", "weekly") |

- **Where each comes from** is each provider's design, in its adapter:
  OpenAI's ID token claims, of which skein reads none beyond `sub`
  today (skein's `oauth/README.md`, 2.2); Claude Code's status and its
  rate-limit events (smith's `protocol/programs.md`, 5.6); a limit
  error's reset (skein's `llm/README.md`, 4.8).
- **A fact no source reports is absent,** never guessed, as a reset is
  never guessed (`../domain/providers.md`, 2.6).
- **Origins** are registered alike: 1 a sign-in, 2 a refresh, 3 a
  call's answer, 4 a limit error, 5 a program's status or event, 6 a
  provider's usage read.

## 4. The format

```
AccountFacts
  account     the account's number
  facts       list, at most 24

Fact
  kind        u16, the registry's (section 3)
  label       bytes 32: which one, where a kind has several (a usage
              window's name); empty otherwise
  value       Value
  seen        wall time the provider said it
  source      option: the account's source it came through
  origin      u16, the registry's (section 3)

Value, one of
  text        bytes 255
  count       u64
  instant     wall time
  span        nanoseconds
  share       u16, in basis points, 0 to 10000
  window      length: option span; used: option share;
              resets: option instant; state: u16, the registry's
              (1 open, 2 warned, 3 limited)
```

- **What is fixed and what grows:** the schema fixes the shapes, few
  and generic; the design's registry grows the kinds, the origins and a
  window's states, with no schema change. A new shape (a sum of money,
  say) is the one change the format still needs.
- **A fact is its kind and label:** a new report replaces the fact of
  the same kind and label. A sign-in replaces every fact a sign-in or
  refresh gave, so that one it no longer reports is gone. A window past
  its reset stays as reported until the next report; its client says
  when it reset.
- **Bounds:** 24 facts, a text of 255 bytes, a label of 32: a record of
  under 8 KiB, far within the largest value (`../protocol/store/records.md`,
  4).
- **Unknown kinds:** a kind, origin or state a build does not register
  is kept as it is and sent to no client.
- **To the client,** the accounts document's line for an account
  (`../client/domain/contract/documents.md`, 1) gains its facts, each kind's
  audience applied to the reader: a holder's kinds to its holder alone.

## 5. The format's choice, for the user's review

Three ways were weighed:

- **Fields on the account:** plainest to read, but each new fact
  changes the account's schema, and each usage report rewrites the
  account's record.
- **A blob per provider,** its bytes the provider's or JSON: never a
  schema change, but no client shows it without that provider's
  parser, nothing bounds or checks what is in it, and the store's
  codecs refuse what they cannot read for a reason (skein's
  `data/codec/README.md`, 3 and 4).
- **A typed open list,** section 4: the schema fixes a few shapes, the
  design registers kinds, every client shows a registered kind the
  same way, and each value is bounded and checked. Its cost: a kind's
  meaning lives in the registry rather than in a type, and a new shape
  still changes the format.

The draft takes the third.

## 6. Its order

1. **The format:** the record, its key and codec, the client's line
   with the audiences; nothing writes it yet.
2. **The email:** skein's OAuth reads the verified email claims its
   caller asks for; jig keeps kinds 1 and 2 at sign-in and refresh;
   temper names the account by its email (temper's look request
   `ready-account-email`).
3. **Each provider's other facts,** as its adapter learns them: Claude
   Code's usage windows from its rate-limit events first, since it
   already reports them.

## 7. Later

- **A provider's usage read,** polled, where one is documented.
- **Money:** credits or a balance, a new shape.
- **Choosing an account by its headroom** (`../domain/accounts.md`,
  4.3), once usage windows are kept.

## 8. Open questions

None.
