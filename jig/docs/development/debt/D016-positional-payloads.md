# D016: Event payloads travel as positional arguments

- **Kind:** complexity
- **Where:** `jig-domain-core`, `jig-domain-tasks`, `jig-domain-fleet`, `jig-domain-people`, `jig-domain-views`, `examples/ops`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Signals in the step code": lint exceptions, long functions)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (T7 "event payloads spread as positional arguments", A1 F6, A3 F6, A5 F18)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 9: 24 core `too_many_arguments`; T7: event payloads as named structs beyond core)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium (shorter signatures; a field added in one place)
- **Size:** M
- **Status:** open
- **Paid when:** no production function in jig needs a `too_many_arguments` exception, except a seam a comment names as kept whole on purpose.
- **Measure:** `rg -o 'too_many_arguments' jig/crates jig/examples -g '!**/tests/**' -g '!**/tests.rs' -g '!**/generated/**' | wc -l` (68 at base)

## What

68 `too_many_arguments` exceptions remain in production code: 30 in `jig-domain-core`, 14
in `jig-domain-tasks`, 5 in fleet, 4 in people, 3 in views and 3 in `ops` (98 at the
review). Beside them, 97 `too_many_lines` exceptions (same command with `too_many_lines`).
Event variants carry their fields loose, and each route unpacks them into a helper's
arguments with the request, person, project and role alongside.

## Why it matters

A field added to an event is threaded through every helper's signature by hand.

## What pays it back

Variants carrying named structs (`Event::Turn(Turn)`), and an owned route context
(`Party { request, person, project, role }`) passed once. With D015 most
`too_many_lines` cases shrink too.
