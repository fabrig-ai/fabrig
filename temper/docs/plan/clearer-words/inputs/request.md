# Clearer words: three small gaps a person meets in temper's help and ctl

QA's walks found three places where a person reading temper's help or ctl can't tell what to do. Each is small.

## What to change

1. **The merge command's contract in the help.** `temper help configuration`'s `merge_job` line says only "The merge command run from the repository root". A person can't write the command from that. Say what it's given and what it must do: `TEMPER_JOB`, `result.json`, and what its exits 0 and 3 mean, from `temper/docs/design/protocol/tooling.md`, 4.1. `temper help change`'s step `resolving` should point to it.
2. **A cancelled early start in the help.** A producer can end `Cancelled · its early start conflicts with SHA`, and neither `temper help change` nor `temper help task` has the word "early". Say what it means, and that the change goes on with a new producer.
3. **A person task's bound in `task answer`.** `temper ctl task answer` refuses a reason past the person task's contract with a bare `Limit reached`, while its help says TEXT is at most 1048576 bytes. The panel's `reason bytes N` is the only clue. Have the refusal name the bound the task asks for, and have the help say a person task may ask for less.

## How

- Each has a test at the lowest tier that shows it, as the help's and ctl's tests do today.
- Nothing else changes.
