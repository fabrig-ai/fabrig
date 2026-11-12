# A tester that cannot land

Kind: recipe

Run tests under a tester's own charter and report findings for another task to fix.

## Goal

An agent runs and writes tests under its own charter and reports what it finds, landing nothing; another task's change lands any fix.

## Settings

Choose free charter and template numbers; these examples use 3 for both.
Read temper help charter and temper help template for their fields.
Copy stock coding charter 2, keeping its `tools` 514 without the delegate bit (1),
enabling its shell and replacing its instructions:

```
temper ctl DIR settings show charter 2 --json | jq '{items: [{setting: {kind: "charter", number: 3}, set: (.value | .number = 3 | .shell = true | .instructions = "Run and write the tests you are asked for. Report each test, its path, its content and its result. Fix nothing.\n")}]}' > FILE
```

Apply FILE as in temper help configure, step 3.
Then write FILE with this template:

```json
{"items":[{"setting":{"kind":"template","number":3},"set":{
 "number":3,"name":{"bytes":"tester","cut":false},
 "executor":{"charter":3},"contract":{"report":{"words":16000}},
 "resources":[],"shallower":1,"budget":{"as_allowed":{"cap":null}},
 "deadline":"none","tracked":1,
 "flags":{"conversation":false,"listed":true,"retention":"kept","ended_by":"result"},
 "label":2,"grace":null,"llm":null
}}]}
```

Apply it the same way, against the newest revision.
Widen the starter's authority for charter 3 within the ceilings as configure's
*Authority for new charters* says; a template grants no authority.
Start it with the tests to run in TEXT (temper help ctl-goal):

```
temper ctl DIR goal start --project PROJECT --template 3 TEXT --key KEY
```

## Enforced and followed

- **Enforced:** without the delegate family, the charter offers no `change`,
  `goal` or `delegate`. The report template holds no goal branch, so its runs
  have no `deliver`, `write`, `edit` or `remove`; tests are written through the shell.
- **Enforced:** `effect_forge` can write an issue, a comment or a status, not
  a landing. It and `propose` remain offered; calls need the task's authority.
  What the tester proposes waits for a party who may decide it
  (temper help proposal).
- **Followed:** the instructions to test, report each test's path and content,
  and fix nothing; whoever reads its result starts another task's change.
- **Not:** temper does not confine the shell's commands on the worker's host;
  they run with the worker's permissions. A report contract alone does not
  stop a push.

## Checking it took

```
temper ctl DIR settings show charter 3
```

The first line is `charter 3, revision REVISION`; the whole value follows.
Check `tools` is 514, with 514 & 1 = 0.

```
temper ctl DIR task show TASK --json
```

Check `result_contract` is `{"report":{"words":16000}}`, `resources` is empty
(no goal branch), and read the task's `authority`.
