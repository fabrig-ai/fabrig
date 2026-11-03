+++
id = "playground-visitor"
part = "visitor"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-playground/**",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# skein's visitor joins the playground's composition, which runs in skein's slices

## Goal

The composition hosts skein's visitor (`temper/docs/design/playground.md`, 3 and 4): a program
of the world that speaks HTTP to the clients' listener with the deployment's address as its Host,
as a person's browser does. skein's loops (its slices and its replay) step the composition with
the link printed to the world's printer and the factory parameters installed, so the browser's
worker and a native test step one world. This is the surface the native walk, the log's replay
and the worker's entry build on (`notes/contracts.md`, C1 and C2).

Acceptance:

- `temper-playground` gives C1 and C2 exactly: `Process::Visitor`, `Role::Visitor`,
  `documents::{VISITOR_EXCHANGES, visitor}`, `story::{VISITOR, reach, parameters, slices,
  SLICE_STEPS}`, `stories::named`, `ServiceHost::print_to`; `story::capture` and the service's
  standard-output pipe are gone.
- New tests in `temper-playground` (`src/tests/visitor.rs`, beside the crate's others), each run
  inside `programs::with(story::parameters(..), ..)`, browser time moved to each `Wake::At`:
  - `story_a_posts_its_printed_link`: `story::slices(stories::A.start(SEED, Wall::EPOCH),
    SLICE_STEPS, Time::ZERO)` posts, within a bounded number of slices, one
    `bridge::ToPage::Printed` from `temper serve` on standard output whose line is
    `http://127.0.0.1:7102/jig/v1/sign-in/link?secret=` and a non-empty secret, and then waits
    idle (`Wake::Message`).
  - `the_visitor_spends_the_printed_link`: the link's path and query entered as one
    `bridge::Input::Exchange` (GET, no body, a bound of `temper_client_shell_web`'s
    `BODY_BYTES` written as a literal with a comment naming it, since this crate does not depend
    on the shell) is answered, not failed, with a status below 400, within a bounded number of
    slices.
  - `named_finds_each_story`: `stories::named(b"a")` is `A`; an unknown name is `None`.
- `heavy cargo nextest run -p temper-playground -p temper-playground-world` passes; the
  one-process world builds (`heavy cargo check -p temper-one-process-world --tests`) and its
  focused tests pass with the trunk's count.
- `heavy cargo clippy -p temper-playground -p temper-playground-world --all-targets -- -D
  warnings` passes, and `heavy cargo clippy --target wasm32-unknown-unknown -p temper-playground
  -- -D warnings` passes if it passed at your base (run it there first; if it failed at your base,
  report its first error and do not chase it: plan `wasm32-temper` measures it).

## Inputs

- `temper/docs/plan/playground-browser/notes/common.md` and
  `temper/docs/plan/playground-browser/notes/contracts.md`, C1 and C2.
- `temper/docs/design/playground.md`, 3 (the composition) and 4 (the world: one seed, the printed
  link, the clients' listener and the Host).
- `skein/docs/design/playground.md`, 3 (the slices), 4 (the visitor) and 5 (the bridge);
  `skein/docs/design/world.md`, 2 (what processes print).
- skein's own native composition of a visitor, to copy its shape:
  `skein/tests/playground/src/native.rs` (`world`, `enter`, `take`) and
  `skein/tests/playground/src/site.rs` (`config`); skein's API in
  `skein/crates/skein-playground/src/{visitor.rs,slices.rs}` and
  `skein/crates/skein-ui-web/src/bridge.rs`.

## Owns and may change

- `temper/crates/temper-playground/**`: `Cargo.toml` gains `skein-playground.workspace = true`
  and `skein-ui-web.workspace = true` in `[dependencies]` (both product crates, already workspace
  dependencies in the root `Cargo.toml`); `src/{programs.rs,story.rs,service.rs,documents.rs,
  stories.rs}` and `src/tests/**`.
- `Cargo.lock`: the package's dependency lines only.
- Reader files, call sites and compiler-forced arms only:
  - `temper/tests/playground/tests/suite/world.rs`: the process-kind assertion in
    `story_a_starts_with_no_box_calm_and_prints_its_link` gains `Process::Visitor(_)` and its
    message says "and the visitor".
  - `temper/tests/playground/tests/memory.rs`: `name` gains `Process::Visitor(_) => "visitor"`.
  - `temper/tests/one-process/**`: only arms or call sites the compiler forces (`Role::Visitor`,
    `Process::Visitor`, `capture_output` replaced by `print_to`, `story::capture` removed);
    behaviour kept.
- Pre-approved options:
  - **The visitor's bounds** (`documents::visitor`): `exchanges` `VISITOR_EXCHANGES`; `cookies`
    and `cookie_bytes` just big enough for jig's session cookie and any other the sign-in sets
    (find them in jig's `protocol/browser.md`, 11, and the code that sets them), with a comment
    naming them; `io::Limits` and `http::Limits` big enough for story A's largest request and
    answer piece within the client's own bounds (`temper-client-shell-web`'s `limits.rs`: `PIECE`
    for the stream), starting from skein's `site::config` and the service's clients listener
    limits. If a bound you need would break the visitor's asserts (`http::largest_read` within
    io's), raise io's, and say which in the report.
  - **The Host** is the document's `application.clients.listener.socket` text, as the printed
    link's authority shows it; if the document names a `public` address, that one's authority
    instead (story A names none).
  - If a test of the one-process world counted the service's standard-output pipe (an fd or an
    operation count), keep the test's meaning and update its figure, saying so in the report;
    if its meaning would change, stop and ask.
  - If `World::spawn` admits the visitor at another index than 3, fix `VISITOR` to it and
    keep the assertion.

## Steps

1. C2's printer first: `ServiceHost::print_to(printer)`; `ServiceState`'s `Links::Print` writes
   to it (a full printer, `WouldBlock`, shuts the service down as a failed pipe does today);
   `world_metered` hands it the printer and opens no standard-output pipe; `story::capture` goes;
   `story::step` installs the parameters around `World::step` only. Run the playground world's
   tests (`temper-playground-world`): the link still prints.
2. C1: the variant, the role, `documents::visitor`, the visitor admitted in `world_metered`,
   `VISITOR` and `reach`.
3. C2's `parameters`, `slices`, `SLICE_STEPS`; `stories::named`.
4. The three tests; the readers' arms.
5. `cargo fmt`, the old-form counts, the clippy and test commands above; commit per step; push.

## Report

As the implementer's text says, and also: the visitor's bounds you chose and why; the number of
slices and steps `story_a_posts_its_printed_link` took; whether wasm32 clippy of
`temper-playground` passed at your base and at your head.
