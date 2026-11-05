+++
id = "neutral-owner"
part = "owner"
kind = "feature"
needs = []
files = [
  "temper/crates/temper/tests/support/relay_managed.rs",
  "temper/crates/temper/tests/support/managed_client.rs",
  "temper/crates/temper/tests/support/live_github.rs",
  "temper/crates/temper/tests/suite/startup.rs",
  "temper/scripts/proof-live-github-managed.py",
  "temper/scripts/proof-live-github-managed.sh",
  "temper/scripts/proof-live-github.sh",
]
effort = "high"
minutes = 75
+++

# A neutral owner in the relay's tests, fabrig-ai on the live probes, the App's maker an input

## Goal

The personal GitHub account leaves temper's code and scripts. The live probes run under `fabrig-ai`
alone, the relay's tests take a neutral owner, and the one thing that must be a User, the account that
makes the managed sitting's App, becomes an input of the run. **No task reaches a real host: never run
a probe in `live` mode, and never run `proof-live-github.sh` with an owner it accepts.**

Why this is not a rename. Connect always makes the App under the person's own account (`Target::Personal`,
`temper/crates/temper-service/src/setup/public.rs:320`; `temper/docs/design/protocol/setup.md`, 17.2,
steps 1 and 3), and an organization only ever installs it. So the App's owner at conversion is a User,
while the repository and the installation belong to `fabrig-ai`. `relay_managed.rs`'s one hard-coded
login does both jobs today. It must split into two:

| what | who | how it is held |
| --- | --- | --- |
| repository owner, installation account (`learn_repositories`, the installation rows, every `/repos/...` path, the fake git bodies) | `fabrig-ai` | a fixed constant, read through the guard; never an input |
| the App's owner at conversion (the `learn:conversion-owner` check) | the person who makes the App | an input of the run, exact equality, no default in `live` |

Acceptance:

- `git grep -il freeekanayaka -- ':!temper/docs/plan'` lists only `temper/docs/design/domain/testing.md`
  and `temper/docs/design/draft/github.md`, which plan `fail-stop-names` rewords.
- `live_github.rs` and `proof-live-github.sh` accept `fabrig-ai` and nothing else.
- `proof-live-github-managed.py`'s `OWNER` is `fabrig-ai`, and so is the relay's guard in `run()`.
- The relay's policy reads owner and repository from its `Guard`: no login or repository literal is left
  in the policy, and its tests run on the neutral `acme` and `alice`.
- A conversion is admitted only from the input's login: a test shows a conversion owned by the guard's
  owner, and one owned by a third login, each refused with `learn:conversion-owner`, and one owned by
  the person admitted. The guard's reach is no wider than it was.
- With `TEMPER_MANAGED_PERSON` unset or malformed, `proof-live-github-managed.sh live` exits 2 before it
  builds anything; the relay's `run()` refuses to start without it.
- In the sitting's fake, the first account is a neutral User and `fabrig-ai` is an `organization`.
- The fake sitting passes in both modes, or the report says why it could not run (Steps, 7).
- The new tests bite; `temper`'s focused tests, clippy and fmt pass.

## Inputs

- `temper/docs/design/domain/testing.md`, 2.7: the live tier, and that these stories are owner-run.
- `temper/docs/design/protocol/setup.md`, 17.2: steps 1 and 3, the App under the person's own account.
- `temper/docs/plan/github-owner/plan.md`: what the survey found.

## Owns and may change

- Owns: the seven files in this spec's header.
- May change `temper/crates/temper/tests/support/relay.rs` for one thing only: `pub fn from_bytes` on
  `managed_transport::Answer`, beside `Request::from_bytes`, if you want it for the conversion test and
  the socket shape reads worse. `Guard` stays as it is: `live_github.rs` builds it too.
- Not yours: any `docs/design/` or `docs/help/`. `testing.md` line 1238 and `draft/github.md` lines
  620 to 629 still carry the name when you finish; `fail-stop-names` takes them. Don't touch skein:
  the fake already does all this needs. Not yours either: the `hetzner` alias in the managed script's
  `ssh -L` mark.
- Nothing else reads what changes: no test reads these scripts, no file outside `startup.rs` names the
  renamed test, and `Shared`, `REPO` and the `TEMPER_MANAGED_*` variables live in these files alone.

Pre-approved options:

- **An open pull on an organization's repository is read through GraphQL's `PULL` document**
  (`skein/crates/skein-forge/src/github/plans/reads/pulls.rs`, `row.organization`), and
  `relay_managed.rs` admits `VIEWER` alone. Making the fake's owner an organization may bring that read
  out. If the fake sitting's journal shows a refused `/graphql`, or the sitting stops there, add `PULL`
  with `relay.rs`'s own check, `variables.owner == guard.owner && variables.name == guard.repository`,
  with a test. It stays on the probe, so it widens nothing. If neither shows, leave the policy alone and
  say so in the report.
- **If the fake sitting can't run in your slot** (the build wrapper won't take it, its target directory
  is refused, a Python module is missing): stop trying, list it under `## Left` with the first error,
  and run the fallback in Steps, 7. This doesn't stop your acceptance.
- **If it is red for a reason your change can't reach:** say so under `## Left`, with the step it failed
  at and the `relay refusal:` line.

## Steps

### 1. `relay_managed.rs`: the policy reads its guard

```rust
/// The sitting's repository: fixed names, never inputs, so no run can point the guard at another
/// account. The fake run is the live sitting's dry run, and shares them.
pub const OWNER: &str = "fabrig-ai";
pub const NAME: &str = "temper-probe-2";

struct Shared {
    // ...as it is, and:
    /// The login that makes the App: Connect makes it under the person's own account
    /// (protocol/setup.md, 17.2), so a conversion's owner is a User, never the guard's owner.
    person: String,
}
impl Shared {
    fn repo(&self) -> String;      // "/repos/<owner>/<repository>", from the guard
    fn full_name(&self) -> String; // "<owner>/<repository>", from the guard
}
impl State {
    fn learn_repositories(&mut self, value: &Value, guard: &Guard) -> Result<(), Refusal>;
}
```

`const REPO` leaves the policy. The literals, by today's line:

| lines | becomes |
| --- | --- |
| 22, 274, 278, 283, 284, 289, 358, 372 | `self.repo()`, computed once per call where that reads better |
| 141 to 143 | `guard.owner`, `guard.repository`, and their joined name |
| 176, 182, 363 | `self.full_name()` |
| 331 | `self.person`; the label `learn:conversion-owner` stays |
| 344 | `self.guard.owner` |
| 507 | `OWNER`, `NAME` and the input, below |

Keep the label `learn:installation-repositories-not-exactly-probe-2`: the script reads it.

In `run()`, beside the other inputs, a missing or empty value is `invalid()`:

```rust
let person = std::env::var("TEMPER_MANAGED_PERSON").map_err(|_| invalid())?;
```

### 2. The relay's tests take a neutral owner

The three test `Guard`s (lines 585, 674, 791) take `owner: "acme"` and `person: "alice"`, the names skein's
fake fixtures use (`skein/crates/skein-fake-forge-github/src/tests/pages_manifest.rs`). The `REPO` the
tests read stays a test-side constant, `/repos/acme/temper-probe-2`.

- **Line 700 frames its request by hand:** with `acme/temper-probe-2` its `Content-Length: 45` must
  become 36.
- **Lines 757 and 763 keep `"type":"User"`:** that row is a conversion's owner. It takes `alice`.
- Lines 816, 819, 846, 853, 860 take `acme`.

New test `a_conversion_is_admitted_from_the_person_alone`, in the shape of
`journal_distinguishes_learn_redirect_guard_and_upstream_refusals`, one fresh `Shared` a case (a relay
admits one conversion): `owner.login` of `acme` and of `other` are refused with `learn:conversion-owner`
and leave `state.app` empty; `alice` sets it. Bite: compare against `self.guard.owner` instead, see the
`acme` case fail.

### 3. `managed_client.rs`

The steps name the sitting's repository through the constants, so the two can't drift:

```rust
use super::relay_managed::{NAME, OWNER};
```

The `Owner` and `Name` fields, and the adopted card's words, `Adopted {OWNER}/{NAME} on forge 1 as owned.`

### 4. `live_github.rs`, line 43

```rust
assert_eq!(owner, "fabrig-ai", "probe owner only");
```

### 5. `startup.rs`, line 1644

One owner now. Rename to `the_live_github_configuration_passes_startup_admission`, drop the loop, and
bind `let owner = "fabrig-ai";`. Not `acme`: `configure_engine` renders the template's `acme` to the
owner, so `acme` would prove nothing.

### 6. The scripts

`proof-live-github.sh`, line 8: the case and the usage line take `fabrig-ai` alone. It stays an argument:
`testing.md`, 2.7 calls it `OWNER`.

`proof-live-github-managed.sh`, right after the port's checks, before anything builds:

```sh
# Connect makes the App under the person's own account (protocol/setup.md, 17.2), so a live
# sitting says whose: the relay admits that login's conversion alone. No default.
if [ "$mode" = live ]; then
    person=${TEMPER_MANAGED_PERSON:-}
    case $person in
        ''|-*|*[!A-Za-z0-9-]*)
            printf '%s\n' 'live needs TEMPER_MANAGED_PERSON, the GitHub login that makes the App' >&2
            exit 2
            ;;
    esac
else
    person=managed-person
fi
```

and pass `"$person"` as the Python script's fifth argument.

`proof-live-github-managed.py`:

- `OWNER = 'fabrig-ai'`.
- `Sitting.__init__(self, mode, port, scratch, evidence, person)`, and `'TEMPER_MANAGED_PERSON': self.person`
  in the relay's environment (line 668).
- `fake()`, line 353. The first account is the fake's person (`temper-fake/src/forge/github.rs`, line 258),
  who makes the App. This keeps the fake's type User, under a neutral name:

  ```python
  'accounts': [{'login': self.person, 'id': 10, 'kind': 'user', 'installation': None},
               {'login': OWNER, 'id': 30, 'kind': 'organization', 'installation': None},
               {'login': 'managed-other', 'id': 20, 'kind': 'user',
                'installation': {'id': 1, 'repositories': 'all'}}],
  ```

- Lines 687 and 697, the marks, name `REPOSITORY`, and line 687 says who connects:
  `'Connect GitHub as ' + self.person + '; create the App; install on ' + REPOSITORY + ' alone; in Ready choose Start here'`.

### 7. Checks

Narrow first, through `heavy`:

```
heavy cargo nextest run -p temper --test live_github_managed
heavy cargo nextest run -p temper --test suite -E 'test(the_live_github_configuration_passes_startup_admission)'
heavy cargo check -p temper --tests
```

The first selects the relay's five tests and your new one. Check the counts. `live_github` has nothing
that runs here: `check` is its proof.

Then the fake sitting, from `temper/`, on loopback alone, one at a time. Its two prompts take blank lines.
It builds in its own target directory, cold the first time:

```
printf '\n\n' | TEMPER_WRAPPER=heavy scripts/proof-live-github-managed.sh fake
TEMPER_WRAPPER=heavy scripts/proof-live-github-managed.sh fake-wide </dev/null
```

Each ends `managed sitting completed`, or for `fake-wide` exits 0 at the wide refusal. On a failure read
the `relay refusal:` line before anything reruns.

Fallback if the sitting can't run: `sh -n` on both scripts, and
`python3 -I -B -c "compile(open('scripts/proof-live-github-managed.py').read(), 'managed', 'exec')"`.

Either way, the two refusals, which exit before any build or host. Each with the variable **unset or
malformed**, never a valid login, and never an accepted owner:

```
env -u TEMPER_MANAGED_PERSON scripts/proof-live-github-managed.sh live </dev/null   # exits 2
scripts/proof-live-github.sh someone-else </dev/null                                # exits 2
```

Then what your text asks once at the end: `cargo fmt`, `python3 -I -B temper/scripts/old_forms.py`,
`heavy cargo clippy -p temper --all-targets -- -D warnings`, and the acceptance's `git grep`. `temper`'s
dependents read none of this: these are test files and scripts.

Commit and push.

## Report

Beyond your text's report:

- whether the fake sitting ran, both modes, and the last mark of each;
- whether the `PULL` option was needed, and what the journal showed;
- the rule each new test broke and what failed;
- the acceptance's `git grep` output.
