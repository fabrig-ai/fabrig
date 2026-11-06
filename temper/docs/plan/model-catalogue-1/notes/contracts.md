# model-catalogue-1: the shapes that cross tasks

The run's binding calls. Each side builds to these, and neither needs the other's task. Design:
`jig/docs/design/client/domain/contract.md`, 5.1; `jig/docs/design/domain/engine.md`, 11.1.

## A configured model

`engine.llm.models[]` gains, beside `efforts` (low to high) and `description`, all optional:

- `effort`: the default, one of `efforts`. Left out, the first (`effort-config`).
- `shown`: its name for showing, at most 256 bytes, nonempty.
- `picker`: bool, true when left out.

`jig_domain_core::Model` carries `effort` alone: it is stored inside a charter's value, so no new field goes
on it. `shown` and `picker` go where `description` goes (`jig_protocol_config::llm`'s `ParsedModel`, `Llm.shown` and
`Llm.pickers` one per model as `Llm.descriptions` is), and on to `jig_protocol_engine_clients::Model`, with `effort`
as `Model::default_effort` gives it. No store reads them. smith's `agent.json` takes neither `effort` nor `shown`
(`deny_unknown_fields`), only `efforts`.

## `me`

In `jig-client-domain`'s `contract/llm.rs` and `contract/me.rs`:

```rust
pub struct MeModel {
    pub name: Text,
    pub provider: Option<u32>,
    pub window: u32,
    pub output: u32,
    /// The efforts it declares, low to high.
    pub efforts: List<Text>,
    /// The one of `efforts` a task given none takes; none only where it declares none.
    pub effort: Option<Text>,
    pub prices: ModelPrices,
    pub description: Option<Text>,
    /// Its name for showing, as configured; a client shows `name` without one.
    pub shown: Option<Text>,
    /// The accounts that can run it for the party, the one a run would take first first.
    pub accounts: List<u32>,
    /// Whether a picker shows it; all of the offer is shown under "All models".
    pub picker: bool,
}
pub struct MeProject {
    // ...
    /// The offer: what the party can pick here now, in configuration's order.
    pub models: List<super::MeModel>,
}
/// A template's preview, beside the templates: a template's own record is a setting's too.
pub struct MeDefault {
    pub template: u32,
    /// What a run's check of a task given none would take now; the template's `llm` where it names one.
    pub choice: super::LlmChoice,
}
```

and `MeProject` gains `pub defaults: List<MeDefault>`; `jig_client::shapes::resolved_default(me, project, template)
-> Option<&LlmChoice>` reads it. Shapes copy no preview into a request.

The wire, changed in place at v19 (formats change in place before v0.1.0):

```
record MeModel { name: text 4096 provider: option u32 window: u32 output: u32 efforts: list 16 text 32 effort: option text 32 prices: MePrices description: option text 512 shown: option text 256 accounts: list 256 u32 picker: bool }
record MeDefault { template: u32 choice: LlmChoice }
record MeProject { ... models: list 64 MeModel defaults: list 64 MeDefault public: option u64 }
```

`clients.rs`'s `check_settings` holds `accounts.accounts` to `me_model_accounts` and `core.templates` to
`me_project_defaults`, beside its other rows.

- A model with no account is not in `models`; nothing says why. `me.providers` still names every provider.
- A preview is absent for a chosen executor and for no usable model. `account` in it is `None`.
- `effort` on the wire is none only where `efforts` is empty, and otherwise one of them; a client refuses other bytes
  as malformed, as it does a zero price.

## The core

In `jig-domain-core`:

```rust
impl Core {
    /// The offer for `payer` in `project` (domain/engine.md, 11.1): configuration's order.
    pub fn offer(&self, payer: tasks::Payer, project: u32, models: tasks::Models) -> List<Offered>;
    /// What a run's check of a task given none would take now, or none (accounts.md, 4.3).
    pub(crate) fn preview_llm(&self, charter: u32, revision: u32, models: tasks::Models,
                              payer: tasks::Payer, project: u32) -> Option<tasks::LlmChoice>;
}
/// One offered model: its index in `settings.models`, and the accounts that can run it.
pub struct Offered { pub model: u8, pub accounts: List<u32> }
```

- An account can run a model when `accounts.select` chooses it (`Selection::Chosen`) with `Some(account)`, coverage
  `account_coverage(payer, Some(project))`, in `candidates` order: for a model with no `program`, on no host facts,
  so a worker's restart takes no model away, and `account list --until usable` means offered; for one with a
  `program`, on the facts of a connected, undrained worker that has answered its logins and declares the model, as
  `model_readiness` walks them. `model_readiness` itself is unchanged: a run's check still reads it.
- The preview walks `default_llm_at`'s candidates and takes the first with an account, `default_effort` on each.
- The payer of `me`'s offer and preview is `tasks::Payer::Party(party)`, the same as the view's `payer(me, None)`.
- `FrameProjectRole` gains `defaults: List<(u32, tasks::LlmChoice)>` beside its templates, and `offer: List<Offered>`, taken within its role's `models` mask, which stays for what else
  reads it (`tasks::Models::from_parts(mask.bits(), mask.is_all())`, as `documents/task_tree.rs` converts one).
- A task's document: `llm` is `next_llm`, but `None` for a task whose `llm` is `None` when no candidate is kept,
  usable or serving (`jig/docs/design/domain/accounts.md`, 4.3, step 4). `next_llm` itself still falls to the
  charter's first, since `chosen_policy` needs one. The wait stays `Account { None, None, Empty }`.

## The client

- `temper-client-domain`'s `choice::models` goes; `choice::offer(me, project) -> Option<&List<c::MeModel>>` stands
  in its place. Draft's `Act::Model` sets `effort` to the row's `effort`. `Draft::valid` is true while `requested`
  equals `original`: what a template or a task holds stays its own, offered or not.
- `Context::Template` gains `template: u32`, whose `resolved_default` the line reads.
- A model a task or a draft holds that `models` doesn't list reads "MODEL · needs an account" (`W::ModelNeedsAccount`,
  which stays). `W::NeedsAccount`, beside a provider's models, goes.
- A wait `Account { why: Empty }` with `task.llm` none: "No account can run this chat yet.", `W::WaitEmpty`'s
  words now; with `task.llm` some, `W::WaitEmptyModel` as it stands.
- `temper ctl`: `--model X` with no `--effort`, and `--sub-agent X` with none, take the row's `effort`. A name the
  offer lacks prints `model not offered: X`.

## The stock

In `temper-service`'s `stock.rs`:

```rust
const FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6.1-sol"];
const CONFIGURING_FIRST_CHOICES: &[&[u8]] = &[b"claude-opus-5-5", b"gpt-6-astra", b"gpt-6.1-sol"];
```

A charter's models are those configured, in that order, then the rest in the document's. C is the largest
`jig_domain_core::most_costly` over `llm.models`.

## The configuring agent

`temper-engine-domain`'s `route/host_route.rs` gains, beside `ProjectPerson`:

```rust
/// One model of the offer, for an authorized inventory (temper/docs/design/domain/agent.md, 8.1).
pub struct OfferedModel { pub name: Box<[u8]>, pub provider: Box<[u8]>, pub efforts: Box<[Box<[u8]>]>, pub effort: Option<Box<[u8]>> }
```

The inventory's text, in `temper-engine-smith`'s `repository_rows`, after `forge people:`:

```
models you can offer:
  claude-opus-5-5: anthropic; efforts low, medium, high, xhigh, max; default high
```

and, with an empty offer, the one line `models you can offer: none; no account can run a model yet`.
