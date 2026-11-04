+++
id = "client-fetched"
part = "client"
kind = "feature"
needs = ["wire-fetched"]
files = ["temper/crates/temper-client-domain/src/forge/forges.rs", "temper/crates/temper-client-domain/src/forge/family.rs", "temper/crates/temper-client-domain/src/forge/words.rs", "temper/crates/temper-client-domain/src/tests/forges.rs"]
effort = "high"
minutes = 35
+++

# The client's row says how far the fetch has gone

## Goal

The client's domain reads the wire's `fetched` onto its row and says it: "Setting up: fetching Forgejo · 42%". Every shell and page that calls `forge_connection` gets the words with no change.

Acceptance:

- `Binary`, `Fetched`, `Row.fetched`, `fetched_figure`, `fetching_name` and `forge_connection_bare`, as `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 6, has them;
- `forge_connection` says, at the fetch step with a line heard: "Setting up: fetching Forgejo · 42%", "Setting up: fetching its runner · 80%", "Setting up: fetching Forgejo · 41 MB";
- with no line heard, and at every other connection, it says what it says today;
- a percentage rounds down, and `u64::MAX` of `u64::MAX` is 100%;
- a wire `fetched` with a zero total or done past total, or at a step other than fetch, reads as none;
- the status line row of `welcome-fetch-percentage`'s account is finished, and each commit carries its trailer.

## Inputs

- `temper/docs/design/client/forge.md`, 5.4, the table's "setting up" row.
- `temper/docs/design/client/README.md`, 4.6, `ForgeRow`.
- `temper/docs/plan/fetch-progress-1/notes/contracts.md`, section 6, and `notes/common.md`.

## Owns and may change

Owns the files in the header.

A departure from the implementer's text, on commits: each commit carries `Look-request: welcome-fetch-percentage` directly above `Co-Authored-By:` (or last, with none): this finishes a deferred row of that look request (`temper/docs/design/draft/look-requests.md`, 3).

Pre-approved:

- `Row`'s literals in `temper-client-domain/src/tests/connect.rs`, `forges.rs`, and wherever the compiler says, take `fetched: None`.
- `forge_connection`'s literal bytes are how this function says all its words, so the new ones are literals too. No `Word`, no phrase count moves; a new `Word` would meet `look-round-3` at `words.rs`'s counts.
- `temper-client-tools`'s `forge list` prints a state and a step from `Connection`, no words: it stays.
- `repository-copies-1/making-facts` is changing `forge/family.rs` and `forge/forges.rs`: keep your hunks to `Row` and `connection_from`'s neighbour.

## Steps

1. `forge/forges.rs`: the two types, `Row.fetched`.
2. `forge/family.rs`: a `fetched_from(&w::Connection) -> Option<Fetched>` beside `connection_from`, its checks those above: the view gives its numbers to `Builder::fraction`, which asserts, so a bad row must stop here.
3. `forge/words.rs`:

   ```rust
   C::SettingUp { step } => match (step, row.fetched) {
       (Some(Step::Fetch), Some(fetched)) => join_words(&[&forge_connection_bare(row), b" \xc2\xb7 ", &fetched_figure(fetched)]),
       ...
   ```

   `fetched_figure`: `done * 100 / total` in `u128`, rounded down, then `%`; with no size, `done / 1_000_000`, then ` MB`.
4. Tests in `tests/forges.rs`: `the_row_says_how_far_the_fetch_has_gone` (a table of the three words, rounding down, a figure of 0 and of 100), `a_fetched_at_another_step_or_out_of_range_reads_as_none`, `no_line_heard_says_the_step_alone`.
5. `cargo fmt`; the old-forms check; `heavy cargo clippy -p temper-client-domain --all-targets -- -D warnings`. This changes what a shared function says, so run its readers: `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools`. Commit and push.

## Report

As the implementer's text has it, with each rule broken for a new test and the failure seen.
