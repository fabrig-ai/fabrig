# D035: World tests speak in bare numbers, and each file names its own fixtures

- **Kind:** tests
- **Where:** the remaining fixture copies and literal identities below, in `jig/tests` and `jig/testing`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Readability of the worlds' tests")
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A1 F10: `ask` defined 7 times; A12 F15: stories in bare numbers)
- **Seen:** 2026-10-31 at `1b5c98834`, in the quality run's debt list (item 19: the core world's "verbs in the kit" never planned)
- **Checked:** 2026-11-06
- **Value:** medium (a reader sees the story a test tells)
- **Size:** M
- **Status:** open
- **Paid when:** each world names its parties, projects, tasks and connectors once, in the world or its kit, and its common acts are helpers named for the design's verbs; a bare number is left only where the number is the point.
- **Measure:** `rg -o -t rust '\b[a-z_]+\((&mut [a-z_]+,\s*)?[0-9]{2,}(_[0-9]+)?,\s*[0-9]+' jig/tests jig/testing | rg -v ':(millis|between|randomness|check_redial|assert_replays|fixture|workers|workers_at|reserve_fixture|world_with_rows)\(' | wc -l` (2; target at most ten, each explained). Both matches are the named `first_turn` and `second_turn` fixtures in `jig/testing/jig-client-test-records/src/activity_reads.rs`: positions 10/11, timestamps 20/21 and turn numbers 1/2, independently pinned by `jig/crates/jig-client-protocol-link/src/tests/fixtures/turns.rs`. The numerical target holds; the reading below keeps the item open.

## What

- Core: `jig/tests/core/tests/suite/stories.rs` declares `PROJECT = 1` inside `a_chat_writes_its_own_scope_and_a_party_reads_it`, duplicating the seeded project's name in `jig/testing/jig-core-kit/src/fixtures.rs`.
- Accounts: `jig/tests/accounts/tests/suite/terms.rs` declares `PARTY = 7` beside its imported `HOLDER`; this is the shared `OWNER = 7` in `jig/tests/accounts/src/lib.rs`, which also defines that holder.
- Client held conversation: `jig/testing/jig-client-kit/src/stories/shared/b_held.rs` declares `OWNER = 1`, the kit's `SIGNING_PARTY`. Its scripted and core setup also repeats party 1 and parent task 1 as literals beside the named owner and parent.
- Engine clients: `jig/tests/engine-clients/tests/requests/full.rs` declares the core kit's seeded `PARTY = 1` again and passes its project as literal 1 to `Ask::Stop`.

These copies span four files; they do not fit a one-file correction. Equal values in
independent codec fixtures, transport-owner tokens and different story roles do not
require a shared identity. The count's excluded functions take spans, bounds or seeds,
where the number is the point. The two named turn fixtures above also keep their exact
numbers for the independent decoding assertions.

The people, connector settings, tasks, accounts restore/model/sign-in, client activity,
client documents, scripted-engine sign-ins and service-capacity rows are named. Common
core acts and the notes world's project fixture are shared. Distinct request verbs and
the four core-kit sign-in constructors outside its shared fixtures keep their different
contexts; request keys retain their retry identities.

## Why it matters

A reader should see which fixture a call uses without tracing its numeric identities
through setup and assertions.

## What pays it back

Read the shared fixture names in the remaining rows and sweep their matching literal
identities through each story, preserving every value and assertion. Explain each
number whose exact value is the point. A passing count alone does not close this item.
