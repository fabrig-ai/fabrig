# LLM providers

2026-11-17. Each LLM provider the engine offers is a setting of jig's
kind `provider` (`settings.md`, 1): the provider whole, with its
sources, its list's patterns and its endpoints. An owner adds, changes
or removes one as any setting, by one decision; it applies at the
engine's next start, since what dials and signs in through a provider
is built at start. This document is the kind, its check, how a start
takes it, and what crosses the store and the client's wire. It is
written from temper's `draft/config.md`, 6.2. What a provider offers its
accounts is `../accounts.md`, 3.4; a provider's list is `settings-catalogue.md`,
2; the `llm` member a document gives is
`../../protocol/configuration-engine.md`, section 1.

## 1. The kind

- **A provider is named by its number,** from 1 to `PROVIDERS`, 16, the
  kind's bound, a constant of the vocabulary as `MODELS` is. Its value is
  the provider whole:

  ```rust
  // jig_domain_core_vocabulary::settings, re-exported by jig_domain_core::settings;
  // jig_protocol_config re-exports Provider, Registration, Endpoint, Header, Transport
  pub struct ProviderSetting {
      pub provider: Provider,         // its number, name, sources, list, plan and usage terms (2)
      pub endpoints: Box<[Endpoint]>, // its endpoints, each naming this provider (2)
      pub withdrawn: bool,            // removed, kept for its name (3)
  }
  pub enum Value { /* ... */ Provider(Box<ProviderSetting>) }
  pub struct Defaults { /* ... */ pub providers: Box<[ProviderSetting]> }
  pub const PROVIDERS: u32 = 16;

  // jig_domain_people, re-exported by the vocabulary
  pub enum Setting { /* ... */ Provider(u32) }
  pub enum SettingFault { /* ... */ Accounts }
  ```

- **One record for a provider.** `Provider`, `Registration`,
  `Endpoint`, `Header` and `Transport` move from `jig-protocol-config`
  (`llm.rs` and `llm/providers.rs`) into the vocabulary, the lowest
  crate both the core and the parser name, with the fields they have
  today, and `jig-protocol-config` re-exports them, so `Llm` names the
  same records (`skein/docs/foundation/programming-model.md`, 4.6). An
  endpoint's address moves with it, as the `Host` and socket forms the
  document reads. A `Provider` keeps no model names: a model names its
  provider by number (`settings-catalogue.md`, 1). `jig_domain_accounts::Provider`
  stays the accounts child's projection, its offers and terms, built
  from a `ProviderSetting` as it is built from a document's provider
  today.
- **Its defaults are the application's,** passed in `Config`'s
  defaults (`Defaults.providers`): temper passes its stock's (temper's
  `protocol/configuration/providers.md`, 2). An application that passes
  no defaults has `jig-protocol-config` read them from the document's
  `llm` member, each provider with the endpoints that name it, as jig's
  reference binary does (6).
- **The application's fixed providers** are its constants, never a
  setting: `Config.fixed_providers: Box<[ProviderSetting]>`, none unless
  it passes some, as temper's box passes its scripted provider (temper's
  `protocol/configuration/providers.md`, 4). A fixed provider joins every
  check as a current one does (4), takes no change, and is in every
  start's running providers (5).
- **The current providers** are the defaults with the stored changes
  over them, as for every kind (`settings.md`, 1), changed by
  `ChangeSettings` or an accepted proposal of settings, running work
  staying. They are read at the newest by the check (4), by the keeping
  ask (5) and by a settings document; what dials, signs in and lists
  through a provider reads the running providers alone (5).

## 2. A provider's parts

- **The provider:** today's `Provider` less its models: its number and
  its name for showing, 1 to `jig_domain_accounts::PROVIDER_NAME` bytes;
  `endpoint`, the endpoint its added models take, and `direct`, the
  program-free endpoint its sign-ins and keys take, each one of its own
  endpoints or none; `list`, its `keep` and `drop` patterns; its
  `signed_in_plan` and `manage_usage` terms (`../providers.md`, 2.6);
  and its sources: `sign_in`, a public registration, `keys`, and
  `programs`, each name at most 32 bytes (`../accounts.md`, 3.4).
- **Its endpoints:** each as today's `Endpoint`: its name and number,
  unique across every provider's endpoints, fixed ones included; its
  dialect; its `provider`, this provider's name; its optional `program`,
  one of the provider's `programs`; its address, the one its agents
  dial from their worker's host; its transport, TLS with its server name
  and optional trust, or plaintext; its headers; and its connect and
  handshake bounds. At most `ENDPOINTS`, 64, across every provider, the
  workers' figures' bound (temper's `protocol/configuration/figures.md`,
  2).
- **Its lists' bounds,** each a *shape* refusal past it: an endpoint's
  headers at most 16, the worker wire's `AgentEndpoint.headers` bound,
  since a header past it never reaches a worker; a provider's `programs`
  and a registration's `scopes` and `must_grant` at most 64 each. The
  stored value's lists take these counts, so a proposal's encoded worst
  case stays within its `u32` envelope.
- **No secret.** A registration with a client secret is out of shape, as
  in a document, and so is a header named, in any case,
  `authorization`, `proxy-authorization`, `x-api-key`, `api-key` or
  `cookie`: a credential is an account's, kept in the secrets' store
  (`../accounts.md`, 3.1), never a setting a client reads.

## 3. Numbers and names

- **A number keeps its name** for the deployment's life, as a model's
  and a worker's do (`settings-catalogue.md`, 1; `workers.md`, 2): a set naming
  another name than its default's or its stored change's is out of
  shape, and so is a name another number holds, withdrawn ones and fixed
  ones included. Accounts and models name a provider by number, so no
  number goes to another provider.
- **A removed provider stays, withdrawn,** its number and name kept:
  out of the current providers, of the keeping ask and of a client's
  choices. A set of its name at its number brings it back. A *reset*
  returns a provider with a default to it; one with no default is out of
  shape. A set whose value carries `withdrawn` is out of shape.
- **A client numbers a new provider:** the lowest number no provider
  setting or fixed provider holds, and a withdrawn name its own number;
  an endpoint's number likewise, the lowest no endpoint holds. The core
  checks only what it is given.

## 4. Its check

The provider kind joins `settings.md`, 1's one check:

- ***shape:*** a provider out of 2's form or past its bounds; an
  `endpoint` or `direct` that isn't one of its endpoints, a `direct`
  naming a program, an endpoint's `provider` not its name, a `program`
  it doesn't offer, a `signed_in_plan` without a sign-in, a
  `manage_usage` that isn't a public HTTPS address, a pattern out of its
  form; a name or an endpoint's name or number held twice; 3's rules of
  numbers and names; a change naming a fixed provider's number. These
  are the checks `jig-protocol-config` makes of a document's `llm`
  member today, moved into the vocabulary (`settings/providers.rs`) so
  that the parser and the core run the same functions; what a document
  alone has, a trust file's existence and a redirect port's place among
  the listeners, stays the start's (`../../protocol/configuration.md`,
  section 7).
- ***count:*** a number past `PROVIDERS`, or endpoints past `ENDPOINTS`.
- ***reference:*** a model, not withdrawn, naming a provider that the
  current and fixed providers don't hold, or an endpoint none of them
  holds; a model naming an endpoint of another dialect or program stays
  *shape*. These are `settings/catalogue.rs`'s `shape` checks, now
  against the current and fixed providers rather than the running
  ones, so a removal names each model of that provider it would strand,
  and its proposal removes them in the same change.
- ***accounts:*** a removal of a provider the deployment holds an account
  of: its accounts are removed first, as a linked worker is stopped
  first (`workers.md`, 5). A decision's alone: at a start the accounts
  are restored, not judged.

A refusal names the setting, `Provider(number)`, and its `at` the item,
as every change's does (`settings.md`, 2); but a *reference* refusal
names the model that holds the reference, `Model(number)`, as a charter
removal that strands a template names `Template(number)`, and its `at`
is the last item naming that model, or 0 when none does (the dispatch's
`malformed_refusal`). The refusal's detail stays empty: no member of
`SettingDetail` is added for it.

## 5. At a start, and while it runs

- **The running providers** are those a start was built with: the
  application gives them to the core as today, in `Config.settings`'
  `llm_providers` and `endpoints`, the fixed ones among them. The
  accounts child, a provider's list, a run's ways and its charter's
  endpoints read them, and nothing changes them until the next start.
- **A change applies at the next start.** It is checked and stored, and
  the running providers stay. A model whose provider or endpoint the
  running providers don't hold is out of the offer until a start runs
  it, as a model no usable account serves is (`settings-catalogue.md`, 1): no
  lookup of a model's or an account's provider among the running ones
  assumes it is there.
- **Kept for the next start.** The core asks the root to keep the
  current providers, `Ask::Providers` with every current provider not
  withdrawn, fixed ones never among them: at the start's settings
  restore, before `Event::Needs` (`restart.md`), and in every decision
  or release's revision that changes a provider, part of that decision
  as `Ask::Workers` is (`workers.md`, 4). The application keeps them
  where its next start reads them before it builds what they configure
  (temper's `protocol/configuration/providers.md`, 3). An application
  that keeps none starts on its defaults each time.
- **A release's new default** comes in as for any kind (`settings-store.md`,
  1): a provider a change is over stays as the change left it, and a new
  default provider applies at the start after the one that brought it,
  once kept.

## 6. Store, wires and the document

- **The store:** jig's settings family gains the `provider` identity and
  value, added variants of a registered family's schema, which move no
  version (`AGENTS.md`, section 2); the families that name a setting (a
  refusal's facts, a proposal's heads) gain the identity each by its own
  rule.
- **The client's wire** moves its version, unbridged (temper's
  `draft/redeploys.md`, 3.1 and section 4), once, for
  `Setting::Provider`, `SettingValue::Provider` and
  `SettingFault::Accounts`. A settings document carries each provider
  setting, withdrawn ones included, as it carries any kind's.
- **The value's JSON form,** one provider whole, is
  `jig-protocol-config`'s: its reader, `llm::provider_setting`, and its
  writer, `llm::render_provider`, over `skein_json`. The form is one
  entry of the `llm` member's `providers`
  (`../../protocol/configuration-engine.md`, section 1), its `number` among its
  members, with an `endpoints` list beside them, each one of the `llm`
  member's `endpoints` less its `provider`:

  ```json
  {"number": 4, "name": "local", "endpoint": "local", "signed_in_plan": false,
   "sources": {},
   "endpoints": [{"name": "local", "number": 5, "dialect": 3,
                  "address": {"socket": "127.0.0.1:8080"}, "transport": "plaintext",
                  "headers": [], "connect_ms": 10000, "handshake_ms": 10000}]}
  ```

  The list form, `{"providers": [VALUE, ...]}`, is
  `llm::providers_setting` and `llm::render_providers`: what an
  application keeps for its next start, and what a settings file names
  item by item.
- **The document's `llm` member.** An application that passes the
  providers' defaults has the member refused, naming its path, with
  `What::Stored`, as `llm.models` is today; one that passes none reads
  it as today, its providers and endpoints the kind's defaults and the
  running providers alike.

## 7. Open questions

- **An account added while a removal waits.** Between a provider's
  removal and the next start, the running accounts child still offers
  it, so an account can be added that the next start strands. Default:
  the stranded account is out of every offer, and its holder removes
  it.
- **A provider waiting for its start, shown.** A settings document
  doesn't say which providers wait for the next start. Default: the
  application's field line says a change applies at the next start;
  limits' settings bring a marker of their own (temper's
  `draft/config.md`, 6.1).
