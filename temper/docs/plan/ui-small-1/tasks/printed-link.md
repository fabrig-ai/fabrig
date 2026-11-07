+++
id = "printed-link"
part = "printed-link"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/process.rs",
  "temper/crates/temper/tests/support/mod.rs",
  "temper/crates/temper/tests/suite/support_tests.rs",
]
effort = "high"
minutes = 50
+++

# A second printed link says why it came

## Goal

A process story failed because a box's stdout held its sign-in link twice, and its panic said nothing of why, while writing the link into the log as decimals. Read what else can print a second link and fix a fault you find at the lowest tier; make the assertion count links and carry the box's stderr, its `secret=` lines filtered.

Acceptance:

- `temper/crates/temper/tests/process.rs`, near lines 577 and 704: neither assertion compares stdout's bytes. Each counts links through one helper.
- A second link, or none, panics with the run's name, the count and the box's stderr with every line holding `secret=` dropped. The panic holds no byte of a link's secret, in any form.
- A stdout holding a line that isn't a link, or a last line with no line feed, still fails, as the byte comparison failed it.
- `temper/crates/temper/tests/suite/support_tests.rs` holds one cheap test of the helper, with rows for one link, two links and a stderr holding `secret=`.
- The process binary compiles: `heavy cargo nextest run -p temper --test process --no-run`.
- The report says what can print a second link, read from the code, which you found to be a fault if any, and what you fixed, at which tier.

The count stays one. A second link stays red: don't widen the assertion to tolerate it.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.6, *One printed link, counted*.
- `temper/docs/design/protocol/setup.md`, 7.2, the links `serve` prints and the box relays.
- `jig/docs/design/protocol/client.md`, 8, *The printed link*: when the component draws a fresh link.
- `temper/docs/plan/ui-small-1/notes/common.md`.

## What the cut read, not proved

Start here; none of it is established:

- The story's assertions on every `ctl` call's exit come first (`process.rs`, 541-549), so in the failing run every call exited 0. A refused `temper ctl sign-in` exits non-zero (`temper-client-shell-cli/src/host.rs`, near line 343), so an ordinary refused sign-in isn't it.
- `Component::fresh_link` (`jig/crates/jig-protocol-engine-clients/src/component.rs`, near line 139) has two callers: the component's start and a `Route::Link` sign-in whose secret isn't live. `temper-service/src/clients.rs`'s `print_link` passes each on.
- **Serve restarted mid-story:** the box restarts what exits, and a new serve draws a fresh link (`temper-box/src/tests/host.rs`, `a_restarted_serve_has_no_stdin_and_its_link_is_fresh`). `ctl` keeps its credential across that, so its calls could still exit 0. That would be the real fault: find what could end serve in this story. The box's terminal says `Restarting`, which the new panic would show.
- **A crossing retry:** jig's client sends a link's sign-in again after `Signing::Lost` (`jig/crates/jig-client/src/client.rs`, `SendAgain::Link`). An earlier attempt whose reply was lost, whose bytes reach the engine after a later attempt has spent the link, would be refused with nobody listening, draw a fresh link, and leave every call at exit 0.
- `temper-box/src/host.rs`, near line 1061, relays each `Output::Line` of a plain step to the terminal, and `Output::Legacy` to stdout: check nothing but serve's links reaches stdout that way.

## Owns and may change

Owns the files in the header. Other assertions on a printed link, by the same reading: run `rg` over `temper/crates/temper/tests` for stdout compared with a link, and change any that writes a real box's link into a log. Stories on a scripted or fake secret, and `assert_box_streams`' `alone` check, stay.

May also change, for a fault found and only then: `temper/crates/temper-box/src/**`, `temper/crates/temper-service/src/clients*`, `temper/crates/temper-client-shell-cli/src/**`, and their tests, with a row in an existing table where one holds it.

Departs from the implementer's text, in two ways:

- `temper/docs/design/domain/testing.status.md` is open, to drop the **Not yet** entry on 2.6.
- **The one slow-tier run of this plan.** The request's limits let this one story run by name, alone, a few times, as a red's repair: `a_csv_change_passes_ci_and_a_blocking_agent_review_on_the_owned_forge_then_lands_and_survives_restart`, under the profile its tier's configuration gives it. It is for reproducing or proving a fault, costs about 50 CPU seconds a run, and is not part of the acceptance. Run nothing else of that tier.

Options, each pre-approved:

- If you prove no fault, change no production code. Say what you read and what the next red's panic will show. That meets the acceptance.
- If the fault is in the engine's link code, `jig/crates/jig-protocol-engine-clients/src/component.rs`, its `Route::Link` arm belongs to the task `link-why`. Make the smallest change there, with its test in `jig/tests/engine-clients/tests/suite/sign_in.rs`, and name it in the report. Any merge conflict comes back to you.
- If a crossing retry is by design (`client.md`, 8: a spent secret draws a fresh link), leave the design as it is and say so under `## Left`, with what the story could do about it. Don't change the story's count.
- If a catch-up finds `process.rs` or `support/mod.rs` changed by `limits-heavy` or `model-catalogue-2`, keep their lines and edit only yours.

## Steps

1. Read what's above, and report what you find even if you fix nothing.
2. In `temper/crates/temper/tests/support/mod.rs`, beside `printed_link`:

   ```rust
   /// A box's stdout holds its links alone, one to a line (domain/testing.md, 2.6).
   pub fn printed_links(stdout: &[u8]) -> usize
   /// `stderr` without its lines holding `secret=`.
   pub fn without_secrets(stderr: &[u8]) -> String
   /// Panics with `run`, the count and `without_secrets(stderr)`, never a link's bytes.
   pub fn assert_one_printed_link(run: &str, stdout: &[u8], stderr: &[u8])
   ```

3. `process.rs`: both sites call `support::assert_one_printed_link`. The `let link` lines above them go if nothing else reads them.
4. The test, by the file's own style: `support_tests.rs` holds thin named tests, so follow its form.
5. Narrow test: `heavy cargo nextest run -p temper --test suite -E 'test(printed_link)'`.
6. The closing checks of your text, with `heavy cargo nextest run -p temper --test process --no-run` for the tier's compile. Commit and push.

## Report

The implementer's report, with the reading above: each path you checked, what it can and can't do, and whether you ran the story, how many times and what it showed.
