+++
id = "repair-cause"
part = "repair-cause"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes/repair_cause.rs",
  "temper/docs/help/change.md",
  "temper/docs/design/domain/forge.status.md",
]
effort = "high"
minutes = 45
+++

# A clean update's red names its check

## Goal

The request's item 3. A repair's brief had conflict words and no cause, which cost one failed `read_job`, and which path gave them was unknown.

From reading, the path is `Repair::Semantic`. `temper-engine-domain-forge-change/src/domain.rs` picks it for any CI red on a change with a clean update behind it (`if change.clean.is_empty() { Repair::Ci } else { Repair::Semantic }`), so a red the base had no part in takes it too. Its words in `route/forge_route.rs`, `repair_words`, are:

```
Repair the base's break at abcdef012345: main
Repair the base update's semantic conflict. Preserve ...
```

They name the base and no check, while `Repair::Ci` names its failed check or says `no failed check named`. A conflict's own words are right already: a resolution's task opens `Resolve the merge conflict.` and its brief `Resolve the conflict from change head`.

Reproduce that first, and say if it's wrong: if your story shows another path giving conflict words with no cause, fix that one too and say so.

Acceptance:

- A clean update's red reads exactly as follows, with the same cut rules as a CI repair's:

  ```
  Repair CI at abcdef012345: <the failed check's context>
  CI failed after a clean update from the base, and no merge conflict waits. Repair what the failed check says. Preserve ...
  ```

  `no failed check named` takes the name's place where the head's failed checks aren't known.
- The repair's brief carries the failed check's name and log tail, then what landed in the base. `assemble.rs` already asks for the CI section on this path: a story pins it. If the story finds it missing, fix that and say so.
- `Repair::Ci` and `Repair::Gate` read as they do. A resolution's words and brief still say conflict, and a story or test pins each.
- `Repair::Semantic` stays as a state, with its name, its stored and wire forms, and its route through the change machine. Only its words change.
- `temper/docs/help/change.md` says the same.
- The `8.4:` entry this plan added is gone from `temper/docs/design/domain/forge.status.md`.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.4, its first bullet, **A repair**, to the sentence beginning `A clean update's red is CI's`; and section 10, **A semantic conflict**.
- `temper/docs/plan/landing-fixes-2/notes/common.md`.

## Owns and may change

Owns the files in the header. `repair_cause.rs` is new.

Reader and neighbour files:

- `temper/tests/engine/domain/tests/suite/forge_routes.rs`: the `mod repair_cause;` line.
- `temper/crates/temper-engine-domain-forge/src/tests/brief.rs`: one test, if nothing pins a resolution's section opening `Resolve the conflict from change head`.
- A test that pinned the old words: `rg "base's break|semantic conflict\." temper` finds them. On the trunk that's `forge_route/tests.rs`, `a_base_break_repairs_words_open_with_its_head_and_base`, which you rename and rewrite.

Stay as they are:

- `temper-engine-domain-forge/src/result.rs`, `repaired`, `the base broke it`.
- `temper-client-domain/src/forge/words.rs`, `Repairing: a semantic conflict with ...`, which is temper/ui's.
- The design's name "semantic conflict".

Report each under `## Left`, with whether it misled in your story.

Other drafts: `saved-close` changes other functions of `route/forge_route.rs`. `resume-told/answer-words` changes `temper/docs/help/change.md`; keep your edit to the one line. `resume-told/clean-merge` and `saved-close` delete their own lines from `forge.status.md`.

Departs from the implementer's text:

- `temper/docs/help/change.md` is yours for this change: in its repair bullet, drop `` `Repair the base's break at HEAD: BASE` `` from the list of openings, and say a clean update's red opens as CI's. Lose no other fact.
- Delete from `temper/docs/design/domain/forge.status.md` the `Not yet` entry beginning `8.4: a clean update's red opens`, with its continuation lines.

Options, each pre-approved:

- `repair_opening`'s `Ci` and `Semantic` arms become one. `unnamed_ci` then asks of both, and may be renamed.
- `previous` is the second line, and the room check counts it. If the longer words make the row at the small bound in `forge_route/tests.rs` (near line 1591) refuse where it didn't, follow that test's rule for a CI repair, and say so.
- The base's name leaves the words' first line. The brief's pull section still has it. If your story shows that the task's cut read needs it, say so in your report rather than adding it.

## Steps

1. **Reproduce.** In `forge_routes/repair_cause.rs`, modelled on the nearest story in `forge_routes/` that lands a change after an update (`light_rules.rs`, `goal_changes.rs`):
   - `a_clean_updates_red_names_its_check_and_no_conflict`: a change takes a clean update, CI fails with a named check at the new head, and the repair's `spec.words` and brief read as the acceptance says. It fails today on `Repair the base's break`.
   - `a_clean_updates_red_with_no_check_read_says_so`: the same, with no check read at that head.
   - `a_resolutions_words_say_conflict`: an update that conflicts still opens `Resolve the merge conflict.`, if `merge_job.rs` or `asks_no_one.rs` doesn't already pin what you need. Name the test that does, if one does.
2. **Words.** In `forge_route/tests.rs`, rewrite the old test, and add a `Repair::Semantic` row beside the CI rows at lines 1584 to 1606.
3. **Fix** `repair_opening` and `repair_words`.
4. Break the rule once: bring back the old opening, see the first story fail. Restore.
5. **Help.** `temper/docs/help/**` is read by tests, so run `heavy cargo nextest run -p temper-docs` and `heavy cargo nextest run -p temper --lib -E 'test(help)'`.
6. **A change of meaning runs its readers' tests,** since a run reads these words: `heavy cargo nextest run -p temper-engine-domain -p temper-engine-domain-world -p temper-engine-forge-world`.
7. `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`, `heavy cargo clippy -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D warnings`. Commit and push.

## Report

The implementer's report, plus: whether `Repair::Semantic` was the path, and how the story shows it; whether the brief carried the check before your fix; and the words as a run reads them, once, in full.
