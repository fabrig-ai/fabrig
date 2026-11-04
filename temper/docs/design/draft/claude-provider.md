# Claude, always offered

Draft, 2026-11-04, from a session with the user that day. The user's
own `temper box` offered no Claude on the welcome's Model step: `temper
init` writes Claude Code's program path and directory into
`agent.json` when it finds `claude` on `PATH`, but ships no Anthropic
provider, so no document offers Claude's way. Only the fake backends'
rewrite adds one, for the fake box (`../protocol/configuration.md`,
12.1). Wanted: every deployment offers a Claude subscription through
Claude Code beside the ChatGPT plan, whether or not `claude` is on
`PATH`; temper fetches Claude Code when it is not (the user,
2026-11-04).

## 1. In one page

- **Both, always:** init writes an Anthropic provider beside OpenAI and
  demo, its one source Claude Code's program, in `temper.json`, and its
  program's endpoint and model in `agent.json`, on every machine
  (section 2). The Model step offers ChatGPT and Claude, always.
- **Claude Code found, or fetched:** the `claude` on `PATH` when init
  finds one; otherwise temper's pinned build, fetched into the cache
  the first time it is needed, as Forgejo's binaries are (section 3).
- **Claude Code keeps apart from the person's own:** it runs with a
  configuration directory of its own under the data directory, never
  the person's (smith's `protocol/programs.md`, 4.2), whichever
  `claude` it is. The welcome's Claude card says so (a look request of
  2026-11-04).
- **Opus by default;** choosing the model and its effort is separate
  work, as it is for ChatGPT (section 4).
- **No compatibility:** until v0.1.0 there is no supported deployment;
  a box made before this draft is made again to offer Claude (the user,
  2026-11-04).

## 2. What init writes

- **The provider,** in `temper.json`, as the fake backends' rewrite
  writes it (`../protocol/configuration.md`, 12.1): `anthropic`, the
  number init leaves free for it between OpenAI's and demo's,
  `signed_in_plan` false, `manage_usage` at Claude's usage settings,
  its models, and Claude Code's program as its one source.
- **The program,** in `agent.json`: its path, the `claude` init found
  on `PATH`, or `"fetched"` for temper's pinned build in the cache,
  as an owned Forgejo's binaries are named (`../protocol/setup.md`,
  6.6); its programs directory, as today; an Anthropic-dialect endpoint
  for it, and its model with a window, an output and prices, as the
  shipped OpenAI model has them.
- **`documents.lines` at least 3,** so that the Model step's offer holds
  OpenAI, demo and Anthropic, as the rewrite raises it.
- **The fake backends' rewrite** then finds the provider written and
  swaps in its own adapter and model, rather than adding the provider.
- **`temper init --box` and init alone** write it alike: the Accounts
  page offers Claude wherever the deployment has it
  (`serve-without-box.md`, 2).

## 3. Fetching Claude Code

- **When it is needed:** the first command of the program's source
  that runs it (its sign-in from the welcome's Claude way, its status,
  a run) finds the fetched build missing and fetches it first; a
  machine that never chooses Claude never downloads it.
- **Pinned, like Forgejo:** a version, and a SHA-256 per platform,
  pinned in temper and checked byte for byte, streamed into a private
  part file and renamed, within the fetch's bounds, at most once at a
  time; into the cache, kept across boxes, never in the forge's or the
  programs' directories (`../protocol/setup.md`, 6.6).
- **Its own updates off:** the fetched build is temper's pin, so Claude
  Code's own updater does not replace it; a new pin is temper's
  release.
- **What the person sees:** the Claude way's first step says it is
  fetching Claude Code, then goes on as it does today.

## 4. The model

- **Opus,** the model the welcome's mockup names, through Claude
  Code's `--model` (smith's `protocol/programs.md`, 5.1).
- **Choosing a model and effort** is the separate work the user asked
  for (2026-11-04), for ChatGPT and Claude alike; this draft fixes only
  the default.

## 5. Its tests

- **Init's:** both documents hold the provider, the program, the
  endpoint, the model and the lines' capacity, with `claude` on `PATH`
  (its path) and without it (`"fetched"`).
- **The fetch's,** at its host: a missing build fetched once, then
  found; a wrong digest refused and nothing left; two commands at once
  fetch it once.
- **The rewrite's:** a written provider swapped, not doubled.
- **The client's worlds:** the Model step draws the ChatGPT and Claude
  cards from init's own documents, not the rewrite's alone.

## 6. Later

- **The person's `claude` installed after init:** taken in place of the
  fetched build, or not; today init's choice holds until the box is
  made again.

## 7. Open questions

- **The model's name and figures:** Claude Code's alias `opus`, which
  follows Anthropic's newest Opus, or a dated model id pinned with its
  window, output and prices; and where those figures come from for a
  subscription, whose calls carry no price.
- **Where the pinned build comes from:** the address of Claude Code's
  native builds and its manifest of digests, which the pin follows.
