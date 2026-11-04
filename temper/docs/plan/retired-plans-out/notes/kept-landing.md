# kept: landing

Read at `9c608917eb`, 2026-11-04. 6 directories, 26 findings.

Non-design counts group each directory's plan logistics into one item, each contract
heading's remaining code shapes into one, each common-note heading's remaining run rules
into one, and the settings note's editing mechanics into one. Product rules within those
headings are accounted for separately under Held, Superseded or Kept. Each Answers file's
operational remainder likewise counts once, including any Rebase heading after Answers.
An undated source is marked “date not stated”; the dates in finding Text record this audit,
not an inferred date of decision. No finding asserts that an unwritten contract is built.

## temper/docs/plan/reference-m5

- **Read:** plan.md whole; plan.toml, 0 questions; tasks' Answers, 5 files
  (foreman-charter, person-answers-chats, relay-stall, relay-story, relay-up), through EOF;
  notes/contracts.md, C1–C8 and Revisions; notes/common.md; notes/settings.md whole.
- **Kept:** landing-1, landing-2, landing-3, landing-4, landing-5, landing-6, landing-7.
- **Held:** C1/C6's three routes, default, four-question allowance, hold and escalation
  (`temper/docs/design/domain/forge.md`, 8.1, 8.11;
  `temper/docs/design/domain/agent.md`, 4.3.1); C2/C7's ordered, durable questions, whole
  words, current asker, dependency-held answers, ended askers, finish prefix and Foreman
  reasons (`temper/docs/design/domain/forge.md`, 8.11;
  `temper/docs/design/protocol/store.md`, 2); C4/C5's read-only charter 6, three verdicts,
  landing-tip workspace, read grants and missing-authority fallback
  (`temper/docs/design/protocol/configuration.md`, 13.4;
  `temper/docs/design/domain/forge.md`, 8.11);
  plan/C4's constant charter 6 pending kind 3's number 3 and unbuilt kind 6 default
  (`temper/docs/design/protocol/configuration.status.md`, Not yet, 13.2, 13.4);
  C6's control-family check and extra task per
  question (`temper/docs/design/domain/agent.md`, 4.3.1); C8's panel/wire, first 8 questions,
  256-byte UTF-8 first lines, state words and hidden answer bodies
  (`temper/docs/design/client/README.md`, 4.6;
  `temper/docs/design/client/forge.md`, 9.2;
  `temper/docs/design/client/tools.md`, 5.5); revisions' accepted-step atomicity
  (`jig/docs/design/domain/connectors.md`, 6.3;
  `jig/docs/design/domain/engine.md`, 5.1–5.2); revisions' dependency test and reset on
  release (`temper/docs/design/domain/forge.md`, 8.11); relay-stall's answer-time stall
  restart (`temper/docs/design/domain/forge.md`, 8.4;
  `jig/docs/design/domain/connectors.md`, 6.2); person-answers-chats and relay-story's
  person-answer switch restricted to conversations (`jig/docs/design/domain/people.status.md`,
  Departs, 6, 8); settings' coordinator answers, amendments, new dependencies, person tasks,
  hold decisions and coding-agent questions (`temper/docs/design/protocol/configuration.md`,
  13.9); common's tree boundaries, deterministic worlds, schema rules and code conventions
  (`AGENTS.md`, 1–2; `skein/docs/foundation/programming-model.md`, 1;
  `skein/docs/foundation/testing-strategy.md`, 1;
  `temper/docs/design/protocol/store.md`, 4).
- **Superseded:** C2's question words read later from question positions
  (now: `temper/docs/design/domain/forge.md`, 8.11, whole words on the record);
  plan's eight executor places and walk's temporary nine
  (now: `temper/docs/design/protocol/configuration.md`, 13.4, twelve;
  13.9, reference sixteen); C7's original room-short handling and C2's original states as
  the final contract (now: landing-4 preserves Revisions' C2/C7 revision 3).
- **Not design:** 17 items: plan logistics (1), C1–C8's code-only residues (8), Revisions'
  bookkeeping, optional parameter-3 workaround and world sizing (1), common's run
  instructions (1), settings' paragraph-edit mechanics (1), and the five Answers files'
  scope, fixture, command and merge instructions (5).

### landing-1. The question-route parameter

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/notes/contracts.md`, C3; date not stated;
  decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.1: describes the route and other
  numbered parameters, without this parameter's encoding; `temper/docs/design/domain/forge.status.md`,
  Not yet, 8.1: mentions parameter 14, without its values.
- **Target:** `temper/docs/design/domain/forge.md`, 8.1; a paragraph in the section.
- **Text:** Recorded 2026-11-04: the question route is number parameter 14: 1 for
  `foreman`, 2 for `up`; absent means `none`. The root writes it only for `foreman` or
  `up`. Any other value refuses the change's start as a malformed parameter.

### landing-2. Open-question admission

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/notes/contracts.md`, C1, C2 and C7;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.11: bounds questions taken, without
  the open-queue ceiling or overflow response; `temper/docs/design/client/README.md`, 4.6:
  holds the separate panel limit of eight and the first-line limit of 256 bytes.
- **Target:** `temper/docs/design/domain/forge.md`, 8.11; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a change keeps at most 16 open questions. Its configured
  question allowance must be less than 16, leaving room for the question past that
  allowance. A question beyond the open-queue ceiling is answered with the refusal used
  by a change that takes no questions; it never enters the relay.

### landing-3. A bounded foreman brief and verdict

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/notes/contracts.md`, C5 and Revisions,
  C5/C6 revision 2; date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.11: says the brief carries both
  texts whole and gives the verdict labels, without a fitting priority or result allowance;
  `temper/docs/design/domain/agent.md`, 4.4: verdict choices have words bounds.
- **Target:** `temper/docs/design/domain/forge.md`, 8.11; a paragraph in the section.
- **Text:** Recorded 2026-11-04: the foreman's brief fits the deployment's brief bound by
  cutting the change's words from their end and marking truncation before cutting anything
  else; the question's words are never cut. Its three verdict choices, in order, are
  code 1 `answer`, code 2 `finish`, code 3 `escalate`, each requiring words and no
  fields. Its words fit both `tasks.result_bytes` and `tasks.message_bytes` less 64 bytes,
  reserving room for the finish prefix in the answer relayed down.

### landing-4. Retaining relay inputs across short steps

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/notes/contracts.md`, C7 and Revisions,
  C2/C7 revision 3; date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.11: orders the questions and holds
  answers for dependencies, but does not specify room-short retention, pending reasons,
  duplicate identity or a late foreman's result; `jig/docs/design/domain/connectors.md`,
  6.3: holds accepted-step atomicity, rather than those relay-specific rules. The current
  `temper/crates/temper-engine-domain-forge-change/src/relay.rs` lacks revision 3's `Up`
  state: this is an intended contract, not evidence of completed implementation.
- **Target:** `temper/docs/design/domain/forge.md`, 8.11; a paragraph in the section.
- **Text:** Recorded 2026-11-04: every relay input read at a step is retained before any
  message is sent. A question is taken once by its asker and number. Reasons awaiting
  passing up, and answers awaiting passing down, stay on the record even when the step
  has no message room; stored pending reasons have a ceiling of 65,536 bytes. Passing up
  after release does not check the spent allowance again.
  Only the result of the current question's assigned foreman settles it; other foreman
  results are dropped. Ask and answer messages share the step's `tasks.batch` with its
  other replies. Work left solely for lack of message room makes the next step due at once.

### landing-5. Relayed answers are fitted too

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/notes/contracts.md`, Revisions,
  C2/C7 revision 3, Answer bullet; date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.11: explicitly fits the question
  passed up, but says the answer goes down whole;
  `temper/docs/design/domain/forge.status.md`, Departs: records only the foreman spend split.
- **Target:** `temper/docs/design/domain/forge.status.md`, Departs, 8.11; a **Departs** entry.
- **Text:** 8.11: recorded 2026-11-04, every answer relayed down is cut from its end to
  `tasks.message_bytes` and marked with `…`, as a question passed up is. This keeps the
  message within its admission bound, where the design says the answer's words are whole.

### landing-6. Stored relay text bounds

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m5/tasks/relay-up.md`, Answers, 2026-11-03;
  `temper/docs/plan/reference-m5/notes/contracts.md`, C2 and Revisions, C2 revision 2;
  2026-11-03 for the answer, contract date not stated; decided by not stated.
- **Checked:** `temper/docs/design/protocol/store.md`, 2, Change, and 4: keeps question
  words and held answers, with runtime bounds checked against schema ceilings, but gives
  neither text's runtime binding; `temper/crates/temper-protocol-store/src/lib.rs`, bounds:
  binds both `open_words` and `held_answer` to `forge.client.op_bytes`.
- **Target:** `temper/docs/design/protocol/store.md`, 4; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a stored open question's whole words and a held answer's
  words each have a schema ceiling of 65,536 bytes. Their runtime bounds, `open_words`
  and `held_answer`, both come from `forge.client.op_bytes`; the start refuses a bound
  above the ceiling by the ordinary schema-bound check.

### landing-7. Inherited reds in the reference's foreman instructions

- **Kind:** decision
- **Source:** `temper/docs/plan/reference-m5/notes/settings.md`, Charter 6, the foreman's;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/protocol/configuration.md`, 13.9: charter 6 says finish
  for a file outside the change's own and escalate an open choice; charter 2 puts failures
  outside its files on Left. Neither explicitly covers a red the change did not cause in
  a file it does own. 13.4's general finish rule may cover it; retained for that distinction.
- **Target:** `temper/docs/design/protocol/configuration.md`, 13.9; a sentence in the section.
- **Text:** Recorded 2026-11-04: the reference's charter 6 also returns `finish` for a red
  the change did not cause, with that red on the asker's Left list.

## temper/docs/plan/reference-m7a

- **Read:** plan.md whole; plan.toml, 0 questions; tasks' Answers, 6 files
  (core-jobs-run, fleet-jobs, service-jobs, tasks-jobs-surface, tooling-capability,
  worker-jobs), through EOF; notes/contracts.md, 1–8; notes/common.md.
- **Kept:** landing-8, landing-9, landing-10, landing-11.
- **Held:** plan's in-place formats and common's append-only schema/total translations
  (`jig/docs/design/protocol/README.md`, 7;
  `jig/docs/design/protocol/store.md`, 5, 5.1);
  contracts 1–3's ascending distinct hello kinds, entrance bounds, hosted phases, typed
  failures, application-only grants, acknowledgement/redial and no job traffic for turns
  (`jig/docs/design/protocol/channel.md`, 4.3, 5.1, 5.4;
  `jig/docs/design/domain/hosts.md`, 6.9); contract 2's lifecycle, no partial delivery after
  cancellation, process drain, no save and absent capability
  (`jig/docs/design/domain/hosts.md`, 6.9;
  `skein/docs/foundation/programming-model.md`, 1);
  contracts 4–6's procedure-only creation, no job messages/amendments, zero LLM spend,
  declared bounds overriding the proposed contract, job delegation, placement, claim and
  restart (`jig/docs/design/domain/tasks.md`, 5.3.1;
  `jig/docs/design/domain/engine.md`, 7.5;
  `jig/docs/design/protocol/store.md`, 5);
  contract 7's no party Create and opaque application parts
  (`jig/docs/design/client/domain/contract.md`, 5.7);
  contract 8's capability entries, spec/result ownership, exit 0/3, failed or missing result,
  exact credential-free environment and wall deadline
  (`temper/docs/design/domain/worker.md`, 4.4;
  `temper/docs/design/protocol/tooling.md`, 1–2);
  prepare/key and Rewrite deferred (`temper/docs/design/domain/worker.status.md`, Not yet,
  4.4);
  plan/common's domain/protocol separation and deterministic tests
  (`skein/docs/foundation/programming-model.md`, 1;
  `skein/docs/foundation/testing-strategy.md`, 1).
- **Superseded:** plan's no declared production kind and configuration without job kinds
  (now: `temper/docs/design/domain/worker.md`, 4.4, merge kind 3;
  `jig/docs/design/protocol/configuration.md`, 3, `job` delegation);
  contract 8's command forced under home and deferred directory layout
  (now: `temper/docs/design/domain/worker.md`, 4.4;
  `temper/docs/design/protocol/tooling.status.md`, Departs, 3, checkout repositories by name);
  worker-jobs' deferred merge-parent delivery as still open
  (now: `temper/docs/design/domain/worker.md`, 4.4, merge delivery commits both parents).
- **Not design:** 16 items: plan logistics (1), contracts 1–8's code-only residues (8),
  common's run instructions (1), and six Answers files' scope, waiting, fixture sizing and
  command instructions (6). The deferred local HEAD operation is scope, not a new delivery
  decision; the later merge rule above holds the product behavior.

### landing-8. Job batch refusal categories

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7a/plan.md`, Calls made, Designer's defaults;
  `temper/docs/plan/reference-m7a/notes/contracts.md`, 1 and 5;
  date not stated; decided by not stated.
- **Checked:** `jig/docs/design/domain/engine.md`, 7.5: refuses undeclared kinds and
  excessive spec/deadline, but does not distinguish their refusal categories or order the
  application's declaration; `jig/docs/design/protocol/channel.md`, 4.3: orders the hello;
  `jig/docs/design/protocol/configuration.md`, 3: defers kind validation to the core.
- **Target:** `jig/docs/design/domain/engine.md`, 7.5; a sentence in the section.
- **Text:** Recorded 2026-11-04: the application's declaration of job kinds is ascending
  by number, each once, as the worker's hello is. A batch naming an undeclared job kind
  is refused as `Executor`; a job's spec or deadline beyond its declared bound is
  refused as `Spec`.

### landing-9. Job failures use the task retry classes

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7a/notes/contracts.md`, 4, failure classes and
  answer mapping; date not stated; decided by not stated.
- **Checked:** `jig/docs/design/domain/tasks.md`, 5.5: defines retry classes;
  `jig/docs/design/domain/hosts.md`, 6.9, and `jig/docs/design/protocol/channel.md`, 5.4:
  enumerate job failures but leave their complete class mapping implicit.
- **Target:** `jig/docs/design/domain/engine.md`, 7.5; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a job's transient preparation or failed delivery uses the
  transient retry class; permanent preparation, stale or refused delivery, and an invalid
  assignment use the permanent class. Its own failure and deadline use the run's-own
  class; failure to start uses the agent's class; an oversized result uses invalid; lost
  contact and shutdown use lost. Engine cancellation ends it as cancelled. Busy before
  starting is a refusal, spending no try. A finished job becomes its task's Job result,
  without asking to cancel delegates.

### landing-10. The job executor's visible name

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7a/notes/contracts.md`, 7, Text a client shows;
  date not stated; decided by not stated.
- **Checked:** `jig/docs/design/client/domain/contract.md`, 5.7: shows the job kind,
  without its text spelling; `temper/docs/design/client/tools.md`, 5.5: gives the task
  reads without specifying this executor label.
- **Target:** `jig/docs/design/client/domain/contract.md`, 5.7; a sentence in the section.
- **Text:** Recorded 2026-11-04: the job executor's visible name is `job KIND`, KIND its
  numeric application kind, in the shared client and in temper's command-line task reads.

### landing-11. Job command paths are checked before starting

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7a/notes/contracts.md`, 8, A job's steps;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/worker.md`, 4.4: gives the command path and
  entrance limits; `temper/docs/design/protocol/tooling.md`, 2: says it is relative to
  the job directory. Neither explicitly rejects a parent segment before starting.
- **Target:** `temper/docs/design/domain/worker.md`, 4.4; a sentence in the section.
- **Text:** Recorded 2026-11-04: before writing the request or starting the command, the
  job capability refuses a command path that is absolute or contains a `..` segment.
  The path resolves relative to the job's workspace root.

## temper/docs/plan/reference-m7b

- **Read:** plan.md whole; plan.toml, 0 questions; tasks' Answers, 7 files
  (catch-up-call, change-merge-route, change-merge-surface, merge-help, merge-kind,
  merge-r1-paths, merge-settings), through EOF; notes/contracts.md, C1–C6, C3.1, R1, R2;
  notes/common.md, including Corrections found after release.
- **Kept:** landing-12, landing-13, landing-14.
- **Held:** plan/C1/R2's kind 3, verb, delivery, 4,096/512 bounds, ten-minute deadline,
  conditional engine declaration and root-owned spec encoding
  (`temper/docs/design/domain/worker.md`, 4.4);
  C2/C3/C3.1's merge-in-progress workspace, branch reread rather than result parsing,
  ordinary conflict's one resolution charge, same-base fallback and ordinary landing path
  (`temper/docs/design/domain/forge.md`, 8.2, 8.4, 10);
  C4/R2's stored merge result within the kind's result bound and in-place version
  (`temper/docs/design/domain/worker.md`, 4.4;
  `temper/docs/design/protocol/store.md`, 4);
  C5/C6's optional command, authority inheritance, oversized-spec fallback, merge refusals,
  control-family restriction and parameter 16
  (`temper/docs/design/protocol/configuration.md`, 3;
  `temper/docs/design/domain/agent.md`, 4.3.1;
  `temper/docs/design/domain/forge.md`, 8.1, 8.4);
  R1's repository name, directory-relative command, same-repository scripts and R2's
  bounded index-order result with more count (`temper/docs/design/domain/worker.md`, 4.4;
  `temper/docs/design/protocol/tooling.md`, 2, 4.1);
  R2's no job words/parameters/inputs or LLM (`jig/docs/design/domain/tasks.md`, 5.3.1);
  change-merge-route's prepared/typed workspace, shared workstream, attempt cleanup and
  connector-supplied writer names (`jig/docs/design/domain/engine.md`, 7.1, 7.5;
  `jig/docs/design/domain/hosts.md`, 7);
  catch-up-call's contents rather than landed ancestry under squash
  (`temper/docs/design/domain/forge.md`, 8.2 step 7, 10, 11.1);
  prepare/key, configurable tooling limits and reuse of the job tree left for later
  (`temper/docs/design/domain/worker.status.md`, Not yet, 4.4;
  `temper/docs/design/protocol/tooling.md`, 7);
  reference additions over stock (`temper/docs/design/protocol/configuration.md`, 13.9);
  common's schemas and testing rules (`temper/docs/design/protocol/store.md`, 4;
  `skein/docs/foundation/testing-strategy.md`, 1).
- **Superseded:** C1/C4's 8 KiB spec and 64 KiB result and unconditional engine declaration
  (now: `temper/docs/design/domain/worker.md`, 4.4, R2 bounds and conditional declaration);
  C2's home-named merge repository and home-relative command
  (now: `temper/docs/design/protocol/tooling.md`, 4.1, repository's own name);
  plan's stock six-executor sizing (now: `temper/docs/design/protocol/configuration.md`,
  13.4, twelve); R2/merge-kind's claim that a 512-byte result cap is jig's settings schema
  (now: `temper/docs/design/protocol/configuration.md`, 13.9, larger design allowance and
  its status's Departs, store/carrier bounds);
  deferring a goal landing's merge path because it did not exist
  (now: `temper/docs/design/domain/forge.status.md`, Not yet, 8.4, 10, 11.1: built path,
  missing dedicated story).
- **Not design:** 19 items: plan logistics (1), code-only residues of C1–C6, C3.1, R1 and
  R2 (9), common's run rules and Corrections' paths/test/edit mechanics (2), and seven
  Answers files' operational residues (7). Setter placement, literal prices, fixture
  dependencies and conflict resolution introduce no additional product rule.

### landing-12. The merge request's byte spelling

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7b/notes/contracts.md`, C2 and C6;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/protocol/tooling.md`, 4.1: gives the three JSON members;
  4 gives hexadecimal commit lengths, but neither specifies compact encoding, member
  order or lowercase. `temper/docs/design/domain/worker.md`, 4.4: root encodes the spec.
- **Target:** `temper/docs/design/protocol/tooling.md`, 4.1; a paragraph in the section.
- **Text:** Recorded 2026-11-04: temper writes the merge request exactly as
  `{"version":1,"head":"HEAD","base":"BASE"}`, without spaces, in that member order.
  Each commit is lowercase hexadecimal, 40 digits for SHA-1 and 64 for SHA-256. The
  catch-up commit carried by bytes parameter 16 uses that same text form; another form
  refuses the change's start.

### landing-13. Catch-up production and resolution accounting

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7b/notes/contracts.md`, C3.1;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/forge.md`, 8.2 step 1: a catch-up's producer is
  the merge job, then a resolution if needed; 8.4 charges a merge job one resolution and
  its following resolution none, without stating the catch-up exception.
- **Target:** `temper/docs/design/domain/forge.md`, 8.4; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a catch-up's first merge job is its producer and spends
  no resolution allowance. If it leaves conflicts, its following resolution spends one.
  This differs from a conflict after production, whose merge job spends one and whose
  following resolution spends none. A resolution asked again after release is a
  resolution directly, with no repeated merge job before it.

### landing-14. Merge-command path admission

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m7b/notes/contracts.md`, C5;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/protocol/configuration.md`, 3: optional relative path,
  at most 4,096 bytes, without parent segments, refusal at its member; it does not say
  empty and NUL-containing paths are invalid or distinguish form from byte-bound refusal.
- **Target:** `temper/docs/design/protocol/configuration.md`, 3; a paragraph in the section.
- **Text:** Recorded 2026-11-04: `merge_job`, when present, is a string of 1 to 4,096
  bytes, with no leading `/`, NUL or `..` segment. An invalid form refuses the start at
  `application.landing.projects[i].merge_job` as a form error; a length past the ceiling
  is a bound error. Leaving it out configures no merge command.

## temper/docs/plan/reference-m8

- **Read:** plan.md whole; plan.toml, 0 questions; tasks' Answers, 9 files
  (help-pages, panel-steps, tiers-config, tiers-reports, tiers-root, tiers-runs,
  tiers-stories, tiers-surface, tiers-verification), through EOF; notes/contracts.md, 1–6;
  notes/common.md. No notes/old/ or patches read.
- **Kept:** landing-15, landing-16, landing-17, landing-18, landing-19.
- **Held:** plan/contracts 1's pure policy, records and commit-before-dispatch ownership
  (`skein/docs/foundation/programming-model.md`, 1;
  `jig/docs/design/domain/engine.md`, 5.1–5.2;
  `temper/docs/design/domain/after-landing.md`, 10);
  contracts 1–2's retry/release rule, baseline, inherited failures, first-failing landing,
  unchanged-key skip, verification, red wait, cancellation and no-verdict bound
  (`temper/docs/design/domain/after-landing.md`, 2–6);
  contract 2/plan's unfollowed landing finishing at once (the retained Rust state name
  Landed is code shape) (`temper/docs/design/domain/after-landing.md`, 2, 6);
  contract 3's branch selection, retry and verification defaults, per-repository tier bound
  (`temper/docs/design/protocol/configuration.md`, 3;
  `temper/docs/design/domain/after-landing.md`, 10–11);
  contract 4's procedure, purpose tag 8 and operation ordering, verification rows, faults,
  clearing result words, repaired run retained rather than just its commit, and deferred
  execution/status operations (`temper/docs/design/domain/after-landing.md`, 3, 6, 10;
  `temper/docs/design/protocol/store.md`, 2;
  `temper/docs/design/domain/after-landing.status.md`, Not yet, 9–10);
  plan's family numbers 28–33 (`temper/docs/design/protocol/store.md`, 2);
  contract 5's artifact/range reads, private inflater, fake selection/flakes and GitHub
  refusal (`skein/docs/design/forge.md`, 2.1, 3.2–3.3, 5.2;
  `skein/docs/design/fake-forge.md`, 2.3;
  `skein/docs/design/forge-github.status.md`, Not yet, 4.5);
  contract 6/tiers-root/tiers-runs' connector-owned deadlines, one per tier task, wake on
  expiry and restart (`jig/docs/design/domain/connectors.md`, 6.2;
  `temper/docs/design/domain/after-landing.md`, 10;
  `temper/docs/design/domain/forge.md`, 16);
  panel/help's verifying, red, cleared and until-cleared behavior
  (`temper/docs/design/client/forge.md`, 2.1;
  `temper/docs/design/client/tools.md`, 5.4–5.5);
  plan's remaining landing log, late news, findings, suspects and red pause gaps
  (`temper/docs/design/domain/after-landing.status.md`, Not yet, 5.2–10);
  common's code, world and schema rules (`skein/docs/foundation/programming-model.md`, 1;
  `skein/docs/foundation/testing-strategy.md`, 1;
  `temper/docs/design/protocol/store.md`, 4).
- **Superseded:** contract 1's comment calling the policy family 25
  (now: `temper/docs/design/protocol/store.md`, 2, Tier is 28, Forge is 25);
  contract 3's branch-only shape and absence of span
  (now: `temper/docs/design/domain/after-landing.md`, 2–3, goal branches and span);
  contract 4's repair commit alone
  (now: `temper/docs/design/domain/after-landing.md`, 6, retained clearing run;
  panel-steps' dated revision 1); early minimal tier-result wording as the full report
  (now: `temper/docs/design/domain/after-landing.md`, 10, per-landing attributions and
  bounded remainder).
- **Not design:** 17 items: plan logistics (1), contracts 1–6's code-only residues (6),
  common's run instructions (1), and nine Answers files' scope, dependency, command,
  fixture accounting, help-fitting and merge instructions (9). The separate timer map
  and its precise capacity implement the held one-deadline-per-procedure rule.

### landing-15. Ordered probes rather than bisection

- **Kind:** decision
- **Source:** `temper/docs/plan/reference-m8/notes/contracts.md`, 1, policy decisions;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 5.2: says bisected, oldest
  first, first failing landing owns a test, skipping unchanged keys; its status's Not yet,
  5.2 calls every range bisected. `temper/crates/temper-engine-domain-forge-tiers/src/verdict.rs`,
  probe: visits successive changed-key candidates, assigning remaining failures to the
  last without probing it.
- **Target:** `temper/docs/design/domain/after-landing.status.md`, Departs, 5.2;
  a **Departs** entry.
- **Text:** 5.2: recorded 2026-11-04, probes visit changed-key landings linearly, oldest
  first, rather than bisecting the range. The first candidate whose commit fails a test
  owns it; the last candidate owns what remains without a probe. Unchanged-key landings
  are skipped. No reason for choosing this traversal over bisection is recorded.

### landing-16. Overflow failure verdicts

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m8/notes/contracts.md`, 1, policy decisions;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 4: calls overflow many and
  blames by landing; 5.2: says rerun failed tests once; 5.3: says later passing failures
  repairs a landing, without saying how unenumerated failures are handled.
- **Target:** `temper/docs/design/domain/after-landing.md`, 5.2; a paragraph in the section.
- **Text:** Recorded 2026-11-04: failures reported as many have no failed-test rerun;
  their probes run the whole tier. Enumerated failure IDs are sorted and unique before
  judging. A landing blamed for many is repaired by a later
  judged run whose failures can be enumerated by test, even if that run fails other
  tests; a later many verdict does not repair it.

### landing-17. A failed receipt skips confirmation

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m8/notes/contracts.md`, 1, policy decisions;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 2–3: a failed receipt keeps
  IDs and a tip with a receipt is judged without a run. 5.2 describes the ordinary
  confirmation rerun, without explicitly stating how the failed receipt enters blame.
- **Target:** `temper/docs/design/domain/after-landing.md`, 3; a sentence in the section.
- **Text:** Recorded 2026-11-04: a failed receipt at the tip is treated as a failed-test
  rerun that failed again. Its failures go directly to inheritance and blame, with no
  further confirmation run.

### landing-18. The tier's cumulative flake bound

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m8/notes/contracts.md`, 1, policy decisions;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 5.2: keeps and counts flakes
  by test; 10 bounds failing tests per run, but not cumulative distinct flake IDs or
  overflow retention.
- **Target:** `temper/docs/design/domain/after-landing.md`, 10; a sentence in the section.
- **Text:** Recorded 2026-11-04: a tier's record keeps at most `failing_tests` distinct
  flake IDs. Once full it keeps counting the IDs already retained and drops newly seen
  flake IDs, without holding the tier.

### landing-19. Tier names and artifact names at admission

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m8/notes/contracts.md`, 3, paragraph after the
  type sketches; date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 2, 10: gives tier name
  characters, 32-byte bound and tier count; `temper/docs/design/protocol/configuration.md`,
  3: gives the artifact string. Neither specifies shared gate/tier uniqueness or the
  artifact's name bound and slash refusal.
- **Target:** `temper/docs/design/protocol/configuration.md`, 3; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a tier's name is unique among both the gates and tiers
  of its repository. Its artifact name is non-empty, at most 32 bytes, and contains no
  `/`. The repository's tier count remains bounded by `forge.tiers.tiers`.

## temper/docs/plan/reference-m9

- **Read:** plan.md whole; plan.toml, 0 questions; tasks' Answers, 4 files
  (help-pages, repairs-call, repairs-verdict, store-surface), through EOF;
  notes/contracts.md, 1–6; notes/common.md.
- **Kept:** landing-20, landing-21, landing-22, landing-23, landing-24, landing-25, landing-26.
- **Held:** plan/contracts 1–3's stored red/repaired-by relation, live-landing check,
  repository and landing-branch match, control-family restriction, no value repairs
  nothing, newest naming repair in a judged range, run-only clearing and twelve-digit
  repaired-by result (`temper/docs/design/domain/agent.md`, 4.3.1;
  `temper/docs/design/domain/after-landing.md`, 5.3, 6–7, 10;
  `temper/docs/design/protocol/store.md`, 2);
  contract 4's landing/default, named branch, symbolic goal branch, span and zero default
  (`temper/docs/design/domain/after-landing.md`, 2;
  `temper/docs/design/protocol/configuration.md`, 3);
  contract 5's red payload, current subscription, tracked ancestor, one notice per goal,
  waking after-landing red, goal red new-versus-kept, failed jobs of the final failed part,
  no-log marker and reread after restart (`temper/docs/design/domain/after-landing.md`,
  5.4, 7; `temper/docs/design/domain/forge.md`, 7.2, 16);
  contract 6's first-move following, one flight across goals, newest due tip, no receipts,
  no landing verdict, span, green silence, close discards news and goal-branch identity
  in task words (`temper/docs/design/domain/after-landing.md`, 2–3, 5.4, 7, 10);
  plan's triage, repair ownership and learned notes
  (`temper/docs/design/protocol/configuration.md`, 13.9);
  common's worlds, code and in-place format rules
  (`skein/docs/foundation/programming-model.md`, 1;
  `skein/docs/foundation/testing-strategy.md`, 1;
  `temper/docs/design/protocol/store.md`, 4).
- **Superseded:** store-surface's questioned 65,536-goal ceiling
  (now: `temper/crates/temper-protocol-store/schema/store-v11.schema`, TierRow, list 256;
  landing-20 preserves the bound and reason); contract 4's inert goal-tier intermediate
  state (now: `temper/docs/design/domain/after-landing.md`, 3, following implemented);
  repairs-verdict's early fallback rendering
  (now: `temper/docs/design/domain/after-landing.md`, 6, retained run for each tier).
- **Not design:** 12 items: plan logistics (1), contracts 1–6's code-only residues (6),
  common's run instructions (1), and four Answers files' scope, help shortening, accessor,
  merge and fixture instructions (4). Exact Rust refusal sentence construction implements
  the held rule to name the commit and reason, rather than adding another refusal policy.

### landing-20. Goal-tier rows fit the codec envelope

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/tasks/store-surface.md`, Answers,
  2026-11-03; `temper/docs/plan/reference-m9/notes/contracts.md`, 1;
  2026-11-03 for the answer, contract date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 10: bounds goal rows by
  forge holds; `temper/docs/design/protocol/store.md`, 4: checks runtime limits against
  ceilings, but names neither this ceiling nor why it is small.
- **Target:** `temper/docs/design/protocol/store.md`, 4; a paragraph in the section.
- **Text:** Recorded 2026-11-04: a tier's stored goal-branch list has a ceiling of 256,
  sorted by goal number with no duplicate. Its runtime bound is `forge.holds`; a value
  above 256 refuses the start. The ceiling keeps the tier row's worst-case encoding,
  including each goal's failing-test IDs, within the codec's u32 length envelope.

### landing-21. Scheduling does not alter a tier key

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 4, tiers::definition;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 2: key covers the tier's
  definition and ignored paths, without a span exception or goal-following discriminator;
  `temper/docs/design/domain/landing.md`, 6.1: defines gate keys, not these tier additions.
- **Target:** `temper/docs/design/domain/after-landing.md`, 2; a sentence in the section.
- **Text:** Recorded 2026-11-04: a tier's span is excluded from its key. Default landing
  and explicitly named branch definitions keep their byte encodings; following goal
  branches has a distinct marker, so its key cannot equal either form merely because
  their branch text matches.

### landing-22. Fitting red news preserves its identifying lines

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 5, The words and The logs;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 7: retains header, IDs, URL,
  failed-log ends sharing brief_bytes and cuts logs first. It does not specify equal
  shares, whole identifying lines or how an empty read is represented.
- **Target:** `temper/docs/design/domain/after-landing.md`, 7; a paragraph in the section.
- **Text:** Recorded 2026-11-04: red news fits `tasks.message_bytes` in order: header and
  test IDs as whole lines, then the run URL, then failed-job log ends sharing the remaining
  room equally. Only a log's first line may be partial, since cutting a log preserves
  its end. Each failed-job read is bounded by `brief_bytes` divided by the failed-job
  count, using at least one as divisor. A failed or empty read yields `[no log]`.

### landing-23. Comparing unenumerated goal-tier failures

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 6, Finish;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 5.4, 7: news distinguishes
  new from still failing and wakes only for new failures; it does not define comparison
  when one or both runs report many.
- **Target:** `temper/docs/design/domain/after-landing.md`, 5.4; a paragraph in the section.
- **Text:** Recorded 2026-11-04: on a goal branch, many after many is still failing and
  is kept for the goal's next run. Many after an enumerated result is new and wakes it.
  When the new result enumerates tests but the previous result was many, none of those
  IDs is known to be still failing; each is new.

### landing-24. The goal-red tip is abbreviated

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 5, The words;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 7: explicitly says an
  after-landing red names the landing commit whole; the goal-red header uses COMMIT
  without specifying its length. The source distinguishes HEX from HEX12.
- **Target:** `temper/docs/design/domain/after-landing.md`, 7; a sentence in the section.
- **Text:** Recorded 2026-11-04: the commit in `Red on BRANCH: tier NAME run N at COMMIT`
  is the tested tip's first twelve hexadecimal digits; an after-landing red continues
  to name its blamed landing's full commit.

### landing-25. Ties among due goal branches

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 6, Due;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/after-landing.md`, 3: runs the branch due
  longest, but gives no tie order or priority for a branch never run.
- **Target:** `temper/docs/design/domain/after-landing.md`, 3; a sentence in the section.
- **Text:** Recorded 2026-11-04: among due goal branches, one never run sorts before one
  with a previous start; otherwise the oldest start runs first. Equal starts, including
  two branches never run, are ordered by goal number, lowest first.

### landing-26. The repairs parameter

- **Kind:** contract
- **Source:** `temper/docs/plan/reference-m9/notes/contracts.md`, 2, Kept;
  date not stated; decided by not stated.
- **Checked:** `temper/docs/design/domain/agent.md`, 4.3.1: names a full commit and its
  semantic checks; `temper/docs/design/domain/forge.md`, 8.1: spells out several other
  parameters; `temper/docs/design/domain/forge.status.md`, Not yet, 8.1: mentions 12
  without its encoding. No section states how repairs is carried into the procedure.
- **Target:** `temper/docs/design/domain/forge.md`, 8.1; a paragraph in the section.
- **Text:** Recorded 2026-11-04: the red landing a change repairs is carried as bytes
  parameter 12, its merge commit as 40 or 64 lowercase hexadecimal digits. The start
  keeps it with the change; a value outside that text form refuses the start as a
  malformed parameter. Without it the change names no red landing to repair.

## temper/docs/plan/reference-m10

- **Read:** plan.md whole; plan.toml, 1 question (case-workflows, open, no answer);
  tasks' Answers, 1 file (room-wait), through EOF; notes/contracts.md absent;
  notes/common.md whole.
- **Kept:** none.
- **Held:** plan's completion condition, landing body, coding charter's shell and checks,
  triage delegate, designers asking up, full reference profile and hundred-change sizing
  (`temper/docs/design/protocol/configuration.md`, 13.9);
  change task allowance and job-produced changes left open
  (`temper/docs/design/domain/agent.md`, 4.3.1, 10);
  next producer merges the goal branch's newest tip
  (`temper/docs/design/domain/forge.md`, 8.11, Early starts);
  template resources in settings files (`temper/docs/design/client/tools.md`, 5.3);
  room-wait and its Answers' behavior (`jig/docs/design/domain/connectors.md`, 6.2);
  pre-push check decision still awaiting measurements
  (`temper/docs/design/protocol/configuration.md`, 11, and its status's Departs, 13.9);
  intended reference capacities and their smaller built values due to store/carrier bounds
  (`temper/docs/design/protocol/configuration.md`, 13.9, and its status's Departs, 13.4,
  13.9: 461-byte results, one-change calls and twelve delegates);
  case-workflows, including its still-open ownership and runner-budget choice
  (`temper/docs/design/protocol/configuration.status.md`, Not yet, 13.9, line 7: “who
  writes them, and the runner's budget they take, are the user's”, still citing
  reference-m10's case-workflows). As the audit spec directs, this is Held at that line:
  the options to be preserved by citations-out are workflows only in the walk's copy,
  workflows in the repository after a runner budget is set, or a smaller case repository;
  no option is chosen; common's settings-over-stock and user-owned budgets
  (`temper/docs/design/protocol/configuration.md`, 13.9;
  `skein/docs/foundation/testing-strategy.md`, 8.1).
- **Superseded:** plan's listed Not yet entries for profile, texts, task bound, goal-tip
  merge and template resources (now: `temper/docs/design/domain/agent.status.md`,
  `temper/docs/design/domain/forge.status.md`, `temper/docs/design/protocol/configuration.status.md`,
  and `temper/docs/design/client/tools.status.md`, those entries closed or replaced by
  the narrower current gaps); plan's list of remaining case-draft gaps
  (now: `temper/docs/design/draft/reference-case.status.md`, No gap;
  `temper/docs/design/draft/reference-code-production.status.md`, Design lags and
  Design departs; `temper/docs/design/draft/reference-landing.status.md`, No gap).
- **Not design:** 3 items: plan logistics (1), common's remaining file ownership, JSON
  paragraph editing and run instructions (1), and room-wait's Answers' assignment of
  unrelated fixtures and test repairs to the later check (1).
