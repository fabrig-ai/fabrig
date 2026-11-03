+++
id = "native-walk-a"
part = "native-walk"
kind = "feature"
needs = ["playground-visitor"]
files = [
  "temper/tests/playground/**",
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
