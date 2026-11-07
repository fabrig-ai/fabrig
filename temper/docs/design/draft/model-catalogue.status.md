# model-catalogue: status

2026-11-07. Where the design lags or departs from [model-catalogue.md](model-catalogue.md).

Sections 4 and 6's store step is carried by jig's model settings and
catalogue, smith's endpoint limits and charter model names, temper's binary
stock, the configuring tools, Settings, Models and the help. The Models
page account is `415e659c54`; its render comparison remains temper/ui's
after-landing work (`../client/README.md`, section 11). Section 5's lists
remain the next step, so this draft stays.

- **Design lags:**
  - 3: `../client/README.md`, 3.5: an unusable account has a fix only
    while its provider has no offered model, so it has no separate line beside
    a usable account of the same provider. Section 11 defers those separate
    account rows. Settings, Models is
    `../client/README.md`, 3.5, *The `Models` tab*.
- **Design departs:**
  - 5, *Who reads*: smith's `protocol/agent.md`, 6.1: the agent makes the one request in a process of its own, `--list`, where smith's host half runs it as it runs a program's status, and no channel carries it, since a run's channel, domain and lifecycle hold nothing a read needs: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 3, against 5's *A read that fails keeps what the store has*: jig's `domain/accounts.md`, 3.7: an account whose read fails after a good one keeps that good read's names, where 3 has its read failing take the stock, so that a provider's bad hour moves no picker. An account with no good read takes the stock, as 3 says: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 3: jig's `domain/accounts.md`, 3.7: a person's added name is offered whatever a list says, where 3's "its last read holds the name" would hide it, since 6 is the way to what a provider doesn't list: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 5, *The table*: jig's `domain/accounts.md`, 3.7: an account's first source says whether it has a list, so an Anthropic account with Claude Code first and a key after reads nothing, since the order decides who pays and a key's list doesn't say what that subscription serves: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 5: jig's `domain/engine.md`, 17.12: a list's window and output come no larger than the stock's, and a name's figures are its first read's, since 2 has them stay the stock's whatever a provider allows and a read must not undo what the configuring chat set: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 4: jig's `domain/engine.md`, 17.11: a person's removal of a list's model makes it a person's, so that a list naming it again doesn't bring it back: decided by the cut of `model-catalogue-3`, 2026-11-07.
  - 3: `../domain/agent.md`, 8.1 gives the inventory the offer alone, but
    serves a charter's full value when read, including unoffered models.
    Instructions restrict what the configuring agent may mention or propose;
    proposal validation still accepts configured models outside the offer.
    The `model-catalogue-1` cut chose this because proposals need whole
    charter values; no real-model proof of that instruction is supplied.
  - 3: `resolved_default` is each template's, not one a project's (jig's `client/domain/contract.md`, 5.1), since the configuring charter's order differs from a chat's, so that Configure temper and a new chat can each say their own: decided by the cut of `model-catalogue-1`, 2026-11-06.
  - 4: a model's removal withdraws it and keeps its setting, as an exception to jig's `domain/engine.md`, 17.1's general removal rule, specified in its 17.11, so that a number is never given to another name and a task keeps its model: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 6: "Add a model by name" and the picker's switch are `ChangeModel`, a request of their own that the core makes into one change at the newest revision, where the draft has them as any setting's change, since a client holds no model whole and a switch can't go stale: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 4: a provider names an `endpoint` for its added models and an endpoint declares its `limits`, which the draft leaves implied: decided by the cut of `model-catalogue-2`, 2026-11-06.
  - 3: this draft and `../client/README.md`, 3.5 don't say what a started chat's change across LLM providers, or to a window too small for its conversation, does. What smith and jig already do stands, as the plain thing: decided by the cut of `ui-model-next-turn`, 2026-11-06, which changes none of it. Across LLM providers or accounts the run's check chooses an account again (jig's `domain/tasks.md`, section 7), and the window is carried over, compacted on the new model without the provider's opaque replay (smith's `domain/session.md`, 8.2). On the same account, a smaller window is reached through a compaction on the old model. A conversation too large for the new model fails as a compaction's *context too long* does (smith's `protocol/transcript.md`, section 11): the chat is held, "the run no longer fits the model's window of N tokens" (jig's `client/domain/contract.md`, 5.5), its transcript kept and nothing cut, and its person can choose another model, since a held task can still be amended (jig's `domain/tasks.md`, section 7).
- **Open:**
  - 4: a deployment with no stock provider, such as a local model's alone, has no default model, and its start is refused at `engine.llm.providers`; how its first model comes in is undecided.
  - 8: Haiku 5.5; a plan that lacks a model; a role's models on a page.
