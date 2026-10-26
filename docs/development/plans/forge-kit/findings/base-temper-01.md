# temper 01: base evidence

2026-10-19. Base temper dd12b8e (code of 27995fb75), skein 25d2551
(code of d120820); source roots tmp/planning/{temper-base,skein-base}.
Only temper/01-forge.md and this findings file persist. No commit or landing.
All prescribed commands below ran directly, with unoptimized builds and
NEXTEST_DOUBLE_SPAWN=0. Queued waits are excluded from nextest Summary times.

## Discovery and boundaries

- Code graph project srv-data-git-runner-src-temper: broad search_graph for
  Plan/Settings/Forge/host/start_request/drive_starts included temper.old.
  The current component's qualified namespace returned zero nodes. No index
  operation was used. rg in the specified base verified each relied-on name.
- The vocabulary ends at Collaborator in api.rs. The answer checker starts
  at Shape and remains client-private. lib.rs's answer_bytes_unbounded wraps
  bounds.rs; it is not in the vocabulary part of api.rs. It needs the byte
  walkers and has seven named occurrences in temper, zero in the other repos.
- **Draft inconsistency needing outer-plan reconciliation:** skein/01-forge.md
  says no sizing helper moves in 01.1; code-organization.md and decisions.md (decision 4)
  still put the unbounded counter there. The current skein/02-client.md
  now agrees with skein 01 that all four counters remain through 02.1. This session follows
  the current skein 01 interface and leaves the counter until the measure step.
  No new design decision was made: measure ownership is already decision 24.
- **Start extraction:** drive_starts includes scheduling, not just dialect
  mapping. Skein 01.4 explicitly copies no scheduling. Temper must retain the
  bounded scheduling loop and replace only request selection/answer mapping.
- **Host count:** four references: service forge.rs call, binary forge.rs call,
  component tests.rs call, component settings.rs comment. The component has
  no production call to conventions::host at the base. Source helper is 58
  lines plus imports/constants; no revision.rs exists (reading::revision does).
- **Private hex helper:** three call-site lines: conventions.rs one and
  hooks/connection.rs two. The latter cannot import skein's private marker::hex.
  The same checked index into b"0123456789abcdef" stays locally at those uses.
- Hooks::drive and Hooked::pump borrow enclosing Settings rather than adding an
  eighth argument; verified borrows its word. Every constructor is in the
  session's Files, including the protocol world and conformance.
- Skein 03's current draft expects 112 inherited component tests: the four
  pure marker/table tests and one pure Host test moved before it. Its new
  Host wiring test is its own. This session removes those five copies.
- No additional dependency beyond README's start and per-increment Needs.
  Existing tools-plan start prerequisite remains necessary (repin failure below).

## User counts at the base

The namespace search counts matching Rust source lines, not occurrences or
production lines. The client has **15 dependent Cargo manifests** and 193
lines spelling its crate name. Alias-qualified vocabulary paths add work:
652 lines match client::api (including forge_client::api and qualified names).
The query also searches client-local crate::api/crate::locate, whose answer
checker must be distinguished from the moved vocabulary.

| Client dependent path | crate-name lines |
|---|---:|
| crates/temper-engine-protocol-forge | 32 |
| crates/temper-engine-smith | 22 |
| crates/temper-protocol-store | 21 |
| crates/temper-engine-domain-forge | 21 |
| crates/temper-engine-domain | 20 |
| tools/forgejo-conformance | 13 |
| tests/engine/protocol-forge | 13 |
| tests/engine/domain | 10 |
| tests/conformance | 10 |
| crates/temper-service | 10 |
| tests/engine/forge/client | 8 |
| tests/engine/forge | 6 |
| tests/system | 5 |
| tests/codecs | 1 |
| tests/client | 1 |

| Dialect dependent path | Rust lines |
|---|---:|
| crates/temper | 2 |
| crates/temper-forge-setup-shell | 7 |
| testing/temper-fake-forge-protocol | 38 |
| testing/temper-throwaway-forgejo | 2 |
| tests/codecs | 40 |
| tests/forge-setup/reconcile | 12 |
| crates/temper-engine-protocol-forge | 48 |
| crates/temper-forge-setup | 30 |
| tests/engine/protocol-forge | 1 |

Qualified client/dialect names: 372 matching lines in temper; 0 in skein,
jig, smith, hearth. Other broad-name hits were inspected: skein's marker
is core/std::marker::PhantomData; jig's plan/tables hits are UI task plans;
smith's start_request hit is a local-service test name, not a forge caller.
Generic Settings matches are unrelated outside temper. No upstream consumer
or cross-repository edit was found.

Production prefix matches: engine-domain 14, domain-forge 289, client 20,
protocol-forge 15, engine-smith 2, protocol-store 7, conformance 12.
The broad crate::api query also hits 22 fake-domain lines: those are its own
vocabulary and do not change. Additional direct imports/manifests put 01.1's
production path-edit estimate at about 400, not 193. Tests are excluded from
Size. The dialect count's 180 includes tests; it is a conservative user-edit
estimate, while the 9,300 deleted source lines are Removes, not Size.

## Outcomes that must not be called passes

1. README's literal src/rust binary paths do not exist on this machine. The
   prescribed command failed its first process test (prepare could not start).
   This workspace uses src/tmp; the corrected absolute paths are in 01.4.
2. The corrected process run failed a_running_engine_lends_the_record_temper_sign_in_kept:
   sign-in world deadline at 30.166 s, stopped false, issuer deadline None.
   Five preceding tests passed; 13 did not run. Summary 154.170 s, exit 100.
   No further process rerun or source change was made. Earlier base-runs.md's
   19/19 in 145.694 s remains historical evidence, not this draft's result.
   The failure is outside the assigned forge extraction; outer planning must
   resolve the baseline before claiming the plan's proof passes.
3. cargo update moved all 29 skein packages to 25d2551, but its workspace check
   failed in temper-worker-protocol-git/src/files.rs's two exhaustive Op matches,
   missing Op::Lock. The base still pins smith 29fa97c9. The tools plan's first
   repin must land before 01.1; its readiness check must include the workspace
   check, not only the smith hash. No unrequested pin repair was made here.

## Scratch compilation

Created with git worktree add --detach at dd12b8e, under scratch-temper-01.
After the repin experiment, restored that scratch Cargo.lock to the base.
Compiled public shape declarations for Settings at 01.3 and 01.4, Forge,
Hooks::drive, Hooked::pump/verified and start::heard; method signatures were
expressed as traits so no invented production body was needed.
A scratch-only skein-forge crate supplied the exact planned Credential and
start::Heard declarations. They have no implementation: this checks the new
signatures' types and lints, not linkage against future skein code. The ready
check must compile against the landed upstream interfaces. Only the component
package was selected (-p), all targets, warnings denied. No lint exemption.
The worktree was removed after recording the evidence.

## Runs

Each block contains at most 30 lines. Whole output remains in the named
log; excerpts retain Summary/Finished/errors. Count tables above expand the
long user-search output without repeating its command.

### Gate

```text
$ scripts/check.sh
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 37.80s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Summary [   7.184s] 1879 tests run: 1879 passed (1 slow), 1 skipped
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.26s
     Summary [  25.879s] 50 tests run: 50 passed (1 slow), 0 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013015-733.log

### Source test inventory

```text
$ cargo nextest list -p temper-engine-domain-forge-client -p temper-forge-forgejo -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
temper-engine-domain-forge-client=76, temper-engine-protocol-forge=117, temper-forge-forgejo=84
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013225-742.log

### 01.1 callers

```text
$ cargo nextest run -p temper-engine-domain-forge-client -p temper-engine-domain-forge -p temper-engine-domain -p temper-engine-smith -p temper-protocol-store -p temper-service -p temper-engine-forge-client-world -p temper-engine-forge-world -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world -p temper-engine-domain-world -p temper-conformance-world -p temper-codecs-world -p temper-client-world -p temper-system-world -p forgejo-conformance
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 37.51s
     Summary [   5.821s] 997 tests run: 997 passed, 0 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013225-743.log

### 01.2 callers

```text
$ cargo nextest run -p temper -p temper-forge-setup-shell -p temper-fake-forge-protocol -p temper-throwaway-forgejo -p temper-codecs-world -p temper-forge-setup-reconcile-world -p temper-engine-protocol-forge -p temper-forge-setup
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 36.84s
     Summary [   3.156s] 446 tests run: 446 passed, 1 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013226-744.log

### 01.3 callers

```text
$ cargo nextest run -p temper-engine-protocol-forge -p temper-service -p temper-engine-protocol-forge-world
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.28s
     Summary [   1.600s] 376 tests run: 376 passed, 0 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013315-748.log

### Component world fuzzy

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.30s
     Summary [   7.027s] 6 tests run: 6 passed, 0 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013315-749.log

### 01.4 callers

```text
$ cargo nextest run -p temper -p temper-forge-setup-shell -p temper-service -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world -p forgejo-conformance
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.50s
     Summary [   2.934s] 483 tests run: 483 passed, 0 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013318-750.log

### README process path

```text
$ TEMPER_FORGEJO=$HOME/src/rust/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=$HOME/src/rust/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.43s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    thread 'a_change_lands_by_deliveries_with_every_poll_raised' (3123011) panicked at crates/temper/tests/support/owned.rs:382:5:
    assertion `left == right` failed: temper forge prepare: forge 1: command could not start: Other
     Summary [   0.034s] 1/19 tests run: 0 passed, 1 failed, 0 skipped
error: test run failed
exit 100
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013235-751.log

### Process at this machine's binary paths

```text
$ TEMPER_FORGEJO=/srv/data/git/runner/src/tmp/temper/forgejo/v17.0.0/forgejo-17.0.0-linux-amd64 TEMPER_FORGEJO_RUNNER=/srv/data/git/runner/src/tmp/temper/forgejo/runner-v13.2.0/forgejo-runner-13.2.0-linux-amd64 scripts/proof-one-process.sh
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.13s
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
    thread 'a_running_engine_lends_the_record_temper_sign_in_kept' (3134183) panicked at crates/temper/tests/support/sign_in.rs:277:9:
    sign-in world deadline; stopped: false; empty hosts: [true, true, false, true, false, false, false]; issuer deadline: None; stderr: 
     Summary [ 154.170s] 6/19 tests run: 5 passed (4 slow), 1 failed, 0 skipped
error: test run failed
exit 100
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013326-733.log

### Repin update in a disposable base checkout

```text
$ cargo update -p skein-lib
Directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-01
     Locking 29 packages to latest Rust 1.95 compatible versions
    Updating skein-lib v0.0.0 (https://github.com/forgetic/skein.git#8a51e10b) -> #25d2551e
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013312-745.log

### Check after the isolated update

```text
$ cargo check --workspace --all-targets
Directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-01
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
error[E0004]: non-exhaustive patterns: `skein_io::kernel::Op::Lock { .. }` not covered
error: could not compile `temper-worker-protocol-git` (lib) due to 2 previous errors
exit 101
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013602-735.log

### Unmodified base workspace check

```text
$ cargo check --workspace --all-targets
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 38.52s
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013855-736.log

### Compiled interface sketches

```text
$ cargo clippy -p temper-engine-protocol-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-temper-01
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.83s
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013935-748.log

### Namespace/users inventory

```text
$ python3 -c 'import subprocess,tomllib
from pathlib import Path
from collections import Counter
root=Path(".")
patterns={"client":"temper_engine_domain_forge_client","dialect":"temper_forge_forgejo","vocabulary":"client::api"}
for label,pat in patterns.items():
 r=subprocess.run(["rg","-n",pat,"--glob","*.rs","crates","testing","tests","tools"],capture_output=True,text=True)
 counts=Counter()
 for line in r.stdout.splitlines():
  p=Path(line.split(":",1)[0]).parent
  while not (p/"Cargo.toml").exists(): p=p.parent
  counts[tomllib.loads((p/"Cargo.toml").read_text())["package"]["name"]]+=1
 print(label+" total="+str(sum(counts.values()))+" "+", ".join(k+"="+str(v) for k,v in sorted(counts.items())))
for pat in ["conventions::host","marker::","start_request|drive_starts|start::Heard","plan::|tables::|reading::|connection::(Fields|Heard|Body|Failure)","Settings \\{|settings::Settings \\{|settings::Forge \\{","answer_bytes_unbounded"]:
 print("PATTERN "+pat)
 for repo in [root,Path("../skein-base"),Path("../../../jig"),Path("../../../smith"),Path("../../../hearth")]:
  r=subprocess.run(["rg","-n",pat,"--glob","*.rs","--glob","!target/**",str(repo)],capture_output=True,text=True)
  print(str(repo)+": "+str(len(r.stdout.splitlines())))
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
client total=193 forgejo-conformance=13, temper-client-world=1, temper-codecs-world=1, temper-conformance-world=10, temper-engine-domain=20, temper-engine-domain-forge=21, temper-engine-domain-world=10, temper-engine-forge-client-world=8, temper-engine-forge-world=6, temper-engine-protocol-forge=32, temper-engine-protocol-forge-world=13, temper-engine-smith=22, temper-protocol-store=21, temper-service=10, temper-system-world=5
dialect total=180 temper=2, temper-codecs-world=40, temper-engine-protocol-forge=48, temper-engine-protocol-forge-world=1, temper-fake-forge-protocol=38, temper-forge-setup=30, temper-forge-setup-reconcile-world=12, temper-forge-setup-shell=7, temper-throwaway-forgejo=2
vocabulary total=652 forgejo-conformance=12, temper-client-world=1, temper-conformance-world=16, temper-engine-domain=86, temper-engine-domain-forge=347, temper-engine-domain-world=26, temper-engine-forge-client-world=2, temper-engine-forge-world=37, temper-engine-protocol-forge=24, temper-engine-protocol-forge-world=20, temper-engine-smith=7, temper-protocol-store=62, temper-service=8, temper-system-world=4
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013312-746.log

### Qualified users and production size

```text
$ python3 -c 'import subprocess,tomllib,re
from pathlib import Path
from collections import Counter
root=Path(".")
patterns=["temper_engine_domain_forge_client|temper_forge_forgejo","client::api|crate::api|crate::locate","conventions::host","marker::hex","start_request|drive_starts|start::Heard","answer_bytes_unbounded"]
for pat in patterns:
 values=[]
 for repo in [".","../skein-base","../../../jig","../../../smith","../../../hearth"]:
  r=subprocess.run(["rg","-n",pat,"--glob","*.rs",repo],capture_output=True,text=True)
  values.append(repo+"="+str(len(r.stdout.splitlines())))
 print(pat+" : "+" ".join(values))
r=subprocess.run(["rg","-n","client::api|crate::api|crate::locate","--glob","*.rs","crates","testing","tests","tools"],capture_output=True,text=True)
counts=Counter()
for line in r.stdout.splitlines():
 f,n,_=line.split(":",2); p=Path(f); s=p.read_text(); i=int(n)
 if "tests" in p.parts or p.stem=="tests" or (s.find("#[cfg(test)]")>=0 and i>len(s[:s.find("#[cfg(test)]")].splitlines())): continue
 q=p.parent
 while not (q/"Cargo.toml").exists(): q=q.parent
 counts[str(q)]+=1
print("production path lines: "+", ".join(k+"="+str(v) for k,v in sorted(counts.items())))
for prefix in ["crates/temper-engine-domain-forge-client/src/api.rs","crates/temper-engine-domain-forge-client/src/locate.rs","crates/temper-forge-forgejo/src"]:
 p=Path(prefix); files=[p] if p.is_file() else list(p.rglob("*.rs")); print(prefix+" total lines="+str(sum(len(f.read_text().splitlines()) for f in files)))
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
production path lines: crates/temper-engine-domain=14, crates/temper-engine-domain-forge=289, crates/temper-engine-domain-forge-client=20, crates/temper-engine-protocol-forge=15, crates/temper-engine-smith=2, crates/temper-protocol-store=7, testing/temper-fake-forge-domain=22, tools/forgejo-conformance=12
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013856-739.log

### Done when: old paths

```text
$ rg -n 'temper_forge_forgejo|temper-forge-forgejo|temper_engine_domain_forge_client::api|client::api|conventions::host' Cargo.toml crates testing tests tools -g '*.rs' -g Cargo.toml
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
testing/temper-throwaway-forgejo/src/lib.rs:10:pub use temper_forge_forgejo::pin::{RUNNER_VERSION, VERSION, names_runner};
testing/temper-throwaway-forgejo/src/binary.rs:8:use temper_forge_forgejo::pin::names_forgejo;
Cargo.toml:20:    "crates/temper-forge-forgejo",
847 matching source/manifest lines at the base; expected to become zero
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013940-749.log

### Done when: deletion

```text
$ python3 -c 'from pathlib import Path; print("dialect_copy_absent=" + str(not Path("crates/temper-forge-forgejo").exists()))'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
dialect_copy_absent=False
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T013942-751.log

## Package totals and moved test times

Times below are sums of nextest PASS durations, not elapsed suite times.
Build-lane package groups ran under concurrent load; they are evidence, not
new per-test ceilings or replacements for base-runs.md's isolated times.

20261019T013225-743.log:

- forgejo-conformance: 25 passed, summed 0.287 s.
- temper-client-world: 37 passed, summed 2.112 s.
- temper-codecs-world: 60 passed, summed 2.213 s.
- temper-conformance-world: 30 passed, summed 10.013 s.
- temper-engine-domain: 14 passed, summed 0.122 s.
- temper-engine-domain-forge: 22 passed, summed 0.171 s.
- temper-engine-domain-forge-client: 76 passed, summed 0.578 s.
- temper-engine-domain-world: 201 passed, summed 19.891 s.
- temper-engine-forge-client-world: 29 passed, summed 0.234 s.
- temper-engine-forge-world: 54 passed, summed 0.511 s.
- temper-engine-protocol-forge: 117 passed, summed 1.497 s.
- temper-engine-protocol-forge-world: 47 passed, summed 1.858 s.
- temper-engine-smith: 12 passed, summed 0.075 s.
- temper-protocol-store: 45 passed, summed 0.289 s.
- temper-service: 212 passed, summed 4.666 s.
- temper-system-world: 16 passed, summed 1.109 s.

20261019T013226-744.log:

- temper: 58 passed, summed 9.570 s.
- temper-codecs-world: 60 passed, summed 2.894 s.
- temper-engine-protocol-forge: 117 passed, summed 3.371 s.
- temper-fake-forge-protocol: 55 passed, summed 0.816 s.
- temper-forge-setup: 98 passed, summed 2.056 s.
- temper-forge-setup-reconcile-world: 21 passed, summed 1.249 s.
- temper-forge-setup-shell: 24 passed, summed 3.015 s.
- temper-throwaway-forgejo: 13 passed, summed 0.203 s.

20261019T013315-748.log:

- temper-engine-protocol-forge: 117 passed, summed 1.304 s.
- temper-engine-protocol-forge-world: 47 passed, summed 1.461 s.
- temper-service: 212 passed, summed 9.764 s.

20261019T013318-750.log:

- forgejo-conformance: 25 passed, summed 0.224 s.
- temper: 58 passed, summed 6.080 s.
- temper-engine-protocol-forge: 117 passed, summed 2.392 s.
- temper-engine-protocol-forge-world: 47 passed, summed 2.871 s.
- temper-forge-setup-shell: 24 passed, summed 2.131 s.
- temper-service: 212 passed, summed 9.472 s.

The four copied marker/table tests in the 01.3 caller run took 0.004,
0.009, 0.004 and 0.003 s in session-file order; Host took 0.003 s.
The historical source-package run in base-skein-01.md was 0.003, 0.006,
0.005, 0.003 s; Host 0.002 s. No test name or budget is changed by a move.

## Final consumer audit corrections

Temper has an explicit workspace member for crates/temper-forge-forgejo;
01.2 deletes that line (the crates/* glob belongs to skein, not temper).
The throwaway re-exports VERSION/RUNNER_VERSION/names_runner. Seven indirect
use-site lines also change: its own tests/throwaway.rs (1), temper's
tests/support/owned.rs (1) and tests/process.rs (2), conformance's
src/runner.rs (1), src/registration.rs (1), src/tests.rs (1): seven lines,
plus the throwaway's re-export itself. These Files and conformance tests
are now explicitly in 01.2; no additional upstream dependency.
The new caller command below supersedes the earlier 446-test selection.
Client facts.rs also imports api::Error. The retained checker remains
crate::api::accepts (calls.rs plus six test assertions); it must not be
confused with the vocabulary imports changed to skein_forge::api.

### Final 01.2 callers

```text
$ cargo nextest run -p temper -p temper-forge-setup-shell -p temper-fake-forge-protocol -p temper-throwaway-forgejo -p temper-codecs-world -p temper-forge-setup-reconcile-world -p temper-engine-protocol-forge -p temper-forge-setup -p forgejo-conformance
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.19s
     Summary [   0.673s] 471 tests run: 471 passed, 1 skipped
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T014443-734.log

### Indirect pin users

```text
$ python3 -c 'import subprocess; roots=['\''.'\'','\''../skein-base'\'','\''../../../jig'\'','\''../../../smith'\'','\''../../../hearth'\'']; pats=['\''temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner)'\'','\''temper-forge-forgejo'\'']; [(print(p+'\'' '\''+r+'\'': '\''+str(len(subprocess.run(['\''rg'\'','\''-n'\'',p,'\''--glob'\'','\''*.rs'\'','\''--glob'\'','\''Cargo.toml'\'',r],capture_output=True,text=True).stdout.splitlines())))) for p in pats for r in roots]'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner) .: 7
temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner) ../skein-base: 0
temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner) ../../../jig: 0
temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner) ../../../smith: 0
temper_throwaway_forgejo.*(VERSION|RUNNER_VERSION|names_runner) ../../../hearth: 0
temper-forge-forgejo .: 12
temper-forge-forgejo ../skein-base: 0
temper-forge-forgejo ../../../jig: 0
temper-forge-forgejo ../../../smith: 0
temper-forge-forgejo ../../../hearth: 0
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T014444-735.log

Scratch lifecycle (no commit):

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-temper-01 dd12b8e
Directory: /srv/data/git/runner/src/temper
HEAD is now at dd12b8ed9 Check reconcile's given values in temper-forge-setup
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-temper-01
Directory: /srv/data/git/runner/src/temper
exit 0; worktree removed
```

## File validation

```text
$ python3 -c 'from pathlib import Path; import re; p=Path("/srv/data/git/runner/src/tmp/planning/temper-forge-kit/docs/development/plans/forge-kit"); s=(p/"temper/01-forge.md").read_text(); f=(p/"findings/base-temper-01.md").read_text(); assert len(s.splitlines())<=400; parts=re.split(r"^### 01\.[1-4] ",s,flags=re.M)[1:]; assert len(parts)==4; assert all(all(x+":" in part for x in ["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]) for part in parts); sizes=re.findall(r"Size: about [\d,]+ lines moved, (\d+) new",s); assert len(sizes)==4 and all(int(x)<=300 for x in sizes); assert "373" in s and "479" in s and "471" in s; assert all(len(block.splitlines())<=30 for block in re.findall(r"\x60\x60\x60text\n(.*?)\x60\x60\x60",f,re.S)); print("4 increments; session "+str(len(s.splitlines()))+" lines; complete fields; new lines <=300; counts 997/471/373/479; evidence blocks <=30 lines")'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
4 increments; session 314 lines; complete fields; new lines <=300; counts 997/471/373/479; evidence blocks <=30 lines
exit 0
```

Log: /srv/data/git/runner/src/tmp/logs/20261019T014645-733.log

The base worktree remains clean. The scratch worktree is absent. No commit or
landing was made. Only this findings file and temper/01-forge.md were written
by this session; other planning sessions' files were read, not changed.
