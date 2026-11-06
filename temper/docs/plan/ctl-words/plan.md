# ctl-words: the plan

From `inputs/request.md`: what `temper ctl` tells a person, five small things. Everything below was judged from reading the code on 2026-11-06. No test ran for this cut, so each task reproduces first and says if its cause is wrong.

## Parts and tasks

| Request's item | Part | Task | Needs | Minutes |
|---|---|---|---|---|
| 1, a refused key | `key-words` | `key-words` | | 25 |

Items 2 to 5 are still being cut.

## What was decided

- **1. Only a bad name changes.** The new `Usage::KeyName` covers a name outside the rule and `--key` with no name. A missing required key, a key on a tool that takes none, and `--key` twice keep the words they have: the request doesn't name them.

## Overlaps not avoided

- `key-words` changes one bullet of `temper/docs/help/ctl.md`, where `ctl-waits/deadline-words` changes another, and adds tests to `temper-client-tools/src/tests/`, which `ctl-waits/kept-answer` also touches.
