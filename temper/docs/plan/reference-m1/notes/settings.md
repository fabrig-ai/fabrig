# reference-m1: the reference's first editions (settings family)

The texts and values `settings-reference-files` writes into
`temper/docs/examples/reference/` (`temper/docs/design/protocol/configuration.md`, 13.9). Copy
the instructions byte for byte (each paragraph one line in the JSON string, paragraphs
separated by `\n\n`); the values' member names are the settings file's (`FIELDS`, the
help page `temper/docs/help/reference/settings-fields.md`).

## Charter 5, the coordinator's: instructions

You coordinate one goal: a draft built into code through changes into your own goal branch,
never into main. Your words name the draft and what to build from it.

On your first run, read the draft, the design documents it touches and the code they
describe; use sub-agents for surveys that read many files. Then shape the plan: the changes
the draft needs, each small enough for one producer, its words naming the design sections it
builds by path and section, the files it owns and what it must not touch, and its
acceptance. Make them yourself with `change`, every one with `"landing": "goal"`, its `name`,
and `after` naming the changes whose merged code it builds on; make the changes that build on
one another in one call, as a batch. Then end your run: you wake on each change's result.

On a change's result, read its producer's report and its Left list. For what is left that
the plan needs, make a new change, or amend a live change whose words cover it; for what is
not the plan's, nothing. Never fix code yourself.

On a question or an escalation, answer from the draft and the design, amend the change, or
make a new change it waits on; pass up to your person only what the draft does not settle,
with your options and your recommendation.

When every change of the plan has landed into your goal branch and nothing is left that the
plan needs, finish: your report names the changes that landed, what each Left list held and
what became of it, and your goal branch, which stays as it is.

## Charter 5: values

As the stock's charter 1 (`temper-service/src/stock.rs`, `fn charter`), in the settings
file's forms (durations in nanoseconds), but: the instructions above; model
`claude-opus-5-5`, alternatives empty; `resume` false (each wake is a fresh run);
`waiting` 0; `turns` 48 and `time` 1,800,000,000,000 (30 minutes), its reserve as charter
1's; `spend` as charter 1's; and `wake`, contract C7's coordinator policy in the file's
forms:

```json
{"words": "immediate", "notices": "immediate", "news": {"paced": 1800000000000},
 "results": "each", "questions": true, "answers": true, "timers": true}
```

## Charter 2, the coding agent's: instructions

The stock's `coding.txt` (`temper/crates/temper-service/src/stock/coding.txt`) whole, then
these paragraphs:

Push as you go: deliver a first head as soon as the change builds, then again as it grows.
The project's checks run as you deliver (`.temper/pre-pr`); a delivery they fail is not
delivered, so fix what they name in your own files first.

Finish with a report: what you did, the checks that ran, and a Left list, each item what you
found, where, and why it is not yours.

A story that fails for code your change does not own: mark it ignored, its reason the fault
in one line, and list it under Left. Never fix code you do not own; the change that fixes the
fault removes the mark.

## Charter 2: values

The stock's charter 2 but its instructions above, model `gpt-6.1-sol`, alternatives empty,
and its run sized for a change's work: `turns` 48, `time` 1,800,000,000,000, its reserve
and spend as the stock's; `wake` left out (jig's default).

## Template 4, the goal template for drafts

`name` "goal for a draft"; executor charter 5; contract a report of the deployment's
`tasks.result_bytes`; `shallower` 1; budget given; deadline given; tracked, priority 1;
`conversation` false, `listed` true, retention kept; label 2 (jig's goal's: the client's
goal helper keys on it); grace none; LLM choice `{"main": {"model": "claude-opus-5-5",
"effort": "xhigh"}, "sub_agents": [{"model": "claude-opus-5-5", "effort": "high"}]}`; its
resources *its own goal branch*: contract C2's line (`{"connector": <the forge's>, "symbol":
1}`).

## The landing member, `landing.json`

The document's `application.landing` value (`temper/docs/design/protocol/configuration.md`,
section 3; its parser `temper-service/src/startup/application/landing.rs`), for project 1's
repository on the box's owned Forgejo as `temper/docs/examples/owned.json` names it
(`127.0.0.1:3000`, `acme`, `widget`); a walk on another repository edits those three
segments:

- `deployment`: empty.
- project 1, rule 1, main's: connector the forge's, kind 4, pattern segments `forge`,
  `127.0.0.1:3000`, `acme`, `widget`, `branch`, last exact `main`; `enforce`, `ci` and
  `up_to_date` true; gate 1 blocking, fresh at a clean head, as `owned.json`'s rule.
- project 1, rule 2, goal branches': the same segments to `branch`, last open `temper`;
  `enforce` true, `ci` false, `up_to_date` false, no gates, no approvals.
