# Claude through an API key

Draft, 2026-11-11, from a session with the user that day: "add support
for using claude via API token, that would be via smith agent natively
(no need to call out to native claude binary) ... Initially it's
sufficient to expose this via "temper ctl" (basically you can add an
account in temper that uses a Claude API token)." Today temper offers
Claude through Claude Code alone, on a person's subscription
(`claude-provider.md`). This draft adds an Anthropic API key as a second
way to the same provider, run by smith's own loop, in two steps: the
first in v0.1.0, the second weighed once the first has landed (the
user, 2026-11-11). temper/migration carries it out. The reasons are
`notes.md`, section 14.

## 1. In one page

- **One Claude, two ways:** the `anthropic` provider offers Claude
  Code's program, for a person's subscription, and API keys. A run on
  a program source goes through Claude Code; a run on a key goes
  through smith's direct backend, which speaks Anthropic's Messages API
  itself, with no `claude` on the machine (section 3).
- **The source picks the route:** each Claude model is reached either
  way, and the source chosen for a run decides the endpoint its charter
  names (section 4). jig's accounts already order an Anthropic
  account's sources, Claude Code then a key; the code keeps them apart.
- **Keys alone, directly:** the direct backend takes a Claude Console
  API key, never a Claude.ai login or a token made from one; a
  subscription stays Claude Code's (section 2).
- **Added through `temper ctl`,** `account add-key`, as an OpenAI key
  is; the pages offer it where they offer OpenAI's, in words of its own
  (section 6).
- **Priced as billed:** a key pays per token, so Claude's models gain
  Anthropic's cache prices (section 5).
- **Most of it is there:** smith's direct backend already speaks
  Anthropic's dialect, a key already reaches it as a header, `ctl`
  already adds keys, and smith already carries a conversation from one
  backend to the other. What is missing is the provider's offer, the
  route by source and the figures.
- **Two steps** (section 8): step 1 is sections 2 to 7, in v0.1.0;
  step 2, a limited subscription's work moving to a paid key, a check of
  each source, and how one Claude with two ways looks, is weighed for
  v0.1.0 once step 1 has landed.

## 2. The terms

- **Anthropic's terms are jig's** `domain/providers.md`, 3.1: products
  use API keys through Claude Console; a key is its owner's, for its
  owner's authorized users, billed to its owner, not resold or
  intermediated; a subscription's credentials are never collected,
  stored or routed through a product. Read again on 2026-11-11 (section
  10), they say the same, with Ultra among the plans.
- **So a key is its owner's,** billed to its organization under
  Anthropic's Commercial Terms: a person's own, or a deployment's added
  by its owner. temper pays for and resells nothing.
- **A subscription reaches Claude through Claude Code alone,** as its
  publisher ships it (smith's `protocol/programs.md`, section 6).
- **The direct backend can't take one:** it sends its credential as
  `x-api-key` with no `Authorization` field (skein's `llm/README.md`,
  4.7), which a Claude.ai token does not authenticate, and no page or
  command offers to paste one. A key's words say where a key comes from
  (section 6).

## 3. The provider

What is there:

- **smith's two backends:** "A credential runs the direct backend; a
  program source runs the program its endpoint names" (smith's
  `protocol/programs.md`, sections 1 and 3). The direct backend's
  Anthropic dialect is skein's LLM client's: streaming, tool calls,
  adaptive thinking with an effort, cache marks and usage (skein's
  `llm/README.md`, 4.7).
- **A key's way through:** jig keeps a key in its secrets' store and
  grants it as a key (`../protocol/credentials.md`, section 3).
- **`ctl` adds keys** to any provider that offers them
  (`../client/ctl/areas.md`, area 5).

What step 1 adds:

- **The offer:** `anthropic` offers Claude Code's program alone
  (`../protocol/setup/init.md`, section 3). It offers keys too, its
  sources `{"programs": ["claude_code"], "keys": true}`, everywhere init
  writes the provider (`claude-provider.md`, section 2).
- **A direct endpoint:** beside Claude Code's, which names the program,
  a second at the same address, `api.anthropic.com:443` over TLS, in
  Anthropic's dialect, naming no program, on the engine and in the
  agent's configuration alike, under the same name on both
  (`../protocol/llm.md`, section 3).
- **The provider names both:** its endpoint stays Claude Code's, the one
  its models take (jig's `protocol/configuration.md`, section 3), and it
  names the direct one for keys. A provider with one way, as OpenAI,
  names one. A stored model keeps its form; a run's copy of it takes
  the endpoint of the source chosen for it (section 4).
- **No Claude Code for a key:** a deployment whose Claude accounts are
  keys never fetches Claude Code, which only a command that runs it
  fetches (`claude-provider.md`, section 3).

## 4. The route by source

- **jig's order holds:** an account's sources are its holder's, in its
  holder's order, an Anthropic account's Claude Code then a key by
  default; a run takes the first usable on its host, and passes to the
  next only when the earlier is unusable there alone, such as a login
  not signed in on that host (jig's `domain/accounts.md`, 3.4). Today a
  model whose endpoint names a program takes program sources alone, and
  one naming none direct sources alone; a model of a provider with both
  endpoints takes either.
- **Account, source, host and endpoint are chosen together:** the
  endpoint a host is checked for is the chosen source's, never the
  model's stored one, at a run's start, in readiness and the offer, and
  when a claim is checked again (jig's `domain/accounts.md`, 4.3).
- **The charter names the chosen source's endpoint:** Claude Code's for
  a program source, the direct one for a key. smith then does what it
  already says: a key on the direct backend, a program source on the
  program (smith's `protocol/programs.md`, section 3).
- **One way per run:** a charter's models of one provider share its
  grant, so a run's alternatives take the same way as its main model.
- **A run keeps its source** (smith's `protocol/programs.md`, section 3),
  and **a task its account** (jig's `domain/accounts.md`, 4.3). A
  source limited everywhere holds its account's runs until its holder
  acts, since the order decides who pays (jig's `domain/accounts.md`,
  3.4): no key is spent because a subscription ran out. Whether it
  should be is step 2's.
- **A conversation crosses backends:** a task whose next run takes the
  other way carries its conversation over from smith's transcript
  (smith's `protocol/programs.md`, 5.8).
- **One row per model:** Claude's models are the same models whichever
  way reaches them, so the catalogue keeps one row each, under
  `anthropic`, and no name is doubled.
- **A key reads Anthropic's list** through the direct endpoint, never
  Claude Code's (`model-catalogue.md`, section 5), for an account whose
  first source is the key. One that lists Claude Code first reads none,
  as jig's rule has it (jig's `domain/accounts.md`, 3.7): the order
  decides who pays, and a subscription has no list to read.

## 5. The models' figures

- **Names and list prices stay** (`model-catalogue.md`, section 2):
  Opus 5.5, Sonnet 5.5 and Fable 5.1, at $4 and $20, $2 and $10, $10
  and $50 per million tokens in and out.
- **Cache prices become Anthropic's,** as read on 2026-11-11: a cache
  read $0.20 per million tokens for Opus 5.5 and Sonnet 5.5, $0.25 for
  Fable 5.1; a cache write 1.25 times the input price, for the
  five-minute cache smith's marks ask for (skein's `llm/README.md`,
  4.7). The stock prices cached input as input and a write not at all,
  its providers' pages giving no figure (`../protocol/configuration/stock.md`);
  Anthropic's does, and a key pays it.
- **The same figures for both ways:** a subscription's runs bill nobody,
  but their spend is counted at the same prices, so that a project's
  budget means the same whichever way a run took.
- **The output:** the stock's 8,192 bounds thinking and the answer
  together on the direct route, since `max_tokens` is the declared
  output (skein's `llm/README.md`, 4.7). Whether Claude's models declare
  more, up to Anthropic's 128,000, M1's design weighs against the
  agent's largest turn, which a larger output raises (`model-catalogue.md`,
  section 2); the walk after landing shows whether a delegated change at
  the stock's efforts is cut.

## 6. Adding a key

- **`temper ctl DIR account add-key PROVIDER FILE --key KEY [--account
  ACCOUNT] [--deployment]`** (`../client/ctl/areas.md`, area 5),
  PROVIDER `anthropic`'s number as `me` prints it, 2 as init writes it:
  a key-only account, a key after ACCOUNT's Claude Code login, or the
  deployment's key, which pays for every payer (jig's
  `domain/accounts.md`, 4.2). FILE is private, as any key's is.
  `replace-key`, `forget`, `rank` and `account list` work on it as on
  OpenAI's.
- **Its proof:** a key-only account of a person's is tried by the model
  check on the pages, one short turn at the key's price
  (`../domain/trial.md`, section 2). A key after a login, which the
  check passes over for the login, and the deployment's key, which the
  check doesn't try, are proven by the first run that takes them, as an
  OpenAI key added through `ctl` is; a refused key says so on its
  source (skein's `llm/README.md`, 4.7).
- **The pages draw it where they draw OpenAI's key**
  (`../client/pages/accounts.md`, section 2): with two providers offering
  keys, the ways read "Use an OpenAI API key" and "Use an Anthropic API
  key", rows under "Or an API key". A key's details read "An Anthropic
  API key", then "From the Claude Console, under API keys. What temper's
  agents use is billed to the key's organization." No new look.
- **The help** (`../../help/account.md`) says a Claude subscription
  connects through Claude Code, and a Claude Console key through
  `add-key`.

## 7. What current models refuse

Opus 5.5, Sonnet 5.5 and Fable 5.1 refuse requests that older Claude
models took. The direct backend's requests keep to them, each held by a
focused test of the dialect:

- **Thinking stays on:** adaptive thinking at the run's effort, never
  `disabled`, which these models refuse at every effort. An effort
  that turns thinking off is not offered for them.
- **No forced tool and no sampling settings:** `tool_choice` never
  `any` or a named tool, as skein's dialect has it (`llm/README.md`,
  4.7), and no `temperature`, `top_p` or `top_k`.
- **The effort is always sent:** Opus 5.5's own default is `medium`,
  below the stock's `high`.
- **History only grows:** for an Anthropic account made on or after
  2026-08-31, a request whose earlier turns were edited is refused
  while it replays their thinking. The direct backend replays thinking
  unchanged, appends only, and carries no thinking across a compaction
  of its own. The fake peer doesn't enforce this, so the walk after
  landing checks it on a new account's key.

## 8. Two steps

### 8.1 Step 1, in v0.1.0

Sections 2 to 7: a Claude key added through `ctl` and run directly,
beside Claude Code's subscription, in one provider. Its acceptance is
section 9.

### 8.2 Step 2, weighed after step 1

Each part is weighed for v0.1.0 once step 1 has landed (the user,
2026-11-11):

- **A limited subscription's work on a key:** today the order holds a
  limited source's runs until its holder acts (section 4). Whether a
  holder may let them pass to the key after it, who allows the spend,
  and how the pages and `account list` say a run took the key.
- **Trying each source:** a check of a key after a login, and of the
  deployment's key, from `ctl` as well as the pages.
- **The usage link for each way:** `manage_usage` is the provider's,
  claude.ai's settings, right for a subscription; a key's usage is the
  Console's.
- **The look of one Claude with two ways,** for the user to take: the
  welcome's and Settings' Claude card with its key, where the welcome
  today shows no key row beside the plan cards.

## 9. Acceptance of step 1

Focused and fuzzy suites alone, each failing first:

- init writes `anthropic` with both ways and both endpoints, on the
  engine and in the agent's configuration, with `claude` on `PATH` and
  without it;
- `ctl account add-key` with `anthropic`'s number makes a key-only
  account, `--account` puts the key after a Claude Code login, and
  `--deployment` makes the deployment's;
- selection: a key-only account runs a Claude model on the direct
  endpoint, a login-only account on Claude Code's; an account with both
  takes the login, and the key on a host where the login isn't signed
  in; a login limited everywhere holds the account's runs and spends
  nothing on the key; a run's alternatives take its way; readiness, the
  offer and a claim's check agree;
- a run on a key reaches the fake Anthropic peer through the direct
  backend with no `claude` configured, a tool call and its result
  included, and a run on a login reaches the fake `claude`, in one
  world; a task whose next run takes the other way carries its
  conversation over;
- an account whose first source is a key reads Anthropic's list through
  the direct endpoint, and one with Claude Code first reads none;
- section 7's requests, each refused shape absent;
- the catalogue's Claude rows carry section 5's prices, and a run's
  spend counts its cache reads and writes.

After landing, a walk on a real key whose account was made after
2026-08-31: a chat turn and a delegated change on Opus 5.5 through
`ctl`, on a deployment with no Claude Code.

## 10. Sources

Read on 2026-11-11:

- Anthropic's legal and compliance page, authentication and credential
  use: https://code.claude.com/docs/en/legal-and-compliance
- Anthropic's Commercial Terms of Service, effective 2025-06-17:
  https://www.anthropic.com/legal/commercial-terms
- The Agent SDK's overview, on Claude.ai logins in third-party products:
  https://code.claude.com/docs/en/agent-sdk/overview
- Anthropic's pricing, prompt caching and models pages, for section 5's
  figures and section 7's rules.

## 11. Open questions

- **Which of step 2's parts are in v0.1.0:** the user's, once step 1 has
  landed.
