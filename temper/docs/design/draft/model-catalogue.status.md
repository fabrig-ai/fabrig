# model-catalogue: status

2026-11-06. Where the design lags or departs from [model-catalogue.md](model-catalogue.md).

- **Design lags:**
  - 3: `../client/README.md`, 3.5 gives an unusable account a fix only while
    its provider has no offered model, so it has no separate line beside a
    usable account of the same provider. Section 11 defers those separate
    account rows; grouping offered models by what pays and "All models" are
    in section 3.5.
  - 4: jig's `domain/engine.md`, 17.1 and 17.7: models are not a setting. jig's `protocol/configuration.md`, section 3: the `llm` member holds the models. smith's `protocol/agent.md`, section 4 and `protocol/charter.md`, section 2: a charter names only a declared model. `../protocol/configuration.md`, section 2: `engine.llm` holds the models. `temper/docs/help/account.md` and `temper/docs/help/deployment.md`: "Offering another model" edits both files.
  - 5: jig's `domain/accounts.md`, 3.4: a host checks a program's source alone. jig's `domain/engine.md`, 17.7: the deciders. smith's `protocol/agent.md`: no request lists an endpoint's models. `../client/README.md`, 3.5, *All models*: no "new" on a name just read, the look request `model-picker`'s page 04, since nothing reads a provider's list.
  - 4, 6: `../client/README.md`, 3.5: no page for models on Settings, the look request `model-picker`'s page 07, with its list, switches, "Read again" and "Add a model by name". *All models* links to Accounts, and says "Accounts, in Settings" where 04 says "Choose what the list shows, in Settings", until that page is there to choose on.
- **Design departs:**
  - 3: `../domain/agent.md`, 8.1 gives the inventory the offer alone, but
    serves a charter's full value when read, including unoffered models.
    Instructions restrict what the configuring agent may mention or propose;
    proposal validation still accepts configured models outside the offer.
    The `model-catalogue-1` cut chose this because proposals need whole
    charter values; no real-model proof of that instruction is supplied.
  - 3: `resolved_default` is each template's, not one a project's (jig's `client/domain/contract.md`, 5.1), since the configuring charter's order differs from a chat's, so that Configure temper and a new chat can each say their own: decided by the cut of `model-catalogue-1`, 2026-11-06.
- **Open:**
  - 8: Haiku 5.5; a plan that lacks a model; a role's models on a page.
