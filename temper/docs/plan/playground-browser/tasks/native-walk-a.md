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
