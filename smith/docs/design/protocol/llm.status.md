# llm: status

2026-11-15. Where smith's code lags or departs from [llm.md](llm.md).

- **Not yet:** 3: the direct backend sends `finish`'s schema as the bare `oneOf` of the contract's forms, with no `"type": "object"` and no `outcome` property, and passes a host tool's declared schema through with no `type` added and no top-level combinator refused; Anthropic refuses such a request. 4: a `finish` call's input is decoded as the form itself, not from its `outcome` member. [in flight: change 3946]
