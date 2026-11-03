+++
id = "public-story"
part = "jig-public"
kind = "feature"
needs = ["public-reads-core", "public-http"]
files = ["jig/tests/core/src/stories/public.rs", "jig/tests/core/src/stories/public/**", "jig/tests/core/src/referee/public.rs", "jig/tests/core/tests/suite/public.rs", "jig/tests/core/tests/suite/public/**", "jig/tests/client-protocol/src/public.rs", "jig/tests/client-protocol/tests/suite/public.rs"]
effort = "high"
minutes = 120
+++

# Public reading's stories: the core's world, its races and referee, and one read over HTTP

## Goal

jig's core world plays people.md 13.6: the stories, each race both ways round, and a referee
that would fail on any rule of public reading broken; and the client-protocol world reads a
public board over real HTTP bytes through `jig-test-service`, public reading figured on in
that story alone (`jig/docs/design/domain/people.md`, 13.6; `jig/docs/design/protocol/client.md`,
section 19's Testing).

Acceptance:

- **Stories,** `jig/tests/core/tests/suite/public.rs` (a module line in `main.rs`), driven
  as the world's other stories are (`src/stories/public.rs`, a module line in
  `src/stories.rs`), with a fixture whose titles, specs, project and charter names, words and
  numbers hold distinct markers:
  - `an_owner_makes_a_project_public_and_the_public_party_reads_its_board_and_a_delegates_card`:
    the answers decoded as the core's `PublicDocument` hold no marker and no internal number
    (identifiers are the component's; at the core, the board names the tasks the index holds
    and nothing else);
  - `a_members_set_visibility_is_refused`;
  - `a_goal_beneath_a_conversation_reads_as_a_public_goal_with_no_parent_and_the_conversation_is_never_named_or_counted`;
  - `a_delegate_moved_out_of_its_goals_tree_leaves_its_board_counts_and_card` (a delegate
    moved in is not built: `Move` under another task, `jig/docs/design/domain/people.status.md`;
    the story says so in its doc and plays no such move);
  - `private_never_made_and_deleted_projects_answer_unknown_with_the_same_work` (the
    outputs of each read compared whole, but for its reply);
  - `a_conversations_delegates_card_is_unknown`;
  - `a_deleted_public_project_is_unknown_from_its_deletions_decision`;
  - `the_public_watermark_and_the_cores_public_reads_refuse_a_public_read_and_admit_a_signed_in_one`;
  - `a_rate_of_zero_refuses_every_public_read` and
    `every_credential_less_exchange_but_a_public_read_is_signed_out`, and the room, rate and
    bytes stories, played here if the world joins the clients' component (as
    `client_peer_routes.rs` does, if it does), otherwise named in the report as covered by
    `public-component`'s world stories.
- **Races,** each played both ways round (the change decided before the read is admitted,
  and while it is under way: an application's document asked of its connector, and an answer
  waiting to be durable), each read answered unknown: `made_private_while_a_public_read_is_under_way`,
  `deletion_admitted_while_a_public_read_is_under_way`, `moved_out_while_a_public_read_is_under_way`,
  `untracked_while_a_public_read_is_under_way`; and `a_reservation_is_given_back_on_every_end`
  if the world joins the component.
- **The referee** (`src/referee/public.rs`, called from the world's referee as its other
  rules are, a module line in `src/referee.rs`), checked after every step of every story of
  the world (not only these): the public party commits nothing and appears in no record; no
  public document holds a task the index did not hold at its decision, or a value outside its
  type's fields; no public read of a project is answered after the decision that made it
  private, began its deletion, or took the task out of its public tree; no public read is
  admitted while the root's backlog is at its public watermark; and, where the world joins
  the component, the bytes reserved and given back sum to the bytes answered. A referee test
  feeds it a planted breach of each rule and sees it fail
  (`the_public_referee_catches_each_breach`).
- **Over HTTP,** `jig/tests/client-protocol/tests/suite/public.rs` (a module line in
  `main.rs`), the world's helpers in `src/public.rs` (a module line in `src/lib.rs`):
  `a_public_board_is_read_over_http_bytes_through_the_service` — with the service's limits
  giving public reading non-zero figures in this story alone, a signed-in party makes a goal
  and its project public by the protocol (`SetVisibility`), reads its public identifier in
  `me`, and a credential-less `GET` built by `jig_client_protocol_http::public` crosses the
  world's network to `jig-test-service`; the `200` body decodes as `public_v1::PublicBoard`
  naming the project's identifier and the goal's line, `cache-control` carries
  `must-revalidate` and a `max-age` within the figure; the project made private, the same
  `GET` is `404`.

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 2,
  3, 4.
- `jig/docs/design/domain/people.md`, 13.2, 13.6 (the stories, races and referee), 13.7,
  13.10; `jig/docs/design/protocol/client.md`, 17.5 and section 19.
- As they stand: `jig/tests/core` (`src/stories.rs`, `src/referee.rs`, `src/world.rs`,
  `tests/suite/deletion.rs` and `projects.rs` as models); `jig/tests/client-protocol`
  (`src/native.rs`, `src/net.rs`, `src/stories.rs`, `src/fixtures.rs`, `tests/suite/protocol.rs`);
  the reports of `public-index`, `public-reads-core`, `public-component`, `public-http`.

## Owns and may change

- Owns the files in the header.
- One module line each: `jig/tests/core/src/stories.rs`, `src/referee.rs` (and the call of
  the public rule where the referee runs its rules), `tests/suite/main.rs`;
  `jig/tests/client-protocol/src/lib.rs`, `tests/suite/main.rs`.
- `jig/tests/client-protocol/src/fixtures.rs`: a function giving the service's limits with
  public reading on, used by this story alone; no other story's figures change.
- `jig/tests/core/src/fixtures.rs` and `src/world.rs`: a constructor or helper this story
  needs (a public-read driver, a fixture with markers), added beside the others; nothing
  existing changes.
- No production code: a fault the stories find is a question, with the failing seed, unless
  it is in a file `public-reads-core`, `public-component` or `public-http` owned and the fix is
  one the spec there already states, which you may make and report.

## Steps

1. The core world's driver and the stories; run them alone
   (`heavy cargo nextest run -p jig-core-world --test suite -E 'test(/public::/)'`).
2. The referee and its breach test; then the whole core world, since the referee runs in
   every story (`heavy cargo nextest run -p jig-core-world`).
3. The HTTP story; run it alone
   (`heavy cargo nextest run -p jig-client-protocol-world --test suite -E 'test(/public::/)'`),
   then the world.
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-core-world -p jig-client-protocol-world --all-targets -- -D warnings`;
   check both worlds' times against their tier's budget (`AGENTS.md`, 6); the citations check.
5. Commit (why: public reading's stories, races and referee, and its meeting point over
   HTTP); push your branch.

## Report

As the implementer's text says, and also: which stories ran where (the core world or the
component's), each race's two orders, the referee's rules with the breach each catches, and
the worlds' times before and after.
