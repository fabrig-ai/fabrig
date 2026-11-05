+++
id = "fetched-story"
part = "client"
kind = "feature"
needs = ["link-fetched", "view-fetched"]
files = ["temper/tests/client/tests/suite/forges_edges.rs", "temper/tests/client/tests/suite/welcome_m3_owned.rs"]
effort = "high"
minutes = 35
+++

# The client's world says how far the fetch has gone

## Goal

The story that runs both real sides: an engine event, through the real `temper_service::family` and wire, to the client's domain and view. It is the first place the row's progress crosses that boundary whole.

Acceptance:

- in the Forges page and the welcome, the row's words with a size, then a later figure, then the runner, and with no size, all driven by `forge::Event::Fetched`;
- the welcome's progress node has its name, text and fraction, and "Each step"'s first step its figure;
- the next step's `Connection` takes the figure away, on the row and the node;
- each stands for both shells' words through the view: the terminal's print is `temper-client-view`'s test, from `view-fetched`.

## Inputs

- `temper/docs/design/draft/fetch-progress.md`, section 7, the client's worlds.
- `temper/docs/design/client/forge.md`, 5.4.
- `temper/docs/plan/fetch-progress-1/notes/common.md`.

## Owns and may change

Owns the two files in the header. A helper beside `connection` in `forges_edges.rs`; a fixture in `temper/tests/client/src/` only if a test cannot be written without it.

Pre-approved: `look-round-3/rail-readers` owns `temper/tests/client/**` for its own lines, and `welcome_m3_owned.rs` sits beside its changes. Add tests at the end of a file or a block, and touch no line it may touch.

## Steps

1. `forges_edges.rs`: beside `connection`:

   ```rust
   fn fetched(world: &mut Pages, number: u16, fetched: forge::SetUpFetched) {
       world.front().engine.send(engine::Event::Forge(Box::new(forge::Event::Fetched { forge: number, fetched })));
   }
   ```

   `a_forge_row_says_how_far_its_fetch_has_gone`: by `forges_rows_are_folded_and_keep_state_account_and_actions`'s way, a table of the row's words.
2. `welcome_m3_owned.rs`: `owned_setup_says_how_far_the_fetch_has_gone`, by `owned_setup_progress_names_both_pins_on_welcome_and_standalone`'s way: send `Fetched`, settle, read the node from `Temper::tree(&draw(&root))` and the words from `text(&root)`; then `Connection` at `Prepare`, settle, and the figure and the `Role::Progress` fraction are gone.
3. `cargo fmt`; `heavy cargo clippy -p temper-client-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world --test suite -E 'test(forges_edges) or test(welcome_m3_owned)'`, and check the count. Commit and push.

## Report

As the implementer's text has it. A failure here that names a lower layer is that layer's: fix it in its own package, in the lowest tier that shows it (`AGENTS.md`, 5), and say so.
