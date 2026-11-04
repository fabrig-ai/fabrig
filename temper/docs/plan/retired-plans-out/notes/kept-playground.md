# kept: playground

Read at `9c608917eb`, 2026-11-04. 20 directories, 7 findings.

The Read lines enumerate the K2 inputs, not the directories' other files. All
`plan.md` files were read whole, both questions with their answers, and every
Answers section through its file's end. Contracts' code sketches were read for
the rules among them; no unlisted inputs, old notes or patches were read.

Accounting is by topic: repeated statements within a directory share a decision,
while the same topic in another directory is decided again. Each Held or
Superseded topic ends with its evidence in parentheses; semicolons inside a
citation separate documents, not topics. Each numbered finding is one Kept topic.
The Not design counts enumerate the operational topics in their parentheses.
An operational topic includes its subordinate instructions, task rows and code
shapes; behavioral clauses are accounted separately under Held or Kept. Thus a
contract's signatures can be Not design while its encoding rule is Kept.

## skein/docs/plan/playground-skein

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 3 files (`bundle-worker.md`, `playground-visitor.md`, `status.md`); notes/contracts.md; notes/common.md.
- **Kept:** playground-1, playground-2, playground-3.
- **Held:** thousandths, initial 1× and slice post order (`skein/docs/design/playground.md`, 5); log version, text, digests, cuts, build mismatch and replay loop (`skein/docs/design/playground.md`, 6, 6.1, 6.2); visitor's reports, headers, cookies and bounds (`skein/docs/design/playground.md`, 4); worker bundle, shared snippets, clashes and digest-relative addressing, including the bundle-worker answer (`skein/docs/design/playground.md`, 7; `skein/docs/design/ui-web.md`, 4.3); platform's abandon/read-on duties (`skein/docs/design/ui-web.md`, 18); idle clock arithmetic, monotonic clamp, deadline conversion, visibility and navigation (`skein/docs/design/ui-web.md`, 18); slices, settled-world stop and native bridge (`skein/docs/design/playground.md`, 3, 5); worker ready/start, transferred posts and fail-stop (`skein/docs/design/playground.md`, 5, 7); application composition versus generic kit (`skein/docs/design/playground.md`, 2); portable/pure halves and native-only fences (`skein/docs/design/testing.md`, 10); ordinary Rust and memory ownership (`skein/docs/foundation/programming-model.md`, 10.2, 6); visitor memory scenario's full holders and measured peak plus a quarter (`skein/docs/foundation/testing-strategy.md`, 6, Memory).
- **Superseded:** none.
- **Not design:** 8 items (task map and waves; file ownership and skeleton order; overlaps; critical-path estimates; gate commands and package insertion; contract modules/types/signatures/derives; common edit/check/report mechanics; Answers' test rename and report-location/departure-review procedure). The last status answer explicitly found no additional behavioral departure; it is not an open choice.

### playground-1. The bridge's byte encoding

- **Kind:** contract
- **Source:** `skein/docs/plan/playground-skein/notes/contracts.md`, 1, The encoding; 2026-11-03 (the enclosing plan's date); decided by not stated.
- **Checked:** `skein/docs/design/playground.md`, 5: unversioned byte messages, transferred buffers and empty ready message, but silent on integer order, lengths, malformed input and allocation; `skein/docs/design/ui-web.md`, 18: tests the encoding without stating these rules. Searches for little-endian, unknown tag and trailing bytes found other codecs' rules, not this bridge's. `skein/crates/skein-ui-web/src/bridge.rs` carries the encoding and validates lengths before reserving header storage.
- **Target:** `skein/docs/design/playground.md`, 5; a paragraph in the section.
- **Text:** Decided 2026-11-03. The bridge's encoding starts each variant with a tag byte, so no encoded message is the empty ready message. Integers are fixed-width little-endian; each byte string and each list starts with its u32 length. Decoding refuses short input, trailing bytes, an unknown tag or a length past the input, and allocates no more than the input's length. Encoding then decoding a message gives that message back. These rules belong to the build's unversioned bridge, separately from the session log's versioned text.

### playground-2. Inputs awaiting their stamps

- **Kind:** contract
- **Source:** `skein/docs/plan/playground-skein/notes/contracts.md`, 4, The session log; 2026-11-03 (the enclosing plan's date); decided by not stated.
- **Checked:** `skein/docs/design/playground.md`, 6, 6.1: keeps inputs and stamps, bounds the text including its ending, and ends snapshots at the last clock; silent on an unmatched stamp and the snapshot before the first clock. `skein/docs/design/ui-web.md`, 18: silent. `skein/crates/skein-ui-web/src/log.rs`, `stamped` and `Log::new`, implements the unmatched-stamp assertion and initial end.
- **Target:** `skein/docs/design/playground.md`, 6; a paragraph in the section.
- **Text:** Decided 2026-11-03. Each stamp keeps the oldest input the page sent that has no stamp yet. A stamp with no input waiting breaks the contract and panics. Before the page has heard any clock, a snapshot ends at the start, step 0 and instant 0, with the digest of nothing. Once a clock is heard, its step and instant, and what was said through that clock, give the snapshot's end.

### playground-3. Where the small store's bound is enforced

- **Kind:** contract
- **Source:** `skein/docs/plan/playground-skein/notes/contracts.md`, 5, The store; 2026-11-03 (the enclosing plan's date); decided by not stated.
- **Checked:** `skein/docs/design/ui-web.md`, 10, 18: writes succeed within the store's bound and are refused past it, without assigning that bound to `store::Store` rather than its platform. The contract assigns the bound to `store::Store` and says platform writes always succeed. `skein/crates/skein-ui-web/src/platform/worker.rs`, `Page::write`, replaces or appends a whole record and returns true without checking a bound; `skein/crates/skein-ui-web/src/store.rs`, `Store::write`, checks its bound before calling the platform. `skein/docs/design/ui-web.status.md` records no gap. The missing ownership distinction is kept for reconciliation; no new bound or externally visible refusal is proposed.
- **Target:** `skein/docs/design/ui-web.md`, 18; a paragraph in the section.
- **Text:** Decided 2026-11-03. The worker's platform keeps the small store's records whole in the page's memory. The small store's own bound is `store::Store`'s, enforced before a write reaches the platform; a write handed to the in-memory platform succeeds.

## temper/docs/plan/playground-temper

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 5 files (`bounds-measured.md`, `one-process-composed.md`, `one-process-documents.md`, `one-process-story-a.md`, `playground-world.md`); notes/contracts.md; notes/common.md.
- **Kept:** playground-4, playground-5.
- **Held:** product-only dependencies, hosted git and one composition shared with the one-process harness (`temper/docs/design/playground.md`, 3); harness processes, captures, referee and faults stay outside the composition (`temper/docs/design/playground.md`, 3, 4); no box, disk handover, printed links, listener and one derived seed (`temper/docs/design/playground.md`, 4); stories as compiled data and A's three requests, script source and browser wording (`temper/docs/design/playground.md`, 5.1, 5.2, 5.4); pace document location, fixed latencies, 60000 ms ceiling and absent-zero default (`temper/docs/design/protocol/setup.md`, 8.4, Both documents may also hold pace); native/browser walks, replay and wasm32 obligations (`temper/docs/design/playground.md`, 9); first-step measurement, story high water still unmeasured, and no new memory bar (`temper/docs/design/playground.md`, 8, 11; `temper/docs/design/playground.status.md`, Not yet 8); handover stall, ignored walk and reset without the box (`temper/docs/design/playground.status.md`, Not yet 4, 9); B0's later transition to story data (`temper/docs/design/playground.md`, 5.3); status form and dated gaps (`DESIGN.md`, 2); test budgets and a world's guide (`skein/docs/foundation/testing-strategy.md`, 8.1; `temper/docs/design/domain/testing.md`, 6.1).
- **Superseded:** none.
- **Not design:** 9 items (parts/dependencies and critical path; overlaps and excluded work's scope; contract C1 module and manifest layout; C2 harness API shapes; C3/C4 construction signatures; measurement seams and temporary knob edits in bounds-measured's answer; runtime GitHub hook and restart factory API repairs in one-process-composed's two answers; document-edit seam, conditional harness edits and missing-report lookup in one-process-documents' answer; dependency wait in one-process-story-a's answer and common edit/check/report rules). The runtime hooks retain the existing worlds' behavior; they do not choose another product backend or stored format.

### playground-4. Story A's pace

- **Kind:** decision
- **Source:** `temper/docs/plan/playground-temper/plan.md`, Parts and tasks, story-a; source date unknown, read 2026-11-04; decided by not stated.
- **Checked:** `temper/docs/design/playground.md`, 5.1: fixed latencies so phases show at 1×, without A's figures; `temper/docs/design/protocol/setup.md`, 8.4: units and 60000 ms ceiling, without A's figures. Searches for 1200 and 4000 found no statement of these story delays. `temper/crates/temper-playground/stories/a/llm.json` and `forge.json` carry them.
- **Target:** `temper/docs/design/playground.md`, 5.2; a paragraph in the section.
- **Text:** Recorded 2026-11-04. Story A's documents give the scripted model a fixed 1200 ms before each reply's first piece and CI a fixed 4000 ms from a head's push to its verdict. These are the story's pace at 1×, so a person can follow the delegate's phases; the frame's speed changes how long the person waits.

### playground-5. A calm world's completed git commands

- **Kind:** contract
- **Source:** `temper/docs/plan/playground-temper/tasks/playground-world.md`, Answers, 2026-11-03; 2026-11-03; decided by not stated.
- **Checked:** `temper/docs/design/playground.md`, 4: calm simulator and peers, but no rule distinguishing successful short-lived git programs from a service or peer ending. `temper/docs/design/domain/testing.md`, 2.5, 4.3: composition and hosted git, without this calm-start assertion. Searches for successful hosted git, calm git and Code(0) found no equivalent design rule.
- **Target:** `temper/docs/design/playground.md`, 9; a paragraph in the section.
- **Text:** Decided 2026-11-03. The calm-start check keeps the service, the fake model peer and the fake forge running. A hosted git command may have ended successfully, including the worker's startup version probe: the world retains completed programs. Every other process end, and a git command's nonzero exit, signal or kill, fails this check. Calm does not require a successful command to remain running.

## temper/docs/plan/playground-decisions

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 1 file (`reset-grace.md`); notes/contracts.md; notes/common.md.
- **Kept:** none.
- **Held:** restart resets both starting and running playgrounds without requesting a box group when no pipe exists (`temper/docs/design/protocol/setup.md`, 8.4); kept task names, release versus erasure, repository-only rows, no revival and independent bound with oldest-covered eviction (`temper/docs/design/domain/forge.md`, 4.1, 5.1; `temper/docs/design/protocol/store.md`, 2); every task erasure tells every connector Unname, including retention when built (`jig/docs/design/domain/connectors.md`, 3.4; `jig/docs/design/domain/connectors.status.md`, Not yet 3.4); deletion carries, admits, saves and resumes its grace, closes conversations as Close, and does not recancel a finishing root (`jig/docs/design/domain/people.md`, 5.2, 5.3, 7); the client supplies the chat template's grace while the core reads no template (`jig/docs/design/domain/people.md`, 5.2, 7); temper's root fills the reset's grace from current template 1 (`temper/docs/design/protocol/setup.md`, 8.4); start acts use the starting person's sign-in (`temper/docs/design/protocol/setup.md`, 8.4); box refusal words direct the person to temper box (`temper/docs/design/client/forge.md`, 5.4); pre-baseline schema changes in place (`jig/docs/design/protocol/README.md`, 7); status form and preserved citations (`DESIGN.md`, 2, 4).
- **Superseded:** C1's bound left open in the mechanism note (now: `temper/docs/design/domain/forge.md`, 4.1, Their bound, explicitly resolves it).
- **Not design:** 7 items (task rows and dependency order; critical path; overlaps; compiler-forced readers and unchanged-watch observation; C1/C2 type, field, route and schema-edit shapes; reset-grace's answer authorizing one extra literal; common ownership, generation, check and reporting instructions).

## jig/docs/plan/wasm32-jig

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** portable playground build and clippy obligation (`temper/docs/design/playground.md`, 3, 9); jig client target/check obligations (`jig/docs/design/protocol/browser.md`, 15, 18; `jig/docs/design/protocol/README.md`, 11).
- **Superseded:** none.
- **Not design:** 7 items (single task and surveyed candidate sites; target-specific lint attribute shapes preserving host types; no-tests/native-clippy choice; warning survey and keep-going commands; other-tree ownership and reports; overlaps and critical path; common machine/toolchain, log, commit and push instructions). No unresolved product choice follows from a pointer-width-dependent lint expectation.

## skein/docs/plan/wasm32-skein

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 1 file (`wasm32-expects.md`); notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** complete portable crate family and lint-as-error obligation (`skein/docs/design/testing.md`, 10); target fences at system-only items, with dead portable code warning (`skein/docs/design/testing.md`, 10; `skein/docs/design/shell.md`, 14).
- **Superseded:** none.
- **Not design:** 8 items (task and candidate survey; new cfg_attr form versus existing pointer-width attributes; native-only dead-item fences as code edits; three-crate measurement and survey commands; other-tree ownership; overlaps and critical path; common check/toolchain/report rules; wasm32-expects' answer permitting allow-invalid on unreachable OwnedFd ban paths). The answer changes lint configuration resolution, preserving the host ban, not a product refusal or portability promise.

## temper/docs/plan/wasm32-list

- **Read:** plan.md; plan.toml, 1 question (`wasm32-more`), text and answer; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md. `.config/gate/README.md`, suggested as a possible holder, is absent.
- **Kept:** none.
- **Held:** portable skein and simulator obligations (`skein/docs/design/testing.md`, 6, 10; `skein/docs/design/lib.md`, 2; `skein/docs/design/shell.md`, 14; `skein/docs/design/world.md`, 8); jig's browser/client/transcript obligations (`jig/docs/design/protocol/browser.md`, 15, 18; `jig/docs/design/protocol/README.md`, 11); temper's client obligations and shell's remaining exclusion (`temper/docs/design/client/browser.md`, 4.3; `temper/docs/design/client/browser.status.md`, Not yet 4.3); smith's hosted face obligation (`smith/docs/design/shell.md`, 2.5); record gaps as they stand rather than the check promised (`DESIGN.md`, 2).
- **Superseded:** whether to measure more packages, keep only six or narrow the design, question wasm32-more (now: `.config/gate/wasm32.txt` has the expanded selection, its answer records 37 additions and 9 reds, and `skein/docs/design/testing.status.md`, Not yet 6, 10, and `smith/docs/design/shell.status.md`, Not yet 2.5, retain the remaining gaps).
- **Not design:** 6 items (two-task staging and atomic list/status commits; combined check and handling unrelated reds; survey for a list description; file ownership and overlaps; estimates; common status-edit, toolchain and check commands). The six-package default was a run setting, not an enduring cap on portable builds.

## temper/docs/plan/wasm32-more

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md absent.
- **Kept:** none.
- **Held:** portable skein family and target-specific system fences (`skein/docs/design/testing.md`, 10); browser client family versus the native shell (`jig/docs/design/protocol/browser.md`, 15; `jig/docs/design/protocol/README.md`, 5); jig transcript, temper client and smith hosted-face obligations (`jig/docs/design/protocol/README.md`, 11; `temper/docs/design/client/browser.md`, 4.3; `smith/docs/design/shell.md`, 2.5); dated status gaps track the measured exclusions (`DESIGN.md`, 2; `skein/docs/design/testing.status.md`, Not yet 6, 10; `smith/docs/design/shell.status.md`, Not yet 2.5; `temper/docs/design/client/browser.status.md`, Not yet 4.3).
- **Superseded:** none.
- **Not design:** 7 items (single warm-slot measurement task; package-name resolution; measuring already reached dependencies separately; list insertion and first-error record shapes; combined-run rollback/rerun mechanics; overlaps and estimates; writing the prior question's answer). Excluding the native shell applies its existing native role; it does not narrow the browser's contract.

## temper/docs/plan/wasm32-rustls

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md absent.
- **Kept:** none.
- **Held:** TLS remains linked and refuses configuration/session/signature on the portable target, preserving seeded plaintext worlds (`skein/docs/design/tls.md`, 4; `skein/docs/design/testing.md`, 10); rustls without std and native-only std/peer server, with the listed impurity exception (`skein/docs/design/tls.md`, 1, 2, 4); standalone TLS's own time provider and disabled resumption (`skein/docs/design/tls.md`, 4, 7); no status history once repaired (`DESIGN.md`, 2).
- **Superseded:** conditional web-clock-versus-another-pin choice if rustls without std failed to build (now: `Cargo.toml`, workspace rustls has ring/tls12 without std, and `skein/docs/design/testing.status.md`, Not yet 6, 10, records TLS's standalone wasm32 clippy passing).
- **Not design:** 7 items (task and feature-graph survey; workspace-merge history; two tests' explicit std dev-dependencies; lib-only gate invocation and feature unification; next-blocker survey/report procedure; adjacent-edit conflict and later removal of a redundant dependency; overlaps and critical path). The redundant browser-clock workaround conflicts with TLS's existing exception list rather than creating a new exception.

## temper/docs/plan/wasm32-temper

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** product-only composition on every target and both playground wasm32 checks (`temper/docs/design/playground.md`, 3, 9); both checks remain missing from the selection at this base (`temper/docs/design/playground.status.md`, Not yet 9); status entries close only the gap actually repaired (`DESIGN.md`, 2).
- **Superseded:** none.
- **Not design:** 6 items (single conditional list task; list/status atomicity and narrowed entry wording; combined features check; red changes nothing and other-tree reporting; overlaps and estimates; common ownership, toolchain and check rules). A failed measurement leaves no open product choice: the desired build is already specified.

## temper/docs/plan/wasm32-temper-2

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** playground and web-shell target obligations (`temper/docs/design/playground.md`, 3, 9); skein dependency coverage obligations (`skein/docs/design/testing.md`, 6, 10; `skein/docs/design/world.md`, 8); smith hosted-face obligation (`smith/docs/design/shell.md`, 2.5); current missing coverage, including the independent web shell (`temper/docs/design/playground.status.md`, Not yet 9; `temper/docs/design/client/browser.status.md`, Not yet 4.3); dated gap-only status accounting (`DESIGN.md`, 2).
- **Superseded:** none.
- **Not design:** 7 items (retry task and warning survey; conditional skein/smith status tasks; dependency graph's measured package counts; deferring another writer's browser-status lines; combined check/feature unification and rollback; overlaps and estimates; common list-reader, ownership, toolchain and check instructions). Dependency counts and this run's failures describe a measurement, not a fixed architectural bound.

## temper/docs/plan/wasm32-temper-3

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** portable composition and separate shell build (`temper/docs/design/playground.md`, 3, 9); current lack of both checks and named blockers (`temper/docs/design/playground.status.md`, Not yet 9); status records only the remaining gaps (`DESIGN.md`, 2).
- **Superseded:** none.
- **Not design:** 7 items (third conditional attempt and prior measurements; assigning dependency statuses to another task; preserving browser-status ownership; list append/conflict mechanics; combined check and reporting unchanged reds; critical path; common first-diagnostic format, toolchain and check instructions). Neither the conditional success branch nor the earlier retries supplies an unresolved product decision.

## smith/docs/plan/live-smith-manual

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** ignored owner-only Anthropic key story, automatic signed-in accounts and owner's explicit selection (`smith/docs/design/testing.md`, 2.3, After landing, and the owner's); key story returns automatically only when a key is provided (`smith/docs/design/testing.md`, 9); lack of a read live proof remains unknown (`smith/docs/design/testing.status.md`, Not yet 2.3).
- **Superseded:** none.
- **Not design:** 5 items (one task and allowed files; choosing the ignore implementation and listing rather than running; survey of readers/profile overlap; critical path and stale-status handoff; common no-credential/no-live-run and reporting instructions). The manual/automatic behavior itself is Held, separately from the attribute edit.

## smith/docs/plan/live-smith-reds

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 1 file (`refresh-lead.md`); notes/contracts.md; notes/common.md.
- **Kept:** playground-6.
- **Held:** live outcomes and typed failure reports bounded to 4 KiB (`smith/docs/design/testing.md`, 2.3, Outcomes and A failure says why); separate invocation traces and spawned-agent .agent records (`smith/docs/design/testing.md`, 2.3, Each run's own records; `smith/docs/design/protocol/events.md`, 2); resumed state and cross-backend history/compaction (`smith/docs/design/protocol/transcript.md`, 4, 8; `smith/docs/design/domain/session.md`, 8.2); refresh lead is 400 days in local settings' seconds, while effective limits use milliseconds (`smith/docs/design/testing.md`, 2.3; `smith/docs/design/shell.md`, 6.5; `smith/docs/design/protocol/limits.md`, 9); lowest-world reproduction and credential-free typed evidence (`smith/docs/design/testing.md`, 2.5, 2.3).
- **Superseded:** interpretation that the refresh lead was 9.6 hours (now: `smith/docs/design/testing.md`, 2.3, explicitly gives 400 days and the answer refutes the premise); designer handoff about events.md claiming a spawned agent's records are in the host stream (now: `smith/docs/design/protocol/events.md`, 2, explicitly separates the streams); an unconditional live refresh expectation (now: `smith/docs/design/testing.md`, 2.3, selects by earliest-refresh facts; `smith/docs/design/testing.status.md`, Not yet 2.3, records that the test still lags).
- **Not design:** 7 items (dated six-red log inventory; task sequence and critical path; CLI-version fixture shapes including dynamic source and null estimated_tokens; report function/type signatures; overlaps and after-landing acceptance mechanics; refresh-lead's constant/helper rename, offline pin and ownership of subsequent failures; common scope, commands and report rules). The CLI fixture fields describe the observed version; they do not constrain a provider's future stream.

### playground-6. Failure reports select safe fields

- **Kind:** contract
- **Source:** `smith/docs/plan/live-smith-reds/notes/contracts.md`, The live report, What a report may hold, and nothing else; source date unknown, read 2026-11-04; decided by not stated.
- **Checked:** `smith/docs/design/testing.md`, 2.3, A failure says why: the bound, broad categories, fixed-form stderr, content exclusions and later short-identifier notice rule are Held, but not the field selection and unknown-payload exclusions below. `smith/docs/design/protocol/events.md`, 3.10: failure records may themselves contain provider code, request_id and detail; being typed does not make those safe report fields. Searches for allow-list/content-free/request_id found no equivalent live-report field restriction. `smith/tests/end-to-end/support/live_report.rs` implements explicit selection.
- **Target:** `smith/docs/design/testing.md`, 2.3; a paragraph in the section.
- **Text:** Recorded 2026-11-04. A live failure report selects fields rather than printing a record or a process result with Debug. It may show the process's exit, forced state, fixed harness error and live-process count; the typed end without detail or setup; a run's resume state, message count, backend, status and turns; its failure's class, attempts, state, exit or end, used and bound, turn, completion class, evidence, phase and status, and smith's own which and reason. It may show a conversation's end, a notice's kind, a tool's verdict and delivery, a check's exit and result, and a session's exit and answered state. Tool names are finish, deliver or a built-in; other names are unknown. Unknown enum payloads are never printed, with the short notice kind admitted above the one exception. Provider detail, code and request_id are left out. Standard error contributes counts and only fixed forms: startup and process-group lines, the chat line, trust and unused-preset notices, numeric turns/spend/time budget lines and smith's credential-state lines, together with the agent-failure form above after its detail is removed.

## smith/docs/plan/live-smith-reds-2

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 2 files (`crossing-twin.md`, `refresh-twin.md`); notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** working-directory-relative prompt paths and Claude's six built-ins/permission prompt, no finish or sub-agent (`smith/docs/design/testing.md`, 2.3, Paths and What the program run is offered; `smith/docs/design/protocol/programs.md`, 5.3, 5.5); file-location, short unknown-notice and redacted agent-failure evidence, and refresh facts (`smith/docs/design/testing.md`, 2.3, A failure says why); carried windows on unconfigured endpoints are compacted by the receiving agent rather than refused (`smith/docs/design/protocol/transcript.md`, 4, 8); start forwards saved turn bytes as told, within the resume limits (`smith/docs/design/protocol/channel.md`, 3; `smith/docs/design/protocol/hosts.md`, 5.3); freshly kept credentials get the half-life floor, loaded records retain their existing lead and issuer earliest-refresh still wins (`skein/docs/design/oauth.md`, 6.3); live refresh case selection is left as a code gap rather than a new open choice (`smith/docs/design/testing.md`, 2.3; `smith/docs/design/testing.status.md`, Not yet 2.3).
- **Superseded:** after-landing's conditional question about what to expect from a fresh issuer record (now: `smith/docs/design/testing.md`, 2.3, has explicit absent/past, distant, near and missing-refresh-token cases); the answer's fallback of preserving endpoint names in decoded turns if saved bytes cannot be forwarded (now: `smith/crates/smith-local-protocol/src/store.rs` constructs `Transcript::from_saved`, carrying the saved piece beside decoded history, and `smith/crates/smith-host-protocol/src/tests.rs`, `a_resumed_start_forwards_saved_turns_from_an_unconfigured_endpoint`, pins the chosen path; the wire remains `smith/docs/design/protocol/channel.md`, 3).
- **Not design:** 7 items (dated red/cause table; task assignments and critical path; overlaps; after-landing observation/repair routing; crossing-twin answer's scope, representation alternatives and test/edit order; refresh-twin answer's crate scope, fixture preservation and separately owned abort; common scope, credential and check/report instructions). The answers' behavioral reasons are Held above; their permissions do not become design text.

## temper/docs/plan/live-temper

- **Read:** plan.md; plan.toml, 1 question (`pair-timeout`), text and answer; tasks' Answers, 1 file (`pair-stories.md`, two dated responses); notes/contracts.md; notes/common.md.
- **Kept:** none.
- **Held:** automatic owned-forge landing pair alone, other stories owner-only, and owner's script selections (`temper/docs/design/domain/testing.md`, 2.7, After every landing, What stays the owner's, Outside the suites); separate fresh/kept stores and held-record move only into a fresh store (`temper/docs/design/domain/testing.md`, 2.7); model choices gpt-6-luna and claude-haiku-5-5 (`temper/docs/design/domain/testing.md`, 2.7); private TMPDIR, record return hook, stat-only login checks, leftover process cleanup and required modes (`temper/docs/design/domain/testing.md`, 2.7, The runner's part and What the stories need); focused harness checks occur once (`temper/docs/design/domain/testing.md`, 2.7); pair-timeout answer keeps 50 minutes with a future figure based on first pair wall times, each story capped at 40 minutes (`temper/docs/design/domain/testing.md`, 11, The live pair's bound); binaries table names executable prerequisites rather than test targets (`tools/tiers/design.md`, 2.1).
- **Superseded:** none.
- **Not design:** 7 items (three tasks and critical path; overlaps; exact profile/filter and package-map edit mechanics; launcher default-path preservation and proof-script repairs; removed duplicate-test inventory; pair-stories' shellcheck fallback and permitted use-line edit; common edit/check/report and no-live-run rules). The answer's x1.5 recalibration procedure belongs to setting a future run budget; the design already holds the retained bound and the user's ownership, so no new budget is proposed.

## temper/docs/plan/live-temper-stories

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** absent address is DIRECTORY not signed in, other DIRECTORY failures name step/error, and FILE failures keep their own cause (`temper/docs/design/client/tools.md`, 8.1); one signed-in client DIRECTORY for import, return hook and all sitting reads (`temper/docs/design/domain/testing.md`, 2.7, The owned-forge landing story); ended-delegate cursor paging, no cut without cursor and 32-page bound (`temper/docs/design/domain/testing.md`, 2.7); runner's absolute store, writable parent of record, private files/TMPDIR and no ancestor mode/name requirement (`temper/docs/design/domain/testing.md`, 2.7, What the stories need); gap-only status closeout (`DESIGN.md`, 2).
- **Superseded:** none.
- **Not design:** 6 items (two dated failure diagnoses and timings; three-task split and critical path; fixing the caller's wrong directory separately from ctl's error wording; overlaps; shared-type/no-contract-note choice; common scope, credential, stdin and check/report rules). The diagnosis introduces no extra requirement on directories above DIRECTORY or FILE.

## temper/docs/plan/live-temper-stories-2

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** terminal changes keep CI at the exact head across restart (`temper/docs/design/domain/forge.md`, 8.8); the landed page says Passed on both runs and unknown has the narrower no-read-before-landing meaning (`temper/docs/design/domain/testing.md`, 2.7; `temper/docs/design/client/README.md`, 5.10); harness step checks run once (`temper/docs/design/domain/testing.md`, 2.7).
- **Superseded:** landed CI must say Not read since the restart (now: `temper/docs/design/domain/forge.md`, 8.8, and `temper/docs/design/client/README.md`, 5.10, retain a landed verdict).
- **Not design:** 5 items (one repair task and critical path; dated code/test drift diagnosis; shared assert_landed_ci helper rather than two assertions; no red-before and no contract note; overlaps and common scope/no-live/check/report instructions). What later assertions had not yet run live is an observation, not a passing claim or an open design choice.

## temper/docs/plan/openai-manual

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** none.
- **Held:** two positive public Responses stories manual for lack of a suitable scoped login, alongside the three Anthropic key stories (`skein/docs/design/llm.status.md`, Departs 5); unauthorized OpenAI story remains automatic with the supplied token replaced (`skein/docs/design/llm.status.md`, Departs 5); owner invocation and possible future restoration with a scoped sign-in are deferred (`skein/docs/design/llm.md`, 6, How the owner runs the live stories).
- **Superseded:** none.
- **Not design:** 5 items (one attribute/status/README task and critical path; overlaps and reader survey; exact test listing counts and offline-only checks; nextest list subcommand and profile exclusions; common ownership, credential and report instructions). The observed 401 is already the reason in the status; no new provider/API claim is inferred from it.

## temper/docs/plan/citation-burndown

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md; notes/findings.md, decisions defining citations and exclusions (the 67-finding inventory is Not design).
- **Kept:** none.
- **Held:** absolute no-broken-citation rule and stable surviving section numbers (`DESIGN.md`, 4; `tools/citations/README.md`, The full run); arbitrary parent traversal, tracked non-Markdown links and query/anchor stripping, narrow possessive/parenthesized sections, inherited owner and list-heading target (R1/R2/R4/R8/R9) (`tools/citations/README.md`, What is read); capitalized bare-reference exclusions, RFC sections, counts and bare possessive numbers (R3/R5/R6) (`tools/citations/README.md`, Not citations); By step status labels excluded while their citations still count (R7) (`tools/citations/README.md`, Not citations); code strings as data, other repositories' paths and explicit allow-list exceptions (`tools/citations/README.md`, What is read, Not citations, allow.txt); committed-tree checking and deliberate plan/dated-development exclusions (`tools/citations/README.md`, What is read).
- **Superseded:** none.
- **Not design:** 8 items (67-finding inventory and owner table; task order and critical path; numbered store headings preserving cited item numbers; individual path/README citation repairs; allow.txt scheduling and count checks; qa.md deletion deferred because its reader remains, and test-support comment scope; overlaps and external gate-switch handoff; common narrow-edit, reader-test and reporting rules). The allow-list examples are historical findings, not new blanket exclusions.

## temper/docs/plan/collector-starved

- **Read:** plan.md; plan.toml, 0 questions; tasks' Answers, 0 files; notes/contracts.md absent; notes/common.md.
- **Kept:** playground-7.
- **Held:** only a starved run's recordless SIGTERM/SIGKILL cancellation abort is unobserved, with identity, count, warning and exit 0 absent other failures, and all other missing records fail coverage (`tools/tiers/design.md`, 7.3); outcome-based attempt reconciliation includes retries, with offline cancellation proof (`tools/tiers/design.md`, 7.3, 7.4); downstream readers must map unobserved separately from raw JUnit failure (`tools/tiers/design.md`, 8); recorded cancellation-abort reconciliation in the standalone summary remains later (`tools/tiers/design.md`, 8); status files record gaps and then No gap (`DESIGN.md`, 2; `tools/tiers/design.status.md`).
- **Superseded:** none.
- **Not design:** 6 items (particular cancelled run's four missing records and diagnostic; one-task scope and tests; where producers set the starved flag; preserving Python function names/six-argument replacement seam; downstream change ownership, overlaps and critical path; common build/test/commit/report instructions). Detecting the signal requires an abort type and a word-bounded signal number, not arbitrary stderr containing a number; the design's named nextest cancellation form holds that rule.

### playground-7. Empty unobserved fields stay present

- **Kind:** contract
- **Source:** `temper/docs/plan/collector-starved/plan.md`, Calls made, New keys only; source date unknown, read 2026-11-04; decided by not stated.
- **Checked:** `tools/tiers/design.md`, 7.3: defines the unobserved identities and count when cancellation occurs; 7.5: tools summaries and coverage; neither states that both new keys appear in every summary, tools suites included. The contract adds keys without changing any existing one. Searches for unobserved across the design found the outcome rule and downstream mapping, not the always-present empty form.
- **Target:** `tools/tiers/design.md`, 7.3; a paragraph in the section.
- **Text:** Recorded 2026-11-04. Every summary includes the top-level `unobserved` list and the suite's `unobserved` count, including a tools-suite summary. With no unobserved attempt they are an empty list and zero. These are additional fields; existing summary keys keep their names and meanings.
