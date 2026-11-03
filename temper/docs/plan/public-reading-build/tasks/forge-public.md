+++
id = "forge-public"
part = "temper-public"
kind = "feature"
needs = ["forge-public-wire", "public-reads-core", "public-component"]
files = ["temper/crates/temper-engine-domain-forge/src/public.rs", "temper/crates/temper-engine-domain-forge/src/public/**", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests/public.rs", "temper/crates/temper-engine-domain-forge/src/tests/public/**", "temper/crates/temper-engine-domain-forge/Cargo.toml", "temper/crates/temper-engine-domain/src/route/asks.rs", "temper/crates/temper-engine-domain/src/route/public.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/admission.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/decision.rs", "temper/crates/temper-engine-domain/src/tests/public.rs", "temper/crates/temper-engine-domain/src/tests.rs", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/clients/**", "temper/crates/temper-service/src/route.rs", "temper/crates/temper-service/src/tests/public.rs", "temper/crates/temper-service/src/tests.rs", "temper/tests/engine/forge/**"]
effort = "high"
minutes = 150
+++

# The forge's public documents: projections, the root's admission and routing, the kinds declared

## Goal

A visitor's read of a forge kind reaches the forge and is answered with its public record:
the forge projects *change*, *landings* and *landing* from its changes', landings' and
queues' state, filtered by the public tasks jig's core hands it, names a task only by its
place in the answer's list, and answers `Event::PublicAnswered`; temper's root admits public
reads under its public figures and routes the forge's kinds to the forge; temper-service
declares the kinds to jig's clients' component and keeps the public names' key
(`temper/docs/design/client/forge.md`, section 11; jig's `domain/people.md`, 13.7 and 13.10;
`jig/docs/design/domain/root.md`, 3 and 10).

Acceptance:

- **The projections,** in `temper-engine-domain-forge`'s new module `public`
  (`src/public.rs`, with `src/public/{change,landings,landing}.rs` as you split them),
  declared in `lib.rs` with its types re-exported:

  ```rust
  /// A public task jig's core handed, with the public goal of its tree (forge.md, 11.2).
  pub struct PublicTask { pub task: u64, pub goal: Option<u64> }
  /// What the core asks the forge for (jig's `Ask::Public`), its tasks ascending.
  pub struct PublicAsk<'a> { pub kind: u16, pub version: u16, pub project: u32,
                             pub task: Option<u64>, pub public: &'a [PublicTask] }
  /// The forge's answer: the tasks it names, by place, and its encoded record.
  pub struct PublicMade { pub tasks: Box<[u64]>, pub record: Box<[u8]> }
  impl Domain { pub fn public(&self, ask: &PublicAsk<'_>, limits: &Limits) -> Option<PublicMade> }
  ```
  `None` is unknown: a version other than `public_v1::PUBLIC_VERSION`, a kind other than
  `PUBLIC_CHANGE`, `PUBLIC_LANDINGS` or `PUBLIC_LANDING`, a task missing where the kind needs
  one or given where it takes none, a task not in `public`, a change that is not public, a
  landing that is not public, and an encoding past the codec's limits, each by the same
  return. The records are `temper_client_protocol_wire::public_v1`'s, built from the source
  table of forge.md 11.3 alone and encoded with the generated codec at
  `public_v1::CEILINGS` (as `temper-service/src/family.rs` encodes the panels at
  `v1::CEILINGS`); `temper-client-protocol-wire` becomes a dependency of
  `temper-engine-domain-forge`.
- **Which are public** (forge.md 11.2), checked before anything is ordered or counted:
  a change is public when its task is in `public` (a task jig did not hand is never
  public); a landing is public when it is of `ask.project` and every member's change is
  public (a pin-only member, or any member that is not a public change, makes it not
  public at all); the landing's own task is never named, in `tasks` or anywhere.
  - *change* (`task` a public change's task): forge.md 11.1's row; "a member of a public
    landing" true only when the newest landing it is a member of is public.
  - *landings*: the turn's holder only when it is a public landing; the project's public
    landings newest first, the bound applied after the filter (the panels'
    `landing.limits.landings`); each landing branch's queue as its repository's address and
    its public changes in the queue's order (`domain::queue_ready`'s), each its task, its
    goal (the `PublicTask.goal` handed with it; a queued change whose goal is `None` or not
    in `public` is left out) and when it became ready (`ready_since`, to the second): no
    place and no tasks ahead; a queue listed only when it holds a public change, the
    queues ordered by public fields alone (the repository's address, then the first public
    change's `ready_since`, then its place in `tasks`).
  - *landing* (`task` a public change's task): the newest landing that change is a member
    of, when it is public; unknown otherwise, through any of its members alike.
- **Addresses** (a repository's, a pull's, a CI run's) are made as the panels' are, from the
  forge's records the forge domain holds (`address.rs`'s `pull_address`, and the run's
  address as `temper-service/src/family.rs` makes a panel's link), empty for a fake forge as
  the panels' are; never a title, body, message, branch or name.
- **`tasks`:** every task a record names, the queued changes' goals among them, each once,
  in first-named order; a record's task field is its `u16` place there.
- **temper's root** (`temper-engine-domain`):
  - `core::Ask::Public` (`route/asks.rs`) from the forge's connector
    (`domain.config.forge_connector`) goes to `route/public.rs`, which builds the
    `PublicTask` list from `public` and the core's task records (each task's goal: the
    outermost tracked task at or above it, walking requesters as `jig_domain_core::Core::goal_of`
    walks, `None` when the core holds no live record of it; if jig's core by then exposes
    the public goal of a task, use it instead and say so), calls `domain.forge.public`, and
    steps the core with `core::Event::PublicAnswered { reply, answer }` in the same decision;
    from any other connector, `answer: None` as before.
  - `Event::PublicRead` from the clients' component is admitted when the root's backlog is
    below `limits.public_watermark` and fewer than `limits.public_reads` public reads are in
    the core (a counter in `Domain`, raised on admission and lowered when its
    `Held::PublicRead` leaves, whatever its answer); past either, it is handed back
    `PublicAnswer::Busy` at once, the core not stepped. Admitted, its `owner` is mapped as
    `notes/contracts.md`, section 4 says: 0 stays jig's; the forge family's owner (one more
    than `number(Owner::Forge)`) becomes the forge connector's number plus one
    (`domain.config.forge_connector`); any other owner is answered `Unknown` by the root, after
    the same admission.
  - `Held::PublicRead { reply, answer }` goes to the clients' component as `public-component`
    takes it (mirror `jig/testing/jig-test-service`'s glue that `public-component` wrote).
- **temper-service:** `client_settings` (`clients.rs`) declares in `kit::Settings.public`,
  beside jig's three, the forge's three kinds: `owner` one more than
  `temper_client_protocol_wire::number(Owner::Forge)`, kinds 1, 2, 3, `first` and `last`
  `PUBLIC_VERSION`, `task` false for *landings* and true for the others, `bound` from
  `temper_client_protocol_wire::public_bounds(&bounds)` with the panels' `Bounds` as the
  service makes them; `place` maps the component's public names' owner to
  `(jig_protocol_store::SecretOwner::PublicNames, 0)`; the component's public read reaches
  the root and its answer the component (`route.rs`), as jig-test-service's do.
- **Step tests,** `temper-engine-domain-forge`'s `src/tests/public.rs`:
  - `each_kind_projects_from_the_source_table`: a forge state with one public change, a
    two-member public landing and a queue; each answer decoded and compared with a record
    built independently in the test from forge.md 11.3's table (never by calling the
    projection's helpers);
  - `forbidden_sources_never_reach_the_bytes`: the change's branch, title and body, a gate's
    name, a hold's reason, a red tier's text, a question line, and every task number
    (chosen distinct, e.g. `0x5EC0_0000_0000_0101`, matched as little-endian bytes and as
    decimal text) hold markers, and no answer's bytes contain any of them;
  - `the_meaning_check_catches_a_field_from_the_wrong_source`: the comparison of the first
    test reports a difference when the expected record takes the base hash for the head, or
    the goal's place for the change's (a mutation of the check, kept as a test);
  - `filtered_before_ordered_or_counted`: a queue with private changes ahead of a public
    one: no place, no count of them, the public order kept; a landing's counts of public
    members alone;
  - `a_landing_with_a_member_not_public_is_absent_and_unknown`: absent from *landings*,
    *landing* `None` through either member, and *change* says not a member of a public
    landing;
  - `a_task_not_handed_is_never_named`: a change whose goal was not handed leaves the
    queue; a task not in `public` is unknown for *change* and *landing*;
  - `unknown_is_one_return`: wrong version, unknown kind, a task given to *landings*, a
    missing task, another project's change: each `None`;
  - `the_largest_answers_fit_their_declared_bounds`: each kind at its bounds encodes within
    `public_bounds`' value for it.
- **Step tests,** `temper-engine-domain`'s `src/tests/public.rs`:
  `a_forge_kind_is_asked_of_the_forge_with_its_public_tasks`,
  `a_public_read_past_the_public_watermark_is_busy`,
  `public_reads_past_public_reads_are_busy_and_freed_on_every_answer`,
  `a_public_goal_is_the_outermost_tracked_task`; `temper-service`'s `src/tests/public.rs`:
  `the_forge_declares_its_three_public_kinds`, `the_public_names_key_is_kept_under_its_owner`.
- **The forge's world** (`temper/tests/engine/forge`, forge.md 11.5), a new
  `tests/suite/public.rs` (and `src/public.rs` in the world for its helpers), each story with
  forbidden sources holding markers and asserting no answer's bytes hold one:
  `a_public_change_read_at_each_step_until_landed`;
  `a_two_member_landing_with_a_failed_gate_through_either_member` (the run's address in it);
  `a_private_project_change_an_outside_change_and_a_conversations_change_are_unknown`
  (the handed list empty, missing the task, or missing the conversation's tree);
  `a_queued_change_outside_every_goal_is_counted_nowhere`;
  `a_landing_with_a_member_not_public_is_absent_and_unknown`;
  `a_task_jig_did_not_hand_is_never_named`. The golden's failure on an added field is
  `forge-public-wire`'s test; this world does not repeat it.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md` ("The rules that are the point"
  above all); `notes/contracts.md`, sections 1 (the roots' public figures), 4 (`Ask::Public`,
  `Event::PublicAnswered`, `Held::PublicRead`, `PublicKind`) and 5.
- `temper/docs/design/client/forge.md`, 11.1-11.5; `temper/docs/design/domain/landing.md`,
  4.1 (pin-only members) and 4.2 (the landing's own task); `temper/docs/design/domain/forge.md`,
  section 9 (changes outside goals queue); jig's `jig/docs/design/domain/people.md`, 13.7 and
  13.10; `jig/docs/design/domain/root.md`, 3 and 10; `jig/docs/design/protocol/client.md`,
  section 19.
- `tasks/forge-public-wire.md` (the records, `public_bounds`), `tasks/public-vocabulary.md`
  (the root's arms you replace), and the merged code of `public-reads-core` and
  `public-component` (jig's root admission in `jig/testing/jig-test-domain`, the component's
  glue in `jig/testing/jig-test-service`).
- As they stand: `temper-engine-domain-forge`'s `views.rs` (`document`, `landing_view`,
  `landings_view`, `turn_holder`), `panel.rs` (`panel`, `queue_place`), `domain.rs`
  (`queue_ready`), `address.rs`; `temper-engine-domain`'s `route/asks.rs`, `route/panels.rs`,
  `admission.rs`, `domain.rs` (`backlog`); `temper-service`'s `clients.rs` (`client_settings`,
  `place`), `route.rs` (the deliveries), `family.rs` (`panel`, `pull_address`);
  `temper/tests/engine/forge`'s `src/world.rs`, `src/landings.rs`, `tests/suite/landing.rs`.

## Owns and may change

- Owns the files in the header; in `lib.rs`, `route.rs`, `domain.rs`, `boundary.rs`,
  `decision.rs`, `tests.rs` and `clients.rs` only the hunks public reading needs (a module
  line, a counter, a variant, a call). `public-limits` gave temper's root `public_watermark`
  and `public_reads`; read them, never change their meaning.
- **Reader files:** `temper/crates/temper-engine-domain/Cargo.toml` and
  `temper/tests/engine/forge/Cargo.toml` gain `temper-client-protocol-wire` (a dependency
  line each, the world's as a dependency for decoding); `temper-service`'s
  `src/tests/fixtures.rs` only for a literal the compiler names.
- `temper-client-protocol-wire` is not yours: a record that cannot carry a field of forge.md
  11.1 is a question, never a schema edit.
- Pre-approved:
  - If `public-component` already declared the forge's kinds or mapped `PublicNames` in
    temper-service, check them against the acceptance and keep them.
  - If temper's root receives the component's public read through a `PartyInput` rather than
    a root `Event`, keep the path `public-vocabulary` gave it and admit there.
  - If a CI run's address cannot be made from what the forge domain holds, carry it empty
    as a fake forge's is and report it under `## Left`.
  - If the panels' landings bound is not `landing.limits.landings`, use the bound
    `public_bounds` was computed at and say which.
  - If `public_v1` has no `CEILINGS`, encode with `public_limits` of a `Bounds` made from
    the forge's `Limits` as the service makes the panels', and say how.

## Steps

1. `temper-engine-domain-forge`'s `public` module and its step tests; run them alone
   (`heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/public/)'`).
2. The root's routing and admission, its step tests
   (`heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/public/)'`).
3. temper-service's kinds, secret and glue; its step tests
   (`heavy cargo nextest run -p temper-service --lib -E 'test(/public/)'`).
4. The forge world's stories
   (`heavy cargo nextest run -p temper-engine-forge-world --test suite -E 'test(/public::/)'`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-forge -p temper-engine-domain -p temper-service -p
   temper-engine-forge-world --all-targets -- -D warnings`; the focused tests of those
   packages and of `temper/tests/engine/domain`'s world (`-p temper-engine-domain-world`); the citations check. This changes what temper's root answers a public
   read (busy before): run `temper-one-process-world`'s focused tests too.
6. Commit (why: a public project's forge read by anyone, with nothing but its structure and
   links, and no task jig did not hand); push your branch.

## Report

As the implementer's text says, and also: each kind's record filled field by field from
which source, how the queue's goal is found, the three declared bounds at the default
limits, and the path a public read takes through temper's root and service.
