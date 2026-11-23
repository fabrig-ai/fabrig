# Model speeds: standard, fast and ultrafast

Draft, 2026-11-21, from the user's words that day: support setting fast or ultrafast mode for implementer runs on gpt-6.2-sol, and "can you make a draft for the 'proper support' of ultrafast as above? I accept your points there". OpenAI serves its newer models at faster speeds for a higher price: Fast and Ultrafast (section 2). This draft lets a deployment choose a speed wherever it chooses an effort. Each request asks for that speed, and each completion is charged at the speed that served it.

## 1. In one page

- **A model declares its speeds** (section 3), as it declares its efforts: each by the provider's name, with its own prices, never below the standard ones. A model may also name a default speed. One that declares none runs at its provider's standard speed only.
- **A choice selects one** (section 4), wherever a choice selects an effort: a task's main model and its sub-agents, a template's choice, a charter's named model, and temper's producers and reviewers. A choice names a speed, or `standard`, or nothing, which takes the charter's or the model's default. `temper ctl` takes `--speed` and `--sub-agent-speed MODEL:SPEED`.
- **A run's requests ask for it** (section 5). The charter carries the speed to the agent, and the OpenAI dialect sends it as `service_tier`.
- **Each completion is charged at the speed that served it** (section 6), as the response reports it. A budget and a run's check hold the chosen speed's prices, the dearest.
- **The stock adds gpt-6.2-sol** (section 7) with its speeds and their prices. No stock model or stock template defaults to a faster speed: a faster speed costs more, so it's a deployment's choice.
- **Upgrades** (section 8): what's stored reads as it runs today, and a run with a speed waits for an agent that knows speeds.
- **dev.fabrig.ai** (section 9) keeps Ultrafast across the redeploy that retires today's stopgap, by giving its own gpt-6.2-sol speeds and an Ultrafast default.

## 2. Why

- **Implementers wait on their model.** A goal's implementers run on gpt-6.2-sol at effort `high` on dev.fabrig.ai, since 11-20. A faster speed shortens every wave without changing the model or its effort.
- **OpenAI's terms,** read on 2026-11-21 from its fast-mode, ultrafast-mode and pricing pages:
  - A Responses request takes `service_tier`: `default` for the standard speed, `fast` (or `priority`, the same) for Fast, and `ultrafast` for Ultrafast. Leaving it out takes the project's own default, which may be Fast. The response's `service_tier` says which speed served.
  - Past a ramp limit, Fast may be served at the standard speed and charged as standard. The pages state no fallback for Ultrafast.
  - Ultrafast serves GPT-6.2 Astra and Sol, GPT-6 Astra and GPT-6.1 Sol, with rate limits of its own. Where it serves them differs by model: GPT-6.2 Sol's only in the US.
  - gpt-6.2-sol's prices per 1,000,000 tokens, at short context: $2 input, $0.10 cached input, $2.50 cache write and $10 output at the standard speed. Fast is twice those. Ultrafast is three times them until 2027-02-28, and six times from 2027-03-01.
- **A stopgap for dev.fabrig.ai,** ready since 11-21 for the user's yes ([dogfood-hacks.md](dogfood-hacks.md), 2.15): a patched agent asks every OpenAI request for Ultrafast, whatever was chosen, while temper charges each completion at the model's configured prices. It goes once the setting of section 9 takes over.
- **Nothing in temper knows a speed today.** The request writers send none, the response's tier isn't read, and a model has one set of prices.

## 3. A model's speeds

- **`speeds`,** a new member of the `model` setting (jig's `protocol/configuration.md`, section 3; jig's `domain/engine/settings.md`, section 1): at most four, each with:
  - its name, as its provider takes it (`fast`, `ultrafast`), sent as it is;
  - its prices, the same four as the model's (input, cached input, output, and cache writes where priced apart), in the model's price unit, each at least the model's own, since a faster speed never costs less;
  - `also`, optional: at most two other names the provider may report it served under, as `priority` for OpenAI's Fast.

  The model's own prices stay its standard speed's.
- **`speed`,** optional: the model's default speed, taken where neither a choice nor its charter names one. Left out, that's standard.
- **One namespace per model.** A speed's name and its `also` names are at most 32 bytes of lower-case letters, digits, `-` and `_`. None appears twice in the model, and none is `standard` or the dialect's own name for standard (`default` for OpenAI's).
- **Out of shape,** besides those rules: a default speed the model doesn't declare; a speed's price of zero, or below the model's own; any speed on a model whose dialect takes none, which today is Anthropic's, or whose endpoint names a program, which chooses its own (section 10).
- **An added model takes no speeds** (jig's `domain/engine/settings.md`, sections 11 and 12). Neither an owner's add by name nor a provider's list says them, so an owner gives them by a set.
- **A model's speeds are part of each resolution that includes it** (jig's `domain/engine/settings.md`, section 11). A change of a model's speeds or its default speed changes every charter resolution that includes the model, and so makes those charters' next versions, as a change of its efforts does. A charter that leaves the model out doesn't move.
- **They count in a charter's size,** with their names and prices, as its models' other members do (jig's `domain/engine/settings.md`, section 8).

## 4. Choosing a speed

- **Wherever an effort is chosen, a speed may be** (jig's `domain/tasks/README.md`, section 3): a task's main model, each of its sub-agents, its later choice's, a template's choice, and a charter's named model. temper's change producers and reviewers take one too: a change's own choice, kept today in its `model[.effort]` parameter, gains its speed, and the old spelling still reads.
- **A choice's speed** is a speed's name, or `standard`, or nothing:
  - a name or `standard` is taken as it is;
  - nothing takes the charter's speed for that model, else the model's default speed, else standard.
- **It's kept by its name,** at most 32 bytes, never by its position in the model's list, so that a reordered list moves no choice. It's checked against the task's charter version: at its batch, at an amendment, and at a move, where a task whose speed the newest version doesn't offer stays on its revision (jig's `domain/engine/settings.md`, section 4). A speed its model doesn't declare is refused, naming the model and its speeds.
- **As with an effort,** a task keeps the choice it was given until an amendment changes it, and a run reads its revision's models (jig's `domain/engine/settings.md`, section 3).
- **`temper ctl`** ([areas.md](../client/ctl/areas.md)): `goal start`, `chat start` and `task amend` take:
  - `--speed SPEED`, for the main model, needing `--model` as `--effort` does;
  - `--sub-agent-speed MODEL:SPEED`, repeatable, for a sub-agent its `--sub-agent` names, split at the last colon, since a speed's name holds none.

  `--sub-agent MODEL[:EFFORT]` keeps its meaning. `task show` prints a choice's speed beside its effort.
- **`me` and the client wire** carry each model's speeds and default speed, so that a picker can offer them later (section 10).
- **Help:** the help's model and goal pages say what a speed is, that it costs more, and how to choose one.

## 5. A run's requests

- **The charter carries it** (smith's `protocol/charter.md`, section 2). Where a run's speed for an LLM isn't standard, that LLM gains `speed`, its name, with its `also` names and its prices. A standard one carries nothing new, so its charter is today's.
- **The agent passes it** to each completion of that LLM, main or sub-agent, for the conversation's life, as it passes an effort.
- **The OpenAI dialect sends `service_tier`** (skein's `llm/wire.md`, 1.8):
  - the speed's name, for a run with a speed;
  - `default`, for a model that declares speeds run at standard, so that a project's own default can't make it faster;
  - nothing, for a model that declares none, as today.

  Both of skein's Responses writers send it: the live client's, from the prompt, and the owned request's.
- **Anthropic's dialect refuses a prompt with a speed** as unsupported (skein's `llm/wire.md`, 1.7). `model-speeds.md`, section 3's shape rules keep that from happening, as they keep a speed off a program's route.

## 6. Charging what served

- **The response says what served.** skein's Responses decoder keeps the response's `service_tier`, bounded, for a completed and for an incomplete response, and a completion carries it beside its usage (skein's `llm/vocabulary.md`, 1.3).
- **smith charges at that speed's prices** (smith's `domain/session.md`, section 6):
  - a reported name that is the run's speed's name, or one of its `also` names, takes its prices;
  - the dialect's name for standard takes the model's own prices;
  - any other name, or none, takes the run's speed's prices, so that a budget stops early rather than late.
- **A budget holds the chosen speed** (jig's `domain/authority.md`, 8.3). A run's check, its reservation and smith's reserve for each completion price it at the chosen speed's prices, which section 3 makes the dearest it may be charged.
- **A refusal is a completion's failure, as today.** When a provider refuses the speed (no access, or past its own rate limit), the completion fails with the provider's words, and the run's retries and holds follow. Nothing falls back to standard on its own.

## 7. The stock

- **gpt-6.2-sol joins the stock's models** at the next stock number ([stock.md](../protocol/configuration/stock.md)): shown "GPT-6.2 Sol", OpenAI's, efforts `low` to `max` with `high` its default, its window and output those of the stock's others. Its prices are per 1,000,000 tokens in thousandths of a dollar, as OpenAI's page gives them at short context:

  | Speed | Input | Cached | Cache write | Output |
  |---|---|---|---|---|
  | standard, the model's own | 2,000 | 100 | 2,500 | 10,000 |
  | `fast`, also `priority` | 4,000 | 200 | 5,000 | 20,000 |
  | `ultrafast` | 12,000 | 600 | 15,000 | 60,000 |

  Ultrafast is at six times standard, its list price from 2027-03-01: until then it overcounts by half, but a budget stops early rather than late, and nothing held needs repricing on that date. Which figure to keep is the user's (status).
- **No default speed,** on it or on any stock model, and no stock template or charter chooses one.
- **A deployment whose catalogue already holds the name keeps its own model.** Its templates and tasks name that one, and a catalogue refuses a name held twice. So the start leaves the new stock model out, before its checks, and keeps the deployment's number.
- **The stock's other models declare no speeds** until their prices are read. A deployment declares them by a set.

## 8. Upgrades

By the upgrades rule ([layouts-both-ways.md](layouts-both-ways.md)):

- **Stored settings and tasks:**
  - the model gains `speeds`, empty, and `speed`, none;
  - every model-and-effort choice in settings, tasks, people and temper's store, held settings included, gains `speed`, none;
  - a change's `model[.effort]` parameter reads as it is, with no speed.

  Each is an added field with a default, which the store derives as it reads an older layout. Everything stored reads as standard.
- **The agent:** a charter with no speed is today's, and reaches an older worker's agent as it does today. One with a speed can't be carried to an older layout without losing it, so it's refused there, and the run waits for a worker on the new build, as the upgrades rule has it for what no plan derives.
- **Clients:** the client wire's model and choices gain them. A client at another layout is refused by name, and reloads.

## 9. dev.fabrig.ai

- **Steps, ops', on the user's yes, once every worker's agent carries F1** (the execution file):
  - set dev's own gpt-6.2-sol, the model its templates name, to section 7's speeds and prices, with `ultrafast` its default speed, decided with running work moving, so that each live goal's next run takes it too (jig's `domain/engine/settings.md`, section 4);
  - check one implementer's completions are reported `ultrafast` and charged at its prices.
- **The stopgap goes then.** Until every worker runs an agent built without it and that setting is in place, each dev build of the agent carries it again.

## 10. Not covered

- **Pickers:** the web composer's and the terminal's controls for a speed. `me` carries what they need (section 4).
- **Anthropic's fast output,** until its Messages dialect takes a speed, and a program's speed, which the program chooses.
- **Long-context prices:** above 272,000 input tokens gpt-6.2-sol costs more. The stock prices it at short context, as it prices gpt-6.1-sol, so a prompt past that undercounts.
- **Choosing a speed by itself,** per wave or by load.
- **Showing what served:** a run's or a goal's share of completions at each speed.
