# D035: World tests speak in bare numbers, and each file names its own fixtures

- **Kind:** tests
- **Where:** `jig/tests` and `jig/testing`, in the remaining rows below
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Readability of the worlds' tests")
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A1 F10: `ask` defined 7 times; A12 F15: stories in bare numbers)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 19: the core world's "verbs in the kit" never planned)
- **Checked:** 2026-11-05
- **Value:** medium (a reader sees the story a test tells)
- **Size:** M
- **Status:** open
- **Paid when:** each world names its parties, projects, tasks and connectors once, in the world or its kit, and its common acts are helpers named for the design's verbs; a bare number is left only where the number is the point.
- **Measure:** `rg -o -t rust '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' jig/tests jig/testing | rg -v ':(millis|between|randomness|check_redial|assert_replays|fixture|workers|workers_at|procedure_watch|procedure_world|reserve_fixture|world_with_rows)\(' | wc -l` (37; target at most ten, each explained). The unfiltered search reads 106 across both directories, or 80 in `jig/tests` alone (262 at the plan's base; 137 on 10-18).

## What

- People: six calls in `jig/tests/people/tests/suite/{start_restore,simulation}.rs` and `jig/tests/people/src/{world,fixtures}.rs`: question identities, deferred hold completion and the sign-in helpers' fixture values.
- Connector settings: six `proposal_record` calls in `jig/testing/jig-test-connector/src/tests/settings.rs`, naming proposal 12 and its setting numbers.
- Tasks: one `finish` call in `jig/tests/tasks/tests/suite/close_project.rs`, naming recurring task 10 and attempt 7.
- Accounts: eight calls in `jig/tests/accounts/tests/suite/{start_restore,model,sign_in}.rs`, naming accounts, providers and sources.
- Client activity: five `turn_line` calls in `jig/testing/jig-client-test-records/src/activity_reads.rs`, naming activity identities and timestamps.
- Client documents: four `panel` calls in `jig/testing/jig-client-test-domain/src/family/wire.rs`, checking identity equality and capacity bounds.
- Scripted engine: four sign-in map inserts in `jig/testing/jig-client-scripted-engine/src/{tests.rs,codec/tests.rs}`; client settings: one in `jig/tests/client/tests/suite/settings.rs`.
- Service: two `System::new` calls in `jig/testing/jig-test-service/src/tests/clients.rs`, giving resource and keyed-effect capacities.

The twelve excluded functions take spans, bounds or seeds, where the number is the point.
The remaining rows need their owners' fixture sweep or an explanation of why their exact
numbers matter before the target is met. Common core request verbs and the notes world's
project fixtures are shared; a request key stays literal so a retry keeps its identity.

## Why it matters

A reader should see which fixture a call uses without tracing its numeric identities
through setup and assertions.

## What pays it back

Finish naming the remaining fixtures in their worlds or kits, and explain each number
whose exact value is the point. Group tests that differ only in data into table tests.
