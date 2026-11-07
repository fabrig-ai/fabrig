# model-catalogue: status

2026-11-07. Where the design lags or departs from [model-catalogue.md](model-catalogue.md).

- **Design lags:**
  - 3: `../client/README.md`, 3.5: an unusable account has a fix only
    while its provider has no offered model, so it has no separate line beside
    a usable account of the same provider. Section 11 defers those separate
    account rows. Settings, Models is
    `../client/README.md`, 3.5, *The `Models` tab*.
  - 5, 7: Settings, Models draws no "Read again", "Couldn't read OpenAI's list · 2 hours ago", "new" tag, "Read from OpenAI 2 hours ago" or another clause on where a list came from beyond temper's own, and the store keeps no per-account record of names: step 3, `model-catalogue-3`, built on `ModelSource::List`, `withdrawn` and `new_model` (jig's `domain/engine.md`, 17.11).
  - 5: jig's `domain/accounts.md`, 3.4: a host checks a program's source alone. jig's `domain/engine.md`, 17.7: the deciders. smith's `protocol/agent.md`: no request lists an endpoint's models.
- **Design departs:**
  - 3: `../domain/agent.md`, 8.1 gives the inventory the offer alone, but
    serves a charter's full value when read, including unoffered models.
    Instructions restrict what the configuring agent may mention or propose;
    proposal validation still accepts configured models outside the offer.
    The `model-catalogue-1` cut chose this because proposals need whole
    charter values; no real-model proof of that instruction is supplied.
  - 3: `resolved_default` is each template's, not one a project's (jig's `client/domain/contract.md`, 5.1), since the configuring charter's order differs from a chat's, so that Configure temper and a new chat can each say their own: decided by the cut of `model-catalogue-1`, 2026-11-06.
  - 4: a model's removal withdraws it and keeps its setting, where jig's `domain/engine.md`, 17.1, erases a setting with no default, so that a number is never given to another name and a task keeps its model: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 6: "Add a model by name" and the picker's switch are `ChangeModel`, a request of their own that the core makes into one change at the newest revision, where the draft has them as any setting's change, since a client holds no model whole and a switch can't go stale: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 4: a provider names an `endpoint` for its added models and an endpoint declares its `limits`, which the draft leaves implied: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 3: this draft and `../client/README.md`, 3.5 don't say what a started chat's change across LLM providers, or to a window too small for its conversation, does. What smith and jig already do stands, as the plain thing: decided by the cut of `ui-model-next-turn`, 2026-11-06, which changes none of it. Across LLM providers or accounts the run's check chooses an account again (jig's `domain/tasks.md`, section 7), and the window is carried over, compacted on the new model without the provider's opaque replay (smith's `domain/session.md`, 8.2). On the same account, a smaller window is reached through a compaction on the old model. A conversation too large for the new model fails as a compaction's *context too long* does (smith's `protocol/transcript.md`, section 11): the chat is held, "the run no longer fits the model's window of N tokens" (jig's `client/domain/contract.md`, 5.5), its transcript kept and nothing cut, and its person can choose another model, since a held task can still be amended (jig's `domain/tasks.md`, section 7).
- **Open:**
  - 4: a deployment with no stock provider, such as a local model's alone, has no default model, and its start is refused at `engine.llm.providers`; how its first model comes in is undecided.
  - 8: Haiku 5.5; a plan that lacks a model; a role's models on a page.
