# LLM accounts: sources and states

## 1. Sources

| Kind | What the application holds | Who runs the agent's loop | Usable where | Renewed by |
|---|---|---|---|---|
| *signed in* (`sign_in`) | a record from its own OAuth registration, in the secrets' store | smith, over skein's LLM client: the direct backend | on any host: its access token goes with the run as a grant | the accounts' component, before it lapses |
| *key* (`key`) | a key, in the secrets' store | smith, the direct backend | on any host, the same way | nobody: its holder replaces it |
| *program* (`program`) | nothing: a program's own login, in the application's configuration directory for the source on each host | the program, adapted by smith's agent: the program backend (smith's `protocol/programs.md`, section 2) | on a worker whose host's half reports its login usable, and that the host rule (below) allows | the program |
| *none* (`none`) | nothing | smith, the direct backend | where its endpoint is reachable, such as a local model on loopback | — |

- **Later kinds fit the same table,** such as an exchange of a cloud
  provider's credentials for short-lived tokens.
- **The holder orders an account's sources.** A run takes the first
  that is usable on the host it is placed on, and passes to the next
  only when the earlier one is unusable there alone: a program's login
  needing sign-in on that host, or kept on another, or, for one task,
  a login its program started there without its tools
  (engine/fleet.md). A source unusable on
  every host (limited, refused, needs sign-in, misconfigured, unloaded:
  2) holds the account's runs until its holder acts on it, resumes it
  or reorders the sources, since the order decides who pays
  (providers.md, 2.5 and 3.4). An Anthropic account lists Claude Code,
  then a key, by default.
- **The source picks the endpoint.** A model whose endpoint names a
  program takes its provider's program sources on that endpoint, and its
  sign-ins, keys and *none* on its provider's `direct` endpoint where the
  provider names one (`../protocol/configuration-engine.md`, section 1),
  passing them over where it names none. A model whose endpoint names no
  program takes sign-ins, keys and *none* on that endpoint, and no
  program source. So one model is reached either way, and the order
  above decides which: an Anthropic account of Claude Code then a key
  runs a Claude model on Claude Code on a host where the login is
  usable, and on the key, directly, on a host where the login is
  unusable there alone; a login held on every host, limited among them,
  holds the account's runs, and nothing of the key is spent.
- **A grant names its source.** A value's grant carries its value and
  how it is sent, a bearer or a key (`../protocol/llm.md`, section 6). A
  program's grant carries none: it names the source, by whose name the
  host's half finds the program's configuration directory, and the agent
  checks the login again as the run starts (smith's
  `protocol/programs.md`, section 3).
- **A program is named** in the LLM provider's configuration, and in the
  agent's `programs`: `claude_code`, Claude Code, whose executable is
  `claude` (smith's `protocol/agent.md`, section 4).
- **A program's source on each host.** A worker's logins follow its
  hello: the engine sends it the program sources it may be given, by
  name, after the hello (`../protocol/channel/README.md`, 4.3), and places no
  run on it until the first `logins` that answers them. The host's half
  then checks each login when told of a new source, on its holder's
  check (`accounts.md`, section 5) and at a configured interval
  (`../protocol/configuration.md`, section 4), and reports what changed.
  A check answers usable, not signed in, signed in otherwise (another
  way of paying than the holder's plan) or failed (smith's
  `protocol/programs.md`, section 3), and the deployment's document
  shows each host's answer. A holder's check withdraws the source's grant;
  a source held Refused is not granted again while so held (Resume clears
  only Limited holds). Withdrawal records no fact: facts are expendable. Every notice names the worker's whole current
  source list, without login contents; the fleet places a run on a
  program's source only where its login is usable (engine/fleet.md). The engine's
  own slots run `smith-inline-agent`, which has no program backend, so
  they never take a program's source.
- **A program source's name** is 1 to 32 characters of `a-z`, `0-9` and
  `-`, given by its holder as they declare it, unique in the deployment,
  never changed, and never another holder's once given. A durable
  reservation keeps its source number, name and original holder after
  removal. Its original holder may declare the name again once its
  source is removed, as a person who signed out signs back in: the
  declaration makes its account, or joins one, as any declaration does,
  and moves the reservation to its new source, whose number it takes,
  so that the name is wanted signed in again and counts no further
  lifetime name; the old reservation's row is erased in the same
  decision, and each host checks the login at once, as on its holder's
  check (`accounts.md`, section 5). While the old source's sign-out is still under way,
  the declaration is refused as a check is while another is out. A
  name a live source holds, or another holder's reservation, is refused
  *name taken* (`accounts.md`, section 5). Both shipped profiles admit 12 lifetime
  names. Restoring an existing live source reserves
  its name before admitting changes; a conflicting reservation is a
  writer's bug, a link that cannot be made, and refuses the start,
  naming the record and what it links to (engine/restart.md).
  Exhaustion refuses a declaration before
  reserving its numbers or writing any row. It, not the account's
  number, names the configuration directory, so that a directory a
  person signed in on a host keeps meaning that source, and a
  developer's own fixed directory serves each fresh deployment a live
  layer's test makes.
- **The host rule.** A host keeps a person's program logins when its
  half reports a login for a program source of theirs, in any state but
  needs sign-in. Runs, and a program's own tools, run as the host's
  user, so a run can read every login kept on its host until a run's
  process tree is contained from the programs directory (hosts/README.md, 5.1;
  smith's `domain/tools.md`, sections 1 and 5), and a subscription
  serves one person. So a host that keeps a person's program logins
  takes only runs that person's accounts pay for, until a run's tree is
  contained from them: the fleet places no other run there (engine/fleet.md
  ), and the deployment's document says why. One person's
  several logins on one host change nothing, and a deployment of one
  person is unaffected. Deployment-held logins restrict no run; only runs
  bound to their account can use their source.
- **Signing out or removing a program's source** has each worker run the
  program's own sign-out in the source's configuration directory (smith's
  `protocol/host.md`, section 2); the answer names any host that did
  not. The answer covers the configured workers, named by their configured
  names: one already offline is not signed out (not in contact), and a channel
  lost during the operation is lost. A worker removed from configuration is
  not reportable and cannot reconnect or run work; its operator clears its
  machine's logins. When removing an account with several program sources,
  its one row per worker keeps the most severe result, independent of arrival
  order: not in contact, then failed, then lost. Lost means the admitted
  operation ended without a worker terminal.
  The issued-name record keeps desired sign-out, independently of observed
  login state. A reconnect or engine restart checks all issued names, including
  retired names; a retained login requiring cleanup is signed out before that
  worker becomes ready for its holder's runs. Cleanup has its own result and
  never changes or retries the original answer. A failed cleanup keeps the host
  restriction until absence is established; a holder's explicit check of a
  live declaration permits a fresh sign-in.
- **Concurrency is the host's slots:** a run on a program is a smith
  run, and counts as one run, as any other does.
- **A host also reads an account's list** of models with the account's grant
  (`accounts-lists.md`, 1), which holds a value, never a program's source.

## 2. States

A source's state is one of:

- **usable:** a signed-in source while the accounts child holds a
  granted generation still valid; a key while it is kept; a program's on
  each host whose half reports its login usable; a *none* source
  always; each unless one of the states below holds;
- **limited:** the LLM provider says it is spent, with when it resets
  where the provider says, and never a guess (providers.md, 2.5). No new
  run starts on it until then, or, with no time given, until its holder
  resumes it, replaces its LLM key or signs in again. An absent reset remains
  absent across new grants and restarts; it is never converted to zero or
  a guessed retry deadline. An explicit zero is a supplied reset. Text a person reads
  says "limited";
- **refused:** the LLM provider refuses the account for what is asked:
  not eligible, a scope not granted, plan usage declined at sign-in, a
  key it does not take. It stays so until its holder acts: signs in
  again, with the scope, replaces the key, or chooses another source;
- **needs sign-in:** no record, or one revoked or past refreshing; for
  a program, on which host;
- **misconfigured:** the LLM provider rejected the application's client;
- **unloaded:** its record could not be read, saying where and why;
- **moving:** a signed-in source whose record is out with a move, or
  back from one and not yet refreshed (`accounts.md`, section 6). No run takes it, and
  nothing of the record's generation is lent.

Each source's mechanism maps its LLM provider's failures onto these,
skein for what a provider's API answers (skein's `llm/vocabulary.md`, section 1.5)
and smith for what a program reports (smith's `protocol/programs.md`,
section 5). A failure a run meets is the run's: unavailable is retried
by smith's session; unsupported, a field the LLM provider will not
take, is the request's error, never sent again unchanged (providers.md,
2.5).

How the accounts child keeps them, from what the accounts' component
says (`../protocol/llm.md`, section 5) and what runs' agents notice
(hosts/README.md, section 2):

- **Every signed-in source is wanted,** once the restart has restored
  the core's records, its accounts' among them (engine/restart.md,
  item 1), to the close, so the component refreshes each before its
  access token lapses whether or not a run holds it, and answers each
  new generation, granted; an idle account keeps its
  refresh token (providers.md, 2.3; skein's `oauth/accounts.md`, section 3). A
  key is granted once it is kept, a replaced key as its next generation.
- **Every live attempt hears each new generation** of its source, on
  whatever host, the engine's own included; and the fleet sends each
  attempt it keeps its current grant when its worker says hello, and
  when it adopts the attempt (engine/fleet.md).
- **Unauthorized:** a run's LLM provider refused a generation. For a
  signed-in source the child passes it to the component, which refreshes
  once; a second unauthorized of the same generation fails it, and the
  source needs sign-in. A key refused so is refused. For a program the
  host's half checks the login again, and the source needs sign-in on
  that host when it fails.
- **Limited:** a run's agent says its source is spent (skein's
  `exhausted`, the program's usage limit, or a rate limit whose wait
  outlasts its session: smith's `domain/session.md`, section 4), with
  how long until it resets where it gives that; the core makes it a time on its clock as
  it hears the notice. The child marks the source limited until then,
  or until its holder resumes it, and asks nothing of the component.
- **Refused:** a run's agent says the LLM provider refused the account
  for what it asked, or a sign-in's answer does (`accounts.md`, section 5). The child
  marks the source refused, and asks nothing of the component.
- **Failed,** in one of five classes the component gives:
  - **later,** with how long: not a state. The source keeps its state,
    the child asks again after it, while the source is still wanted, and
    a run that needs a valid generation waits for the next one, never
    passing to the next source;
  - **sign in again:** its grant was revoked, or it has no record. The
    source needs sign-in, the deployment's document says its holder must
    sign it in, and a new sign-in (`accounts.md`, section 5) makes it usable again;
  - **misconfigured:** the LLM provider rejected the application's
    client. The source is misconfigured until a restart with its
    configuration fixed, and the document says so;
  - **unloaded:** its record could not be read. The source is unloaded,
    the document says where and why, and the child asks again at its
    retry interval;
  - **cancelled,** at the close.
- **Close** releases every source, and ends once the component has
  closed.
- **Deadlines belong to their source.** At creation of a limit or a
  retry, the accounts child retains `Deadline { at: Time, wall: Wall }`
  from that step's injected `env.now` and `env.wall`. Its `Limited` and
  `Later` state and spent-until facts expose that pair to the core.
  Timers use `at`; every task wait, account and deployment document,
  and stored reset uses the retained `wall`, exactly. Later clock
  samples and wall corrections do not change an existing deadline.
- **Limit merging keeps the same precedence.** A later reset replaces
  both `at` and `wall`; an earlier reset keeps the old pair, and a tie
  keeps the old pair too. A limit with no reset overrides a timed
  limit to `None`; an indefinite limit stays indefinite until resumed.
  No wall from a losing reset replaces the winner's wall.
- **What the store keeps** of each source is its kind, its name, its
  hold (limited, refused or moving), and a limit's reset as a wall
  time (engine/decisions.md, 1.4): a restart preserves that saved wall exactly,
  maps it once to a fresh monotonic
  `at`, lifts a limit whose reset passed while the engine was down,
  and holds a later one until it. The optional wall reset
  represents both timed and indefinite limits. A source held moving is stored so, and a restart brings it
  back as a cancelled move does (`accounts.md`, section 6). A program's login on a
  worker is not stored: its
  half reports it as `logins`, which follow the worker's hello
  (`../protocol/channel/README.md`, 4.3).
