# model-catalogue-1: the plan

From `inputs/request.md`: section 7's step 1 of `temper/docs/design/draft/model-catalogue.md`. The trunk is
`model-catalogue-1/integration`. This file grows with each push.

## Parts and tasks so far

| Task | Tree | Needs | Effort, minutes | What it lands |
|---|---|---|---|---|
| `effort-xhigh` | skein | | medium, 15 | Anthropic's dialect takes `xhigh`: it refused it, and section 2 declares it on every model |
| `effort-config` | jig | | high, 50 | `engine.llm.models[].effort`; a task given no choice takes it |

## Critical path, as it stands

`effort-config` then the stock's seven models, then the offer. The rest is being cut.
