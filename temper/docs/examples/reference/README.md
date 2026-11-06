# The reference's settings

`settings.json`, `limits.json`, `landing.json` and `project.json` are the
reference's profile (`../../design/protocol/configuration.md`, 13.9), applied
over the stock as `temper help configure` says. They stay out of the binary:
what a deployment runs until an owner applies them is the stock, and the
configuring agent derives the reference from its description
(`../../design/draft/configuring.md`, 8.1). What follows stays out of it too.

## Authority for the reference charters

Installing charters alone grants no task authority. Widen the rules ceiling,
policy 1's ceiling and the creator-role authority before starting a reference goal.

Required: delegate 1, message 2, control 4 (`amend`, `cancel`, `release`, `decided`),
decide 8 (`decide`, `decide_escalation`), propose 16, subscribe 32
(`subscribe`, `subscribe_forge`), note 128, recall 256, forge reads 512.
Stock tools 1671 include note 128 and settings 1024; widening gives tools 1983.
Writes need goal and project note scopes, bits 1 and 2: notes 3.

`ROLE`: creator role (stock owner 0). `PROJECT`: existing project number.
Preserve budgets, grants, other fields and granted bits:

```sh
temper ctl DIR settings show rules --json > reference-rules.json
temper ctl DIR settings show policy 1 --json > reference-policy.json
cat > reference-authority.json <<'JSON'
{"tools": [8, 16, 32, 128, 256], "notes": [1, 2],
 "kinds": [{"charter": 5}, {"charter": 7}, {"charter": 6},
           {"procedure": 2}, {"job": 3}, {"role": 0}]}
JSON
cat > reference-authority.jq <<'JQ'
def grant($bits):
  reduce $bits[] as $bit (.;
    if ((. / $bit | floor) % 2) == 0 then . + $bit else . end);
def widen:
  .tools |= grant($required[0].tools)
  | .notes |= grant($required[0].notes)
  | .delegation.kinds |= (. + $required[0].kinds | unique);
.items |= map(
  .set.ceiling |= widen
  | if .setting.kind == "rules" then .
    else (.set.roles[] | select(.number == $role) | .authority) |= widen
    end)
JQ
jq -s '{items: map({setting, set: .value})}' reference-rules.json reference-policy.json > reference-input.json
jq --slurpfile required reference-authority.json --argjson role ROLE -f reference-authority.jq reference-input.json > FILE
```

Expect: widened rules and policy 1 in `FILE`, retaining executors and adding
coordinator charter 5, designer charter 7, foreman charter 6, change procedure 2,
merge job 3 and person role 0. Install charters first, offering these families.
Size tasks, depth and spend (temper help budget).

Policy 1 affects future projects. Existing projects need project-policy.
If its current policy equals policy 1 just read:

```sh
jq --argjson project PROJECT '{items: [{setting: {kind: "project-policy", project: $project}, set: .value}]}' reference-policy.json > reference-project.json
```

For a customized project, use its saved whole current `project-policy` settings item
in `reference-project.json` as `{"items": [...]}`, preserving all fields.
Widen its ceiling and creator role; combine:

```sh
jq --slurpfile required reference-authority.json --argjson role ROLE -f reference-authority.jq reference-project.json > reference-project-settings.json
jq -s '{items: [.[].items[]]}' FILE reference-project-settings.json > reference-settings.json
mv reference-settings.json FILE
```

Apply through `temper help configure`, step 3, as owner of both scopes. Tools need their bits at task,
project and deployment; notes also need scope bits. Existing tasks do not gain
widened authority automatically; moving revisions only changes charters.
Instead, amend or allot their authority explicitly, or start anew.

Afterwards, remove `FILE`, `reference-*.json` and `reference-authority.jq`.
