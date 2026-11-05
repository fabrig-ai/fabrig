# debt-23: folding arguments (jig D016)

Every `args-*` task names this note. The sites and folds come from a read-only survey at this plan's base: line numbers are there, so find a function by name. Re-measure first.

## The rules

- **Paid when** a production function needs no `too_many_arguments` exception. The default threshold of 7 applies, and `&self` counts. Each site is at most 5 over, so fold what the table says and no more.
- **Owned values only.** `skein/docs/foundation/programming-model.md`, 10.3 keeps references and lifetime parameters out of step code's fields, so a fold owns or copies what it holds. `core, env, work` and every `&mut Queue` stay loose. No `Route<'a>`, whatever `moving::Moving<'a>` does.
- **A fold names one thing** a reader would call by that name: what a request brings, not "the args of `f`". A pair that doesn't name a thing, split the function instead, or drop an argument nothing reads.
- **An existing type first.** Section "Existing types" has them. A new struct is private, or `pub(crate)` if another file calls the function, and lives by its first site, as the table says. It derives only what its use needs, and has no constructor unless it shortens its callers.
- **No public type changes.** No `Event`, `Request` or `boundary` type, no `pub` signature. D016's `Event::Turn(Turn)` isn't needed for any site: a route unpacks the variant and builds the fold at its helper's call. That's what keeps this from being a breaking change.
- **Behaviour stays.** A fold moves no check, no order of evaluation, no `.clone()` into or out of a path. Building a fold from a variant's fields moves them as the arguments did. A site whose body reads a field after moving another reads it first, as `inbox::State` needs.
- **The exception's attribute goes or shrinks.** `decide_with` and `signin` hold `too_many_arguments` and `too_many_lines` in one `#[expect]`: keep the other lint and its reason true. Where a body drops under `too_many_lines`, that `#[expect]` goes red too, and goes.
- **No new test.** A fold is a move: the tests that reach these functions stay, with only a call's spelling changing. So no rule to break by hand.
- **Size:** net lines in each task are the survey's estimate, plus up to half again before you stop and ask, stating the size. A struct costs about as many lines as it has fields.

## Existing types

| Type | Pays |
|---|---|
| `routing::party::dispatch::Party { request, person, project, role }` | `make_project_act`: replaces (request, maker) |
| `people::ActKey { connector, key }` | `deployment_acts::start` |
| `crate::TurnProof { turn, cumulative, read }` | `route_turn_payload`, to 7 |
| `crate::TurnRecord { opens_window, task, attempt, turn, spent, read, at, transcript }` | `route_accepted_turn(core, env, accepted, record)`, built at its caller in `routing/mod.rs` with `at: env.wall`; on `Accepted::Already` it's dropped unread |
| `accounts::keys::Keeping { request, holder, provider, join, replacing }` | `keys::add`, which builds it first thing today |
| `people::domain::inbox::State { party, project, task, last_reply, live, seen }` | both chat sites, which build it inside; it becomes `pub(super)` |
| `host::hosted::NamedEvent { name, event, sender, finish_requested }` | `inbound`, which only wraps `inbound_named`; a small constructor |
| `fleet::attempt::Names { run, attempt }` | `start`, `start_job`; it becomes `pub(crate)` |
| `views::TurnName { attempt, turn, retry }` | inside core's `Written` |

## New structs

| Struct | Fields | Lives in | Sites |
|---|---|---|---|
| `Signed` | `reply_to: ReplyTo, sign_in: u64, key: [u8; 16]` | `routing/party/dispatch.rs` | `settings::dispatch::start`, `propose_start`, `key_routes::start`, `application_routes::start` |
| `Judged` | `change, room, kept, proposal` | `routing/settings/dispatch.rs` | `decide_judged`, `decide_with`; `decide` stays at 7 |
| `Answer` | `payload, cumulative, end, saved, invalid_saved, read, refused: Box<[u64]>` | `routing/run/payloads.rs` | `route_answer_payload`, to 6 |
| `Prepared` | `writes, holds, credentials, workspace` | `routing/run/preparation.rs` | `workspace_prepared` |
| `JobItems` | `holds, workspace` | `routing/run/jobs.rs` | `jobs::prepared` |
| `Written` | `run: Token, turn: views::TurnName, offset: u32` | `routing/view/routes.rs` | `run_text`, `run_input` |
| `Call` | `to: ReplyTo, key: CallKey` | `routing/run.rs` | `named_proposal_decide`, `named_escalation` |
| `Proposing` | `to, key, reason: Box<[u8]>, as_holder: bool` | `routing/run.rs` | `effects::propose_start`, `complete_proposal`, `named_propose`, `propose_batch` |
| `Described` | `owner: Token, number: u16, description: connector::EffectDescription` | `effects.rs` | `complete_proposal` |
| `ProcedureCall` | `task: u64, step: u64, connector: u16` | `routing/effect/procedure.rs` | `procedure_step`, `procedure_holdings`, `procedure_propose` |
| `AgentAmend` | `token, key, target, revision, amendment`, as `Replay::Agent` has them | `routing/run/calls.rs` | `named_amend` |
| `Creator` | `project: u32, person: u64, role: people::Role` | `acts/bounds.rs` | `create_start` |
| `Drawn` | `numbers: Numbers, source: Source` | authority's `check.rs` | `batch`, `price` |
| `Claiming`, `Placing` | `to, workstream, kinds, version`, which is `State::Waiting` less `serial`; `assignment, job, channel, names` | fleet's `attempt.rs` | `start`, `located`, `assigned`, `hosted` |
| `Ended` | `number, attempt, end, saved` | tasks' `run.rs` | both `activation`s |
| `ProposalDeciding` | `proposer, number, by, message, decision` | tasks' `proposals.rs` | `decide`, `decide_person` |
| `At` | `run: Token, turn: TurnName, offset: u32` | views' `text.rs` | `written`, `input` |

## Single sites

A pair or small struct by the site, named for what it holds. Their arguments and counts are at the base.

| Crate | Site | Over | Fold |
|---|---|---|---|
| core | `effect/topics::subscribe` | 1 | (connector, topic) |
| core | `settings/restart::revision_writes_with_hold` | 1 | (newest, history) |
| fleet | `attempt::start_job` | 2 | `Names`, and (job, restoring) |
| host | `hosted::serve` | 2 | a `Served` |
| host | `hosted::finished` | 1 | a `Last` |
| people | `domain/parties::signin` | 2 | a `SigningIn` |
| people | `domain/requests::admit_ask` | 1 | (sign_in, bytes) |
| tasks | `admission::turn` | 5 | a `Turned` |
| tasks | `run::claim` | 3 | a `Claiming` |
| tasks | `control::amend` | 1 | (by, funder) |
| tasks | `control::dependency_reached` | 1 | (project, task, dependency) |
| tasks | `escalation::decide` | 2 | a `Deciding` |
| tasks | `funders::carve` | 1 | (period, budget) |
| tasks | `moving::apply` | 2 | (person, period, two budgets) |
| tasks | `procedure::stepped` | 1 | (step, read) |
| tasks | `proposals::stalled` | 1 | (proposer, number) |
| tasks | `questions::person_question` | 1 | (project, person) |
| tasks | `questions::person_answer` | 3 | an `Answering` |
| tasks | `wake::met` | 1 | (dependency, removed) |
| views | `watch::open` | 1 | (window, follow) |
| store | `generations::manifest` | 1 | (store, secrets) |
| store | `patching/phase/walking::seal` | 1 | a `Sealing` |
| ops | `domain-infrastructure/application::decide` | 3 | drop `_person`, `_project` (`person: _, project: _` at its caller), fold (token, owner) |
| ops | `domain/client_documents::document` | 1 | (watcher, subject) |
| ops | `domain/client_documents::read` | 2 | (reply, scope, read, party) |

Where a cell says a pair and the function reads better split, or a name fits better, the choice is yours: the binding part is the rules above and an empty search.

## Seams kept whole

D016's **Paid when** allows a seam a comment names as kept whole on purpose. Two, both public, so a fold would break an interface the request says to stop at:

- `jig_domain_accounts::Domain::select`: jig-domain-core and jig-accounts-world read it, and three of its arguments are borrowed slices, which no fold may hold.
- `jig_protocol_worker_agents::sources::Component::start`: temper-service reads it.

So the item's Measure reads **2** when it's paid, not 0, and those two keep their `#[expect]`.
