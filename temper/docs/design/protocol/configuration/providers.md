# LLM providers in the store

Provisional, 2026-11-17, from `../../draft/config.md`, 6.2
(`../../draft/config.execution.md`, N4). The LLM providers and their
endpoints leave the document for the store: the stock's are defaults of
jig's `provider` kind, one added, changed or removed is a setting, and
each applies at the engine's next start. The playground's scripted
provider is the box's constant, its scripts in the binary. The kind, its
check and its wire are jig's (jig's `domain/engine/llm-providers.md`);
this document is what temper adds: its stock, the copy a start reads,
the box's scripted provider, and what the document loses.

## 1. In one page

- **The stock's providers are defaults** (2): OpenAI, 1, and Anthropic,
  2, with their sign-in, keys, program, endpoints and lists, as
  `temper-box init` wrote them.
- **A start builds from the kept copy** (3). The engine keeps the
  current providers in its data directory, `providers.setting`, at each
  restore and each change, and each start reads it before it builds what
  dials, signs in or lists, the stock's when there is none. So a change
  applies at the next start, in both shapes.
- **The box's scripted provider is a constant** (4): provider 3, `demo`,
  its endpoint on a loopback port the box chooses at its start, its key
  made at the playground's start, its scripts in `temper-fake`. It is no
  setting, and every box has it.
- **The document loses `engine.llm` and `application.demo`** (5), each
  refused by name, and `temper-box init` writes neither.
- **People change providers** (6) with `temper ctl settings apply`, as a
  proposal Ask temper makes, or a release's defaults.

## 2. The stock's providers

- **Two providers, numbered for good:** OpenAI, 1, and Anthropic, 2,
  with today's values: the `engine.llm` that `temper-box init` writes
  without `--box` (`../setup/init.md`, 3), OpenAI with its sign-in, keys,
  list and endpoint `openai`, 1, and Anthropic with Claude Code's
  program, keys, list and its endpoints `anthropic`, 2, and
  `anthropic-api`, 4, its `direct`. Their keep and drop lists are
  `settings.md`, section 1's.
- **In the binary, in the setting's JSON form:**
  `temper-engine-service-stock`'s `stock/providers.json`, the list form
  of jig's `domain/engine/llm-providers.md`, section 6, read with
  `jig_protocol_config::llm::providers_setting` by
  `temper_engine_service_stock::providers`, which returns them as
  `Box<[ProviderSetting]>`. A focused test holds them to the values
  `temper-box init` wrote before this document, field by field.
- **Passed as `Defaults.providers`** in every shape, beside the stock's
  models, so the document's `llm` member is refused (5).
- **The stock's models stay as `stock.md` gives them:** the seven of
  OpenAI and Anthropic are every deployment's defaults, whatever its
  providers, and the scripted model, 8, is the box's, with its scripted
  provider (4). A change that removes a stock provider removes its models
  in the same change, or is refused, naming each model it would strand
  (jig's `domain/engine/llm-providers.md`, 4).

## 3. The kept copy

- **`providers.setting`,** in the data directory
  (`directories.md`), private, 0o600: the current providers, fixed ones
  never among them, in the list form of jig's
  `domain/engine/llm-providers.md`, section 6, written by
  `jig_protocol_config::llm::render_providers`. It is the store's value,
  kept where a start reads it before the store opens, as the box keeps
  `agents.setting` (`figures.md`, 4); nobody writes it by hand.
- **Written by the shell that serves the engine,** in both shapes, at
  each `Ask::Providers` (jig's `domain/engine/llm-providers.md`, 5): at
  the restore, and in each decision that changes a provider. The root
  hands the ask on as a delivery; the shell compares its bytes with the
  file's, and where they differ writes them whole to a private
  temporary file, syncs it, renames it into place and syncs the
  directory, as `agents.setting` is written. A file that can't be
  written is logged, `providers.setting: not written: WHY`, and the
  engine goes on: its next start runs the providers it last kept.
- **Read at each start, before anything is built from the providers:**
  `temper-service-shell`'s preparation, for the engine alone and the
  box, reads the file, and the start takes its providers as the running
  ones:
  - *no file:* the stock's providers (2);
  - *a file that reads:* its providers, the stock's none of them unless
    the file holds them;
  - *a file that doesn't read:* logged, `providers.setting: PROBLEM; the
    stock's providers run until the store's are kept again`, and the
    stock's run.

  The box adds its scripted provider (4). The running providers are the
  `Llm` the start builds from (`settings::Engine`'s `jig.llm`, its models
  the stock's), and every start check a document's `llm` member had runs
  on them, each naming `providers.setting` where the file gave it: the
  endpoints' trust files, the registrations' redirect ports among the
  listeners, the programs the agent offers, and at most 64 endpoints
  (`figures.md`, 2).
- **Once restored,** where the current providers aren't the running
  ones, the log says `LLM providers changed; they apply at the engine's
  next start`, once per start and once per change. The workers' figures
  carry the running endpoints (`figures.md`, 2), so a worker takes a
  change at its first start after the engine's.

## 4. The box's scripted provider

- **A constant, every box's:** temper's box passes it as jig's fixed
  provider (`Config.fixed_providers`, jig's
  `domain/engine/llm-providers.md`, 1): number 3, named `demo`, keys
  alone, no list, no plan, its endpoint `demo`, number 3, dialect 3,
  plaintext, no headers, connect and handshake 10,000 ms, at
  `127.0.0.1:PORT`. The stock's model 8, `demo`, is its model
  (`stock.md`). An engine alone has none. No change names it, and the
  kept copy never holds it.
- **Its port is chosen at the box's start,** before the box builds its
  worker, whose agents pin their endpoints at its start (`figures.md`,
  4): 8370 if it's free on loopback, else one the kernel gives. The box
  holds the listener and hands it to the fake LLM's step when the
  playground's group starts (`../setup/box.md`, 1), so nothing takes the
  port between. A forge's place avoids it, as it avoids the box's other
  ports (`../setup/place.md`).
- **Its key is made at the playground's start:** `demo/llm.key` in the
  data directory, 32 bytes from the kernel's random source as 64
  lowercase hexadecimal digits and a newline, 0o600, `demo/` made
  private, written before the playground's group starts and replacing
  any, since the box's start resets a playground it finds
  (`../setup/playground.md`). The fake LLM reads it at its start, and the
  playground's start adds it as the scripted account, as both do today.
- **Its scripts are in the binary:** `temper-fake`'s
  `demo/scripts.json`, the `asked`, `cued` and `otherwise` that
  `temper-box init --box` wrote under `application.demo.llm`, unchanged,
  read and checked by `temper_fake::demo` as the document's were
  (`../setup/playground.md`), into the scripts the fake LLM plays and the
  cues the client suggests.
- **The playground is the box's:** the box's typed configuration holds
  its `Demo` at every start, its provider 3, its key file
  `demo/llm.key`, its cues the scripts'; an engine alone holds none, and
  a test world that runs a playground builds its own in code
  (`moved.md`, 2). Whether the playground is offered stays the welcome's,
  kind 20 (`settings-kinds.md`, 1).
- **`temper-box fake llm DIRS`** takes its listener from the box, its key
  from `demo/llm.key` and its scripts from the binary, and reads no
  document.

## 5. What the document loses

| Member | Refused with |
|---|---|
| `engine.llm` | `engine.llm: moved: LLM providers are settings; see them with temper ctl settings show provider` |
| `application.demo` | `application.demo: moved: the playground's scripted model is the box's own` |

- **Refused by name** through `moved.md`'s table, whose rows these
  join; jig refuses `llm` as `What::Stored` once temper passes the
  providers' defaults, and temper's words for its path are the table's.
- **`temper-box init`** writes neither member, no `demo/llm.key` and no
  scripted model's port (`../setup/init.md`, 2 and 3); `--box` still
  makes `demo/` and `demo/bin/git`.

## 6. Changing a provider

- **Named `provider`, with its number,** in a settings file and a run's
  `settings`, `see` and `propose_settings`, its value jig's JSON form
  (jig's `domain/engine/llm-providers.md`, 6):

  ```json
  {"items": [{"setting": {"kind": "provider", "number": 3},
              "set": {"number": 3, "name": "local", "endpoint": "local",
                      "signed_in_plan": false, "sources": {},
                      "endpoints": [{"name": "local", "number": 5, "dialect": 3,
                                     "address": {"socket": "127.0.0.1:8080"},
                                     "transport": "plaintext", "headers": [],
                                     "connect_ms": 10000, "handshake_ms": 10000}]}}]}
  ```

  A value's `number` is the setting's, refused otherwise. `remove` and
  `reset` take the setting alone, as any kind's do.
- **`temper ctl settings show provider`** prints each provider setting
  in that form, withdrawn ones marked, a default marked as the stock's;
  `settings show provider N` one.
- **Decided by the deployment's owners,** as jig's kinds are. Ask temper
  reads them on its reference page `settings-fields` and proposes one as
  it does any kind (`../../draft/ask-temper.md`, section 5); its field's
  line says "taken at the engine's next start".

## 7. Upgrades

- **No store patch.** The kind starts empty, over its defaults
  (`../../draft/config.md`, section 8).
- **dev.fabrig.ai's steps** (`../../draft/config.execution.md`, N4):
  before the redeploy, `engine.llm` leaves the engine's document. After
  it, each of dev's providers that differs from the stock's is set with
  `temper ctl settings apply`, and the engine restarted, then its
  workers, so that each takes the endpoints at its next start.
- **A box made before** has `engine.llm` and `application.demo` in its
  document, both refused: its file is written again by hand
  (`../../draft/config.md`, section 8), and its `demo/llm.key` is
  replaced at the next playground start.

## 8. Testing

Focused and fuzzy alone; the process tier's stories are written and
compiled, and run after landing.

- **The stock:** `providers()` gives OpenAI and Anthropic with today's
  values; passed as defaults, a document's `engine.llm` is refused with
  5's words; `application.demo` likewise.
- **The kept copy:** a start with no file runs the stock's; with a file,
  its providers; with one that doesn't read, the stock's and the log's
  line; an `Ask::Providers` writes the file, and one with the same bytes
  doesn't; a start after a change runs the changed providers.
- **The box:** its running providers hold the scripted provider at the
  port its start chose; the fake LLM serves the binary's scripts with the
  key the playground's start made; `temper-box init --box` writes no
  `engine.llm`, `application.demo` or `demo/llm.key`.
- **The kind in temper:** a provider applied, shown and removed through
  `temper ctl settings`, and a removal with an account refused
  *accounts*.

## 9. Open questions

- **A provider's trust file** is a path in the configuration directory,
  as a document's was, checked at each start. Default: kept so until the
  TOML file's milestone gives machine paths a home of their own
  (`../../draft/config.md`, section 4).
