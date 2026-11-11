# engine: status

2026-11-11. Where jig's code lags or departs from [README.md](README.md).

`executors.md`, section 5's failed-job cause accompanies its retry class through the hub
link and terminal proof into the task's notice and exhausted hold. The
nonempty worker detail and an invalid start's why use the existing reason
bound; no retry allowance changes.

Section 5.1 has no known gap in *Whole across families, through a kill*:
the core world over its directory-backed store on the simulated disk,
killed at each file operation of a decision that makes a task and the
person task it links, of the one that saves a caller's `Person` row and
of the one that saves the task it requested, then again at chosen
operations of the restart and at each of the next decision's, starts to
Open with the rows of a whole decision
(`jig/tests/core`'s `faults::a_kill_at_each_file_operation_of_a_decision_starts_from_a_whole_one`;
every restart operation under
`fuzzy::faults::every_operation_of_a_restart_after_a_torn_decision_can_be_killed_again`).
A snapshot racing commits and several commits in flight are proven in
the store's world alone (`../../protocol/store/README.status.md`, 9).

Section 5.1 has no known gap in *Nothing links ahead of what it names*:
seventeen routes that take a person's number from outside the core
(a connector's batch, as executor, pool funder or requester; a party's
and a procedure's batch, as executor; an agent's person task; a
party's, a compound, an agent's, a proposed, a party's proposed, a
procedure's and an application's amendment; a role's holding; a
proposal's and an escalation's holder; a proposed batch's pool funder)
write and link a held person and refuse one the people child lacks,
each with its route's `Unknown`, before any child changes
(`jig-domain-core/src/admission/batches.rs`, `amendments.rs`'s
`amendment_person`, `proposals.rs`, `runs/delegation.rs`;
`jig/tests/core`'s
`named_people::a_person_named_from_outside_is_held_or_refused_and_the_start_links`,
and the raw batch and amendment refusals in
`tests/admission/named_people.rs`).

`restart.md` has no known gap in a link the start cannot make: the core's
link pass, its projections and its census, and the people, tasks and
accounts children's restore links each refuse as `Failure::Link`, the
first found, with no commit and no sweep, and temper's shell prints it
as `RECORD links missing LINKED` (`jig/tests/core`'s
`start_links::a_store_that_fails_a_link_refuses_the_start_naming_both`;
temper-engine-shell's
`start_failures_name_the_set_key_setting_and_measured_limit`). A
duplicate person's identity refuses as the second `Person` linking the
first. `start/links.rs`'s `link_resource_projects` stays an assertion:
it reads connectors' reports held in memory, which no stored row holds.
A duplicate key, which the store's map cannot hold, also stays an
assertion. temper's forge link still panics
(`temper/docs/design/domain/root.status.md`, 8.1).

`executors.md`, section 2's immutable failed run end retains the invalid-result kind
computed for its attempt. No known gap remains in this field.

`executors.md`, section 2 has no known gap in *Resuming* across a worker's link: the
core hands the link every call committed since the run's last turn,
whole and in commit order (`jig/tests/core`'s
`settled_answers_resume_in_commit_order_and_a_replay_preserves_exact_bytes`),
and the engine's link fits their answers to its room
(`../../protocol/channel/README.status.md`, 8).

`executors.md`, section 2, and `briefs.md`, section 1 have no known gap in *A turn too large for a page*.
`TranscriptLoad::Cut` hands the core a turn's fields and length.
`Core::append_cut_transcript` admits it as `append_transcript` admits a
turn, counting its length and 4 bytes. It keeps `CUT_TURN_TEXT`,
`[a turn too large to load]`, in the tail and sets `Transcript.cut`, so
the claim resumes nothing and counts no failure, try or refusal
(`jig-domain-core`'s `cut_turns_keep_their_length_and_place_without_resuming`
and `a_cut_page_finishes_a_fresh_brief_with_its_tail`). jig's reference
root maps the store's cut page to it (`jig-test-store/src/route.rs`). The
core world's rows, sized by the frame's charge, hold the room's edge:
a turn charging exactly the room resumes whole, and one byte past
starts fresh at once with the marker in its tail. A 4,096-byte turn
between a 100- and a 50-byte one keeps its marker between their texts
(`jig/tests/core`'s `runs::a_window_turn_charging_exactly_the_room_resumes_whole`,
`a_window_turn_just_past_the_room_starts_fresh_with_a_marker` and
`a_large_tool_turn_keeps_its_place_between_other_texts`). No bound of
`append_transcript` refuses a long window: past the resume limit it
starts fresh (`many_one_byte_turns_start_fresh_when_their_lengths_pass_the_limit`,
`an_oversized_resume_turn_keeps_its_text_independently`). The ops
example's root gets transcript pages as rows, from its tests alone. No
store there cuts a turn, and it maps no cut.

`executors.md`, section 3's asking-family routing and run proposal read are present.
`reads/settings.rs` checks settings authority and project scope before
reading, and checks them and the current attempt after loading a stored
row. `reads/vocabulary.rs` restricts retained task proposals to the
reader's project and settings actions.

`executors.md`, section 3's run read as its person is present but for the two gaps
below. `NamedAction::PersonRead` is routed before replay and call
admission (`routing/run/calls.rs`) to `reads/person.rs`, which checks the
asking family, the run's current attempt and a person requester, admits
the reads naming the run's project or a task of it and no other, and
answers `Now::PersonAnswer`, unrecorded. A stored read's completion
checks again the attempt, the requester, the person's role and each
loaded row's project (`reads/rows.rs`, `person_from_store`), and an
application's read goes to its connector with the person and the
project's scope. No frame is read for the person (`README.md`, section 10). The
core's `tests/reads/person.rs` (`admission_names_exactly_one_project_or_a_task_to_check`,
`stored_person_reads_recheck_the_attempt_requester_and_role`,
`stored_person_reads_refuse_foreign_owners_and_linked_rows`,
`retained_pages_prove_their_owner_even_when_empty`) and the core world's
`suite/person_reads.rs` (`a_run_reads_its_project_as_its_person_and_no_other`,
`a_run_reads_as_its_person_only_with_the_asking_family`,
`stored_person_reads_recheck_the_role_and_share_load_limits`,
`person_application_reads_reach_the_connector_as_the_person`) hold it.

`briefs.md`, section 2.1, with `executors.md`, 3's withheld turns of a run's read as its person and
9's `preferences` section, is present by plan `ui-ask-m4` but for the
gaps below. The notes child keeps `Party { party }`, keyed after every
project's scopes, so no project's notes read or deletion meets it;
`notes.party_entries` and `notes.party_bytes`, 12 and 120 in both
profiles, bound its entries and their descriptions, a body refused, and
its index answers whole up to `party_entries` (`jig-domain-notes`'s
`read.rs`). A run whose requester is the person and whose authority
holds the asking family writes it through `Note` and drops an entry
through `NoteDelete`, any other run refused beyond authority naming the
scope, and a run's `note` and `recall` name no party's scope
(`routing/party.rs`); the party alone reads, corrects and deletes it
through a client, naming project 0, whatever their roles
(`reads/party.rs`); the brief's required `preferences` section is present
where the run's charter names the asking family and its authority holds it,
with the person's lines in name order, `N. DESCRIPTION`, or `None kept.`
(`routing/run/preferences.rs`, held by the core world's
`party_brief::an_asking_run_reads_its_persons_preferences_whole`, including
its charter-without-asking row); every turn answered to a party other
than its task's requester, on its own, in an activity page or beside a
line, and every turn of a run's read as its person, is marked withheld
(`reads/withheld.rs`), and the clients' component projects it by
`jig_protocol_transcript::withhold`, a private tool's live input
reaching the requester alone. The core world's
`party_runs::a_run_keeps_its_persons_preferences_and_no_one_elses`,
`party_reads::a_person_alone_reads_and_keeps_their_preferences`,
`party_brief::an_asking_run_reads_its_persons_preferences_whole`,
`withheld::a_turn_is_withheld_from_all_but_its_person` and
`withheld::a_private_live_calls_input_reaches_its_requester_alone`, the
client protocol world's
`a_person_reads_corrects_and_deletes_preferences_through_the_client` and
`a_turns_private_parts_reach_its_person_alone_over_the_wire`, and
`jig-protocol-transcript`'s
`private_parts_are_withheld_with_the_same_admission` hold it.

`settings.md`, sections 9 and 11 have no known gap in the stored-policy size
refusal or a charter's selected effort. Size facts include parts and
cross proposal answers, client wire 19 and people store 14 unchanged,
with `StoredPolicy` appended after the existing members. Charter
resolution retains a supported selected effort; models admitted by
`rest` retain their catalogue defaults. No wire or store version changes.

`restart.md` have no known gap in interrupting agent claims on
hosts in the engine's process at an orderly stop. The core commits
`End::Recheck` before reporting drained; the fleet forgets each attempt
without telling its host. The restart table distinguishes a stopped
in-process worker from a crash and a worker elsewhere: committed turns
and saved work remain, no try or backoff is charged by the interruption,
and the task is due again. Jobs retain their claims for restart recovery.

`executors.md`, sections 1, 2 and 4, and `briefs.md`, section 1 have no known gap in selecting a charter's
parts from a run's reasons, preparing a run again when a reason arrives
before its claim (`routing/run/parts.rs`, `preparation_current`), taking
only offered messages at a turn's fence, parking a finish while its
task's delegates are live, or an undecided entry its run covered waiting
for a change. Later runs
take `later`, reselecting the kept account when its provider changes;
charter documents still reach every run. The core-world per-wake table
covers first and later instructions and choices, a kept question, an
undecided escalation waking the run that has its part, one its part
covered waiting for a new hold, across a restart too, always-offered
words and amendment, no parts, and a question and a due result arriving
between preparation and claim. Tool access to
`later` remains `README.md`, section 10's later work.

`settings.md`, section 2 has no known gap in an out-of-shape refusal's position:
it names the last request item for the failing setting, counted from
0, or 0 when no item names it. The refusal table covers a third item
failing with position 2 and the settings rows left unchanged.

`fleet.md` and `settings.md`, section 8 have no known gap in retaining a host's optional
reply bound and exposing it by a run's live attempt. The fleet returns
none for an unnamed bound, a lost channel, a job or a non-live claim
(`jig-domain-fleet/src/call.rs`, `reply_bound`). The parent supplies its
fallback; the bound adds no hello-admission condition.

`fleet.md` has no known gap in derived next-run refusals: standing hello
facts retain their figures, and admitted hosts too small for the run's
charter version give the first shared shortfall and their best offer,
or no named bound when their shortfalls differ. Account and credential
waits keep precedence; a fitting host leaves the ordinary host wait.
A refused start's class has no known gap: a refused transcript is
transient, any other invalid start permanent, and a busy one is placed
again, no failure.

`settings.md`, sections 1, 7, 8 and 11 have no known implementation gap in
numbered model settings, release-default overlays, resolved charter versions,
withdrawal, catalogue notifications or the owner's newest-revision act.
Unlisted provisions have no known gap, including the saved-window load
and unread-tail retry guard in `executors.md`, 1 and 2, and the prospective templates
query for settings judges in `settings.md`, 5, and the bounded document refusal
facts carried through proposal answers in 17.9. `settings.md`, section 6's atomic
replacement of a run's own pending settings proposal has no known gap.
At `settings.proposals`, replacement is refused with the old set retained;
counting that set as room is deferred in `README.md`, section 10.
`views.md`, section 1's account-driven marks preserve time-expiring offers: a
not-due account timer skips account work and leaves its alarm scheduled;
a due expiry marks and delivers the changed offer.

`settings.md`, section 5 has no known gap in a default its connector no longer
gives: `jig-domain-core/src/settings/application.rs`, `prepare_start`,
erases, with a removal in the release's revision, only a default-following
state its connector names in `SettingsDefaults`' `dropped`, and a reset
to a default no connector reports or names keeps its state, size and
slot (`jig-domain-core/src/tests/settings/application_start.rs`). temper's two restore
stories that met it run (`temper/docs/design/protocol/configuration/README.status.md`).

`fleet.md` has no known gap in a draining worker leaving placement:
`jig-domain-fleet/src/channel.rs`, `draining`, marks its channel for
the rest of its life, beside the busy mark a freed slot lifts, so that
no run or job is chosen there, and an attempt chosen for it is placed
again unrefused (the fleet's tests
`a_draining_worker_takes_nothing_even_once_it_frees_a_slot`,
`a_chosen_attempt_on_a_draining_channel_is_placed_again_unrefused` and
`a_draining_workers_next_channel_is_placed_on`). temper's process story
of a restarted workstation, which would show it on processes, stays
ignored for its fixture (`temper/docs/design/testing/README.status.md`).

`settings.md`, section 12's held worker reads, filtered catalogue changes decided by
the provider's list, atomic account and settings commits, and retention
of running work have no known implementation gap. The mark table covers
failures, repeats, first empty reads and catalogue changes; account times
alone mark no Frame. The omitted-count mark below remains unimplemented.
Later pages, later changes to a known name's figures
and catalogue-number reuse remain `README.md`, section 10's open questions.

`briefs.md`, section 1.1's room as capacity holds at temper's activation: a task
whose transcript window would find no load slot waits, due, spending no
try. A window's later pages keep its first page's slot there
(`temper/docs/design/domain/root.md`, 7).

## Gaps and departures

- **Not yet:** `executors.md`, 3, *A run's read as its person*: the gap remains `executors.status.md`, 3, *A run's read as its person*.
- **Not yet:** `executors.md`, 3, *Of its project alone*: the gap remains `executors.status.md`, 3, *Of its project alone*.

- **Not yet:** `briefs.md`, 2.1, *Its runs' parts are its party's*: the gap remains `briefs.status.md`, 2.1, *Its runs' parts are its party's*.
- **Not yet:** `briefs.md`, 2.1, *Read and kept by its party alone*: the gap remains `briefs.status.md`, 2.1, *Read and kept by its party alone*.
- **Not yet:** `briefs.md`, 2.1, *Read by those runs*: the gap remains `briefs.status.md`, 2.1, *Read by those runs*.

- **Design departs:** 10, from the request `note-parts` of 2026-11-05, decided
  by its plan's cut: a range names the revision it read in `recalled`, the tool's
  member that already says it, where the request has `revision`, so that one fact
  keeps one name and a call without `first` stays as it is.
- **Design departs:** 10, from the request `ui-chat-draft` of 2026-11-05, decided by
  its plan's cut: `core::Held::NotesWritten` keeps its owner, name and revision
  (`jig-domain-core/src/vocabulary.rs`), where the request gives it `lines`, `added`,
  `removed` and `first` too, because nothing makes it: a write's answer goes by the
  call's own part, which has them.
- **Design departs:** `settings.md`, 6, from the request `ui-walk-large`: the proposal-projects departure remains `settings.status.md`, 6.

- **Not yet:** 2: engine slots, hello and placement remain `fleet.status.md` (Not yet, the whole document). 4.2, 5.3: the remaining point-load fences.
   the brief's missing sections remain `briefs.status.md`, 1.  turn-call objects remain `executors.status.md`, 2.  the missing tools remain `executors.status.md`, 3.
  9: the restart-oracle gaps remain `../testing.status.md`, 9.
   the pending-reasons continuation gap remains `settings.status.md`, 6.
   the settings-holder gap remains `settings.status.md`, 6.
   the ignored restore stories remain `settings.status.md`, 10.

- **Not yet:** 5.1: the whole project-deletion commit has no
  established admission price against `journal.writes`, because the
  established allowance covers settings-proposal withdrawals alone,
  excluding the remaining state writes, closing the project and cancellation
  operations. No reason is recorded.

- **Departs:**  the unstarted-core departure remains `restart.status.md` (Departs, the whole document).
  4.2: the optional-answer facts departure remains `jig/docs/design/client/domain/contract/README.status.md`, 3.
  4: the root interface has no text for public `Core::effect_owner` and
  `application_part_request`
  (`jig-domain-core/src/routing/flight.rs`), `call_room`
  (`jig-domain-core/src/runs/calls.rs`), or `next_finishing_deadline` and
  `finishing_due` (`jig-domain-core/src/routing/timers.rs`), which the test
  service uses; no reason is recorded.
  4.1: children's timers arrive through `fire`, but the section has no
  text on how many due children a pass fires.
  `jig-domain-core/src/routing/timers.rs`, `due`, returns one `Timer`, the
  first due child in people, tasks, fleet, brief and accounts order;
  `temper-engine-domain/src/domain.rs`, `fire_routed`, fires it once per
  pass, and `jig-test-service/src/engine.rs`, `fire`, also fires one;
  no reason is recorded.
   the acceptance-capacity question remains `settings.status.md`, 2, 6 and 8.

- **Not yet:** 7: the preparations waiting for notes are bounded by `brief.briefs`, not by live tasks (`briefs.md`, 2.2) [in flight: change 232].
