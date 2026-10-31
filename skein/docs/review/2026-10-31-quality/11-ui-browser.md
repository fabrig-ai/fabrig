I read the shared brief and reviewed against it.

Tree root (all paths below are relative to it): `skein (at 3371027)`

## 1. Scope read

**Read closely:**
- skein-ui: `tree.rs`, `builder.rs`, `diff.rs`, `view.rs`, `binding.rs`, `scene.rs`, `outline.rs`, `look.rs`, `limits.rs`.
- skein-ui tests: `tests/fixtures.rs`, `tests/diff.rs`, part of `tests/markdown.rs`, the head of `tests/tree.rs`.
- ui-text: `render.rs` (lines 1–320), the head of `terminal.rs`, `tests/fixtures.rs`.
- ui-web: `mapping.rs`, `svg.rs`, `page.rs` (structure, `apply`, `insert`, `remove`), `tests/fixtures.rs`, the head of `tests/page.rs`.
- fake-person: `lib.rs`, `tree.rs`, `random.rs`. fake-person-browser: `face.rs`.
- skein-browser: `find.rs`, the type section of `browser.rs`.
- lab-browser: the `runtime.rs` pump and spawn, the `hand.rs` Hand.
- The spawn and pump code in the fake-person-browser `harness.rs`, and `tests/browser/tests/support/real.rs`.
- examples/ui: `view/{view,state,limits}.rs`, `shell-web/src/lib.rs`, the view tests and fixtures.

**Skimmed:** `phrases.rs`, `markdown.rs`, the rest of ui-text, `page/web.rs`, ui-bundle, `tests/ui`, the tests in `tests/browser`, lab-browser's `calls`/`config`/`outline`, the server example, and the design docs (by section).

## 2. Scores

1. **Cleanliness 3.** The core is tidy. Against that: generated doc comments, many bare numbers, and lint waivers covering whole files in a crate that is meant to be step code.
2. **Organization 3.** The crate split matches ui.md §2. But `browser.rs` is 2,289 lines and `harness.rs` 1,294, and test support is copied between crates.
3. **Taste 3.** skein-ui reads as written with care. The testing crates drift into bags of flags and re-matching.
4. **Naming 4.** Mostly the design's own words. Slips: `Made`, the singular `Attribute`, `title_written`, `find_boxed`, `types::of`.
5. **Primitives 3.** Node, role, act, patch and scene are the right concepts. But Node and Attribute are parallel structs, Tree has no navigation methods, `takes` leaks a state, and role sets are re-listed at each site.
6. **Elegance 3.** The diff's phases and the builder follow the problem. `Person::next` and `BrowserFace` do not.
7. **Economy 2.** Three copies of the Chromium host loop, role tables written twice, fixtures retyped in many crates.
8. **Complexity 3.** Mostly just enough. `BrowserFace` and the host loops carry state that could be an enum or a single shared host.

## 3. Strengths

- **skein-ui's builder/diff/view.** It is small and bounded, every `expect` says why it holds, and `view.rs:6-9` puts the state table in the header. The diff is a clean match → remove → move → insert → attributes pipeline (`diff.rs:361-379`).
- **Reuse at the boundaries.** `BrowserFace` serves both the harness and the lab's hand. ui-web's `mapping::accessible`/`person` gives the face and the lab outline one role vocabulary. lab-browser reuses `role_name` (`outline.rs:4-6`).
- **ui-web page tests are the model to copy.** One behaviour each, golden HTML, and named ids with their derivation beside them: `const MAIN: NodeId = NodeId(2); // page, main` (`tests/page.rs:12`).
- **Exhaustive role matches where the design wants them** (`mapping.rs` `tag`/`name`/`value`, `role_name`): adding a role is a build error in every shell.

## 4. Findings (ranked by payoff)

**1. Three copies of the Chromium host loop** *(Economy, Organization)*
- `browser::command::Command` → `io::Spawn` is converted three times: `testing/skein-lab-browser/src/runtime.rs:289-317`, `testing/skein-fake-person-browser/src/harness.rs:279-307`, `tests/browser/tests/support/real.rs:92-120`.
- Event-owner extraction is copied verbatim in `runtime.rs:895-910` and `harness.rs:1279-1294`.
- Routing `Down` to the three pipes appears three times: `runtime.rs:441-447`, `harness.rs:1117-1124`, `real.rs:1014`.
- The pending/until fold and the ring reap/deliver/register/enter steps are copied too: `runtime.rs:399-487` vs `harness.rs:1138-1182`.
- `chromium()` is written three times (`tests/ui-web/src/lib.rs:28`, `real.rs:63`, `testing/skein-lab-browser/tests/support/mod.rs:35`) and `fonts()` twice, with different fallback rules.
- **Fix:** one Chromium host in skein-fake-person-browser (both other users already depend on it) providing `spawn(command, root)`, `route(down)`, `chromium()` and `fonts()`. Event-owner extraction belongs on `io::Event` itself.
- **Size:** about 200 lines now, more if the pump is shared.

**2. One `Find`, two matching rules** *(Primitives)*
- The tree face folds case: `testing/skein-fake-person/src/tree.rs:37-45` compares `to_lowercase()` forms.
- The kit keeps case: `testing/skein-browser/src/find.rs:8-19`, browser.md:258, and its test asserts that `"Failing"` does not match `"3 failing"`.
- So a script that passes on the tree can fail in Chromium, although ui.md §17.1 says the browser face "answers the same queries".
- There are also four hand-written substring searches beside `skein_lib::bytes::find`: `find.rs:9`, `browser.rs:1811`, `tree.rs:37`, `tests/browser/tests/qa_look.rs:340`.
- **Fix:** pick one rule (browser.md's: case kept) and use it on both faces through `bytes::find`.
- **Size:** about 30 lines, plus a real consistency gain.

**3. `Node` and `Attribute` are parallel structs** *(Primitives, Economy)*
- `tree.rs:278-306` (Node, 25 fields) restates 17 of the 19 fields of `diff.rs:19-40`.
- Field-by-field code is written out in `Node::vacant`/`Node::new` (`tree.rs:309-367`, which differ only in capacities), `Attribute::of` (`diff.rs:43-65`) and the 19-term `||` comparison (`diff.rs:273-292`).
- Adding one property means editing six places.
- **Fix:** give `Node` a field holding the shell-visible attributes, with tokens (binding, link, key, slot) beside it. The comparison becomes `old.shown != new.shown || …`. Have `vacant()` call `new()`.
- **Size:** about 70 lines in skein-ui.

**4. Role vocabulary written twice, and role sets that escape exhaustiveness** *(Primitives, Economy)*
- `next_role` is a 50-line exhaustive successor function, copied verbatim in `crates/skein-ui-web/src/mapping.rs:449` and `testing/skein-fake-person/src/tree.rs:233`.
- `role_name`/`role_named` (`tree.rs:118-230`) are two hand-kept inverse tables.
- `acted_on` (`builder.rs:569-622`) restates exactly the set of roles that `takes` accepts (`binding.rs:143-203`).
- About 20 role sets are written as `[Role::…].contains(..)` arrays, which a new role passes silently, against ui.md §3's "closed, so an addition breaks the build":
  - landmarks: `builder.rs:263-277`, again as an `||` chain in `outline.rs:121-129`
  - cell roles: `render.rs:262`, `builder.rs:225`
  - focusable roles, two lists that differ: `ui-web page.rs:526` and `mapping.rs:226-238`
  - native-disabled roles, the same list twice in one expression: `mapping.rs:~306-318`
- **Fix:** `Role::ALL` in skein-ui. Named exhaustive predicates there (`landmark`, `field`, `collection`, `cell`) used by every shell. `acted_on` derived from `takes`. One `(Role, &str)` table for person names.
- **Size:** about 150 lines.

**5. `Tree` gives only an index and a size, so each consumer re-derives navigation** *(Primitives, Elegance)*
- The subtree end (`index + size`) is recomputed about 25 times across ui-text `render.rs` (43-66, 87, 107, 163, 262, 325, 339, 435, 578), fake-person `tree.rs:29,57,72,96`, and skein-ui itself (`diff.rs:115` parents, `builder.rs:530` check_depth, `tree.rs:461-482` reverse ancestor scan).
- Two shells consume two different interfaces:
  - ui-text reads the full `Tree`/`Node`, including the private tokens.
  - ui-web reads `Patch`/`Made` and rebuilds its own parent/children mirror (`page.rs:48-57`, `293-327`).
- `takes(role, submits_on_enter, act)` makes every caller re-read `SubmitsOnEnter` (`tree.rs:452`, fake-person `tree.rs:112`, `random.rs:53,79`).
- `TreeFace::allows` (`tree.rs:103-113`) re-implements `Tree::act` (`tree.rs:449-457`).
- **Fix:** `Tree::end(i)`, `Tree::parent(i)` (the list from `diff::parents`, made public), `Tree::index(id)` and `Tree::takes(id, &Act)`.
- **Size:** about 80 lines, and consumers become clearer.

**6. Test fixtures retyped across crates** *(Economy, §10 "a setup written once")*
- The 12-field `Limits` literal is written out about 14 times. It is retyped even inside skein-ui: `tests/slots.rs:5`, `tests/diff.rs:13`, while `tests/tree.rs` and `tests/markdown.rs` already use `..LIMITS`.
- `tests/ui-text/src/lib.rs:1-44` is a verbatim copy of `crates/skein-ui-text/src/tests/fixtures.rs:1-49`. Getting ids requires calling `diff` against an empty tree (fixtures line 37).
- ui-web `tests/page.rs` and `tests/page/events.rs` contain 76 calls of the form `page.apply(&fixtures::patches(&mut view, N, |b| { main(b); …; end(b); }), &mut Vec::new())`.
- `tests/browser/src/tests.rs` opens 20 tests with `World::new(); start(); open();` and writes `Token::new(3)` 21 times. `src/tests/look.rs:6-14` in the same crate already has `PAGE`, `READ` and `world()`.
- **Fix:**
  - `Limits { … , ..LIMITS }` everywhere.
  - One ui test-support crate holding the page/banner/main builder and `shown(build) -> Tree`.
  - A `Shown { page, view, shown }` fixture with `show(build) -> Vec<Op>` in the ui-web tests.
  - Reuse look.rs's helpers in tests.rs.
- **Size:** about 400 lines.

**7. Omnibus tests in skein-ui** *(§10)*
- One test carries three behaviours: `fields_keep_typing_and_binding_changes_are_private_while_drawings_change_whole` (`tests/diff.rs:211`).
- Two behaviours appear only in one test's body: `every_patch_kind_preserves_ids…` also asserts staleness along the way (`diff.rs:153-154`).
- Bodies branch on a row's value: `if shown == 2` and `if shown == 3` (`diff.rs:226,229`), `if shown == 1` (`markdown.rs:84`).
- Bare indices: `get(8)`, `get(2)`, `kinds[0..5]` (`diff.rs:155,186-191,207`).
- **Fix:** split along the behaviours already named, and name the indices.

**8. examples/ui is not yet an exemplar** *(Naming, §10 "named fixtures and sizes")*
- `view/src/view.rs` refers to phrases by bare index: 18 `phrase(state, N)` calls, plus `zip(4..7)`, `zip(22..26)`, `(DISCARD, 20)`, and `Some(16)` in `state.rs:355`.
- The label→phrase mapping is written three times (`view.rs:157-161`, `253-257`, `273-281`).
- `title_written` is the write count for the whole form (it is also used for body, due and minutes: `view.rs:185,195`).
- `State::start(…, _offset, _languages)` has unused parameters (`state.rs:76`).
- The reference shell retypes the view's owner and style tokens (`shell-web/src/lib.rs:24,93,95`; token `2` appears in both `view.rs:183` and `lib.rs:93`).
- The shell converts the view's `Record`/`Exchange`/`StreamDuty` (`state.rs:8-37`) field by field into ui-web's types (`lib.rs:82-86,169-190`). Every application that copies the shell will copy these roughly 40 lines.
- The view tests hand-scan `view.tree().nodes()` at 8 sites with a bare `Queue::with_capacity(1200)` at 14 sites, instead of using `TreeFace` + `Find` and `limits().patches`, although `tests/person.rs` shows how.
- **Fix:**
  - A `Word` enum or named constants, and one function from label to word.
  - Export `OWNER`, the style tokens and `NAMES` from the view.
  - Rename `title_written` to `draft_written`.
  - Move the duty vocabulary to one shared no_std place (design-level; unsure where it best lives).

**9. `BrowserFace` is a bag of 25 Options and flags** *(Complexity, Elegance)*
- The struct is at `face.rs:53-78`. Whether it is pending is derived from four of those fields (`pending`).
- `title`, `find_boxed` and `Face::find` (`face.rs:117`, `248`, `637`) are near-copies that differ only in two flags.
- The `token(); acting = Some(op); requests.push` sequence is written 10 times, and `seen.get(usize::try_from(node.0)…)` 7 times.
- **Fix:** an enum for the one operation in flight (Search, Act, Keep, Command, Choice), plus one `query(find, title, boxes)`.
- **Size:** about 80 lines.

**10. `Person::next` matches on the step again inside its own arms** *(Elegance)*
- `testing/skein-fake-person/src/lib.rs:214-229`, `240-249` and `258-267` each re-match `step` with `unreachable!` arms.
- **Fix:** give `See`/`Gone` and each act step its own arm, calling shared `expect(…)`/`act_on(…)` helpers.
- **Size:** about 25 lines, three `unreachable!` gone.

**11. skein-browser does not follow the rules it is supposed to** *(Cleanliness, Primitives)*
- browser.md §2 says the crate is "step code under the Rust subset". Yet:
  - six modules waive `disallowed_types` for the whole file (`browser.rs:3`, `browser/look.rs:2`, `command.rs:3`, `wire/{frame,encode,decode}.rs`);
  - entities are kept in hand-made `Vec<Option<T>>` slabs (`browser.rs:178-180`) instead of `Slab`/`Id`;
  - lifecycles are booleans (`Person{ready,closing,closed}`, `Page{navigating,opening,closed}`, `browser.rs:35-66`) instead of a state enum;
  - `browser.rs` is 2,289 lines.

**12. Small items**
- **Scene encoding decoded twice.** The byte encoding is decoded in both `scene.rs:419-485` and `svg.rs:83-151`. skein-ui should export the decoder.
- **Splice room repeated.** `if limits.slots > 0 { limits.nodes } else { 0 }` appears five times (`builder.rs:65,72,77`; `view.rs:35,59`). It should be a single `Limits` method, and `Builder::new` should call `lend`.
- **Dead `Option`.** `Print::new` always returns `Some` (`render.rs:15-21`).
- **`while` in step code.** ui-text's renderer is step code but uses `while` (`render.rs:47`, `207`).
- **Doc-comment noise.** Generated comments such as `/// A item node.` and `/// The node is submitsonenter.` (`tree.rs:49,147`).

## 5. Cross-area suspicions (not checked beyond my area)

- `skein-browser/src/wire/decode.rs` (527 lines) is a second JSON reader beside `crates/skein-json`, which the crate already uses for writing.
- Extracting an `io::Event`'s owner recurs at `tests/forge/machine/src/machine.rs:434` and `crates/skein-oauth-accounts/src/component.rs:503`. It probably belongs on `io::Event` in skein-io.
- The ring-loop boilerplate (reap/deliver/register/enter, the pending/until fold) likely recurs in `examples/ui/server/src/main.rs` and `testing/skein-world`. A shared helper in skein-shell or skein-world may be due.
- The full `Limits` literal is probably retyped in other test crates as well.

## 6. Rule costs (short)

- **Exhaustive role matches for true/false answers.** Over 49 roles these cost about 300 lines in my area:
  - `acted_on`: 54 lines
  - `takes`: 60 lines
  - ui-web `base`, `name` and `value`: about 45 each, even though `value` only tells `TextArea` apart
- With `Role::ALL` and a few named predicates kept in skein-ui, the rule would still hold, at roughly a third of the size.