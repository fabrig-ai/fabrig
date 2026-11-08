+++
id = "propose"
part = "propose"
kind = "feature"
needs = ["declare", "see"]
files = ["temper/crates/temper-settings-form/src/file.rs", "temper/crates/temper-settings-form/src/settings_file_values.rs", "temper/crates/temper-settings-form/src/settings_file_tests.rs", "temper/crates/temper-engine-smith/src/settings.rs", "temper/crates/temper-engine-smith/src/calls.rs", "temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/lib.rs", "temper/crates/temper-engine-smith/src/tests/settings.rs", "temper/crates/temper-engine-domain/src/boundary.rs", "temper/crates/temper-engine-domain/src/store.rs", "temper/crates/temper-engine-domain/src/domain.rs", "temper/crates/temper-engine-domain/src/lib.rs", "temper/crates/temper-engine-domain/src/route.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/crates/temper-engine-domain/src/route/kinds.rs", "temper/crates/temper-engine-domain/src/route/settings_parts.rs", "temper/crates/temper-service/src/tools.rs", "temper/crates/temper-service/src/route.rs", "temper/tests/engine/domain/src/host_tool.rs", "temper/tests/engine/domain/tests/suite/configuring.rs", "temper/tests/engine/domain/tests/suite/configuring/fixture.rs", "temper/tests/engine/domain/tests/suite/settings.rs", "temper/docs/help/proposal.md", "temper/docs/help/settings.md", "temper/docs/design/domain/agent.status.md", "temper/docs/design/protocol/configuration.status.md"]
effort = "high"
minutes = 330
+++

# ui-ask-m1: `propose_settings` does what charter 3's instructions teach

## Goal

`propose_settings` takes a charter document's repository as the inventory names it,
`OWNER/NAME`, and writes its forge's host; takes a large set in parts by `"more": true`,
held by the engine with no note; refuses, for an asking charter's run, a landing rule
whose role has too few eligible approvers at the call, as acceptance would; and answers,
for either form, where the card waits and whose decision it is. Charter 3's form keeps
working; its tests pass but for the answer's words.

Acceptance:

- **`OWNER/NAME`** (`temper/docs/design/domain/agent.md`, 8.4.2, *A repository as the
  inventory names it*): a host call's inline item whose charter document leaves out
  `forge` is resolved, at decoding, among the task's project's readable repositories;
  the refusal's words exactly; a document that gives `forge` keeps it; a settings file,
  ctl's, and a note's body still require it.
- **Parts** (8.4.2, *A large set in parts*): every rule of that bullet, the part's
  answers and the refusals in 8.4.2's words exactly; `declare`'s decode-time line
  `propose_settings: more is not taken yet; give the whole set in one call.` is gone.
- **Approvals at the call** (8.4.2, *Approvals at the call*): an asking charter's
  proposing call is judged by `route/kinds.rs`' approval check for the chat's project
  alone, refused in 8.4.2's words; a charter 3 run's proposal is judged at acceptance
  alone, as today.
- **The answer** (8.4.2, *Its answer*), for both forms, exactly; other proposals' words
  (`Proposed as ...`) unchanged.
- The rows below pass, each seen failing first for its rule's reason; the configuring
  chat's tests pass as they stand, but for the answer's words in the helpers named below.
- Status (a departure from the implementer's text: the status must say what is built):
  delete the **Not yet** "8.4.2: `propose_settings` takes no parts ..." from
  `temper/docs/design/domain/agent.status.md` and the **Not yet** "13.2: an asking
  charter's proposal is judged for its approvals at acceptance alone ..." from
  `temper/docs/design/protocol/configuration.status.md`, and nothing else.

## Inputs

- `temper/docs/plan/ui-ask-m1/notes/common.md`; `notes/contracts.md`, sections 5 (the
  asking form's schema), 6, 7, 9.5 (who writes `settings.rs` and `host_route.rs` before
  you) and 10 (10.3, this task's shapes; 10.2, its files).
- `temper/docs/design/domain/agent.md`, 8.4.2 (all of it: its words are the spec), 8.1
  (*A proposal's source*, its last sentence; the inventory's repositories),
  4.3 (the asking charter, `asking`).
- `temper/docs/design/protocol/configuration.md`, 13.2 (the approvals judgement at
  acceptance); `temper/docs/design/domain/forge.md`, 8.9.
- The code it follows: `temper-engine-smith/src/settings.rs` (`proposal`,
  `proposal_items`); `temper-settings-form/src/settings_file_values.rs` (`document`) and
  `file.rs` (`proposal_items`, `message`); `temper-engine-domain/src/route/tools.rs`
  (`propose_settings`, `room_refusal`'s use); `route/kinds.rs` (`approval_refusal`,
  `approval_words`, `fake_approval_words`, `typed`); `route/host_route.rs`
  (`settings_repositories`, and `read`, which renders a typed answer with no record);
  `domain.rs` (`models`, `catalogue`); `temper-service/src/tools.rs`
  (`decode_configured`) and `route.rs` (`host_request`).

## Owns and may change

- Owns the files in the header. `route/settings_parts.rs` is new (`mod settings_parts;`
  in `route.rs`).
- **Help** (read by tests; within each page's 12,288 bytes): `temper/docs/help/proposal.md`
  (near line 88) and `temper/docs/help/settings.md` (near line 121) say `propose_settings`
  is charter 3's alone in the stock; since the task `stock`, charter 11 is offered it too,
  in its asking form: correct each sentence, and say in a sentence of `proposal.md` what
  the answer now tells a run (where the card waits, whose decision it is) and that an
  asking charter may give a large set in parts. Name both in the report.
- **The shapes** (`notes/contracts.md`, 10.3): `Tool::ProposeSettings` gains `more:
  bool`; `CallAnswer::Parts(Box<PartsAnswer>)` and `PartsAnswer` in `store.rs`, beside
  `CallAnswer`, exported from `lib.rs`; `Domain::repository_identities`; the decoder's
  repositories. `document` adds `CallAnswer::Document` beside it; keep both.
- **Readers, compiler-forced only:** every `Tool::ProposeSettings { .. }` literal gains
  `more: false` (`temper-engine-smith/src/tests/settings.rs`,
  `temper-engine-domain/src/tests/room_checks.rs`,
  `temper/tests/engine/domain/tests/suite/forge_routes/approval_identity.rs`,
  `temper/tests/client/tests/suite/configuring_chat.rs`); every exhaustive match on
  `CallAnswer` gains the `Parts` arm (`translate.rs`' `keep_connector_answer`,
  `store.rs`' `call_answer_bytes`, `answers.rs`), each edited only for that.
- **`answers.rs`:** `proposed`'s settings words, and one arm, `CallAnswer::Parts`,
  calling `settings.rs`' renderer. Nothing else there.
- **`calls.rs`** (`declare`'s): no `Spec` changes. Default: `call_configured` keeps its
  signature and passes no repositories; a sibling, `call_in(name, tool, input, models,
  catalogue, repositories: &[temper_engine_domain::RepositoryIdentity])`, exported
  beside it, passes them to `settings::proposal`; the service and the world's
  `host_tool.rs` call `call_in`. If one function with the new parameter is plainer, change
  `call_configured` and its callers instead (the forge-route tests'
  `call_configured` calls pass `&[]`), and say so.
- **`host_route.rs` stays untouched:** `Domain::repository_identities` (in `domain.rs`)
  computes what `settings_repositories` gives; leave that function as it is. A typed
  answer with no record goes through `host_route::read` (it takes any `CallAnswer`).
- **Test oracles your words trip,** yours to repair, keeping what they check:
  `temper/tests/engine/domain/tests/suite/configuring/fixture.rs`, `propose` (its
  `strip_suffix(" to the deployment's owners")`), and
  `temper/tests/engine/domain/tests/suite/settings.rs` near line 793 (the same). The
  `strip_prefix("proposed as tN/")` then `split_whitespace` helpers in `configuring.rs`
  keep working, since the key is followed by a space.
- **A change of meaning:** the answer's words for every settings proposal; the decoder
  for every inline `propose_settings`. Run the readers' tests: `heavy cargo nextest run
  -p temper-engine-domain-world --test suite -E 'test(/configuring::/) |
  test(/settings::/) | test(/approval_identity/)'`, `-p temper-settings-form`, `-p
  temper-service --lib`, and `heavy cargo check -p
  temper-client-world --tests` (`configuring_chat.rs`; its stories are not run).
- **No compatibility:** the decoder's input, `Tool::ProposeSettings` and the answer's
  words change; nothing is stored. Say so in the report.
- Pre-approved:
  - **The restart guard's knowledge**, default: at the end of restore (where
    `restart.rs` pushes `Work::ForgeRestored`, or the root's first live step), the root
    keeps the `(task, attempt)` of each run in `domain.core.proofs` whose charter is an
    asking one; a part or the once-refusal removes it. If that point is out of reach,
    keep the root's start wall and refuse once a run whose `TaskRecord::run_since` is
    before it, and say so (`restart.rs` and `route.rs` may take the one line it needs).
  - **The bound:** default jig's `jig_domain_core::settings::capacity`, twice it plus
    one, as `change_shape` counts a change's items; if application items need their own
    count, hold them to the same total and say so.
  - If `declare` left `settings_form::file::proposal_items` refusing the member `more`,
    allow it there.

## Steps

1. **The rows first** (below); run; see them fail.
2. **The decoder.** In `temper-settings-form/src/file.rs`:

   ```rust
   /// A readable repository a host call's charter document may name by OWNER/NAME
   /// (temper/docs/design/domain/agent.md, 8.4.2).
   pub struct Repository<'a> { pub host: &'a [u8], pub owner: &'a [u8], pub name: &'a [u8] }

   /// `proposal_items`, with `forge` left out resolved among `repositories`; `None`
   /// requires `forge`, as a settings file does.
   pub fn proposal_items_in(bytes: &[u8], limits: &Limits, repositories: Option<&[Repository<'_>]>)
       -> Result<List<c::SettingItem>, Box<[u8]>>;
   ```

   `proposal_items` calls it with `None`; `settings_file_values.rs`' `document` takes the
   resolver (its `Decoder` carries it). The refusal is the parser's own form,
   `settings file item I: charter.documents[J].repository: no repository OWNER/NAME in
   this project; its repositories: A/B · C/D` (`; it has none` with none), the names as
   the identities spell them. In `temper-engine-smith/src/settings.rs`, `proposal` takes
   the repositories and `more` (with `note`, refused as `items` is today), and decodes
   `Tool::ProposeSettings { .., more }`; `settings_note` stays as it is.
3. **The root.** `domain.rs`:

   ```rust
   /// The task's project's readable repositories, as its inventory names them
   /// (domain/agent.md, 8.1 and 8.4.2); empty for a task with no project.
   pub fn repository_identities(&self, task: u64) -> Box<[RepositoryIdentity]>;
   ```

   and two fields for `route/settings_parts.rs`: the held parts by task, at most one
   entry per live run (an entry whose run is no longer current, `current_proof`, is
   dropped before another is held), and the restart guard's set.

   `route/settings_parts.rs`:

   ```rust
   /// One run's parts, typed as the decoder gave them (domain/agent.md, 8.4.2).
   pub(crate) struct Held { attempt: u64, against: u32,
       items: List<jig_domain_core::settings::Item>, kinds: List<KindValue> }

   /// A propose_settings call of any form, from `route/tools.rs`' `propose_settings`:
   /// holds a part, or gives the whole set to the proposal's path.
   pub(crate) fn call(domain: &mut Domain, to: ReplyTo, key: CallKey, tool: Tool);
   ```

   In order, for a run whose charter is an asking one (`temper_engine_domain::asking`
   over `domain.core.charter(..)`'s tools, as `workspace` reads it): `more` from any
   other run refused (`NotOffered`); the guard; a duplicate setting against the held
   (`Held`); the bound (`PastBound`); a part holds (`Holding`, with `dropped` when its
   `against` differs) and answers; a proposing call against another revision than the
   held is refused (`Revision`) and drops them; otherwise the held items and kinds,
   then its own, are the set, the held dropped, and the set goes on as today's
   `propose_settings` does it: `room_refusal`, then, for an asking run, the approvals
   (`Approvals`), then `domain.kinds` and `NamedAction::ProposeSettings`. A charter 3
   run's call without `more` takes today's path unchanged. Every `PartsAnswer` is
   rendered through the flight with no record (`host_route::read`).
4. **Approvals,** in `route/kinds.rs`:

   ```rust
   /// Acceptance's approval refusal for a proposal's landing rules, judged for one
   /// project at the call (domain/agent.md, 8.4.2; forge.md, 8.9).
   pub(super) fn call_approval_refusal(domain: &Domain, project: u32, kinds: &[KindValue])
       -> Option<Box<[u8]>>;
   ```

   reusing `approval_refusal`'s body (its loop over projects narrowed to `project`; a
   value as `typed` builds it), `approval_words` and `fake_approval_words` unchanged.
5. **The answers,** in `temper-engine-smith/src/settings.rs` (`PartsAnswer`'s words,
   the setting named as `temper-settings-form`'s `render::setting_text` names it, item
   and items by the count) and `answers.rs` (`proposed`'s settings form: 8.4.2's words,
   the key `tN/K`, or `proposal K` with no task).
6. **The callers:** `temper-service/src/tools.rs`' `decode_configured` takes the
   repositories (its `_task` named), `route.rs`' `host_request` passing
   `&engine.domain.repository_identities(task)`; the world's `host_tool.rs` passes
   `&driver.root.repository_identities(run.task)`.
7. **The tests**, cheap, beside their siblings:
   - `temper-settings-form/src/settings_file_tests.rs`: one table of
     `proposal_items_in`: `forge` left out resolved (host with its port written, the
     identity's spelling for `ORG/Repo`); unknown, with repositories and with none, in
     the exact words; `forge` given kept; `None` requires `forge` as today.
   - `temper-engine-smith/src/tests/settings.rs`:
     `a_settings_proposal_answer_prints_its_ctl_name` to 8.4.2's words (its rows, and a
     row with no task); one table of `PartsAnswer` to its exact text and error flag; the
     decoder's `more` (true, false, absent; with `note` refused).
   - `temper/tests/engine/domain/tests/suite/configuring.rs`, beside the proposal tests,
     a chat from template 5 (the constant `see` added to `configuring/fixture.rs`, or
     yours, beside `CONFIGURING`):
     - `an_asking_chat_names_a_repository_as_the_inventory_does`: the world of
       `a_configuring_chat_discovers_the_forge_host_for_charter_documents`
       (`repository_projects`); a charter document `{"repository":"org/repo",...}`
       with no `forge` proposed and accepted, the charter read back holding
       `forge.example:3847`; `org/other` (project 2's) refused in the exact words,
       naming `org/repo` alone; the store unchanged by the refusal.
     - `an_asking_chat_proposes_a_large_set_in_parts`: one run, steps: a part, a
       duplicate refused, a second part, the proposing call (its review holds the three
       items in order); a proposing call after it proposes its own alone; a part at
       revision 1, a revision 2 made (`world.change`), a part at 2 dropping the first;
       a proposing call naming 1 refused and dropping; a part, then the run ended
       (`world.next`), the next run's proposing call its own alone; a part, the
       engine restarted (as `restart_box_with_second_owner` restarts, or the driver's
       store), the proposing call refused once in the restart's words, then proposing
       its own; and a charter 3 chat's `"more": true` refused. Each answer's words
       exact.
     - `an_asking_chat_is_refused_an_unsatisfiable_approval_at_the_call`:
       `box_approval_world(false)` with the box person linked (as
       `a_box_person_known_by_the_forges_link_approves_with_nothing_added`'s first
       row): role 0 requiring 2 refused at the call with `requires 2 distinct known
       forge people; currently 1 eligible`, nothing proposed; requiring 1 proposed.
       That test, charter 3's, keeps refusing at acceptance.
8. `heavy cargo nextest run -p temper-settings-form`, `-p temper-engine-smith --lib`,
   `-p temper-engine-domain --lib`, the world's tests above; `cargo fmt`; `python3 -I
   -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-settings-form -p
   temper-engine-smith -p temper-engine-domain -p temper-service -p
   temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo check -p
   temper-system-world --tests`, `-p temper-playground --tests` and `-p temper-client-world
   --tests`.
9. Commit and push.

## Report

As the implementer's text says; name the options taken, each compiler-forced arm's file,
and the oracles repaired with what each still checks.
