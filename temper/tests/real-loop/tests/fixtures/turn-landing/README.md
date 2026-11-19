# Turn landing's scripts

`scripts.json` is the box's `--scripts` object, not a whole model document.
The real-loop red story decodes it with `Document::decode_with_scripts` and uses
that domain unchanged. The setup catalogue copies these bytes.

## Goal cues

- `@turn-a`: make `turn-a.txt` contain `First turn member\n`.
- `@turn-b`: make `turn-b.txt` contain `Second turn member\n`.
- `@turn-red`: in place of A, make `.fake-forge/gate` contain `fail\n`.

Each goal's words include `Person: P; Repository: R;`. P is the signed-in
party number from `ctl me`. R is the forge repository's resource number:
`(forge number << 32) | repository id`, using the connected forge's number
and the repository id from its repository listing. No recorded task, pull,
commit, port or checkout path appears in the scripts. Takes bind P and R
from those words and each review's pull and head from its brief.

The project uses `turn`, `batch` 2, with blocking `gate.yml`, the fake
Forgejo's declared `.fake-forge/gate` cue, and both manual-merge settings on.
Its agent-review landing rule uses **clean** freshness: disjoint clean merges
carry their reviews to their prefixes. The real-loop world has one worker
with two slots. Charter 1 makes changes and charter 4 reviews them.

The agent review gate belongs to the **stored project landing rule (kind 2)
number 1**, whose `gates` member includes
`{"number":1,"blocking":true,"freshness":"clean"}`. Stock `review_rule()`
already sets that value. The setup keeps the stock rule, or applies that
value through settings. It puts no `rules` or review-freshness member in
the start's document: it refuses project `rules`. The world fixture's
typed defaults set
`world_settings.landing.projects[0].rules[0].gates[0].freshness` to `"clean"`;
`finish_configuration` separates those world defaults before engine startup.

## The person's latch

Commands below omit only `temper ctl DIR` and optional replay keys. Use a new
key for every write when keys are required. Goal budgets and deadlines are
chosen within the deployment's offered bounds; the story uses 500 and
`2030-01-01T00:00Z`. Substitute the fresh project's name or number for PROJECT.

1. `goal start --project PROJECT --charter 1 --budget 5 --deadline 2030-01-01T00:00Z --key first '@turn-a First turn member\nPerson: P; Repository: R;'`
   (Give an actual newline before `Person`, not a literal backslash-n.)
   For red, replace `@turn-a First turn member` with `@turn-red Failing turn member`.
2. `task show A --until phase:idle --json`, with A from goal start.
   `task show A --tree --json` names its person-task latch L and change C1.
3. `change show C1 --until pull --json`; `task show C1 --tree --json`
   names its reviewer V1 once that review exists.
4. `task show L --json` supplies L's person-task revision LR.
5. `task show C1 --json` supplies C1's spec revision S1.
   `task amend C1 --rev S1 --dependency L 'Wait for the batch latch' --key keep-a`
6. `goal start --project PROJECT --charter 1 --budget 5 --deadline 2030-01-01T00:00Z --key second '@turn-b Second turn member\nPerson: P; Repository: R;'`
7. `task show B --until phase:idle --json`; `task show B --tree --json`
   names change C2. `change show C2 --until pull --json` and
   `task show C2 --tree --json` name reviewer V2. Read C2's spec revision S2.
8. `task amend C2 --rev S2 --dependency L 'Wait for the batch latch' --key keep-b`
9. `task message V1 'Approve now' --key approve-a`; `task show V1 --until ended --json`.
   `change show C1 --json` must show its approved review and `gating`.
10. `task message V2 'Approve now' --key approve-b`; `task show V2 --until ended --json`.
    `change show C2 --json` must show its approved review and `gating`.
11. `task show C1 --json`; `task show C2 --json`: both depend on L.
    `landing list --project PROJECT --ended --json`: no landing formed yet.
12. **Release once:** `task answer L --rev LR 'Ready together' --key release`.

The review scripts read the named pull/head, then wait for `Approve now`.
Their input rules wait again on unrelated input, including the engine's
nothing-more notice; they use no fake-model hold or referee release.
Both reviewed changes are then kept by one person task. Answering L is one
durable completion making them eligible together, not two timed releases.

The green landing on these scripts belongs to the setup's walk and the
engine domain world's story: both changes land in one landing of two, the
gate green at the top. The real-loop red story reads the first failed gate receipt in landing history, captures
the dispatched run/jobs/tree through the forge API before any rerun replaces
that attempt, then cancels the goals and landing for teardown. It checks that
the red member is not an ancestor of main. No outcome of narrowing or repair
is promised.
