# Base checks: skein 04, skein-forge-setup

2026-10-19. Source temper `27995fb75`; target skein `d120820`.
Only the session and this findings file are deliverables. No commit or landing.

## Discovery and decisions still pending

The specified graph project `srv-data-git-runner-src-temper` returned no
symbols for `crates/temper-forge-setup`; broader Reconcile results named
`temper.old/`. Its `search_code` for `pub struct Reconcile` under the
current crate also returned zero. Those results are insufficient for this
checkout; source names below were checked with rg and direct reads at the
base. No list, indexing or status graph operation was called.

Only `src/tests/reconcile.rs` moves: 13 tests, not the survey's 29.
The members/read/steps tests exercise configuration and commands that stay.
The limits tests exercise the owner’s `setup.*` diagnostics; they stay too.

Questions 24–28 remain pending in decisions.md. During this draft the
designer's answer to 23 arrived: Place per given field/limit, org by index,
Option<owner>, and relation order as written. The draft follows that answer,
including Forge and Listener places. One further clarification of
23 is required: the brief says the kit derives admin/API/git names from the
word, whereas forge-setup 4.1 explicitly lists those names as independent
given inputs and describes the derived names as the first owner's choice.
The draft follows the brief's recommendation, explicitly pending that
answer; it does not claim the published design already requires it.
No additional cross-session dependency was found: word checks here do not
import skein 01.3's marker module.

## Test inventory and base form of every Proof

`skein-forge-setup` does not exist at the base. The prescribed nextest
commands use `temper-forge-setup` as their base form. The target counts are
16 after 04.1 (13 moved + 3 new), 29 after 04.2 (+13), and 50 after 04.3
(+21). These future counts are test plans, not claims of executed tests.
The filtered list below inventories moved tests; execution is unfiltered.

```
$ cargo nextest list -p temper-forge-setup -E 'test(/^tests::reconcile::/)'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
temper-forge-setup tests::reconcile::a_bot_that_is_a_site_admin_is_refused
temper-forge-setup tests::reconcile::a_bot_with_the_marker_is_kept_and_nothing_is_written
temper-forge-setup tests::reconcile::a_draw_of_another_length_stops_before_creating_the_bot
temper-forge-setup tests::reconcile::a_forge_that_stops_answering_stops_the_run_at_its_step
temper-forge-setup tests::reconcile::a_lower_hook_met_later_deletes_the_higher_one_kept
temper-forge-setup tests::reconcile::a_missing_admin_token_stops_before_any_request
temper-forge-setup tests::reconcile::a_refused_request_stops_naming_its_step_and_subject
temper-forge-setup tests::reconcile::a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login
temper-forge-setup tests::reconcile::an_absent_bot_is_made_with_a_drawn_password_and_the_marker
temper-forge-setup tests::reconcile::an_admin_token_of_a_plain_user_stops_at_the_admin
temper-forge-setup tests::reconcile::an_empty_admin_token_stops_before_any_request
temper-forge-setup tests::reconcile::hook_urls_are_written_for_loopback_v4_and_v6
temper-forge-setup tests::reconcile::the_bots_id_is_written_in_decimal_only_when_the_file_differs
```

The same unfiltered command is the base form of all three Proofs and the
final suite condition; run once here, with setting NEXTEST_DOUBLE_SPAWN=0.

```
$ cargo nextest run -p temper-forge-setup
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 98 tests across 1 binary
        PASS [   0.017s] (33/98) temper-forge-setup tests::reconcile::a_bot_that_is_a_site_admin_is_refused
        PASS [   0.013s] (38/98) temper-forge-setup tests::reconcile::a_lower_hook_met_later_deletes_the_higher_one_kept
        PASS [   0.013s] (39/98) temper-forge-setup tests::reconcile::an_absent_bot_is_made_with_a_drawn_password_and_the_marker
        PASS [   0.012s] (40/98) temper-forge-setup tests::reconcile::a_missing_admin_token_stops_before_any_request
        PASS [   0.015s] (41/98) temper-forge-setup tests::reconcile::a_bot_with_the_marker_is_kept_and_nothing_is_written
        PASS [   0.014s] (42/98) temper-forge-setup tests::reconcile::a_refused_request_stops_naming_its_step_and_subject
        PASS [   0.006s] (43/98) temper-forge-setup tests::reconcile::a_draw_of_another_length_stops_before_creating_the_bot
        PASS [   0.007s] (44/98) temper-forge-setup tests::reconcile::a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login
        PASS [   0.008s] (45/98) temper-forge-setup tests::reconcile::a_forge_that_stops_answering_stops_the_run_at_its_step
        PASS [   0.006s] (48/98) temper-forge-setup tests::reconcile::an_empty_admin_token_stops_before_any_request
        PASS [   0.009s] (50/98) temper-forge-setup tests::reconcile::an_admin_token_of_a_plain_user_stops_at_the_admin
        PASS [   0.010s] (51/98) temper-forge-setup tests::reconcile::the_bots_id_is_written_in_decimal_only_when_the_file_differs
        PASS [   0.015s] (56/98) temper-forge-setup tests::reconcile::hook_urls_are_written_for_loopback_v4_and_v6
     Summary [   0.158s] 98 tests run: 98 passed, 0 skipped
```

All moved tests are under 20 ms: 0.006–0.017 s. The full source package
contains several prepare/upgrade tests at 0.021–0.034 s; none moves. This
run's total is 0.158 s; the earlier base-runs.md sample was 0.140 s.
The run includes `the_small_profile_passes_the_setup_limit_checks`, which
passed at 0.007 s: the shipped builder's 8 orgs / 120 s / 100 ms / 5 s
is admitted alongside temper's configuration checks.

## Clippy and changed signature sketch

The source-package clippy command is the base form of the target-package
Done when command.

```
$ cargo clippy -p temper-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Checking temper-forge-setup v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/crates/temper-forge-setup)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.78s
```

The detached scratch worktree holds a public signature-only module inside
temper-forge-setup, using the base dialect's Request and source Failure.
It compiles Given's complete fields, Limits, Place, Refused, Item and the
changed Need fields; checked Reconcile::new; given/limits checks; Names
and its derivation signature; Orgs::new. Bodies are explicit
`unreachable!("signature-only planning sketch")` placeholders, not behavior
implementations. The module exposes internal shapes solely to compile them;
production visibility and paths are those of the session. All selected
code builds unoptimized. No plain moved interface needs a new sketch.

```
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-04 27995fb75
directory: /srv/data/git/runner/src/temper
Preparing worktree (detached HEAD 27995fb75)
HEAD is now at 27995fb75 Point temper's forge documents to skein's forge kit
```
```
$ cargo clippy -p temper-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04
    Checking temper-forge-setup v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-04/crates/temper-forge-setup)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 31.01s
```

A second compile, recorded below when complete, adds the internal changed
signatures `Tokens::new(id: u64, names: &Names) -> Self` and
`Tokens::name(&self) -> &[u8]`. The ready check must compile against the
landed skein-forge paths; those paths do not exist at this base.

## Moved source size and names

The four production machine files total 931 lines. Limits contributes its
type and relation check (not its apply/small/override names); setup contributes
only login/email validation. The approximate 900 moved / 250 new production
Size allows for lines changed by those boundary substitutions; tests and
fixtures are outside that production Size. No moved file exceeds 800 lines.
The implementation's per-pair diff count is code-organization.md 6's, against
the source origin/main commit at the step's start, not this base snapshot.

```
$ wc -l crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile/hooks.rs crates/temper-forge-setup/src/reconcile/orgs.rs crates/temper-forge-setup/src/reconcile/tokens.rs crates/temper-forge-setup/src/limits.rs crates/temper-forge-setup/src/tests/reconcile.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
  340 crates/temper-forge-setup/src/reconcile.rs
  200 crates/temper-forge-setup/src/reconcile/hooks.rs
  175 crates/temper-forge-setup/src/reconcile/orgs.rs
  216 crates/temper-forge-setup/src/reconcile/tokens.rs
   65 crates/temper-forge-setup/src/limits.rs
  210 crates/temper-forge-setup/src/tests/reconcile.rs
 1206 total
```
```
$ rg -n -i 'temper' crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-forge-setup/src/reconcile.rs:13:use temper_forge_forgejo::{
crates/temper-forge-setup/src/reconcile.rs:127:            Stage::Admin => bytes::copy_of(b"temper-admin"),
crates/temper-forge-setup/src/reconcile.rs:132:    fn id(&mut self, user: temper_forge_forgejo::document::User) -> Need {
crates/temper-forge-setup/src/reconcile.rs:136:        if bytes::find(&user.full_name, b"temper:bot").is_none() {
crates/temper-forge-setup/src/reconcile.rs:233:                    full_name: bytes::copy_of(b"temper:bot"),
crates/temper-forge-setup/src/reconcile.rs:244:fn account(event: Event) -> Result<temper_forge_forgejo::document::User, Why> {
crates/temper-forge-setup/src/reconcile/tokens.rs:7:use temper_forge_forgejo::{
crates/temper-forge-setup/src/reconcile/tokens.rs:38:            File::ApiToken => b"temper-api",
crates/temper-forge-setup/src/reconcile/tokens.rs:39:            File::GitToken => b"temper-git",
crates/temper-forge-setup/src/reconcile/hooks.rs:11:use temper_forge_forgejo::{
crates/temper-forge-setup/src/reconcile/orgs.rs:8:use temper_forge_forgejo::{
```

The whole-crate name scan is recorded below as the base form of Done when.
It necessarily finds names in the source; only the moved/new kit files must
have none. Import renames alone cannot remove the four literal identities.

## Users of every moved machine symbol

The script extracts every function/type/enum/constant in the moved machine,
then adds the moved limits/check/validators and the relevant staying Step
and fixture Run. It runs rg separately for every name in all five repos.
These are lexical occurrence counts, including definitions and unrelated
same-named symbols, not resolved call counts. Missing conventional source
directories are absent coverage, not hidden code; the qualified import
search below identifies the users that actually depend on this crate.

```
$ python3 -c 'import pathlib,re,subprocess
roots=[pathlib.Path("/srv/data/git/runner/src")/r for r in ["temper","skein","jig","smith","hearth"]]
files=list(pathlib.Path("crates/temper-forge-setup/src/reconcile").glob("*.rs"))+[pathlib.Path("crates/temper-forge-setup/src/reconcile.rs")]
names=sorted(set(re.findall(r"\b(?:fn|struct|enum|const)\s+(\w+)", "\n".join(p.read_text() for p in files))))
names+=["Limits","Unfit","check","login","email","Step","Run"]
for root in roots:
    print(root.name)
    counts=[]
    for name in names:
        p=subprocess.run(["rg","-w","-o","--glob","*.rs","--glob","!temper.old/**",name,str(root/"crates"),str(root/"testing"),str(root/"tests"),str(root/"tools")],capture_output=True,text=True)
        if p.returncode not in (0,1,2): raise RuntimeError(p.stderr)
        counts.append(name+"="+str(len(p.stdout.splitlines())))
    print(" ".join(counts))
print("Patterns: rg -w -o --glob '\''*.rs'\'' NAME <repo>/{crates,testing,tests,tools}; absent subdirectories count zero; lexical hits, not resolved calls.")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper
Event=3575 Failure=326 Hooks=54 MAX_OUT=93 Need=100 Orgs=18 Phase=872 Reconcile=36 Reply=405 Stage=220 State=848 Tokens=77 Why=114 account=277 admin=304 advance=209 begin=74 bot=220 cap=186 create=187 draw=43 event=1746 hex=69 hook_url=5 id=2081 listing=91 name=3617 new=8769 next=975 remake=4 reply=426 rows=1711 save=199 send=599 spec=486 start_hooks=2 start_orgs=3 step=1097 stop=165 stop_named=3 teams=68 unexpected=56 Limits=3181 Unfit=66 check=712 login=493 email=113 Step=532 Run=353
skein
Event=4284 Failure=556 Hooks=0 MAX_OUT=209 Need=0 Orgs=0 Phase=1104 Reconcile=1 Reply=347 Stage=134 State=1452 Tokens=30 Why=66 account=427 admin=0 advance=33 begin=29 bot=0 cap=493 create=64 draw=171 event=1945 hex=69 hook_url=0 id=2034 listing=28 name=2872 new=6166 next=1369 remake=0 reply=341 rows=139 save=18 send=617 spec=29 start_hooks=0 start_orgs=0 step=609 stop=207 stop_named=0 teams=0 unexpected=112 Limits=2231 Unfit=0 check=478 login=0 email=3 Step=478 Run=242
jig
Event=9416 Failure=510 Hooks=0 MAX_OUT=85 Need=9 Orgs=0 Phase=2483 Reconcile=0 Reply=1413 Stage=326 State=983 Tokens=3 Why=440 account=863 admin=0 advance=255 begin=97 bot=0 cap=388 create=91 draw=54 event=3037 hex=6 hook_url=0 id=2260 listing=15 name=5737 new=44455 next=1752 remake=0 reply=1459 rows=6590 save=261 send=1216 spec=1573 start_hooks=0 start_orgs=0 step=1962 stop=197 stop_named=0 teams=0 unexpected=43 Limits=17693 Unfit=14 check=3785 login=242 email=0 Step=907 Run=784
smith
Event=4168 Failure=842 Hooks=0 MAX_OUT=91 Need=0 Orgs=0 Phase=876 Reconcile=6 Reply=193 Stage=236 State=701 Tokens=119 Why=56 account=1094 admin=0 advance=97 begin=48 bot=0 cap=365 create=56 draw=57 event=1597 hex=48 hook_url=0 id=1303 listing=97 name=4680 new=9858 next=857 remake=0 reply=264 rows=29 save=31 send=153 spec=451 start_hooks=0 start_orgs=0 step=1411 stop=392 stop_named=0 teams=0 unexpected=55 Limits=3339 Unfit=0 check=1074 login=126 email=1 Step=421 Run=237
hearth
Event=1136 Failure=114 Hooks=0 MAX_OUT=18 Need=0 Orgs=0 Phase=50 Reconcile=0 Reply=307 Stage=20 State=267 Tokens=0 Why=117 account=13 admin=15 advance=30 begin=7 bot=0 cap=1 create=9 draw=42 event=346 hex=146 hook_url=0 id=146 listing=267 name=3213 new=2462 next=466 remake=0 reply=49 rows=148 save=5 send=15 spec=547 start_hooks=0 start_orgs=0 step=224 stop=12 stop_named=0 teams=3 unexpected=11 Limits=735 Unfit=0 check=222 login=33 email=5 Step=69 Run=104
Patterns: rg -w -o --glob '*.rs' NAME <repo>/{crates,testing,tests,tools}; absent subdirectories count zero; lexical hits, not resolved calls.
```

## Qualified consumer paths

```
$ rg -n 'temper_forge_setup|temper-forge-setup' crates/temper-box crates/temper/src crates/temper/tests crates/temper-forge-setup-shell tests/forge-setup/reconcile
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper/src/setup.rs:3:use temper_forge_setup::{
crates/temper-box/src/tests/fixtures.rs:8:use temper_forge_setup::Step;
tests/forge-setup/reconcile/src/world.rs:6:use temper_forge_setup::{
crates/temper/src/forge.rs:8:use temper_forge_setup::{
crates/temper/src/forge.rs:16:use temper_forge_setup_shell::{command, prepare, run as host};
crates/temper/src/forge.rs:171:        return temper_forge_setup_shell::reconcile::reconcile(settings, &skein_shell::Clock::new())
crates/temper/src/forge.rs:226:    checked: &temper_forge_setup::check::Checked,
crates/temper/src/forge.rs:228:) -> Result<temper_forge_setup_shell::reconcile::Settings, String> {
crates/temper/src/forge.rs:255:    Ok(temper_forge_setup_shell::reconcile::Settings {
tests/forge-setup/reconcile/src/exchange.rs:14:use temper_forge_setup::reconcile::Reply;
crates/temper-box/src/machine.rs:6:use temper_forge_setup::Step;
crates/temper-box/src/host.rs:17:use temper_forge_setup::Step;
crates/temper-box/Cargo.toml:10:temper-forge-setup.workspace = true
crates/temper-forge-setup-shell/tests/exchange.rs:2:use temper_forge_setup_shell::exchange::{Gather, Next, pause};
crates/temper-forge-setup-shell/tests/exchange.rs:53:    assert_eq!(reply, temper_forge_setup::reconcile::Reply::Rows(Box::new([Row::Commit(first), Row::Commit(second)])));
crates/temper-forge-setup-shell/src/reconcile.rs:10:use temper_forge_setup::{
crates/temper-forge-setup-shell/src/exchange.rs:15:use temper_forge_setup::reconcile::Reply;
crates/temper-forge-setup-shell/Cargo.toml:2:name = "temper-forge-setup-shell"
crates/temper-forge-setup-shell/Cargo.toml:10:temper-forge-setup.workspace = true
tests/forge-setup/reconcile/Cargo.toml:2:name = "temper-forge-setup-reconcile-world"
tests/forge-setup/reconcile/Cargo.toml:14:temper-forge-setup.workspace = true
# Selected consumer lines; full output in test log, 74 matching lines.
```

The actual machine callers are the shell and reconcile world. The shell's
exchange and tests also name Reply. temper's binary builds shell Settings
and reads the owner's setup; box holds only Step. No moved machine type
belongs in box. These consumers are listed read-only in skein 04.1's Files,
and temper 04.1 owns their eventual migration. The old config limits/places
remain live until that step replaces their users; no re-export is planned.

The source `setup::account` calls both `login` and `email`, and `strings`
calls `login` for org names (rg confirms those three call sites). The kit
exposes the copied validator functions so temper 04.1 can delete their old
bodies while still checking the person's email, which is not a Given field.
Only visibility changes; their signatures and bodies are plain moves.

## Completed queued checks

The second scratch compile includes Tokens::new/name; it passed after the
shared queue became available.

```
$ cargo clippy -p temper-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04
    Checking temper-forge-setup v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-04/crates/temper-forge-setup)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.33s
```

The final name-scan base form below is expected to match source code; the
future target scan must print nothing. Selected moved-source lines only:

```
$ rg -n -i 'temper' crates/temper-forge-setup
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-forge-setup/src/tests/reconcile.rs:6:use temper_forge_forgejo::{document::Document, request::Request};
crates/temper-forge-setup/src/tests/reconcile.rs:7:fn user(user: temper_forge_forgejo::document::User) -> Event {
crates/temper-forge-setup/src/tests/reconcile.rs:21:            subject: b"temper-admin".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:36:                login: b"temper".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:37:                email: b"temper@example.invalid".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:38:                full_name: b"temper:bot".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:45:    assert_eq!(run.step(user(fixtures::person(b"temper", b"temper:bot"))), Need::Read(File::ApiToken));
crates/temper-forge-setup/src/tests/reconcile.rs:54:    assert_eq!(run.step(user(fixtures::person(b"temper", b"note temper:bot account"))), Need::Read(File::ApiToken));
crates/temper-forge-setup/src/tests/reconcile.rs:75:    let mut bot = fixtures::person(b"temper", b"temper:bot");
crates/temper-forge-setup/src/tests/reconcile.rs:79:        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::BotIsAdmin })
crates/temper-forge-setup/src/tests/reconcile.rs:93:            subject: b"temper".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:104:        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::Down })
crates/temper-forge-setup/src/tests/reconcile.rs:112:        run.step(user(fixtures::person(b"temper", b"temper:bot")));
crates/temper-forge-setup/src/tests/reconcile.rs:134:            subject: b"temper-admin".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:148:            subject: b"temper-admin".as_slice().into(),
crates/temper-forge-setup/src/tests/reconcile.rs:160:        Need::Stop(Failure { forge: 1, step: Stage::Bot, subject: b"temper".as_slice().into(), why: Why::Unexpected })
crates/temper-forge-setup/src/tests/reconcile.rs:178:    use temper_forge_forgejo::request::{HookSpec, Page};
crates/temper-forge-setup/src/reconcile/tokens.rs:7:use temper_forge_forgejo::{
crates/temper-forge-setup/src/reconcile/tokens.rs:38:            File::ApiToken => b"temper-api",
crates/temper-forge-setup/src/reconcile/tokens.rs:39:            File::GitToken => b"temper-git",
crates/temper-forge-setup/src/reconcile/orgs.rs:8:use temper_forge_forgejo::{
crates/temper-forge-setup/src/reconcile/hooks.rs:11:use temper_forge_forgejo::{
# 89 source matching lines in all.
```

Qualified crate imports and manifests, including source tests, give the
following counts. No qualified user exists in skein/jig/smith/hearth. The
77 lines include the source crate's own manifest/imports; the earlier
explicit consumer-path search has 74 lines.

```
$ python3 -c 'import pathlib,re,subprocess
roots=[pathlib.Path("/srv/data/git/runner/src")/r for r in ["temper","skein","jig","smith","hearth"]]
for root in roots:
 p=subprocess.run(["rg","-n","--glob","*.rs","--glob","Cargo.toml","temper_forge_setup|temper-forge-setup",str(root/"crates"),str(root/"testing"),str(root/"tests"),str(root/"tools")],capture_output=True,text=True)
 print(root.name+": "+str(len(p.stdout.splitlines()))+" matching lines")
 for group in ["crates/temper-box","crates/temper/src","crates/temper/tests","crates/temper-forge-setup-shell","tests/forge-setup/reconcile"]:
  lines=[l for l in p.stdout.splitlines() if "/"+group+"/" in l]
  if lines: print("  "+group+": "+str(len(lines))+" lines in "+str(len(set(l.split(":")[0] for l in lines)))+" files")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper: 77 matching lines
  crates/temper-box: 7 lines in 5 files
  crates/temper/src: 8 lines in 3 files
  crates/temper/tests: 19 lines in 3 files
  crates/temper-forge-setup-shell: 24 lines in 15 files
  tests/forge-setup/reconcile: 16 lines in 6 files
skein: 0 matching lines
jig: 0 matching lines
smith: 0 matching lines
hearth: 0 matching lines
```

Copies of the four changed literal identities were also searched. Their
remaining temper consumers are the later owner/fake/world/throwaway moves;
none asks this session to edit downstream code.

```
$ python3 -c 'import pathlib,subprocess
for repo in ["temper","skein","jig","smith","hearth"]:
 root=pathlib.Path("/srv/data/git/runner/src")/repo
 paths=[str(root/p) for p in ["crates","testing","tests","tools"] if (root/p).is_dir()]
 parts=[]
 for pattern in ["temper-admin","temper:bot","temper-api","temper-git"]:
  p=subprocess.run(["rg","-F","-l",pattern,*paths],capture_output=True,text=True)
  parts.append(pattern+"="+str(len(p.stdout.splitlines()))+" files")
 print(repo+": "+"; ".join(parts))
'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper: temper-admin=18 files; temper:bot=21 files; temper-api=15 files; temper-git=10 files
skein: temper-admin=0 files; temper:bot=0 files; temper-api=0 files; temper-git=0 files
jig: temper-admin=0 files; temper:bot=0 files; temper-api=0 files; temper-git=0 files
smith: temper-admin=0 files; temper:bot=0 files; temper-api=0 files; temper-git=0 files
hearth: temper-admin=0 files; temper:bot=0 files; temper-api=0 files; temper-git=0 files
```

The designer's newly received answer 23 adds Place::Forge and Place::Listener
to the signature sketch, so every given value has a place. The final clippy
run and scratch removal are recorded below.

```
$ cargo clippy -p temper-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04
    Checking temper-forge-setup v0.0.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1.95s
```

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-04
directory: /srv/data/git/runner/src/temper
exit 0; no output; scratch worktree removed
```

All four prescribed command forms have run at the base (nextest run/list,
clippy and the source-name scan). New test counts and the absence of owner
names can only be checked in the implemented target crate; their base
forms are explicitly distinguished above. No build/test command failed.

## Review revision, 2026-10-19

This section supersedes the draft's naming/dependency conclusions, signature
sketch and planned test counts above; the recorded base runs remain evidence.
The interrupted revision had already edited the session and created a detached
skein scratch at `d9f3e39`. This continuation read and retained its corrections,
completed their wording, compiled its revised signatures and removed the scratch.
The inputs are skein-base `d9f3e39` and temper-base `b4cfb999`: their code is
the earlier bases', with the designer's documentation answers added.

The graph's `search_graph` for the specified temper project, restricted to
`crates/temper-forge-setup/.*`, again returned zero results for
Reconcile/Orgs/Tokens/Run/check/managed/owned/hook_listing. Skein's graph found
Duration in `crates/skein-lib/src/time.rs`, confirmed with rg, but no valid_word.
The latter is a planned skein 01.3 export: its signature and word rule were
checked with rg in `skein/01-forge.md`. The source crate's graph coverage is
insufficient; direct rg and source reads verified every existing name used by
this revision. No index/list/status operation was used.

The six defects are resolved as follows:

1. `Given` carries `admin_name`, `api_name` and `git_name`, with AdminName,
   ApiName and GitName refusal places. Each empty name is a separate row of
   `invalid_given_values_name_their_place`; both check and constructor refuse
   it. Only `<word>:bot` is derived. Names/names() are gone. The independent
   names test keeps `kit-admin`, `kit-api`, `kit-git` while changing the word
   to `other`, and answers token runs with the matching `other:bot` marker.
   This follows forge-setup 4.1 directly; no naming question remains.
2. The move uses `skein_forge::marker::valid_word` and needs skein 01.3's
   shared check. It adds no duplicate word validator.
3. All four zero-limit refusals and `a_zero_limit_names_its_place` are in
   04.2. The move retains only the base check's two relations, in answer 23's
   order. The zero test calls limits::check, given::check and Reconcile::new.
4. Forge and Listener refusal places are dropped: every u16 forge number
   and optional SocketAddr is admitted. No listener, or a non-loopback one,
   skips hooks as 4.3 says. The accepted-input test covers these values.
5. Shape names the Account replacement (owned login/email), Unfit's removal
   in favor of Refused, Orgs' optional owner and Run's three constructors.
   Managed is forge 1/acme/None; owned is forge 2/work/Some(ada); hook_listing
   starts managed and rebuilds with listener 127.0.0.1:9898. The fixture's
   forge()/bot() accessors read Given. Reply scripts retain their behavior.
6. Both production increments have a temper's-side line. The shell and
   world change their constructor and Item arms; the owner maps places and
   supplies Given. The binary's checked/steps calls and box's Step consumers
   are named accurately as transitive consumers whose signatures stay put.
   Any later behavioral repair belongs to the next temper repin.

Answer 23's optional owner and ordered place pairs are retained. Questions
24–28 stay pending the designer (decisions.md, Design questions 24–28).
The revised counts are 17 after 04.1 (13 moved + 4 new), 31 after 04.2
(4 given-value tests and 10 reconcile tests added), and 51 after 04.3
(17 step tests and 3 limit tests added). The reconcile listing becomes 23.
These are planned counts, not executed target tests. The 98-test base run,
0.158 s, and its 13 moved tests at 0.006–0.017 s remain unchanged evidence.
Sizes are 900 moved/280 new, 0/120, and 0/0 production lines respectively.

One integration edit is outside this revision's two-file scope: README's
skein 04 row still lists only temper 01.2 at start. It must also name skein
01.3's shared word check to agree with this session's first Needs. The two
steps both follow temper 01.2, so the README currently does not guarantee
that the shared check has landed. No README or other session was edited.

The resumed scratch contains only the revision's new/changed signatures:
Given, Place, Refused, the given and limits checks, marker/login/email,
Reconcile's constructor and bot fields, Orgs::new, and Tokens::new/name.
Bodies are unreachable signature placeholders; internal types are public
solely for this compile. It depends on skein-base's actual skein-lib, uses
the workspace's lints and has no lint exception. The existing Item/Need
sketch was not changed by this review. The future skein-forge imports and
valid_word call still require the ready check against landed skein 01.3.

```
$ cargo clippy -p skein-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04-review
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.04s
```

The consumer scan corroborates the two direct constructors and the indirect
owner/Step callers. The box limits check and FakeMachine hits below are
distinct, staying types; they are not reconcile repairs.

```
$ rg -n 'Reconcile::new|limits::check|check::checked|temper_forge_setup::steps|Service::new|Machine::new' crates/temper-forge-setup-shell/src/reconcile.rs tests/forge-setup/reconcile/src/world.rs crates/temper-forge-setup/src/check.rs crates/temper/src/setup.rs crates/temper/src/forge.rs crates/temper/src/box_command.rs crates/temper-box/src/host.rs crates/temper-box/src/tests/fixtures.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-box/src/tests/fixtures.rs:36:        let machine = Machine::new(limits, &owned_steps()).expect("valid limits");
crates/temper-box/src/tests/fixtures.rs:155:        let mut machine = FakeMachine::new();
crates/temper-box/src/tests/fixtures.rs:167:        let service = Service::new(&limits::small(), Box::from(steps), settings).expect("valid limits");
crates/temper/src/box_command.rs:23:    let steps = temper_forge_setup::steps(&checked.setup, configuration.as_os_str().as_bytes());
crates/temper/src/box_command.rs:39:        let service = Service::new(&limits, steps, settings).map_err(|why| format!("box limits: {why:?}"))?;
crates/temper-forge-setup/src/check.rs:154:    if let Err(w) = limits::check(&limits) {
crates/temper/src/setup.rs:54:    let checked = check::checked(&document, at, &members, named.webhooks.is_some(), &setup_overrides)
crates/temper/src/setup.rs:74:    if let Err(why) = limits::check(&limits) {
tests/forge-setup/reconcile/src/world.rs:38:        let mut service = Service::new(fixtures::config(), &limits, seed).expect("world service");
tests/forge-setup/reconcile/src/world.rs:118:        let mut machine = Reconcile::new(&self.entry, self.listener);
crates/temper-forge-setup-shell/src/reconcile.rs:370:        machine: Reconcile::new(&settings.entry, settings.listener),
crates/temper-box/src/host.rs:90:        let machine = Machine::new(limits, &steps)?;
```

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-04-review
directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output; scratch worktree removed
```

No new Proof or Done-when command form was added. Their base equivalents
already ran above, once each. Only the session and this appended findings
section are deliverables; no commit or landing was made.

## Second answers revision, 2026-10-19

This appendix supersedes all earlier pending-question statements for 23–29
and the earlier planned counts. Inputs: skein `8938e16`, temper `dd12b8e`,
the designer's two answers, the current forge-setup design (4.1, 4.3, 7),
and the first review at `tmp/planning/reviews/skein-04.txt`. The alternate
`reviews/skein-04.txt` under the plan was absent. No commit or landing.

Graph search_graph and search_code for project
`srv-data-git-runner-src-temper`, scoped to temper-forge-setup, both returned
zero results. As in the earlier audit, this is insufficient coverage, not
absence of code. Existing names and callers were verified with rg at the
base; no project list/index/status operation was used.

The prior review's independent names/Places, shared valid_word dependency,
optional org owner, ordered refusal pairs, zero checks in 04.2, admitted
forge/listener values and forced move/fixture changes all remain.

1. Answer 24: base `setup::strings` refuses an empty list and limits its
   length, but Reconcile::new/Orgs::new assume a parsed Entry. The 04.1 move
   keeps that machine behavior. 04.2 adds the zero/over-limit admission to
   given::check and the constructor; the existing planned zero, bound and
   bound-plus-one tests assert Orgs/None or acceptance before any need.
2. Answer 25: base Reconcile::id uses bytes::find, with neither boundary.
   04.2 requires both edges of the complete `<word>:bot` to meet the stated
   `[a-z0-9-]` boundary, scanning past an invalid occurrence. The left-edge
   test uses word b against ab:bot; the separate right-edge test refuses
   b:botx, b:bot0 and b:bot-. Acceptance covers start/end, punctuation and a
   valid occurrence after a rejected left or right edge.
3. Answer 26: base id checks is_admin first. 04.2 checks the mark first;
   the simultaneous case (is_admin true, full_name `a note`) must stop with
   forge 1, Bot, subject kit, Unmarked. The moved marked-admin test still
   expects BotIsAdmin.
4. Answer 27: base already treats empty AdminToken as missing and writes
   decimal 42 over an empty User. It sends empty API/git tokens to GET User,
   and keeps an empty WebhookSecret. 04.2 normalizes all five Items. Four
   new tests plus the retained empty-admin test cover every Item explicitly.
5. Answer 28: base Tokens::event remakes on 401 only; 403 currently stops.
   04.2 adds 403, keeps every other status and Down terminal, and tests both
   token slots. The 401 full delete/create/write test moves from planned
   04.3 to 04.2, retaining its name. The 403 test uses the same script and
   asserts fixed scopes; 404/429/500/502/503 and Down stop without a remake.
6. Answer 29: both temper's-side paragraphs now put the mapping and check
   in temper-forge-setup's step code, reached through checked for every
   command, serve included, with no shell. Refused Places name the paths
   of temper protocol/setup.md 5. temper 04.1 builds it; later repins repair
   changed expectations. The base serve caller is lib.rs::serve through
   prepare/setup::check, not a serve.rs file. The binary's reconcile_settings
   consumes mapped values for the host; it does not own the mapping/check.
7. forge-setup 4.3 step 2: base Reconcile::draw already sends
   must_change_password false. The moved
   an_absent_bot_is_made_with_a_drawn_password_and_the_marker asserts it;
   04.1 names preserving both, with no additional behavior or test count.

All seven changes are specified in their owning increment. No new public
signature was needed in this pass. The world connection references now
name skein 07.3, matching the README. The session is 399 lines. Counts:

| Increment | Moved tests | New tests | Running total | Size, moved/new production |
|---|---:|---:|---:|---|
| 04.1 | 13 | 4 | 17 | about 900 / 280 |
| 04.2 | 0 | 16 (4 given, 12 reconcile) | 33 | about 0 / 140 |
| 04.3 | 0 | 19 (16 steps, 3 limits) | 52 | about 0 / 0 |

The final reconcile listing is 25 (13 moved + 12 new). These are planned
kit counts; the target crate and the new tests do not exist at the base.
No Proof/Done-when command form was added. Their base test forms were
rerun once below; previous base clippy/name-scan evidence remains valid.
The suite was unoptimized: 98 passed, 0 skipped, 0.043 s; all 13 moved
reconcile tests took 0.002–0.004 s. The shipped-profile admission test
passed in 0.003 s. Earlier recorded timings remain historical samples.

```
$ cargo nextest run -p temper-forge-setup
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 98 tests across 1 binary
        PASS [   0.003s] (14/98) temper-forge-setup tests::limits::the_small_profile_passes_the_setup_limit_checks
        PASS [   0.002s] (35/98) temper-forge-setup tests::reconcile::a_bot_that_is_a_site_admin_is_refused
        PASS [   0.003s] (37/98) temper-forge-setup tests::reconcile::a_bot_with_the_marker_is_kept_and_nothing_is_written
        PASS [   0.003s] (38/98) temper-forge-setup tests::reconcile::a_draw_of_another_length_stops_before_creating_the_bot
        PASS [   0.003s] (40/98) temper-forge-setup tests::reconcile::a_forge_that_stops_answering_stops_the_run_at_its_step
        PASS [   0.002s] (41/98) temper-forge-setup tests::reconcile::a_missing_admin_token_stops_before_any_request
        PASS [   0.002s] (42/98) temper-forge-setup tests::reconcile::a_lower_hook_met_later_deletes_the_higher_one_kept
        PASS [   0.002s] (43/98) temper-forge-setup tests::reconcile::a_refused_request_stops_naming_its_step_and_subject
        PASS [   0.003s] (44/98) temper-forge-setup tests::reconcile::a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login
        PASS [   0.002s] (45/98) temper-forge-setup tests::reconcile::an_admin_token_of_a_plain_user_stops_at_the_admin
        PASS [   0.003s] (46/98) temper-forge-setup tests::reconcile::an_absent_bot_is_made_with_a_drawn_password_and_the_marker
        PASS [   0.002s] (47/98) temper-forge-setup tests::reconcile::hook_urls_are_written_for_loopback_v4_and_v6
        PASS [   0.002s] (48/98) temper-forge-setup tests::reconcile::an_empty_admin_token_stops_before_any_request
        PASS [   0.004s] (52/98) temper-forge-setup tests::reconcile::the_bots_id_is_written_in_decimal_only_when_the_file_differs
     Summary [   0.043s] 98 tests run: 98 passed, 0 skipped
```

```
$ cargo nextest list -p temper-forge-setup -E 'test(/^tests::reconcile::/)'
directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
info: NEXTEST_DOUBLE_SPAWN=0 set, disabling double-spawn for test processes
temper-forge-setup tests::reconcile::a_bot_that_is_a_site_admin_is_refused
temper-forge-setup tests::reconcile::a_bot_with_the_marker_is_kept_and_nothing_is_written
temper-forge-setup tests::reconcile::a_draw_of_another_length_stops_before_creating_the_bot
temper-forge-setup tests::reconcile::a_forge_that_stops_answering_stops_the_run_at_its_step
temper-forge-setup tests::reconcile::a_lower_hook_met_later_deletes_the_higher_one_kept
temper-forge-setup tests::reconcile::a_missing_admin_token_stops_before_any_request
temper-forge-setup tests::reconcile::a_refused_request_stops_naming_its_step_and_subject
temper-forge-setup tests::reconcile::a_user_of_the_bots_login_without_the_marker_is_refused_naming_the_login
temper-forge-setup tests::reconcile::an_absent_bot_is_made_with_a_drawn_password_and_the_marker
temper-forge-setup tests::reconcile::an_admin_token_of_a_plain_user_stops_at_the_admin
temper-forge-setup tests::reconcile::an_empty_admin_token_stops_before_any_request
temper-forge-setup tests::reconcile::hook_urls_are_written_for_loopback_v4_and_v6
temper-forge-setup tests::reconcile::the_bots_id_is_written_in_decimal_only_when_the_file_differs
```

The following base scans corroborate the behavior deltas and direct callers.
The complete source was read for the empty-id case, token-status fallback,
limits' relation-only check, and serve -> prepare -> setup::check chain.

```
$ rg -n 'bytes::find|if user.is_admin|must_change_password|Refused\(401\)|Read\(Some\(secret\)\)|Some\(secret\) = value|!items.is_empty' crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile/tokens.rs crates/temper-forge-setup/src/reconcile/hooks.rs crates/temper-forge-setup/src/setup.rs crates/temper-forge-setup/src/tests/reconcile.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-forge-setup/src/tests/reconcile.rs:40:                must_change_password: false,
crates/temper-forge-setup/src/reconcile.rs:133:        if user.is_admin {
crates/temper-forge-setup/src/reconcile.rs:136:        if bytes::find(&user.full_name, b"temper:bot").is_none() {
crates/temper-forge-setup/src/reconcile.rs:235:                    must_change_password: false,
crates/temper-forge-setup/src/reconcile/hooks.rs:103:                    if let Some(secret) = value {
crates/temper-forge-setup/src/setup.rs:151:    valid(!items.is_empty(), p.clone(), ps);
crates/temper-forge-setup/src/reconcile/tokens.rs:99:                Event::Read(Some(secret)) => {
crates/temper-forge-setup/src/reconcile/tokens.rs:108:                Err(Why::Refused(401)) => Ok(self.remake(machine)),
```

```
$ rg -n 'setup::check|check::checked|fn (serve|prepare|reconcile_settings)|Reconcile::new' crates/temper/src/lib.rs crates/temper/src/setup.rs crates/temper/src/forge.rs crates/temper/src/box_command.rs crates/temper-forge-setup-shell/src/reconcile.rs tests/forge-setup/reconcile/src/world.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/src/world.rs:118:        let mut machine = Reconcile::new(&self.entry, self.listener);
crates/temper-forge-setup-shell/src/reconcile.rs:370:        machine: Reconcile::new(&settings.entry, settings.listener),
crates/temper/src/forge.rs:168:    let (named, checked, _) = super::setup::check(&bytes).map_err(refused)?;
crates/temper/src/forge.rs:224:pub fn reconcile_settings(
crates/temper/src/forge.rs:226:    checked: &temper_forge_setup::check::Checked,
crates/temper/src/setup.rs:54:    let checked = check::checked(&document, at, &members, named.webhooks.is_some(), &setup_overrides)
crates/temper/src/lib.rs:244:fn server_name(bytes: &[u8]) -> Result<skein_tls::Name, String> {
crates/temper/src/lib.rs:324:fn prepare(directory: &Path, bytes: &[u8], owned: &mut Vec<Fd>) -> Result<service::Service, String> {
crates/temper/src/lib.rs:325:    let (named, _, _) = setup::check(bytes)?;
crates/temper/src/lib.rs:370:pub fn prepare_in(directory: &Path) -> Result<service::Service, String> {
crates/temper/src/lib.rs:381:fn serve(path: &Path) -> Result<Exit, String> {
crates/temper/src/box_command.rs:11:    let (_, checked, limits) = super::setup::check(&bytes)?;
```

The signature sketch was checked again in a detached skein scratch at the
requested design base. It contains only the interfaces changed by the move:
Given, Place, Refused, Limits and their checks, marker/login/email,
Reconcile::new and bot fields, Orgs::new, and Tokens::new/name. Placeholder
bodies diverge; internal types are public only for compilation. It uses the
actual skein-lib Duration and workspace lints, with no lint exceptions.
Unchanged Item/Need and the future skein-forge paths were not resketched;
those imports/shared valid_word still need the ready check after skein 01.3.

```
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-04-answers-2 8938e16
directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 8938e16)
HEAD is now at 8938e16 Answer the forge kit planner's other questions
exit 0
```

The first scratch compile failed on scaffold lint details: missing Debug and
must_use attributes and untyped discarded placeholder arguments. Added the
source-style attributes and typed placeholder bindings, without changing a
signature or adding any production body. Selected diagnostics and the clean
rerun follow; the failed run is retained, not counted as a passing check.

```
$ cargo clippy -p skein-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04-answers-2
error: non-binding `let` without a type annotation
error: this function could have a `#[must_use]` attribute
error: non-binding `let` without a type annotation
error: this function could have a `#[must_use]` attribute
error: non-binding `let` without a type annotation
error: this function could have a `#[must_use]` attribute
error: non-binding `let` without a type annotation
error: non-binding `let` without a type annotation
error: non-binding `let` without a type annotation
error: this method could have a `#[must_use]` attribute
error: non-binding `let` without a type annotation
error: this method could have a `#[must_use]` attribute
error: non-binding `let` without a type annotation
error: this method could have a `#[must_use]` attribute
error: type does not implement `core::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: type does not implement `core::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: type does not implement `core::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: could not compile `skein-forge-setup` (lib) due to 17 previous errors
error: type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: type does not implement `std::fmt::Debug`; consider adding `#[derive(Debug)]` or a manual implementation
error: could not compile `skein-forge-setup` (lib test) due to 17 previous errors
```

```
$ cargo clippy -p skein-forge-setup --all-targets -- -D warnings
directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-04-answers-2
    Checking skein-forge-setup v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-04-answers-2/crates/skein-forge-setup)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.14s
```

```
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-04-answers-2
directory: /srv/data/git/runner/src/tmp/planning/skein-base
no output
exit 0; scratch removed
```

The mechanical inventory initially used a letters-only test-name expression,
which omitted an_admin_401_names_the_given_admin and reported 18 rather than
19 new tests in 04.3. The plan's count was correct. The corrected audit accepts
digits and also checks the line cap, new-line Sizes and absence of pending
marks. Both audit results are recorded below.

```
$ python3 -c 'from pathlib import Path; import re
p=Path("docs/development/plans/forge-kit/skein/04-setup.md"); s=p.read_text(); print(str(p)+": "+str(len(s.splitlines()))+" lines"); assert len(s.splitlines()) <= 400
counts=[]
for number,part in enumerate(s.split("### 04.")[1:],1):
 t=part.split("Test shape:",1)[1].split("Proof:",1)[0]
 names=set(re.findall(r"`([a-z]+(?:_[a-z]+)+)`",t))
 names.discard("an_empty_admin_token_stops_before_any_request")
 counts.append(len(names)); print("04."+str(number)+": "+str(len(names))+" new tests")
assert counts == [4,16,19], counts
print("Totals: 17, 33, 52; reconcile: 13 moved + 12 new = 25")
assert "pending the designer" not in s
print("No pending designer marks; all three new-line Sizes <= 300")'
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
Traceback (most recent call last):
  File "<string>", line 9, in <module>
    assert counts == [4,16,19], counts
           ^^^^^^^^^^^^^^^^^^^
AssertionError: [4, 16, 18]
docs/development/plans/forge-kit/skein/04-setup.md: 399 lines
04.1: 4 new tests
04.2: 16 new tests
04.3: 18 new tests
```

```
$ python3 -c 'from pathlib import Path; import re
p=Path("docs/development/plans/forge-kit/skein/04-setup.md"); s=p.read_text(); print(str(p)+": "+str(len(s.splitlines()))+" lines"); assert len(s.splitlines()) <= 400
counts=[]
for number,part in enumerate(s.split("### 04.")[1:],1):
 t=part.split("Test shape:",1)[1].split("Proof:",1)[0]
 names=set(re.findall(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+)+)`",t))
 names.discard("an_empty_admin_token_stops_before_any_request")
 counts.append(len(names)); print("04."+str(number)+": "+str(len(names))+" new tests")
assert counts == [4,16,19], counts
print("Totals: 17, 33, 52; reconcile: 13 moved + 12 new = 25")
assert "pending the designer" not in s
assert all(int(n) <= 300 for n in re.findall(r"Size: about [0-9]+ lines moved, ([0-9]+) new",s))
print("No pending designer marks; all three new-line Sizes <= 300")'
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
docs/development/plans/forge-kit/skein/04-setup.md: 399 lines
04.1: 4 new tests
04.2: 16 new tests
04.3: 19 new tests
Totals: 17, 33, 52; reconcile: 13 moved + 12 new = 25
No pending designer marks; all three new-line Sizes <= 300
```

No setup design issue remains unsettled. The previously recorded README
integration issue remains outside this two-file revision: its skein 04 row
puts skein 01.3 in “increments that need more,” whereas this session needs
its shared word check at start. The target behavior tests and complete kit
compile await implementation, as development.md 5.4's ready check requires.
Only skein/04-setup.md and this appended findings section are deliverables.

## Re-check rulings revision, 2026-10-19

This appendix supersedes the preceding planned counts and outstanding README
integration note. Inputs: `tmp/planning/reviews/skein-04b.txt` and its three
planner rulings, the earlier `skein-04.txt`, all three designer answer files,
skein-base `25d2551` and temper-base `dd12b8e`. Answers 23–29 and all earlier
fixes remain; there is no pending setup design question.

1. 04.3 adds two machine tests in `src/tests/steps.rs`, driven by replies
   using the shared Run fixtures. With owned(), an org read answered 404
   sends CreateOrg with person `ada`, name `work`, and ADMIN. With managed(),
   the same 404 stops with forge 1, Stage::Org, subject `acme`, OrgMissing.
   Their names reuse the base world's
   `an_owned_forge_makes_a_missing_org_for_its_person` and
   `a_managed_forge_missing_an_org_is_refused_naming_it`. These are two new
   machine tests, not moves of the world tests: skein 07.3 still copies
   those world tests with their names, budgets and times intact.
2. The header now matches the README row: temper 01.2 at session start;
   04.1 additionally needs skein 01.3's shared word check. The increment's
   Needs retains that check. No README change is required or made.
3. 04.1's Shape now explicitly names the forced fixture/test replacements:
   temper to kit in admin's login, person call arguments, kept_tokens and
   hook_listing bytes, team_spec's name and moved tests; File to Item in
   bot_lookup and kept_tokens; the machine retains given::marker(word)
   for both draw and id. Reply scripts and behavior stay as prescribed.

The session is 400 lines after wording reductions; all three increments,
their Sizes, the existing tests and their bounds remain. Revised counts:

| Increment | Moved tests | New tests | Running total | Size, moved/new production |
|---|---:|---:|---:|---|
| 04.1 | 13 | 4 | 17 | about 900 / 280 |
| 04.2 | 0 | 16 (4 given, 12 reconcile) | 33 | about 0 / 140 |
| 04.3 | 0 | 21 (18 steps, 3 limits) | 54 | about 0 / 0 |

04.3's Proof and Done when now require 54 passing tests. The reconcile
listing remains 25 (13 moved + 12 new): the two added tests live in steps.
These remain planned kit counts, not an executed target suite. The existing
98-test base result and its 13 moved tests' timings remain the evidence.

No signature or prescribed Proof/Done-when command changed. The earlier
scratch clippy evidence still applies; no scratch worktree was needed in
this pass. Compilation against the future skein-forge imports and execution
of the new machine tests remain implementation/ready-check work.

The graph search for Reconcile/Orgs/Run and the affected helpers, in project
`srv-data-git-runner-src-temper`, scoped to `crates/temper-forge-setup/.*`,
returned zero results. This is insufficient coverage, not absent source.
rg and direct reads at temper-base verified the names and branches:
Orgs::event handles Status(404) by taking owner or OrgMissing and sending
CreateOrg; the fixtures contain the named helpers and the world contains
both test names. No project list/index/status operation was used.

Both revision checks ran directly; output follows in full.

```
$ rg -n 'fn (managed|owned|hook_listing|bot_lookup|kept_tokens|admin|person|team_spec|draw|id)\b|fn (an_owned_forge_makes_a_missing_org_for_its_person|a_managed_forge_missing_an_org_is_refused_naming_it)|CreateOrg \{|ok_or\(Why::OrgMissing\)' crates/temper-forge-setup/src/tests/fixtures.rs crates/temper-forge-setup/src/reconcile.rs crates/temper-forge-setup/src/reconcile/orgs.rs tests/forge-setup/reconcile/tests/reconcile.rs
directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/forge-setup/reconcile/tests/reconcile.rs:70:fn an_owned_forge_makes_a_missing_org_for_its_person() {
tests/forge-setup/reconcile/tests/reconcile.rs:79:fn a_managed_forge_missing_an_org_is_refused_naming_it() {
crates/temper-forge-setup/src/reconcile/orgs.rs:84:                    let person = self.owner.clone().ok_or(Why::OrgMissing)?;
crates/temper-forge-setup/src/reconcile/orgs.rs:86:                    Ok(machine.send(Request::CreateOrg { person, name: self.name().into() }))
crates/temper-forge-setup/src/tests/fixtures.rs:64:pub fn owned() -> crate::setup::Owned {
crates/temper-forge-setup/src/tests/fixtures.rs:139:pub fn draw(value: u8) -> skein_lib::Secret {
crates/temper-forge-setup/src/tests/fixtures.rs:190:pub fn admin() -> temper_forge_forgejo::document::User {
crates/temper-forge-setup/src/tests/fixtures.rs:198:pub fn person(login: &[u8], full_name: &[u8]) -> temper_forge_forgejo::document::User {
crates/temper-forge-setup/src/tests/fixtures.rs:211:    pub fn managed() -> Self {
crates/temper-forge-setup/src/tests/fixtures.rs:214:    pub fn owned() -> Self {
crates/temper-forge-setup/src/tests/fixtures.rs:233:    pub fn bot_lookup(&mut self) {
crates/temper-forge-setup/src/tests/fixtures.rs:275:    pub fn kept_tokens(&mut self) {
crates/temper-forge-setup/src/tests/fixtures.rs:331:    pub fn hook_listing() -> Self {
crates/temper-forge-setup/src/tests/fixtures.rs:352:pub fn team_spec() -> temper_forge_forgejo::document::TeamSpec {
crates/temper-forge-setup/src/reconcile.rs:132:    fn id(&mut self, user: temper_forge_forgejo::document::User) -> Need {
crates/temper-forge-setup/src/reconcile.rs:204:    fn admin(&mut self, event: Event) -> Need {
crates/temper-forge-setup/src/reconcile.rs:226:    fn draw(&mut self, event: Event) -> Need {
```

```
$ python3 -c 'from pathlib import Path
import re
p=Path("docs/development/plans/forge-kit/skein/04-setup.md")
s=p.read_text()
print(f"{p}: {len(s.splitlines())} lines")
assert len(s.splitlines()) <= 400
counts=[]
for number,part in enumerate(s.split("### 04.")[1:],1):
 t=part.split("Test shape:",1)[1].split("Proof:",1)[0]
 names=set(re.findall(r"`([a-z][a-z0-9]*(?:_[a-z0-9]+)+)`",t))
 names.discard("an_empty_admin_token_stops_before_any_request")
 counts.append(len(names))
 print(f"04.{number}: {len(names)} new tests")
assert counts == [4,16,21], counts
assert "**0 moved, 21 new**, 54 total" in s
assert "these 18 tests" in s
assert "shows 54 passing" in s and "prints 54 passed" in s
assert "prints 25 tests" in s
assert "**04.1 also needs skein 01.3**" in s
assert "pending the designer" not in s
assert all(int(n)<=300 for n in re.findall(r"Size: about [0-9]+ lines moved, ([0-9]+) new",s))
print("Totals: 17, 33, 54; reconcile: 13 moved + 12 new = 25")
print("Header matches README; no pending answers; all new-line Sizes <= 300")
'
directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
docs/development/plans/forge-kit/skein/04-setup.md: 400 lines
04.1: 4 new tests
04.2: 16 new tests
04.3: 21 new tests
Totals: 17, 33, 54; reconcile: 13 moved + 12 new = 25
Header matches README; no pending answers; all new-line Sizes <= 300
```

No issue remains unsettled by this re-check. Only the session and this
findings appendix changed; nothing was committed or landed.

