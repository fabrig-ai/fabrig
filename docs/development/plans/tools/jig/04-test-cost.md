# Session 04: jig's test-cost pass, and the focused budget back to 15 s

Find the shared cost in jig's focused tests, fix it where it is, then
restore 15 s by reverting ruling 275's timeout change. The profile names
three repeated computations: page bounds, vocabulary capacities and the
client's output bound. First-load phrase work remains a bounded question.
Start alongside the repin, needing nothing. The restore needs **jig on
skein's and smith's main**, including test configurations. It does not
wait for named send or turn read; 15 s binds those sessions as they land.

## Read first

- The downstream repository's tools plan: `findings/jig-test-profile.md`
  1–4; `findings/jig-test-cost.md` 1–2 for the older measurements only;
  README's rules and `code-organization.md`'s limits and fixture owners.
- The downstream repository's client-api plan: `decisions.md` ruling
  275 and "For the user" item A; `later.md`'s focused-suite debt.
  The user's 10-19 ruling in this task supersedes the optimization
  proposal and the old 12 s closing aim.
- jig's `docs/development/workflow.md` 2 and
  `docs/development/tech-debt.md` 4; `.config/nextest.toml`'s default and
  measure profiles; `scripts/check.sh`'s focused nextest invocation.
- jig's `docs/design/client/domain/view.md` 7, 10 and 18.9;
  `docs/design/client/domain/client.md` 3, 11 and 14;
  `docs/design/client/domain/testing.md` 5–6.
- skein's `docs/foundation/testing-strategy.md` 6, 8 and 10, especially
  "A slow test is understood, never hidden", at `b4c6241`;
  `docs/foundation/programming-model.md` 3, 6.3 and 10;
  `docs/foundation/development.md` 5.3, 5.5–5.7 and 11.

## What the design asks

- **testing-strategy.md 8:** the focused suite fits 15 s; each test has
  its own process. Profile a breach and fix its cause. Test builds,
  dependencies included, stay unoptimized.
- **view.md 10:** page and section bounds cover complete content and
  the arrangement's sum. Reusing a computed bound must preserve its
  value, checked arithmetic and refusal on overflow.
- **view.md 7:** applications supply languages beside jig's vocabulary.
  Its capacity covers every loaded language, including an unselected one;
  a partial book must still refuse a complete jig page.
- **client.md 3, 11, 14:** the client settles its bounded composition,
  keeps the store barrier and prices its containers. An unchanged bound
  need not be derived and validated at every drain.

## Choices this plan makes

- **Three fixes, in expected-gain order.** Page sizing: 2.1 s attributed,
  1–1.5 s expected saving; vocabulary scans: 1.4 s attributed, 0.8–1.2 s
  expected after retaining one scan per book; drain validation: about
  1.0 s of `max_out`'s 1.6 s, 0.7–0.9 s expected. These overlapping ranges
  rank work, not measured reductions; their combined 2.5–3.6 s is no
  promise of wall-time headroom. Each commit names its cause and measured
  before/after sum. This source review supersedes findings 3C's cache
  exclusion and section 4's two-fix schedule, not their measurements.
- **Start at jig's base.** `f50c3f63` supplied samples and serial times;
  `b2d920e4` differs only in STATUS.md. The repin targets skein `b4c6241`
  and smith `a277e922`. Compare each fix with its parent at the same pins;
  never credit repin changes. The restore waits for the full capability.
- **Phrase capacity belongs with the book.** Make `Words` opaque and
  compute its capacity on construction and book replacement. No mutable
  book accessor, cross-test state or new heap allocation. Keep the raw
  `phrases::jig_capacity` calculation for construction and independent
  checks. Loading's 2.7 s is not all removable: skein owns the checked
  constructor, with runtime limits and application languages. See Questions
  for the prepared-data work needed before that saving can be bounded.
- **No speculative fixture rewrite.** Observer snapshots, core startup and
  the drift outlier have no demonstrated avoidable saving of 0.5 s.
- **Same measurement selection before and after each fix.** Build first
  with `cargo nextest run --workspace --no-run`; measure committed
  trees in the gate lane. For each Proof selection, run once on its
  parent and once on its candidate, using `--profile measure -j 1`.
  Sum nextest's PASS times with the findings' Python command, pointing
  it at each test log. Record count, sum, difference, machine, pins,
  revisions and log paths in the commit. Retain all tests and comparisons;
  report added tests separately. Millisecond-rounded sums and sampled
  estimates are informational; the restored timeout is the gate.
- **30 s until the revert.** Keep ruling 275's timeout while fixes land;
  serial diagnostics use the measure profile. Restore on three default
  runs' median plus a serial sum at kept pins, with no 12 s condition.

## Rules that bind it

- Keep crate/fixture ownership and test observations/settings. Only 04.2
  changes public `Words` access, migrating its consumers. No STATUS.md edits.
- Every cargo command goes. Measurements use its gate
  lane; builds use its build lane. Each landing has the gate;
  this session adds no separate whole-gate run.
- No optimization override, changed opt-level, reduced debug assertions
  or overflow checks, merged test processes, deleted, weakened, merged
  or ignored tests, moved tests between suites, or changed assertion.
  No timeout change except the inverse of `0509b985`. Leave workflow.md
  2, tech-debt.md 4, Cargo.toml, filters, slow periods and other budgets
  untouched. A hot leaf crate needs a profile and the user's or
  user's sign-off beside its override; this session takes none.

## Increments

### 04.1 Compute common card and page bounds once per sizing call
Scope: the repeated-bound family in findings 3A, retaining view.md 10's
complete page capacities. Expected saving: 1–1.5 s summed.
Needs: nothing beyond the session's start.
Files: jig `crates/jig-client-view/src/sizes/pages.rs`,
`crates/jig-client-view/src/owner_tree.rs`,
`crates/jig-client-view/src/sections/project/owner.rs`.
Shape: `arranged_nodes` computes `card_nodes` and `chip_nodes` once and
passes them through its task and project calculations. Add private
`task_header_in`, `task_section_in` and `project_section_in` helpers;
the public companions remain wrappers with their existing signatures.
`owner_tree::max_nodes_with_card` and
`sections::project::owner::max_nodes_with_card` accept the already
checked card bound; their public `max_nodes` wrappers compute it for
standalone callers. The project's `historical_nodes` receives that
bound. Compute the common project-owner maximum once per project page,
then reuse it for its ten sections. Keep every alternative in each
maximum, the arrangement's sum, and every checked operation. No global
cache, retained state or handwritten replacement bound.
Test shape: leave `crates/jig-client-view/src/tests/page_capacity.rs`
unchanged, including `each_page_bound_keeps_its_value_at_the_client_limits`,
`an_arranged_task_page_counts_the_sum_of_its_sections`,
`no_arrangement_keeps_each_page_bound`,
`capacity_composes_siblings_fragments_and_application_pages_with_checked_overflow`
and `every_page_with_full_windows_nested_lists_and_maximum_text_fits_its_composed_capacity`.
They reuse `tests/cards.rs::{CLIENT,words,start}` and
`tests/fixtures/client.rs::Fixture`. Keep their exact numbers, overflow
refusals, full windows, fragment counts and limits. Keep the joint
client world's existing `test::fixture_with` and
`joint/fixture.rs::bounded_world`, with its current settings and every
referee. No new test kind or fixture. Pure sizing checks stay under
20 ms; focused world stories under 0.5 s (development.md 5.6).
Proof: joint, once before and after, run
`cargo nextest run --profile measure -j 1 -p jig-client-view -p jig-client-world`.
Its existing joint stories run the real view, client and engine with
simulated I/O, above the sizing tests. Every test passes with unchanged
checks; the 927-test base cohort sums to 17.768 s. Quote both measured
sums and the saving in the commit, named "Reuse common page bounds".
Bound: 60 s per diagnostic after building. An absent or reversed gain
is the question below, not a claimed improvement.
Don't: change public capacity values, drop an overflow path, replace
an observed size with a larger guess, trim a full fixture, or edit
`owner_escalation::nodes`' eight cases to save work.
Size: at most 180 production lines across the three files.
Removes: repeated card/chip and common project-bound calculations
inside one page sizing call; no API or assertion.

### 04.2 Keep the vocabulary capacity with the replaceable book
Scope: findings 3C's repeated capacity scans; view.md 7 and 10's exact
whole-vocabulary envelope. Expected saving: 0.8–1.2 s summed, from 1.4 s.
Needs: nothing beyond the session's start.
Files: jig paths below; braces enumerate files, not open-ended globs.
`crates/jig-client-view/src/{words,limits,historical_pages,literal_segment,
confirmation,policy_edit,proposed_task,runtime,owner_activity,owner_amendment,
owner_authority,owner_authority_words,owner_budget,owner_budget_words,
owner_chip,owner_chip_words,owner_deployment,owner_historical,owner_history,
owner_milestone,owner_outcome_words,owner_person_task,owner_proposal,
owner_result,owner_tree}.rs`; `cards/proposal.rs`, `turns/call.rs`,
`sections/{deployment,inbox,task}.rs`, `sections/project/owner.rs` under
that same `src/`. Shared with 04.1: `owner_tree.rs`, project `owner.rs`;
with the repin/named-send/turn-read work: words and their consumer tests.
Under that `src/tests/`: `{phrases,cards,conversation_work,frame_facts,
historical_objects,historical_pages,owner_activity,owner_amendment,
owner_budget,owner_chip,owner_deployment,owner_history,owner_inbox,
owner_milestone,owner_outcome,owner_person_task,owner_project,owner_proposal,
owner_result,owner_tree,page_loss,proposed_task,question_sender,turns}.rs`,
`owner_history/proposal_fragment.rs`.
`testing/jig-client-test-view/src/{connector_values,connector_values_words,
family,limits}.rs`, its `tests/{connector_values,world_limits}.rs` and
`tests/connector_values/deployment_diagnostics.rs`;
`testing/jig-client-test-domain/src/tests/root/navigation.rs`;
`examples/ops/client-view/src/{limits,tests}.rs`;
`tests/ops/src/{client/app,joint/fixture}.rs`.
Shape: `Words::new(book, language, offset_minutes)` computes the existing
`phrases::jig_capacity(&book)` once into private `jig_capacity: Option<u32>`.
Make all fields private (`partial_pub_fields` forbids a half-opaque type).
Expose shared `book()`, `language()`, `offset_minutes()` and value setters;
`set_book` computes the new capacity before replacing book and capacity
together. No mutable reference escapes. Clone carries both together.
Language/offset changes do not change a maximum over all loaded languages.
Use `Words::jig_capacity()` in each current whole-vocabulary caller,
including `limits::{of_words,compose}` and `Words::node_text`. Retain the
latter's authority-only fallback on `None`, at its present call site:
constructing an unrelated partial book must not acquire a new panic.
Keep `phrases::jig_capacity` and `owner_words::phrase_bytes` unchanged:
they exclude unrelated owners and refuse any missing required phrase.
Migrate literals and field access mechanically; family-specific `Words`
types are unchanged. The `Option<u32>` is inline; heap envelopes and
fixture allowances stay unchanged. The closing repin repairs downstream
API users (development.md 5.7); no direct user was found in its current code.
Test shape: preserve assertions while adapting construction/access. Keep
`tests/phrases.rs`'s partial-book and English composition checks. Add
`replacing_words_recomputes_capacity_without_changing_a_clone`: use
`phrases::english::english(cards::CLIENT.text)`, clone, replace by a
different hole bound, then the existing partial-book fixture; compare
each result with the unchanged raw scan and require `None` for partial.
Add `words_capacity_covers_unselected_languages_and_ignores_other_owners`:
load the existing English shapes/language with a longer second translation
and an unrelated owner's still longer phrase; use `Phrasebook::load`,
retain checked limits, and compare with the raw vocabulary scan before
and after selecting either language. Reuse `tests/phrases.rs`'s loader
pattern; these are ordinary phrase unit tests, each under 20 ms. Retain
`owner_tree_view_uses_checked_selected_language_and_finite_node_text_scratch_envelopes`,
all owner partial-book/refusal checks and the joint world's referees and
limits; its focused stories stay under 0.5 s.
Proof: joint, once before and after on this increment's own parent, run
`cargo nextest run --profile measure -j 1 -p jig-client-view -p jig-client-world`.
The real view and client meet the engine above the phrase unit checks.
Compare the same 927 base tests (17.768 s at the profile base); report
new tests separately. Quote counts, sums and saving in the commit,
named "Keep vocabulary capacity with its book". Bound: 60 s after
building. The migrated test-kit and ops consumers also pass the gate.
Don't: cache by address, language or text limit; substitute a maximum
over unrelated phrases; rescan on each getter, including debug builds;
bypass loading checks; change a partial book's acceptance or assertions.
Size: at most 280 production lines, including mechanical consumer edits.
Removes: repeated whole-vocabulary scans from sizing/rendering and public
field mutation that could separate a book from its computed capacity.

### 04.3 Read the client's settled bound from its existing queue
Scope: findings 3B's repeated wire validation inside `Client::drain`,
with client.md 3, 11 and 14 unchanged. Expected saving: 0.7–0.9 s summed.
Needs: nothing beyond the session's start.
Files: jig `crates/jig-client/src/client.rs` (`Client::drain`). This is
a shared file with the named-send session; keep its event changes.
Shape: at entry to `drain`, read `let bound = self.domain_out.capacity()`.
Use it for the iteration bound and the replacement `saves` and `erases`
lists. `Client::build` already constructs both output queues and those
lists with the same checked `max_out` value; they never change capacity.
Keep `new`, `unlinked`, `build`, public `max_out`, the empty-queue
assertion and construction-time validation. No extra cached field,
changed memory allowance or new interface is needed.
Test shape: keep `crates/jig-client/src/tests/store.rs`,
`tests/unlinked.rs`, `tests/versions.rs` and their shared
`tests/fixtures.rs::Client` unchanged. In particular retain
`a_keyed_request_waits_for_its_record_to_be_stored`,
`a_record_carried_again_is_decided_by_its_later_write`,
`a_step_s_records_leave_in_one_save` and
`an_unlinked_client_s_records_are_the_table_s`.
Keep `tests/client-protocol/tests/protocol.rs`'s
`a_request_whose_answer_was_lost_is_made_once`, and the whole view and
client-world cohorts, with all queue, store, memory and delivery checks.
Fixtures retain their limits; no tests added. Unit checks stay under
20 ms and focused world stories under 0.5 s (development.md 5.6).
Proof: joint, once before and after, run
`cargo nextest run --profile measure -j 1 -p jig-client -p jig-client-view -p jig-client-world -p jig-client-protocol-world`.
The real client meets the real engine over the protocol world, above
its unit tests. All tests pass unchanged; this selection at the base
has 1,042 tests and sums to 22.816 s. Use this increment's own parent
for the comparison, so earlier savings are not counted twice. Quote both
sums and the saving in the commit, named "Reuse the client's drain
bound". Bound: 60 s per diagnostic after building.
Don't: skip admission validation, change `max_out`'s contract, shorten
the drain, grow a queue, discard an output, or relax the store barrier.
Size: at most 15 production lines.
Removes: repeated wire-limit construction and validation from draining
and pending-record list replacement; nothing from startup validation.

### 04.4 Revert ruling 275 and restore the focused 15 s timeout
Scope: testing-strategy.md 8 and the test-cost debt: the
focused suite's timeout back to 15 s after the fixes at the kept pins.
Needs: jig on skein's and smith's main; common page bounds reused;
vocabulary capacity retained with its book; the client's drain bound
reused. No dependency on named send or turn read; 15 s binds their additions.
Files: jig `.config/nextest.toml`, `[profile.default]` only.
Shape: apply `git revert --no-commit 0509b9859aa6de4b6582dfbb689464cf0c2796b9`
and retain just its inverse: remove the ruling-275 comment and replace
`global-timeout = "30s"` with `global-timeout = "15s"`. If surrounding
filter text conflicts, keep the current filter and resolve those two
lines alone. Leave workflow.md and tech-debt.md as they are.
Test shape: no test changes; the same full focused selection as the
gate, with each test's own process, unchanged filters and settings.
Proof: joint, the focused suite's real-component worlds with simulated
I/O, measured at its default parallelism. On the committed parent with
the Needs present, build once, then run exactly three times
`cargo nextest run --workspace`.
Require all tests passing and the median of nextest's three Summary
times **under 15 s**. Run once
`cargo nextest run --workspace --profile measure -j 1`
and sum its PASS times. The commit names the three fixes, the machine,
pins, measured revision, three times, median, test count and serial sum.
The serial sum accompanies wall time for comparison across machines;
it is still machine-dependent. Bound: 30 s for each default run under
the existing timeout, 120 s for the serial diagnostic after building.
Then make the revert; the gate proves the candidate under 15 s.
If the median misses, leave 30 s in place and raise the measured gap;
this increment is unfinished, and no repeated unchanged trials or
alternative timeout are prescribed.
Don't: wait for the remaining jig features; edit a test or another
profile; replace the timeout with a warning; change either cost document
or client-api's plan files to make the debt look closed.
Size: at most 1 configuration line changed, plus removal of its comment.
Removes: ruling 275's temporary doubled timeout and its comment.

## Interface sketches

04.1 owns these internal companions. Callers pass a card bound from the
same `Limits`, never from a different configuration.

```rust
// crates/jig-client-view/src/owner_tree.rs
pub(crate) fn max_nodes_with_card(l: &Limits, card: u32) -> Option<u32>;
// crates/jig-client-view/src/sections/project/owner.rs
pub(crate) fn max_nodes_with_card(l: &Limits, card: u32) -> Option<u32>;
// crates/jig-client-view/src/sizes/pages.rs
fn task_header_in(l: &Limits, card: u32, chip: u32) -> Option<u32>;
fn task_section_in(l: &Limits, section: TaskSection, card: u32, chip: u32) -> Option<u32>;
fn project_section_in(l: &Limits, section: ProjectSection, card: u32, chip: u32, owner: u32) -> Option<u32>;
```

04.2 owns `Words`; language/offset access adds no admission rules.

```rust
// crates/jig-client-view/src/words.rs, impl Words (all fields private)
pub fn new(book: Phrasebook, language: u16, offset_minutes: i32) -> Self;
pub fn book(&self) -> &Phrasebook;
pub fn set_book(&mut self, book: Phrasebook);
pub fn jig_capacity(&self) -> Option<u32>;
pub fn language(&self) -> u16;
pub fn set_language(&mut self, language: u16);
pub fn offset_minutes(&self) -> i32;
pub fn set_offset_minutes(&mut self, offset_minutes: i32);
```

04.3 reads skein's existing `Queue::capacity(&self) -> u32`. The graph
and source confirm it; it is already available at jig's base pin.
New signatures are checked when implemented; the profile exercised the
existing fixture and proof selections. No code was changed in this revision.

## Questions to raise when reached

- **A fix does not reduce its matched cohort's summed time:** report the
  two logs, counts, pins and diff to the supervisor. The samples give
  causes, not a promised speedup; resolve the discrepancy before calling
  the increment a cost fix. Do not retry unchanged until a good time.
- **After the repin, the default median is at least 15 s, or the gate
  still exceeds 15 s:** give the three times, serial sum and changed
  package counts. Attribute the remaining cost before proposing a fix.
- **First-load phrase work (C, about 2.7 s) remains:** no safely bounded
  jig-only no-sort/no-validation increment was found. In skein
  `crates/skein-ui/src/phrases.rs::Phrasebook::load`, storage is private
  and the only constructor sorts, checks forms/holes/counts and computes
  capacities. jig's `phrases/{english,jig}.rs` and `owner_words.rs` assemble
  owned data at runtime: shapes depend on `text`; the test view's
  `testing/jig-client-test-view/src/connector_values_words.rs` adds owners
  and translations. Sorted input still goes through both loader passes.
  The profile does not separate sorting from checks/capacity work: 2.7 s
  cannot be credited to presorting. A fix needs a prepared-data constructor in skein
  and generated/constant ordered tables, proving duplicate/missing/form/hole
  invariants once while retaining checked runtime limits and capacities;
  debug checks must expose bad tables; the arbitrary-language loader and
  refusal tests remain. Separate those phase costs at the kept pins before
  scheduling upstream work and its own repin. Suppressing test-build
  validation is not a fix under the ruling.
- **The slowest baseline test persists:** it is
  `jig-client-wire-world::drift::client_wire_schema_matches_generated_code_and_goldens`
  (0.873 s serial). `tests/client-wire/tests/drift.rs::check` parses and
  generates once, then reads and compares committed code and every golden.
  Saved default logs in workspace `tmp/logs/20261018T{232723,232758,232840}-3795.log`
  show 0.833, 0.122 and 0.123 s. No repeated parse, sleep or avoidable
  ≥0.5 s cause is established, so no increment yet. If it stays slow,
  separate parse/generate CPU from file-read time at the kept schema/pins;
  keep every comparison.
- **A pass here could still exceed 15 s on four cores:** the base is
  5.546 s here but earlier carbon-forge gates were about 14.4 s with
  excursions past 15 s. Give the supervisor the host and serial before/after
  sums to assess its margin. A local pass proves no result on carbon-forge;
  a pinned four-core measurement answers it. Infer no fixed speed ratio
  and add no unapproved machine gate.
- **A profile isolates unavoidable cost in one hot leaf crate:** raise
  its stack and cost to the supervisor for the user's
  sign-off. No override is part of this session.

## Done when

In jig, with `origin/main` refreshed. These read current file properties;
Proof supplies measurements. The timeout enforces each subsequent gate.

- The restored timeout and the other suites' settings are present:
  `git show origin/main:.config/nextest.toml | python3 -c 'import sys,tomllib; p=tomllib.loads(sys.stdin.read())["profile"]; assert p["default"]["global-timeout"]=="15s"; assert p["fuzzy"]["global-timeout"]=="60s"; assert p["measure"]["global-timeout"]=="1h"'`.
- The old comment is gone:
  `! git show origin/main:.config/nextest.toml | grep -q 'ruling 275'`.
- Test builds remain unoptimized, with only the existing tools exempt:
  `git show origin/main:Cargo.toml | python3 -c 'import sys,tomllib; p=tomllib.loads(sys.stdin.read())["profile"]; d=p["dev"]; assert d.get("opt-level",0)==0 and d.get("debug-assertions",True) and d.get("overflow-checks",True); t=p.get("test",{}); assert t.get("opt-level",0)==0 and t.get("debug-assertions",True) and t.get("overflow-checks",True); assert set(d.get("package",{}))=={"jig-boundary","skein-codegen","skein-ui-bundle"}; assert not t.get("package",{})'`.
- The existing cost documents remain unchanged by this session:
  `diff <(git show b2d920e4:docs/development/workflow.md | sed -n '/^## 2\. /,/^## 3\. /p') <(git show origin/main:docs/development/workflow.md | sed -n '/^## 2\. /,/^## 3\. /p')`
  and `diff <(git show b2d920e4:docs/development/tech-debt.md | sed -n '/^## 4\. /,/^## 5\. /p') <(git show origin/main:docs/development/tech-debt.md | sed -n '/^## 4\. /,/^## 5\. /p')`.
- Each sizing owner has its internal companion:
  `for path in crates/jig-client-view/src/owner_tree.rs crates/jig-client-view/src/sections/project/owner.rs; do git show "origin/main:$path" | grep -q 'fn max_nodes_with_card(' || exit 1; done`.
- Draining reads the queue's bound and performs no `max_out` computation:
  `git show origin/main:crates/jig-client/src/client.rs | sed -n '/fn drain(/,/fn route_link(/p' | grep -q 'self.domain_out.capacity()'`
  and `! git show origin/main:crates/jig-client/src/client.rs | sed -n '/fn drain(/,/fn route_link(/p' | grep -q 'crate::max_out'`.
- Words owns replacement and exposes only a shared book:
  `git show origin/main:crates/jig-client-view/src/words.rs | grep -q 'fn set_book('`
  and `git show origin/main:crates/jig-client-view/src/words.rs | grep -q 'fn book(&self) -> &Phrasebook'`;
  `! git show origin/main:crates/jig-client-view/src/words.rs | grep -Eq 'pub(\([^)]*\))? book:'`;
  `git show origin/main:crates/jig-client-view/src/words.rs | grep -q 'jig_capacity: Option<u32>'`.
- Whole-vocabulary callers use the stored result; the raw calculation is
  confined to construction/replacement and tests:
  `! git grep -n 'phrases::jig_capacity(' origin/main -- crates/jig-client-view/src ':!crates/jig-client-view/src/words.rs' ':!crates/jig-client-view/src/tests'`.
