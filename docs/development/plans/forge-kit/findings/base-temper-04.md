# Base runs: temper 04, setup on skein's kit

2026-10-19. Code: temper dd12b8ed904edced9e65af9fa62cc769c5a09905
(the code of 27995fb75); skein 25d2551. Only this findings file and
`temper/04-setup.md` are deliverables. Scratch changes are removed below.

## Discovery and corrections to the survey

Graph calls used project `srv-data-git-runner-src-temper`: search_graph
for checked, reconcile_settings and the setup types, then search_code
for Reconcile under crates/temper-forge-setup. All returned zero results.
Targeted rg and source reads in the supplied base verified the names.

- The source machine is 931 lines in four files. Limits/Unfit/check and
  login/email are additional deletions; small/apply and override refusals stay.
- Exactly 13 tests in src/tests/reconcile.rs move. The survey's 29 included
  members/read/steps, which test temper's policy and remain; trace-setup
  sections 1 and 4 already correct this.
- check::checked receives only a boolean for webhooks at the base. The
  changed signature receives the optional listener so the step code can
  check the complete Given. check::members keeps its boolean.
- crates/temper/src/{lib,sign_in,box_command}.rs all reach setup::check.
  temper-box uses temper's Step only: no new signature or direct kit dependency.
- Shell Settings holds Entry, listener, forge and three timing values.
  The draft replaces them with Given, plus setup_index for refusal paths.
  Its support::settings fixture has zero orgs at the base; use acme so a
  later repin that includes skein 04.2 also admits this host fixture.
- Setup reader account/strings use login/email, and owned person's email
  uses account too. All switch to the kit's public validators.
- Reconcile's world remains in temper until skein 07.3/temper 07. Its
  Item-to-File mapping uses the owner's place::item, then its existing store.
- No new cross-session dependency: the tools plan's first repin and
  temper 01.2 already precede skein 04.1 under the README table.

## Focused inventory and runs

The JSON listing's selected tests are counted by package. The same command
also records every moved name below; no filter is used to run the tests.

```text
$ cargo nextest list -p temper-forge-setup -p temper-forge-setup-shell -p temper-box -p temper -p temper-forge-setup-reconcile-world --message-format json
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo]
test-count: 236
temper-forge-setup: 98
temper-forge-setup-shell: 24
temper-box: 35
temper: 58
temper-forge-setup-reconcile-world: 21
exit 0; task 734, ran 1s
log: /srv/data/git/runner/src/tmp/logs/20261019T013132-734.log
```

Moved names, all with prefix `tests::reconcile::`, from that JSON listing
and the PASS lines of task 735:

| Name | Base time |
|---|---|
| a_bot_that_is_a_site_admin_is_refused | 0.004 s |
| a_bot_with_the_marker_is_kept_and_nothing_is_written | 0.004 s |
| a_forge_that_stops_answering_stops_the_run_at_its_step | 0.004 s |
| a_draw_of_another_length_stops_before_creating_the_bot | 0.004 s |
| a_lower_hook_met_later_deletes_the_higher_one_kept | 0.004 s |
| a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login | 0.004 s |
| a_missing_admin_token_stops_before_any_request | 0.004 s |
| a_refused_request_stops_naming_its_step_and_subject | 0.004 s |
| an_absent_bot_is_made_with_a_drawn_password_and_the_marker | 0.004 s |
| hook_urls_are_written_for_loopback_v4_and_v6 | 0.003 s |
| an_empty_admin_token_stops_before_any_request | 0.004 s |
| an_admin_token_of_a_plain_user_stops_at_the_admin | 0.004 s |
| the_bots_id_is_written_in_decimal_only_when_the_file_differs | 0.007 s |

```text
$ cargo nextest run -p temper-forge-setup -p temper-forge-setup-shell -p temper-box -p temper -p temper-forge-setup-reconcile-world
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Summary [   0.989s] 236 tests run: 236 passed, 0 skipped
exit 0; task 735, ran 2s
log: /srv/data/git/runner/src/tmp/logs/20261019T013132-735.log
```

Package time sums, not parallel wall times: setup 98/0.562 s (range
0.003–0.014); shell 24/1.579 s (0.003–0.552); box 35/0.138 s
(0.003–0.018); binary 58/3.128 s (0.003–0.334); world 21/0.580 s
(0.005–0.067). Shell's 0.552 s test is
`std_process_and_file_locks_appear_only_in_the_process_module`, an existing
source scan spawning rg, not a step unit or simulated world; it is retained
and not filtered. New units stay below 20 ms; the new command test targets
0.5 s. After deleting 13 and adding five setup units, one shell unit and one binary test:
setup 90, shell 25, box 35, binary 59, world 21 = 230 focused tests, before
independent tools-plan changes. Existing test names stay.

```text
$ cargo nextest run -p temper-forge-setup-reconcile-world --profile fuzzy
directory: /srv/data/git/runner/src/tmp/planning/temper-base
PASS [   0.493s] temper-forge-setup-reconcile-world::fuzzy_reconcile random_initial_forges_converge_and_a_second_run_writes_only_hook_patches
Summary [   0.493s] 1 test run: 1 passed, 0 skipped
exit 0; task 736, ran 8s
```
```text
$ scripts/check.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `dev` profile [unoptimized + debuginfo]
Summary [   6.770s] 1879 tests run: 1879 passed (1 slow), 1 skipped
Summary [  26.715s] 50 tests run: 50 passed (1 slow), 0 skipped
exit 0; task 737, ran 43s
log: /srv/data/git/runner/src/tmp/logs/20261019T013132-737.log
```

The existing gate's slow tests and skip were not changed. The gate is the merge's
during implementation; planning ran it once because development.md 5.4
requires every prescribed command at the base.

## Repin and workspace check

The update runs in the detached scratch so the supplied clean base's
Cargo.lock stays untouched. This is a base-form repin: the new forge kit
crates do not exist yet. No commits were made.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-temper-04 /srv/data/git/runner/src/tmp/planning/temper-base/HEAD
directory: /srv/data/git/runner/src/temper
fatal: invalid reference: /srv/data/git/runner/src/tmp/planning/temper-base/HEAD
exit 128; corrected to the base commit below
```
```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-temper-04 dd12b8ed904edced9e65af9fa62cc769c5a09905
directory: /srv/data/git/runner/src/temper
Preparing worktree (detached HEAD dd12b8ed9)
HEAD is now at dd12b8ed9 Check reconcile's given values in temper-forge-setup
exit 0
```
```text
$ cargo update -p skein-lib
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-04
Updating git repository `https://github.com/forgetic/skein.git`
Locking 29 packages to latest Rust 1.95 compatible versions
Updating skein-lib v0.0.0 (#8a51e10b) -> #25d2551e
All 29 packages of that git source moved together.
exit 0; task 739, ran 7s
```
```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-04
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
  --> crates/temper-worker-protocol-git/src/files.rs:296:37
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
  --> crates/temper-worker-protocol-git/src/files.rs:352:11
error: could not compile `temper-worker-protocol-git` (lib) due to 2 previous errors
exit 101; task 734, ran 19s
log: /srv/data/git/runner/src/tmp/logs/20261019T013602-734.log
```

This is the pre-tools-repin base. README Rules already require the tools
plan's first repin before temper 01.1, hence before this session. The draft
keeps that ordering and requires repairing any intervening upstream
breakage in the first repin commit. No out-of-scope worker fix was made.
The failed base-form command is reported to the planner; it is not evidence
that the future ready checkout builds.

```text
$ git restore --source=HEAD -- Cargo.lock
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-04
exit 0; only the scratch's repin was reverted before compiling the base-form sketches
```

## Process tier and binary-path correction

```text
$ TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Starting 19 tests across 1 binary
FAIL [   0.036s] temper::process a_change_lands_by_deliveries_with_every_poll_raised
crates/temper/tests/support/owned.rs:382:5:
assertion `left == right` failed: temper forge prepare: forge 1: command could not start: Other
Summary [   0.037s] 1/19 tests run: 0 passed, 1 failed, 0 skipped
warning: 18/19 tests were not run due to test failure
exit 100; task 738, ran 10s
```

README spells the binaries under $HOME/src/rust. On this machine the
workspace is /srv/data/git/runner/src; neither src/rust binary exists.
The workspace AGENTS.md explicitly allows this layout substitution, and
base-runs.md used $HOME/src. The pinned executable files exist under
/srv/data/git/runner/src/tmp/temper/forgejo. The corrected command is the
one prescribed in temper/04-setup.md. This changes the launch paths only.

## User searches

Searches run rg over Rust files in temper-base, skein-base and the primary
jig, smith and hearth checkouts. The lexical-name counts include homonyms;
the qualified-import scan identifies which are actual setup callers.

The Python commands below are recorded with equivalent ANSI-C shell quoting
for their multiline -c argument, keeping each command on one physical line.

```text
$ python3 -c $'import subprocess\nroots=[("temper","/srv/data/git/runner/src/tmp/planning/temper-base"),("skein","/srv/data/git/runner/src/tmp/planning/skein-base"),("jig","/srv/data/git/runner/src/jig"),("smith","/srv/data/git/runner/src/smith"),("hearth","/srv/data/git/runner/src/hearth")]\nnames=["Reconcile","Need","Event","Reply","Failure","Stage","Why","step","MAX_OUT","hook_url","Limits","Unfit","check","login","email","checked","Checked","Settings","bound","file_path","reconcile_settings","Step","small","apply"]\nprint("symbol temper skein jig smith hearth (matching Rust lines; includes homonyms)")\nfor name in names:\n    counts=[]\n    for label,root in roots:\n        p=subprocess.run(["rg","-n","-w",name,"--glob","*.rs",root],capture_output=True,text=True)\n        counts.append(str(len(p.stdout.splitlines())) if p.returncode in (0,1) else "ERROR:"+p.stderr.strip())\n    print(name+" "+" ".join(counts))\n'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
symbol temper skein jig smith hearth (matching Rust lines; includes homonyms)
Reconcile 36 1 0 7 0
Need 96 0 9 0 0
Event 3396 4098 9003 5231 1079
Reply 345 369 1335 172 295
Failure 313 518 460 776 113
Stage 195 111 273 170 20
Why 111 69 418 56 98
step 1021 627 2188 1449 212
MAX_OUT 90 228 115 87 18
hook_url 5 0 0 0 0
Limits 3099 2301 19363 3323 690
Unfit 66 0 14 0 0
check 692 493 4162 1610 222
login 418 0 209 180 33
email 96 3 0 6 5
checked 342 343 1015 459 162
Checked 29 84 75 165 16
Settings 518 204 394 630 125
bound 423 1107 1812 1274 341
file_path 14 0 0 0 0
reconcile_settings 2 0 0 0 0
Step 522 374 914 393 54
small 221 191 418 268 82
apply 48 88 114 26 8
exit 0
```

Qualified names, manifests included (rg expression matches the shell prefix too):

```text
$ python3 -c $'import subprocess\nroots=[("temper","/srv/data/git/runner/src/tmp/planning/temper-base"),("skein","/srv/data/git/runner/src/tmp/planning/skein-base"),("jig","/srv/data/git/runner/src/jig"),("smith","/srv/data/git/runner/src/smith"),("hearth","/srv/data/git/runner/src/hearth")]\npattern=r"temper_forge_setup|temper-forge-setup"\nfor label,root in roots:\n p=subprocess.run(["rg","-n",pattern,"--glob","*.rs","--glob","Cargo.toml",root],capture_output=True,text=True)\n lines=p.stdout.splitlines(); files={s.split(":")[0] for s in lines}\n print(label+": "+str(len(lines))+" lines / "+str(len(files))+" files")\n if label=="temper":\n  for f in sorted(files):\n   if f.endswith(".rs"):\n    print(f.removeprefix(root+"/")+": "+str(sum(s.startswith(f+":") for s in lines)))\n'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper: 81 lines / 35 files
crates/temper-box/src/host.rs: 1
crates/temper-box/src/machine.rs: 1
crates/temper-box/src/tests/boundary.rs: 3
crates/temper-box/src/tests/fixtures.rs: 1
crates/temper/src/box_command.rs: 1
crates/temper/src/forge.rs: 6
crates/temper/src/setup.rs: 1
crates/temper/tests/forge.rs: 8
crates/temper/tests/process.rs: 2
crates/temper/tests/support/owned.rs: 9
tests/forge-setup/reconcile/src/exchange.rs: 1
tests/forge-setup/reconcile/src/fixtures.rs: 6
tests/forge-setup/reconcile/src/world.rs: 1
tests/forge-setup/reconcile/tests/fuzzy_reconcile.rs: 1
tests/forge-setup/reconcile/tests/reconcile.rs: 5
skein: 0 lines / 0 files
jig: 0 lines / 0 files
smith: 0 lines / 0 files
hearth: 0 lines / 0 files
exit 0
```

The shell's remaining matching files are src/{command,exchange,files,prepare,
process,reconcile,run}.rs (one line each); tests/{boundary,commands,exchange,
files,locks,reconcile}.rs (1/2/2/1/1/2), tests/support/mod.rs (6).
Temper-box's six lines use Step or its boundary scan. Binary tests/forge.rs
(8), tests/process.rs (2), tests/support/owned.rs (9) use the retained
place/state/host APIs: audited, no moved-type signature there. Binary
src/forge.rs (6) owns reconcile_settings and src/setup.rs (1) calls checked;
src/box_command.rs (1) uses steps. No setup caller exists in the other repos.

```text
$ wc -l crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile/{hooks,orgs,tokens}.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
340 crates/temper-forge-setup/src/reconcile.rs
  200 crates/temper-forge-setup/src/reconcile/hooks.rs
  175 crates/temper-forge-setup/src/reconcile/orgs.rs
  216 crates/temper-forge-setup/src/reconcile/tokens.rs
  931 total
exit 0
```

## Completion commands at the unchanged base

```text
$ cargo check --workspace --all-targets
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `dev` profile [unoptimized + debuginfo] target(s) in 26.01s
exit 0; task 746, ran 26s
```
```text
$ rg -n 'pub mod reconcile|pub struct (Limits|Unfit)|^pub fn check|^fn (login|email)' crates/temper-forge-setup/src/{lib,limits,setup}.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-forge-setup/src/setup.rs:99:fn login(b: &[u8]) -> bool {
crates/temper-forge-setup/src/setup.rs:110:fn email(b: &[u8]) -> bool {
crates/temper-forge-setup/src/limits.rs:5:pub struct Limits {
crates/temper-forge-setup/src/limits.rs:19:pub struct Unfit {
crates/temper-forge-setup/src/limits.rs:57:pub fn check(limits: &Limits) -> Result<(), Unfit> {
crates/temper-forge-setup/src/lib.rs:11:pub mod reconcile;
exit 0; six source definitions to remove
```
```text
$ rg -n 'skein_forge_setup::|pub mod given' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
no matches
exit 1; expected at the base, before the kit/mapping exist
```

These searches intentionally show the pre-change state: six local definitions
exist and the future kit imports do not. The ready check uses landed skein
04.1, and Done when requires the opposite source properties. The base's
workspace check passes without the isolated repin; the repinned check's
Op::Lock errors above remain a reported base limitation.

## Changed-signature sketch

In the detached scratch, public signature-only modules were added inside
just temper-forge-setup and temper-forge-setup-shell. They compile
values(&Entry, Option<SocketAddr>, &Limits) -> Given;
refusal(u32, kit Refused) -> Problems; checked with the listener replacing
its boolean; place::item(Item) -> File; all fields of the new shell Settings
and its complete Refused enum including Setup { forge, problems }.
Given, Place, Refused and Item mirror skein 04.1's declared shapes; the base
Limits and dialect Limits/Failure are the base forms of plain moved types.
Bodies are unreachable!("signature-only planning sketch") placeholders;
there are no test/behavior claims or lint exceptions. Future kit imports
must be checked against the landed crate at readiness, as development.md
5.4 requires. No plain move was implemented or resketched.

```text
$ cargo clippy -p temper-forge-setup -p temper-forge-setup-shell --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-04
Checking temper-forge-setup v0.0.0
Checking temper-forge-setup-shell v0.0.0
Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.69s
exit 0; task 745, ran 6s; no warnings
```
```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-temper-04
directory: /srv/data/git/runner/src/temper
exit 0; scratch removed, no output
```

## Corrected process run

```text
$ TEMPER_FORGEJO=/srv/data/git/runner/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=/srv/data/git/runner/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
directory: /srv/data/git/runner/src/tmp/planning/temper-base
Starting 19 tests across 1 binary
PASS [  85.951s] temper::process a_change_lands_by_deliveries_with_every_poll_raised
PASS [   2.087s] temper::process a_prepared_forge_runs_forgejo_and_its_runner_once_each
PASS [   7.271s] temper::process a_push_is_delivered_signed_with_the_secret_reconcile_drew
PASS [   3.766s] temper::process reconcile_converges_an_owned_forgejo_and_a_second_run_writes_nothing
Summary [ 151.063s] 19 tests run: 19 passed (8 slow), 0 skipped
exit 0; task 737, ran 172s
log: /srv/data/git/runner/src/tmp/logs/20261019T013603-737.log
```

No live service was used; these are the pinned Forgejo and runner on loopback.
The 145.694 s in base-runs.md is the earlier base observation; this run is
151.063 s. The process script itself is unchanged and still runs once where
the planned step changes the production setup path.

## Review and scope

The development.md 5.4 consistency review was read-only and found no concrete
API/design/dependency defect. The literal-execution review is recorded below.
No commit or landing was made. The supplied temper-base still has an empty
git status and HEAD dd12b8ed904edced9e65af9fa62cc769c5a09905. Only the session
and this findings file persist; the scratch worktree was removed.

## Literal review fixes and their commands

The literal reader identified missing proofs of empty-setup limit checking,
the shell's constructor refusal, deletion and aliases, and a concrete
production-size command. The draft now:
- runs the limit-pair unit through check::checked with TWO and [] rows;
- adds invalid_given_values_are_refused_before_io with an empty bot login
  and insufficient io.output, expecting Setup's literal configuration path;
- checks the four deleted source files and moved-type aliases explicitly;
- counts production additions by named source paths before landing.

The reviewer also noted code-organization's interface-user table lists
temper-box. The inspected callers use only Step (host, machine, fixtures,
boundary); the session makes that distinction explicit. Binary's unchanged
place/state users are read-only, and it needs no direct kit dependency for
passing the step module's values to the host. Neither a new design decision
nor an additional dependency is needed. No other plan file was edited.
These fixes change no sketched signature, so the successful scratch clippy
result remains applicable.

```text
$ git diff --numstat origin/main -- crates/temper-forge-setup/src/{lib,check,limits,setup,place,given}.rs crates/temper-forge-setup-shell/src/{reconcile,exchange}.rs crates/temper/src/{setup,forge}.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
no output: 0 added production lines at the clean base
exit 0; task 734, ran 0s
```
```text
$ python3 -c 'from pathlib import Path; root=Path("crates/temper-forge-setup/src"); paths=[root/"reconcile.rs",*[root/"reconcile"/(name+".rs") for name in ("hooks","orgs","tokens")]]; remaining=[str(p) for p in paths if p.exists()]; print("reconcile copies:",len(remaining)); print("\n".join(remaining)); raise SystemExit(bool(remaining))'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
reconcile copies: 4
crates/temper-forge-setup/src/reconcile.rs
crates/temper-forge-setup/src/reconcile/hooks.rs
crates/temper-forge-setup/src/reconcile/orgs.rs
crates/temper-forge-setup/src/reconcile/tokens.rs
exit 1; expected source copies at the base, before their planned deletion
```

The first alias expression lacked a word boundary after use and matched
pub user fields. Running it at the base exposed that check defect; it was
corrected before the final draft:

```text
$ rg -n -U 'pub\s+use[^;]*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b|(?m:^\s*(pub(\([^)]*\))?\s+)?type\s+\w+\s*=\s*([A-Za-z0-9_]+::)*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b)|skein_forge_setup[^;]*\bas\s+\w+' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src crates/temper/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-forge-setup-shell/src/reconcile.rs:20:    pub user_agent: &'static str,
crates/temper-forge-setup/src/place.rs:8:    pub user: Box<[u8]>,
crates/temper-forge-setup/src/check.rs:25:    pub user: User,
exit 0; false positives, not aliases; replaced by the command below
```
```text
$ rg -n -U 'pub\s+use\b[^;]*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b|(?m:^\s*(pub(\([^)]*\))?\s+)?type\s+\w+\s*=\s*([A-Za-z0-9_]+::)*(Given|Reconcile|Need|Event|Reply|Failure|Stage|Why|Item|Place|Limits|Unfit|Refused)\b)|skein_forge_setup[^;]*\bas\s+\w+' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src crates/temper/src
directory: /srv/data/git/runner/src/tmp/planning/temper-base
no matches
exit 1; no moved-type aliases/re-exports at the base
```

Final prescribed-command coverage: repin, workspace check, focused listing
and run, fuzzy run, gate, corrected process proof, production-size count and
all five completion checks have base runs recorded. Commands needing the
future crate use source types in the signature sketch. The failed repinned
workspace check is the remaining base limitation, already covered by the
README's tools-first ordering; it is reported, not treated as a pass.




## Recount for the three-increment split (rulings of 2026-10-19)

The name checks below are reads at the base worktree (temper dd12b8e), run
as plain `rg`/`git` from `temper-base`: they build and test nothing, and
`rg` sat queued behind the paused build lane (task 795, removed from the
queue unrun). No base build or test run was added; every count and time used
by the three increments is one recorded above or in `base-runs.md`.

### What leaves tests/fixtures.rs, and what stays

```text
$ rg -n '^pub (const|fn|struct) (ADMIN|admin|person|team_spec|hook_row|subscribed|secret|run_command|runner_added|failed|Run)\b|^impl Run' crates/temper-forge-setup/src/tests/fixtures.rs
186:pub const ADMIN: &[u8] = b"site-admin-token";
187:pub fn secret(value: &[u8]) -> skein_lib::Secret {
190:pub fn admin() -> temper_forge_forgejo::document::User {
198:pub fn person(login: &[u8], full_name: &[u8]) -> temper_forge_forgejo::document::User {
206:pub struct Run {
210:impl Run {
274:impl Run {
315:pub fn run_command(command: crate::spec::Command) -> crate::prepare::Request {
318:pub fn runner_added() -> crate::prepare::Found {
323:pub fn failed() -> crate::prepare::Event {
330:impl Run {
352:pub fn team_spec() -> temper_forge_forgejo::document::TeamSpec {
361:pub fn hook_row(id: u64) -> temper_forge_forgejo::document::Row {
370:pub fn subscribed() -> Box<[Box<[u8]>]> {
exit 0
```

```text
$ rg -n 'fixtures::(secret|run_command|runner_added|failed)\(' crates/temper-forge-setup/src/tests -g '!reconcile.rs' -g '!fixtures.rs'
crates/temper-forge-setup/src/tests/upgrade.rs:17:    assert_eq!(drive.send(Event::Exists(false)), fixtures::run_command(Command::Dump));
crates/temper-forge-setup/src/tests/upgrade.rs:23:    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Migrate));
crates/temper-forge-setup/src/tests/upgrade.rs:43:    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Migrate));
crates/temper-forge-setup/src/tests/upgrade.rs:63:    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
crates/temper-forge-setup/src/tests/upgrade.rs:74:    assert_eq!(drive.reach(fixtures::runner_added()), Request::Lock(File::RunnerLock));
crates/temper-forge-setup/src/tests/upgrade.rs:82:            contents: fixtures::secret(b"0404040404040404040404040404040404040404")
crates/temper-forge-setup/src/tests/upgrade.rs:85:    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
crates/temper-forge-setup/src/tests/upgrade.rs:94:    let mut found = fixtures::runner_added();
crates/temper-forge-setup/src/tests/upgrade.rs:100:    assert_eq!(drive.send(Event::Done), fixtures::run_command(Command::Register));
crates/temper-forge-setup/src/tests/upgrade.rs:195:        drive.send(fixtures::failed()),
crates/temper-forge-setup/src/tests/upgrade.rs:215:        drive.send(fixtures::failed()),
crates/temper-forge-setup/src/tests/upgrade.rs:235:        drive.send(fixtures::failed()),
exit 0
```

`secret` is also called by `failed()` (fixtures.rs:326) and by `kept_tokens`
and `hook_listing` (both in the deleted `impl Run`). So: `Run` (struct 206-209,
impl blocks 210-249, 274-292 and 330-351), `ADMIN`, `admin`, `person`,
`team_spec`, `hook_row` and `subscribed` go; `secret`, `run_command`,
`runner_added` and `failed` stay. Lines counted per item (start-end):

```text
$ awk (closing "}" at column 0 after each start) over fixtures.rs
186-186 (1)  ADMIN
190-197 (8)  admin
198-205 (8)  person
206-209 (4)  struct Run
210-249 (40) first impl Run
274-292 (19) second impl Run (kept_tokens)
330-351 (22) third impl Run (hook_listing)
352-360 (9)  team_spec
361-368 (8)  hook_row
370-376 (7)  subscribed
total 126
```

`Ipv4Addr` and `SocketAddr` (imported at fixtures.rs:8) stay used by
`members()`, so these deletions leave no import unused.

### Helpers for the forge rows, and the forced edits in retained code

```text
$ rg -n '^pub fn (refused|command|prepare|reconcile)\(' crates/temper/tests/support/owned.rs
50:pub fn prepare(directory: &Path, forge: u16) -> Output {
203:pub fn command(directory: &Path, program: &str) -> Command {
341:pub fn refused(directory: &Path, verb: &str, expected: &[u8]) {
390:pub fn reconcile(directory: &Path, forge: u16) -> Output {
exit 0
```

`refused` runs `prepare(directory, 1)`, `reconcile(directory, 1)`, and
otherwise `command(directory, "forgejo").output()`; it asserts exit 1 and
stderr equal to `expected`. It has no `runner` arm, so the `run runner` row
calls `command(directory, "runner").output()` itself.

```text
$ rg -n 'limits::(check|Limits)|, (true|false), &\[|members, (true|false)' crates/temper-forge-setup/src/tests/{limits,read,members}.rs
members.rs:9:    let problems = check::checked(&fixtures::document(bytes), 0, members, false, &[]).unwrap_err();
members.rs:96:    let problems = check::checked(&fixtures::document(fixtures::TWO), 0, members, true, &[]).unwrap_err();
read.rs:42:    let limits = limits::Limits { orgs: 1, ..limits::small() }; // Below the shipped eight.
limits.rs:5:    assert_eq!(limits::check(&limits::small()), Ok(()));
limits.rs:7:        check::checked(&fixtures::document(fixtures::TWO), 0, &fixtures::members(), false, &[]).unwrap().setup,
limits.rs:17:        let problems = check::checked(&fixtures::document(fixtures::TWO), 0, &fixtures::members(), false, &[(name.into(), value)]).unwrap_err();
exit 0
```

```text
$ rg -n 'pub fn entry|pub entries' crates/temper-forge-setup/src/setup.rs
48:    pub entries: Box<[Entry]>,
62:    pub fn entry(&self, number: u16) -> Option<&Entry> {
exit 0
```

`entry(number)` returns the entry alone, no index, so `reconcile_settings`
(crates/temper/src/forge.rs) takes `setup_index` from
`checked.setup.entries.iter().position(|e| e.forge == number)`.

```text
$ git diff --numstat origin/main -- . ':!Cargo.lock'
exit 0   (prints nothing at the clean base)
```

### Sizes with tests counted (all "estimated" unless a line says "counted")

| Increment | New (est.) | Of which tests | Removes |
|---|---|---|---|
| 04.1 repin, kit limits and validators, refusal words | 125 | 45 (`tests/given.rs`) | about 50 production lines (limits.rs `Limits`, `Unfit`, `check`; setup.rs `login`, `email`) |
| 04.2 owner's values, check for every command | 170 | 110 (`tests/given.rs` 65, `startup.rs` 45) | nothing |
| 04.3 shell, world, deletions | 150 | about 75 (shell tests 30, world 30, item test 15) | 931 machine lines, 210 test lines (counted: 340+200+175+216; 210), 126 fixture lines (counted) |

Ruling 2's basis: the first draft's "280 new" and "about 980 removed" had no
basis in this file (it records 931). The 931 counts as 340 (`reconcile.rs`) +
200 (`hooks.rs`) + 175 (`orgs.rs`) + 216 (`tokens.rs`); `wc -l` at the base.

### Test counts by increment (joint = 5 packages as the base run above)

| After | setup | shell | box | binary | world | joint | gate focused |
|---|---|---|---|---|---|---|---|
| base | 98 | 24 | 35 | 58 | 21 | 236 | 1879 |
| 04.1 (+1 table test) | 99 | 24 | 35 | 58 | 21 | 237 | 1880 |
| 04.2 (+3 units, +1 binary) | 102 | 24 | 35 | 59 | 21 | 241 | 1884 |
| 04.3 (-13 moved, +1 item test, +1 shell test) | 90 | 25 | 35 | 59 | 21 | 230 | 1873 |

The 98-test package time, 0.043 s, is the figure skein/04-setup.md records for
`cargo nextest run -p temper-forge-setup` at its revised base; this file
records the joint 0.989 s only, and nothing was re-run for it.

### Reachability and the table (ruling 5)

The kit's check returns 13 Places (skein/04-setup.md, `enum Place`): Word,
BotLogin, BotEmail, AdminName, ApiName, GitName, Orgs, Org(u32), OrgOwner,
OrgsLimit, Deadline, Backoff, BackoffMax. With the two pair refusals
(Backoff against BackoffMax, BackoffMax against Deadline) the table has 15
rows. Through `check::checked`, temper's reader refuses a bad login, email,
org name, owner login or count before the kit's check runs, and `limits::apply`
refuses a zero or overflowing limit; only the two pair refusals can reach the
kit's check from a document, so the other thirteen rows are tested on a
constructed `Refused` (the table test), and the four literal names are proven
accepted by the two mapping tests, whose `given::check` returns `Ok(())`.
The four literals (`temper`, `temper-admin`, `temper-api`, `temper-git`) are
temper's own constants, not setup fields; none is invented.

### Sketch status

Sketch A's signatures are those compiled in the first revision's scratch
(source types); this revision changes none of them. `What::Own` and
`Problems::extend` are described in prose only and not recompiled; the
exhaustive `match` in `given::refusal` is likewise prose, to be written and
compiled by the author (clippy denies the wildcard arm).

### Done-when searches at the base (reads, run directly in temper-base)

```text
$ rg -n 'pub mod reconcile|pub struct (Limits|Unfit)|^pub fn check|^fn (login|email)' crates/temper-forge-setup/src/{lib,limits,setup}.rs
crates/temper-forge-setup/src/setup.rs:99:fn login(b: &[u8]) -> bool {
crates/temper-forge-setup/src/setup.rs:110:fn email(b: &[u8]) -> bool {
crates/temper-forge-setup/src/lib.rs:11:pub mod reconcile;
crates/temper-forge-setup/src/limits.rs:5:pub struct Limits {
crates/temper-forge-setup/src/limits.rs:19:pub struct Unfit {
crates/temper-forge-setup/src/limits.rs:57:pub fn check(limits: &Limits) -> Result<(), Unfit> {
exit 0
```
```text
$ rg -n 'webhooks: bool|listener: Option<SocketAddr>' crates/temper-forge-setup/src/check.rs
60:pub fn members(setup: &Setup, members: &Members, webhooks: bool, ps: &mut Problems) {
137:    webhooks: bool,
exit 0
```
```text
$ rg -l --sort path 'skein_forge_setup::' crates/temper-forge-setup/src crates/temper-forge-setup-shell/src
exit 1   (no file at the base; the plan's expected set after 04.3 is ten paths)
```
```text
$ rg -n 'Reconcile::new' crates tests -g '*.rs'
tests/forge-setup/reconcile/src/world.rs:118:        let mut machine = Reconcile::new(&self.entry, self.listener);
crates/temper-forge-setup/src/tests/fixtures.rs:219:        Self { machine: crate::reconcile::Reconcile::new(&entry, None), entry }
crates/temper-forge-setup/src/tests/fixtures.rs:335:        run.machine = crate::reconcile::Reconcile::new(&run.entry, Some(SocketAddr::from((Ipv4Addr::LOCALHOST, 9898))));
crates/temper-forge-setup-shell/src/reconcile.rs:370:        machine: Reconcile::new(&settings.entry, settings.listener),
exit 0   (four at the base; the two in fixtures.rs go with `Run`, leaving the shell's and the world's)
```
