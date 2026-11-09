# smith code-quality review, 2026-10-31: area reports and rubric

Companion to `2026-10-31-quality.md`. The rubric and the 13 reviewers' reports, unedited apart from this header and the dividers. Snapshot: `manual/component-kit` at `aa95945cc`. Paths are relative to `smith/`.

---

# Code-quality review of smith: shared brief

Tree: a clone of github `manual/component-kit` at `aa95945cc`.
Paths below are relative to `smith/` in that clone unless they start with `skein/`.

Read only: do not build, run tests, or change anything in the clone. Never touch any other
path outside that clone. The only file you write is your report. Review the code as it
stands on this branch; do not diff it against main.

## What is being asked

Not a bug hunt. The user wants an assessment of **code quality** of production code **and
tests (tests are code and are judged the same way)** on these criteria:

1. **Cleanliness**: readable, no dead code, no noise, comments that earn their place.
2. **Organization**: module/file layout, where things live, file sizes, cohesion.
3. **Taste**: idiom, judgement, does it look like it was written by someone who cares.
4. **Naming**: precise, consistent across the area, matches the design's vocabulary.
5. **Primitives**: orthogonality and decoupling. Are the abstractions the right ones, each
   doing one thing, composable, with narrow interfaces? Leaky or overlapping concepts?
6. **Elegance**: does the shape of the code follow the shape of the problem?
7. **Economy**: could the code be smaller? Duplication, near-copies, boilerplate, over-long
   tests, fixtures retyped, parallel hierarchies. Estimate the lines that could go
   (e.g. "~600 of 2,100 lines in tests/x.rs").
8. **Complexity**: is it just complex enough for what it needs, and no more? Speculative
   generality, layers that only forward, state that could be derived.

## Scores

Score each criterion 1-5 against this scale, and do not default to 3:
- **5**: a model for the rest of the tree; you would point a new contributor at it.
- **4**: good; the problems are local and few.
- **3**: works, with debt that is visible across the area.
- **2**: the debt shapes the area: most changes here pay for it.
- **1**: needs reworking.

A score below 5 needs one line saying what keeps it from the next score up.

## Calibrate to the house rules (important)

smith follows skein's deliberately small Rust. Read, before you judge anything:
- `AGENTS.md` at the repository root, sections 1 and 2, and `smith/AGENTS.md`;
- `skein/docs/foundation/programming-model.md`, sections 1, 3, 4, 5.4 and 10 (no std in step
  crates, no closures/traits/generics/dyn, exhaustive matches with no `_`, no indexing, no
  `as`, checked arithmetic, one handler per state×event cell, the `mem::replace` transition
  idiom, free functions over fields, `Writer`/`Reader`, slabs and `Id<T>`);
- `skein/docs/foundation/testing-strategy.md`, section 10 (a test's shape: one behaviour per
  test, rows carry data, the setup the assertion reads, a setup written once, named fixtures
  and sizes, a bound's two sides, a recorded stream edited as records);
- `docs/development/tech-debt.md`, smith's known debt. Do not report an entry there as a
  discovery; do report how it shows up in your area, with evidence and size.

Then:
- Do **not** flag verbosity the rules force (a long exhaustive match) as uneconomical. Judge
  economy *within* the rules: given these rules, is this the smallest clear form?
- Do flag where code breaks its own rules in ways that hurt quality (tests that retype
  setups, bare numbers, branching on row labels, an exception that has no written reason).
- If a rule itself costs a lot of code in your area, note it once, separately, with a concrete
  example and rough cost. Low priority.
- Comments in this repo use a particular plain prose style; do not flag the style itself.
- Test crates (`tests/*`, inline `tests.rs` and `*_tests.rs`) are ordinary Rust (may unwrap,
  index, use closures): judge them as ordinary Rust held to section 10.
- Files under `generated/` are output: skip them, and judge their schema and how they are
  produced only if your area owns that.
- smith's dependencies point one way, onto skein. Where smith reimplements something skein
  should own, that is a finding (name the skein crate); `smith/AGENTS.md` forbids second
  copies of test kit mechanisms.

## How to work

- You cannot read everything. Map the area first (module trees, file sizes, public surface,
  how tests are organized), then read closely where quality matters most: the core types and
  state machines, the biggest files, the test support and fixtures, and a sample of tests.
  Say what you read closely and what you only skimmed.
- Your area's tests count as much as its production code. A crate's inline test files and its
  test crate are both yours. Sample tests from several files; do not stop at the world.
- Prefer **patterns** (a thing done 20 times) over one-off nits. A finding is worth reporting
  if fixing it would make the code noticeably smaller, simpler or clearer.
- Verify every finding against the source; quote `file:line`. No speculation reported as
  fact; mark uncertain items "(unsure)". Count a pattern with a search rather than guessing.
- Look across crate boundaries inside your area: duplicated helpers, overlapping types, a
  test harness reinvented per test crate.
- A clean area is a valid result. Do not pad the findings to look thorough, and do not
  invent a concern to fill a section.

## Report (write it to the path your task names; ≤ ~1,800 words)

1. **Scope read**: closely / skimmed.
2. **Scores**: 1-5 for each of the 8 criteria, one line of justification each.
3. **Strengths**: 2-4 bullets (what to keep doing; be specific).
4. **Findings**, ranked by payoff, numbered. Each: title; criterion; evidence (`file:line`,
   several sites for a pattern); why it matters; concrete fix; estimated size effect.
5. **Cross-area suspicions**: things you noticed that likely recur outside your area or
   involve crates outside it (name them), unverified beyond your area.
6. **Rule costs** (optional, short).

After writing the report, your final message is short (≤ 120 words): the report's path, the
eight scores on one line, and your top three findings by title. The report holds the rest.

---

# 01: smith-domain, smith-domain-tools, tests/tools

Paths: `D/` = `crates/smith-domain/src`, `T/` = `crates/smith-domain-tools/src`, `W/` = `tests/tools`.

## 1. Scope read

Production is 8,850 lines; tests are about 11,500.

**Closely:** all production in `T/`; in `D/`: `domain`, `route`, `peer`, `operation`, `translate`, `feedback`, `config`, `limits`, `facts`, `waking`, `boundary`. All of `W/src` except `world/jobs.rs`. The fixtures and harnesses of `T/tests.rs` and `D/tests.rs`, and about a third of their tests. `W/tests/suite/paths.rs`, the first third of `simulation.rs` and of `jobs.rs`.

**Skimmed:** `D/llm.rs` past line 120, `D/tests/{program,builtin,completion_retry}.rs`, `T/tests/jobs.rs`, `W/src/world/jobs.rs`, `W/tests/suite/memory.rs`, the fuzzy suites, and test names only for the remainder.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 2 | The code is tidy, but comments are unreliable: filler field docs, some wrong; the core modules' contracts are stale. Dead code carries an `#[expect]`. |
| Organization | 3 | `T/` is one topic per small file. `route.rs` holds five topics, and the test files interleave fixtures and tests. |
| Taste | 3 | Careful state machines, with lapses: a hand-rolled decimal twice, a terminal wrapped only to be unwrapped, false lint reasons. |
| Naming | 3 | Local names are good (kit, flight, knowledge). The public vocabulary is repository and checkout where the design says directory and workspace. |
| Primitives | 3 | The typed boundaries and tickets are right. `Done` is one union of every operation's terminals, and `Repo.mount` is wider than anything production makes. |
| Elegance | 4 | `flight.rs`, `kit.rs` and `route.rs` follow state × event and parent routing. Terminal lists and job reads are the local exceptions. |
| Economy | 2 | About 500 production and 1,250 test lines could go. One field added to `Outcome`, `Event::Start` or `Limits` is paid for at 19 to 36 sites. |
| Complexity | 3 | Mostly just enough. Namespace shapes no run makes, a queue emulated on a map, and an unused bound are the excess. |

## 3. Strengths

- **State machines to imitate:** `T/flight.rs` has its table in the module doc, `mem::replace`, one handler per state, and `settle` and `follow` in one place. It is the 5.4 idiom done properly.
- **Small pure modules:** `T/edit.rs`, `window.rs`, `knowledge.rs` and `command.rs`, each with exact table tests (`T/tests.rs:627`).
- **Order predicates named for the design:** `run_handoffs_drained` and `child_facts_gathered_before_handoff` (`D/route.rs:466`, `D/domain.rs:399`) are the names the design and worlds cite.
- **The world reuses skein:** `Schedule`, `Ledger`, `Stage`, `Trace`, `Meter`, `assert_replays` and the fake checkout. Its referee (`assert_told`, `assert_untouched`, the version checks at `W/src/world.rs:770`) earns its size. `tests/session` imports its `translate` rather than copying it.

## 4. Findings

**1. Filler field docs, some false, and stale contracts** (Cleanliness)
- **False:** `T/boundary.rs:56` calls `call: Call` a "Parent-issued tool-call token". `:95` says `outcome` is "validated against the host's contract". `T/path.rs:107` documents `Part` as "One bounded block accepted from the provider stream". `T/call.rs:175,183` say "Contained" where `tools.md`, section 5 and `T/boundary.rs:208` say commands are not.
- **Keyed by field name, whatever the type:** `more` ×4 ("Problems or matches omitted"), `bytes` ×4, `tail` ×4, `hits` ×3 (on `Box<[Hit]>` and on `u32`), `version` ×3, and `D/boundary.rs` `grant` ×3 ("Credential name or refresh notice", on a `GrantName`).
- **Stale contract:** `T/kit.rs:8` says an edit must be of a file its LLM has read, and `T/flight.rs:10-12,40` say the load must match the known version. `tools.md`, section 4 says no read is needed, and `to_edit` checks none. Also `T/facts.rs:134`, `T/call.rs:164`.
- **Stale panic text:** "the tools issue no yield or job act yet" ×6 (`T/flight.rs:448,471,504,561,629,655`), while `:317` handles `Held`.
- **Misplaced:** `D/translate.rs:213-216` is the doc of `end`, attached to `PROGRAM_NAME`. `:25,:28` describe tickets that are `DELIVER` and `WAIT`. Reflow scars at `D/route.rs:615` and `D/boundary.rs:361`.
- **Banned or historical words:** "actual" on 20 production lines, "legacy" ×4, "V2", "old product limit" (`T/tests.rs:963`).

*Why:* `smith/AGENTS.md` documents a field only when name and type don't say it. These docs add reading cost and now mislead. *Fix:* delete what name and type say, and correct the module tables. *Size:* about −200 of 587 `///` lines in the four boundary and fact files.

**2. Test literals retyped, against "A setup written once"** (Economy)
- `W/tests/suite/jobs.rs` holds 36 full `Outcome::Running` or `Outcome::Job` literals of 7 fields, and most vary only in `head` and `end`. `simulation.rs:20` already shows the cure (`read_of`). Three scenario functions are also called by one test each (`:109`, `:167`, `:220`).
- `Event::Start` with 11 fields is written 19 times (`D/tests.rs:559,590,620,642…`). Each copy retypes `window`, and `Token::new(7)` appears 28 times.
- `D/tests.rs:280-323` holds four near-identical `ConfiguredModel`s.
- The `b"host"` `Repo` literal appears 19 times in `T/` and `W/`.
- `tools::Limits` is retyped: 19 `shell_tail:` literals across 14 files, 13 of them test files.

*Fix:* `running(head)` and `job_read(head, end)`; `model(endpoint, name)`; `repo(...)`. Make `Event::Start` carry a `Start` struct, so tests write `Start { grants, ..start() }` and `D/route.rs:287` loses its 12 arguments and its `#[expect]`. *Size:* about −280 of 951 lines in `jobs.rs`, about −230 of 2,651 in `D/tests.rs`.

**3. The tools serve, and the worlds pin, a namespace no run produces** (Primitives, Complexity)
- `D/translate.rs:160` is the only production `Repo` constructor: `mount: Box::new([name])`, one `Name`, which `/` cannot enter. `cwd` is that name or empty.
- Code that exists only for other shapes: `T/authority.rs:49-52`, `:221-262` (join, then split at the first name), `T/kit.rs:276-315` (`first_name`), `beneath`'s empty mount, and the longest-mount choice. The comments call it "Legacy".
- The fixtures use it: nested `/work/temper/vendor/lib` and two-name mounts (`W/src/fixture.rs:40-42`, `T/tests.rs:217-229`) and a root mount (`W/src/memory.rs:64`). So `W/tests/suite/simulation.rs:108` asserts `Resolved::Working { path: "temper/escape" }`, where a run's one-name mount gives `escape`.

*Why:* tests prove a shape other than the product's, while the real shape gets one test (`simulation.rs:1119`). *Fix:* `Repo.name: Name`, `Checkout.working: Option<Name>`, one-name fixture mounts. *Size:* about −80 production lines; a mechanical test rewrite. The nested-repository story goes.

**4. `Done` is one union of every operation's terminals** (Primitives, Economy)
- Each of the six handlers (`T/flight.rs:431-669`) relists the same common terminals, then 8 to 10 other operations' as `unreachable!`: 16 `unreachable!` sites in the file. `T/jobs.rs:231-251` and `:378-398` add 18-line lists.
- `T/jobs.rs:149` builds `Done::Job` only so `answer` can unpack it.

*Fix now:* `answer` takes the four fields (−20 lines, one impossible cell gone). *Then:* per-operation terminals, or `Done::Common(Common)` with one `common()` mapping, so each handler has one impossible arm. The change crosses `smith-protocol-machine`. *Size:* about −150 lines.

**5. `D/route.rs` holds five topics in 1,208 lines** (Organization)
- Event routing and hand-offs; start admission (`:287-419`); completion admission and settlement (`:128-226`, `:743-922`); opening (`:924-1082`); grants (`:1103-1163`) and capture (`:1165-1208`). All reach through `pub(crate)` fields of a 21-field `Domain`.
- Repeats: the refused `Request::Answer` block ×5 (`:53`, `:62`, `:307`, `:316`, `:333`) and `run::Event::Ended` ×5 in `open` (`:957-1006`).
- Two `Exhausted → BudgetDenial` maps (`:757`, `:866`) sit outside `translate.rs`.
- Nested `Option` matches that a small `?` helper would flatten (`:352-366`, `:849-859`).
- `:704-706` compares own enums with `==`, then relists them in a second exhaustive match.

*Fix:* `start.rs`, `completion.rs`, `open.rs`, `grants.rs`, capture into `facts.rs`; a `refuse` and a `refused_open` helper. *Size:* about −60 lines, and `route.rs` near 450.

**6. Behaviours tested twice, and test shape** (Economy, Organization)

`testing-strategy.md`, 2.1 gives step tests one transition or refusal, and 2.2 says "Behaviour is tested here". `T/tests.rs` repeats world stories:
- `:1840` / `simulation.rs:1329` (edit batches);
- `removal_requires_a_current_read_*` in both;
- no-ops beside real edits, host aliases, `list /`, kept-output split, closing a kit.

I found about a dozen pairs by name and compared one closely, so the size is an estimate: about −500 of 2,234 lines.

Shape:
- 12 of 52 `simulation.rs` tests run two or more worlds, with 23 narrating comments ("One behaviour per test").
- `simulation.rs:1128-1161,1193-1217` branch seven times on `several` ("Rows carry data").
- `D/tests.rs` opens with a test before its constants, with fixtures at `:243` and `:496-800` among tests. Two of four submodules use `#[path]` (`:6-12` versus `:2130`).
- A test name says `an_listing` (`simulation.rs:255`), and others say `the_worker` (`D/tests.rs:1111`).

**7. "World contract" on every item of `W/src`** (Cleanliness)

121 identical lines plus 2 variants, on private items too, each costing two lines. They cite `tools.md`, section 9, which is "From temper". `smith/AGENTS.md` says module or type level, never every field; no script or lint requires it (searched). *Fix:* one line in `lib.rs`. *Size:* about −240 of 2,330.

**8. Three exhaustive `Outcome` matches in one fact, and a pair repeated** (Economy)
- `T/facts.rs:194-349`: `answered`, `outcome_job` and `observed` each walk the same 28 variants; one match returning the four values does it. About −75.
- The fact-then-answer pair appears 7 times (`T/kit.rs:123,173,180`, `T/jobs.rs:128,433`, `T/flight.rs:384,392`), with `Observation { 0, false }` 5 times. A helper saves about −35.
- `T/kit.rs:488-509` guards `usize`→`u64` with four spelled `TooLarge` values. About −15.

**9. Job reads emulate a queue on a map** (Complexity)

`Jobs.reads: Map<u64, Read>`, `next_read` and `Job.reader` are a per-job FIFO, recovered by three scans (`T/jobs.rs:198,258,284`). `cancel_one` issues an unexplained read of `u64::MAX` nanoseconds (`:318-322`); it needs a comment. `Job.reader` means something in one state only (5.4). *Fix:* a `Queue<Read>` on each job, with an active flag or a `Reading` state. *Size:* about −40.

**10. Dead and second-copy production code** (Cleanliness, skein reuse)
- `D/limits.rs:122-152`: `prompt_payload` is unused, under `#[expect(dead_code)]`, and describes a "legacy root result union". `:84` `uncharged` is test-only.
- `D/feedback.rs:539-552` and `D/waking.rs:66-81` reimplement `skein_lib::Decimal`, which sibling crates use (`smith-domain-run/src/prompt.rs:796`).
- `D/translate.rs:216` hard-codes `b"claude_code"`, though `Config.programs[].name` carries it (`D/config.rs:93`).
- `D/config.rs:100`'s `manual_let_else` reason is false: 10.2 allows `let else` on `Result`.
- `D/route.rs:802`'s reason cites 10.1, and it covers a 100-line function.

*Size:* about −75.

**11. Vocabulary drifts from the design** (Naming)

`T/` has 77 lines saying "checkout" and 30 "repository"; `tools.md` has no "checkout" and 61 lines of "director-". `Repo`, `repos`, `Checkout` and `Mount` are public or crate-wide. The seam shows at `D/route.rs:329` (`run.directories > tools.repos`) and `D/translate.rs:158-161`.

Smaller:
- `Timed` holds a job wait (`T/jobs.rs:204`).
- `Next` and `Read` each name two types (`T/call.rs:141`/`T/flight.rs:102`, `T/jobs.rs:42`/`T/window.rs:38`).
- `Jobs::held` is a mutator that mints a number.
- One idea is `after`, `yielded` and `wait: after` (`T/facts.rs:316`).
- "engine" and "worker" leak in (`T/call.rs:354`, `D/boundary.rs:165`, `D/domain.rs:323`).

*Fix:* rename in one commit, with no line change. Its stored or public reach is unverified.

## 5. Cross-area suspicions

- `tests/session/src` has 235 more "World contract" lines, also likely citing the wrong section.
- Filler field docs recur in `smith-domain-run` and `smith-domain-session` ("Terminal classification…" ×5).
- `keep_facts` and `pop_fact` are copied in four inline harnesses (run, session, tools, root). A shared test mechanism could take them.
- `tests/session/tests/suite/jobs.rs:11` also relies on the root mount of finding 3.
- `D/domain.rs:416-469` scans `run_out` because the run's `Opened` fact lacks opening data. `smith-domain-run` could carry it.
- `Limits` literals in `tests/agent` and `tests/protocol-llm` follow finding 2.

## 6. Rule costs

No own-enum `if let` costs about 35 lines in `D/domain.rs:417-466` to find one variant, and similar lists in `T/facts.rs` and `operation.rs`. Worth it for a state machine; less obviously so for one-variant lookups. Otherwise the area's exhaustive matches read as the rules intend.

---

# 02: smith-domain-run and tests/run

Paths: bare names are under `crates/smith-domain-run/src/`; `world.rs`, `partner.rs` and `suite/` are under `tests/run/`. Nothing was built or run.

## 1. Scope read

- **Closely:** `run.rs`, `run/jobs.rs`, `boundary.rs`, `domain.rs`, `budget.rs`, `land.rs`, `agent.rs`, `call.rs`, `lease.rs`, `wind_down.rs`, `inbox.rs`, `charter.rs`, `host.rs`, `opening.rs`, `prepare.rs`; `tests.rs` lines 1-560, 1380-1700, 2740-3040, 3900-4120, 4880-5279; `tests/open.rs` (first third); `suite/messages.rs`, `reserve.rs`; `src/noisy.rs`, referees, `order_log.rs`.
- **Skimmed:** `facts.rs`, `outcome.rs`, `prompt.rs`, `delivery.rs`, `limits.rs`, `workspace.rs`, `schema_costs.rs`; `world.rs` (about half), `partner.rs` (top), `suite/simulation.rs`, `memory.rs`, `orders.rs`, `fuzzy/simulation.rs`.
- **By search only:** `src/host.rs`, the rest of `tests.rs`, `suite/jobs.rs`, `wind_down.rs`, `lease.rs`.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | No dead code to speak of. Restating field docs, 156 uses of the banned "actual", and 487 boilerplate lines in the world. |
| Organization | 3 | Small modules are right. `run.rs` (2,762), `tests.rs` (5,279) and `World` (64 fields) hold too much. |
| Taste | 3 | The machines and sealed types show care; bulk-generated docs and hand-chained `Option`s do not. |
| Naming | 3 | Mostly precise. Collisions on `wind_down`, `owner`, `Report`, `Phase`, and "push" against the design's "delivery". |
| Primitives | 4 | Lease, Call/Work, Inbox, Landing and Child are narrow and orthogonal. The budget gate has no home of its own. |
| Elegance | 3 | `land.rs`, `agent.rs` and `lease.rs` follow the problem; `ended`, `yielded` and the permit/reserve pair do not. |
| Economy | 2 | Roughly 1,500 test lines and 500 production lines could go. Most edits in `tests.rs` pay for retyped literals. |
| Complexity | 4 | The states are earned by the design. A few counts could be derived, and some layers only forward. |

## 3. Strengths

- `land.rs`, `agent.rs`, `lease.rs` and `wind_down.rs`: one machine, one file, `mem::replace`, exhaustive arms. `agent.rs` opens with a table that matches its code. A new contributor should read these first.
- Sealed values (`HostInput`, `HostAnswer`, `Receipt`, `Delivered`, `Marker`) keep bounds in constructors, so the run revalidates only authority.
- The world reuses skein's `Stage`, `Schedule`, `Ledger`, `Referee`, `Trace` and heap meter, and referees are themselves tested by mutating real observations (`suite/lease.rs`, `reserve.rs`).
- `tests/open.rs` shows the good test shape: named `HOST`, `PEER`, `FIRST`, and small event helpers.

## 4. Findings

1. **`ended` and `yielded` are inline, and `ended` skips `follow`.** Elegance, Primitives.
   `run.rs:1424-1534` (111 lines) and `:633-742` (110) hold many cells inline; `Working` in `yielded` nests six outcomes. `ended` replaces `run.state` twice (`:1469`, `:1512`). Its child path returns at `:1507` with no `follow`, though `:1465` may begin the wind-down and `:1470` may move the run to `Winding`.
   That breaks programming-model.md 5.4 ("applied in one place after every transition"). Alarms stay armed for the old state; `deadline` (`:1569`) and `park` (`:534`) call `unreachable!` on `Winding`. No test in this area sends a child's overflow end (unsure it is reachable; not run).
   Fix: split `child_ended` and `main_ended`, each ending in `follow`; one handler per `yielded` outcome. Same size, far clearer.

2. **Boilerplate docs in the world crate.** Cleanliness, Economy.
   `Scripted-world contract: domain/run.md, sections 13 and 14; ...` appears 241 times with 246 blank `///` lines (`world.rs` 104, `partner.rs` 85, `src/host.rs` 51), plus 29 "Count of X observed at this scripted boundary" (`partner.rs:218`). AGENTS.md: cite "at module or type level, not on every field".
   Fix: one citation in `src/lib.rs`; drop the rest.
   Effect: about 520 of 4,902 lines, no behaviour touched.

3. **`tests.rs` is one 5,279-line file of retyped literals.** Organization, Economy.
   152 tests, three `impl Harness` blocks, helpers scattered between tests. `Event::Ended {` appears 75 times, 28 multi-line, and 9 as `answered(...).1`. `Token::new(` 256 times, `Token::new(100)` 44. `drop(h.step(...))` 93 times. `h` (481 uses) and `harness` (312) split the file in two styles, as do `emitted` and `requests`. This breaks "A setup written once" and "Named fixtures and sizes".
   Fix: split by topic under the existing `tests/` (admission, landing, cancel, agents, host, messages, budget); add `ended(conversation, end, spend)`, `pass_checks(owner)`; adopt `open.rs`'s constants.
   Effect: about 700 lines, and no file over 1,000.

4. **`run.rs` holds four machines.** Organization.
   Beside run lifecycle it carries the host relay (`:2426-2691`, 266 lines, with its own stage), the budget gate (`:744-905`, `:953-1167`, about 375) and sub-agent admission (`:2029-2199`, about 170), while `host.rs`, `budget.rs` and `agent.rs` hold only types. `run/jobs.rs` already shows the way out.
   Fix: `run/relay.rs`, `run/gate.rs`, `run/child.rs`.
   Effect: about 800 lines moved; `run.rs` under 2,000.

5. **The budget decision is written three times.** Primitives, Economy.
   `Denied` reason: `run.rs:1035-1041` and `:1070-1075`. Ceilings: `completion_permit` through `ordinary_left`, `reserve` by its own arithmetic (`:1116-1131`). Three "what is left" sums: `Budget::remainder`, `ordinary`, `left` (`budget.rs:349`, `:407`, `:416`).
   `Failure::Budget {` appears 19 times in `run.rs`, mostly with `used: None, bound: None`, filled in later by `budget_answer` (`:2323`): a half-built value. `terminal_failure` (`:1406`) and `ending` (`:2347`) both map the overflows.
   Fix: `Ending::Over(Exhausted)` built whole at the answer; one `gate(run, now)`.
   Effect: about 80 lines.

6. **Near-copies in `run.rs`.** Economy.
   `may_finish` (`:1685`) and `host_active` (`:2601`) are identical, and five more sites match "is it Working" (`:850`, `:978`, `:1395`, `:2708`, `:2745`). `finish`'s change arm (`:1951-1974`) and `deliver` (`:2006-2026`) are one body. The name check repeats three times (`:1952`, `:2006`, `:2467`). `read` and `probed` differ in one call. `Change::owned_bytes` equals `change_bytes` (`outcome.rs:106`, `:364`).
   Fix: `working_main(&State) -> Option<Id<Conversation>>`, `begin_landing(..., finish: bool)`.
   Effect: about 90 lines.

7. **Production docs restate types and use a banned word.** Cleanliness.
   1,006 indented doc lines, against smith/AGENTS.md ("documented only when its name and type don't say it"). `/// Typed or byte-valued terminal classification in the enclosing boundary.` on `code: u8` (`boundary.rs:608`). `Spend`'s "Actual fresh input tokens." (`budget.rs:157`). `outcome.rs` has lines past 120 columns. Copies drift: `call` is "Root-issued" at `boundary.rs:218` and "Run-issued" at `facts.rs:78`. "actual" appears 38 times in production, 156 in the area (root AGENTS.md, 2).
   Fix: delete rather than reword.
   Effect: about 400 lines of the 1,006.

8. **Naming collisions.** Naming.
   - The design's wind-down is the finishing stage (run.md 9.1), but `State::Winding` and `fn wind_down` mean "ending decided, closing main" (`run.rs:179`, `:2271`), while the design's sense is `run.wind_down`.
   - `Call.owner` is the conversation's token and `Request::Check.owner` the run's, and both meet in `checked` (`run.rs:1347-1354`).
   - `host_name` and `host_run` name one value.
   - Two public `Report`s (`boundary.rs:909`, `outcome.rs:113`), two `Field`s, two `Found`s; exported `Phase` is an LLM timing phase while the private `Phase` is a conversation's.
   - "push" (181 uses in tests; `Stats.pushes`, `Pushing`) survives where the design says delivery, 58 mentions against 3 of "land".
   - `prepare.rs`, `land.rs` and `prompt.rs` say repository; the type and design say directory.
   Fix: rename `Winding` to `Ending`, `Call.owner` to `token`; retire "push".

9. **Test shape breaches.** Taste.
   - Bodies branching on rows: `tests.rs:2801`, `:2873`, `:2949` (a nested pair), `:4747`, `open.rs:81`.
   - Integer rows with `_ => unreachable!()`: `suite/lease.rs:60`, `messages.rs:52`, `reserve.rs:28`, `:132`, `wind_down.rs:116`.
   - Two stories in one test: 14 tests build more than one `Harness`; e.g. `tests.rs:1589`, and `:1385` runs two in one.
   - Delivery scaffolding left in: `assert_check_fact_pair` compares Debug text for "current-parent runtime RED" (`tests.rs:5174-5212`); `facts.rs:491` likewise.
   - `an_unread_charter_version_has_its_own_start_refusal` (`tests.rs:32`) tests that enum variants differ.
   - Garbled comment at `tests.rs:133`, where `Harness::step` silently adds a `Priced`.
   - `#![allow]` at `tests.rs:4917`, `tests/guide_room.rs:3`, `facts.rs:461`, against root AGENTS.md 2; `open.rs:2` uses `expect`.

10. **Suite setups retyped.** Economy.
    `messages.rs` opens four tests with the same five lines. `reserve.rs:36-49` and `:66-79` match for 12 lines. `settled` and `answers` are copied into `fuzzy/simulation.rs`. `crash_take(&mut domain, &env, &mut out, ...)` appears 18 times, a local harness (`simulation.rs:75`). The 44-line `Limits` literal is typed in `tests.rs:43`, `world.rs:189` and `memory.rs:54`, though `memory.rs` could use `..Settings::calm(0).run`. `= 1000;` (always) appears 45 times, against "Named fixtures and sizes".
    Effect: about 250 of 3,883 lines.

11. **`World` is one object.** Organization.
    64 fields and 27 public methods (`world.rs:513`); `impl World` runs some 1,700 lines. Host relays, io, referee feeds and run views all share it. Five `let mut out = Vec::new()` blocks in `deliver` (`:1869-1889`).
    Fix: extract `Io` (files, checks, looks) and `Relays`, each owning its fields.
    Effect: moves about 500 lines; shrinks `World::new` and its 64-field list.

12. **Hand-chained `Option`s.** Taste, Economy.
    `charter.rs:339-373` matches `Some`/`None => None` in a function returning `Option`, where `?` is allowed (10.2). `has_spare` and `has_room` repeat the threshold (`:566-598`). `inbox.rs` measures a rendered message four times and renders it twice (`:38`, `:64`, `:94`, `:214`).
    Effect: about 70 lines.

13. **Derived state and forwarding layers.** Complexity.
    `run.conversations` always equals `1 + children.len()` while main lives (`run.rs:147`, `:1472`). `WindDown::Settling.children` shadows `run.children`. `Event::Start` fields are typed four times (`boundary.rs:86`, `run.rs:249`, `:279`, `domain.rs:119`); `Event::Start(Start)` saves about 45 lines. Four `domain.rs` wrappers only forward (`:208-263`): re-export them.

14. **Second copy of referee plumbing.** Primitives.
    The per-run `judge` loop is byte-identical in `lease_referee.rs:117`, `wind_down_referee.rs:53`, `messages_referee.rs:75`, against smith/AGENTS.md.
    Fix: a keyed helper in `skein-world`'s `domain` module. Effect: about 18 lines.

## 5. Cross-area suspicions

- `Cap`, `CompletionFailure` and `Limit` (about 130 lines of `boundary.rs`) are unread by this crate outside it and recur in `smith-domain-session`, `smith-host-domain` and generated `smith-channel`.
- The full `Limits` literal also lives in `smith-domain`, `smith-host-domain`, `smith-local-domain`, `tests/agent`, `tests/host`, `tests/protocol-llm`, so a new limit touches about nine files.
- The "Contract:" field-doc style and "actual" probably recur in other step crates and worlds.
- `smith-domain-run` dev-depends on `smith-machine-world` for two tests (`tests.rs:5139`): an inline test reaching a world tier.
- Seeds are treated as a stored format ("zero preserves the earlier fixture's random draws", `partner.rs:133`; `let _ = pick(0, 64)`, `noisy.rs:105`).

## 6. Rule costs

- No closures: measure-then-write doubles each prompt body (`Text::measuring` 18 times), about 40 lines; pairwise-unique loops recur six times, about 40 lines.
- Split borrows: `let Domain { .. }` ten times in `run.rs`. Small.

---

# 03: smith-domain-session and tests/session

Sizes: production 7,649 lines (session.rs 3,788), inline tests 4,343 (151 tests), world 3,969, suite 6,376 (147 tests).

## 1. Scope read

**Closely:** every production file; design session.md; tests.rs lines 1-700 and samples past 2245; heads of tests/builtin.rs, line.rs, program.rs; world.rs 1-1100; src/recorded.rs 1-320; window_referee.rs; suite memory.rs 1-800, compaction.rs 1-260, recorded.rs 1-300, simulation.rs 1-170, budget.rs 1-110, orders.rs 1-140.

**Skimmed (outline, search, counts):** the rest of world.rs, translate, tickets, fixture, noisy, provider, fuzzy, suite jobs, wind_down, paths, referee, window_room.

## 2. Scores

| Criterion | Score | What keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | Code is clean; 236 stamped citations, boilerplate and wrong field docs, 49 lines of history prose in memory.rs, one dead match. |
| Organization | 2 | session.rs holds five concerns, not in reading order; every change here pays for it. |
| Taste | 3 | The handlers show care; the generated-looking docs and memory.rs do not. |
| Naming | 3 | Follows the design (window, kit, line, guard), but "call" and "spent" each mean two things. |
| Primitives | 3 | Window, Final, Jobs and the tools boundary are good; `Conversation` is a 43-field bag, and a run's tool name leaks in. |
| Elegance | 3 | The transition idiom and `conclude`/`follow` fit the problem; part of the state sits off the table. |
| Economy | 3 | ~750 of 7,649 production and ~1,250 of 14,688 test lines could go (9% of the area). |
| Complexity | 4 | The reservation machinery earns its place from the design; excess is local (F9, F10). |

## 3. Strengths

- **The cell discipline:** `mem::replace`, a handler returning `State`, then one `conclude`/`follow`/`settle` (session.rs:1231-1430). Keep this shape.
- **Small leaf primitives:** window.rs (113 lines), final_turn.rs (66), jobs.rs (102).
- **World-backed behaviour tests:** suite compaction.rs and budget.rs read like the design's stories, with short named helpers (`started`, `asked`, `inert`).
- **Exceptions are few and reasoned:** no `while` in production and three `#[expect]`s, all with reasons.

## 4. Findings

**F1. Contract stamp on every world field.** *Cleanliness.* `World contract: domain/session.md, sections 10 and 12; ...` appears 236 times: world.rs 174, src/recorded.rs 27, tickets 15, fixture 10, translate 7, noisy 2, fuzzy 1. AGENTS.md says a citation sits at module or type level, never on every field. Section 12 is "From temper", so 162 lines of world.rs alone cite the wrong section. world.rs also has 27 docs of the form "Count of X observed at this scripted boundary" (world.rs:339, 455), which only restate the field name. Fix: delete them, keep one line in each module doc. **About −450 lines, mechanical.**

**F2. session.rs is five modules in one file, out of order.** *Organization.* It holds the state machine, byte accounting (2539-2870), transcript validation (2944-3340 and again 3575-3715), retry policy (2464-2537) and turn telling (3342-3409).
- "Helpers." opens at 2146, yet entry points follow: `open` 2873, `delegate_ended` 3433, `delegate_withheld` 3514.
- `After` and `Bill` sit at 3413-3428; `#[cfg(test)]` hooks sit at 3720-3732, mid-file.
- `#[path = "batching.rs"]` (133-142) puts children beside their parent, and they reach in through 56 `super::` uses.

Fix: `session/` with `accounting.rs`, `restore.rs`, `retry.rs`, `tell.rs`, leaving ~1,800 lines of machine. tests.rs (3,214) splits by the same seams. Size neutral; drops the `#[path]` lines.

**F3. The session's state is not in `State`.** *Primitives, elegance.* programming-model 5.4: "no field is meaningful in some states only". `Conversation` has 43 fields (session.rs:169-228).
- `closing_tools` has 13 uses and is also held while **Yielded** (3533).
- `provider_credit` means Calling.
- `compaction.asking()` branches 17 times, so the machine is really State × Compaction × Final × Owed.
- `kit: Token::new(0)` is a placeholder, and `State::Closed` doubles as "opening" (2246-2248, 421-424).
- `Slot::Withheld` means two things (317).

The header table claims every other cell is unreachable (session.rs:32), but it omits withheld, narrowed, denied and overflowed; Tooling→Yielded (3534) is not listed. Fix: put the batch in the states that hold it (`Closing { tools }`, a parked Yielded), put compaction in the state or a `Calling` purpose, regenerate the table, and split `Recording` into spend and transcript cursor. −100 to −150 lines; this is the change that makes the rest cheaper.

**F4. memory.rs retypes its fixtures and narrates its past.** *Economy, test shape.* 1,514 lines for 10 tests.
- The `Spec` literal is typed 4 times (414, 535, 628, 756), `Opening` 4 times (401, 558, 674, 784).
- `2 + … + block + 2 + 26 + 23` appears 8 times (108, 484, 624, 726, 1009, 1346, 1399, 1465), breaking "named fixtures and sizes".
- The metered step block repeats (7 `meter.start()`).
- Rows are a bare tuple `(bool, u32, u32, u32, bool)` (148).
- 49 lines of prose about "original", "V1", "former" (118-120, 135) break the no-history rule.

Fix: one `spec(limits)`, one `opening(spec, history)`, a named `SPEC_BYTES`, one metered `Drive`, a `Seen` struct. **About −450 lines.**

**F5. Retry policy lives in five exhaustive `Failure` lists, one of them dead.** *Cleanliness, economy.* The lists are session.rs:1895, 1921, 2478, 2512 and compaction.rs:287. `failed` sends `SessionGone` and `context_failure` away (864-869) before `call_failed`, so the first match there (1895-1916) can never change anything, and those variants' arms below cannot be taken. Policy also sits inside a cell (863-881), against one handler per cell. Fix: one total `class(failure) -> Class { Gone, Context, Transient { floor }, TimeUp, Lasting }`. **About −70 lines.**

**F6. Every walk of a message retypes `Block`.** *Economy.* The arm listing all non-call variants appears 13+ times (9 in session.rs, 3 in builtin.rs). The text-or-`unreachable!` extraction appears 5 times (session.rs:3241, 3253, 3272; compaction.rs:131, 234). `call_id`, `result_id` and `current_block` are variants of one idea. Fix within the rules: `Block::ToolCall(ToolCall)` with one `Block::tool_call() -> Option<&ToolCall>` and one `text()`, then `let Some(..) else continue`. **About −150 lines**, and a new block kind then breaks one match.

**F7. Suite-wide fixture plumbing and row smells.** *Test shape.*
- `.as_slice().into()` appears 228 times; `Block::Text { text: b"…` 30 times; `said` lives only in compaction.rs, a similar `text` in budget.rs. No shared `bytes()`.
- Bool rows that drive branches: 10 sites, e.g. recorded.rs:142, 192, 371, compaction.rs:218-225 (`if block`). Rows that need different code are different tests.
- Multi-behaviour tests: `oversized_history_and_fresh_specs_are_refused_at_the_entrance` (recorded.rs:109) holds four stories.
- 36 tests exceed 40 lines; the longest is 133 (recorded.rs, `a_write_refused_as_too_large…`).
- 102 bare seeds (`World::new(4840)`) where orders.rs names `SEED`; bare `Token::new(31)` and `19`.
- `recorded::World` shares its name with `World`, so orders.rs aliases it; its doc says "version-three" (`VERSION` is 6) and "Bounded queue" over `Queue::uncapped()`.

Fix: a `smith_session_world::said/bytes/ask` fixture set; split the rows; rename to `Scripted`. **About −300 lines.**

**F8. Production field docs are stamped, some wrongly.** *Cleanliness, taste.* smith/AGENTS.md: a field is documented only when name and type don't say it. boundary, facts, llm and record carry 553 doc lines, with 24+ repeated sentences. Wrong ones:
- `Complete.prompt`: "First user message supplied by the opener" (boundary.rs:218).
- `Delegate.call`: "Typed checkout-tool call", on a `Token` (boundary.rs:267).
- `CompletionStarted.messages`: "Oldest-first conversation messages…", on a `u32` (facts.rs:75).
- `CompletionAnswered.calls`: "Maximum provider calls held at once" (facts.rs:89).

A paste glitch has `///` mid-line (boundary.rs:233, tests.rs:472). lib.rs:28-41 is fragments, not prose. Fix: delete what restates the type, correct the rest. **About −120 lines.**

**F9. Byte accounting's facts have several homes.** *Primitives, economy.*
- `compaction::new_bytes` (compaction.rs:267) must track `admit`'s sum (session.rs:2189) by hand.
- 128 and 32 are bare in session.rs:2173-2177, compaction.rs:42-43, facts.rs:59-63 and 256-263; 37, 17 and 4 in history.rs:143-144, 302, 314.
- Compaction's room sits inside `complete` (session.rs:2339-2354), with `history` computed and then discarded when `last_prompt` is set.
- `reserve_batch` walks the calls twice with near-identical loops (history.rs:156-183, 209-231), and its room sum recurs in builtin.rs:93-102.
- `outcome_cost` (session.rs:2798) belongs beside `Call::owned_bytes` in smith-domain-tools.
- `compaction::due` takes `&mut` only to write the derived `window.appended` (compaction.rs:80), which could be an argument to `check`.
- `reserve_provider` returns a bool that is always asserted (1998), so `provider_room`'s refusals are dead paths.

Fix: one `fixed_bytes(conversation)`, named consts, `compaction::room`, one batch walker. **About −120 lines.**

**F10. Spend and usage arithmetic is written twice.** *Economy.* `preview_completion` (577) repeats `used` (2079-2084), and its doc admits it. `checked_usage`/`sum_count` (2099-2117) and `checked_reported`/`add_reported_count` (2120-2138) are near-twins, one returning `Result`, one `Option`. Tests add two more copies (tests.rs:520, world.rs:2235). Fix: `Usage::plus`, `Usage::plus_reported` in llm.rs and one `accepted(conversation, usage) -> Result<Accepted, End>` both callers use. **About −60 lines.** `reserve_completion` also mutates outside `step` (lib.rs:66; caller smith-domain/src/route.rs:853); whether that belongs as an event is (unsure).

**F11. A run's tool name leaks into the session.** *Primitives.* Design section 1 says a session knows nothing of runs, yet `b"wait"` decides behaviour at session.rs:1824, 3154 and 3664. Fix: the opener names the withheld call in `Opening`, with an `Origin`, as `answered` already does. Also `Event::Answered` and `AnsweredAndYield` are twins (domain.rs:129-148); one `after` field removes ~15 lines. About −40 lines in all.

**F12. Overloaded names.** *Naming.*
- "call" means both an LLM completion (`call`, `State::Calling`, `cancel_call`, `call_failed`, `Waiting.call`) and a tool call (`Calls`, `call_id`, `tools::Call`), though the design says *completion*.
- `spent` is a `u64`, a `Dimension` and a function (2303).
- Two "budgets": `Conversation.budget` and `Recording.budget`.
- `Tokens` holds bytes by its own doc (window.rs:8).
- `Yield` is an enum, an event, a request and an `After` variant.
- `KIT` is an opaque constant; `held` is both function and list.

Fix: rename `call`→`ask`, `Calling`→`Completing`; `Tokens` plus a `Bytes`. Size neutral.

**F13. Local near-copies.** *Economy.*
- `.iter().nth(index)` rebuilds iteration three times (builtin.rs:54-61, 176-184, 198-206), because skein-lib's `Map` has no `iter_mut`: add it in skein.
- `Ready` ignores `Set::pop_first` (session.rs:379-381, 399-402).
- `tell`'s two matches with a skip-`unreachable!` (1346-1408) can be one.
- `close`/`expire` are copies (499, 1023); tell-then-yield appears three times (1729, 1769, 2026).
- `_env` is unused (1085); `finish_closing` only forwards (3569); `{ let Recording { … } }` blocks are odd scoping (1452, 1991, 2304).

**About −80 lines.**

**F14. Small rule breaks.** `#![allow]` in tests/builtin.rs:1 and tests/line.rs:3, where AGENTS.md demands `#[expect]`. "actual" appears on 15 production lines (e.g. session.rs:2644-2690) and 59 test lines, against the emphasis rule; "invented" and "fabricated" at boundary.rs:128, session.rs:677, 695 argue with a past. `#[path]` in tests.rs:3, 3158, 3213 is unnecessary.

## 5. Cross-area suspicions

- `tests/tools` carries the same stamps: 123 (world.rs 80, translate 12, fixture 10, memory 9, calls 7).
- `smith-domain/src/route.rs`: the caller side of F10's preview and reserve.
- Exhaustive `Failure` lists probably recur in `smith-domain` and `smith-protocol-llm` (not checked).
- `skein-lib`: `Map::iter_mut`; a `u64` length helper (below).
- Tech-debt "Volume and churn" shows up here as tests at 1.9× production, and F4 and F7 as its "repeat one another".

## 6. Rule costs

- **No `as`:** 30 `u64::try_from(size_of::<T>())` and 39 `u64::try_from(x.len())` in production. A `skein_lib::bytes::len` and a block-cell constant function would absorb most; session's private `len` (2862) goes unused by compaction.rs (14 sites).
- **No `_`:** the 20-plus-arm `Outcome` list appears three times (batching.rs:51-76, jobs.rs:36-61, session.rs:2828-2844), ~70 lines. Worth it; noted only.

---

# 04: smith's wire formats

## 1. Scope read

**Closely:** the three schemas; `smith-channel/src/kinds.rs`, both `lib.rs` of charter and transcript, `patches.rs`; `smith-events` `records.rs` (first half), `write.rs` (first 1,000 lines), `read.rs` (about 1,400), `limits.rs` (head and tail), `words.rs`, `tests.rs`; `smith-protocol-channel` `component.rs`, `notices.rs`, `calls.rs`, `credentials.rs`, `limits.rs`, `translate.rs`, `result.rs`, `facts.rs`, `live.rs`; `tests/codecs` `drift.rs`, `channel.rs`, `charter.rs`, `support/patches.rs`, `suite/patches.rs`, `suite/events.rs` (about 40%), `support/events.rs` (head, tail, shape); `tests/channel/src/lib.rs` (about 60%), `stories.rs` (first 200 lines, helpers, counts); `tests/transcript-patches` all.

**Skimmed:** `turn.rs`, `transcript.rs`, `charter.rs`, `smith-protocol-channel/src/tests.rs`, `tests/jobs.rs`, `memory.rs`, `transcript.rs` (codec suite), `qa_patches.rs` (first 160 lines), `events_max.rs`, `composed.rs`, `referee.rs`, the fuzz files. Design: `channel.md` 1-2, `events.md` 1-3.2, 6, 8, `transcript.md` 12, `README.md` names and versions; `limits.md` only by search.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | Codec crates are spotless; `smith-events` carries 73 dead parameters, 105 filler doc lines and a dead reader mode. |
| Organization | 4 | `schema/`, `golden/`, `generated/`, one test crate per concern. Held back by `translate.rs` and test modules mid-file. |
| Taste | 3 | The schemas show care; the events crate and its fixtures read as a generator's output kept by hand. |
| Naming | 4 | Schemas follow the design closely. Local clashes remain: `CallName`, `Overflow`, `Error` variants. |
| Primitives | 3 | Schema-first codecs are the right primitive; `Capture` and `Unknown` overlap policy and vocabulary, and no primitive names a kind or sends a frame. |
| Elegance | 3 | Schemas fit the problem; `drain` and the five per-record hierarchies do not. |
| Economy | 2 | One event field touches about eight places; 20% of `smith-events` and half its fixtures can go. |
| Complexity | 3 | Little speculation, but much accidental weight: `drain` reaches indent 15, and bounds are computed by zeroing structs. |

## 3. Strengths

- **Schema as the wire's single source.** The three `.schema` files (379 lines) generate 24,000 lines. Their comments say what the codec cannot check. `drift.rs` holds code, goldens and schema together. Keep this.
- **`smith-transcript`'s pre-baseline patch set** (`src/patches.rs`, `tests/transcript-patches`) is small, data-only and matches `transcript.md` 12 exactly, with the fixture owner made by skein's command.
- **Exhaustive translation.** `translate.rs`, `turn.rs` and `facts.rs` are long but honest: total matches with no `_`, and `facts.rs:13-61` says what the channel drops by naming it.
- **Fuzzing and refereeing reuse skein** (`assert_replays`, `skein_world::domain`); `referee.rs` sees boundary events only.

## 4. Findings

**1. `smith-events` is five hand-kept parallel hierarchies** (Economy, Elegance). Every record exists in `records.rs`, `write_*`/`encode_*`, `read_*`, `validate_*` (59 functions, 665 lines of `write.rs`) and `size_*`/`heap_*` (84 functions, 801 lines), then again in `value_*` and `max_*` fixtures. The code shows a generator's marks with no generator in the tree: `limits.rs:920-977` (`{ let candidate = {...}?; ...}`), 105 `The \`x\` classification.` lines in `records.rs`, `let mut length = 0_u64; length = length.checked_add(...)` runs, and bare record overheads (`limits.rs:23-41`, retyped at `:56-74`; each is 53 plus the type name's length). Counted repetition: 63 `Some(item) => Some(...)` in `read.rs`, 17 `[Token::Null] => None`, 94 `if let Some` in `write.rs`, 17 `None => json.null()`, 76 `checked_mul(6)?.checked_add(2)`, 26 near-copies of the `text(..., limits.string.max(64))` then re-check dance (`read.rs:342-355`), with stray variants `.max(6)`, `.max(9)`, `.max(12)` and none at `:868`.
*Why:* a new event field is about eight edits, and the sizing, the most error-prone part, is hand arithmetic. *Fix, smallest first:* helpers `optional_text`, `optional_unsigned`, `nullable_unsigned`, `put_optional_*`, `word(tokens, limits)`, `string_size(limits)`; one `spelled(value)` per word enum, shared by writer and validator; a list cursor for the 7 `read_list_*`. Then decide whether the vocabulary belongs in a schema with a JSON-line backend in `skein-codegen` (unsure of fit: capture rules and nullable fields would need schema words). *Size:* about 1,300 of 6,372 lines with helpers; about 5,000 become generated if the backend fits.

**2. Dead parameters and a dead reader mode** (Cleanliness, Complexity). `_capture: &Capture` on 38 `write.rs` functions and `_capture`/`_limits` on 35 in `limits.rs`, none read. `Reader::with_session_ended_aborted` (`read.rs:52`) has no caller in any tree; its doc cites "historical benchmark faces" while `events.md` 6 says no older version is read. It forces `SessionEnded.aborted: Option<bool>` (`records.rs:803`), a check (`write.rs:1362`) and an `expect` (`:1677`). Also `edge_room(_limits)` (`smith-protocol-channel/src/limits.rs:116`) ignores its argument at 7 call sites, and two doc comments repeat. *Fix:* delete all; `aborted: bool`. *Size:* 73 parameters, about 30 lines.

**3. `Capture` is both a recorded field and the write policy; `Unknown` leaks into writing** (Primitives). `write(record, capture: &Capture, ...)` accepts `Capture::Unknown`, and the sites disagree: `write.rs:598,637` treat it as `None`, while `:1947` (`*capture != Capture::None`) lets `input` through. There are four spellings of the decision. Separately, `smith-agent-service/src/events.rs:238` writes `NoticeKind::Unknown(b"not_contained")`, a kind `events.md:398` documents and the enum lacks: the reader's fallback is being used as vocabulary. *Fix:* a `Policy` enum with no `Unknown`, and `shows_calls(policy)`/`shows_everything(policy)`; add `NoticeKind::NotContained`. Consider refusing `Unknown` at write; much of `validate_*` exists only to bound it. *Size:* clearer more than smaller; with refusal, about 190 validate lines go (unsure).

**4. `Component::drain` is not a state machine** (Elegance, house rule 5.4). `component.rs:675-930`, under `#[expect(too_many_lines)]`: one 255-line `match` on `Some(Event::Body { kind: 0x0101, body }) if self.phase == ...` guards. An out-of-phase kind falls into the Start arm by default. The Start path reaches indent 15 (`:839-881`); 11 arms read `Err(_) => self.refuse_rules(below)`, 8 repeat `Request::Read`. There is no `mem::replace`; 13 `self.phase != Phase::Admitted` checks sit in send methods; `window`, `live`, `turns`, `ended` and `cancelled` are meaningful in some phases only. `refuse_rules` sends "invalid Start" for every breach (`:934`). *Fix:* one handler per kind returning `Result<(), Broken>`, one place that refuses and one that demands the next read; move phase-held fields into `Phase` variants, per 5.4. *Size:* about 60 of 255 lines, indent from 15 to about 4. The module table in `lib.rs:8-16` should then be rewritten; it omits most kinds.

**5. Kinds, reasons and versions are bare numbers; every send retypes framing** (Primitives, Economy). 15 sites repeat measure, `Writer::new`, `encode`, `frame_writer(0x0106, ...)`, `put`, `finish` (6-7 lines; `component.rs:188-195` and 14 more); 15 repeat the `down`/`drain`/`fire` tail. Kind numbers: 20 in `V5`, 18 in the agent half, 23 in the host half, none named. Reasons: `RULES = 256` twice (both `component.rs`), `reason: 2` (`:696`). Versions: `highest.min(5)` (`:683`), `version: 5` (`kinds.rs:115`), `[0, 6]` (`transcript.rs:49,133`, the first redundant), `[0, 4]` (`translate.rs:296`), while `FORMAT_VERSION` is hand-copied from each schema's header and hung on `KindRule`, a table row (`kinds.rs:157`). `decode_declared`'s `reader.remaining()` check (`result.rs:14`) is dead: generated `decode` already refuses trailing bytes. *Fix:* named kinds and reasons in `smith-channel`; generated `VERSION` and `encoded()` from `skein-codegen`; a private `Component::send(token, frame, ...)`. *Size:* about 130 of 1,174 lines here, and similar in `smith-host-protocol`.

**6. Event fixtures are stringly typed** (Taste, section 10 "Rows carry data"). `support/events.rs` (2,181 lines): `value_capture(name, spelling)` returns a default unless `name == "Capture"` (20 such guards, `:14-23`), 159 calls pass string pairs, 13 pass `("", "")`, 13 builders ignore both. `samples_0` to `samples_14` are numbered chunks of 6-line rows. `events_max.rs` (507 lines) rebuilds about the same 40 values, with 8 `_limits` unused. *Fix:* base builders taking a fill (`{ text, count }`), rows as `("session_started_none", session_started(Capture::None), Capture::None)`, with sample and largest sharing one builder set. *Size:* about 1,300 of 2,779 lines.

**7. Event tests break section 10** (Economy). `suite/events.rs` and `src/tests.rs` hold 80 `.replace(` edits of recorded lines ("A recorded stream edited as records"). `response_tool_choice_...` (`events.rs:89-128`) checks about ten behaviours in 40 lines. 21 capture loops and 6 retyped bound assertions could be one `round_trips_within_bound(record)`. Also `smith-events/src/tests.rs:1` is `#![allow(...)]`, which root `AGENTS.md` forbids, and a test sits mid-production at `read.rs:752`, as do `edit_tests` and `found_tests` at `transcript.rs:578,600`. *Fix:* parse, edit the record, count the edit; split by behaviour. *Size:* about 250 of 1,085 lines, more readable than smaller.

**8. The channel world pays for one `Peer` playing both halves** (Primitives, section 10 "A setup written once"). `tests/channel/src/lib.rs`: 28 `if let Half::...` that silently do nothing on a mismatch; `Peer::host` and `Peer::agent` retype 15 fields (`:165-213`); 8 `send_*start*` variants. `stories.rs` spells the charter golden's path 27 times, mostly in the same opening lines; `admitted_channel` (`:308`) and `admitted_notice_world` (`:861`) are identical; `CHARTER` is a constant only in `fuzzy/stream_cuts.rs:11`. `RefusedHere(2)`, `(256)` are bare; `an_accepted_report_keeps_its_fields_in_charter_v2` (`:159`) names a dead version, as do `..._version_three_...` in `smith-protocol-channel/src/tests.rs:223,253`. *Fix:* `HostPeer` and `AgentPeer` around a shared `Pipe`; one `StartAt` struct; `World::started()`, `World::admitted()`. *Size:* about 300 of 2,900 lines.

**9. Hand copies of what skein and codegen know** (Economy). `smith-protocol-channel/src/limits.rs:123-148` retypes `skein_channel::valid`'s `capacity_bound` (6, 4, 14, 262, 8), though `valid` is exported. `start_cells`, `charter_cells`, `turn_cells` (`:178-272`) zero 75 named fields to reach a cell count, and a new text field silently inflates the sum. `memory.rs:10-82` retypes 65 limit fields. `schema_for_start` (`:158`) patches kind `0x0100` after the build. *Fix:* call `skein_channel::valid`; `skein-codegen` emits `Limits::NONE` and `cells_bound`. *Size:* about 150 lines.

**10. `translate.rs` and the crate's edges** (Organization, Naming). `translate.rs` (917 lines) holds Start context, delivery decoding, `Endpoints`, charter decoding, result encoding, answers, and `host_path` (`:802`), a path normaliser with 13 users. The crate is "the agent's half" (`README.md:171`) yet `smith-host-protocol` and `smith-local-protocol` import its shared translation. `Error::ResultCapacity` (40 sites) and `Error::Answer` name their first user, not their meaning; 5 `impl From` (`limits.rs:70-98`) and a `TryFrom` (`credentials.rs:45`) break "no `From` conversions" with no written reason. Transcript's `CallName { name }` clashes with the channel's and the design's call name; `Overflow` needs `as BudgetOverflow` in `smith-channel/src/lib.rs`. `credentials.rs:20` says `Program` is refused, but `smith-agent-service/src/program.rs:271` installs it; `channel-v5.schema:19` cites a task. *Fix:* split into `start`, `answer`, `endpoints`, `delivery`; move the shared translation to a named crate (needs the designer); plain error variants. *Size:* neutral.

**11. Drift and golden tests repeat** (Economy). `drift.rs:8-52`: three identical tests; the write path, `SKEIN_CODEGEN_WRITE`, is retyped in each tree. Seven "decodes at measured length" loops across `channel.rs`, `charter.rs` and `transcript.rs`. `smith-charter` has an unused `skein-codegen` dev-dependency. `qa_patches.rs` rebuilds `support/patches.rs`'s patcher and three constants in a second binary. *Fix:* `assert_family(crate, schema, version)`; the check-or-write helper in `skein-codegen`. *Size:* about 90 lines.

## 5. Cross-area suspicions

- `smith-host-protocol/src/component.rs` shows the same bare kinds (23), `RULES`, framing and a local `hex`; smith has five hex digit helpers (both `component.rs`, two in `smith-protocol-llm`, `words.rs`), a candidate for `skein-lib`.
- `tests/agent-process` holds 57 lines with bare kind numbers and 13 `frame_writer` calls.
- The drift check-or-write path recurs across `skein/tests`, `jig/tests`, `temper`.
- `skein-codegen`: constructors of bound-free records are fallible, so every `new(...)?` on `BudgetTurns` or `CallName` pays for a failure that cannot happen.

## 6. Rule costs

- **"A variant holds one record, or nothing"** (`codec.md`, 3) costs 13 wrappers in the channel schema (`PriceRateValue`, `BudgetTurns`, ...) and more in transcript, about 4 lines each in `translate.rs` and again in the host: roughly 100 lines, plus inconsistent names (`*Value`, `PathCap`, `OwnedOutcome`). Allowing a variant to hold an enum or scalar pays this back.
- **No closures or generics** forces `manual_map` expectations (6 in this crate) and per-type list readers; helpers recover most of it.

---

# 05: smith-protocol-llm and tests/protocol-llm

Paths are relative to `smith/`. `PL` = `tests/protocol-llm`; `C` = `crates/smith-protocol-llm/src`.

## 1. Scope read

- **Closely:** every production file in `C`; `C/tests.rs` (first part and test list); `PL/src/wire.rs`, `adapter.rs`, `translate.rs`, `lib.rs`, head of `world.rs`, head of `pool_peer.rs`; suite files `guard`, `pool_waiting`, `referee`, heads of `budget`, `brief`, `client`, `adapter_boundary`, `production_render`, `production_tools`, `component_boundary`, `native_root_restore`, `root_memory`; `llm.md`; `limits.md` 3.2, 3.4, 5, 11.
- **Skimmed:** the rest of `world.rs`, `referee.rs`, `script.rs`, `host_referee.rs`, `messages_referee.rs`; the middles of the larger suites. All of `PL` was measured by script: line diffs against other test crates, duplicate functions, test counts.

## 2. Scores

| Criterion | Score | What keeps it from the next score up |
|---|---:|---|
| Cleanliness | 3 | Production reads well but carries client-era docs; the test crate has 70 citations of a missing file and "actual" on every third line. |
| Organization | 2 | The crate's modules are good; the test crate is a copy of `tests/agent` with domain stories in a protocol world. |
| Taste | 3 | `render`, `decode` and `contract` show care; a module-wide lint exception and test-only public API do not. |
| Naming | 3 | The design says component and dialect; the code says adapter, shared, application, native. |
| Primitives | 3 | `Component`, `decode`, `render`, `schemas` are right; `ResolvedCall` and the pure terminals overlap `Component`. |
| Elegance | 3 | Each call is decoded, copied, compared with itself and tokenized again; schema JSON is built four ways. |
| Economy | 2 | About half of `PL` could go or move. |
| Complexity | 2 | Production validates its own output; the world models a client lifecycle production gave to skein's pool. |

## 3. Strengths

- **`render.rs`:** one exhaustive cell per outcome, and one function serves measuring, cutting and writing, so live and resumed bytes cannot drift. `schema_costs.rs` likewise measures the production writer instead of pricing by hand.
- **`decode.rs`:** total, with typed field-named problems and no state; `contract.rs` builds schemas cleanly on skein's `Encoder`.
- **`PL/tests/suite/guard.rs`:** the model test file: named sizes with their derivation, one behaviour each, literal expectations. `suite/referee.rs` gives the stable-prefix referee real negative controls.
- **Reuse of skein's kit:** `Exchange`, `Referee`, `Stage`, `Ledger` and `Meter` are all used; no second allocator or schedule.

## 4. Findings

**Is the ratio earned? No.** Of 16,920 test-crate lines: 4,310 are identical to `tests/agent/src`; 5,038 are domain stories (finding 4); about 6,300 are suites on this crate and its seam, the part proportionate to 5,250 production lines; about 1,200 are unique support. A defensible test crate is 8,000 to 9,000 lines, about 1.6:1. Inline tests (about 1,070 lines) are proportionate.

**1. The world is a copy of `tests/agent/src`.** *Organization, Economy.*
- Line-level diff, identical / `PL` lines: `world.rs` 1,779/2,078; `wire.rs` 440/562; `referee.rs` 418/543; `host_referee.rs` 380/384; `script.rs` 318/318; `messages_referee.rs` 294/309; `translate.rs` 244/357; `world/jobs.rs` 198/198; `limits.rs` 163/169; `fixture.rs` 45/45. Total 4,310 of 5,020; `lib.rs:1` still calls it "the copied agent".
- The typed backend (`world.rs:278`, 40 mentions) has three users, and all three tests also exist in `tests/agent`: `two_priced_models…` (`budget.rs:418`, three differing lines), `ordered_sections…` (`brief.rs:267`, identical), `mixed_git…` (`workspace.rs:351`, identical).
- `smith/AGENTS.md` forbids second copies of referee plumbing, and every change to the domain's boundary is paid twice.
- **Fix:** delete those three tests and `Backend::Typed`; depend on `smith-agent-world` with the LLM backend as a seam, keeping only `wire`, `pool_peer` and the native-request half of the referee here.
- **Size:** about -4,000 of 5,537 support lines.

**2. The world bypasses `Component` and speaks a second finish grammar.** *Primitives, Elegance.*
- `llm.md` section 10 says the world runs "this component". `Wire::prepare` (`wire.rs:398`) calls `adapter::prepare` (`PL/src/adapter.rs:56`), which drives `client::Client` directly. Guard, capture, identity, grants and `schemas_for_contract` are not on `Wire`'s route; only `pool_peer`, `component_peer` and `component_boundary` reach `Component`, with no root domain. (Composition elsewhere: not checked.)
- `wire.rs:517` `resolutions` reimplements `Component::resolve` (`component.rs:664`).
- `wire.rs:535` sends `finish` and `deliver` to the fixture decoder `translate.rs:204`, which reads `{"report":…,"source":…}` (`budget.rs:24`), a form production's `decode_finish` refuses (it requires `form`, `contract.rs:268`). So production's finish path never runs in a root story; only `production_contract.rs` calls it directly.
- `Composition` (`wire.rs:81-359`: `Binding`, `Observed`, `retained_at_start`, deferred close) models per-call close rights that skein's connection pool now owns.
- **Fix:** back the world with `Component` plus a byte peer, growing `pool_peer`, whose core is already that; then remove `adapter.rs`, `Composition` and the fixture grammar.
- **Size:** about -700 lines of support, and it enables finding 3.

**3. Production keeps a caller-handoff surface only that bypass uses.** *Complexity, Primitives.*
- `resolve` copies every call's name and arguments into `ResolvedCall` (`component.rs:714`) so that `take_resolution` can compare them with the block they came from (`completion.rs:248`), then tokenizes the input again (`completion.rs:220`; first at `decode.rs:45`).
- Dead in production:
  - the `hosted` scan, `component.rs:694-708`: `offered` already `continue`d, and `prompt.rs:294` refuses a shared name;
  - the owner checks at `completion.rs:27` and `failure.rs:25,78`: the `active` key is that owner;
  - the checks of the `application` list in `inventory`, `required`, `fixed_name` and `offered` (`prompt.rs:276-440`): `prepare_component` just generated that list. The served and limit checks there stay;
  - `finishing()` and `delivering()`'s literal schemas (`tools.rs:139-155`), always overwritten by `schemas_for_contract`; `finishing()` is the fixture grammar living in production.
- `Endpoints::apply` (`endpoints.rs:174`) is called only from `suite/endpoints_grants.rs` and duplicates `component.rs:446-455`.
- `prepare_prompt`, `prompt`, `completion`, `failed`, `cancelled`, `refusal` and `schemas` have no caller outside the crate but tests, as `lib.rs:12` admits. `valid` runs twice per call (`prompt.rs:24,43`).
- `served_unique` (30 lines) is `served_name(a) == served_name(b)`.
- **Fix:** make decoding a step of completion translation; shrink the public surface to what `smith-protocol-program` and the service use.
- **Size:** about -300 production lines, -400 test lines (`adapter_boundary`'s owner, attestation and inventory tests).

**4. Domain stories live in the protocol world.** *Organization, Economy.*
- `root_memory.rs` (1,742 lines, 2 tests) measures the root domain. `limits.md` section 11 places memory "in the agent's simulated world", and section 5 says "no doubled formula". The file hand-prices domain types (`prompt_bytes`, `turn_bytes`, `copy_prompt`) and pins another crate's text (`SUMMARY_BYTES = 415`, `:1217`). Any domain field change re-preserves it: the tech-debt "volume and churn" entry in one file.
- `budget.rs` (808) is spend accounting, absent from `llm.md` section 10's stories. Most of `brief.rs` (1,067 with `open.rs`), `workspace.rs` (710), `native_root_restore.rs` (534) and `root_wire.rs` is domain behaviour run in two dialects.
- **Fix:** keep what is visible on the wire: prefix across completions, replay, absent usage fields, descriptions, UTF-8 copied into an edit. Move the rest to `tests/agent`, dropping what it already holds.
- **Size:** about 5,000 lines down to about 1,200 here; perhaps half of the remainder is deleted rather than moved (unsure).

**5. Fixtures retyped.** *Economy; section 10, "a setup written once".*
- `fn limits()` byte-identical in 5 suites (`client`, `adapter_continuation`, `production_contract`, `_render`, `_tools`), with variants at `adapter_boundary.rs:20` and `translate.rs:194`.
- `ComponentLimits` written 3 times, about 46 lines each (`pool_peer.rs:15`, `component_boundary.rs:14`, `component_peer.rs:20`).
- `llm::Prompt {` 25 literals in 12 files; `Receiving {` 14 in 10; `run::Charter {` 12 in 4; `call`, `calls`, `says`, `part_bytes` 3 times each.
- Three `Component` harnesses (`Peer`, the local one in `component_boundary`, `run_story`).
- **Fix:** a `fixture` module (limits, receiving, prompt, charter) and one peer.
- **Size:** about -700 lines.

**6. Text assembly is hand-rolled eight times.** *Economy, Elegance.*
- Measure-then-`Writer` at `guard.rs:85`, `prompt.rs:186,215`, `failure.rs:100,180,237`, `tools.rs:110`, beside two sink types (`render.rs:60` `Sink`, `descriptions.rs:10` `Text`).
- `prompt.rs:228-257` hand-writes decimal digits, which `skein_lib::Decimal` already gives (`failure.rs:181`).
- The cut marker is written twice with a bare `17` (`render.rs:41,520`; `CUT_MARKER_BYTES` is 17 + 20 digits). `hex` twice (`failure.rs:174`, `render.rs:149`); `schema_error` twice; `size` twice; `finish_schema` and `deliver_schema` are the same 12 lines.
- Schema JSON is built four ways: `Text` fragments, `Writer` in `sub_agent`, literals, `Encoder`.
- `render_bound` (`limits.rs:125-153`) uses unnamed room (192, 288, 96), in a limit the design calls derived.
- **Fix:** add a `concat(parts)` to skein-lib beside `Writer`; move descriptions to `Encoder`; one marker function.
- **Size:** about -200 lines.

**7. JSON member walking belongs in skein-json.** *Primitives, Economy.*
- `decode.rs:267` `field`, `:292` `member`, `:398` `replacement_count`, `:537` `families`; `contract.rs:361` `required_value`, `:380` `value_end`. `Document` offers only `token` and `text`.
- Object admission is written four times: `decode.rs:45-64`, `contract.rs:247-265`, `prompt.rs:553`, `completion.rs:220,264`.
- **Fix:** `Document::member` and `value_end` in skein-json; one `admit` here.
- **Size:** about -150 lines. `if match …` at `decode.rs:59` goes too.

**8. Comments break root `AGENTS.md` section 2.** *Cleanliness.*
- "actual": 41 in `C`, 555 in `PL`, including test names; AGENTS.md names that word.
- `scratch/client.md`, cited 70 times, does not exist in the tree.
- `Contract:` 182 times in `PL/src`, mostly per field; "Scripted-world contract: …" appears 68 times, verbatim.
- History: "migration 05s5" (`translate.rs:4`), "later migration work" (`lib.rs:20`), 41 "copied", "migration" or temper-revision references.
- Production docs describe a dead shape: "callback owner", "the actual Client remains owned" (`failure.rs:16,76`), "caller handoff" (`completion.rs:18`).
- **Fix:** one mechanical pass.
- **Size:** about -400 comment lines.

**9. `component.rs` local rules.** *Taste, Cleanliness.*
- `#![expect(clippy::manual_let_else, clippy::single_match)]` (`:11`) is module-wide, with a reason that explains nothing; 10.2 allows `if let` on `Option` and `Result`.
- `component.rs:667` is an own-enum `let … else`, which `smith/AGENTS.md` forbids; a new tool-call block kind would pass silently.
- `route`'s three terminal arms repeat their shell (`:568-622`).
- `local_failure` always says "credential or endpoint unavailable" (`:755`), even for pool memory or a duplicate owner.
- **Size:** about -40 lines.

**10. Test shape.** *Section 10.*
- Dialect is a bare index: `index in 0..2`, `configuration(index)`, `if index == 0 { Some(0) }` (`brief.rs:424,543,778,810`; `workspace.rs:204,233`).
- A row branch at `budget.rs:463-467`.
- `tests.rs:80` is a random test of one case, `for case in 0_usize..1`, with `case % 8` left in; `limits.md` section 11 wants random outcomes at random caps.
- Bare numbers: `peer.start(owner, 60, 60, 300)`, `peer.complete(100)`, `most: 4097`, `next: if 2 + 2 < 9` (`production_render.rs:61`).
- Long stories assert on the way; `call_attestation_checks_position_name_literal_input_kind_uniqueness_and_consumption` is many behaviours by its own name.
- **Fix:** a named `Dialect` fixture; rows that carry their expected value.
- **Size:** small, mostly clarity.

**11. Vocabulary.** *Naming.*
- `adapter` (22 in `C`, `ComponentLimits.adapter`) appears nowhere in `llm.md` or `limits.md`, which say component.
- `shared` aliases `skein_llm`; "application" means smith's tools; `bounds: Receiving` sits beside `Limits`. `decode::Kind`, `ToolKind` and `tools::Tool` are three tool identities; `tools::Tool` to name is tabulated four times (`prompt.rs:398`, `decode.rs:32`, `tools.rs:19,82`).
- Test files sort by migration era (`production_*`, `adapter_*`, `root_*`, `native_*`); "both native forms" means each dialect.

**Total, net of overlap** (the comments and retyped fixtures sit largely in lines counted above): about -6,000 lines deleted and -2,000 moved from `PL`; about -700 from production.

## 5. Cross-area suspicions

- `tests/agent/src/wire.rs` (509 lines) is not declared in that crate's `lib.rs`, and the crate's normal dependencies lack `smith-protocol-llm`: an orphan file. Check `tests/agent`.
- `tests/run` has its own `messages_referee.rs`; `run/src/world.rs` shares 25 lines with `PL/src/limits.rs`.
- `tests/session/src/translate.rs` shares 31 lines with `PL/src/translate.rs`, and may hold its own fixture finish grammar.
- `ToDomain`, `FromDomain` and `Measure` carry variants only `smith-protocol-program` uses (`BuiltinResult`, `ReadVersion`, `Retried`, `Yield`); that boundary may belong to neither backend's crate.
- `Document` member lookup is probably reimplemented elsewhere; check the channel codec.

## 6. Rule costs

Without closures or `?`-with-`From`, `.or(Err(Error::Limit))?` and `.checked_add(…).ok_or(…)?` account for much of `prompt.rs`, `completion.rs` and `failure.rs`, perhaps 10% of their lines. A checked-sum helper in skein-lib would absorb most of it; the rule itself is fine.

---

# 06: smith-protocol-machine, smith-protocol-program, tests/machine

Paths are relative to `smith/`; `M` = `crates/smith-protocol-machine/src`, `P` = `crates/smith-protocol-program/src`.

## 1. Scope read

Size: about 11,500 production lines and 11,300 test lines.

- **Closely:** all of `M` production; `M/tests.rs` to line 720, `tests/routing.rs`, `tests/deadlines.rs`; `P/backend.rs`, `start`, `conversation`, `stop`, `session`, `turns`, `stream`, `replies`, `tail`, `calls`, `builtins`, `memory`, `permission`; `P/command.rs`, `mcp.rs`, `mcp/json.rs`, `relay.rs`; `tests/machine/src/*`; `stories.rs` (first 300 lines), `simulated_jobs.rs`, `fuzzy/worlds.rs`.
- **Skimmed:** `backend/files.rs`, `policy.rs`; heads of `stop_tests`, `review_tests`, `builtin_tests`, `lifecycle_tests`, `P/tests.rs`, `mcp/tests.rs`.
- **By search only:** `measures`, `native`, `intake`, `redaction`, `recovery`, `diagnostics`, `compact`, `source`, `status`, the other test files.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | Module docs and prose are good; bare slot numbers and sizes, and stale test comments, recur (F10, F14). |
| Organization | 3 | `M` is well split; `P` scatters one struct over 15 files, and `start.rs` holds the io router. |
| Taste | 3 | `Guide`, `Removing`, `Create`, `Server` have care in them; `Backend` and the older tests do not. |
| Naming | 3 | Design vocabulary holds, but the stop family and Backend's cursors are hard to tell apart (F13). |
| Primitives | 2 | `Backend` and `Process` are field bags; leaf primitives are sound, but most program changes pay for that centre. |
| Elegance | 3 | Machine transitions follow the problem; the program's lifecycle is encoded in flags instead of states. |
| Economy | 3 | Production is near its rule-forced size; about 2,250 lines could go, mostly tests (sum below). |
| Complexity | 2 | `Phase` × `Stop` × `WindDown` × 7 bools × 19 Options is far more state than the design's lifecycle needs. |

Estimated reduction: tests about 1,600 of 11,300 (F2-F4), production about 650 of 11,500 (the rest), some of it moving into skein.

## 3. Strengths

- **Small, total state machines:** `M/guide.rs`, `M/removing.rs` and `files::Create` are what section 5.4 asks for: each state holds its data and each terminal is handled once. Point newcomers here.
- **`M/tests/routing.rs` and `deadlines.rs`:** a named harness, named tokens, stories of a few lines. `routing.rs` collides token values on purpose to prove namespaces stay apart.
- **Narrow leaves in `P`:** `mcp::Server`, `Relay`, `Command` and `policy::judge` know no policy beyond their own; their module docs say what is kept and never known.
- **Measured ceilings** state their peak, date and headroom, and the machine world uses skein's harness with no private loop.

## 4. Findings

**F1. `Backend` is one 54-field record, not a state machine.** Primitives, Complexity.
- Evidence: `P/backend.rs:161-216` has 54 fields: 7 bools, 19 Options, `stopping: u8`, and `Phase`, `Stop`, `WindDown` beside `switching` and `resuming`. It needs `struct_excessive_bools`. Fifteen files reopen `impl Backend`, with 955 `self.` lines reaching any field.
- `session.rs:191-210` resets about 20 per-process fields by hand, mirroring `new` (`backend.rs:239-294`): a value trying to exist.
- `request: Option<FromDomain>` only ever holds `Complete`, which forces 23 `Cancel | Yield` arms (`start.rs:46,262,266`, `stop.rs:98,142`, `session.rs:139,228`).
- Why: it breaks section 5.4 ("no field is meaningful in some states only"). Preconditions are rebuilt by hand (`start.rs:127-142`, `session.rs:168-176`, `conversation.rs:27,236`).
- Fix: `enum Phase` holding data; a `Program` struct for one process (child, pipes, exit, deadlines, stderr, intake, ledger, turns), so a window switch drops it; an `Active` struct holding `Complete`'s fields.
- Size: about 150 lines from reset and arms directly, and it unlocks F2.

**F2. Program tests reach into private fields and retype their setups.** Economy, Cleanliness.
- Evidence: `active()`/`running()` exist in seven variants (`stop_tests.rs:10`, `review_tests.rs:11`, `builtin_tests.rs:15`, `exit_tests.rs:6`, `measure_tests.rs:11`, `conversation_tests.rs:4`, `open_tests.rs:6`).
- Counts: 83 `up`/`below` openers, 45 `let FromDomain::Complete … else`, 29 `prompt.messages = Box::new`, 44 bare `Token::new(80)`, 145 assignments to private state (`backend.phase` 26, `request` 21, `owner` 19).
- Shared fixtures live in `lifecycle_tests` and `review_tests`, siblings imported for their helpers.
- Rule breaks: recorded streams edited with `str::replace` (`builtin_tests.rs:40,92,401,402`), which section 10 forbids; two stories in one test (`builtin_tests.rs:70-106`); `P/tests.rs:90` branches on a row's value; `backend/tests.rs:161-206` hand-restores each field across a long sequence instead of rows.
- Fix: one `backend/fixtures.rs` with a `Harness { backend, up, below }`, named tokens, `prompt_with(messages)`, `assistant(...)`, `result()`; drive state through `line`/`from_io`.
- Size: about 1,000 of 6,450 lines.

**F3. `M/tests.rs` predates its own harness.** Economy.
- Evidence: 17 three-line openers, 57 multi-line `from_below` calls, 19 retyped `FromDomain::Op`, 95 `Token::new`.
- Row branching at `tests.rs:557-580` (`if probe` twice). Several stories per test at `254`, `348`, `395` ("…and…", "…or…"); `395-433` is three.
- Fix: the `Listings` shape from `routing.rs`, generalized.
- Size: about 400 of 1,343 lines.

**F4. The machine world carries copies.** Economy, Complexity.
- `Job::copy` (`tests/machine/src/lib.rs:319-358`) clones `FromDomain` field by field though it derives `Clone`; `#[derive(Clone)]` removes about 40 lines.
- The owner match is written twice (`lib.rs:307-315`, `904-912`), the referee impl twice (`lib.rs:407`, `simulated_jobs.rs:94`), the run scaffold three times (`lib.rs:251`, `simulated_jobs.rs:17`, `jobs.rs:16`).
- Replay is hand-rolled (`suite/simulated_jobs.rs:15-26`, `fuzzy/worlds.rs:122-124,186`) although `stories.rs:229` uses skein's `assert_replays`; `smith/AGENTS.md` forbids the second copy.
- `jobs_after`'s `delays` is never non-empty. `world.settle()` is called 21 times though `run` already settles (`lib.rs:273`). Six `World::new(n, &[]).root()` build a world to read a constant. `place` has five copies across the machine's tests.
- `fuzzy/worlds.rs:40-112` uses bare `mode` numbers where an enum would read; `:145` is `for owner in 0..3 { match owner … }`, three statements in disguise.
- Size: about 200 of 3,000 lines.

**F5. `Process` splits one fact across fields.** Primitives.
- Evidence: `Purpose` beside `Option<Search>`, `Option<Listing>`, `directory`, `directory_done`, `directory_owner` (`M/process.rs:163-181`), recovered with `expect("search state")` (`:294,295,443,457`).
- `failed` is exactly `failure.is_some()` (`:306-310`). `child_closed = true` also means "never spawned" (`:226,230,237`).
- `process::list` builds a `Below::Spawn` that `start_process` unwraps and re-wraps (`component.rs:360-374`), with `unreachable!` lists on both sides.
- `Sees` and `sees()` exist for one match (`environment.rs:110`).
- Fix: `enum Work { Shell, Check, Search(Search), List(Listing, Directory) }`, and let `list` return the open request.
- Size: about 80 lines.

**F6. The bounded byte tail is hand-written four times.** Economy; skein-lib.
- Evidence: `M/process.rs:86-103,135-150`, `P/command.rs:237-270`, `P/backend/start.rs:549-578` (plus `stderr_wrapped`), and `smith-local-protocol/src/git_child.rs:140-150`.
- Fix: a `skein_lib` tail type. Size: about 90 lines here.

**F7. JSON mechanism belongs in skein-json and skein-lib.** Economy.
- One-shot collection: `P/mcp/json.rs:29-56`. `M/search.rs:268-324` hand-drives the tokenizer and retypes `Intake::meet`.
- `Document` lacks field navigation, so `json.rs:58-102` provides it, and `Match` in `search.rs:12-100` is a second way to read one document.
- Digit parsers: `turns.rs:367`, `search.rs:326`.
- String escape tables: `tail.rs:188-210`, `session.rs:81-105`.
- Hex tables at `mcp.rs:171`, `session.rs:92,109`, `tail.rs:198` although `skein_lib::bytes::hex_digit` exists.
- Size: about 150 lines, after skein gains `collect` and `field`.

**F8. Near-copies inside the production code.** Economy.
- Three kernel-error maps, one exhaustive and two wildcard (`M/files.rs:321`, `removing.rs:174`, `process.rs:703`).
- `in_git` twice (`files.rs:260`, `listing.rs:214`); `other()` three times; `--glob=` building twice (`process.rs:569,672`).
- `outside` and `outside_choice` and a literal render one message (`permission.rs:87,43`, `calls.rs:241`).
- "After the last assistant" twice (`replies.rs:52`, `tail.rs:42`).
- "No longer accepting input" block twice (`conversation.rs:27,236`).
- `Failure::Program { exit: self.exit }` 13 times, usually in a seven-line `fail` call.
- Size: about 120 lines.

**F9. Two hand-rolled bounded ordered prefixes.** Primitives.
- `Search::retain` (`M/search.rs:147-220`) is an insertion sort cloning both hits per swap, with the same `more` increment five times. `Listing::retain` (`listing.rs:108-156`) keeps `List<Option<Entry>>` slots.
- `Map` has `first`, `last` and ordered keys, and section 10.2 says application code does not hand-roll data structures.
- Fix: `Map` keyed by `(path, line)` and by name. (unsure: tuple key cost under the ceilings.)
- Size: about 50 lines.

**F10. Bare slots and sizes in the backend.** Cleanliness.
- `self.token(3..6)` at 22 sites; `pipes[0|1|2]` at 13; output rights `Token::new(1)`/`(2)` (`start.rs:468,538`, `stop.rs:238`).
- `4096` at 10 backend code sites beside named `READ_BYTES`, `CHUNK`, `OUTPUT_READ`; `max_completion_blocks.checked_mul(5)` six times; `65` at `mcp.rs:335,358`.
- `b"mcp__smith__permission_prompt"` (`start.rs:621`) retypes `SERVER` and `PROMPT_TOOL`. The tokenizer limits `depth: 8, number: 20, chunk: 64` are typed in `limits.rs:45` and `search.rs:270`.
- Fix: constants, and `struct Pipes { input, output, errors }`. Size: neutral, large clarity gain.

**F11. Two policies for foreign enums.** Taste.
- `component.rs:454-467,477-486,494-505,515-526` lists about ten `io::Event` variants four times, while `guide.rs:42` and six others use `_` under a reasoned `expect`.
- Merge each lookup and dispatch into one match, and give `io::Event` an `owner()` as `file::Event` has (`skein-io/src/file.rs:305`), which `start.rs:337-351` also wants.
- Size: about 40 lines.

**F12. Effects leave by two paths.** Primitives.
- `Backend` takes `below` yet keeps `self.io`, drained by `flush`.
- `mcp::Server` keeps `above` and is polled by `for _ in 0_u32..2` (`start.rs:190`).
- Fix: pass the queues in as the step shape says. Size: small; removes a capacity coupling.

**F13. Naming.**
- `Stop`, `stop()`, `stopping`, `stop` (field), `finish_stop`, `stopped_result`, `terminate`, `close`, `close_turn`; `Stop::WindDown` beside `WindDown`.
- `running`/`recorded` are session ids, `permitted` a message index, `tail` a ring cursor; none is documented, though `smith/AGENTS.md` asks for it when the name does not say.
- `Guide` also holds probes; `Making::Making`; two `Next` enums.
- `review_tests.rs` is named for its origin, not a behaviour.
- `lib.rs:1-4` of `P` describes source commands, not the backend that is most of the crate.

**F14. Placement and stale words.** Organization, Cleanliness.
- `M/limits.rs:121-309` (61% of the file) is heap tests with a `#[global_allocator]`.
- `use` mid-file (`removing.rs:171`); test module mid-file (`stream.rs:119`); `cfg(test)` shims (`stream.rs:38`, `tail.rs:22`).
- `#[expect(clippy::single_match)]` (`process.rs:536,637`) gives a reason section 10.2 contradicts: `if let` on `Option` is allowed.
- `signal_tests` (`process.rs:723-728`) compares Debug strings so it compiles "on the defect's parent": history, and a weaker test.
- `suite/simulated_jobs.rs` has seven "Lower test of smith-machine-world's …" docs: six cite tests that exist nowhere in smith, and the seventh lives in `tests/tools`, another crate.
- `tests/machine/README.md:31` says a two-MiB bound; the code's ceiling is 21,000 (`lib.rs:159`).
- "actual" recurs in test names, messages and the README against the root AGENTS rule (`M/tests.rs:436,556,570`, `suite/fuzzy/listing.rs:79`).

## 5. Cross-area suspicions

- **smith-local-protocol `git_child.rs`:** the ring, and likely `Process`'s and `Command`'s child-plus-two-pipes lifecycle, a third copy.
- **smith-agent-service `service/down.rs:182-238`** and `tests/machine/src/lib.rs:702-768` route `Below` near-identically (unsure a shared router fits).
- **smith-domain-run `Abort` / `Cancel`:** for checks the machine treats them alike (`component.rs:127-130,198-201`).
- **smith-events `read.rs`, smith-protocol-llm:** one-shot JSON driving and hex tables (`failure.rs:175`, `render.rs:150`).
- **`impl Referee<…>` boilerplate** recurs across smith's test crates.
- **`machine::Derivation`** mirrors `Limits` field for field; `run`, `session`, `tools` likely do too.

## 6. Rule costs

- **No closures:** Option plumbing is written long (`next_deadline`, `backend.rs:545-572`; `manual_map` expects, 4), and each JSON document is spelled twice for measure then write (5 pairs in `P`). About 100 lines.

---

# 07: agent service (`crates/smith-agent-service`, `tests/agent`)

Paths are relative to `smith/`. `S/` is `crates/smith-agent-service/src/`; `TA/` is `tests/agent/`.

## 1. Scope read

**Closely:** `S/service.rs` and `S/service/{up,down,routing,request,effects}.rs`, `teardown.rs`, `program.rs`, `sinks.rs` (with tests), `responses.rs` to line 1280, `answers.rs`, `events.rs`, `calls.rs`, `live.rs`, `children.rs`, `observations.rs`, `inputs.rs`, `messages.rs`, `calls/{builtin,jobs}.rs`; `TA/src/world.rs` (types, constructors, `drive`, start of `request`).

**Sampled:** `S/profile.rs`, `validate.rs`, `effective.rs`, `list.rs`; `S/tests.rs`, `service/tests.rs`, `profile/tests.rs`, `tests/line.rs`, `tests/suite/*`; `TA/tests/suite/{budget,brief,simulation,feedback_table,messages,referee}.rs`; `TA/src/{memory,referee,fixture}.rs`.

**Only counted or skimmed:** `sinks/view.rs`, `responses/retry.rs`, `profile/{room,path,words,quantities}.rs`, `TA/src/{script,translate,host_referee,messages_referee,orders}.rs`. Counted test statistics cover 330 tests in the area.

Size: about 11,000 production lines (event projection 4,600, composition 3,200, profile 3,200), 5,500 lines in separate test files plus about 2,900 inline, and 13,600 in `TA/`.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | A 509-line orphan, a banned word 457 times and 178 citation lines; production layout is otherwise tidy. |
| Organization | 3 | `service/` splits well, but the event projection hides in a composition crate, and `#[path]` and 500–800-line inline test modules blur the layout. |
| Taste | 3 | Strong state enums; `Option` as a mode, one queue cell per byte and an unexplained trait undercut them. |
| Naming | 4 | `Teardown`, `Step`, `Close`, `Answered` match the design; `Connection`/`Room` misname and "Failure" has four meanings. |
| Primitives | 3 | `Writer`, `Attempt`, `Book` are narrow; `Service` is two things selected by `Option`. |
| Elegance | 4 | `iterate` reads as the pipeline and `Teardown` is agent.md's phase; `work_pending` mirrors every pass's gates by hand. |
| Economy | 2 | Adding a `Start` field touches up to 33 sites; ritual and copied projections add hundreds of lines. |
| Complexity | 3 | Limit derivation earns its size; 11 world constructors, a one-variant enum and the byte queue do not. |

## 3. Strengths

- **The end is one enum.** `S/teardown.rs:14` carries data per phase and is agent.md section 6 nearly word for word; `Step` orders the reports. Keep this as the model for state.
- **`sinks.rs`'s `Stream` and `Attempt` in `responses.rs`:** states with their data, exhaustive matches, and fixtures (`sending`, `settled`, `counted`, `sinks.rs:1475–1597`) that keep the given-up tests to a few lines each.
- **`TA/tests/suite/referee.rs`:** 18 deliberately bad transcripts through `started()` and `broken()`, each one behaviour. Point new contributors here.
- **Typed startup refusals:** `Check`, `LimitName` and `Value` name every relationship, and `words` is asserted byte for byte (`profile/tests.rs:144,176`). Lint exceptions in production all carry reasons, and `live.rs`'s tests name both sides of its bound (`live.rs:326–329`).

## 4. Findings

**1. `Service` is two things, selected by `Option`** (Primitives, Complexity).
`Effects` wraps a `Service` built with `domain`, `channel` and `io` as `None` (`service.rs:145–151`, `effects.rs:22`). The mode is chosen by `seed: Option<u64>` (`service.rs:277,281`), which leaves `Effects::new(config, _seed)` an unused parameter (`effects.rs:29`). Cost: 40 `.expect("framed agent owns…"/"standalone IO")` and 27 branches on those fields. `lib.rs:4` ("only progression entry point") and `service.rs:2` ("one root domain") are no longer true.
*Fix:* extract the lower stages (LLM, machine, program, files, queues, routes) as one struct; `Service` owns it plus domain, channel and io; `Effects` wraps it alone. *Effect:* about 40 expects and 150 lines gone; the framed-only code can no longer compile against an absent channel.

**2. Fixtures retyped at every level** (Economy; "a setup written once").
`Event::Start { … }` appears at 33 sites (21 in `TA/`, 12 in `S/`; a few are patterns), `run::Charter {` at 7 in `TA/` plus `S/tests.rs:46`, the 7200-second grant 9 times. Local `fn charter` in 3 suite files, `calls` in 6, `call` in 5, `says` in 3. `S/tests.rs:100` and `:133` are the same call; `profile/tests.rs:114` a third. The world has 11 public constructors, and three (`world.rs:788–871`) are near-copies that end in an 8-argument `with_selected_backend`. `declared.memory = u64::MAX` is repeated 11 times; `service/tests.rs:10` hand-types 110 lines of `Limits` while `profile::derive` exists.
*Fix:* one `Scenario` struct with defaults, plus `start(charter)`, charter overrides on `scripted_charter`, and script helpers in `TA/src/fixture.rs`. *Effect:* about 450 of 13,600 lines in `TA/`, about 100 in `S/`; a new `Start` field touches one line.

**3. Dead wire backend** (Cleanliness).
`TA/src/wire.rs` (509 lines) is declared by no module (`lib.rs` has no `mod wire`). `Backend` has one variant and is still named 16 times (`world.rs:267–300,1101,1139`), and comments still promise an "actual wire backend" (`world.rs:372,2314`). `stimuli: Vec<()>` is passed only to satisfy an API (`world.rs:431`); `World::wall_at` has no caller. `Producer::new` takes `_endpoints` and `_declared` (`events.rs:199–200`).
*Fix:* delete them. *Effect:* about 570 lines.

**4. The channel send ritual** (Economy).
`S/service/request.rs` repeats the same 10 lines 13 times: `next_send`, `as_mut().expect`, `send_x(…, token, &mut channel_events, &mut channel_below)`, `is_err`, `mark_failed(Failure::Send)`; 17 `mark_failed(Failure::Send)` crate-wide. `live::Connection` already bundles exactly these parts.
*Fix:* a `HostLink` borrow of channel, queues and token, plus `sent(service, result)`; no closure needed. *Effect:* about 75 of 374 lines.

**5. Failure projected to events twice** (Economy, Primitives).
`answers.rs:297–418` and `responses.rs:916–1045` are the same 120 lines over `run::CompletionFailure` and `llm::Failure`. The run's types are separate by rule (siblings), but `smith-domain/src/translate.rs:314` already maps one to the other, one to one.
*Fix:* export that translation from `smith-domain`; `responses` calls it, then `answers::completion`. *Effect:* about 125 lines, and `limit_name` exists once.

**6. `live.rs` departs from the subset with no written reason** (Taste, Complexity).
`pub(crate) trait Room` and `<R: Room>` on 7 methods (`live.rs:11,134–290`), only so its tests can fake the channel; `smith/AGENTS.md` bans application traits and generics. `HeldByte` stores time, call and offset per byte (`live.rs:74`), up to 16,384 cells, forcing `IDENTITIES = HELD + 1` and a byte-by-byte rescan in `pump`. Sizes are bare at `[0; 4096]` (`:270`, beside `PIECE`) and `256`, `64` (`:167`). Its module doc omits what the house rule requires, and methods have no blank lines between them.
*Fix:* store runs (`call`, `offset`, `arrived`, `bytes`); take concrete queues; test with a real `channel::Component`. *Effect:* about 60 production lines, and the rule break gone.

**7. `io::Event` vocabulary reimplemented here** (Primitives; belongs in skein-io).
`file::Event::owner()` exists, but `io::Event` has none. `routing.rs:214` and `:237` hand-write owner and re-owner, and "is this `Closed`" is a 15-line match at `routing.rs:140`, `:169`, `program.rs:477`. Nine full listings of the variants.
*Fix:* `Event::owner`, `with_owner`, `is_closed` in skein-io. *Effect:* about 90 lines.

**8. Citation boilerplate, and a citation to nothing** (Cleanliness; `smith/AGENTS.md`: "never on every field").
178 citation lines in `TA/`, 68 of them the literal "Scripted-world contract: …" on each field and variant (`world.rs:107–130`, `referee.rs:80–120`). `scratch/client.md` is cited 60 times in `TA/src`; no such file is in the repository.
*Fix:* keep the module-level citation. *Effect:* about 190 lines.

**9. House comment rules broken in bulk** (Cleanliness, Naming).
AGENTS.md section 2 bans "actual" and history. "actual" appears 130 times in `S/` source files, inline tests included (35 in comments, the rest in messages and names), and 327 times in `TA/`; 19 test names carry it. History: "previously lived" (`service.rs:230`), "Do not probe its heap sum" (`:205`), "legacy" (`up.rs:541`, with an 80-line test at `up.rs:553`), "copied"/"temper `25ac2ad`" (`TA/src/lib.rs:15`, `limits.rs:1`, `Cargo.toml:4`). `#![allow]` in 9 test modules despite "never `#[allow]`". "bounded" appears about 90 times and often says nothing.
*Fix:* one mechanical pass, then a gate grep. *Effect:* lines stay; the prose gets shorter.

**10. Tests break their own shape** (Economy; section 10).
Rows as code: `match case` over `0_u8..7` (`program.rs:731`), with a pre-assertion. Bodies branching on rows: `sinks.rs:1395–1439` (`exit == 0`, `match usage`), `sinks.rs:1334`. Expected values computed by branching on their text: `feedback_table.rs:19`. Two worlds in one test: `messages.rs:111`. Named sizes: 58 bare `.run(2000)`-style calls, and `ITERATIONS` defined in 3 files. 34 of 330 tests exceed 60 lines (`calls.rs:819` is 190), 20 have 10 or more asserts, 123 names contain `_and_`, and `nonrenewable_401_count_proof` (`accounts.rs:129`) names no behaviour. This is how "Volume and churn" (tech-debt.md) shows here.
*Fix:* split at the `_and_`s; move data into rows. *Effect:* mostly clarity, about 300 lines.

**11. Layout** (Organization).
The event projection (`events`, `sinks`, `responses`, `calls`, `children`, `inputs`, `messages`, `observations`, `answers`) is 4,600 lines, meets the service only at `Limits`, `Config` and `answers::last`, and smith-local-service also uses it through `Producer`. Move it to its own crate. `program.rs` and `teardown.rs` belong in `service/`, not behind `#[path]` (`service.rs:27,32`). `S/tests.rs` is a grab-bag under a profile doc line; `use crate as profile_service` plus `#[path]` (`tests.rs:36–39`) exists to share one fixture. `calls.rs` is 54% tests, `responses.rs` 43%; `sinks.rs` mixes the writer, sizing formulas and tests. `program.rs` has two test modules. `world.rs::request` is 400 lines. *Effect:* no lines; files become about half their size.

**12. `work_pending` mirrors every gate by hand** (Elegance).
`service.rs:750–790` restates each pass's conditions; `channel_event_ready` shows the right pattern, the others do not. The channel-room check is typed 4 times (`up.rs:63`, `:251`, `:295`, `down.rs:41`), the `takes` guard 4 times, the root-close condition twice (`teardown.rs:185`, `:267`). `next_deadline` and `next_policy_deadline` repeat one fold (`service.rs:642–692`). A drifted gate is a stall.
*Fix:* name each gate once (`sinks_take`, `channel_room`, `roots_closable`) and use it in both places. *Effect:* about 60 lines.

**13. Bare numbers and magic names in step code** (Taste).
`program.rs:61–113` (`7`, `4096` four times, `65_536` twice, 5 s as a `Time`, 30 s); `events.rs:53,58,74,85` (`46`, `18`, `128`, `2048`); `profile.rs:71,92,109,127`. `Token::new(u64::MAX)` means two different things (`routing.rs:309`, `teardown.rs:280`), and `Token::new(1)` is the Start's reply right at `service.rs:573` and `request.rs:113`. Name them with their derivation. Also, 37 `/ 1_000_000` and two private `millis` (`calls.rs:409`, `responses.rs:903`): add `Duration::as_millis` to skein-lib. *Effect:* about 40 lines.

**14. Pass-level white-box tests** (tech-debt "fixtures that pin implementation details").
`service/tests.rs` has 31 direct writes to private fields and queues, and calls `component_up` and `domain_down` by name, so a pass reshuffle rewrites tests that prove routing. Prefer `iterate` with injected events. Small effect, but it lowers the cost of finding 1 and finding 12.

## 5. Cross-area suspicions

- **`tests/protocol-llm` is a fork of `tests/agent`.** Distinct lines shared: `fixture.rs` 36/36, `limits.rs` 134/136, `host_referee.rs` 262/280, `script.rs` 200/230, `referee.rs` 264/311, `messages_referee.rs` 208/257, `translate.rs` 183/257, `world.rs` 1181/1607. `smith/AGENTS.md` forbids second copies. This is the largest saving near my area (thousands of lines).
- `scratch/client.md` is cited in `tests/protocol-llm` too (130 in all).
- `temper-service/src/clients.rs:901` has its own `event_owner`, supporting finding 7.
- `NoticeKind::Unknown(b"not_contained")` is emitted on purpose (`events.rs:238`) and matched as bytes in `smith-local-shell/src/front/interactive.rs:181`: smith-events lacks the variant. `Notice` has no constructor; 3 ten-line literals.
- `mcp::Limits.unauthenticated` is a duration typed `Time` (smith-protocol-program `mcp.rs:55,331`).
- smith-local-service keeps its own usage owner and settlement (`stream.rs:377`, `process.rs:1038`), which may overlap `teardown.rs` (unsure).

## 6. Rule costs

"List every variant" costs most on `session::FactKind`: 7 near-identical listings of about 20 arms in production. Part of that is not the rule: `children.rs` matches twice (`:66`, `:92`), `calls.rs` does the same with `call_opener` (`:261`), and one match each would save about 40 lines.

---

# 08: agent shell, root package, process and end-to-end tests

Tree `aa95945cc`. Paths are relative to `smith/`. Nothing was run.

## 1. Scope read

- **Closely:** every production file of `smith-agent-shell`; `src/`; root `Cargo.toml`; `config/tests/mod.rs`; `tests/agent-process/src/{lib,world,program,tracing,fake}.rs`; `suite/stories.rs`; `tests/end-to-end/support/mod.rs`, `end_to_end.rs`, `program_runs.rs`, `program_commands.rs`; `tests/fake-claude/src/{fake,story}.rs`, `tests/support/mod.rs`; all of `tests/real` but `settled.rs`.
- **Sampled:** `print.rs` (first 420 lines), `simulated.rs`, `turns.rs`, `support/relay.rs`, `program_tools.rs`, `program_resume.rs`, `content.rs`, `grants.rs`, `exec.rs`, `login.rs`, `provider_login.rs`, `live.rs`, `support/live.rs`, `tests/live/program.rs`.
- **Counted only:** the other suites; `mcp.rs`, `server.rs`, `events.rs`, `orders.rs`, `window.rs`, `live_run.rs`.
- **Size:** agent shell 3,506 lines; agent-process 9,486; end-to-end 4,279; fake-claude 7,428; real 559. 394 tests.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | Shell code is tidy; "actual" 107 times, 9 `#[allow]`, unused parameters and dependencies, and history comments recur. |
| Organization | 4 | Layout follows testing.md 7 and suites split by topic; `world.rs`, `config.rs`, `agent-process/fake.rs` and the root's third binary are the exceptions. |
| Taste | 3 | `Agent`, `Effect` and test names show care; scenarios passed as strings (`spawn.args[0] == b"8"`) do not. |
| Naming | 4 | Test names read as sentences and use the design's words; `Refusal` (a general judge), `run_inner`, eight `Judge`s and `start_workspace_message` fall short. |
| Primitives | 2 | Shell primitives are right, but the worlds have no named frame kind, charter edit, frame builder or scenario value, so most test changes pay. |
| Elegance | 3 | `Agent::iterate` and the fake's state machine follow the problem; the scenario flag ladders follow history. |
| Economy | 2 | About 1,800 of 25,000 lines could go, mostly retyped wire records, run setups and harnesses (section 4). |
| Complexity | 3 | Bounds and lifecycle are as complex as needed; `World` and `Parent` flags and the one-variant program map are not. |

## 3. Strengths

- **The agent pass.** `agent_shell.rs:373-433` is one readable shell pass that binary and worlds share; `given_up_line` and the `service::Failure` match are the model.
- **The fake's shape.** `Fake` runs unchanged in world and binary, with `print.rs` returning `Vec<Effect>`, and `Story` is a typed, documented script. Keep it.
- **Outside observation.** Worlds decode channel frames, append files and peer queries independently; `tests/real` stays small (559 lines) by reusing local-process scenarios.
- **Typed refusals.** `ConfigurationRefusal` with `LimitName` gives exact named refusals, tested by exact string.

## 4. Findings

**1. A world keeps a second copy of the agent adapter.** *Primitives, Economy.*
- Evidence: `tests/agent-process/src/tracing.rs:98-220` reimplements `Agent`'s `iterate`, deadlines, `is_empty`, `exit` and close-token scheme. It retypes `SessionStarted` (`:138-159`, versions as literals) and `effective_header` (`:237`), and it omits the operator lines.
- 28 call sites of `observe_facts`, `signal_after_answer` or `order_pressure` in 9 files run on it, including `memory.rs` and `content.rs`.
- Why: testing.md 2.1 says "No world keeps a copy of it" and section 4 says "No copies of shipped code". testing.status.md lists no departure.
- Fix: host `Agent` with a `StartupAppend`, as `program.rs:641-653` already does. Put pressure in a `Delegate` wrapper. `channel_loss` is in `session.ended`. `lost_live_bytes` needs a read-only face (unsure).
- Size: about -170 of 246 lines.

**2. `World` and `Parent` are flag bags, with scenarios smuggled through spawn arguments.** *Primitives, Complexity, Taste.*
- Evidence: `World` has 31 fields (`world.rs:1042-1074`); `Parent::new` takes 12 arguments (`:51`).
- The scenario travels as `b'0' + case.number()` (`:127`), then `spawn.args[0] == b"8"` and `b"6"` (`:521`, `:530`), `"openai"`, `"environment"`, `"roots"`, `"coding"`, `"order-requests"`. `program.rs:184` encodes the same idea differently, as serde JSON.
- `make_peer` (`:1528-1589`) is an order-dependent `else if` ladder; `Referee::observe` branches on eight flags (`:767-832`).
- `run_inner` repeats one eight-variant `matches!` three times (`program.rs:558`, `:581`, `:594`).
- `Parent` carries program-only fields for the other world (`:42-46`).
- Fix: one serde `Setup` value as the single spawn argument, read by one `make`; a `Peer` enum in place of the ladder; `Case::completes()`.
- Size: about -300 lines. A new scenario then touches one type.

**3. No wire primitives in agent-process.** *Economy, Primitives.*
- Evidence: 57 bare kinds (`0x0110` ten times, 20 files); 21 charter decode-edit-encode triples (`Charter::new`); 35 `Writer::new(...measure())`; 13 `frame_writer`; five decode-and-rewrite Starts (`lib.rs:408`, `program.rs:789`, `:833`, `carry.rs:20`, `coding.rs:55`). A bare kind cannot be searched by meaning.
- Fix: named kinds exported by `smith-channel` (`KindRule` has only numbers); `edit_charter(bytes, |parts| ...)` with `budget(parts, turns, spend)`; one `frame(kind, record)`; one `edit_start`.
- Size: about -300 lines.

**4. Option ladders instead of a value.** *Economy, Complexity.*
- `agent-process/fake.rs`: `configured` → `configured_transport` → `configured_dialect` → `configured_bounds`, with positional bools, and `Credential` retyped at `:140`, `:246`, `world.rs:931`, `:1029`, `:1577`.
- `run_inner` takes 9 positionals (`program.rs:546`, called with `None, None, false`).
- `Live::start*` has five rungs (`live/program.rs:22-38`). `exec.rs run_fixture` takes 7. `run` → `run_files` → `run_files_observing` → `run_items` in `fake-claude/tests/support/mod.rs:540-570`.
- Fix: a `Default` options struct per harness, with a shared `credential()` fixture.
- Size: about -120 lines; removes two `too_many_arguments` expects.

**5. End-to-end run setup is retyped.** *Economy, Naming.*
- The environment list appears 6 times with drift (`mod.rs:206`, `:652`, `exec.rs:234`, `live.rs:239`, `provider_login.rs:177`, `login.rs:84`; the last omits `XDG_RUNTIME_DIR`).
- `Binary::start`, `take_streams`, `spawn_with_fds`, drain assert and positional `procs` downcasts occur in 5 runners. The login file is read twice (`mod.rs:233`, `exec.rs:107`).
- The "binary exited, shut everyone down" loop is in 5 referees. `Refusal` is the general pipe judge, used for successful runs too.
- Fix: one `Product::start(scratch, mode, ...)` and an `Outcome` with named accessors; a shared `ending()` helper; rename `Refusal` to `Settling`.
- Size: about -150 lines.

**6. `smith-agent-shell` decoding does effects.** *Primitives.*
- `config::parse` and `from_value` resolve names (`config.rs:497`, `:709`), read trust files (`:573`, `:745`) and `eprintln!` (`:804`, outside the supplied error writer).
- So `run` resolves and loads trust before `open_termination_signals` (`agent_shell.rs:501-505`), against shell.md 2.2's order, with no status entry. This also drives 38 `cfg` lines and duplicate non-unix stubs (`config.rs:493`, `:567`; `:709`/`:715`).
- Fix: split a pure `decode` from a native `resolve` step; report the trust line through the writer.
- Size: about -40 lines, and a checkable startup order.

**7. `config.rs` walks its endpoints four times.** *Economy, Complexity.*
- `build_models` runs twice per endpoint (`:388`, `:481`). Prices are joined by index (`:398-402`, `:426`). Provider strings are matched twice (`:499`, `:627`).
- `effective_header` is written twice with different words (`:324`, `agent_shell.rs:152`). `identity()` exists yet `prepare` retypes it (`agent_shell.rs:296`).
- `event_limits` has two unused parameters (`:306-307`). Versions are literals though `FORMAT_VERSION` exists (`agent_shell.rs:308-310`). `512` is bare twice (`:771`, `:793`).
- Fix: one `PreparedEndpoint` built once; move documents and trust into `config/` modules.
- Size: about -60 lines; `config.rs` under 500.

**8. Fake-claude's own harness is written twice.** *Economy, Cleanliness.*
- `simulated.rs:260-548` (`mod no_stdin`) has its own `Parent`, `Process`, `MachineFiles` and `Judge`, though `Invocation.no_input` exists (`support/mod.rs:23`, `print.rs:98`). Reusing it is likely possible (unsure: it spawns through io, not raw ops).
- `support/relay.rs:23` adds a third `limits` and pump. `tests/hosts/src/program_world.rs` repeats `MachineFiles`.
- Eight `#[allow(dead_code)]`, against root AGENTS.md 2, are stale: `suite/main.rs` is the only binary that mounts `support`, though `relay.rs:3` says several do.
- `story.rs:117`, `:304`, `:315`, `:337` are four copies of one line reader.
- `print.rs:271` keeps a one-byte session marker nothing writes, kept alive only by `a_legacy_session_marker_starts_at_zero` (`cues.rs:209`). A fake needs no compatibility with its own past.
- Fix: one `Invocation::no_input()`; delete the `allow`s and whatever they then expose; one `records(bytes, key)`; drop the marker and its test.
- Size: about -300 lines.

**9. Profile overrides are 41 near-copies of one block.** *Economy.*
- `config/profile.rs:24-233` repeats the positive check 23 times; durations and counts are mixed (`max_time` sits in `apply_counts`, `:130`).
- Fix: a `positive(value, "name")?` helper, or a local macro, which a shell may use.
- Size: about -100 of 235.

**10. End-to-end hand-types the fake's schema.** *Economy, Taste.*
- 18 hand-written story sites (`{"Text":..}`, `"Truthful"`) though `Story::encode` exists; three private observation parsers (`program_runs.rs:38`, `program_commands.rs:40`, `program_check.rs:30`) plus an inline fourth (`program_commands.rs:105`) beside `smith_fake_claude::observations`; 17 file-name literals beside `STORY_FILE`.
- Three ways to find the binary (`program_check.rs:8`, `program_runs.rs:13`, `program_commands.rs:52`).
- Fix: use the typed crate, already a dependency.
- Size: about -60 lines; a story's shape is checked by the compiler.

**11. smith hand-drives io beside `skein_io::stage`.** *Economy, Primitives.*
- `stage` says "written once for every loop", yet `relay.rs:222-237`, `fake.rs:464`, `server.rs:261`, `support/relay.rs:253` and `simulated.rs:347` drive the four entries with unbounded `while`s.
- `relay.rs:201` `owner()` has copies at `support/relay.rs:181` and in three crates outside this area.
- Fix: `stage::up`/`down`; `Event::owner` in `skein-io`.
- Size: about -50 lines here. Relay diagnostics hold three correlated fields (`diagnostic`, `_waiting`, `_done`; `relay.rs:38-40`) that want one enum.

**12. Generality for one adapter.** *Complexity.*
- `BTreeMap<u32, ProgramName>` of a one-variant enum costs 10 `zero_sized_map_values` expects, while `programs.rs:56`, `:105`, `:135` hard-code Claude Code anyway.
- Fix: `Option<Program>` and a set of endpoints until a second adapter exists.
- Size: about -30 lines, across `smith-local-shell` too.

**13. Root package and manifests.** *Organization, Cleanliness.*
- `smith-fake-claude` is a production dependency for a third `[[bin]]` (`Cargo.toml:26-28`, `:48`). shell.md 3.1 lists two binaries, and 2.4 says none links a testing crate. `tests/fake-claude` declares a bin of the same name (`output filename collision`, unverified).
- 25 of 43 dev-dependencies have no reference in `src` or `tests/end-to-end`; three repeat `[dependencies]`.
- `libc` and `smith-transcript` are unused in `smith-agent-shell`.
- `main.rs` tries three commands, each calling `command::parse`; shell.md 3.1 has main parse once. The trailing `ExitCode::from(2)` looks unreachable.
- Fix: move the fake's path to a test-only binary (a design question); prune; match once.
- Size: about -35 manifest lines.

**14. Startup logging depends on remembering.** *Taste.*
- `run` promises failures are "already logged" (`agent_shell.rs:498`) and four steps do it by `inspect_err`, but the three `?`s of the program block (`:510-513`) write nothing, and `smith-agent.rs:26` then exits 1 silently, even on an unsafe programs directory.
- `Agent::read` is reached only by tests. `Configuration.memory` has no non-test reader (`.memory` search).
- Fix: an inner function with one logging exit; drop both.
- Size: about -15 lines.

**15. Test-shape breaches.** *Cleanliness; testing-strategy.md, 10.*
- Branching on row labels: `tests/real/tests/suite/stories.rs:51-63`, `fuzzy/process.rs:12,17`, `content.rs:92,111`.
- Several behaviours per test: `the_startup_refusals_of_both_commands...` (`end_to_end.rs:174`), `program_commands_use_the_fake_program`, `a_status_without_stdin_answers`.
- The agent JSON is retyped 4 times in `config/tests/mod.rs` beside `shaped_document` (`:175-215`), and 3 more outside.
- 17 bare `.windows(N)` searches; 9 `is_err` refusals that name nothing; delivered-file asserts retyped 4 times; `Resources` retyped 7 times; "actual" 107 times and "former" (`coding.rs:9`, `end_to_end.rs:222`), against root AGENTS.md 2.
- Size: about -100 lines; a `contains(bytes)` helper and `assert_delivered(checkout)` carry most.

## 5. Cross-area suspicions

- `smith-protocol-channel/src/component.rs` writes raw `0x0110`; named kinds belong to `smith-channel`.
- `Event::owner` copies: `smith-agent-service/routing.rs:216`, `smith-local-service/process.rs:1374`, `smith-protocol-program/backend/start.rs:338`.
- Hand-driven io: `smith-local-shell/.../runner.rs:33`, `tests/hosts/src/program_world.rs:90`, `tests/local-process/src/{terminal,oauth}.rs`, `tests/machine`.
- The worlds' `Machine` adapter with identical `open_root`, `close_root` and `open_append` (`tests/hosts`, `tests/local-process`) suggests a skein-fake-machine default.
- `REAL` (`agent-process/fake.rs:9`), a global that `Scratch::new` flips, and `loopback`, used by `tests/real` and end-to-end, want a home in `skein-fake-peers`.
- `smith-local-shell` shares the `zero_sized_map_values` cost.

## 6. Rule costs

No house rule costs much here. One skein shape does: `HostedProgram.make` is a plain `fn` (`skein-sim-world/src/program.rs:28`), so scenario data must travel through spawn arguments (finding 2). A typed setup value keeps that cost to one line per world; a data slot in skein would remove it.

---

# 09: host domain, host protocol, inline agent, tests/host, tests/hosts

## 1. Scope read

**Closely:** `smith-host-domain`: `domain.rs`, `channel.rs`, `boundary.rs`, `parent.rs`, `process.rs`, `inbox.rs`, `limits.rs`, `long.rs`, `values.rs`, `facts.rs`, `program.rs`, the first half of `delivery.rs`. `smith-inline-agent`: all production files. `smith-host-protocol`: `component.rs`, `translate.rs`, `answer.rs`, `process.rs`, `live.rs`, `program.rs`. `tests/host/src/lib.rs`; `tests/hosts/src/{lib,referee}.rs`.

**Sampled:** `smith-host-domain/src/tests.rs` (420 of 651), `smith-inline-agent/src/tests.rs` (400 of 745), `smith-host-protocol` `tests.rs` and `component/tests.rs`, `tests/host/src/inline.rs` (first 330, tail), `suite/simulation.rs`, `simulation/inline.rs`, `memory.rs`, `referee.rs`, `tests/hosts/src/world.rs`.

**Skimmed or unread:** `tests/hosts/src/program_world.rs`, fuzzy suites, `process/tests.rs`, `program/tests.rs`, the design's section 8.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score up |
|---|---|---|
| Cleanliness | 3 | 101 banned "actual" and "genuine", 19 "V1/V2", a stale crate doc, and dead public items. |
| Organization | 4 | Crates split as the design says. `domain.rs` is 1,503 lines of three concerns, and `translate.rs` hides the router. |
| Taste | 3 | The inline `Stage` and `Inbox` show care; `drain`, the `Sent` overload and suppressed lints do not. |
| Naming | 3 | `OpenEvent`, `Component`, "capability" and "rights" leave the design's words; `agent` names both a domain and a token. |
| Primitives | 3 | Parent and process faces are right. Failure vocabulary and `OpenEvent` overlap what already exists. |
| Elegance | 3 | Inline stop follows the problem. `Agent` and `Component::drain` do not follow 5.4. |
| Economy | 2 | About 900 lines of verbatim vocabulary and translation; any new failure kind pays for it. |
| Complexity | 3 | Most is inherent. 12 bools plus 7 Options beside a `Phase`, and forwarders, are not. |

## 3. Strengths

- **The shared parent face works.** `parent.rs` and `process.rs` keep the faces apart (host.md, 9.4). The inline agent speaks `parent::{Event, Request}` unchanged, and `Seen::observe_parent` (`tests/host/src/lib.rs:577`) referees both kinds, taking the proof of containment as a parameter.
- **`smith-inline-agent/src/stop.rs`.** `Stage` and `Settling` hold exactly what each state needs, in 140 lines. It is what `Agent` should look like.
- **`Inbox` and `Long`.** Small, one job each, shared or tested alone. `live.rs`'s `each_order_rule_refuses_its_breach` is a good rows-carry-data table.
- **Skein's kit is reused.** The worlds use `Stage`, `Schedule`, `Ledger` and `Trace`; `tests/hosts/src/referee.rs` is `Expectations` and `Judge`. No second referee mechanism, and the should-panic tests in `suite/referee.rs` prove the referee catches what it claims.

## 4. Findings

### 1. The host crate retypes the run's vocabulary, and the inline agent translates it back
**Economy, Primitives.** A variant-by-variant comparison of `smith-host-domain` with `smith-domain-run` found these identical or differing only in order or in a nested type's name: `Cap`, `Limit`, `Phase`, `CompletionFailure`, `CompletionEvidence`, `Exhausted`, `Overflow`, `ReceivingLimit`, `TranscriptRefusal`, `Policy`, `Refusal`, `Part`, `Ceiling`, `BudgetRefusal`, `Rate`, `Invalid`/`RunInvalid`, `MessageRefusal`, `Delivery`, `DeliveryReason`, `DeliveryStatus`, and the struct `Report`. Copies: `channel.rs:228-373`, `:435-496`, `:526-637`. `delivery.rs` (431 lines) is a near-copy of `smith-domain-run/src/delivery.rs` (459), tests included.

The only material difference is `ModelFault::Limited { reset: Duration }` against `Fault::Limited { reset_at: Time }`. `smith-inline-agent/src/translate.rs:151-253, 262-345, 562-638` exists to map one to the other.

`delivery.rs:2` says this avoids sharing agent internals, but `values.rs` already wraps `run::Charter`, `lib.rs:73` re-exports `smith_domain::Operation`, and host.md, 3 says "payloads are smith's vocabulary". The crate already depends on `smith-domain`.

**Fix:** `pub use smith_domain::run::{...}` with aliases for the host names; keep `ModelFault` and `RunFailure` plus their ~40 lines of translation for the one `reset` field, or make the run's relative. `answer.rs` keeps its wire mapping, now aimed at run types. **Size:** ~-900 lines (320 + ~420 + ~230, less what stays). (Designer's check: nothing in host.md asks for a firewall here, but confirm.)

### 2. `OpenEvent` is a second `Up`, and every host hand-writes the mapping
**Primitives, Economy.** 13 `OpenEvent` variants (`component.rs:51-97`) repeat `channel::Up` field for field; only `Answer` adds its wire record. hosts.md, 2 makes that translation the half's job, yet it lives in hosts: `smith-local-service/src/process.rs:1305-1361` and `tests/hosts/src/lib.rs:333-401`, whose header says "the part another host copies". `process.rs:453-470` and the local service then list every variant to pass them on.

**Fix:** `OpenEvent::Received { message: Up }` beside the transport variants, and a `protocol::to_input` shared by hosts. Rename it `ChannelEvent`: it is not about opening, and `smith_protocol_channel::OpenEvent` is a different type with the same name. **Size:** ~-150 lines across the area and its users.

### 3. `Component::drain` is a 280-line, 16-level dispatch with bare kinds
**Taste, Elegance.** `component.rs:383-662`, under `too_many_lines`, reaches 16 indent levels. 14 hex kinds appear as literals (`0x0109`), though the wire has a `Body` enum (`smith-channel/src/kinds.rs`). 9 arms repeat `if self.phase == Phase::Admitted`, and each kind repeats the same decode-or-refuse shape. The text `b"invalid answer"` is sent 19 times, also for turns, calls and notices that are not answers. Other bare values: `reason: 2` (`:400`), `for _ in 0_u8..8_u8` (`:386`).

**Fix:** named kinds; one handler per kind returning `Result<OpenEvent, Refuse>`; a single phase gate and a single refusal site. **Size:** ~-100 lines, and depth from 16 to about 5.

### 4. `Agent` is a phase plus 19 loose fields, against 5.4
**Elegance, Complexity.** `domain.rs:89-130` has 12 bools and 7 Options under `#[expect(struct_excessive_bools)]`, and code guards on `agent.phase` by comparison at 9 sites. Rule 5.4 says no field is meaningful in some states only, yet `start`, `process` (`expect("spawned process")` ×3), `until`, `owed` and `exited`+`exit` are exactly that. `told`, `last_word` and `dropped` overlap. There are two orthogonal lifecycles, the run's words and the process tree, folded into one `Phase`.

`step` fights its own shape: Spawn is matched, returned early, then `unreachable!` (`:241`) and re-matched in `spawn` (`:434-457`); Sent/Unsent re-match the whole event space to get one bool (`:297-320`).

**Fix:** split `Input::Parent` and `Input::Process` into two functions that bind fields in their arms (~-60 lines now); then two state enums with data, on the model of `stop.rs::Stage`. Move `valid_start` and helpers (`:533-684`) and the record rules (`:978-1121`) to their own modules. **Size:** ~-100 lines, 5 or more bools gone (estimate).

### 5. `domain.rs` repeats checks and overloads types
**Complexity.** `receive` runs `too_large` (`:831`), then `valid_record` tests the same sizes again (`:1016-1029`, `:1076`, `:1083`, `:1118`), so those branches can never fail. `busy: Queue<Sent>` only ever holds `Sent::Busy`, which costs two `unreachable!` (`:1047`, `:1273`) and two matches. `CallStage` and `TurnStage` are one enum. `known_read`, `mark_read` and `valid_spend` forward one line each. `relative` (`:665`) is a third copy of `run::delivery::relative_path`.

**Fix:** remove each; `Queue<BusyCall>`; export the path check from `smith-domain-run`. **Size:** ~-70 lines.

### 6. Dead surface still carried
**Cleanliness.**
- `Domain::facts_lost()` (`domain.rs:195`) is `const 0`; its doc admits the name stays for readers, and only two tests read it.
- `Limits::facts` ("Legacy", `limits.rs:73`) caps nothing, yet every fixture and `smith-local-service/src/profile.rs:93` set it.
- `Report::BYTES` and `ModelFault::BYTES` have no users.
- `cancel_sent` is write-only.
- `Process::kill_if_spawned` has no caller.
- `DeliveryStatus` and `Delivery::status` go unused through this crate.
- `Seen.bounces` is a subset of `refused_messages`; `Seen.rejected` of `account_notices`.
- Forwarders: `configuration_value()`, `root_requests_routed`.

**Size:** ~-50 lines. Mechanical.

### 7. Frame encoding is pasted six times
**Economy.** `component.rs:235-245, 282-292, 321-331, 345-356` and `translate.rs:125-129, 295-302` repeat measure, write, `frame_writer(0x01..)`. All `send_*` end with the same down, drain, fire tail, and 8 `Process::send_*` (`process.rs:153-231`) only forward. `Error::MissingValue` (17 sites) means list overflow and length failure as well as a missing value, which its doc does not cover.

**Fix:** generate `fn frame(&self)` with its kind on each `smith_channel` record, which is the only route without generics, plus a private `send_frame`. **Size:** ~-60 lines.

### 8. Inline routing lives in `translate.rs`, against its own doc
**Organization.** The doc says "No state", yet `route` (`:348-556`) mutates slots and is half the file's logic. After finding 1 the rest nearly vanishes, so move `route` to `route.rs`.

Near-copies around it:
- `assert_eq!(slot.logical_run, ...)` ×9 in that one match, with one message;
- `Slot.turns: Map<u32, u64>` whose value is never read, so a `Set`;
- `Machine`'s variants each hold `{cancelled}`, with four parallel arms at `workspace.rs:123-150`;
- the live-or-settle block ×3 (`domain.rs:326-332, 344-352, 383-389`), and `facts_drained = false` pairs ×3;
- `if unanswered` three times in 14 lines (`stop.rs:53-66`).

**Size:** ~-70 lines.

### 9. Lint suppressions that hide a simpler form
**Taste.** Four module-wide `#![expect(clippy::manual_let_else)]` in the inline crate, plus two item-level ones in `smith-host-protocol`, keep `match ... None => return` (11 sites in the inline crate). 10.2 allows `let ... else` on `Option` and `Result`, and `smith-host-domain` uses it. The stated reasons ("checked routes remain exhaustive") are not reasons: exhaustiveness on `Option` is untouched. **Fix:** apply the lint, drop the attributes. **Size:** ~-30 lines.

### 10. Comments carry banned words and history
**Cleanliness.** Root AGENTS.md, 2 bans history and emphasis. Counts: 101 "actual" or "genuine" across production and tests, including assertion text; 19 "V1/V2", including both crate descriptions, where one format exists; `Temper 19735a06` ×2; `OLD_REFUSAL` and "T1 first refused" (`smith-inline-agent/src/tests.rs:294-317`). `smith-inline-agent/src/lib.rs:10` says "the later stop increment adds" grace and drop, which exist now. "Capability" and "rights" appear 12 and 54 times in production; the design uses "kind of agent" and "terminal" (one hit in host.md). **Fix:** one sweep. **Size:** neutral.

### 11. Host-domain unit tests spell out enums they don't care about
**Economy (tests).** `smith-host-domain/src/tests.rs` lists all 17 `parent::Request` variants 5 times (`:114`, `:195`, `:242`, `:305`, `:365`), and `tests/host/tests/suite/memory.rs:134` does the same. Test crates may use `_` and `let else`, as `smith-inline-agent/src/tests.rs` does 14 times. The Spawn, Spawned, drain preamble is retyped (`:62-70`, `:187-192`, `:394-402`) beside `admitted_with_limits`. `an_unread_charter_version_is_a_distinct_agent_refusal` tests `derive(PartialEq)`. **Size:** ~200 of 651 lines.

### 12. `tests/host/tests/suite/simulation.rs` breaks section 10
**Economy, Taste (tests).**
- Rows selected by label: `for field in 0..6 { match field`, `for mutation in 0..4`, `for mode in 0..3 { if mode == 0` (`:156`, `:621`, `:754`, `:1278`), `for settle` (`:373`).
- 13 of 49 tests build more than one world, so they hold several behaviours; `the_watchdog_pauses_...` (`:697`) has five stories.
- 35 bare sizes (`charter_value(257)`, `vec![0; 257]`, `fields_value(129)`) for limit+1.
- `cleanup(Some(Exit::Code(0))); settled();` appears 29 times beside `finish` and `faulted`.
- Module doc: "Preserved Temper 19735a06".

**Fix:** a `clean_end` helper, named sizes off `limits()`, split the mode tests. **Size:** ~-120 of 1,464 lines net, with a larger clarity gain.

### 13. Fixtures retyped and inline world noise
**Economy (tests).**
- The `Limits` literal appears three times in `smith-host-protocol/src/tests.rs` (`:13`, `:105`, `:140`) and again at `component.rs:723`, though `process_limits()` is named in the same file. ~-55 lines.
- `seen[&Token::new(1)]` ×36 in `simulation/inline.rs`, and the `for _ in 0..100` until-answer loop ×3 plus one in `referee.rs`: add `const CLIENT` and `run_until_answered`. ~-60 of 648 lines.
- `a_drop_after_an_answer_...` and `a_message_past_the_runs_bound_...` run both kinds in one body.
- `smith-inline-agent/src/tests.rs:9-67` near-copies `tests/host/src/inline.rs:27-95`. Its dev-dependency cycle forbids importing the world; move the fixtures into `smith-agent-world`, where `LIMITS` lives. ~-50 lines.

### 14. `Process` failure handling tricks itself
**Taste.** `process.rs:287-289` sets `Phase::Spawning` so that `fail` will not return early. The `Failed` and `Exited` arms (`:300-331`) duplicate their close steps. `keep_tail` rebuilds a `List` on every stderr chunk. **Fix:** a `tree_ended` helper and an explicit transition. **Size:** ~-25 lines.

## 5. Cross-area suspicions

- **`Cap`, `Limit` and `Phase`** are also in `smith-domain-session/src/llm.rs` and the generated `smith_channel`, so four definitions; `smith-domain/src/translate.rs` is probably finding 1 again.
- **`smith-protocol-channel/src/component.rs`** shares the names `Component` and `OpenEvent` and uses `frame_writer`; findings 3 and 7 likely recur.
- **`smith-local-service/src/process.rs`** (1,974 lines) has three more `OpenEvent` lists (`:981`, `:1250`, `:1305`).
- **`start_bytes: u32::try_from`** limits literals recur in `tests/channel`, `smith-agent-service` tests and `smith-protocol-channel` tests.
- **`Limits::facts`** flows through `smith-local-service`'s `HOST_FACTS` and `smith-local-domain/src/child_backlog_tests.rs:47`.
- **Tech debt:** "Volume and churn" shows here as near-copy tests and fixtures; "Strict-subset exceptions" as 18 `expect`s, of which findings 4 and 9 remove about eight.

## 6. Rule costs

- **No generics:** `values.rs` is five identical sealed wrappers, about 30 lines each, so about 120 lines stand. Record encoding likewise cannot share a helper, which is why finding 7 needs the code generator.

---

# 10. Local host

## 1. Scope read

**Closely:** `smith-local-domain` (production except the end of `config.rs` and `limits.rs`; `tests.rs`, first third of `parent_tests.rs`); `smith-local-protocol` `store.rs`, `store/machine.rs`, `compatibility.rs`, `delivery_store.rs`, `git.rs`, `terminal.rs`, schema; `smith-local-service` most of `service.rs`, `ending.rs`, `choices.rs`, half of `process.rs`'s production code; `tests/local` `world.rs`, `referee.rs`, a third of `simulation.rs`.

**Skimmed:** `git_child.rs`, `plain.rs`, `markers.rs`, `failure.rs`, `qa_store.rs` and `suite/store.rs` (first 120-215 lines, then by test name), `src/tests/store.rs`, `credentials/*`, `program_check/*`, `live.rs`, `stream.rs`, `profile/*`, service `tests.rs`, `tests/local/src/{git,person}.rs`, `fuzzy/simulation.rs`, the drift test's local-store case. Goldens not opened.

## 2. Scores

| Criterion | Score | Keeps it from the next score |
|---|---|---|
| Cleanliness | 3 | Dead state and variants, unused dependencies, a crate doc that describes one module of eight. |
| Organization | 2 | `domain.rs` and `process.rs` each hold several subsystems; most host changes pass through one of them. |
| Taste | 3 | Lower machines show care; `git_result` and the two big owners do not. |
| Naming | 3 | Design words are kept, but `ProcessAdapter`, `remote`, `AgentLife`, and four `Step`s mislead. |
| Primitives | 3 | Narrow typed boundaries; the three central owners are flat bags of fields. |
| Elegance | 2 | The central machines do not have the shape of their state tables. |
| Economy | 2 | About 2,000 lines could go: ~900 production, ~1,150 test. |
| Complexity | 3 | No speculation, but much state is derivable or held twice. |

## 3. Strengths

- **`git.rs` is the area's model:** a `Step` whose variants hold their data, `mem::replace`, a total match, and a table in the module doc. `markers.rs` and `plain.rs` follow it.
- **The domain boundary:** `boundary.rs` gives every event and request a typed shape and a stated terminal; the service routes it without interpreting it.
- **The referee (`tests/local/src/referee.rs`)** is built only from observations, and `suite/referee.rs` proves with 21 bad histories that it fails.
- **Store generation:** a 20-line schema, a thin `format.rs`, and fingerprints and code checked by one drift test (`tests/codecs/tests/suite/drift.rs:85`). `credentials/` and `program_check/` are well-sized modules. Two soft spots only: `DeliveryAnswer` wraps a single field (schema line 16; codegen may require it, unsure), and `1_u32 << 20_u32` at `machine.rs:222,535` is a bound with no name.

## 4. Findings

**1. The chat machine is a flat bag, not state values.** *Primitives, Elegance.*
Evidence: `Domain` has 38 fields (`domain.rs:26-65`). `Phase` holds no data apart from `gone` (`chat.rs:11-26`). What each phase owns sits beside it: `grants`, `grant_failed`, `head_survey`, `heads`, `run`, `wall_deadline`, `cancel_requested`, `turns.answer`, `turns.detail`. Five overlapping end fields (`end_cause`, `agent_failed`, `stop_failed`, `agent_life`, `cancel_requested`) appear on 54 lines. `phase` is assigned 26 times across `domain.rs` and `person.rs`, compared by `==`/`!=` 18 times, and the `mem::replace` idiom is never used for it. Chat's four flags have 37 uses, under an `expect` for excessive bools (`chat.rs:36`).
`Phase::Answered { gone }` is written twice and never read (`domain.rs:1272`, `person.rs:531`): it duplicates `agent_life`.
Why: this breaks programming-model 5.4 ("no field is meaningful in some states only"). The 7×18 table is implicit in early returns; `lib.rs`'s 15-row table can't be checked against it.
Fix: `Phase::Starting { grants, survey, .. }`, `Running { run, .. }`, `Ending`, `Answered { answer, detail }`, one transition idiom, and one `End` value for the five fields.
Size: about 15 fields leave `Domain`; ~150 lines net, more in what later changes avoid.

**2. Delivery lives in `domain.rs`, as a 300-line function and two parallel machines.** *Organization, Elegance, Economy.*
Evidence: `delivery.rs` is 221 lines of helpers; the delivery transitions are `domain.rs:1347-2135`, about 790 lines of a 2,147-line file. `git_result` (`1520-1818`) sits under `too_many_lines` and runs three `if in_place.step == …` pre-passes before `match in_place.step`, rewriting `result` on the way.
`Step` holds nothing while `uncertainty` and `marker_head` wait in `Option` fields (`delivery.rs:49-50`). `Reconcile` reimplements inspect and push with two bools (`domain.rs:85-86`) beside `Step::InspectPush`.
`domain.landed` duplicates `in_place.receipts`: both are pushed at the same two sites (`1459-1466`, `1740-1744`).
Six seven-line `delivery_failure(… TooLarge …)` calls appear, three behind one identical head check (`1608,1643,1660`). `delivery_failure` and `delivery_failed` differ by a letter.
Why: the riskiest logic in the host is its least reviewable; `git.rs` shows the right form.
Fix: a data-holding `Step` shared by delivery and recovery, in `delivery.rs` plus `reconcile.rs`, one handler per cell, `landed` dropped.
Size: ~250 of 790 lines; `domain.rs` falls to about 1,100.

**3. `ProcessAdapter` is the host's IO hub under a process's name.** *Naming, Organization, Primitives.*
Evidence: its doc says "One spawned agent's IO" (`process.rs:1`), yet its 46 fields and 43 functions own the terminal, store, git, markers, plain snapshots, program checks, file layer, signals and trace. In inline mode it runs with no agent process: `Effects::Inline { agent, local: ProcessAdapter }` (`service.rs:71`).
Both `Effects` variants hold one, so the enum needs four accessors and 37 production `process_mut()` calls. `Service` adds 32 fields. `stream: Option<…>` exists only when spawned, and `self.remote || headless` guards recur at three sites.
Fix: rename it (for example `HostIo`), and move delivery IO and store routing into their own owners. Then `Service { io, agent: Option<Effects>, .. }` removes the enum.
Size: ~80 lines from `Effects` and its matches; the larger gain is three files instead of one.

**4. The store's `Phase` has 18 dataless variants listed 230 times.** *Elegance, Economy.*
Evidence: 230 `| Phase::` lines (186 in `machine.rs`, 28 in `compatibility.rs`, 16 in `ephemeral.rs`). This is not forced verbosity: the machine is three short sequences (compatibility, load, fresh-history), and the flat enum makes every match list all of them. `loading`, `fresh`, `index`, `entries`, `window` and `writing` belong to single sequences. `path()` guesses `state` for most phases (`machine.rs:869`), and `record()` falls back to it.
Fix: nested `Phase::Compatibility(..)`, `Load(..)`, `Fresh(..)`, `Write(..)` holding their data; each match then has about five arms.
Size: ~170 of 1,384 lines in `machine.rs` and its modules.

**5. Two real-directory store suites test the same thing.** *Economy.*
Evidence: `tests/qa_store.rs` (21 tests, 395 lines) and `tests/suite/store.rs` (11, 331). By name, eight of the eleven have a twin: `every_unrecorded_old_record…`/`unrecorded_records_refuse…`, `empty_create_is_durable…`/`empty_directory_gets_marker…`, `only_regular_exact_reserved_residue…`/`near_miss_temporary_names…`, `selecting_a_different_root…`/`selecting_another_chat…`, and so on (the pairing was read closely for three, by name for the rest). `state`, `marker` and the refusal helper are retyped. `qa_store.rs:1-3` records how the file was written ("production and core tests are unread"), not what is true of it. It is a separate binary sharing `support/disk.rs` through `#[path]`.
Fix: one suite, keeping the stricter case of each pair.
Size: ~300 of 726 lines.

**6. The world's fixture is a constructor per story and a 300-line branch.** *Economy, Complexity.*
Evidence: 23 constructors (`world.rs:258-398`) and 7 in `headless.rs`. There are 21 calls of `with_capacity(seed, …, true, 2, 64, Scenario::X)` with bare sizes, and paired `with_x`/`with_x_store` for six scenarios. `with_mode` (`460-762`) restates scenario groups in `matches!`: the git group at 509, 521, 566, and `instructions` is a ten-arm `if` chain. Across the suite: 115 bare `drive(600)`-style counts, 34 `into_store` crash stories, 24 `windows(..).any(..)` scans with no helper.
Why: a new scenario edits several scattered lists. The bare counts break "named fixtures and sizes".
Fix: make `Scenario` public with a function returning a facts struct; `World::of(scenario, seed)` and `.resumed(store)`; named `ENOUGH_ITERATIONS`; helpers `crash_then_resume` and `saw_text`.
Size: ~350 of 1,758 in `world.rs`, ~150 of 848 in `simulation.rs`.

**7. Fixtures retyped across inline tests.** *Economy; testing-strategy 10.*
Evidence: a 16-line `Turn` literal at seven sites (`person.rs:331`, `tests.rs:344,625`, `parent_tests.rs:207`, `protocol/store.rs:234`, `suite/store.rs:234`, `service/tests/output.rs:115`). `local::Config` and `local::Limits` are each typed four times (`tests.rs`, service `tests.rs`, `world.rs`, `tests/local-process`). An empty `Loaded` setup at 17 sites. `parent::Request::Gone` 11 times and `client: Token::new(1)` 35 times in `parent_tests.rs`. The simulator pump is copied (service `tests.rs:350,411`).
Rows branch on labels, against "rows carry data": domain `tests.rs:219-226,322-327,570-576`; `parent_tests.rs:39-85`, whose `store_first` picks both code and expected value.
19 test lines assign domain fields directly (`domain.chat.phase = …`, `parent_tests.rs:17`), which pins internals and will cost more once finding 1 is done.
Fix: a shared turn and config fixture in `smith-local-world`, `loaded_service()`, `gone()`, split the branching tables.
Size: ~350 lines, ~150 of them in `parent_tests.rs`.

**8. Byte helpers reimplemented, one against skein.** *Primitives, Economy.*
Evidence: `skein_lib::Decimal` exists and is used (`domain.rs:128`, `failure.rs:24`), yet `decimal` is hand-written at `delivery.rs:157`, `terminal.rs:329`, `git.rs:338`. Fixed-width digits twice (`store.rs:87`, `delivery_store.rs:343`). Five byte appenders (`delivery.rs:136,178`, `choices.rs:87`, `person.rs:295`, `terminal.rs:288`); the last two contain the same UTF-8-boundary cut. The trailer format lives in two crates, three places (`delivery.rs:147`, `git.rs:157,305`), so a format change can silently break recovery. The `commit ` receipt is built three times (`git.rs:247`, `domain.rs:1554,2029`).
Fix: use `Decimal`; add a character-bounded append to `skein-lib`; one trailer function exported by the domain.
Size: ~120 lines.

**9. Person-facing text has two routes.** *Primitives.*
Evidence: failures are typed `Notice`s rendered in `terminal.rs`, but the domain also composes literal text: `choices.rs:32,66-73`, `person.rs:215-286` (`Tool:`, `Tool finished`), `agent.rs:333-335`. `Service::notice` then patches around it with `effort_notice` (`service.rs:366,1033`). `Domain::tool_verdict` and `result_text` are selfless forwarders that expose rendering (`domain.rs:143,149`). `Older::words` holds three sentences of about 450 bytes that differ by a phrase (`boundary.rs:142-153`).
Why: host.md 8 asks for exhaustive words per kind; the model says step code does not format.
Fix: typed `Notice` variants; rendering in `terminal.rs`; split the shared tail of `words` (shell.md 5.3 fixes the text, so check what reads it).
Size: ~100 lines moved, ~30 saved.

**10. Service copies.** *Economy.*
Evidence: `local_pass` matches the same event twice in a row (`service.rs:1295-1344`), with `continue` to dodge a trailing step; one match returning "forward" saves ~30 lines. `flush_submissions` is three near-identical loops (`846-872`), and two more at `process.rs:1102-1117`; a free function takes the queues. `end_work` and `advance_end` hold the same predicates (`ending.rs:24-39,136-151`). `Ending` borrows the agent's `Step` and so needs `unreachable!` twice for `Usage` (`ending.rs:50,175`): it should own three variants.
Size: ~90 lines.

**11. Small cleanups, many in few lines.** *Cleanliness.*
- `deliver_to_agent` takes `_out` and ignores it (`domain.rs:1928`).
- `plain_status` branches to pass `true` or `false` (`1507-1511`).
- `AccountFailure::{Unavailable, NotRefreshed}` are never constructed outside rendering tests.
- `Lines::fence` and `refused` return values only tests read.
- `skein-json`, `smith-charter` and `smith-channel` appear in the protocol manifest and in no source.
- The protocol crate doc describes only the terminal (`lib.rs:1-14`).
- `Terminal::notice_text` and a free wrapper of the same name (`terminal.rs:113,227`).
- `Token::new(0)` and `Token::new(1)` are unnamed sentinels at ten sites.
- `process.rs:174,189,1391,1408` hold bare capacities outside the profile (`64 << 20`, `4096`); shell.md 2.1 places the local layers' share in the profile.
- `person.rs` has production code between and after its two test modules (`369,377,519`); four test-layout conventions coexist.
- Comment words AGENTS.md section 2 bans: `domain.rs:141,147,324`, `end.rs:39`; `git_child.rs:5` cites a plan file outside the tree.
- `Checks` hand-writes `Debug` (`program_check.rs`, 10.3) where siblings use `expect(missing_debug_implementations)`.

## 5. Cross-area suspicions

Unverified outside this area.
- **Parallel vocabularies:** `run::Delivery`, `host::Delivery` and the wire form (`agent.rs:114-164`, `delivery_store.rs:298-326`) and `inline::{Below, Lower}` against `agent::{Event, Request}` (`service.rs:1387-1446`) are field-for-field copies. Look at `smith-host-domain` and `smith-inline-agent`.
- **Kernel operation multiplexing** appears three ways: bits (`service.rs:1282`), `ROOT_FILE` (`credentials.rs:23`), a route map (`process.rs:120`). Perhaps `skein-io`'s to own (unsure).
- **`.temper/pre-pr`** names a downstream application in smith fixtures here and in `tests/agent` and `smith-domain` tests.
- **`smith_events::clip`** is a third truncation (`crates/smith-events/src/words.rs:13`).
- **The `Turn` literal** likely recurs in other smith test crates.

## 6. Rule costs

A match acting on one variant lists all the others: `writer_up` names 13 `io::Event` variants to treat `Closed` (`process.rs:949-963`); `drain_process` spends about 25 lines to skip `WriteFailed` (`980-1006`). Rough cost: 300 lines beyond finding 4. Low priority.

---

# 11: local shell and local-process tests

`LS` = `crates/smith-local-shell`, `LP` = `tests/local-process`. Nothing was built or run. Only code in the clone was read; the tokens and keys seen there are test fixtures.

## 1. Scope read

Size: `LS/src` is about 13,600 lines, of which about 4,400 are tests (estimated). `LP` is about 7,050 lines: 4,960 harness and 2,090 suite.

**Closely:** `local_host.rs` and submodules; `compose.rs`, `compose/*`; `check.rs`, `check/*`; `credentials/` production code; `settings/mod`, `layers`, `schema`, `environment`, `resolve`, `resolve/document`, `inherit`, `input`, `models`, `programs`; `LP/src/world.rs`; the `make_local` part of `process.rs`; `credentials/tests.rs`; `check/catalog_tests.rs`, top; `settings/tests.rs`, first 230 lines; `LP` suite `lending`, `replay`, `memory`, `configuration`, `fuzzy/world`, and the top of `simulation`, `authentication`.

**Skimmed:** `front/*` (structure only), `command.rs`, `presets.rs`, `resolve/relationships`, `providers`, `local`; `referee.rs` (key parts); `terminal.rs`, `git.rs`, `oauth.rs` (outlines); `fuzzy/store.rs`, `driver/tests.rs` (tops); remaining inline tests (sampled or counted).

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---:|---|
| Cleanliness | 3 | Comments are mostly good; dead paths, unused parameters and 20 dangling test citations remain. |
| Organization | 4 | Cohesive modules, one file per source; five test layouts and two suite files in the wrong crate. |
| Taste | 3 | Secret handling shows real care; the world's sentinels and bool tuple do not. |
| Naming | 4 | Follows the design; `Source` names a settings layer, a credential source and a service source, and `Files` names three types. |
| Primitives | 3 | `Directory`, `Lent`, `Credential` are right; accounts stay string maps that every consumer re-reads. |
| Elegance | 3 | `Local`'s passes follow the problem; startup and the fake disk mirror do not. |
| Economy | 3 | About 1,300 lines could go, about 6%; the weight is in test setups. |
| Complexity | 3 | Little speculation in production; forwarding ladders and the test world's flags add avoidable state. |

## 3. Strengths

- **Redaction by construction.** `Source`, `Environment`, `Directory`, `Paths` and `Settings` have hand-written `Debug` that prints no contents (`LS/src/credentials.rs:148`, `credentials/login.rs:17`, `credentials/directory.rs:12`, `settings/mod.rs:59,113`). Failures collapse to fixed words (`credentials/sign_in.rs:168-174`, `credentials/key.rs:38-47`). Keep this.
- **Small, sharp primitives.** `local_host/lend.rs` (51 lines) and `credentials/directory.rs` each do one thing, with narrow interfaces.
- **Model tests.** `credentials/state_tests.rs:7` is rows of data under one body. `LP/tests/suite/live_referee.rs` edits records and asserts how many changed, as section 10 asks.
- **Refusals keep their origin and typed sides** (`settings/mod.rs:121-167`), so an operator line names file, line and key.

## 4. Findings

**1. Account facts stay strings after validation.** *Primitives.*
`Settings.values` (`settings/mod.rs:110`) is a flat string map, and `Account.fields` (`settings/resolve.rs:30`) keeps that shape after `checked`. Production builds 49 keys with `format!` and walks 11 prefixes. `named_field` exists, yet six sites decode names by hand (`credentials/login/provider.rs:103`, `settings/environment.rs:26`, `settings/resolve.rs:207`, `resolve/inherit.rs:36`, `resolve/programs.rs:34`, `resolve/document.rs:92`). Source and provider are compared as text at `compose/placement.rs:8`, `check/accounts.rs:13`, `credentials/program_check.rs:21`, `resolve/local.rs:112,193`, `credentials.rs:178`, `sign_in.rs:20`, `login/provider.rs:46`.
- "Codex" is derived three ways: `credentials/product.rs:27`, a key scan at `credentials/login.rs:88-98`, and a hard `false` at `check.rs:394`.
- An account is assembled three ways: `settings/resolve.rs:188-195`, `login.rs:78-84` (with `number: 0`), `account_report.rs:56-70`.
- `credentials.rs:177` turns any unknown text into `SignIn`, and missing directories into sentinel paths (`"missing-key"`, `:173,187`), which `key.rs` then detects by `is_absolute`.
- `:179` builds a whole registration to read one constant.

*Why:* a new source or provider touches about eight text comparisons the compiler cannot see. *Fix:* in `checked`, parse each account once into a typed `AccountSource` and `Option<Provider>`, with one `accounts(&Settings)` constructor. Make `directory` required at the source boundary. Move the path-key helpers into `settings`. *Size:* about 150 lines out of `credentials/` and `settings/resolve`.

**2. The test world selects behaviour by sentinels and flags.** *Taste, Primitives.*
- `World::configured` takes `behavior: (bool, bool, bool)` (`LP/src/world.rs:599`), at seven call sites such as `(false, true, false)`.
- Faults switch on by seed range: `launch.seed >= 100` (`:760`).
- Answer text doubles as a mode: `b"Crossed wait"` arms a referee action (`:793`), and `b"After three tools"` and `b"Queued while starting"` choose scripts (`:1078,1109`).
- `Launch` carries 11 bools and nine `#[serde(default)]`s (`process.rs:32-68`), though its two constructors and sole writer are in this crate.
- `Meeting::observe` runs 242 lines (`referee.rs:97-338`), with 16 `matches!` on `Ending`. A chain of 12 suffix strips decides allowed stderr (`:200-250`).

*Why:* this breaks "the body never branches on a row's label"; renaming an answer changes a story silently. *Fix:* a named `Behaviour` struct; explicit `Scenario.faults`, `line_on_wait` and `scripts`; split `observe` into one function per `Observation`; drop the serde defaults. *Size:* about 120 of 2,000 lines, with a large clarity gain.

**3. Suite stories retype their frame.** *Economy.*
Counts in the 2,090-line suite: 20 `for placement in [..]`, 33 `assert_passed(seed)`, 28 `Exit::Code(0)`, 18 `Ending::Report(b"First answer"...)`, and 26 `.windows(` byte searches in suite and harness, 18 with a bare length (`simulation.rs:153`, `.windows(34)`). `Event::RunCompleted` extraction appears three times in `simulation.rs`.
The retyped seed has already drifted: `World::authenticated(21, ...)` is reviewed with `assert_passed(20)` (`authentication.rs:37,43`), so a failure would name the wrong seed.
*Fix:* `World::assert_ended(Ending)` using the world's own seed; `const PLACEMENTS`; a `contains(hay, needle)` over `skein_lib::bytes::find`; a `run_completed(records)` helper. *Size:* about 200 of 2,090 lines.

**4. Inline settings fixtures are retyped.** *Economy.* A setup written once.
- `tests/protocol-llm/fixtures/root.der` is joined at 13 sites in `LS/src`.
- `.unwrap_or_else(|why| panic!("{}", why.line()))` appears 37 times.
- `settings::Source { .. }` is spelled at about a dozen test sites; `SMITH_FIXTURE_TOKEN` at 12.
- `Resolved` is hand-built twice, with one identical lint exception (`check.rs:571-605`, `credentials/tests.rs:24-60`).
- `MEMORY = 2_281_701_376` is retyped (`check/program.rs:9`, `check/catalog_tests.rs:89`), though it is the profile's value.
- `no_source` branches on its input (`credentials/tests.rs:122`).

*Fix:* one `fixture` module in `LS` with `settings(toml)`, `TRUST`, `ready(result)` and `resolved(files)`. *Size:* about 250 of 4,400 test lines.

**5. Whole-file startup stores are written twice, in the wrong crate.** *Primitives; skein-shell.*
`credentials/private_file.rs` (a 156-line `Writing` host) and `compose/files.rs:205-224` (`write_startup`) both drive open, store and close on a private kernel. skein-shell offers `read_private_file` and no write. `ResultFile` also does three jobs: the result, the generated agent configuration, and removing the ephemeral directory (`:249-259`, where a parent-open error is reported as `Failure::Close`). The name fits one.
*Fix:* a `skein_shell` startup store; keep `ResultFile` for results; give cleanup its own small host. *Size:* about 180 lines.

**6. One source's inspection is written three times.** *Economy.*
The env arm's "usable, expired or unset" logic appears at `credentials.rs:197-205`, `credentials/product.rs:97-105` and `credentials/login.rs:115-131`. `sign_in::record` and `inspect` each read and decode the record (`sign_in.rs:168,185`). The `chatgpt_account_id` selector appears twice (`credentials.rs:230`, `sign_in.rs:43`). `check.rs:388-409` rebuilds part of `prepare`.
*Fix:* one `env_state(value, codex, wall) -> Credential`; `inspect` built on `record`; a named selector constant. *Size:* about 50 lines.

**7. Dead paths and forwarders.** *Cleanliness, Complexity.* Each verified by search.
- `check.rs:283-288`: the `None` arm cannot run, because `local` is always `Some` (`:270-273`).
- `resolve(_, _command)` (`settings/resolve.rs:52`) ignores its argument, yet `check.rs:81-84` clones an `Exec` to build it, and six more call sites type a `Command`. `credentials::source` has an unused `_settings` (`credentials.rs:164`).
- `private_source_at`'s `inherited` is never `Some` (`sign_in.rs:218-223`).
- `Local::new`, `select_chat` and `front_lost` have no caller, yet the module doc names `Local::new`.
- `compose.rs:575-579` branches between two identical calls; `open` then has no other caller.
- `struct Composed { local }` (`compose.rs:160`), `derive_execution` (`check.rs:314`), `Files::prepare_stream` (`compose/files.rs:77`) and `credential_words` (`login.rs:48`) add nothing.
- `provider::Registration` is built from a bool (`login.rs:101`) and turned back into one (`login/provider.rs:56-63`). `logout: bool`, `Operation` and `Stage` hold one fact.
- `profile_refusal` is duplicated (`local_host.rs:921`, `startup.rs:80`), with three more inline spellings. `Refused::Local(startup::Failure::..)` is spelled 16 times, with `host()` serving only `io::Error`.

*Size:* about 120 lines.

**8. `Local` holds more state than its shape needs.** *Elegance.*
`headless` and `interactive` are two `Option`s (`local_host.rs:67-68`), unpacked from the existing `Front` enum (`:138-142`) so that "both" is representable. `with_admission` runs 193 lines under an exception. The closing condition is written twice with drift risk (`:810-820`, `:872-887`). Unnamed counts recur: `3` program sockets (`:216`, `memory.rs:29`) and `4` result operations (`compose.rs:129`, `memory.rs:33`, `compose/files.rs:437`, with `5` at `LP/src/world.rs:807`). `token_limits` lives in `local_host.rs:904`, but all nine uses elsewhere in `LS/src` are in `credentials/`.
*Fix:* hold `Option<Front>`; split startup into validate, adopt and assemble; one `settled_below()`; name the counts; move the limits to `credentials`. *Size:* about 60 lines.

**9. The fake disk is mirrored to the real disk.** *Elegance, Complexity.*
`Machine::step` lists a real directory and writes the token record out on every kernel call (`LP/src/world.rs:351-371`). `configured` copies chats in (`:606-614`) and `settle` copies them back (`:896-910`). The referee then reads the real copy (`referee.rs:657`).
*Fix:* let `Files` own the fake `Filesystem` across invocations, and observe it directly. Real-loop use may constrain this (unsure). *Size:* about 80 lines.

**10. Test kit mechanisms have second copies.** *Reuse; `smith/AGENTS.md`.*
- Scratch directories: `skein_scratch::Scratch` is already a dev-dependency, yet `credentials/tests.rs:6-20`, `LP/src/world.rs:54-80`, `settings/tests.rs:698` and `tests/suite/local_library.rs:31` each make their own (about 45 lines).
- Replay: `fuzzy/world.rs:13` hand-rolls `for _ in 0..2` and `previous` where `skein_world`'s `assert_replays` exists, and picks stories by `seed % 8` (about 15 lines).
- `Proc` hand-forwards nine `Host` methods (`process.rs:474+`). `skein_shell::Delegate` needs its three overrides moved into wrapper hosts (about 30 lines).
- `LP/src/oauth.rs` is a scripted HTTP browser, as is `skein/tests/oauth-accounts/src/sign_in/browser.rs` (overlap judged from their heads only, unsure). The shared one belongs in skein-fake-peers.

**11. Comments that went stale.** *Cleanliness.*
Of 28 distinct names cited by `Lower test of ... end_to_end::x`, 20 exist nowhere but the comment (`check/catalog_tests.rs:36`, `login.rs:150`, `composition.rs:95`). The form records where a test came from, which is history: delete all 31 such lines, the dangling ones at the least. `LS/clippy.toml:1` still says "the echo's main". `compose/files.rs:154` carries "revised 2026-10-20". `actual` appears in eight comments or expects, against root `AGENTS.md`, 2 (`LP/src/process.rs:429`).

**12. Test placement.** *Organization.*
Five layouts coexist: `tests.rs`, `*_tests.rs`, inline `mod tests`, named one-test modules (`compose.rs:600,631`), and a test-only file with a production name (`check/program.rs`). `fuzzy/store.rs` (450 lines) tests store and io, and `suite/configuration.rs` tests the machine adapter; each takes only `lower_configuration` from this crate. Move them to the owners' tests and choose one layout. *Size:* no line change.

## 5. Cross-area suspicions

- `tests/end-to-end` retypes the `SavedToken` literal (`credentials.rs`, `support/live.rs`). `skein-oauth` or a shared fixture could own a builder.
- Other `tests/*` crates likely repeat the private scratch directories and hand-rolled replay.
- `document.rs:86-88` writes dialect as bare `3, 1, 2`, and `smith-agent-document/src/config.rs:627-634` then checks it against the provider: the number looks derivable.
- `GrantValue::Withdrawn` still forces an arm in `lend.rs`, though its doc in `smith-protocol-channel` says `accounts-sources` removed it.
- jig and temper may carry `Lower test of` comments and their own scripted browsers.
- `skein/tests/oauth-accounts` holds a browser in a test crate, not a kit crate (see finding 10).

## 6. Rule costs

"No file beside the ring" means every one-shot startup operation needs a hand-written `Host`: `Writing`, `Runner`, `Input`, `Login`, `ResultFile`. That is about 100 lines each, about 30 of them trait boilerplate. A skein helper for one-shot startup hosts would recover perhaps 150 lines here (low priority).

---

# 12: benchmarks

`smith/benchmarks/`: 24,218 lines in `src` (2,179 in test files, plus inline test modules), 10,215 in `tests`, 980 in task fixtures. 341 `#[test]`s. Lints: ordinary Rust under `benchmarks/clippy.toml` (no `HashMap`/`HashSet`, `Instant`/`SystemTime`, `thread::spawn`), plus `Cargo.toml`'s pedantic and `wildcard_enum_match_arm = deny`. Closures, generics and `std` are allowed here; the step-crate subset does not apply.

## 1. Scope read

- **Closely:** `run.rs`, `probe.rs`, `batch.rs`, `agent/mod.rs`, `process.rs` (head, tail), `judgment.rs`, `passive.rs`, `tests/common/*`, `tests/suite/events.rs` (head, umbrella, caches), `large_write_probe.rs`, the umbrellas in `src/tests.rs`.
- **Sampled:** `summary.rs`, `check/events.rs`, `events/{checks,slow}.rs`, `parity.rs`, `validate.rs`, `isolate.rs`, `guard.rs`, `formats.rs`, and heads of `probes.rs`, `parity.rs`, `budget_visibility.rs`, `tool_output_probes.rs`, `offline.rs`, `pipeline.rs`, `overrides/suite.rs`; README; suites; design 5.1, 5.2, 13 and testing.md 2.3, 2.4.
- **By search or listing only:** the rest of `events/`, `agent/codex/*`, `credential*`, `tokens.rs`, `stats.rs`, `task.rs`, fixture and task trees. Summaries and baselines by size; `agents/smith/accounts.toml` not opened.

## 2. Scores

| Criterion | Score | Why, and what keeps it from the next score |
|---|---|---|
| Cleanliness | 4 | No TODO or dead code, three lint exceptions. Held back by stale docs, emphasis words and test-only production branches. |
| Organization | 3 | `agent/smith/events/` is excellent; `probe.rs` and `batch.rs` are grab-bags, and tests live in five placements. |
| Taste | 3 | `Measure` and strict readers show care; string errors, closure-as-`try` and tuple parameters undercut it. |
| Naming | 3 | Design vocabulary is followed; four `Prepared`s, `codex_home`, and an overloaded `Agent::ClaudeCode` mislead. |
| Primitives | 3 | Data primitives are strong; the adapter seam leaks, and behaviour hangs on task and suite names. |
| Elegance | 3 | `attempt` follows the design's phases, but agent branches and narrowing twice blur them. |
| Economy | 2 | Tests retype setup with almost no shared kit, and one fixture's history is kept whole. About 1,200 test lines could go. |
| Complexity | 3 | Mostly proportionate; test-only caches and one test-only production path add needless layers. |

## 3. Strengths

- **`Measure<T>`** (`result.rs:19`) carries "unknown is never zero" from stream to parity verdict, each gap with its reason. The clearest idea here.
- **Closed, strict author vocabularies:** `EventCheck`, `Passive` and `formats.rs` refuse with file and key path before any spend.
- **`agent/smith/events/`:** one small module per check (27 files, 5,379 lines), read only through `smith-events`.
- **One of each:** the write guard is shared with the live tests through `smith_bench`, supervision is skein's (`Binary`, `Referee`), and draws use skein's seeded `Rng`. Recorded evidence states its provenance (`tests/recorded/codex/README.md`).

## 4. Findings

**1. Test setups are retyped; `tests/common` is 368 lines (Economy).**
- About 20 local "feed records into an `Observer`" helpers: `events.rs:18,855,864`, `failure_probes.rs:182,191,281`, `parallel_probes.rs:91,135`, `probes.rs:51`, `path_probes.rs:58`, `events/carried.rs:40`, and others.
- Tool start and end pairs are rebuilt: `large_write_probe.rs:485,604`, `delegation.rs:32`, `tool_output_probes.rs:50+`.
- 100 `CARGO_MANIFEST_DIR` joins in test files, 39 `check_task(` calls, 34 reads of `results/attempt.json` across 20 files.
- 43 `temp_dir()` sites with `remove_dir_all` at the end, so a panic leaves the directory behind. `fn root` is written in three files.
- 7 `run::Prepared` literals in tests, e.g. `batch.rs:986,1068,1118`.

This breaks "a setup written once" and is the helper duplication named in tech-debt's "Volume and churn". Fix: `common::{observe, tool_pair, repository, attempt, task, request, Scratch}`, with `Scratch` a `Drop` guard. **~600-800 of 12,400 test lines.**

**2. The adapter seam leaks into the runner and supervisor (Primitives).** Design 6 says agents are adapters, yet `run.rs` has 11 `Observing` or `Prepared` branches by agent. `collect_observation` (:559) and `observe` (:608) both match `Observing`, and Codex's delegation line is set twice (:568, :653). `archive_preparation` (:739) and `source_arm` (:790) are agent logic too.

`process.rs:53,96` gives every process a `codex::Observer`, default-built for Smith (:645). `Capture::Codex` is reached in production only through `configuration: None` (`run.rs:836-851`), which `run.rs:45` calls "offline stream replays"; tests use it. `run.rs:675-679` then writes counts and tokens that `collect_observation` replaces, always for Smith and for Codex once it has a thread.

Fix: `Observing::collect(&mut result)` and `Prepared::archive(..)` on the adapters; replays become a Codex `Observing`; drop `ProcessResult.observer` and `Capture::Codex`. **~250 lines leave `run.rs`, ~60 go.**

**4. Streams are edited by position, and a miss does nothing (Naming, test shape).** There are 221 `if let Event::X(v) = &mut ..` edits and about 110 constant indices in `events.rs` alone (`values[7]`, `records[4]`). If `balanced()` moves, an indexed `if let` changes nothing and says nothing, which is what "a recorded stream edited as records" forbids; "named fixtures" is broken too.

About 10 sites edit recorded streams with `str::replace` (`codex_adapter.rs:54-63,314,369`, `summary_resources.rs:63`, `retried.rs:8`, `comparison.rs:235`). Fix: typed accessors that panic on a miss, such as `common::events::response_mut(&mut records, 0)`, and a parsed edit for JSON lines. **Small line change, a large gain in trust.**

**5. Six retired large-write versions, and tests that compare them (Economy).** `tests/fixtures/large-write/` holds v4 to v9 (539 KB, six copies of one README). `version_seven_*` (:222) and `version_six_*` (:291) assert frozen documents against each other, and `version_five_*` (:321) and `version_four_*` (:519) restate theirs, so little but those files can fail most of their lines. An 11-variant match is pasted twice (:240, :263).

Design 5.2 says results of different versions are never compared, and I found no retention requirement; git holds the history. Fix: retarget observer tests at the current task and delete the rest. **~250 of 805 lines and ~500 KB.**

**6. Behaviour is keyed on names; `probe.rs` does four jobs (Primitives, Organization).** Names in production code:
- Suites: `"smoke"` at `probe.rs:518,659,741`, `batch.rs:222`; `"parity"` at :768; `"probes"` at :771.
- Tasks: `"probes/credentials"` at `probe.rs:127,430`, `credentials.rs:12`, `validate.rs:36`, `main.rs:287`; `"probes/first-request"` at `probe.rs:962`; `"repository/shell-cwd"` at `main.rs:77`.
- Configuration: `"smith-claude-code"` at `probe.rs:300`, `agent/mod.rs:252`.

`suite.change` already shows the better form. `main.rs:75-92` computes `codex_lower_bound` from criterion names, and that belongs in `parity`. Logical attempt size is derived twice (`probe.rs:124`, `batch.rs:476`).

`probe.rs` (1,087 lines) is option parsing, planning, calibration and reporting. Suites are also narrowed twice: as `toml::Value` (`probe.rs:422`) and then as `Suite` (`batch.rs:441`). Fix: authored fields (a suite's `verdict`, a task's `group`), then `command`, `plan` and `calibrate` modules, with one narrowing. **~100 lines go; large clarity gain.**

**7. Errors are strings, and closures stand in for `try` (Taste, Economy).** There are 185 `Result<_, String>` and 184 `.map_err(|error| error.to_string())` across 21 files, beside `Refusal`, `Setup` and `HarnessFault`. 16 immediately called closures, in those I read, attach an error or a cleanup; `run.rs` alone mutates `step` 14 times to label failures (:417-472, :1054). Fix: one `Fault` with a blanket `From`, and `at(SetupStep::Seed, result)?`-style labelling. **~150 lines, plus most of the closures.**

**8. Tests restate authored tasks (Economy).** About 100 assertions of `task.event.contains(..)` or `assert_eq!(task.*)` repeat `task.toml` (39 are finding 5's), e.g. `probes.rs:32-48`, `tool_output_probes.rs:11+`, while `check` and calibration digests already freeze those inputs. A prompt edit therefore needs a test edit, the "fixtures that pin details" debt. Keep what derives something (the chain walk at `probes.rs:15-31`) and drop restatement. **~150 lines beyond finding 5.**

**9. Tuples and threaded arguments (Taste).** `run: (&str, &str, Option<SmithFace>, &str)` is read as `run.0`, `.2`, `.3` (`probe.rs:275,298-325`); `policy: (u64, &BTreeMap)` is at `run.rs:189`. Six `run.rs` functions thread `clock`, `machine` and mostly `private` and `prepared` by hand, while `Work` (:722) exists for later phases. `task_requests` and `raw` nearly only forward. Tuples look like a way past clippy's argument limit (unsure). Fix: named structs, and a setup context beside `Work`. **~60 lines.**

**10. Names mislead (Naming).**
- `Prepared` is four types: `run::`, `agent::`, `codex::`, `smith::`.
- `Prepared.codex_home` also locates smith's login (`run.rs:869`, `credentials.rs:19-24`); call it `logins`.
- `Agent::ClaudeCode` means a refused agent (`run.rs:441,825`) and also smith on Anthropic (`probe.rs:433,654`).
- Five verdict vocabularies: `Judgment`, `ProbeVerdict`, `parity::Verdict`, `CheckVerdict`, `fix::Shown`.
- Of 419 distinct story names, 82 exceed ten words and 141 contain `_and_`, against "about eight words" and one behaviour.

**11. Small helpers copied (Economy).** Hex checks 8 times (`arm.rs:59`, `bundle.rs:57`, `check.rs:331`, `agent/mod.rs:248,264`, `faces.rs:72`, `parity.rs:329`, `change.rs:69`); name checks 4 times (`arm.rs:239,292`, `main.rs:112`, `run.rs:161`); the pin path 6 times (`suite.rs:321`, `summary.rs:814`, `check.rs:73`, `batch.rs:226`, `probe.rs:280,566`). 9 `Bounds` literals, mostly from `Prepared` (`run.rs:209,1011,1103`). The prompt rule is in `check.rs:334` and again at `run.rs:427`, with a bare 4088; `run.rs:444` and :451 both check zero. Task-variant resolution appears in `probe.rs:246`, `run.rs:381` and `batch.rs:251`. **~120 lines.**

**12. Stale text and rule breaks (Cleanliness).**
- `lib.rs:1-4` describes only task manifests.
- README says "version-three" results (`RESULT_VERSION` is 4) and `gpt-6-astra` (`models.toml` has `gpt-6.1-sol`).
- `main.rs:54`'s usage omits `--include` and `--parent-arm`; option lists also live at `probe.rs:627`.
- `clippy.toml` cites a "clock module" that does not exist.
- `rollouts.rs:65` uses `#[allow]`.
- 26 `src` comments and 15 function names say "actual" or "genuine", and 40 test strings do; `judgment.rs:2` and `fix.rs:3` cite a plan step ("00.12"), and `large_write_probe.rs:1,66` narrates history.
- Very long literals: `pipeline.rs:25,43` (over 900 characters), `overrides/suite.rs:7`, `main.rs:54`.
- Production and test modules interleave in `batch.rs` (`cache_verdicts` at :1035 sits between test modules). Tests sit in `tests.rs`, `lower_tests.rs`, inline modules, `read_tests.rs`, and `tests/overrides` included by `#[path]`.

## 5. Cross-area suspicions

- Hand-written `impl Host` child-process pumps: `process.rs:294`, `smith-agent-shell/relay.rs:219`, `smith-local-shell/credentials/program_source/runner.rs:23`. A skein-world mechanism may be missing (unverified).
- Stream fixtures built from `smith-events` golden lines plus positional edits may recur wherever tests read that stream.
- Live tests in `tests/end-to-end` and `tests/agent-process` depend on `smith_bench`; check whether they duplicate its fixtures.

## 6. Rule costs

`wildcard_enum_match_arm = deny` over the foreign `smith_events::Event`: 443 production and 138 test lines of `| Event::X(_)`, and each new variant edits every such match. The rule protects step code; this is a harness that should ignore events it does not read. `let else`, already used at `slow.rs:60`, or a small typed record iterator would cut most of it.

---

# 13: cross-area review of smith

Tree `aa95945cc`. Paths are relative to `smith/`. Nothing was built or run. Counts come from scripts over the tree; comments and generated files are excluded unless a line says otherwise. "Prod" and "test" split by file and by `#[cfg(test)]` module.

## 1. Scope read

**Closely:** all 12 reports; the design on vocabulary and layers (programming-model 4, 4.5, 5.4, 10; domain/README; host.md; protocol/README; hosts.md; limits.md; testing.md 2, 7); every manifest; the source behind each claim below.

**By script, across the tree:** line classes, test-crate overlap, function clones, members of types in 19 crates, `Limits` literals, citations, lint exceptions. **By search only:** jig and temper.

**Claims I checked and correct:**
- 09 F10: "capability" is a design word (domain/README:129-132). Only "rights" is absent from the design (0 hits).
- 09 F11: tests may not use `_`. The workspace denies `wildcard_enum_match_arm` (`../Cargo.toml:516`). `smith-inline-agent/src/tests.rs` has 0 `_ =>` and 17 `let else`. The fix stands, by `let else`.
- 08 F11: `skein_io::stage` cannot serve the two services that multiplex completions (`service/up.rs:31`, `process.rs:754`). It fits the single-owner loops.
- 04 and 06: together, nine hex sites, not five.
- 05 F9: `component.rs:667` is a `let else` on `shared::Block`, which is skein's enum, so the own-enum rule does not strictly reach it.
- tech-debt, "Strict-subset exceptions": stale. `while` and `loop` are at 0 in the seven domain crates, and 47 `#[expect]`s remain there, against the entry's 102.
- No report counted agent `Service`. At 57 fields it is the largest owner, ahead of `Backend` (54).

**Confirmed:** 02 F1, 05 F1, the orphan `tests/agent/src/wire.rs`, 08 F13, and all of 10's unused dependencies.

## 2. Scores

| Criterion | Score | What keeps it from the next score up |
|---|---|---|
| Cleanliness | 3 | The code is tidy, and citations resolve. 1,893 banned-word lines, 1,002 stamps and 130 dead citations are visible everywhere. |
| Organization | 3 | The crate split follows the design. Six production files exceed 1,500 lines, and `smith-agent-service` has three jobs. |
| Taste | 3 | The subset is held by count. 25 exceptions carry reasons that look false, and bulk-generated docs and test strings undercut it. |
| Naming | 3 | Design words mostly hold. `Phase`, `Step`, `Stage` and `State` name 33 types for one idea; "checkout" and "adapter" drift. |
| Primitives | 3 | Typed boundaries are right. skein's kit goes unused where it exists, and the host retypes the run's values. |
| Elegance | 3 | Sub-machines follow 5.4. Six of the eleven largest owners do not. |
| Economy | 2 | Limits, failure kinds and test setups are paid for at 12 to 30 sites each. |
| Complexity | 3 | Most state is earned. Lifecycles folded into one struct are not. |

## 3. Strengths

- **The subset is held.** Production step code has 0 `matches!`, 0 tuple `match`es, one tuple `if let` (`run.rs:1468`), 0 closures, 0 `macro_rules!`, 0 `impl Drop`, and 0 `while` or `loop` in the seven domains. One `while let` remains, in `service/up.rs:608`.
- **Citations are sound.** 3,350 citations: 137 dead files, 130 of them one path; 2 bad section numbers.
- **Verbatim copying is one problem, not many.** Beyond the agent and protocol-llm pair, no pair shares more than 136 lines in runs of six or more.

## 4. Findings

### Q1. Test harness overlap

**Method:** code lines in identical runs of six or more lines, then whole-file multiset overlap, then function bodies with literals abstracted.

| Pair | Lines in runs ≥ 6 | Note |
|---|---|---|
| protocol-llm → agent | 4,005 of 15,100 (support 3,162 of 4,422, 72%) | The fork. 05's 4,310 counts comments too. |
| tools ↔ protocol-llm, tools ↔ agent | 136, 122 | Mostly `world/jobs.rs`: a third copy of 268 lines. |
| hosts → fake-claude, hosts → agent-process | 124, 90 | `MachineFiles`, the io pump |
| local-process → fake-claude, agent-process | 104, 100 | Same |
| run → agent; session → agent | 63; 45 | Limits and `failure` |
| All other pairs | ≤ 89 | Mostly shared `Limits` and pump lines |

Function clones across crates, leaving out the fork: 39 groups, 531 lines. `keep_facts` and `pop_fact` are 11 lines × 4 crates, so leave them. `impl Referee`: 22. `impl Machine`: 9. `impl Host`: 32. Beyond the fork, harnesses repeat by shape, not by text.

**The cycles that decide placement:**
- A crate's inline tests cannot import a world that depends on that crate, because Cargo builds a second instance and the types differ. Four crates have such cycles: `smith-domain-run`, `smith-local-service`, `smith-local-shell` and `smith-agent-world`.
- So a fixture used by inline tests lives in the owner crate.
- `smith-protocol-llm-world` can depend on `smith-agent-world` with no cycle. The agent world's normal dependencies do not reach it, and they already link `smith-protocol-llm` through the session world.

| Piece | Copies, sites | Home |
|---|---|---|
| World, script, referees | 2 | `smith-agent-world`, with `World<B: Backend>` (test crates may use traits). protocol-llm keeps `wire`, `pool_peer` and the native-request half of the referee. |
| `world/jobs.rs` | 3 | `smith-tools-world`; the agent world already imports its `translate`. |
| `Limits` presets | about 44 full literals in four domain layers | Each owner crate, behind a `fixture` feature (Q3). |
| Charter, `Event::Start`, `Turn` | about 86, 88, 218 sites | Worlds: `smith-agent-world::fixture`. Inline: one fixture per crate. |
| Frame, charter and start edits | 57 bare kinds in agent-process | Kinds in `smith-channel`; helpers in `smith-agent-process-world`. |
| One-host io pump | 9 test files | skein: make `skein-fake-peers`' `Face` public (it is `pub(crate)`). |
| `limits()` for io | 5 identical 15-line bodies, 1 in production | skein: a preset beside `skein_io::Limits`. |
| `MachineFiles` root adapter | about 5 impls | skein: `skein-fake-machine`, a rooted `Machine`. |
| `Proc` forwarders | 0 uses of `Delegate` | Exists: `skein_shell::Delegate`. |
| Per-run `judge` | 3, one crate | One function in `smith-run-world` now; skein-world's `domain` when a second crate wants it. |

Moving the two `smith_machine_world` tests out of `smith-domain-run` (`tests.rs:5139`, `:5165`) ends its cycle.

### Q2. Parallel vocabularies

**The test I applied:** a copy earns its place if a rule requires the boundary, or if the translation changes something: a clock base, sealing, tickets resolved, content removed. Programming-model 4.1 concerns entities with different lifetimes (socket, connection, session), not values. So it does not settle this; 4.5 and protocol/README do.

| Family | Copies | Required? |
|---|---|---|
| `Cap` (21), `Limit`, `Phase`, `Evidence`, `Report`, `Prices`, `Refusal` | session, run | **Yes.** 4.5: siblings share no domain types. Costly: the run reads none of `Cap` or `Limit` (0 sites). Accept it. |
| Same, plus `CompletionFailure`, the `Delivery` family, `InvalidStart`, `tools::Outcome` | channel, events, transcript, local store | **Yes.** "Records are smith's own types" (protocol/README). |
| Root `llm::{Prompt, Block, Returned}` | session, root | **Earned.** Tickets are resolved into values. |
| `ModelFault` against `Fault` | host, run | **Earned.** `reset: Duration` against `reset_at: Time`; channel.md 9. |
| `host::Charter`, `Transcript`, `TurnValue` | wrappers | **The right shape:** a bound around the run's value, no copy. |
| **32 host types** (`channel.rs`, `delivery.rs`) | host, run | **No.** See below. |
| `OpenEvent` (host half) against `host::Up` | 13 of 19 variants | **No.** hosts.md 1 makes translation the half's job. Each host hand-writes it (`process.rs:1305`, `tests/hosts/src/lib.rs:333`). |
| `OpenEvent` (agent half) against `agent::Event` | 7 of 11 | **Probably no,** same shape; `Start` differs in its fields (unsure). |
| `Below`, `Lower` against `agent::{Event, Request}` | subsets, field for field | **No.** Four hops carry identical fields. The closed face is worth keeping: nest it in `smith-domain` as `Event::Below(..)` and `Request::Lower(..)`, and `to_below`'s 9-variant `unreachable!` arm goes. |
| `Launch` | 3 identical structs (`command.rs:15`, `host-protocol/process.rs:23`, `local-service/process.rs:28`) | **No.** No report named this. |
| `completion_failure`, `completion` projections | `responses.rs`, `answers.rs` | **No.** `smith-domain/src/translate.rs:314` already maps one to the other. |

**The key judgement: the host's copies are retyped, not layered.** 32 host types match the run's by variant, in 490 definition lines, and 22 have identical bodies.
- **For keeping them:** hosts.md 6 says "as its own vocabulary", and host.md 12 leaves a vocabulary crate open.
- **Against:** host.md 1 and 3 say "the same vocabulary" and "payloads are smith's vocabulary, as values". `smith-host-domain` is no sibling of `smith-domain`: it depends on it, and already names `run::Charter`, `run::Identity` and `Operation`, so the firewall in `delivery.rs:2` does not exist. `smith-local-domain` stores `run::Delivery` and converts it only to reach the copy (`agent.rs:114`). `host::Cap` has no reader in its own crate.

**Fix:** re-export, keep `ModelFault` and `RunFailure`. This is the designer's call. Adding one `Cap` variant now touches 13 Rust lines in 12 files plus the schema; after, 10 files. **About -700 lines net.** (09's -900 did not subtract the mapping that stays.)

### Q3. Limits

| Limit | Code lines | Files | Of which plumbing |
|---|---|---|---|
| `job_wait` (policy) | 61 | 27 | 18 production lines in 10 files; 12 fixture literals in 10 files |
| `shell_tail` (declared) | 86 | 42 | |
| `tool_payload` | 175 | 75 | |
| A wire limit (`message_label`) | 6 | 4 | plus the schema; generated code carries the rest |

The wire is cheap; the cost is above it.

Full literals with no `..`: 245, in 4,125 lines, 163 of them in tests. By layer: tools 10, session 8, run 10, host 6, machine 9.

`Derivation` is mostly a retype of `Limits`: fields copied unchanged are tools 13 of 17, machine 11 of 14, session 7 of 17, run 9 of 23 (plus 12 renames).

**A shape that keeps derivation:**
1. **One preset per layer,** in the owner crate behind a `fixture` feature, because inline tests cannot import a world. Tests write `Limits { x, ..SMALL }`. limits.md 1 allows a test to override a derived limit. This takes 12 sites down to 1.
2. **Each layer's `Limits` holds its `Derivation` by value** for what it passes through. That removes a field, an assignment and one of two inventory destructures per limit.
3. **One table for settings names:** `config.rs`, `config/profile.rs`, `schema.rs` and `document.rs` each list `job_wait`. This overlaps 08 F9.

`job_wait` falls from 30 plumbing sites to about 13. `effective/list.rs`'s exhaustive destructure stays; it is what forces a name.

### Q4. State machines

| Owner | Fields (bool, Option) | State enum | `mem::replace`, state assigns, state compares | 5.4 |
|---|---|---|---|---|
| agent `Service` | 57 (3, 16) | `Teardown`, end only | 2, 0, 0 | Composition; a mode held in `Option` |
| `Backend` | 54 (7, 19) | `Phase`, `Stop`, `WindDown` beside each other | 12, 14, 37 | **No** |
| `ProcessAdapter` | 46 (3, 13) | none | 1, 0, 0 | **No:** a hub |
| `Conversation` | 43 (1, 7) | `State` with data | 17, 22, 0 | **Partly:** compaction, final, owed sit off it |
| host `Agent` | 39 (12, 7) | dataless `Phase` | 0, 15, 19 | **No** |
| local `Domain` | 38 (2, 15) | dataless `Phase` | 2, 26, 18 | **No** |
| local `Service` | 32 (2, 7) | none | 1, 0, 0 | Router |
| `Run` | 27 (1, 4) | `State` with data | 20, 23, 0 | **Mostly** |
| channel `Component` × 2 | 24, 19 | dataless `Phase` | 0, 5, 23 each | **No** |
| `Store` | 24 (3, 6) | 18 dataless variants, 230 arm lines | 2, 17, 0 | **No** |
| machine `Process` | 17 (2, 7) | `Purpose` beside Options | 1, 0, 0 | **Partly** |

**One shared cause: several lifecycles folded into one struct.** 5.4 describes one machine per entity, and these owners are several machines:
- `Backend`: process × turn × stop.
- `Conversation`: `State` × compaction × final × owed.
- host `Agent`: the run's words × the process tree (09 F4).
- local `Domain`: chat × agent life × delivery × end, with five end fields.

No single enum fits a product, so the data stays in the struct. Every named model is one lifecycle pulled out into its own type, of 132 to 679 lines. skein's review saw the same drift (S2). **The fix is one enum per lifecycle, not one large enum,** and 5.4 should say so.

**The cost is not only reading.** `ended` confirms 02 F1. A failed child reaches `hard_stop` (`run.rs:1469`), which may set `Winding`, then returns at `:1507` with no `follow`. `deadline` then calls `unreachable!` on `Winding` (`:1569`) if the alarm fires. I read the path and did not run it; its reachability is unproven.

**One list of models, for the summary:**
- **The whole idiom:** `flight.rs`, `land.rs`, `git.rs`.
- **By-value handler:** `guide.rs` (`event(self) -> Step`), and `removing.rs`.
- **Copy state, nothing to move, so no `mem::replace` needed:** `teardown.rs`.
- **`stop.rs`** is the shape for host `Agent`, but not a model finish: it has `if unanswered` three times and a false lint reason.

### Q5. What belongs in skein

Half of what the reports ask for already exists. smith has not taken it.

| Mechanism | In skein now? | smith sites | Lines saved | jig / temper / skein |
|---|---|---|---|---|
| `u64` length | No | 199 | 0, shorter | 264 / 211 / 212 |
| Byte search | **Yes,** `bytes::find` | 186 test `.windows(` | few | 47 / 222 / 187 |
| Bare kinds, `frame` from codegen | No; `frame_writer` only | 145 kinds, 37 `frame_writer` | 250 | 11 `frame_writer` in jig |
| `Duration::as_millis` | No | 65 | 40 | 3 / 31 / 6 |
| Scratch directories | **Yes,** `Scratch` | 55 `temp_dir()`, 43 in benchmarks | 60 | 2 / 23 / 13 |
| `stage` | **Yes,** 51 skein users, 0 elsewhere | 18 files, about 13 fit | 60, plus `MAX_OUT` checks | 14 / 22 lines |
| Hex | **Yes,** `hex_digit` | 0 adopters, 9 sites | 40 | 21 temper |
| Decimal write | **Yes,** 28 adopters | about 8 left | 75 | |
| Decimal parse | **Yes,** `bytes::decimal` | 0 adopters, 5 sites | 30 | |
| `Event::owner`, `with_owner`, `is_closed` | No | 7 lists | 110 | 8 / 10 / 28 |
| `Document::member`, one-shot parse | No | 7 walkers, 6 tokenizer drivers | 150 | 3 / 5 / 4: 19 in all |
| Byte tail | No | 5 | 100 | 0 by name |
| Machine root adapter, io preset, `Face` | No | 5, 5, 7 | 150 | |
| Check-or-write drift | No | 2 | 10 | 17 / 21 / 7: temper and jig gain most |
| `Delegate` | **Yes** | 0 adopters | 30 | |
| `Map::iter_mut` | No | 3 `.nth` | 15 | |
| `skein_shell` write-file; `concat`; generated `VERSION`, `Limits::NONE` | No | 2; 8; 3 | 180; 60; 100 | |

Take adoption first; it needs no skein change. `Referee`'s defaults (D6) already landed, and no empty stubs remain in smith.

### Q6. Crate boundaries

| Item | Verdict | Fix |
|---|---|---|
| **Domain on a codec** | **New, not in any report.** The role table gives a domain lib and its children only. `smith-domain-run → smith-charter` is one constant (`charter.rs:473`). `smith-local-domain → smith-events` is `AgentExit`, `clip`, `effort_words`; it pairs with 10 F9's text in the domain. | Pass the bound down; move the words to `terminal.rs`. |
| Event projection in `smith-agent-service` | **Confirmed.** 4,600 lines; only `events`, `answers` and `calls` touch the service, at 3 cuts. The crate has three jobs: composition, profile, projection. | A `smith-protocol-events` component, the name programming-model 4 hints at. Designer's call. No lines saved. |
| Shared translation in `smith-protocol-channel` | **Confirmed.** About 1,670 lines (`charter`, `transcript`, `turn`, `result`, `credentials`) plus `Endpoints`, imported by host and local protocols and the local service. | Split it from the agent's `Component`. Designer's call. |
| `smith-fake-claude` in `[dependencies]` | **Confirmed.** Two packages also declare a `fake-claude` bin. testing.md 7 keeps end-to-end in the root for bin paths. | An optional dependency with `required-features`. Designer's call. |
| Unused dependencies | **56 by source search:** 15 in production crates (`smith-local-shell` 8, `smith-agent-shell` 3, `smith-local-protocol` 3, `smith-charter` 1), 16 in test crates, 25 in root dev. | Prune. |
| Domain worlds on protocol crates | **New.** testing.md 2 says "never protocol crates", yet `tests/session/src/translate.rs:243` calls `smith_protocol_llm::decode`, and `tests/tools/src/translate.rs:155` uses `Listing`. No status entry. | Fake them, or record the departure. |
| `Launch`, two `component.rs` | Near-twins: 144 of 842 lines in runs ≥ 4; shared `hex`, `kind_text`, `refuse_rules`. | One `Launch`; shared kinds and frame helpers. |

No domain or codec crate should merge. `smith-inline-agent` and `smith-host-domain` are two by design (host.md 9.4). Among test crates, only protocol-llm's support should collapse.

### Q7. Rule drift

| Drift | Prod | Test | Gate? |
|---|---|---|---|
| "actual", "genuine": comments | 163 | 517 | Yes |
| Same words in code, names, messages | 148 | 1,065 | Yes (hearth's precedent, testing-strategy 10) |
| History words in comments, with V1/V2 and temper hashes | 61 | 59 | Yes, word list |
| `Lower test of` lines | 3 | 95 | Yes |
| `#[allow` / `#![allow` | 1 | 27 (21 inner) | Yes. Workspace `allow_attributes` is set, yet these stand. |
| Per-field stamps | 27 | 975, plus 731 blank `///` | Yes |
| Dead citations | 5 | 132 (130 `scratch/client.md`) | Yes, by resolving paths |
| Citations of a "From temper" section | | about 805 | Goes with the stamps |
| False reasons: `manual_let_else`, `single_match` | 25 | 1 | Yes: ban those two names in `expect` |
| `while`, `loop` in domains; step crates | 0; 1 | | Yes, as an old form |
| Own-enum `if let`, `let else` | 3, plus 5 on skein enums | | Partly: a regex with `Some`, `Ok`, `Err` allowed |
| `impl From`; application trait with generics | 9; 1 trait, 7 fns | | Yes |

With no `matches-for-let-else` setting in the tree, clippy's default fires only on well-known types such as `Option` and `Result`, which 10.2 allows (clippy behaviour from memory, not run here). So all 20 "exhaustive" reasons look false; report 01 confirmed one by reading, and 09 confirmed eleven sites.

**The tool exists:** `scripts/old_forms.py` is a ratchet, in four copies (skein 234 lines; smith, jig and temper 169). It strips comments, so it needs a comment-side twin, and one root copy. Ratchet these too: `Limits` literals with no `..`, `io::resume` outside skein, `temp_dir()`, `.windows(`, `#[path]`, bare `0x01` kinds. skein's review argued the same (S5, S6).

### Q8. Naming

| Name | Means | Do |
|---|---|---|
| `Phase` (12 types, 8 crates) | 3 are LLM timing, with the wire's and the events' forms beside them; 9 are lifecycles | Rename the timing ones `CompletionPhase`, as the wire does. |
| `Phase`, `Step`, `Stage`, `State` (33 types) | One idea, plus `Step` as handler result (`guide.rs`) and as report order (`teardown.rs`) | State enums say `State`; a handler result says `Next`. |
| `OpenEvent` (2) | Everything heard on a channel, not an opening | `Heard` (agent), `Received` (host). |
| `adapter` (162 prod) | Design: only the shell's `Agent` adapter (shell.md) | `ProcessAdapter` → `HostIo`; `limits.llm.adapter` → `component`. |
| `Source` (4) | Event field, credential, service source, settings layer | Settings: `Origin`. |
| `Prepared` (9) | Unrelated | Name each. |
| `Failure` (4) | LLM, run end, local, service | Run's becomes `RunFailure`, as the host says. |
| `repository`, `checkout` | Design: a workspace has directories; a repository is a git one; "checkout" is not a domain word (135 prod) | `Repo` → `Directory`, `Checkout` → `Working`. |
| `push` | Design: delivery; `git push` stays local-host | Retire `Pushing`, `Stats.pushes`. |
| `capability` | A design word | Keep. |
| `rights` (0 in the design) | Terminals owed | Say "owed" or "terminal". |

### Q9. Size

**Base, in lines:** production 104,300 code and 10,300 comment; tests 159,800 code and 7,300 comment; 17,300 blank; 298,000 hand-written. Generated: 26,300.

The reports' estimates add up to about 29,000. **Method:** I group claims by what each removes, count each group once, and use measured sizes where I have them. I take 30 to 80% of claims I could not measure, lowest where a report marked itself unsure, and 40% of state conversions, which skein's review found mostly neutral in size.

| Group | Claimed | Taken | Kind |
|---|---:|---:|---|
| **Production code** | | | |
| Dead code and parameters | 500 | 500 | Mechanical |
| Same-crate near-copies | 2,400 | 1,450 | Mechanical |
| `smith-events` helpers | 1,300 | 900 | Mechanical |
| skein adoption and additions | 600 | 450 | Mechanical; skein grows about 200 |
| Host vocabulary | 900 | 700 | Redesign |
| `OpenEvent`, `Below`, `Lower`, `Launch` | 300 | 250 | Redesign |
| 5.4 conversions | 1,000 | 400 | Redesign |
| protocol-llm surface, `Done`, namespace | 530 | 400 | Redesign |
| `Service` split, `ProcessAdapter` | 230 | 150 | Redesign |
| **Subtotal** | | **-5,200 (5.0%)** | 3,300 mechanical |
| **Production comments** | 1,200 | **-800 (8%)** | Mechanical, read as you go |
| **Test code** | | | |
| Orphans | 570 | 400 | Mechanical |
| The fork | 4,000 | 3,500 | Redesign: one seam |
| `Limits` presets | spread across reports | 1,500 | Mechanical |
| Retyped setups and fixtures | 9,000 | 4,500 | Mechanical |
| Same behaviour tested twice | 1,400 | 800 | Mechanical |
| Domain stories in protocol-llm | 5,000 | 1,500; 2,000 more move | Redesign (unsure) |
| **Subtotal** | | **-12,300 (7.7%)** | 7,200 mechanical |
| **Test comments** | | **-1,800 (25%)** | Script: measured stamps, blanks, `Lower test of` |

**Total: about -20,000 lines of 298,000 (6.7%), 13,100 mechanical.** Treat it as 15,000 to 25,000.

**Not checked:** whether helpers fit every site; how much of the stories survives a move; test budgets after splits; net lines in skein. Renames and crate splits count as zero.

### Other cross-area findings, by payoff

1. **smith's debt is mostly adoption lag.** Eight skein mechanisms sit at or near zero use here (Q5). skein's review noted the tier problem; this is its downstream half. A ratchet stops it recurring.
2. **`#[allow]` and test lints share a cause.** Skein's D2 (allow wildcards in test crates) is not applied: 10 of 16 test manifests here still deny the lint, 32 test `expect`s pay for it, and benchmarks pay 581 lines (12). The 19 inner `#![allow(` lists in inline tests exist because `crates/clippy.toml` bans `Vec` and `String` in test modules too.
3. **Retyped idioms at a scale no area saw:** `Token::new(N)` on 1,872 test lines and `.as_slice().into()` on 2,024. Named tokens and a `bytes()` helper save no lines but read better.

## 5. Cross-area suspicions

jig and temper show the same patterns for `u64` length, the owner match and check-or-write (search only).

## 6. Rule costs

- **`manual_map`:** 31 production `expect`s, each honest, since closures are out. Allowing that one lint in step crates' `[lints]` removes about 90 lines of attributes and costs nothing.
- **Sibling rule:** `Cap` alone costs 21 arms × 9 mappings. Keep the rule; the run reading none of them is the price of 4.5.
