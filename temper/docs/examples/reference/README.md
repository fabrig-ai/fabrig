# The reference's settings

`settings.json`, `limits.json`, `room.json`, `landing.json` and `project.json` are the
reference's profile (`../../design/protocol/configuration.md`, 13.9), applied
over the stock as `temper help configure` says. They stay out of the binary:
what a deployment runs until an owner applies them is the stock, and the
Ask temper chat derives the reference from its description
(`../../design/draft/configuring.md`, 8.1). What follows stays out of it too.

Before the first start, merge `limits.json` over the generated deployment document,
retaining its other members and overrides.

## A call's room

The reference takes a payload of 63,740 bytes and output of 48,000 tokens,
the fallback in `../../design/protocol/configuration.md`, 13.8. Its agent's
measured largest turn is 18,431,842 bytes; `limits.json` and `room.json`
carry the bounds derived from that turn and payload.

Set the agent, the engine, every worker and the application before the first start,
retaining the document's other application members and overrides. Run these
in the deployment's configuration directory, with the reference's `room.json`:

```sh
jq --slurpfile room room.json '.profile.declared.tool_payload = $room[0].agent.tool_payload | .endpoints[].limits.output = $room[0].agent.output' agent.json > agent.next.json
mv agent.next.json agent.json
jq --slurpfile room room.json '.engine.limits.overrides["io.output"] = $room[0].worker.limits.overrides["io.output"] | (.workers[] |= (. * $room[0].worker)) | .application = (.application * $room[0].application)' temper.json > temper.next.json
mv temper.next.json temper.json
```

`root.journal.run_bytes` bounds a run's stored policy, a charter's whole
policy among it, and the box's bound of 24,576 bytes refuses a charter of 31 KB, so
the room sets 65,536 bytes. A charter's `turns`, its `time` and the run's
spend must fit the bounds a worker advertises, or no host takes its tasks
(`../../../../jig/crates/jig-domain-fleet/src/needs.rs`, `charter_shortfall`
and `shortfall`).

Apply `settings.json` after that start, in the order below. Its charter models
require the reference agent's output allowance; a stock agent's smaller
allowance cannot host them. No charter's spend, period or time changes.

## Authority for the reference charters

Installing charters alone grants no task authority. Widen the rules ceiling,
policy 1's ceiling and the creator-role authority before starting a reference goal.

Required: delegate 1, message 2, control 4 (`amend`, `cancel`, `release`, `decided`),
decide 8 (`decide`, `decide_escalation`), propose 16, subscribe 32
(`subscribe`, `subscribe_forge`), note 128, recall 256, forge reads 512.
Stock tools 3975 include note 128 and settings 1024; widening gives tools 4031.
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
  | .delegation.kinds |= (. + $required[0].kinds | unique | map(select(. != {"charter": 12})));
.items |= map(
  .set.ceiling |= widen
  | if .setting.kind == "rules" then .
    else (.set.roles[] | select(.number == $role) | .authority) |= widen
    end)
JQ
jq -s '{items: map({setting, set: .value})}' reference-rules.json reference-policy.json > reference-input.json
jq --slurpfile required reference-authority.json --argjson role ROLE -f reference-authority.jq reference-input.json > FILE
```

Expect: widened rules and policy 1 in `FILE`. The stock's
delegation already holds coordinator charter 5, foreman charter 6, change procedure 2,
merge job 3 and person role 0; the widening adds designer charter 7 and takes out
the drafter's charter 12, which the reference's work never starts, keeping each
kind once. Install charters first, offering these families.
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

Once the case's project exists, `project.json` sets its project's ways and its
rule 1, main's, in place of the stock's rule 1 over every branch, whose CI and
review would hold merges into its goal branches. Set its project number as for
the `project-policy` above, then combine it with `FILE`:

```sh
jq --argjson project PROJECT '.items[].setting.project = $project' project.json > reference-project-ways.json
jq -s '{items: [.[].items[]]}' FILE reference-project-ways.json > reference-settings.json
mv reference-settings.json FILE
```

Apply through `temper help configure`, step 3, as owner of both scopes. Tools need their bits at task,
project and deployment; notes also need scope bits. Existing tasks do not gain
widened authority automatically; moving revisions only changes charters.
Instead, amend or allot their authority explicitly, or start anew.

Afterwards, remove `FILE`, `reference-*.json` and `reference-authority.jq`.
