# goal start's model flags

Draft, 2026-11-14. One fault, seen on a deployment: the model and effort a person names when starting a goal don't reach it.

## 1. What happens

- **Seen:** `temper ctl DIR goal start --project P --template T --as-allowed --model M --effort E --sub-agent S:EFFORT TEXT --key K` made a goal whose main model and sub-agent were the template's (`claude-opus-5-5/high`, `gpt-6.1-sol/high`), not the flags' (`claude-sonnet-5-5`, `medium`, `gpt-6-luna:low`). The start answered `done` and said nothing about the flags.
- `temper ctl help goal start` lists all three flags, so a person reading it expects them taken.

## 2. Wanted

- **The flags reach the goal:** `--model` and `--effort` set the goal's main model and effort, and each `--sub-agent MODEL[:EFFORT]` the models its delegates may run, within what the template and the party's allowance permit.
- **What it can't take, it says:** when a template or the party's allowance doesn't let a flag apply, the start refuses before making anything, naming the flag, its value and the bound that refused it. A start never silently uses something other than what it was asked.
- `temper ctl help goal start` says which wins when a flag and the template disagree.

## 3. Out

- Templates' own model choices; how sub-agents are picked for each delegate past the flags.

## 4. Tests

Cheap, focused and fuzzy suites only; walk = false. A focused test starts a goal with each flag and reads them back (`task show`'s main model and sub-agent lines); one where the allowance refuses a flag sees the refusal and no goal made. Run only the new and affected tests by name.
