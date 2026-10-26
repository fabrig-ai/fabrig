# forge-kit: the forge kit's extraction

2026-10-19. Start commits: skein `d120820` ("The forge kit: five designs
from draft/forge.md"), temper `27995fb75` ("Point temper's forge
documents to skein's forge kit"); jig `b2d920e43` and smith `a277e922`
are read only, but for jig's boundary check (Rules: the boundary check's
jig repair). Every command of this plan ran once at these commits
(`findings/base-runs.md`, and each session's `findings/base-<session>.md`);
each session starts from its repository's `origin/main`.

temper's forge crates move to skein, made neutral, as skein's forge kit:
the vocabulary, Forgejo's dialect and its call plans (`skein-forge`), the
client (`skein-forge-client`), the component (`skein-forge-connection`),
reconcile's machine (`skein-forge-setup`), the fake forge, its forge peer,
the throwaway Forgejo, the conformance and the kit's worlds. Upstream
first, one crate at a time, in temper's design's order (`draft/forge-kit.md`,
section 5): each crate is a skein landing, then a temper step that repins
temper onto it, moves temper's users onto it, hands it temper's values
and deletes temper's crate (development.md, 5.7). The move closes the two
gaps the designer named: the head check counts the user agent (skein
03.2), and the start's two reads are the dialect's plans (skein 01.4).

## Inputs

- **skein's design,** at `d9f3e39` (`d120820` with the designer's
  answers, decisions.md): `forge.md` (2 to 9, 11),
  `forge-client.md` (2 to 10), `forge-connection.md` (2 to 10),
  `forge-setup.md` (2 to 4, 6 to 8), `fake-forge.md` (2, 4 to 8);
  `testing-strategy.md`, 8.
- **temper's design,** at `b4cfb999`: `protocol/forge.md`,
  `protocol/setup.md`, `domain/forge.md`, `domain/testing.md` (what stays
  temper's: the owner's values, the connector, the stories);
  `draft/forge-kit.md` (the connector's neutral top, later).
- **`decisions.md`:** the plan's decisions, what the user may veto, and
  the design's answers.
- **`findings/`:** the code audit at the base, one file per area
  (`survey-01-forge.md` to `survey-07-pins.md`), the design trace per
  document (`trace-*.md`), `base-runs.md` (each command of this plan, run
  once at the base), `drafters.md`.
- **`code-organization.md`:** what moves where, names, owners, fixtures,
  the "Don't" list, sizes and the count.
- **`later.md`:** what the design names that this plan does not build.

## The sessions

A session waits for what "Needs at start" names; an increment that needs
more says so in its row and its file. Dependencies are by capability:
"temper 01.2" means that increment has landed on temper's `origin/main`.

### skein

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `skein/01-forge.md` | skein 01: `skein-forge`: the vocabulary, Forgejo's dialect, its call plans on plain bounds with keys' markers on the owner's word, the start's two reads, credentials' kinds and `host` | nothing | 01.3: temper 01.1, 01.2; 01.4: temper 01.3 |
| `skein/01.s1-test-shapes.md` | skein 01.s1: sampled unit test shapes and fixture costs (ruling 118) | skein 01.4 | |
| `skein/02-client.md` | skein 02: `skein-forge-client`: the client moved, effects' keys on the owner's namespace; the measure, the dialect's and the client's (decision 24); the design's differences; the tests section 10 asks for | temper 01.1 | 02.2: skein 01.3 |
| `skein/02.s1-contract-tests.md` | skein 02.s1: contract-test shapes, paired row bounds and source checks | skein 02.4 | |
| `skein/02.s2-recovery-tests.md` | skein02.s2: recovery test shapes, paired restore bounds and scoped AST checks | skein02.s1 | |
| `skein/03-connection.md` | skein 03: `skein-forge-connection`: the component moved, on plain numbers, the owner's values and the transport's name; the head check counts the user agent; the checks at their bounds; page sizes on the measure | temper 01.4 | 03.3: skein 02.2 |
| `skein/03.s1-connection-shapes.md` | skein03.s1: independent backoff and terminal checks, named page/eviction capacities and scoped AST recurrence controls | skein03.3; normal ready check | |
| `skein/04-setup.md` | skein 04: `skein-forge-setup`: reconcile's machine moved, on the owner's given values and the kit's item; the design's differences; section 7's step tests | temper 01.2 | 04.1: skein 01.3 |
| `skein/05-fakes.md` | skein 05: the fake forge's domain and protocol layer moved, with their tests; the forge peer in `skein-fake-peers` | temper 01.2 | 05.4: temper 05.1 |
| `skein/06-conformance.md` | skein 06: the throwaway Forgejo on skein's HTTP client; the conformance on the kit, with its transcripts, and its script; the dialect's document tests on the transcripts | temper 01.2 | 06.2: temper 03.1, temper 06.4 |
| `skein/06.s1-oneshot.md` | skein06.s1: bounded throwaway driver cause and evidenced scheduling repair | skein06.1 | |
| `skein/06.s2-document-test-shape.md` | skein06.s2: name the document boundary-table row type | landed skein06.4; normal ready check | closing temper-repin waits |
| `skein/07-worlds.md` | skein 07: the client's and setup's worlds, `tests/forge/client` and `tests/forge/setup`, moved, with client recovery; extra setup stories deferred | temper 02.1, temper 05.1 | 07.1: skein 02.2; 07.3: temper 04.1, skein 08.1 |
| `skein/08-machine-world.md` | skein 08: the component's machine and listener world, `tests/forge/machine`, moved, with plaintext and TLS faults, deadlines, and closing; independent allocation measurement deferred (forge-connection.md 10; decision 23) | temper 03.1, temper 05.1, skein 06.2 |08.4: skein08.s1 |
| `skein/08.s1-deadline-test-shape.md` | skein08.s1: separate existing deadline outcomes and name time allowances | landed skein08.2; normal ready check |08.4 and closing temper-repin wait |
| `skein/08.s2-close-test-shape.md` | skein08.s2: explicit Close expected rows and named fixture allowances | landed skein08.3 and08.s1; normal ready check | closing temper-repin waits; later08.4 shared-file integration |
| `skein/09-protocol-world.md` | skein 09: the protocol world's component and fake tests, `tests/forge/protocol`, moved (five tests); added owner/stories/invariants deferred (forge-connection.md 10; decisions 22, 23) | skein 07.1, skein 08.1, temper 03.1, temper 05.1, skein 06.2 | |
| `skein/10-plans.md` | skein 10: `skein-forge`'s plans' pure tests on canned exchanges, and the plans' differences from the design (answers 7 to 12 and 35), each its own increment (decision 28) | skein 01.4 | 10.5: skein 08.1 |
| `skein/10.s1-named-page-fixtures.md` | skein10.s1: named page fixtures and scoped AST recurrence checks | skein10.5 | |
| `skein/10.s2-plan-contracts.md` | skein10.s2: independent plan contracts, exhaustive repository matches and scoped AST checks | skein10.4; normal ready check | |

### jig

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `jig/01.s1-client-names.md` | jig01.s1: boundary naming seeds and whole project prefixes under the designer's answer | jig design9e9c350; normal ready check | |
| `jig/01.s2-smith-adapters.md` | jig01.s2: settled channel-v3 and no-live smith compatibility adapters | live-text/smith-02.1, jig01.s1; normal ready check | |
| `jig/01.s3-http-room.md` | jig01.s3: HTTP mount room and application busy refusal | jig01.s2, jig design8922bfa; normal ready check | temper02.1 commit7 and closing temper-repin wait |
| `jig/01.s4-busy-test-shape.md` | jig01.s4: extract busy-test socket drainage while retaining visible assertions | landed jig01.s3; normal ready check | existing closing temper-repin waits; other starts unheld |

### temper

| File | Session | Needs at start | Increments that need more |
|---|---|---|---|
| `temper/01-forge.md` | temper 01: temper on `skein-forge`: the vocabulary's and the dialect's users moved, temper-forge-forgejo deleted, the component on skein's plans and start reads, temper's word handed | skein 01.1; the tools plan's first repin (Rules) | 01.2: skein 01.2; 01.3: skein 01.3; 01.4: skein 01.4 |
| `temper/02-client.md` | temper 02: temper on `skein-forge-client` and the measure; temper's client deleted; effects' purposes in temper's connector | skein 02.2, temper 01.3 | 02.1's naming proof and landing: jig01.s1; compatible repin: jig01.s2; commit 7: jig01.s3 (Rules) |
| `temper/03-connection.md` | temper 03: temper on `skein-forge-connection`; the component's crate deleted; temper-service hands the word, the user agent, the transport's name and the limits; the new refusals mapped | skein 03.1, temper 02.1 | 03.2: skein 03.3 |
| `temper/04-setup.md` | temper 04: temper on `skein-forge-setup`; reconcile's modules deleted; temper's setup hands its values | skein 04.1 | |
| `temper/05-fakes.md` | temper 05: temper on skein's fakes, temper's deleted; the one-process world on skein's forge peer | skein 05.2 | 05.2: skein 05.5 |
| `temper/06-throwaway.md` | temper 06: temper's process tier, tools and conformance on skein's throwaway; temper's deleted; the conformance on skein-json (decision 27) | skein 06.1 | 06.2: skein06.s1 |
| `temper/07-worlds.md` | temper 07: the moved worlds, the dialect's tests and the conformance with its transcripts deleted from temper; the protocol world keeps temper's own tests (decision 22) | skein 06.3; temper 06.1 | the client's world: skein 07.1; setup's: skein 07.3; the component's: skein 08.1 and 09.1; 07.1: temper 05.2 |

17 original sessions: skein 10, temper 7; plus supplements skein01.s1,02.s1,02.s2,06.s1,06.s2,03.s1,08.s1,08.s2,10.s1,10.s2 and jig01.s1,01.s2,01.s3,01.s4. The brief's fifth step, the fakes and
the conformance, is skein 05 to 09 and temper 05 to 07, by size. The
increments per session are in each file.

**A copy follows temper's code onto skein's.** A skein step copies
temper's code only once that code names skein's crates for everything
the copy uses: skein 02 copies temper's client after temper 01.1 has put
it on skein's vocabulary, skein 03 the component after temper 01.4, its
worlds (skein 08 and 09) after temper 03.1, 05.1 and skein 06.2, skein 04 reconcile and skein 05 the fakes
after temper 01.2, the forge peer after temper 05.1, the conformance
after temper 06.4 has put it on skein-json, and each world after temper has put it on skein's
crates. The copy is then a move of files and a rename of paths
(code-organization.md, 6), and temper deletes its own once skein's has
landed.

**How many at once:** as many as the dependencies and the machine allow.
At the start one needs nothing: skein 01; temper 01 starts when skein
01.1 lands. Then, as they become ready, first what others wait on: skein
03 (temper 03, skein 06.2, skein 08), skein 02 (temper 02, skein 07, 09),
skein 05 (temper 05, skein 07 to 09), skein 04, skein 06, skein 10; the temper
sessions as their skein steps land; skein 07 to 09, then temper 07
last.

**The critical path:** skein 01.1 → 01.2 → temper 01.2 → skein 01.3 →
temper 01.3 → skein 01.4 → temper 01.4 → skein 03.1 → temper 03.1 →
skein 06.2 → skein 08.1 → skein 09.1 → temper 07.1 → 07.2: fourteen
steps to temper's last session. Retained machine refinements08.2–08.4 may land
after it; the closing `temper-repin` follows all retained landings and never waits
for the five dropped increments in the scope rule below.
Beside it, skein 03.2 → 03.3 after skein 03.1, then temper 03.2;
temper 06.1 → … → 06.4 after skein 06.1, before skein 06.2 (decision
27); and skein 10.1 → … →
10.4 after skein 01.4, with 10.5 after 10.4 and skein 08.1. temper 07.1
also waits for temper 05.2, behind skein 05.5 (decision 26): a shorter
chain.

## Rules of this plan

- **Scope cut, ruling189 (2026-10-20):** drop skein07.4,08.5,09.2,09.3,09.4 from this run, to the GitHub draft, section17. PartC reshapes teams/hooks, runtime credentials, App key, installation tokens and puller; the review owns that draft. Keep every other step, especially05.2first when slots are scarce,07.3,08.1–08.4,09.1,10.5,10.s1 and Temper07.1/07.2. Preserve every test temper deletes by its retained move; the protocol package ends with the five09.1tests, no fuzzy run; the retained machine steps end at41, plus08.s1’s two separated identities (43 final, no additional scenario). The independent component measurement runs once after GitHub holders exist. The closing temper-repin waits only for retained steps. Session Done-when checks and later.md apply this cut; no runtime limit or kept proof changes.

- **Workspace license inheritance:** every new workspace member in skein or smith carries
  `license.workspace = true` in `[package]`, beside its other workspace-inherited fields.
  This includes a crate moved into either workspace: set the destination manifest explicitly;
  the gate does not check it yet. Skein's workspace and existing members were licensed MIT in
  `1222e7b73ff33d446e68cd364b8370a6df5207a9`; smith's license landing belongs to accounts.
  Each crate-adding step states this requirement. In this plan, the remaining additions are
  skein06.2, 07.1, 07.3, 08.1 and 09.1. Jig and temper follow only when accounts licenses their
  workspaces. Keep inherited metadata when refreshing or copying a destination manifest;
  list any further reached member in the commit body. This rule adds no gate or repin.

- **Connection component test shape:** skein03.s1 owns hourly07’s sampled corrections after03.3: separate backoff progression from terminal-state immutability and derive named capacities from the two-row page and sequential single-resource eviction premises. Preserve the assertion mapping in findings/connection-shapes.md, inherited tests, wire values, inputs, outcomes, limits and timing budgets. Add only the scoped connection-shapes AST recurrence mode to the existing checker/gate under development.md14.4; preserve recovery, plan-shapes and any already-landed listing mode/controls. Normal readiness precedes implementation. Serialize shared checker/gate edits without holding existing starts; closing temper-repin also waits for03.s1. Final connection counts gain relative+1; historical increment proofs keep their original counts.

- **The Cargo org cutover:** the GitHub org is now `fabrig-dev`; remotes already use it and old URLs redirect. The review moves smith’s, jig’s and temper’s `Cargo.toml` and `Cargo.lock` source URLs together, in one change after temper’s tools part 1 ends, with pins unchanged. Until that landing every Cargo source line stays `forgetic`; afterward every source line uses `fabrig-dev`. A step adding a dependency copies the URL its `Cargo.toml` already uses; this rule governs the plans’ older literal URL examples. If a rebase conflicts on a source line in either Cargo file, take main’s file and rerun the step’s authorized repin. A gate error naming two skein sources identifies a remaining `forgetic` line: rename it. Preserve the existing checks and one source per upstream; the GitHub redirect does not unify Cargo source identities.

- **Plan contracts and repository matches:** skein10.s2 owns hourly05’s three sampled corrections after10.4: separate the literal subscription contract from delivery mapping, keep word-validity assertions separate from marker round trips, and match repository Documents exhaustively under programming-model.md10.2. Preserve every mapped assertion, outcome and ordering. Add only the scoped plan-shapes AST gate mode under development.md14.4; preserve landed recovery controls, and listing mode/controls only when present from10.s1. 10.s2 must neither add nor require10.s1 files; later10.s1 preserves plan-shapes mode/controls. Normal readiness precedes implementation; serialize shared checker/gate edits with10.s1 without holding existing starts. Closing temper-repin waits for10.s2. Final calls/edges counts include its relative +2; historical increment proofs retain their original counts.

- **Document boundary table shape:** skein06.s2 names the private `BoundaryRow` type in `crates/skein-forge/tests/forgejo_bounds.rs`, removing only its `type_complexity` expectation and preserving the eight zero-field and two depth rows. Existing test counts, assertions, corpus and decoder behavior stay. Normal readiness precedes implementation; the closing temper-repin also waits for06.s2. Refresh further reached copies within this scope and list users.
  The existing scoped `disallowed_methods` expectation in `forgejo_deliveries.rs::every_subscribed_event_has_a_recorded_delivery_the_decoder_knows` is retained as a stated fixture exception to the hourly checklist: it converts the fixed subscribed-event byte names into fixture-directory names in a protocol document test. The lint's stated reason is "the protocol layer validates text"; no production domain parses here. Keep its explicit reason. Changing `core::str::from_utf8` to inherent `str::from_utf8` merely avoids the lint's path match and is not an authorized repair. No broader lint or parsing exception is granted.

- **Deadline test shapes:** skein08.s1 separates the existing seven08.2 deadline cases into start/retry, unseen-call Unavailable/cost0 and seen-call Timeout/cost1 tests, with shared setup/observation and no row-selected assertion behavior. Name the unchanged six-second observation allowance (fixture idle5s plus existing SECOND startup allowance, not a proved startup-duration bound) and eleven-second retry point (deadline10s plus initial backoff1s). Preserve all inputs, D-1/D, retry/teardown assertions and runtime workload. Normal readiness precedes implementation;08.4 and closing temper-repin wait for08.s1. Its relative+2 focused identities give final43/8; historical proofs retain their counts. Preserve/rebase W73's independent08.3append in the shared test file; refresh reached copies and list users. Both designer questions, five scope drops and all timing caps remain.

- **Close table shape:** skein08.s2 owns hourly19 points1–2 on landed08.3/08.s1: literal six-column expected data for35 machine and5 typed listener rows, the same single Close test and three211atomic assertions. Name the current32 refusal prefix as a fixture allowance preventing fallback to Made, and reuse CALL_ADMISSION for its three existing6s observation points; neither is a derived hard bound. No input, outcome, driver, scenario, identity or cap changes. Normal readiness precedes implementation. Closing temper-repin waits for08.s2. Preserve the dirty08.4 append; whichever shared-file landing follows rebases and runs its normal actual-tree checks, including exhaustive once that profile exists. Other independent08.4 work stays unheld. If the combined file exceeds800lines, move Close helpers/rows to tests/machine/close.rs while keeping the parent test identity. Refresh further reached copies and list users in the commit body.

- **Terminal cuts in world fixtures:** stop scripted dispatch after its configured terminal EOF/reset across every reached handshake, request and response cut path. Retain receiving/decrypting and independent write accounting; drain ciphertext and faults and retire peers through the existing io/Closed paths. A dead script wait is distinct from actionable room/grant work: establish quiescence before advancing the virtual clock. Keep Silent and held-response controls nonterminal. Preserve every byte row, assertion and cap, refresh further reached copies and name users in the commit body. Skein08.4's explicit follow-up budget governs its current draft and grants no retry elsewhere.

- **Graph freshness:** the supervisor refreshes only the primary checkout indexes. Use the exact primary project names and check each result in the actual worktree. Until a stale index's refresh is reported, use source searches for that repository; a missing graph result proves no absence. A refresh applies through its reported source commit; later source landings require a fresh comparison. Source search remains available throughout.
  The supervisor also resolves primary-behind preflight drift: fast-forward
  only a clean primary `main` to its already-landed `origin/main`, check
  alignment and the dry-run push again, and preserve the passed gate's
  exact tree/result. Dirty or divergent primaries return a report. This
  synchronization touches no author's worktree or the pinned kit.

- **Busy admission test shape:** jig01.s4 extracts only the reference busy test’s typed socket-drain plumbing into its existing server fixtures. Keep all eight assertions visible and ordered in the single test, with the same calls, queued ends, retirement, fixture values and budgets. No runtime, pin, accounting, gate or test-count change; normal readiness comes first. The existing closing temper-repin also waits for01.s4; Temper02.1 and Accounts retain independent ownership, and other starts stay unheld. Refresh further reached copies within this test-only scope and list users in the commit body.

- **World migration fixture copies:** Skein07 retains every assertion and identity suffix when changing namespace frames to `kit`, including simulation issue/post and fuzzy seeds. Its branch recovery watch is nonparticipating; the durable Writer claim and recovery outcomes stay. Setup's direct `fixtures::given` and every Item parameter, including `fixtures::seed_token`, follow landed types; preserve limits and the independent decoder. Refresh any further reached fixture or derived copy and list users in the owning commit body. A scratch correction without execution stays unproved; remaining setup validation is supervisor-owned and does not hold unrelated starts.

- **Recorded fixture expectations:** pure document tests compare a captured
  input with its independently transcribed expected document/rows. When a
  required recording changes those input fields, refresh the corresponding
  literal expectations, including masked times/identities and captured page
  membership. First compare old/new raw bytes independently of the decoder;
  record each field, conversion and expected value in the commit body.
  Preserve every case, assertion structure, exact row equality and captured
  order; preserve row-count, boundary, error and scenario-outcome guarantees.
  A captured-row refresh does not assert a new stable API selection/order.
  Never use actual decoder output as the expectation, edit the corpus outside
  recording, or change a production decoder to fit it. A changed interpretation,
  refusal, scenario outcome or lost case returns as a finding. Refresh further
  reached literal/derived copies and list users, including06.4's consumers.

- **Empty-body recording framing:** Temper06.2–06.4 keep the stable-fact
  digest and committed corpus unchanged. Exactly four request paths must differ
  by only `Content-Length: 0` before Connection: `setup-team-member-added`,
  `setup-team-member-missing-user`, `update-conflict`, `update-pull` (each `/request`).
  `merge-conflict/request` and `merge-stale-head/request` may each equal the corpus;
  their only alternative is the already-recorded masked `head_commit_id` variant:
  conflict's40hex digits ending `e` become `a`, stale's ending `d` become `e`,
  with all39leading zeros and every other byte unchanged. Thus4to6paths differ;
  no other path or byte is allowed. This is a raw byte comparison, never header
  stripping, body normalization or a new corpus. Refresh reached proof-script
  copies and list them in the owning commit. Completed recordings satisfying
  this exact rule need no repeat; all pending gate/increment proofs remain.

- **Readiness proof preparation:** preflight explicit shell scripts and any source generator before submitting a bounded attempt. Verify the clean candidate first; keep temporary accounting diagnostics in a separate scratch copy so a diagnostic setup defect cannot prevent its compile, clippy and existing-profile proof. Reuse an executed baseline only after source, test-identity and count verification. Setup failures still consume attempts; a new budget needs a supervisor ruling. Use isolated owned sources/targets, retain provenance and remove only completed owned targets; no global cache or dependency-code changes.
  Freeze and hash each absolute Bash wrapper after bash-n preflight and before submission. Never rewrite a queued or running script. Establish deadline/watchdog handling before launch; any correction uses a new immutable script and only an already authorized remaining attempt. A checker-caused wrapper failure consumes its attempt and leaves the underlying result incomplete, with original commands, diagnostics and locks retained.

- **Diagnostic process attempts:** before spending an authorized process probe, select a fresh run-owned instance path and the harness for its process tier; verify prerequisites and the exact request comparison before execution. Preserve earlier evidence instead of reusing a nonempty instance. Setup failures still consume attempts. Exhausting a diagnostic budget requires a report and a new ruling before another attempt; it does not expand unit-suite time limits or authorize retries.

- **Throwaway reset repair:** skein06.s1 owns the bounded driver diagnosis/repair before temper06.2 resumes recording proof. Preserve io's Failed contract and genuine incomplete-reply failure; no further full recording precedes deterministic cause proof. The closing temper repin waits for skein06.s1.

- **Wait discipline:** follow development.md 7.4. Prefer completion notifications; use supported work-completion waits of at least 300 seconds and identify tool-imposed exceptions instead of repeatedly polling on expiry. This does not delay handling messages or required operational checks.

- **Client recovery fixtures:** skein02.s2 owns hourly-03 recovery findings after02.s1, with the mapped assertions, independent boundary records and bounded Rust AST gate controls in its session/findings. Preserve every rejection cause and failed-instance persistence; the repeated identical generic fresh success is one independent behavior. The closing temper repin waits for02.s2. Final counts are relative; historical increment proofs retain their original meaning.

- **Client contract fixtures:** skein 02.s1 owns hourly-02's new contract-test corrections after 02.4, following testing-strategy.md 10's One behaviour per test, The setup the assertion reads, A setup written once, Named fixtures and sizes, and A bound's two sides. The repeated setup/size findings also require the scoped Rust gate checks in that supplement (development.md 14.4). The closing temper repin waits for skein 02.s1; skein 01.s1's preserved-test scope stays.

- **Named page fixtures:** in the plan-call tests, use the shared PAGE_SIZE and a named, derived one-row fixture for page-size rows and expected slices. Preserve every literal expected wire byte and assertion case. Skein10.s1 owns the scoped AST gate check after10.5 under development.md14.4, reusing the existing checker executable with a separate listing-sizes mode. Normal readiness precedes implementation; the closing temper-repin waits for10.s1. Skein10's existing start and increments remain unheld.

- **The hourly test supplement:** new tests follow testing-strategy.md 10.
  Only `skein/01.s1-test-shapes.md` may split its five named preserved tests,
  after the source moves; it retains every assertion case with an old/new
  mapping. Its fixture-limit audit governs those sampled unit defaults;
  shipped/profile values and boundary coverage stay. All other move-preservation
  rules remain. The closing temper repin also waits for skein 01.s1.



- **The tools plan runs beside this one,** on another machine, on the
  same temper `main` (temper's tools, area 1, first part: temper
  `1093544c3`). Its jig 01 and temper 01.1 repin jig and temper onto
  skein's and smith's `main`; temper 01.1 here starts after its temper
  01.1 has landed: temper's `Cargo.lock` names smith at or after
  `b415f737` (`findings/survey-07-pins.md`). Its hotspots, the files this
  plan copies, deletes or edits that the tools plan also changes, each
  with the steps that hold it:
  - temper's `Cargo.lock` and root `Cargo.toml`: every repin of both
    plans; its temper 02.1 adds a member, and each temper session here
    deletes some.
  - `tests/client/Cargo.toml`: its temper 01.1, before temper 01.1 and
    02.1 here, so a rebase only.
  - The forge client's importers that its temper 03.1 changes:
    `crates/temper-service/src/tests/{fixtures,family}.rs`,
    `crates/temper-protocol-store/src/tests/records.rs`,
    `tests/conformance/src/scenarios.rs`; temper 01.1 and 02.1 here
    change their forge-client imports.
  - `testing/temper-fake-forge-domain`, `testing/temper-fake-forge-protocol`:
    no step of the tools plan changes them. They are copied by skein 05.1
    and 05.2 and deleted by temper 05.1; from skein 05.1's start to temper
    05.1's landing a change to them waits, and after it lands in skein's
    `testing/skein-fake-forge-*` (the designer's ordering rule: a change
    to a fake forge crate lands where the crate lives on `main`).
  - `tests/one-process/src/forge.rs` (`ForgeHost`) and
    `testing/temper-fake-git/src/transport.rs` (its shared forge handle):
    the tools plan leaves the forge host, its delivery and the real-loop
    world to its next part, unplanned (its `later.md`; drafts in its
    `findings/next/`, whose 07 moves the host to
    `testing/temper-fake-forge-host` with delivery). skein 05.4 copies the
    base host into skein's forge peer, skein 05.5 adds delivery there,
    and temper 05.2 deletes the host (decision 26), so that the next
    part's real loop hosts skein's peer (domain/testing.md 4.2). From skein
    05.4's start to temper 05.2's landing a change to the host waits.
  - For the next part, when it is planned: its drafts also change
    `testing/temper-throwaway-forgejo` and
    `crates/temper/tests/support/{owned,forgejo}.rs` (its 10.1, 10.2;
    skein 06.1 copies them, temper 06.1 deletes them),
    `tests/client/src/{front,fixtures,translate}.rs` (its 06; temper 02.1
    here) and the webhooks' listener,
    `temper-engine-protocol-forge/src/{hooks,component}.rs` with
    `tests/engine/protocol-forge/{src,tests}/hooks.rs` (its 08.1; skein
    03.1 copies them, temper 03.1 deletes them).
  `temper ctl adopt`'s forge-facing flags (its temper 02.3, 11.1 and
  11.2: `temper-client-tools`, `temper-client-domain`'s `forge/`,
  `temper-client-view`) and its temper 03.1 and 03.2's
  `crates/temper-service/src/family.rs` and `src/family/refusals.rs`
  (its README's hotspots name them for the forge kit) touch none of this
  plan's files: temper 03.1 here changes temper-service's `forge.rs`
  and its neighbours, not the family. By the order for this
  run, it starts before the tools plan's temper 11 lands. The tools run
  is on `debian-16gb-hel1-1`, on the same temper `main`. The coordinator
  must register `plan await coordinator tools/temper-01.1` and must not
  start temper 01.1 here until that step is on `origin/main`. No other
  hotspot above adds a wait for tools temper 11; rebase conflicts are
  resolved before merging. The fake and host
  copy windows above still hold until their named landings.
- **Each temper increment that needs a new skein step repins first,** in
  a commit of its own: `cargo update -p skein-lib`, then `cargo check --workspace --all-targets`. It moves every skein crate,
  jig's and smith's included (one revision per source); skein's new
  crates and modules break neither (Repin). **`Cargo.lock` is never
  merged by hand:** a conflict takes `origin/main`'s, and the update runs
  again. A repin takes all of skein's `main`, and repairs in its own
  commit what a skein step since temper's last repin broke, as
  development.md 5.7 allows: names and paths that moved, and the tests
  an upstream change of behaviour broke, adapted as the skein commit
  adapted its own. A skein increment that changes an interface temper
  uses after its move names temper's lines it breaks in its Shape
  ("temper's side"), so that whichever temper increment repins next
  knows the repair.
- **The first repin's jig repair** (temper 01.1): if its jig source is
  still `719bd70d4`, the first skein repin also takes landed tools
  `jig-02.1`, `b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`, with
  `cargo update -p jig-client-domain --precise b2eebe6ab91af9eca121588f2d44072fcbdf3b9c`.
  This repairs the RequestTable admission-number interface. Keep one jig
  revision per source, retain the skein update, and run the existing
  `cargo check --workspace --all-targets` before the repin commit.
  If another landing has already moved jig forward compatibly, keep it;
  this rule authorizes no backward pin. The remaining tools steps add no
  wait to this repair. Refresh derived fixtures reached by the normal check
  and name further copies and repairs in the repin commit body.
- **The boundary check's jig repair** (temper 02.1): jig's `README.md`,
  section 15, as jig `9e9c350` amended it, says how `jig-boundary` finds
  client crates. A dependency's name counts only under a client prefix of
  jig's or of the checked project, the project by its whole name, less the
  records both ends read, or as one of skein's UI kit's crates, by name; a
  `-client` in any other namespace, `skein-forge-client`'s among them,
  counts for nothing. Until `tests/boundary/src/client_names.rs` does so
  on jig's `origin/main`, the crates 02.1 moves onto `skein-forge-client`
  read as client crates, and `tests/client`'s
  `client_crates_carry_the_client_prefix` fails. The boundary-check repair is jig01.s1 (`jig/01.s1-client-names.md`), with normal
  readiness before implementation: the diagnosis's candidate
  (`client_names.rs` and its eight fixtures) is its evidence, with no
  system or forge word in jig and `breaches`' signature kept. temper 02.1
  takes it in its repin, with `cargo update -p jig-boundary --precise
  <the repair's commit>`: one jig revision per source, a newer compatible
  jig kept, no backward pin. Its migration, deletion and clippy go on
  meanwhile; its naming proof and its landing wait. `tests/client/tests/names.rs`
  does not change: its empty `breaches` and its two exempted crates
  (`temper`, `temper-one-process-world`) stay, and no crate is exempted
  for using a capability's client.
- **The HTTP mount's room** (temper 02.1, commit 7): jig's
  `protocol/client.md`, 17.4, as jig `8922bfa` amended it, gives the
  mapping's engine end its calls in flight alone: it holds a call until
  it has handled the call's end, and the application asks for its room
  before it hands it a call, and answers one that finds none `503`,
  closing the connection, as it answers a call no route claims. jig's
  repair is a jig step the supervisor adds (jig01.s3, as it added
  jig01.s1), with normal readiness before implementation:
  `Mount::room`, the calls the engine end can still take, counting a
  call whose end it has heard and not yet handled; the mount's insertion
  assertion kept, as the application's contract; `jig-test-service`'s
  `clients.rs` refusing as temper's does, its refused calls held until
  their end in a `Map<Token, ()>` at the server's connections, with
  `clients::worst_case` and `limits::worst_case` re-derived for it; every
  other caller of the mount's `Event::Call` in jig asking first; a test
  at each seam (the room across a heard, unhandled end; the reference's
  call past the room answered `503`). No profile or connection number
  changes. temper 02.1's commit 7 takes it with `cargo update -p
  jig-protocol-engine-clients-http --precise <the repair's commit>`, one
  jig revision per source, and the closing temper repin waits for it.
  In temper's shared `unclaimed` path, both the existing 404 and the new
  503 send `Request::Discard` before `Respond`, including for an empty
  request body (skein's `http.md`, 5.3). Keep each entry until its terminal
  event is handled and `server.done` is called. The existing map is already
  priced at the server's connections; this adds no allocation or sum term.
  Refresh further reached copies within the owning step and list users in
  its commit body.
- **A compatible smith source during a temper repin:** if taking skein main
  breaks the pinned smith source, the same temper repin may take the earliest
  compatible smith revision landed on origin/main. Keep an already newer
  compatible source; keep one revision per git source, the full skein update,
  and the existing workspace/all-target check and gate. Name the old/new pins,
  upstream repair and reached copies in the commit body. This changes temper's
  lock only; decision13 still leaves smith's own skein pin to its next repin.
  If no compatible repair has landed, continue independent work and report
  the owning upstream step for the coordinator's trailer-based await. Never
  patch Cargo's dependency checkout.
  The fake-LLM Event::Call.call / Request::Piece,Hold repair landed in
  live-text/smith-02.1 at53b9d6db52ff6cbe999212160e49056cb8ded44e. Taking it
  also reaches the explicitly settled channel-v3/no-live adapters below;
  establish compatibility with the normal checks, rather than treating
  that landed trailer as proof of downstream compatibility.
- **Accounts compatibility has separate owners:** accounts/smith-01.3 owns
  smith's changed credential, failure and report interfaces; its landing is
  not proof that downstream repositories compile. Jig's OAuth `SavedToken`,
  inner `Limits`, `ClientLimits` (`nonce_bytes`, `parameter_bytes`,
  `resource_bytes`) literals and `Failure::Incomplete` translation belong to
  accounts' existing closing `jig-repin`, inventoried by skein05.2–05.4.
  Skein08.2's subsequent account-removal vocabulary has separate closing
  owners: `Event::Removed` in smith-local-service's `Accounts::emit`
  (`credentials.rs`) belongs to accounts/smith-repin; Jig's `translate::up`
  for `Event::Removed` and `Asked::{Add,Remove}`, and `translate::refused`
  for `Refusal::{Taken,Unusable}`, belong to accounts/jig-repin. Skein08.2's
  Downstream and decision16 prescribe those mappings. Keep the Smith
  closing-repin await separate from the earlier landed smith01.3 repair.
  Skein04.1's fake-provider `Config::without_credential` literals in smith's
  `tests/agent-process/src/fake.rs::configured_bounds` and
  `tests/protocol-llm/tests/component_peer.rs::run`, and the latter's
  `provider::{start,down,up,resume}` credential arguments, also belong to
  accounts/smith-repin. Its Downstream prescribes `without_credential: false`
  and one-entry credential slices; retain the exact sources, locks and
  diagnostics until that repair and downstream compilation establish proof.
  Skein04.2's `Observation::Credential` in the same fake's `queries` also
  belongs to accounts/smith-repin. Keep its full Downstream request-count,
  observation-capacity and derived-memory sweep with that owner.
  Accounts/jig-repin also owns skein08.3's `Ends::Refresh`/`Asked::Want`
  matches and skein07.1–07.2's OpenID limits and `Failure::Unverified`:
  retain the existing Grant mapping, decision16's unreachable arms, and
  decision15's smallest valid unused limits, including the third accounts
  fixture already listed by the owning step.
  Smith02.2–02.3's notice/fault adapters stay with Accounts' closing Jig
  and downstream repins: Jig's test-worker `domain.rs`, `hand_out.rs` and
  `translate.rs`; Temper's worker-domain `agents.rs`, `translate.rs`,
  `tests/entrance.rs` and engine-smith `finish.rs`. Accounts ruling130
  settles the worker notice: Unauthorized maps as Rejected, Limited as
  Exhausted, Refused raises no event. For model faults keep smith02.3's
  Limited/Declined vocabulary and decision16 defaults; include the newly
  reached non-Copy users in the existing owner's sweep. Retain exact
  source/lock/error evidence and take compatible landed downstream adapters
  before resuming the held Temper migration; do not duplicate that work.
  The channel-v4 copies in `tests/core/src/{protocol,agents_tools}.rs` belong
  to accounts/jig-repin.s1 under smith01.2; forge-kit/jig01.s2's earlier
  live-text adapters do not cover them. Report each reached typed failure
  with exact sources and locks to its existing owner and retain independent
  work plus this plan's normal all-target check and gate. Do not patch
  dependencies, backpin or duplicate another plan's adapter sweep. Refresh
  any further reached copy or derived value within the owning repin, and
  list every user in its commit body.
- **New Accounts-2 interfaces:** Jig's newer host/fleet vocabulary in
  `155773d20ae312294fcfa9be48fe19b645f57f8c` and
  `a5f3a733a12dd3908e2eccfb52f9987377064545`, and configuration refusals
  `Confidential`/`NoProvider` in `9cd5dc229e955a9036ecbd540b67d3658d620ba0`,
  have separate Accounts-2 downstream custody. Before an owning repin takes
  those interfaces, obtain the compatible downstream repair/disposition from
  that owner; preserve the exact host/fleet payloads and refusal meanings.
  The reached Temper users are worker-domain `agents.rs`/`translate.rs`,
  engine-domain `decision.rs`/`link.rs`, and forge-setup `problem.rs`.
  Earlier Accounts mappings to removed `Rejected`/`Exhausted` are not a
  translation for this newer range. Preserve compatible committed pins and
  independent starts; this requires no early pin or duplicate adapter sweep.
  Refresh further reached copies within the owning repair and list users in
  its commit body; only actual-tree compilation establishes compatibility.
  Retain hourly17's additional Temper diagnostics under that pending custody:
  engine-domain `route/forge_route.rs`'s `Delegate.llm` and `tests.rs`'s
  `Model.effort`/`efforts` and `Authority.models`; protocol-store
  `records/application.rs` and `records/application/refusal.rs`'s unavailable
  `people_v4`; client-tools `jig/render/document/limits.rs`'s `Lacking::Models`;
  client-tools, client-view and client-domain test fixtures' unavailable wire
  `v9`. These name reached sites, not chosen defaults or target versions.
  Obtain the actual downstream owner/disposition and repair Step
  before consuming the newer interfaces; preserve the retained exact locks
  and diagnostics in `scratch-hourly-17`. The introducing interface step is
  not itself a downstream repair trailer.
  Hourly18 also reaches twelve `Core::Accounts` record/key match sites in
  Temper engine-domain `archive.rs`, `decision.rs`, `restart.rs`,
  `route/escalation.rs`, `route/proposals.rs` and `store.rs`. Retain the
  exact site/message/source/lock map in `scratch-hourly-18` under the same
  pending owner/disposition; choose no persistence, key or match behavior
  from the compiler's missing-arm suggestion. Refresh further reached
  copies in the eventual owning repair and list users.
  Hourly19 additionally reaches six Temper sites: engine-domain
  `route/applications.rs` (`Event::Application.carried`), `route/held.rs`
  (`Held::CarriedUnused`), `store.rs` (`Ask::AccountSignIn/AccountSignOut`);
  client-domain `forge.rs` (`Outcome::Unsent/Lost`) and `tests/form.rs`
  (`Down::Send.value` and `link::Event::Send.value`). These extend only
  the same Accounts-2/smith-sources inventory. Retain exact source hashes,
  compiler messages and locks from scratch-hourly-19/diagnostic-custody.json.
  Await the actual downstream owner/disposition and repair Step;
  no defaults, forced pins, dependency patches, new diagnosis or repeated
  adapter audit. The eventual owner refreshes reached copies and proves
  its actual tree. This inventory is no newer-head compatibility proof.

- **Host grant forms:** Smith `a26eadcf50888eee56f5d4b453bbedd83981c5de`
  (smith-sources/smith-01) adds `Grant.form`. Its newly reached Jig users are
  `testing/jig-test-worker/src/translate.rs::grant` and `src/tests.rs`'s
  account-notice grant literal. Obtain smith-sources' downstream adapter
  custody/disposition before consuming that interface, coordinated with the
  existing Accounts-2 ownership. Preserve the intended credential/program/
  withdrawn form; do not guess a mapping, force a new pin or repeat the
  adapter sweep. Keep exact diagnostics/locks, refresh further reached copies
  within that owning repair and list users. Actual-tree compilation remains required.
  Hourly17 also reaches Temper's worker-domain `agents.rs`,
  `tests/fixtures.rs` and `tests/lifecycle.rs` grant literals. Retain these
  with the Jig sites through the same dependency; choose no form here.
- **Settled smith adapters at the next repin:** smith's live-text
  decisions2.2/answer4 and For the user9 settle v3 paths, `live: None`
  and ignored `OpenEvent::Live`; its later.md expressly assigns them to
  the closing repins "or their next repin of smith". This plan's
  jig01.s2 supplies jig's next repin because tools/jig-repin already
  landed before live-text. Normal readiness precedes its implementation.
  Temper02.1 owns the corresponding first compatible temper repin:
  `crates/temper-service/src/worker_route.rs::{decode,process,channel}`,
  `crates/temper-worker-domain/src/tests/fixtures.rs::start`,
  `tests/worker/domain/src/protocol.rs`'s Start construction, and
  `crates/temper-service/src/tests/worker.rs`'s derived `encode_start` proof.
  Set the three Start literals to no ask and ignore only Live in the
  two worker_route matches; use the landed codec for derived sizes.
  This is the recorded policy, not live-display implementation.
  Temper05.2 and06.2 retain their checkpoints and wait for jig01.s2 and
  temper02.1, then rebase onto the adapter landing and keep every own proof.
  Other runs' next repins reuse the landing instead of redoing its sweep;
  no tools/temper-repin wait is added. Keep one revision per upstream source,
  newer compatible pins, the full skein update, all assertions and limits.
  Refresh any further reached copy or derived value and name it in the
  commit body; a further structural/design difference returns a report.
- **Authority models in newer Jig repins:** the first owning Forge-kit repin taking Jig's models authority (31b0db486 or a later compatible revision) also adapts `tests/engine/domain/src/walking.rs::authority_value` with `models: authority::Models::ALL`. This fixture builds the unrestricted rules ceiling and policy/role values; Jig domain/authority.md3 defaults to every configured model unless rules/policy/role narrow it. Preserve the downstream procedure restriction through `Models::run_by`, existing assertions, grants, budgets and kinds. Refresh further reached Authority literals/derived accounting, list users in the repin commit, and run its existing actual-tree checks/profiles/gate. This is no demand for an early Jig pin or a claim that current-main gates prove newest-source compatibility.

- **Mechanical repairs during repins** belong to the existing repin commit.
  Follow landed upstream renames, paths, signatures and generated-version
  moves through their whole sweep, including source, tests, schema comments,
  copied values and derived fixtures, for the dependency update. Keep fields,
  behaviour, test identities and assertions; list further users and repairs
  in the commit body. Use the increment's existing all-target check and gate.
  A straightforward successor-name repair needs no further scope block;
  report a structural or behavioural difference beyond the landed change.
  For temper 01.1's authorized jig pin, replace `people_v3` with `people_v4`
  in `crates/temper-protocol-store/src/records/application.rs`,
  `src/records/application/refusal.rs`, `src/tests/records.rs` and the comment
  in `schema/store-v2.schema` (all under that crate). This brings forward
  only the tools plan's temper 03.1 store-family name repair; its remaining
  work follows the existing hotspot/rebase rule.
- **Derived owned-example memory:** setup.md, 8.1 says the example's
  `memory_bytes` hold its worst cases, with the variable strings at their
  documented bounds; configuration.md, 7 requires the complete service sum
  to fit. When an authorized change of retained state changes that sum,
  refresh `docs/examples/owned.json`'s derived allowance from the complete
  `system::startup_worst_case` calculation at those unchanged inputs. First
  identify and measure the changed accounting term; keep operational limits,
  settings, test names and assertions. Record the old/new complete sums,
  cause and further reached copies in the commit body, and run the unchanged
  owned-example startup regression and the increment's remaining proofs.
  This does not authorize changing an admission cap or hiding an unexplained
  size/profile failure. For temper02.1 the checked client migration adds
  eight inline bytes and only `engine.memory_bytes` changes, from
  78,675,187,547,016 to 78,675,187,547,024 on the checked tree; rederive the
  complete sum if a rebase changes its inputs. Refresh further reached copies
  and list every user; later increments keep their own accounting scope.
- **A move copies files and renames paths** (code-organization.md, 6):
  besides that, it changes only what its new home forces, each named in
  the increment's Shape: temper's names out of skein's code (the owner
  hands them), and a type of a crate downstream of the new home made
  plain values. A behaviour change, and every difference the audit
  found, is a later increment's, with its own test.
- **A moved test keeps its name, its budget and its time.** A moved
  test over its ceiling (the protocol world's four `fuzzy_machine`
  tests, 7.5 to 7.7 s each at the base, `findings/base-runs.md`, moved
  by skein 08.1) moves as it is, its time and its cost's cause in the
  increment's report, and is named in decisions.md (11) for the user; none is split, trimmed or hidden by a filter.
- **Sizes.** A move is sized "about N lines moved, M new": the moved
  lines are those code-organization.md 6's command finds unchanged but
  for paths, and the increment's tolerance applies to the new lines
  alone.
- **A completed supervisor check releases its build outputs.** Once its worker
  has ended and no queued/running command uses its paths, remove its scratch
  build targets, including a custom CARGO_TARGET_DIR, while retaining source,
  patches, reports and logs. Count available filesystem bytes after cleanup:
  Btrfs targets may share cache extents, so du totals do not measure reclaimed
  space. When those targets are insufficient, preview tracked targets unused
  for at least an hour and remove only inactive build outputs, after checking
  the queue. Keep development.md 11.4's floors, concurrency limit and GC interval.
- **The process tier runs where a Proof line or "Done when" says,**
  once, never in the gate, with the pinned binaries named:
  `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
  TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
  scripts/proof-one-process.sh` in temper, with `WS` the
  workspace: `~/src/rust`, or `~/src` on a machine that keeps its
  checkouts there (AGENTS.md). skein's throwaway and conformance take
  the same paths as flags (skein 06). The tier is load-sensitive: two
  base runs failed beside other builds, three passed alone
  (`findings/base-runs.md`); a failure is rerun once alone before it is
  read as the step's.
- **The suites' aims for the whole run** (decisions.md): skein's default
  suite takes at most 1.5 s more than at the base, and its fuzzy suite
  at most 6 s more; temper's suites take no more than at the base, by
  package (the tools run lands beside it): the deleted crates' tests
  leave, the moved worlds' with them. No session has a share.

## Live services

None. The throwaway Forgejo and the conformance run the pinned local
binaries on loopback. At the base, temper's tests read `TEMPER_FORGEJO`
and `TEMPER_FORGEJO_RUNNER`, and they keep reading them. From skein
06.2 the conformance takes `--forgejo`, `--scratch`, `--out` and an
optional `--runner`, and reads no variable. The kit's ignored tests
read `SKEIN_FORGEJO` and `SKEIN_FORGEJO_RUNNER` in their test files,
and fail naming the one that is unset (decisions.md, Design questions
31). The conformance is a tool outside the suites, run by its script.

## Repin

temper's, in each temper increment that needs a new skein step (Rules),
and the closing `temper-repin` once every skein step, including03.s1,08.s2 and10.s2, and jig01.s1/01.s2/01.s3/01.s4 have
landed, dropped or split (development.md, 5.7): temper 07.1's repin may land
before skein's last tests do. skein's changes are new crates, a module
of `skein-fake-peers` and new test crates, so nothing of smith, jig or
hearth breaks; they take skein at their next repins (decisions.md, 13):
`later.md`, and each repository's `STATUS.md`, under "Not planned", in
the designer's next landing.

## Pulse

```
words: actual|genuine|legacy|compat|workaround
types: ^(Legacy|Old|Compat)[A-Z]|V2$
file-lines: 800
budgets: 15 60
```

## Done when

Each command runs once, after `done.md`, in a worktree of temper (or
skein, for skein's) at `origin/main`, the plan's worktrees removed.

1. Every step is landed or dropped: `plan status` lists none waiting,
   ready, running or blocked.
2. The plan's commits: `git log origin/main --grep='^Plan: forge-kit$'
   --format='%(trailers:key=Step,valueonly,separator=)' | sort -u` in
   skein lists the retained, non-dropped skein steps, and the same in
   temper the temper steps; a supplementary step (`.s<k>`) adds a line.
3. Each session file's "Done when" holds, in the sessions table's order.
4. temper holds none of the moved crates: `ls crates testing tools`
   names none of `temper-forge-forgejo`,
   `temper-engine-domain-forge-client`, `temper-engine-protocol-forge`,
   `temper-fake-forge-domain`, `temper-fake-forge-protocol`,
   `temper-throwaway-forgejo`, `forgejo-conformance`; `ls tests` names
   none of the moved worlds (code-organization.md, 1); and `rg -l
   'skein_forge' Cargo.toml crates/*/Cargo.toml` lists temper's users.
5. skein's kit names no system: `rg -n -i 'temper' crates/skein-forge*
   testing/skein-fake-forge-* testing/skein-throwaway-forgejo
   tests/forge testing/skein-fake-peers/src/forge*` prints nothing.
6. The process tier: in a worktree of temper at `origin/main`, with the
   binaries named as Rules say,
   `TEMPER_FORGEJO=$WS/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64
   TEMPER_FORGEJO_RUNNER=$WS/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64
   scripts/proof-one-process.sh` exits 0.
7. The conformance: skein 06.4's stable-fact check, its "Done when"'s
   `bash` line, prints the five timing outcomes, then
   `facts=218 stable=213 holds=207 fails=6 timing=5 unchecked=0` and the
   base's digest, `d1f7ba2b…18347d` (`findings/base-skein-06.md`).
8. The test-time aims (Rules), measured once: `sh -c
   'export NEXTEST_DOUBLE_SPAWN=0; for i in 1 2 3; do cargo nextest run
   --workspace 2>&1 | grep Summary; cargo nextest run --workspace
   --profile fuzzy 2>&1 | grep Summary; done'` in skein, against the
   same at skein's parent of `skein-01.1`; in temper, the per-package
   times of the moved crates' users against the base. An aim missed is
   recorded with the three slowest tests by name in decisions.md, for
   the user; the run is done either way.
9. No budget was raised: `git log origin/main --grep='^Plan: forge-kit$'
   -p -- .config/nextest.toml` prints nothing, in skein and in temper.
