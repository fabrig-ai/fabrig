# Review before landing

Kind: recipe

Require a person's approval on the forge before a project's changes land, keeping the agent review.

## Goal

Each change into PROJECT's branches lands only after a person holding role
ROLE approves it on the forge, with the agent review kept.

## Settings

Read project PROJECT's landing rule 1 whole; keep every member except
`approvals`. With the stock rule, this covers every branch of each of the
project's repositories and keeps its agent review. For the rule and
freshness, including when to choose `clean`, read temper help landing-rule.

The person must first be known on the forge in that project role: follow
*Human approvals* in temper help configure. A refusal names the project,
role, forge, required count and eligible count. A rule covering the
playground's forge is refused.

Put ROLE's number in place of `1` after `"role"` below. Read the current
value and set its `approvals` to one approval requiring one person:

```
temper ctl DIR settings show landing-rule 1 --project PROJECT --json | jq '{items: [{setting: .setting, set: (.value | .approvals = [{"role": 1, "people": 1, "freshness": "exact"}])}]}' > FILE
```

As a project owner, apply against REVISION, the newest revision printed by
`settings show`; for proposing instead, follow step 3 of temper help configure.

```
temper ctl DIR settings apply FILE --against REVISION --key KEY
```

## Enforced and followed

- **Enforced:** temper merges no change covered by the rule until the
  approval counts. After CI and its gate pass, the change waits at the
  queue's front, keeping its turn, with `step: first`.
- **Followed:** the person reviews the pull request on the forge and
  approves its head.
- **Not:** a push made on the forge outside temper is not stopped by this rule.

## Checking it took

```
temper ctl DIR settings show landing-rule 1 --project PROJECT
```

The value prints `"approvals":[{"role":1,"people":1,"freshness":"exact"}]`,
with your role's number in place of `1`.

On a change past its CI and gate, awaiting that approval:

```
temper ctl DIR change show TASK
```

These lines name PROJECT, ROLE and the pull request's address URL:

```
approval: scope PROJECT rule 1 role ROLE exact required 1; reviews complete; counted 0; remaining 1
next: review the pull request URL for the required role; only a mapped project person counts
```
