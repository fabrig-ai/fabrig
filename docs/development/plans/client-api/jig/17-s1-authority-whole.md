# Supplement jig-17.s1: an authority crosses whole

Added by ruling 319 (the designer's answer of 2026-10-18T06:20:06Z, to
the question of 2026-10-18T05:58:00Z). Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`; this plan's README, "Rules of this
plan"; `code-organization.md`, 6; jig's `client/domain/contract.md`,
5.3 ("An authority crosses whole"), `client/domain/client.md`, 14 and
16, `client/domain/view.md`, 7.

Scope: the engine never cuts an authority's lists at `documents.lines`,
nor refuses a task document or its Amend offer for one: its bounds are
the authority's, derived, never configured; the link holds an authority
whole within a bound of its own; the view shows part of it.
Needs: nothing.

Shape:
- Engine: `DocumentLimits` (`jig-domain-core/src/documents.rs:88`)
  gains `grants`, `executors` and `implications`, derived where
  `segments` is (`jig-protocol-config/src/overrides.rs:524`;
  `profile/small.rs:158`, beside `name_segments`) and refused as
  overrides as it is (`src/tests/overrides.rs:381`): the most of
  `authority.grants` and `tasks.authority_grants`; the most of
  `authority.executors` and `tasks.executor_kinds`;
  `authority.implications`. `task_tree::authority`
  (`documents/task_tree.rs:338-373`) sizes grants and notes' scopes by
  `grants`, executors by `executors`, and cuts nothing;
  `AuthorityLine.cut` goes, and `jig-protocol-engine-clients`'
  `translate/owner.rs:262` writes the wire's `cut` false. These go:
  `act_authority`'s refusal (`documents.rs:456-459`; its callers,
  `acts/bounds.rs:76`, `:165`, `:168`, take `task_tree::authority`),
  `task_tree.rs:736-745`, `documents/offers.rs:38`'s `!authority.cut &&
  !ceiling.cut` and `:59-62`. `acts/bounds.rs:172-175` bounds
  implications by `implications`; `offers.rs`'s grants check and
  `heap_worst_case` take the new bounds.
- Link: `jig_client_domain::Limits` gains `authority` beside `rows` and
  `segments`, 8 in jig's fixtures; each authority site of
  `jig-client-protocol-link` that takes `client.rows` takes it instead
  (`git grep -n rows -- crates/jig-client-protocol-link/src
  ':!crates/jig-client-protocol-link/src/tests' | grep -E
  'grant|kinds|note_resource|scopes|implication'` prints 17 lines: the
  16 sites and `translate/asks.rs:213`'s signature). A document whose
  authority the wire marks cut is refused, too large.
- View: `jig-client-view/src/owner_authority.rs`'s `body()` shows at
  most `rows` entries of each list, then `AuthorityPhrase::Cut` once,
  its English "More are held than shown"
  (`owner_authority_words.rs:142`). `document_nodes` and
  `worst_case_words` stay.
- Tests, one each: the engine serves a task document to its owner,
  whose role holds `authority.grants` grants, at `documents.lines` 2,
  with Amend `Admission::Bounded` and the role's grants whole; the link
  decodes an authority of `authority` grants at `rows` 2 and refuses
  one more, too large; the view shows `rows` grants and the mark for a
  longer authority.
- Size: aim for at most 150 production lines; past it, the report
  says why. Don't: change the wire's schema (client-v8); change what
  `rows` bounds besides an authority's lists; raise `limits::compose`;
  change creating's charters and procedures (`acts/bounds.rs:47-63`,
  the plan's `later.md`).

Proof: the gate (the three tests above are step tests). The
repository downstream of jig repins to this step in its own step.

## Corrections after the ready check (ruling 328)

The ready check (skein 0be1255e7, jig 7d3becac7, temper 9ea74ad40)
found the Shape off main in these places. They replace the Shape above
where they differ.
- **Engine, a second cut:** `documents/project_note.rs:175-214`
  (`authority_line`) also cuts at `documents.lines` and sets `cut`. It
  sizes grants and notes' scopes by `documents.grants` and executors by
  `documents.executors`, and cuts nothing, as `task_tree::authority`
  does. It joins the Files.
- **Engine, `act_authority` stays** (`documents.rs:457-460`), a thin
  wrapper over `task_tree::authority`, which is `pub(super)`
  (`task_tree.rs:338`); `tests/document_offers.rs:60-100` calls it. Its
  refusal goes, and `acts/bounds.rs` keeps calling it.
- **Engine, the profiles:** `profile/standard.rs:98` spreads
  `..small::DOCUMENTS`, so it sets the three derived fields from its own
  `TASKS` and `AUTHORITY`, or `overrides.rs:80` refuses it.
- **Engine tests, each literal that gains or loses a field:**
  `AuthorityLine { cut }` in `documents/offers/tests.rs:40`,
  `tests/frame_facts.rs:192`, `tests/project_baselines.rs:62`
  (`jig-domain-core`); `jig-protocol-engine-clients/src/tests/fixtures/documents.rs:117`;
  `tests/client/src/joint/wire/{me.rs:118, offers.rs:255-278,
  owner.rs:244, :400, :468-473, :643-646}` (`owner.rs:244` copies
  `translate/owner.rs:314`, which writes `cut: false`);
  `tests/client/src/joint/stories.rs:47`;
  `tests/core/src/referee/offers/facts.rs:54`, `:159`.
  `DocumentLimits` in `tests/store/src/root.rs:106`,
  `jig-protocol-store/src/tests/fixtures.rs:582`,
  `tests/core/src/world.rs:126`, `jig-domain-core/src/tests.rs:126`,
  `documents/offers/tests.rs:59`.
- **Engine tests, the derived values:** `jig-protocol-config/src/tests/domain.rs:195`'s
  Memory total moves from 18_576_824_223 to 18_576_825_183;
  `tests/ops/src/conformance/config.rs:21-44` also sets
  `core_limits.documents.grants` and `.executors`;
  `jig-domain-core/src/tests/frame_facts.rs:145-146` sets
  `limits.documents.executors = 3`, or `task_tree.rs:351` panics;
  `tests/acts/kind_order.rs:16`, `:155`, `:173` take
  `documents.implications`, and `make(..)` at `:160` is now `Ok`;
  `tests/document_offers.rs`' `TooLarge` assertions become the step's
  engine test, the role's grants whole. Rule: a test that raises an
  authority bound after it builds its `Limits` derives the document
  bounds again, as `overrides.rs:524` does.
- **Link, `jig_client_domain::Limits` literals** take `authority: 8`:
  the fixtures of `jig-client`, `jig-client-domain`,
  `jig-client-protocol-link`, `jig-client-test-domain` and
  `jig-client-test-view`; `examples/ops/client-domain/src/tests_support.rs`;
  `jig-client-view/src/tests/fixtures/client.rs:73`;
  `tests/client/src/test.rs:90`; `tests/client/tests/unlinked.rs:27`;
  `tests/client-protocol/src/fixtures.rs:139`.
- **Link, tools stay on `rows`:** `translate/pages.rs:45` and
  `pages_part5.rs:121` (`authority_tools`). The design's lists
  (`contract.md` 5.3, "An authority crosses whole") are grants, a grants
  listing, notes' scopes, executors and implications; an authority's
  tools are a set of at most 64 families (`jig-domain-authority`'s
  `Tools`), which the engine never cuts. The step's tests give their
  authorities at most `rows` tools.
- **View, `rows` is threaded:** `body()` (`owner_authority.rs:146`),
  `required_words` (`:51`) and `document_required_words` (`:89`) take
  none now. `body()`, `render_words`, `document_words`, `render`,
  `document` and the required-words functions take `rows` beside
  `text_limit`, and their callers pass it: 18 calls in 9 files (`git
  grep -n -E 'owner_authority::(render_words|document_words|required_words)\('
  -- crates/jig-client-view/src`), `policy_edit.rs:68` among them, plus
  the wrappers' own. The required words count at most `rows` entries
  and the mark. The threading counts toward the aim; past 150, the
  report says why, and that is no block. The Cut arm is at
  `owner_authority_words.rs:142`, now "Authority document is cut"; its
  English becomes "More are held than shown".
- **Scripted engine** (ruling 333): `jig_client_scripted_engine::Bounds`
  gains `authority`; its admission and pricing of an authority's lists,
  and the joint fixtures' builders, join the Files.
- **Downstream:** skein has no user. temper's breaks at its repin
  (`temper-client-domain/src/limits.rs:58-68`,
  `tests/engine/domain/src/walking.rs:124-126`,
  `tests/engine/domain/tests/forge_routes/paths.rs:86-88`) are
  temper-04.s1's, which names them.

Refresh any further literal or copy of these that the compiler finds,
and list it in the commit body.
