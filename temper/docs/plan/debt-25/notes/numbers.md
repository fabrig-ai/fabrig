# debt-25: naming a world's numbers (jig D035)

Every D035 task names this note, after `common.md`.

## The rule

The item's **Paid when**: each world names its parties, projects, tasks and connectors once, in the world or its kit, and its common acts are helpers named for the design's verbs; a bare number is left only where the number is the point.

- **A name says what the thing is in the story** (`OWNER`, `LIFTED_HOLD`). `const SEVEN` or a helper hiding the same literal pays nothing, even when the Measure drops.
- **Sweep, don't patch:** the Measure misses `account(5, 1, ..)` and `signin(11, OWNER)`. Replace a number throughout the file or world that uses it, not only on the matched line. The sweep is what pays the item.
- **Same value, same thing, one name.** The same number meaning two things keeps two names. A world's different values under one name (`PERSON` is 9, 1 and 7 across `jig/tests/tasks`) stay different, under names of their own, and no value changes. If unifying two files onto one number would change an order or an outcome, keep their values under distinct names.
- **Expected values don't change.** An assertion's expression may change by name only (`21` to `BASE + 1`, where that states the rule).
- **A helper moves to the kit only if a second file reads it,** and a copy goes only if the bodies are the same. Different verbs under one name (core's `ask`s, people's) aren't copies.
- **Fewer test lines.** Each task's report gives its net test lines. A task that ends higher says why.

## The Measure

```
rg -o -t rust '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' jig/tests jig/testing | rg -v ':(millis|between|randomness|check_redial|assert_replays|fixture|workers|workers_at|reserve_fixture|world_with_rows)\(' | wc -l
```

This is the item's command without `procedure_watch` and `procedure_world`, which match nothing. It reads 37 at this plan's base, and it should end at 2: the `turn_line` calls inside `first_turn` and `second_turn`, the one place that fixture is named, whose values `jig-client-protocol-link/src/tests/fixtures/turns.rs` pins.

Each task's own count of it is in its spec.
