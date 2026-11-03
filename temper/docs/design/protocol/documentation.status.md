# documentation: status

2026-11-03. Where temper's code lags or departs from [documentation.md](documentation.md).

- **Not yet:** 8: the reference pages `limits` and `limits-engine` (`docs/help/reference/` has `commands`, the `ctl-GROUP` pages, `host-tools` and `settings-fields`). 8: the reference pages `configuration` and `configuration-engine`. 9: every area's errors brought to the rule, each with its remedy: the forge's connect refusals that ctl renders still say a bare `busy` or `no key` (`temper-client-domain/src/forge/forges.rs`, `connect_refusal`).
- **Departs:** 2.2, 3: `temper-docs` still declares `Kind::Guide` and its step tests exempt it, with no page of that kind, because the last guide was rewritten without removing the kind; a code task removes it.
