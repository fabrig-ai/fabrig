# LLM accounts: an account's list and what a provider says

## 1. An account's list and what a provider says

Each LLM provider says what it lists, and an account's list says what it serves
(providers.md, 2.4). The accounts child says when an account's is read and keeps
what its last read gave; what a read does to the catalogue is `engine/settings-catalogue.md`, 2.

- **Which accounts have one:** an account of a provider whose configuration gives
  `keep` (`../protocol/configuration-engine.md`, section 1), whose first source in its
  holder's order is a sign-in or a key. A program's source has none: Anthropic
  allows no other use of a subscription's credentials (providers.md, 3.1), and the
  order decides who pays, so an account that lists Claude Code first and a key
  after has none. Nor has a source of none.
- **When it is due:**
  - as the account becomes usable: its first source turning usable, from any
    state but a restored one, so that a restart is no reason to read;
  - a day after its last attempt, kept as a wall time across restarts; a day past
    while the engine was down is due at the start;
  - when a holder, or an owner for a deployment's account, asks: **read again**,
    which names a provider and reads every account of it they may act on
    (`accounts.md`, section 5), the answer saying how many.

  An account being read isn't read a second time. A source not usable holds what
  is due until it is: nothing is asked of a limited or a refused source.
- **How it is read:** the child asks with the account's source's grant, by its
  number and generation as a run's, and hears one answer. The core gives it to the
  fleet (`engine/settings-catalogue.md`, 2), and a worker reads it (`hosts/README.md`, 5.1). It is no
  run, and spends no budget and no slot. It goes to the provider's `direct`
  endpoint where the provider names one, else to its `endpoint`
  (`../protocol/configuration-engine.md`, section 1): a key's read is the direct way's,
  never through a program's endpoint, so that an Anthropic key reads Anthropic's
  list directly.
- **What the store keeps** on the account's record (`../protocol/store/keys.md`, 1.4):
  - `names`: which of the catalogue's models its last good read gave, a bit per
    setting, as a task's choice holds a number (`engine/settings-catalogue.md`, 1);
  - `read`: when that read was, or none;
  - `tried`: when its last attempt ended, or none;
  - `failure`: why its last attempt failed, or none: refused with a status, the
    provider unreachable, its list unreadable, a deadline, a worker's fault, or
    the catalogue's room;
  - `left_out`: how many names of its last good read no number was free for.

  A failed read changes `tried` and `failure` alone. No credential, body or
  provider's words are kept, and a failure's words are the client's.
- **What it decides:** whether this account can run a model, for the offer (`engine/views.md`,
  1) and for a task given no choice (`accounts-uses.md`, 1.3). An account **allows** a model of
  its provider when:
  - a person added the model, whatever any list says (`engine/settings-catalogue.md`, 1); or
  - it has a good read, and `names` holds the model; or
  - it has no good read: no list, none read yet or only failures, and the model
    is the stock's.

  So a failure keeps the names of the last good read, and an account whose first
  read fails takes the stock.

What an LLM provider says of an account is kept apart from what jig
decides about it (`accounts.md`, 3.1): the account's holder and uses change at people's acts,
its facts as often as a provider reports.

```
AccountFacts
  account     the account's number
  facts       at most 24

Fact
  kind        u16, the registry's
  label       at most 32 bytes: which one, where a kind has several (a
              usage window's name); empty otherwise
  value       Value
  seen        the wall time the provider first said this value through
              this source with this origin
  source      option: the account's source it came through
  origin      u16, the registry's

Value, one of
  text        at most 255 bytes
  count       u64
  instant     a wall time
  span        a duration
  share       u16, in basis points, 0 to 10000
  window      length: option span; used: option share;
              resets: option instant; state: u16, the registry's
```

- **The registry** is this section's. A new kind, origin or window state
  is a line here, and no change of the format; a new shape is the one change
  the format still needs.

  | Kind | Name | Shape | Audience | Said by |
  |---|---|---|---|---|
  | 1 | email | text | holder | a signed-in source's ID token, its `email` with `email_verified` true; a program's status, its `email`, on a usable answer |
  | 2 | name | text | holder | a signed-in source's ID token, its `name` |
  | 3 | plan | text | account | nothing yet |
  | 4 | workspace | text | account | nothing yet |
  | 5 | renews | instant | holder | nothing yet |
  | 6 | ends | instant | holder | nothing yet |
  | 7 | usage | window | account | a program's `rate_limit_event`, origin 5 (smith's `protocol/programs-claude-code.md`, 1.6); a program's usage read, origin 6 (*A usage read*, below) |

  Origins: 1 a sign-in, 2 a refresh, 3 a call's answer, 4 a limit error, 5 a
  program's status or event, 6 a provider's usage read. A window's states:
  1 open, 2 warned, 3 limited.
- **A fact is its kind and label.** A new report replaces the fact of the
  same kind and label. A sign-in first drops every fact a sign-in or refresh
  gave, so that one it no longer reports is gone; a refresh or a program's
  report replaces only what it says. A report that says what the fact already
  holds, in value, source and origin, writes nothing, so that `seen` is when
  the provider first said it and an hourly refresh rewrites no record and marks
  no document.
- **A fact no source reports is absent,** never guessed. A claim that is
  empty, past its bound or holds a control byte is not reported (skein's
  `oauth/README.md`, 2.2; smith's `protocol/programs.md`, section 3). A report that
  would pass 24 facts keeps those already held and drops the new one.
- **A program's report:** a host's `logins` naming an email for a usable
  login (`../protocol/channel/records.md`, 1) reports kind 1, origin 5, through
  that program source. A login that lapses or a status that stops naming one
  removes nothing, and a later report replaces it, from whichever host. A
  source's facts are the account's: when two hosts' logins of one source name
  two emails, the latest report stands (`accounts.md`, 3.3).
- **A program's usage window:** a window a run's program reports
  (`../protocol/channel/records.md`, 1, `usage`) is kept through the run's
  source as kind 7, labelled with the window's name, origin 5; a usage read's,
  through the source read, origin 6. A report replaces only the window it
  names, and a field it leaves absent, its share, length or reset, keeps the
  one held for that label, so that a report equal to what is held writes
  nothing.
- **A usage read,** when no run speaks: the program's own usage command, run on
  a host that keeps the login (smith's `protocol/programs-claude-code.md`, 1.11):
  - *Which accounts:* one with a program source; its first program source in
    its holder's order that is usable is read.
  - *When it is due:* 30 minutes after the latest of a kind 7 report for the
    account, a run's or a read's, equal or not; the end of its last read; and
    the moment its program source turned usable in this process, the engine's
    start counting as one. And once at the reset of each window held limited,
    the held facts read at the start too. A source not usable holds what is due
    until it is, and an account being read isn't read again.
  - *How:* the core gives the fleet the account's read number and the source's
    name. The fleet holds it until a ready worker whose last `logins` names the
    source usable speaks a version with `read usage`, sends it there, one usage
    read in flight per worker, and holds it again if that worker's channel is
    lost before it answers. The host's half runs it (`hosts/README.md`, 5.1).
    No budget, no slot, no turn and no model call.
  - *Its answer:* each window a report, as above. A failure keeps the facts
    held, the next read due 30 minutes after it. A read *spent*, whose program
    used the model, isn't asked of that account again until the engine
    restarts.
  - *Nothing is stored:* due times and reads in flight are memory, and a
    restart starts them again.
- **A window's rate,** its pace, is the core's memory beside the fact, never
  stored, and a restart starts it again:
  - *Readings:* for each kind 7 fact with a share, at most 13, each a wall time
    and a share. A report keeps the fact's share after it as a reading when the
    newest is at least 5 minutes older, or none is kept, and those more than 60
    minutes older than the newest go. A share lower than the newest reading's,
    a reset, clears them first. A fact gone takes its readings.
  - *The rate,* a share per hour in basis points, saturating: from the oldest
    reading to the newest when the oldest is at least 30 minutes older;
    otherwise the window's average since it opened, its share over the time
    from its reset less its length to the newest reading, where both are held
    and that time is above zero; otherwise none. It is computed as the readings
    change, and served beside the fact (`../client/domain/contract/documents.md`, 1).
  - *Marks:* a rate that changes marks the accounts documents that show the
    account, as a fact's change does; one that doesn't marks nothing. So an
    equal report may move a window's rate at most once every 5 minutes.
- **A source gone** takes the facts that came through it, sign-in and refresh
  or program's alike, with the record rewritten in the decision that removes
  it.
- **Unknown kinds:** a fact whose kind, origin or window state this build does
  not register is kept as it is and sent to no client.
- **Who reads a fact:** a kind's audience is its holder alone, or whoever
  sees the account; `../client/domain/contract/documents.md`, 1 applies it to each
  document's reader. For the deployment's accounts the holder's readers are
  its owners (`accounts.md`, 3.2).
- **Gone with the account.** An account moved to another deployment moves
  without its facts (`accounts.md`, section 6): the refresh that follows a move in, or its
  next sign-in, reports them again.
- **Not a secret, and no identity.** An email never tells accounts apart
  (`accounts.md`, 3.3), never opens a session, and is never checked against a subject. The ID
  token stays in the secrets' store (`../protocol/store/secrets.md`).
- **Kept as:** `../protocol/store/keys.md`, 1.4, and `../protocol/store/README.md`, 10.9.
