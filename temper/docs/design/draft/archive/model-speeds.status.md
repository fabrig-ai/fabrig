# Model speeds: status

2026-11-27. Where the design lags or departs from [model-speeds.md](model-speeds.md).

- **Design departs:**
  - 5: a model of OpenAI's dialect that declares speeds, run at the standard speed, gives its charter's LLM the speed `default`, at the model's own prices, where the draft has a standard run's LLM carry nothing (jig's `domain/engine/executors.md`, 1). The agent has nothing else to tell it that the model declares speeds, and sending `default` is what the draft asks for it. So such a run waits for a worker on a build that knows speeds, as a run at a faster speed does; a charter whose models declare no speeds still goes to an older worker as today. The goal's designer, 2026-11-27.
