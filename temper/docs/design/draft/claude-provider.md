# Claude where Claude Code is

Draft, 2026-11-04, from a session with the user that day. The user's
own `temper box` offered no Claude on the welcome's Model step: `temper
init` writes Claude Code's program path and directory into
`agent.json` when it finds `claude` on `PATH`, but ships no Anthropic
provider, so no document offers Claude's way. Only the fake backends'
rewrite adds one, for the fake box (`../protocol/configuration.md`,
12.1). Wanted: wherever init finds `claude`, the deployment offers a
Claude subscription through Claude Code, as the fake box does.

## 1. In one page

- **Init writes Anthropic** when it finds `claude` on `PATH`: a
  provider beside OpenAI and demo, its one source Claude Code's program,
  in `temper.json`, and its program's endpoint and model in
  `agent.json` (section 2).
- **With no `claude`, nothing changes:** no Anthropic provider, and the
  Model step offers ChatGPT alone.
- **Claude Code keeps apart from the person's own:** it runs with a
  configuration directory of its own under the data directory, never
  the person's (smith's `protocol/programs.md`, 4.2). The welcome's
  Claude card says so (a look request of 2026-11-04).
- **Opus by default;** choosing the model and its effort is separate
  work, as it is for ChatGPT (section 3).
- **No compatibility:** until v0.1.0 there is no supported deployment;
  a box made before this draft is made again to offer Claude (the user,
  2026-11-04).

## 2. What init writes

- **The provider,** in `temper.json`, as the fake backends' rewrite
  writes it (`../protocol/configuration.md`, 12.1): `anthropic`, the
  number init leaves free for it between OpenAI's and demo's,
  `signed_in_plan` false, `manage_usage` at Claude's usage settings,
  its models, and Claude Code's program as its one source.
- **The program's endpoint and model,** in `agent.json`, beside the
  program's path and directory init writes today: an Anthropic-dialect
  endpoint for the program, and its model with a window, an output and
  prices, as the shipped OpenAI model has them.
- **`documents.lines` at least 3,** so that the Model step's offer holds
  OpenAI, demo and Anthropic, as the rewrite raises it.
- **The fake backends' rewrite** then finds the provider written and
  swaps in its own adapter and model, rather than adding the provider.
- **`temper init --box` and init alone** write it alike: the Accounts
  page offers Claude wherever the deployment has it
  (`serve-without-box.md`, 2).

## 3. The model

- **Opus,** the model the welcome's mockup names, through Claude
  Code's `--model` (smith's `protocol/programs.md`, 5.1).
- **Choosing a model and effort** is the separate work the user asked
  for (2026-11-04), for ChatGPT and Claude alike; this draft fixes only
  the default.

## 4. Its tests

- **Init's:** with `claude` on `PATH`, both documents hold the provider,
  the endpoint, the model and the lines' capacity; without it, neither
  does, as today.
- **The rewrite's:** a written provider swapped, not doubled.
- **The client's worlds:** the Model step draws the Claude card from
  init's own documents, not the rewrite's alone.

## 5. Later

- **Claude installed after init:** offering it then, without making
  the box again, from the Accounts page or a `temper ctl` change.

## 6. Open questions

- **The model's name and figures:** Claude Code's alias `opus`, which
  follows Anthropic's newest Opus, or a dated model id pinned with its
  window, output and prices; and where those figures come from for a
  subscription, whose calls carry no price.
