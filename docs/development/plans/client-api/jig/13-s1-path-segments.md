# Supplement jig-13.s1: a path's segments bounded apart from the rows; the view cuts a long path

Added by ruling 143, from the design answer of 2026-10-16T15:20Z (jig
`7efaeef4d`: `client/domain/contract.md`, 5.3; `client/domain/client.md`,
14 and 16; `client/domain/view.md`, 7 and 10; `protocol/client.md`, 3).
temper 04.4 needs its landing. Read first: jig's `AGENTS.md`,
`docs/development/workflow.md`, the sections above; this plan's README,
"Rules of this plan"; `jig/13-reference-ui.md`, "Rules that bind it".

Scope: `contract.md`, 5.3: "A document carries each whole, with as many
segments as the engine admits in a name (the most of its authority's and
its tasks' bounds on a name's segments): the engine never cuts one, nor
refuses a document for one." `client.md`, 14: "a path's segments: ... a
bound of its own, never a window's rows; the application sets it at
least the most its engine admits in a name". `view.md`, 7: "When the
phrase before them and their segments would pass a node's text, the view
leaves out the first segments, shows in their place the mark `…` ...
then the last segments that fit, the last one always. ... what the view
acts on (a filter by a resource, a draft's pattern) takes the whole
path". jig-view's compose is not raised (`view.md`, 10).
Needs: jig 13.2's landing (landed, jig `c92d4a09d`; the step follows
12.1's and 13.2's moves of the files below).
Files, at jig `e1b483760`'s code, lines refreshed at jig `c92d4a09d` by
ruling 198 (follow each by its name):
- the client's limits: `segments: u32` beside `rows` and `text`, in
  `jig_client_domain::Limits` and in the view's own `Limits`
  (`crates/jig-client-view/src/limits/ui.rs:18`), set by the application
  in code; each literal of either gains `segments`, 4 in jig's (the small
  profile's `authority.segments`, `tasks.hold_segments` and
  `tasks.authority_segments`, `profile/small.rs:47, :67, :120`); the
  literals are the ones the compiler names (the example and test
  domains' `Limits` among them: `infrastructure.rs:106`,
  `observability.rs:91`, `first.rs:73`, `second.rs`'s `Side::new`); list each literal in the commit body;
- the scripted engine (ruling 206): its `Bounds`
  (`testing/jig-client-scripted-engine/src/lib.rs:38`) gains `segments`,
  which its `Default` (`lib.rs:51-59`) sets to its `rows` (32);
  `payload.rs:80-91` bounds a path's or a pattern's list by
  `bounds.segments`, every other list by `bounds.rows` as today; the
  joint admission (`tests/client/src/joint/wire/documents.rs:19-26`)
  passes the client's `segments` beside `rows`; the full literals
  (`tests/client/src/stories.rs:81`,
  `tests/client/src/stories/client_construction.rs:43`,
  `tests/ops/src/client/stories.rs:328`) set it; an engine test that
  pins a path refused by `rows` sets `segments` beside it; list each in
  the commit body;
- link: `crates/jig-client-protocol-link/src/limits.rs:41, :51, :89`
  (`part_resource_path`, `owner_part_key_path_value`,
  `owner_pattern_segments`, each `client.rows` today),
  `translate/documents.rs:312, :330`, `translate/pages.rs:84, :108`,
  `translate/pages_part7.rs:107, :117` take `segments`;
- view (`crates/jig-client-view/src/`): `owner_chip.rs:39-45, :95-97`,
  `jig-client-domain`'s `contract/resource_names.rs:9-11`
  (`resource_path_fits(path, rows, text)`), `owner_amendment.rs:27-29`,
  `owner_historical.rs:487` (`path.capacity() > l.rows`) and `:572-574`,
  `owner_authority.rs:95-103` (`worst_case_words` from `rows`),
  `sections/project/owner.rs:203-205` take `segments`; the cut where
  paths are written (`words/held.rs:118-122`, `owner_authority.rs:117-125`,
  `policy_edit.rs:107-108`, `proposed_task.rs:91-95`,
  `owner_historical.rs:92-94`);
  `limits.rs:74-78`'s node text holds a phrase, the mark and
  `literal_segment::max_bytes(text)`; the cut's bound is that node
  text, `limits::envelope`'s `text_per_node`, derived once in a function
  that `envelope` and the cut both call, and passed to the five writers
  above from the client's `Limits` or `JigBindings`' `text_limit`
  (`binding.rs:52`) that their callers hold, never from skein's
  `Builder` (ruling 200); the node counts derived from
  `rows` take `segments` for a path's part, `rows` for the rest:
  `owner_authority::document_nodes` (`owner_authority.rs:180`, its 9
  callers) and `proposed_task::part_nodes` (`proposed_task.rs:146`, its
  4 callers); the notes filter (`sections/project/notes.rs:36, :78-102`)
  takes the whole path;
- the view's tests that pin a page's capacity or scale `rows` to make a
  path fit (`page_capacity.rs:857, :878`; `owner_amendment.rs:391`;
  `owner_project.rs:296, :460, :982`) set `segments` beside `rows`, and
  their pins are regenerated; refresh any further such test, and list it
  in the commit body;
- `resource_path_fits` callers, which pass `segments`:
  `examples/ops/client-domain/src/infrastructure.rs:955`,
  `examples/ops/client-domain/src/observability.rs:678`,
  `testing/jig-client-test-domain/src/first.rs:953`, `second.rs:44`;
- core: `crates/jig-domain-core/src/documents/task_tree.rs`, `path`
  (`:328-340`, `documents.lines` today) and the approval part
  (`:404-411`), and `project_note.rs`' `fn path` (`:147-148`), by the
  names' bound: the most of the three above, derived where the engine's
  limits are made; `documents.lines` bounds short lists alone. The bound
  is `DocumentLimits`' new field `segments` (`documents.rs:90`), which
  no operator sets: `set_documents`
  (`jig-protocol-config/src/overrides.rs:392`) destructures it as
  `segments: _` and has no name for it, and the engine's limits set it to
  the most of the three once the overrides are applied (one place: name
  it in the report). The worst-case sizers size a path by it, not by
  `lines`: `task_tree.rs:968-1018` (`overview_worst_case`,
  `grant_worst_case`, `authority_worst_case`), `offers.rs:221-223`,
  `project_note.rs:569-577`. Its literals: `small.rs:157` (derived from
  the profile's three), `standard.rs:100` (by its spread),
  `offers/tests.rs:60`, `jig-domain-core/src/tests.rs:127`, and any
  further one the compiler names.
Test shape:
- a task holding a name of `tasks.hold_segments` segments (4 in
  `tests/core/src/world.rs:35`; a held name is bounded by it,
  `jig-domain-tasks/src/holds.rs:174`), in a world whose
  `documents.lines` is 2 (32 in `world.rs:126`), has its task line and
  `task` document served: the test sets `lines` on the world's limits,
  and if the world's `limits()` takes none, the step adds the one
  parameter it needs;
- the link decodes a path of `segments` segments at `rows: 2`, and
  refuses one more as too large;
- a path past a node's text shows `…` and its last segments, and the
  notes filter made from it carries the whole path.
Proof: each test beside its kind (the engine's task line in
`tests/core/tests/task_queries.rs`, its `task` document in
`tests/core/tests/application_documents.rs`; the link's decode in
`crates/jig-client-protocol-link/src/tests/documents.rs`; the view's cut
in `crates/jig-client-view/src/tests/owner_chip.rs`, the notes filter's
whole path beside `sections/project/notes.rs`'s tests); jig's gate, by
merging. Downstream, the temper step that first pins a jig holding this
step sets `segments` in `temper-client-shell-terminal/src/limits.rs:52-59`,
`temper-client-domain/src/tests/fixtures.rs:37-44`,
`temper-client-view/src/tests/fixtures.rs:34-41` and `forge_route.rs:611`
(ruling 198), and lists them in its commit body.
Size: aim for at most 160 production lines; past it, report why. The
bound's threading to the writers counts apart: aim for at most 80 lines
of it; past it, report the count and go on (ruling 200). The scripted
engine's `segments` counts apart too: aim for at most 40 lines; past
it, report the count and go on (ruling 206).
Don't: cut a path in the client's data or the engine's documents; change
what `rows` bounds besides paths; raise `compose`; give the engine a
configured limit for a path; name temper (`git -C <worktree> grep -n -i
-w temper -- crates testing tests examples scripts` prints nothing).
Done when: `git -C ~/src/rust/jig log origin/main --all-match
--grep='^Plan: client-api$' --grep='^Step: jig-13.s1$' --format=%H -1`
prints its landing.
