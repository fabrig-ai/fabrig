# Settings fields

Kind: reference

Every member of a settings value: its type, its bound and what it does to work, as a settings file sets it and `setting --json` prints it.

## file

- `items`: list of at most 1,024, each `item`.

## item

- `setting`: `setting`.
- `set`: the setting's kind's value: `charter`, `template`, `policy` or `rules`.
- `edit`: `"removed"` or `"reset"`.

## setting

- `kind`: `"charter"`, `"template"`, `"policy"` or `"rules"`.
- `number`: integer, 0 to 4,294,967,295.

## charter

- `number`: integer, 0 to 4,294,967,295.
- `instructions`: text, at most 1,048,576 bytes.
- `waiting`: nanoseconds.
- `resume`: true or false.
- `turns`: integer, 0 to 4,294,967,295.
- `time`: nanoseconds.
- `reserve_turns`: integer, 0 to 4,294,967,295.
- `reserve_time`: nanoseconds.
- `model`: `model`.
- `alternatives`: list of at most 16, each `model`.
- `inspect`: true or false.
- `modify`: true or false.
- `shell`: true or false.
- `agents`: true or false.
- `call_timeout`: nanoseconds.
- `hosts`: `"workers"`, `"engine"` or `"both"`.
- `spend`: integer.

## model

- `dialect`: integer, 0 to 4,294,967,295.
- `endpoint`: integer, 0 to 4,294,967,295.
- `program`: text, at most 32 bytes, or null.
- `name`: text, at most 4,096 bytes.
- `window`: integer, 0 to 4,294,967,295.
- `output`: integer, 0 to 4,294,967,295.
- `input_price`: integer.
- `cached_price`: integer.
- `cache_write_price`: integer, or null.
- `output_price`: integer.
- `price_unit`: integer, 0 to 4,294,967,295.
- `efforts`: list of at most 16, each text, at most 32 bytes.
- `effort`: text, at most 32 bytes, or null.

## template

- `number`: integer, 0 to 4,294,967,295.
- `name`: `text`.
- `executor`: `executor`.
- `contract`: `contract`.
- `shallower`: integer, 0 to 4,294,967,295.
- `budget`: `budget`.
- `deadline`: `deadline`.
- `tracked`: integer, 0 to 4,294,967,295, or null.
- `flags`: `flags`.
- `label`: integer, 0 to 4,294,967,295.
- `grace`: nanoseconds, or null.
- `llm`: `llm`, or null.

## text

- `bytes`: text, at most 32,768 bytes.
- `cut`: true or false.

## executor

- `"chosen"`:
- `charter`: integer, 0 to 4,294,967,295.

## contract

- `approval`: `approval`.
- `report`: `report`.
- `verdict`: `verdict`.
- `change`: `change`.

## approval

- `state`: hex, at most 32,768 bytes.
- `part`: `part`.

## part

- `connector`: integer, 0 to 65,535.
- `kind`: `"resource"`, `"pool"`, `"result"`, `"procedure"`, `"effect"`, `"outbox"` or `"hold"`.
- `key`: `key`.

## key

- `path`: list of at most 9, each hex, at most 32,768 bytes.
- `number`: integer.

## report

- `words`: integer, 0 to 4,294,967,295.

## verdict

- `choices`: list of at most 1,048,576, each `choice`.
- `cut`: true or false.

## choice

- `code`: integer, 0 to 4,294,967,295.
- `words`: integer, 0 to 4,294,967,295.
- `followups`: integer, 0 to 4,294,967,295.
- `fields`: list of at most 32, each `choice_field`.

## choice_field

- `name`: hex, at most 32,768 bytes.
- `max`: integer, 0 to 4,294,967,295.

## change

- `connector`: integer, 0 to 65,535.
- `kind`: integer, 0 to 65,535.
- `words`: integer, 0 to 4,294,967,295.

## budget

- `"given"`:
- `amount`: integer.
- `as_allowed`: `as_allowed`.

## as_allowed

- `cap`: integer, or null.

## deadline

- `"none"`:
- `"given"`:
- `span`: nanoseconds.

## flags

- `conversation`: true or false.
- `listed`: true or false.
- `retention`: `"kept"` or `"forgotten_once_read"`.
- `ended_by`: `"result"` or `"requester"`.

## llm

- `account`: integer, 0 to 4,294,967,295, or null.
- `main`: `model_effort`.
- `subagents`: list of at most 64, each `model_effort`.

## model_effort

- `model`: text, at most 4,096 bytes.
- `effort`: text, at most 32 bytes, or null.

## policy

- `escalation_role`: integer, 0 to 4,294,967,295, or null.
- `ceiling`: `authority`.
- `projections`: list of at most 32, each `grant`.
- `period_spend`: integer.
- `roles`: list of at most 32, each `role`.
- `requirements`: list of at most 32, each `requirement`.
- `permissions`: list of at most 32, each `permission`.
- `templates`: list of at most 16, each `template_override`.

## role

- `number`: integer, 0 to 4,294,967,295.
- `authority`: `authority`.
- `period_spend`: integer.
- `requests`: integer, 0 to 65,535.
- `decides`: integer, 0 to 255.

## template_override

- `template`: integer, 0 to 4,294,967,295.
- `executor`: `executor`, or null.
- `budget`: `budget`, or null.
- `deadline`: `deadline`, or null.
- `priority`: integer, 0 to 4,294,967,295, or null.

## permission

- `connector`: integer, 0 to 65,535.
- `permission`: integer, 0 to 65,535.
- `role`: integer, 0 to 4,294,967,295.

## rules

- `ceiling`: `authority`.
- `period_spend`: integer.
- `minimum_run_spend`: integer.
- `maximum_run_spend`: integer.
- `implies`: list of at most 32, each `implication`.
- `requirements`: list of at most 32, each `requirement`.

## implication

- `connector`: integer, 0 to 65,535.
- `kind`: integer, 0 to 65,535.
- `implies`: integer, 0 to 65,535.

## authority

- `models`: `models`, or null.
- `tools`: integer.
- `grants`: list of at most 1,048,576, each `grant`.
- `delegation`: `delegation`.
- `spend`: integer.
- `deadline`: nanoseconds since 1970 UTC, or null.
- `notes`: integer, 0 to 255.
- `note_resources`: list of at most 1,048,576, each `resource`.

## models

- `"all"`:
- `named`: list of at most 64, each hex, at most 4,096 bytes.

## grant

- `connector`: integer, 0 to 65,535.
- `kind`: integer, 0 to 65,535.
- `pattern`: `pattern`.

## pattern

- `segments`: list of at most 9, each hex, at most 32,768 bytes.
- `last`: `last`.

## last

- `exact`: hex, at most 32,768 bytes.
- `open`: hex, at most 32,768 bytes.

## delegation

- `kinds`: list of at most 12, each `delegate`.
- `tasks`: integer, 0 to 4,294,967,295.
- `depth`: integer, 0 to 4,294,967,295.

## delegate

- `charter`: integer, 0 to 4,294,967,295.
- `procedure`: integer, 0 to 4,294,967,295.
- `role`: integer, 0 to 4,294,967,295.

## resource

- `connector`: integer, 0 to 65,535.
- `pattern`: `pattern`.

## requirement

- `connector`: integer, 0 to 65,535.
- `kind`: integer, 0 to 65,535.
- `pattern`: `pattern`.
- `judge`: `judge`.
- `guard`: `guard`.
- `must_be_guarded`: true or false.

## judge

- `connector`: integer, 0 to 65,535.
- `requirement`: integer, 0 to 65,535.
- `parameters`: integer, 0 to 4,294,967,295.

## guard

- `"guarded"`:
- `observed`: `observed`.

## observed

- `freshness`: nanoseconds.
