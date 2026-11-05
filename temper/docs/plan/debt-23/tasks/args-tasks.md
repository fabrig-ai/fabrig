+++
id = "args-tasks"
part = "payloads"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-tasks/src/**"]
effort = "medium"
minutes = 90
+++

# jig-domain-tasks: the domain's helpers take their arguments folded

## Goal

Part of jig D016: `jig-domain-tasks`' 16 sites lose their `too_many_arguments` exception, by `notes/arguments.md`'s folds. Their caller is `domain.rs`'s step, which unpacks each `Event` variant and builds the fold there; the variants stay.

The sites: `admission.rs`'s `turn`, `activation`; `run.rs`'s `activation`, `claim`; `control.rs`'s `amend`, `dependency_reached`; `escalation.rs`'s `decide`; `funders.rs`'s `carve`; `moving.rs`'s `apply`; `procedure.rs`'s `stepped`; `proposals.rs`'s `decide_person`, `stalled`, `decide`; `questions.rs`'s `person_question`, `person_answer`; `wake.rs`'s `met`.

Acceptance:

- `rg -o 'too_many_arguments' jig/crates/jig-domain-tasks/src -g '!**/tests/**' -g '!**/tests.rs' | wc -l` reads 0.
- `heavy cargo nextest run -p jig-domain-tasks --lib` and `-p jig-tasks-world` pass, with no assertion changed.
- About +50 net lines.

## Inputs

- `temper/docs/plan/debt-23/notes/common.md`
- `temper/docs/plan/debt-23/notes/arguments.md`
- `jig/docs/development/debt/D016-positional-payloads.md`

## Owns and may change

- **Owns:** the crate's `src/`, tests only in a call's spelling.
- **`closing.rs`** is `settings-name`'s in this plan, and no site here needs it.
- **`funders.rs`'s `carve`:** its reason holds the word "actual", counted in `scripts/old-forms.txt`. If the reason goes and the script reports a fall, `--write` lowers it.
- **Stays:** `moving::Moving<'a>` and `waits::View<'a>` as they are: they depart from the owned rule, and aren't this task's.

## Steps

1. `Ended` and `ProposalDeciding`, which pay two sites each; then `Turned`, `Claiming`, `Deciding`, `Answering`; then the pairs.
2. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-tasks --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; each site's fold where it departs from the note's table.
