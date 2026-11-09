# authority: status

2026-11-08. Where jig's code lags or departs from [authority.md](authority.md).

Section 3 has no known gap in the asking family, bit 2,048, or
`ToolKind::Ask` admission. A call without that family is beyond authority.

Section 3's model bits retain the catalogue setting numbers. Section 8.3's
host evidence uses endpoint window and output bounds, without a model list.

Section 6.1: a person's task in project 0 draws on their pool, has no
tools or delegation, is watched by its payer and is absent from the
person's lists. Its restoration leaves cancellation to the application.
`PersonSeed::models()` exposes its allowed models, and `Event::PersonTask`
passes its optional `llm` into the task's ordinary admission and run
checks. No choice keeps the default selection; a choice can hold it to
one account.

## Gaps and departures

- **Not yet:** `ui-ask-m1`'s `proposal-read` part was never designed,
  according to the retirement brief. Its landed run proposal read needs
  design coverage reconciliation against `jig/docs/design/domain/engine.md`,
  7.3; this is a design gap, not an absent implementation.

- **Not yet:** 7: a widening funded by whoever gives it
  (the task's existing funder is charged).

- **Not yet:** 3, *Notes*: the asking family reaches no party's scope.
