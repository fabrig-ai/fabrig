+++
id = "native-walk-a"
part = "native-walk"
kind = "feature"
needs = ["playground-visitor"]
files = [
  "temper/tests/playground/**",
  "temper/crates/temper-playground/src/story.rs",
  "temper/crates/temper-playground/src/stories.rs",
  "temper/crates/temper-playground/src/tests/**",
  "temper/crates/temper-playground/Cargo.toml",
  "temper/tests/one-process/tests/suite/story_a.rs",
  "temper/crates/temper-playground/src/documents.rs",
  "temper/crates/temper-playground/src/documents/**",
  "temper/crates/temper-playground/stories/a/deployment.json",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/crates/temper-engine-domain/src/tests/setup.rs",
  "temper/crates/temper-playground/src/programs.rs",
  "temper/tests/one-process/src/world.rs",
  "temper/crates/temper-service/src/stock.rs",
  "temper/crates/temper-service/src/tests/stock.rs",
  "temper/tests/engine/domain/tests/suite/settings/b2.rs",
  "temper/docs/design/protocol/configuration.md",
  "temper/docs/help/budget.md",
  "Cargo.lock",
]
effort = "high"
minutes = 150
+++

# Story A walked natively through the web client's host, its log kept and replayed

## Goal

"Natively, in the gate, `tests/playground` (`temper-playground-world`): each story walked whole
in the playground's world through the web client's host built natively, driven by skein's fake
person on its tree as a journey's replay drives it, the host's duties carried to the world by
skein's bridge in memory: story A from the welcome to the CSV change landed and the other two
requests answered" (`temper/docs/design/playground.md`, 9). This is the plan's meeting point: the
client's real host, the bridge, the visitor and the composition, in one thread. Build the page
natively (`notes/contracts.md`, C3), walk story A, keep its session log (C7) and replay it.

Acceptance:

- `temper-playground-world`'s library gives C3: `page::{Page, Report, Walked}` and
  `stories::a`.
- New tests in `temper/tests/playground/tests/suite/walk.rs` (its `mod walk;` in `main.rs`):
  - `story_a_walks_from_the_welcome_to_the_csv_change_landed`: `Page::start(&A, SEED,
    Wall::EPOCH)`, then `walk` with `stories::a()` ends `Walked::Done`; the person saw each of the
    three requests' reply lines and the CSV change landed; the forge's `main` holds the scripted
    CSV edit with every check passed (as the one-process world's `outcome` reads it,
    `temper/tests/one-process/src/stories/playground.rs`, ~83-115); the page's log equals the kept
    `tests/fixtures/story-a.log` byte for byte, written first under `SKEIN_CODEGEN_WRITE` (C7).
  - `story_a_kept_log_replays_natively`: the kept log, read by skein's `log` reader, replays with
    `skein_playground::replay::replay` over `temper_playground::world(A.start(seed, start))` at
    the log's seed and start, `story::reach()`, no referee (`&mut ()`), inside
    `programs::with(story::parameters(..), ..)`: `Ok`.
  - `the_visitors_exchanges_are_the_pages_fetch_bound`: `documents::VISITOR_EXCHANGES ==
    temper_client_shell_web::limits::EXCHANGES + 1` (C1).
- Each test passes alone and in the package's run; the walk within a world's budget
  (`temper/docs/design/domain/testing.md`, 6.1); over it is red (`notes/common.md`).
- `heavy cargo nextest run -p temper-playground-world` passes with the trunk's count plus three;
  `heavy cargo clippy -p temper-playground-world --all-targets -- -D warnings` passes.

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md`;
  `temper/docs/plan/playground-browser/notes/contracts.md`, C1, C2, C3 and C7.
- `temper/docs/design/playground.md`, 2 (what the visitor sees), 5.2 (story A), 6 (the starter)
  and 9; `temper/docs/design/client/journeys.md`, 2 (the host a replay drives).
- `skein/docs/design/playground.md`, 5 (the bridge in memory) and 6 (the session log, its text,
  the replay).
- Code to copy the shape of:
  - the browser shell's loop, `temper/crates/temper-client-shell-web/src/lib.rs` (`start` and
    `Shell::turn`): the host's reports, `adapter::requests`, `transport::carry`, `Carried`
    raised again, `read_on` after a piece;
  - skein's native driver over `Memory`, `skein/tests/playground/tests/suite/memory/support.rs`
    (`Shell::new`, `carry`, `until_message`);
  - the person's loop over a tree, `temper/tests/one-process/src/world.rs` (~640-690):
    `TreeFace::new(tree)`, `Person::next`, `Doing::Act` as `host::Report::Act`, `Doing::Go` as
    `host::Report::Place` with the address printed by `temper_client_domain::address::print`;
  - story A's script, `temper/tests/one-process/src/stories/playground.rs` (`script`, ~20-58),
    and its outcome (~83-115);
  - jig's native sign-in take, `jig/crates/jig-client-shell-web/src/sign_in.rs` (`take`, `Route`
    as the shell passes it: `/jig/v1/sign-in/link`, home `/welcome`).

## Owns and may change

- `temper/tests/playground/**`: `Cargo.toml` gains, in `[dependencies]`, the workspace crates the
  driver uses (`temper-client-shell-web`, `temper-client-domain`, `jig-client-shell-web`,
  `skein-ui`, `skein-ui-web`, `skein-playground`, `skein-fake-person`, `serde_json`, and
  `temper-fake-git` or `temper-fake-git-hosted` for the outcome); `src/lib.rs` its `pub mod page;
  pub mod stories;`; the new files; `tests/suite/main.rs` its one `mod` line;
  `tests/fixtures/story-a.log`.
- `Cargo.lock`: this package's dependency lines.
- No other file. In particular `temper-client-shell-web`, `temper-playground` and the one-process
  world stay as the trunk has them: a change one of them would need is a question.
- Pre-approved options:
  - **The browser's time:** the driver moves its browser instant only to `Wake::At`, the host's
    deadline through `Memory::timer`, or the person's `Wait { until }` converted the same way,
    whichever is first; no wall clock. The world runs at 1×; if story A's walk exceeds the world's
    budget at 1× because of time steps alone, pace it at 10× (`Memory::pace`) and say so.
  - **The person's script:** the one-process script's steps and words; where the web client
    names a control otherwise than the terminal client (find it in temper-client-view's words),
    use the web's and list each difference in the report. The last steps see the CSV change's
    "Landed" (`temper-client-view`'s words) on the chat or the change's page, wherever the web
    client shows it first.
  - **The page's store** is `skein_ui_web::store::Store` as the shell makes it, over the
    `Memory` platform's memory store.
  - If story A stalls in this world (the person's `Failed` or the bound passed), print the
    person's failure and the page's named nodes as the one-process world's `failed` does, and
    stop and ask with that output: a stall is a finding, not this task's to work around.
  - If the replay of the kept log differs, report `Differs` whole and stop and ask: the walk and
    the replay step one world.

## Steps

1. `page.rs`: `Report`, `Page::start` (the world in slices, the link among the printed lines, the
   host started at the link's path and query with its secret), the turn loop, `walk`, `log`.
   A first test of your own may stop at the welcome; keep it only if it adds what the walk does
   not.
2. `stories.rs`: story A's person.
3. The three tests; record the log with `SKEIN_CODEGEN_WRITE=1` once, then run without it.
4. `cargo fmt`, the old-form count, clippy and the package's tests; commit per step (the fixture
   with the test that keeps it); push.

## Report

As the implementer's text says, and also: the walk's world time, step count and wall time; the
log's size in bytes and its input count; the script's differences from the one-process one; the
pace used.

## Answers

2026-11-03. **Who diagnoses story A's stall at "Starting" (person step 3, no "Add CSV export to
widget"): the composition's owner, or this task?** This task, diagnosis first: the native walk is
the plan's meeting point, the stall is its own acceptance, and nothing yet says which side holds
the fault. One-process story A walks the same composition through the terminal client; what is
new here is the driver, the web host and the visitor. Steps, with throwaway diagnostics only:

1. **The world's side at the stall.** Keep the world's printed lines through the walk (the turn
   loop drops `Heard::Printed` now) and print, on failure, every line of each process's standard
   error and output after the link. Read the world's state as the one-process world's `outcome`
   does: did `StartPlayground` reach the service, is the Playground forge's record there and
   connected, is `playground/widget` adopted on the fake forge, does the project exist.
2. **The page's side.** Count, through the walk, the host's requests carried by
   `transport::carry` and the bridge's answers (`StreamOpened`, `StreamPiece`, `StreamRefused`,
   each fetch's status), and whether `read_on` follows each piece; check the person's wait and
   the host's deadline move `self.browser`, so the world steps between the person's polls.

Then:

- **The fault is in this package** (the driver's carrying, its stream reading, its time steps,
  the script): fix it here and go on with the acceptance as written.
- **The world finished `StartPlayground` but the page never shows it,** and the cause is in
  `temper-client-shell-web` (`host`, `adapter`, `transport`) or skein's bridge: stop and ask with
  the evidence, file and line. Those stay as the trunk has them.
- **The world itself never finishes `StartPlayground`** (the composition with the visitor, the
  fakes, the service): stop and ask with the evidence and the process lines from step 1, naming
  the component; `temper-playground` is `playground-visitor`'s and `playground-worker`'s now.

The report gives the diagnosis whichever way it goes, with the counts and lines that show it.

2026-11-03. **Who repairs the stall at `Starting` (forge 1 `NotSetUp`, nothing adopted): a
re-opened `playground-worker`, or this task widened?** This task, widened by exactly that repair:
its acceptance cannot hold without it, and `playground-visitor` and `playground-worker` have both
merged into the trunk, so either would be a rerun anyway. This answer overrides the previous one
for this repair only.

The cause, confirmed in the code: with no box the service reads a fresh fake forge's handover from
its setup entry's file. A forge `StartPlayground` makes is `Prepared` with no handover, so
`start_pending` (`temper/crates/temper-service/src/setup/lifecycle.rs`, ~956) sets
`FileStep::Reading` on `<data>/playground-N/admin_token`. The file is missing, so `file_answer`
calls `start_local` with no handover, and `StartFake` with no handover is `NotSetUp` (~981). The
design says the composition lays it: "its setup entry the playground's, whose handover the
composition lays on the disk" (`temper/docs/design/playground.md`, 4). `world_metered`
(`temper/crates/temper-playground/src/story.rs`, ~59) lays handovers only for a `setup` array,
which story A has none of. The one-process world's `story_a` is ignored for the same reason
(`temper/tests/one-process/tests/suite/story_a.rs`, its `#[ignore]` text).

The repair, the service unchanged:

- `world_metered` also lays `fakes::ADMIN_TOKEN` with `machine::lay_handover` at the playground
  forge's setup-entry file. That is the fake entry's directory for the number `StartPlayground`
  gives (in a fresh world, 1, after any forges the document's `setup` array makes), joined with
  `admin_token`, in the deployment's data directory as the service resolves it. Derive the path as
  the service does (`temper_forge_setup::machine::entry(.., BoxKind::Fake, ..)` and
  `place::Place::path(File::AdminToken)`, if they are public; otherwise the same join), not a
  hand-written string. Lay it only when the deployment has `application.demo` (the playground is
  offered).
- A test in `temper-playground`'s `src/tests/` (beside `world.rs`'s handover test, ~123-142): a
  fresh world of story A holds the handover at that path with the fake's admin token.
- `temper/tests/one-process/tests/suite/story_a.rs`: run its ignored test once alone with
  `--run-ignored all`. If it passes, remove the `#[ignore]`. If it fails, keep the ignore, change
  its reason to the new stall in one line, and report the failure under `## Left`; do not chase it.
- `Cargo.toml` of `temper-playground` only if the path's derivation needs `temper-forge-setup` as
  a dependency.

Then the walk goes on as the acceptance says. If it stalls again past the fake forge's connection
(the takeover, the account, the project or the adoption), the previous answer's diagnosis and
stop rules apply: a fault in the service or the engine is a question, not this task's. Add
`heavy cargo nextest run -p temper-playground` to the checks.

2026-11-03. **Takeover is refused (401): lay story A's seeded admin credential, or change the
peer to accept `fakes::ADMIN_TOKEN`?** Lay the seeded credential; the peer stays as it is. The
handover the composition lays is the one its own forge peer accepts (`temper/docs/design/playground.md`,
4), and a story's peer draws its credential from the visit's one seed as `temper fake` does (5.1,
section 4's one seed); the previous answer's `fakes::ADMIN_TOKEN` was wrong for a story's world, and
this answer overrides it for that value only. `fakes::ADMIN_TOKEN` stays for the `setup` array's
entries, whose peer `Fakes::new` builds with it.

- `stories.rs` (added to this task's files for this alone): one function gives the credential,
  `pub(crate) fn admin_token(seed: u64) -> [u8; 40]`, returning
  `credential::credential(documents::seeds(seed).forge, credential::ADMIN_TOKEN)`, and
  `Story::start` uses it in place of its line 44. No other change there.
- `story.rs`: `world_metered` lays `stories::admin_token(seed)` at the playground forge's path,
  still only when `application.demo` is present; every world with a demo today is built by
  `Story::start`. The regression expects that credential.
- `temper/tests/one-process/tests/suite/story_a.rs`: run its ignored test once alone again with
  `--run-ignored all`; passing, remove the `#[ignore]`; failing, keep it with a one-line reason
  and report it under `## Left`, not chased.

Then the walk goes on as the acceptance says, with the earlier answers' diagnosis and stop rules
past the takeover.

2026-11-03. **Project creation fails after the account (`StartFailure::Adoption`, account 1, no
project): widen this task, or leave it to an owner?** Widen this task, narrowly, diagnosis first. No
task of this plan owns the engine or the composition's documents, and the native walk, the replay
(`one-process-replay`) and the browser walk all wait on story A starting: the acceptance cannot hold
without it. This task's files gain `temper-playground`'s `src/documents.rs` and `src/documents/**`,
and the engine's `src/route/playground.rs` and `src/tests/setup.rs`, for this repair alone.

1. **Which branch fired.** With throwaway diagnostics only, find whether `project`
   (`temper/crates/temper-engine-domain/src/route/playground.rs`, ~112) found no
   `playground_policy` (the first of `config.core.defaults.policies`, `domain.rs` ~334), or
   `Reply::Project` (~170) got a reply other than `ProjectMade`; print that reply whole (a
   `Refused`'s reason, a `Full`'s capacity). The report names it.
2. **Then repair where the cause is:**
   - **The composition's documents** (no default policy, a project or name capacity, the policy's
     number or its authority as the playground's documents set them, `documents.rs` ~120-200 and
     ~341-400): fix the documents, so they stay just big enough for the stories
     (`temper/docs/design/playground.md`, 8). A test in `temper-playground`'s `src/tests/` that a
     fresh world of story A, after `StartPlayground`, holds the project; or, if a document check is
     the narrower proof, that.
   - **The engine's start** (it asks with the wrong policy, sign-in, key or holder for what
     `protocol/setup.md`, 8.4 says the start makes): fix `route/playground.rs`, with a test in
     `src/tests/setup.rs` beside its playground tests that the start's project ask is answered
     `ProjectMade`. No stored format, record or protocol changes; one that seems needed is a
     question.
   - **jig's people or core refuse a well-formed ask** (`jig-domain-core`, `jig-domain-people`):
     stop and ask with the reply, file and line; jig is not this task's.
3. **Past the project** (the uses, the adoption): the same rule again, once; a second new failure
   past this repair is a question with the same evidence, not chased.
4. **The read loop** (the host asking `/jig/v1/read` at once after the failure, world and browser
   held at 0): not this repair. If it is the driver's (a read answered at once that should wait on
   the world), fix it here; if it is `temper-client-shell-web`'s host, leave it, report it under
   `## Left` with its file and line, and make the driver's walk bound also count turns at one
   instant, so a stall fails the walk with the person's failure and the page's nodes instead of
   reaching nextest's wall limit.
5. **The one-process story A** (`story_a.rs`): after the repair, run it once alone with
   `--run-ignored all`; passing, remove the `#[ignore]`; failing, keep it with a one-line reason and
   report it under `## Left`, not chased (its account is absent there, a different stall).

Add `heavy cargo nextest run -p temper-engine-domain --lib -E 'test(/playground/)'` to the checks
if the engine changed; then the walk goes on as the acceptance says.

2026-11-03. **Four runs without landing: go on, or rethink the task?** Go on, with the previous
answer (project creation) as the next run's work; no new decision. The runs are no loop: each
reached a new stall its stop rule named, its answer repaired it, and the next got further (the
handover, then the takeover, then the account). The branch's pushed commits stay; the next run starts
from them, not afresh. The previous answer's step 3 still holds: a second new failure past the
project repair is a question with its evidence, and the run commits and pushes its repairs, each with
its own test passing, before it asks.

2026-11-03. **Project creation is refused `Again(Busy)` at `people.pending` 2: add story A's
`stories/a/deployment.json` to this task's files, or leave the walk blocked?** Add it, for this
repair alone. The cause is the composition's documents, the branch the previous answer named, and
story A's compiled deployment is one of those documents: it is not just big enough for its story
(`temper/docs/design/playground.md`, 8) while the outer `StartPlayground` application and the
account request hold both pending slots when the project is asked.

- `people.pending` becomes 3 in `temper/crates/temper-playground/stories/a/deployment.json` and in
  `src/documents.rs` (~172) together, the value the diagnosis measured sufficient; no other value
  of either changes for this.
- The previous answer's regression: a test in `temper-playground`'s `src/tests/` that a fresh world
  of story A, after `StartPlayground`, holds the project.
- Past the project (the uses, the adoption): if the next refusal is again a capacity of the same
  documents (`Again(Busy)` or a `CapacityRefusal` naming a limit), raise that one limit in both
  places to the value a diagnostic run shows sufficient, name it and its evidence in the report,
  and go on. Any other new failure past the project is a question with its evidence, as the
  previous answer's step 3 says.
- Then the previous answer's steps 4 and 5 (the read loop under `## Left` if it is the host's; the
  one-process `story_a.rs` run once alone with `--run-ignored all`) and the acceptance as written.

2026-11-03. **Adoption is refused `ForgeRefused(TooLarge)` at the small profile's
`authority.segments` 4: repair it here, or leave it to an owner?** Repair it here; it is the same
class as the previous answer's allowance, a bound of the composition's documents, though its refusal
names no limit. The code shows it: `invalid_fields`
(`temper/crates/temper-engine-domain-forge/src/adoption.rs`, ~510-529) needs 7 segments plus one per
slash of the prefix, the start asks with `temper/` (8), and the forge takes `authority.segments` from
jig's core (`temper/crates/temper-service/src/limits.rs`, ~409), 4 in the small profile. The
composition's own `ENGINE_OVERRIDES` (`src/documents.rs`, ~59) already sets it to 8; story A's
document omits it.

- `authority.segments` becomes 8 in story A's engine overrides, in
  `temper/crates/temper-playground/stories/a/deployment.json` and in `src/documents.rs` where story
  A's document is made, together; nothing else changes for it. Extend the existing project
  regression (`story_a_start_makes_its_project`) or add one beside it so a fresh world of story A,
  after `StartPlayground`, holds `playground/widget` adopted (the playground row `Ready`, or the
  repository among the forge's adoptions, whichever the world reads first).
- **Past the adoption, the allowance widens to this class:** a refusal or `Again(Busy)` whose cause
  the code shows is an engine, jig or forge limit compared against a value of these documents (file
  and line of the comparison, and the limit's name) is raised in both places to the value a
  diagnostic run shows sufficient, named in the report with its evidence, and the walk goes on. A
  refusal whose cause is not a document bound (a wrong ask, a fault in the engine, service, forge,
  host or jig) stays a question with its evidence. Commit and push each repair with its test before
  asking.
- Then the previous answers' steps (the read loop under `## Left` if it is the host's; the
  one-process `story_a.rs` once alone with `--run-ignored all`) and the acceptance as written: the
  full walk, its kept `tests/fixtures/story-a.log` recorded under `SKEIN_CODEGEN_WRITE`, and its
  replay.

2026-11-03. **Story A's deployment advertises no cues (`application.demo.llm.asked = []`): fill
them from its LLM document here, or leave it to an owner?** Fill them here. It is the same
documents as the earlier allowances, and the design says what they hold: story A offers the box's
three suggested requests (`temper/docs/design/playground.md`, 2 and 5.2), and its deployment's
public view of them is the `asked` array the service reads (`temper-service`'s
`src/startup/application/demo.rs`, ~48-62), which story A leaves empty while its LLM document holds
the three.

- `temper/crates/temper-playground/stories/a/deployment.json`: `application.demo.llm.asked` holds
  one object per cue of `stories/a/llm.json`'s `asked`, in its order, each `{"cue": ...}` alone
  with the same words: the service reads `cue` alone, and the scripts stay in the LLM document,
  their one source. `cued`, `otherwise` and every other value stay as they are.
- A regression in `temper-playground`'s `src/tests/` (beside the stories' tests): story A's
  deployment's `asked` cues equal its LLM document's `asked` cues, in order.
- **The allowance widens once more, to this class:** where a value the service or engine reads from
  story A's deployment disagrees with, or omits, what story A's own LLM or forge document holds for
  the same thing, make the deployment agree, with a regression comparing the two, name it and its
  evidence (file and line of the read) in the report, and go on. Anything else (a fault in the
  service, engine, forge, host, client, skein or jig, or a value neither document holds) stays a
  question with its evidence. Commit and push each repair with its test before asking.
- Then the previous answers' steps and the acceptance as written: the full walk, its kept
  `tests/fixtures/story-a.log` recorded under `SKEIN_CODEGEN_WRITE`, and its replay.

2026-11-03. **Story A's checkouts are refused 401 by the typed git transport's empty token
snapshot: route story A's git through the live peer's `Route`, or repair `Typed`'s authentication
elsewhere?** Route it, here; `temper-fake-git` stays as it is. A story's forge peer issues the bot's
token at its takeover, after the world is built, so a table copied at `Story::start` can never hold
it; the peer story A builds is `temper_fake::forge::peer`, whose git route is bounded by
`temper_fake::forge::git_limits()`, and the shipped `temper fake git` reaches that same peer over
`Route::new(temper_fake::forge::git_limits())` (`temper/crates/temper/src/fake/git.rs`, ~49). The
world runs the fake git program as `temper fake` runs it (`temper/docs/design/playground.md`, 3 and
5.1); the typed transport stays for the worlds whose tables are seeded before they start (the
`setup` array's entries, the one-process world's own scenarios). This task's files gain
`temper-playground`'s `src/programs.rs` and the one-process world's `src/world.rs`, for this
repair alone.

- `programs::Parameters` gains `pub route: Option<skein_fake_forge_protocol::Limits>`: the world's
  git reaches its forge peer over HTTP with these bounds. `programs::git` builds
  `Process::GitRoute` with `Route::new(..)` when `H::github()` (with `H::git_limits()`, as now) or
  when `parameters().route` is `Some(limits)` (with `limits`); otherwise `Process::Git` with the
  typed transport, as now. Nothing else of `programs.rs` changes (in particular `agent_root` keeps
  reading `H::github()` alone).
- `story.rs`: one function decides it from the deployment, `pub fn route(document:
  &serde_json::Value) -> Option<skein_fake_forge_protocol::Limits>`, `Some(temper_fake::forge::git_limits())`
  when `application.demo` is present (the playground is offered, the same rule as the handover's
  above), `None` otherwise; `world_metered` fills `Parameters::route` with it. `Fakes`, `Start` and
  `Story::start` stay as they are: `Fakes::forge`, the typed view over the peer's shared forge, is
  still what the outcome and the referee read.
- `temper/tests/one-process/src/world.rs`: `parameters` sets `route: None`, and its
  `Story::PlaygroundA` branch sets `parameters.route` from `temper_playground::story::route` of the
  story's deployment, so the one-process story A runs the composition's git as the playground does
  (`one-process-replay` replays over `temper_playground::world`, which gets it from
  `world_metered`). No other line there.
- A regression in `temper-playground`'s `src/tests/`: story A's fresh world's
  `story::parameters(..).route` is `Some(temper_fake::forge::git_limits())`, and the `setup`-array
  world of `src/tests/world.rs` keeps `None`.
- If `Route` itself fails in this world (a connection, an encoding bound, a refusal with the bot's
  credential) for a cause in `temper-fake-git`, `temper-fake-git-hosted`, skein or the peer rather
  than in the selection above, stop and ask with the evidence (the git child's error lines, file
  and line); a protocol bound of these documents falls under the earlier answers' allowance.
- Then the earlier answers' steps and the acceptance as written: the full walk, its kept
  `tests/fixtures/story-a.log` recorded under `SKEIN_CODEGEN_WRITE`, its replay; the one-process
  `story_a.rs` once alone with `--run-ignored all` (its `#[ignore]` lifted only if it passes). Add
  `heavy cargo nextest run -p temper-one-process-world` to the checks.

2026-11-03. **Four runs without landing again (runs 5 to 8): go on, or rethink the task?** Go on,
with the previous answer (story A's git over the live peer's `Route`) as the next run's work; no new
decision. As before, the runs are no loop: each reached a new stall its stop rule named, the run
committed and pushed its repairs with their tests, its answer repaired the next cause, and the next
run got further (the project, the adoption, the cues, the chat; now the first checkout). The branch's
pushed commits (to `6b9885849d`) stay and the next run starts from them. The earlier rules hold: a
failure of `Route` itself, or any new failure outside the allowances' classes, is a question with its
evidence, asked after committing and pushing what is repaired.

2026-11-03. **Story A's chats are `EndedBy::Result` (finite), so the brief's cue never reaches the
user message and the fake LLM answers the fallback: keep the stock requester-ended chat for demo
worlds in `documents::given_with`, or leave the walk blocked?** Keep the stock ending, here. Story A
is the box's playground (`temper/docs/design/playground.md`, 5.2), run on product code with only the
composition's documents differing (1, 3): the box's chats are the stock's, requester-ended, and the
`Result` override is the harness worlds' choice, not the playground's. The rule is the earlier
answers' one: decided by `application.demo`.

- `temper/crates/temper-playground/src/documents.rs`, `given_with`: when `read_wait` is false,
  override `defaults.templates[0].flags.ended_by` to `EndedBy::Result` only when the document's
  `application.demo` is null; a demo document keeps the stock template's ending. `read_wait`'s
  branch, `stock`'s and every other value stay as they are. The one-process world's playground
  stories reach it through `documents::given` and so get the same ending; its other stories, whose
  document has no demo, keep `Result`.
- A regression in `temper-playground`'s `src/tests/`: story A's deployment through
  `documents::given` keeps the stock template's `ended_by` (`Requester`), and `world_document()`'s
  stays `Result`.
- Past this, the earlier allowances' classes hold as written; any other new failure (in the service,
  engine, smith's run, the fake LLM, the host, skein or jig) is a question with its evidence, asked
  after committing and pushing what is repaired.
- The wasm32 clippy expectations the run listed (`skein-llm-connection`, `skein-forge-setup`,
  `temper-engine-domain-forge`) are not this task's: leave them under `## Left`.
- Then the acceptance as written: the full walk, its kept `tests/fixtures/story-a.log` recorded under
  `SKEIN_CODEGEN_WRITE`, its replay, and the one-process `story_a.rs` once alone with
  `--run-ignored all` (its `#[ignore]` lifted only if it passes).

2026-11-03. **The delegate is refused for authority (spend 21,140 over stock's 768, tasks 4 over
`tasks.tree_tasks` 1): align story A's model prices with the box template, or have the script's
owner revise its request?** Align the prices, here; the script stays as it is. Story A is the box's
playground with one source for its scripts, `temper-init`'s template (`temper/docs/design/playground.md`,
5.2), and only the composition's documents differ (1, 3). The template's 21,140 is funded by the
stock budgets `temper-service`'s `stock::checked` derives from the template's own prices
(`unit` 1000; its comment's C = 1057, K = 4228); story A's deployment prices its model at `unit`
1000000, which shrinks K to 32 and the chat to 768. The script is right; the deployment's prices
are the value that disagrees with the box. This is no test budget or timeout of the user's
(`AGENTS.md`, 7): it is a value of the composition's documents, and no authority check is relaxed.

- `temper/crates/temper-playground/stories/a/deployment.json`: the `fake` model's `prices` become
  the template's `demo` model's (`input`, `cached`, `output` 1, `unit` 1000). Its `window`, its
  `output` and every other value stay as they are.
- `tasks.tree_tasks` becomes 4 in story A's engine overrides, in `stories/a/deployment.json` and in
  `src/documents.rs` where story A's document is made, together: the earlier document-bound
  allowance's class (the composition's `ENGINE_OVERRIDES` already holds 4; the script asks
  `delegation.tasks` 4).
- A regression in `temper-playground`'s `src/tests/`: from story A's deployment's `llm`,
  `temper_service::stock::defaults` gives a chat role whose `budget.spend` and the policy's and the
  rules' ceilings hold the script's delegate `budget.spend` (read from `stories/a/llm.json`, not
  written as a number), and the deployment's `tasks.tree_tasks` is at least its
  `delegation.tasks`.
- If the authority refusal remains after both (another `authority ceiling` part: a depth, a kind, a
  pattern), it falls under the earlier allowance only where a limit of these documents is compared
  (file and line, the limit's name); a ceiling of stock's own construction, or anything else, is a
  question with the refusal and its evidence, asked after committing and pushing what is repaired.
- Then the acceptance as written: the full walk, its kept `tests/fixtures/story-a.log` recorded under
  `SKEIN_CODEGEN_WRITE`, its replay, and the one-process `story_a.rs` once alone with
  `--run-ignored all` (its `#[ignore]` lifted only if it passes).

2026-11-03. **The producer is refused as outside its delegation (the composition's forge stock
selects charter 1, the script grants charters 2 and 4): keep the product's forge stock for demo
worlds, or set the harness's `forge_charter` to 2?** Keep the product's forge stock, here; the script
stays as it is. This is the same rule as the chat ending's answer: story A is the box's playground on
product code, only the composition's documents differ (`temper/docs/design/playground.md`, 1, 3 and
5.2), and the box's forge defaults are `temper_service::stock::forge`'s (change charter 2, review
charter 4, the deployment's landing rule; `protocol/configuration.md`, 13.4). The harness's
`world_settings` forge defaults, charter 1 and its own landing rules, are the harness worlds'
choice. A `forge_charter = 2` alone would keep a landing configuration the box never has.

- `temper/crates/temper-playground/src/documents.rs`, `given_with`: when the document's
  `application.demo` is present, `Given::Own`'s `forge` is `temper_service::stock::forge(&limits)`;
  otherwise `settings::forge(&document)` as now. `limits` are the engine limits the service would
  derive (`temper_service::limits::engine` over the parsed engine's jig limits and the document's
  application limits, or, if those take no public path from here, the small profile's,
  `temper_service::limits::profile(Profile::Small)`, which is story A's profile); `stock::forge`
  reads none of them today. `world_settings`' injection, the jig defaults and every other value stay
  as they are; `documents/settings.rs` stays as it is. The one-process world's playground stories
  reach this through `documents::given_with` and get the same; its other stories, with no demo, keep
  the harness's.
- A regression in `temper-playground`'s `src/tests/`, beside the chat ending's: story A's deployment
  through `documents::given` gives the forge stock equal to `temper_service::stock::forge`'s (its
  kind 3 items charters 2 and 4), and `world_document()`'s keeps `settings::forge`'s (charter 1).
- If the stock's landing rule then holds the CSV change (a gate, CI or up-to-date requirement story
  A's fake forge does not meet), that is no document bound of these answers' allowances: a question
  with the change's gates and the rule, asked after committing and pushing what is repaired. Any
  other new failure past the producer's start is a question likewise, unless it falls in an earlier
  allowance's class as written.
- Then the acceptance as written: the full walk, its kept `tests/fixtures/story-a.log` recorded under
  `SKEIN_CODEGEN_WRITE`, its replay, and the one-process `story_a.rs` once alone with
  `--run-ignored all` (its `#[ignore]` lifted only if it passes).

2026-11-03. **The CSV reply comes at 32.4 s of world time, past the person's 30 s wait, and the third
request then shows "Authority missing": authorize a longer reply wait and closing each answered chat,
or leave the walk blocked?** The wait yes, here; closing chats no, diagnosis first.

- **The wait is the script's, not the user's.** `WAIT` in `temper/tests/playground/src/stories.rs` is
  the person's observation window in the world's time, which the story's own pace sets (the fake LLM's
  and CI's latencies, `temper/docs/design/playground.md`, 5.1); `AGENTS.md` 7's timeouts are the
  test's budget and its wall limit (`skein/docs/foundation/testing-strategy.md`, 8.1), which stay as
  they are. `WAIT` becomes 60 s, room for the measured 31.2 s without fitting it to the instant; the
  driver's whole-walk bound, also world time and the driver's own stall guard, may rise to what the
  completed walk measures with the same margin, its turns-at-one-instant bound unchanged. The
  one-process script keeps its 30 s (its story A is ignored for its own stall); list it under
  `## Left`. The walk's wall time stays within the world's budget (`temper/docs/design/domain/testing.md`,
  6.1), measured and reported.
- **No chat-closing steps.** Story A is the box's three suggested requests answered
  (`temper/docs/design/playground.md`, 5.2 and 10); the box's person presses them in turn and the
  one-process script closes nothing. A script that closes chats to make room would hide a refusal the
  box's person meets. The script keeps the one-process steps.
- **Diagnose the third request's refusal,** with throwaway diagnostics only: capture the typed refusal
  behind "Authority missing" whole (the jig reply and what it compared: a period's spend, an
  authority's spend, a request count, a model's account), file and line of the comparison. Then:
  - a limit of story A's documents compared against (the earlier allowances' classes): raise it in
    both places as they say, with its regression, and go on;
  - the stock's own construction (policy 1's period spend of 48K holding two open chats' 24K each,
    `temper/crates/temper-service/src/stock.rs`, ~174-195, `temper/docs/design/protocol/configuration.md`,
    13.4), or anything else outside those classes: stop and ask with the refusal and its evidence; the
    stock is the product's, and the box would refuse the same.
- Commit and push the wait with the walk's progress before asking; then the acceptance as written:
  the full walk, its kept `tests/fixtures/story-a.log` recorded under `SKEIN_CODEGEN_WRITE`, its
  replay, and the one-process `story_a.rs` once alone with `--run-ignored all`.

2026-11-03. **Four runs without landing again (runs 9 to 12): go on, or rethink the task?** Go on,
with the previous answer (the reply wait at 60 s, no chat-closing steps, the third request's
"Authority missing" diagnosed) as the next run's work; no new decision. The runs are still no loop:
each reached a new stall its stop rule named and pushed its repairs with their tests (the chat
ending, the prices and tasks, the forge stock, the task, inbox, parameter, spec and choice bounds),
and the next got further: run 12's CSV change landed with CI and review passed, and its dark-theme
reply was seen. The branch's pushed commits (to `9678bee8e1`) stay and the next run starts from
them. The earlier rules hold: a refusal outside the allowances' classes, the stock's own
construction among them, is a question with the typed refusal and its evidence, asked after
committing and pushing what is repaired.

2026-11-04. **The third suggested request is refused `Budget { needs: 1, left: 0 }`, the stock's 48K
period reserved whole by the first two open chats' 24K: repair the product's stock, or leave the walk
blocked?** Repair it, here, as the user decided (11-04, option (a)): a chat's authority spend becomes
16K and the week's 48K for the project and for role 0 stays, so three open chats fit in one period and
the week's money bound is unchanged. The script stays as it is (no chat-closing steps). This task's
files gain the five below, for this repair alone; its acceptance (the other two requests answered,
`temper/docs/design/playground.md`, 5.2 and 10) cannot hold without it.

- `temper/crates/temper-service/src/stock.rs`, `checked`: `chat_spend` is `run.checked_mul(16)?`;
  `ceiling_spend` stays `run.checked_mul(48)?`. Nothing else there changes.
- `temper/crates/temper-service/src/tests/stock.rs`:
  `stock_budgets_follow_the_default_models_completion_and_run_turn_floor` expects `16 * k` for role
  0's authority spend; and a new regression beside it, `the_stock_funds_three_chats_in_one_period`:
  for both of that test's rows, three times policy 1's role 0 authority spend is at most the
  policy's, role 0's and the rules' period spends, and a fourth is not.
- `temper/tests/engine/domain/tests/suite/settings/b2.rs` (~237): the stock chat's task budget is
  `16 * K`.
- `temper/docs/design/protocol/configuration.md`, 13.4: policy 1's row says an authority's spend of
  16K; the sizing paragraph's "a chat 24K, its run, an advisor and two coordinators; the ceilings and
  the periods 48K, twice a chat" becomes "a chat 16K, its run, an advisor and a coordinator; the
  ceilings and the periods 48K, three chats open in one period". The rules' row keeps its 48K. Run
  `tools/citations/citations.py` before pushing.
- `temper/docs/help/budget.md` (~31): "a person's chat may spend 16K"; the help is compiled into
  `temper-docs`' tests.
- Story A's own regression (the delegate's 21,140 within the chat role's spend, read from
  `stories/a/llm.json`) must still pass: at init prices 16K is 67,648.
- Checks added: `heavy cargo nextest run -p temper-service --lib -E 'test(/stock/)'`,
  `heavy cargo nextest run -p temper-engine-domain-world -E 'test(/settings/)'`,
  `heavy cargo nextest run -p temper-docs`, and `heavy cargo nextest run -p temper-playground`.
  Any other test that held the old 24K is reported under `## Left` with its file and line, not
  chased.
- Then the acceptance as written: the full walk, its kept `tests/fixtures/story-a.log` recorded under
  `SKEIN_CODEGEN_WRITE`, its replay, and the one-process `story_a.rs` once alone with
  `--run-ignored all` (its `#[ignore]` lifted only if it passes). Past this, a refusal outside the
  earlier allowances' classes is a question with its evidence, asked after committing and pushing
  what is repaired.

2026-11-04. **The completed walk costs 2.197 s CPU (median of three, `-j 1`) against the 2 s guide,
and `perf` is missing on the builder: make a profiler available there, or give the performance work
to another task?** Neither. This task finds the cause, without a profiler and without touching the
builder. A tool on a builder is a machine change, and that is the user's. The world budget stays in
this task's acceptance: the behaviour is done and the cost belongs to the same walk. The pushed commits
(to `3133264738`) stay, and the next run starts from them. No budget, guide, pace, profile or
timeout moves.

- **Split the cost with throwaway timing,** untracked and gone before the push.
  `std::time::Instant` is allowed in that code. The run's own numbers already point at the world:
  the replay drives the same world with no page, host or person and takes 1.876 s, so the page side
  costs about 0.3 s and the world about 1.9 s. Measure, over the walk's 12,972 world steps:
  - the time and call count in each process's turns (service, worker, fake LLM, fake forge, CI,
    visitor);
  - the time in the session log's work (each input's record and any digest it takes);
  - the time in the page's side (the host, `adapter::requests`, `transport::carry`, the view and its
    draws, the person's `next` over the tree).

  Report each one's total and its share. Look first for a cost that grows with the walk:
  a digest, a clone, a serialisation or a tree walk over the whole state on every step or turn.
  That is testing-strategy.md 8.1's question 2 (an O(n) where O(1) would do, a missing cache).
- **The cause in this task's files** (`temper/tests/playground/**`, and `story.rs`, `stories.rs`,
  `programs.rs` and `documents*` in `temper-playground`): fix that cause, keep the behaviour, the
  kept log and the replay as they are (a changed log is re-recorded under `SKEIN_CODEGEN_WRITE` and
  the reason stated), and re-measure the same way: median of three, alone, `-j 1`.
- **The cause outside them** (skein's simulator or session log, `temper-service`,
  `temper-client-view`, `temper-client-shell-web`, jig): stop and ask with the split, the file and
  function, and a proposed fix. skein stays untouched by this plan (`notes/common.md`), and fixing
  a lower tier's cost is that tier's work (testing-strategy.md, 9).
- **No single cause** (the cost spread evenly across the steps' ordinary work): stop and ask with
  the split and the three measurements. That is 8.1's question 3, which goes to the user through
  the director.

2026-11-04. **The unchanged walk costs 2.208 s median CPU against the 2 s guide, with no single
cause: deeper timing and a lower-tier optimization, or a different guide?** A different guide, as
the user decided (11-04, option (a)): story A walked through the web client's host is a story of
the kind 6.1 gives 3 s, running three chats, a delegate with its agents, git, CI and a landing. The
guide's line is in `temper/docs/design/domain/testing.md`, 6.1 ("a playground story walked natively
through the web client's host ... at most 3 s"), added by this answer; this task does not edit it.
The walk lands as it is: its 2.208 s is within the guide, and the acceptance's world budget holds.

- No deeper timing and no optimization: the measured lower-tier costs (the simulator iterating idle
  processes at every one of the 12,972 steps, an idle fake LLM turn at ~17 µs; `temper-service`'s
  readiness, 14,360 calls at ~14 µs) go to the backlog as a performance item, not this plan's.
- The next run merges the trunk if its branch lacks the guide's line, runs the checks the answers
  list once (the package's tests, clippy, `temper-playground`'s, the one-process world's), reports
  the walk's CPU against 3 s, and pushes. No budget, pace, script, fixture or timeout moves.

2026-11-04. **May this task migrate its short startup fixture, `story-a-start.log`, to log version
2, or must `temper-playground`'s startup regression stay blocked?** Migrate it. The fixture is
under `temper/crates/temper-playground/src/tests/**`, one of this task's files, and the checks the
answers list include `temper-playground`'s tests, which cannot pass while it is version 1. The
previous answer's "no fixture moves" kept the walk's own kept log (`tests/fixtures/story-a.log`)
from being rewritten to hide a cost or a difference. It did not freeze a supporting fixture in a
format skein's reader no longer reads (`skein/crates/skein-ui-web/src/log/reader.rs`, version 2
since `ed8b1f0760` gave exchanges `cache=` and `keep=`).

- Make it from the kept full log, with no new recording: the full log's header and its first
  twelve inputs, which are the old fixture's steps (147 to 376, up to Try the playground's
  `exchange id=6` at step 376), then the line `cut`, as the old fixture ends. The full log's
  `said` digests are the composition's as it stands; the old fixture's step-226 digest is stale.
- Check it with `story::tests::story_a_start_makes_its_project_and_adopts_widget` alone, then
  `temper-playground`'s package run. The full walk's fixture stays byte for byte as it is.
- If the prefix replay reports `Differs`, report it whole and stop and ask: the walk and this
  regression replay one world.
- The rest is as the previous answer says: report the walk's CPU against 3 s and push. What the
  report lists under `## Left` stays left.
