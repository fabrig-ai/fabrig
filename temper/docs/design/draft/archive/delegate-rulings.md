# Every delegate of a change works to its current spec and rulings

Draft, 2026-11-15, on the user's word. It follows `merge-deletions.md`, since both edit the resolution's words. On staging, on 11-14 and 11-15, two changes could not converge after a person had ruled on them.
- **Case A:** the ruling was to drop a file the original request had asked for. The change was amended to say so, yet the goal branch went back and forth six times between dropping and restoring it.
- **Case B:** the ruling was to keep both of two lines. A repair did keep both, but the review then asked for changes, because the original request said a single line. The next repair was given the same stock text, and the change ran out of questions and was cancelled.

In both cases the ruling was on record, but the change's repairs, reviews and resolutions were never given it.

## 1. One authoritative context for every delegate

- **Today:**
  - `production_words` gives a change's producer its current spec (`temper/crates/temper-engine-domain/src/route/forge_route.rs`).
  - `repair_words` (with `blamed_repair_words`) gives a repair the failure and `REPAIR_PRESERVATION` ("Preserve the requested change and unrelated base content … do not remove content to make a check pass").
  - `review_words` gives a review the pull's identity and title.
  - `resolution_words` gives a resolution its preservation instructions and the merge job's result.
  - None of the last three gives the change's current spec, its amendments' reasons or its answered questions. A repair or resolution sees only the answer to its own question, and a review asks none. `temper/docs/design/domain/forge/change/questions.md`, 1, even says repairs and resolutions are briefed without the change's words. The pull's title and body keep the original request, so that's what they judge against.
  - A landing change made by a goal's closer has no producer at all, so goal 3531's relay to a live producer (`producer-reach.md`) never reaches its delegates.
- **Wanted:**
  - `change_delegate` gives every agent delegate of a change, its producer, repairs, reviews and resolutions alike, one required context:
    - the change's identity and current spec revision;
    - its current spec;
    - each amendment's reason;
    - its answered questions, each with its answer.
  - The pull's title and body, the original request, CI and review remarks stay as supporting evidence, marked as such.
  - The context says it outranks the original request where they differ. "Preserve the requested change" means the change as now specified and ruled on, in the repair, resolution and review words alike.
  - The context is read from the change's retained history (its spec revisions, amendment reasons and answer positions), through the core's bounded reads. It never depends on an inbox message or open relay entry that is gone once taken.
  - It is a required section, kept through brief budgeting (`finish_brief_plan`, `temper/crates/temper-engine-domain/src/assemble.rs`): cut with a marker and a way to read the rest, never dropped silently.
  - The context is fact carried by code, so it holds under custom charters too. The stock implementer and reviewer charters gain a line saying the same.

## 2. A ruling given mid-flight reaches live delegates

- **Today:** goal 3531's relay passes an amendment to a change's live producer (`amend_producer`, `temper/crates/temper-engine-domain/src/route/forge_route/relay.rs`). A live repair or resolution isn't told. A review's verdict stays valid while the head is unchanged, even if the spec it judged has since been amended (`temper/docs/design/domain/forge/change/gates.md`).
- **Wanted:**
  - An amendment of a change reaches its live repairs and resolutions as it reaches its producer.
  - A review verdict given against an older spec revision no longer counts once the spec is amended: the review is asked again at the same head, under the review rules.
  - Neither words nor amendments answer a question, as goal 3531 has it.

## 3. Docs

Revise `temper/docs/design/domain/forge/change/README.md`, `../../domain/forge/change/repairs.md`, `../../domain/forge/change/gates.md` and `../../domain/forge/change/questions.md`, with their status files, to say what each delegate is given and that the current spec and rulings outrank the original request. Remove `../../domain/forge/change/questions.md`'s line that repairs and resolutions are briefed without the change's words.

## 4. Out

- Attributing case A's flips to particular writers: its branch log names no task.
- How a goal's coordinator records a ruling. The stock coordinator already amends the change before it answers.

## 5. Tests

Cheap: focused and fuzzy suites only; walk = false.
- Word builder tests (`temper/crates/temper-engine-domain/src/route/forge_route/tests.rs`): each delegate's words carry the current spec, amendment reasons and answered questions. The context survives a tight byte bound with its marker.
- Stories in `temper/tests/engine/domain/tests/suite/forge_routes/`:
  1. Case A: an amended change that drops a file the request asked for. Successive repairs and a review keep it dropped.
  2. Case B: an answered question ruling both lines kept. A later repair and a review keep both, and the review passes.
  3. A ruling answered to an earlier repair reaches the next one, across a restart.
  4. An amendment during a live repair reaches it. One after a review's verdict, at an unchanged head, asks the review again.
  5. A closer's landing change with no producer: its repair gets the context.
- `producer_amendment.rs` still passes.

Run only the new and affected tests, by name.
