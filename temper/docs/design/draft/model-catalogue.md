# The model catalogue

Draft, 2026-11-06, from a session with the user that day, who left the
choices below to this draft and reviews them afterwards: "we should add to
the models available by default all popular models ... and/or also have a
way to fetch from the provider all currently available models (so if a new
model comes out we don't need a release/redeploy). That probably means that
the models list should live in the store and not in the config file. Also,
since it could be many models that providers offer, in our picker we should
offer the most popular (and default should be Claude opus 5.5 and sol 6.1).
We're also missing the ability to pick the effort level." And of the
configuring chat: "by default it should be opus 5.5 at high (if claude is
available), with fallback to Astra 6." The look request `model-picker`
draws the pages (`../client/journeys/model-picker/`). This draft carries
what the look cannot.

## 1. In one page

- **Seven models come with temper,** each with its efforts and a default
  effort, and the picker offers them (section 2).
- **New work takes Opus 5.5 at high, or Sol 6.1 at high without Claude;**
  the configuring chat falls back to Astra 6 instead (section 3).
- **The list lives in the store,** as settings, and leaves both
  configuration files (section 4).
- **temper reads each provider's list where it can,** so a new model shows
  up with no release. A Claude subscription has no list to read, so a person
  can also add a model by name (section 5).
- **It lands in three steps,** the first of them with no store change
  (section 7).

## 2. What comes with temper

As read from the providers' pages on 2026-11-06 (section 8's sources):

| Name | Shown | Provider | Default effort | Input, output, per million tokens |
|---|---|---|---|---|
| `claude-opus-5-5` | Opus 5.5 | Anthropic | high | $4, $20 |
| `claude-sonnet-5-5` | Sonnet 5.5 | Anthropic | high | $2, $10 |
| `claude-fable-5-1` | Fable 5.1 | Anthropic | high | $10, $50 |
| `gpt-6.1-sol` | GPT-6.1 Sol | OpenAI | high | $2, $10 |
| `gpt-6-astra` | GPT-6 Astra | OpenAI | high | $10, $50 |
| `gpt-6.1-terra` | GPT-6.1 Terra | OpenAI | medium | $1, $5 |
| `gpt-6-luna` | GPT-6 Luna | OpenAI | medium | $0.10, $0.50 |

- **Terra's name is `gpt-6.1-terra`.** The user said "terra 6"; OpenAI lists
  no `gpt-6-terra`.
- **Each takes the efforts `low`, `medium`, `high`, `xhigh` and `max`,** in
  that order. Terra and Luna also take OpenAI's `none`, left out so that
  the picker has one scale.
- **A model names its default effort apart from its list:** `"efforts"` in
  order from low to high, `"effort"` the default. Today the list's first is
  the default (jig's `domain/tasks.md`, section 3), which would draw the
  picker's scale out of order.
- **The window and output** stay the stock's (`../protocol/configuration.md`,
  13.4), whatever a provider allows. Declaring efforts changes what the
  start checks as the agent's largest turn (`../protocol/configuration.md`,
  13.8), so its measure is taken again.
- **The playground's scripted model** stays with its own provider. It is
  offered only while the playground runs, and never to connect.

## 3. Defaults

A task given no LLM choice takes the first of its charter's models whose
provider has an account usable now and serving its payer (jig's
`domain/tasks.md`, section 3), at that model's default effort.

| Charter | First choices, in order | Then |
|---|---|---|
| the configuring agent's | Opus 5.5, Astra 6, Sol 6.1 | the rest |
| every other stock charter | Opus 5.5, Sol 6.1 | the rest |

- **With both accounts,** Opus 5.5 comes first, as the user named it first.
- **A list narrows this:** an account whose provider list lacks a model is
  passed over for that model (section 5), so a plan without Astra falls on
  to Sol.
- **The words before sending are Create's answer:** the picker's closed line
  is `resolved_default` (jig's `client/domain/contract.md`, 5.1), which `me`
  must supply. Today the view guesses, and it said Opus on a new chat and
  `gpt-6.1-sol` on the configuring chat of one box.
- **Picking stays a person's:** a choice in the picker names the model and
  effort, and the charter's order no longer applies to that task.

## 4. In the store

- **A new kind of setting,** `model` (jig's `domain/engine.md`, 17.1), numbered
  and never renumbered: its name, shown name, provider, endpoint, window,
  output, efforts and default effort, prices, description, whether the picker
  offers it, and where it came from: temper, a provider's list or a person.
- **The defaults are the binary's,** as a charter's are. A release that adds
  to section 2's table adds to the defaults at its start, decided by the
  release, and a model a person turned off stays off (jig's
  `domain/engine.md`, 17.7).
- **What leaves the files:** `engine.llm.models` and each provider's `models`
  from `temper.json`; each endpoint's `models` from `agent.json`. Providers,
  endpoints and sources stay (`../protocol/configuration.md`, section 2).
- **smith takes a model from the charter.** The charter's LLM already
  gives its window, output and effort; the endpoint gives the rest as one
  default for its models, `reasoning_item`, `head` and `idle`. The agent
  checks the charter against its limits in place of a list (smith's
  `protocol/agent.md`, section 4; `protocol/charter.md`, section 2).
- **A charter names its first choices and "the rest":** every other model of
  the catalogue, in its order, so that a new model needs no charter's change
  (jig's `protocol/configuration.md`, section 3).
- **The bound** is the charter's, 64 models (`MODELS` in
  `jig/crates/jig-domain-tasks/src/llm.rs`). A list that would pass it keeps
  the provider's first, and Settings says how many it left out.
- **A change** is as any setting's: a proposal, which the configuring chat can
  make, or an owner's act on Settings, Models. A task keeps the model it has.

## 5. Reading a provider's list

| The account's source | Reads | Gives |
|---|---|---|
| a ChatGPT plan's sign-in | `GET /v1/models` with its token, keeping `visibility` `list` | `slug` and `display_name`, for that account (jig's `domain/providers.md`, 2.4) |
| an OpenAI key | `GET /v1/models` with the key | names alone |
| an Anthropic key | `GET /v1/models` with the key | `id`, `display_name`, `max_input_tokens`, `max_tokens` and each effort under `capabilities` |
| a Claude subscription | nothing: Anthropic allows no other use of its credentials (jig's `domain/providers.md`, 3.1), and Claude Code has no list to read | section 2's table, and names added by a person |

- **Who reads:** a worker, as it checks a program's source (jig's
  `domain/accounts.md`, 3.4). The core sends it the account's grant, and
  smith's agent makes the one request on the endpoint's address and
  transport. It is not a run, and spends no budget.
- **When:** as an account becomes usable, once a day, and on "Read again".
  A read that fails keeps what the store has and says so on Settings, Models.
- **What it keeps:** each provider's configuration says which names are
  models for this work, as `keep` and `drop` patterns; OpenAI's keep `gpt-`
  and drop the audio, image, realtime, search and dated names.
- **What changes:** a name new to the catalogue comes in as a `model`, off the
  picker, tagged "new" for a week on Settings and "All models". A name gone
  from every list stops being offered, and is kept for the tasks that hold it.
  A read that changes nothing makes no revision; one that does is decided by
  the provider's list, a new decider beside the release, and moves no
  running work.
- **Per account:** the store keeps which names each account's last read
  gave. The picker and the default pass over a model for an account whose
  list lacks it. An account with no list takes all its provider's.
- **A listed model is a catalogue entry; only a completed request proves it
  usable** (jig's `domain/providers.md`, 2.4). `model-check.md` is that proof
  on the welcome. Anywhere else, a run the provider refuses says so, as today.
- **What a list doesn't say:** a new model with no window, output or prices
  takes its provider's defaults: its dearest stock model's prices, so that a
  budget stops early rather than late; the stock's window and output; no
  efforts, which leaves the provider's default and hides the picker's row.
  Settings marks its prices as estimated, and the configuring chat can set
  them.

## 6. Adding by name

"Add a model by name" on Settings, Models takes a provider and a name, and
makes a `model` as section 5's new names are, on the picker. It is the way to
a new Claude model on a subscription before a release, and the rest of what a
provider doesn't list. temper checks the name's shape alone: the first run
shows whether the provider serves it.

## 7. In steps

1. **Stock and the picker, no store change:** section 2's table in `temper
   init`'s two documents and its efforts; section 3's orders in `stock.rs`;
   `resolved_default` in `me`; the look request. This answers most of what
   the user asked for, and the help's "Offering another model" still holds.
2. **The store:** section 4, section 6 and the page on Settings.
3. **Lists:** section 5.

## 8. Open questions and sources

- **Haiku 5.5** is not in the user's list, so it is not in section 2. It is
  cheap, and may suit the status charter.
- **Plans that lack a model:** a Claude subscription may not serve Fable 5.1,
  and it has no list to say so. The first run shows it.
- **A role's models on a page:** Settings, Models lists the catalogue. Which
  charter runs on what stays with the configuring chat.

Sources, read 2026-11-06:
<https://developers.openai.com/api/docs/models>,
<https://developers.openai.com/siwc/token-sharing-open-source/models-and-inference>,
and Anthropic's Models API, `GET /v1/models`. Providers change these: read
them again before relying on them.
