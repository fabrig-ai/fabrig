# documentation: status

2026-11-08. Where temper's code lags or departs from [documentation.md](documentation.md).

Section 5 has no known gap in an asking charter reading the same pages
through `see help`, with `read_docs` absent from that charter's offer.
The generated `see` reference page remains section 7's deferred work.

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine.md`,
  7.3; this is a design gap, not an absent implementation.

- **Not yet:** 9: every area's errors brought to the rule, each with its remedy: the forge's connect refusals that ctl renders still say a bare `busy` or `no key` (`temper-client-domain/src/forge/forges.rs`, `connect_refusal`).
