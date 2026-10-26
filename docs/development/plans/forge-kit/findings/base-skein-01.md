# skein 01: base evidence

2026-10-19. Source: temper `27995fb75`, skein `d120820`.
Base directories: `/srv/data/git/runner/src/tmp/planning/temper-base` and
`/srv/data/git/runner/src/tmp/planning/skein-base`. All prescribed cargo,
count and user-search commands go directly. No optimized test build.
The primary checkouts have the same source commits; jig, smith and hearth
are searched read-only. Only this findings file and skein/01-forge.md persist.

## Discovery and corrections to the surveys

- Graph project `srv-data-git-runner-src-temper`, search_graph scoped to
  temper-engine-protocol-forge, returned no symbols; the wider search mixed
  current and temper.old paths. Direct rg and named-file reads verified the
  base. The skein graph located sha256/Secret/Writer; rg confirmed their
  definitions in skein-base before sketching against them.
- api.rs's vocabulary ends with Collaborator. answer_bytes_unbounded is
  lib.rs's wrapper over bounds.rs, not part of api.rs at the base. It needs
  bytes, optional, rows, summary, files, answer_heap and pull. None needs
  client Limits; keep the heap measure unchanged until skein 02.2.
- locate.rs contains no tests. Both tests::locate functions drive Harness
  and Domain, so neither can move unchanged with the vocabulary. 01.1 moves
  **zero** tests and writes two pure tests; both existing locate names stay
  for skein 02.1. This corrects survey-01's apparent two-test move.
- revision is a function in reading.rs; there is no revision.rs. Its long
  input is length:u64 big-endian followed by updated nanos:u64 big-endian,
  absent time 0. Its first digest word is the first eight SHA-256 bytes BE.
- Four pure top-level protocol tests can move unchanged with marker/tables.
  The 68 in reads (27), writes (18), contents (23) stay for skein 03.1 as
  decisions 8 requires, even though reads contains pure conversion tests.
- page_size/largest_row and Listing remain the component's. One selected
  page_size per call suffices: PullFor uses Items, Checks Statuses,
  PullFiles PullFiles. Internal lookups at limit 1 remain literal.
- Fields and Failure accompany Heard/Body; RepositoryEntry/UserEntry must
  become public for limits::worst_case, and marker::hex for conventions::host.
- Additional Needs: **temper 01.2** for skein 01.3, because README/decision 2
  says a copied source already uses skein's earlier vocabulary and dialect.
  README's later-Needs cell is empty; only this session file is authorized
  to change, so the outer planner must reconcile that table.
- Pending recommendations remain marked at their implementation sites:
  Design questions 7, 9, 10, 12. Questions 8 and 11 change no code.
  Drafting does not resolve them or authorize a landing with open answers.

## Base proofs

Future `cargo nextest run -p skein-forge` has no base crate. Its base
forms are these three source crates, each run once. Future cumulative counts
are 2 / 86 / 90 / 97 / 120 for 01.1–01.5; the new tests are planned, not run
at the base. 01.1 moves 0, 01.2 moves 84, 01.3 moves 4, 01.4/01.5 move 0.

```text
$ cargo nextest run -p temper-forge-forgejo
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.14s
     Summary [   0.048s] 84 tests run: 84 passed, 0 skipped
```

```text
$ cargo nextest run -p temper-engine-domain-forge-client
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Summary [   0.084s] 76 tests run: 76 passed, 0 skipped
```

```text
$ cargo nextest run -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.15s
     Summary [   0.131s] 117 tests run: 117 passed, 0 skipped
```

The earlier whole-plan baseline was 0.062 / 0.051 / 0.149 s respectively
(findings/base-runs.md). This run's dialect tests took 0.001–0.008 s each;
the four moved protocol tests took 0.003, 0.006, 0.005, 0.003 s (0.017 s
summed), in session-file order. No changed test budget or build profile.
The client suite's maximum was 0.019 s. The protocol run had five tests at
or above 0.020 s, all staying with the component; these are reported, not
filtered or made cheaper by this extraction:

- tests::contents::checks_leave_the_job_absent_when_the_run_or_index_is_missing: 0.024 s.
- tests::contents::raw_sides_at_the_answer_bound_fit_and_one_byte_more_is_too_large: 0.022 s.
- tests::reads::revisions_and_provenance_follow_the_body_and_its_times: 0.024 s.
- tests::writes::every_failure_of_a_merges_read_is_timeout: 0.023 s.
- tests::writes::straightforward_writes_use_their_methods_and_literal_bodies: 0.020 s.

## Test inventory

```text
$ cargo nextest list -p temper-forge-forgejo -p temper-engine-domain-forge-client -p temper-engine-protocol-forge --message-format json
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished test profile [unoptimized + debuginfo]
"test-count":277
All three rust-suites have "status":"listed".
```

Parsed from each suite's testcase keys (prefix before the final ::):

| Source crate | Module | Tests at the base |
|---|---|---:|
| temper-engine-domain-forge-client | tests | 31 |
| temper-engine-domain-forge-client | tests::keep | 19 |
| temper-engine-domain-forge-client | tests::locate | 2 |
| temper-engine-domain-forge-client | tests::outbox | 24 |
| temper-engine-protocol-forge | tests | 17 |
| temper-engine-protocol-forge | tests::connection | 20 |
| temper-engine-protocol-forge | tests::contents | 23 |
| temper-engine-protocol-forge | tests::hooks | 12 |
| temper-engine-protocol-forge | tests::reads | 27 |
| temper-engine-protocol-forge | tests::writes | 18 |
| temper-forge-forgejo | tests | 37 |
| temper-forge-forgejo | tests::accounts | 9 |
| temper-forge-forgejo | tests::delivery | 15 |
| temper-forge-forgejo | tests::hooks | 7 |
| temper-forge-forgejo | tests::orgs | 7 |
| temper-forge-forgejo | tests::pin | 4 |
| temper-forge-forgejo | tests::tokens | 5 |

The four moving protocol names are a_marker_round_trips_its_key_and_keeps_the_text,
a_body_without_a_well_formed_marker_has_no_key,
a_full_table_drops_the_entry_used_longest_ago, and
a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place. The
locate pair remains a_locate_past_the_call_bound_is_refused_too_large and
a_located_repository_zero_is_an_invalid_answer. Qualified module prefixes
change only where the move requires; function names stay.

## Scratch sketch

Scratch worktree: /srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge.
Created detached from d120820. It contains copied vocabulary/dialect types,
the extracted sizing helpers, table and exchange types, and the signatures
shown in 01.3/01.4: Bounds, Plan, marker functions, reading's word arguments,
hint, Credential, Forgejo authorization and the start Plan/Read/Step/Heard.
Signature-only bodies use unreachable!("signature sketch"); they are not
production implementations and no behavioural result is claimed for them.
The forced sha256 conversion is compiled with the original revision body.
The four reference test bodies also compile; the start document includes
both max_response_items and default_paging_num. Sketch results follow below.

## Queue note

The build lane was paused by exclusive gate task 771 while task 746
(python3 -, owned by another session) waited for stdin. Our queued script
attempts 763 and 788 were removed before they ran: queues commands,
not shell stdin. Their waiting wrappers exited 1 with "did not finish" (0 s run time).
Their replacements pass the source with python3 -c and passed.
No missing run is described as passing. All replacement results appear below.

## Clippy at the source base

```text
$ cargo clippy -p temper-forge-forgejo -p temper-engine-domain-forge-client -p temper-engine-protocol-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Checking temper-forge-forgejo v0.0.0 (/srv/data/git/runner/src/tmp/planning/temper-base/crates/temper-forge-forgejo)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.10s
```

## Sketch compile, first pass

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge
error: mod.rs files are not allowed (forgejo/mod.rs and forgejo/plans/mod.rs)
error: public pure functions need #[must_use]
error: non-binding let without a type annotation (signature stubs)
error: long literal lacking separators (reference test)
error: could not compile skein-forge (lib test) due to 26 previous errors
```

Fixed the plan and sketch: forgejo.rs and forgejo/plans.rs; must_use on
newly public pure functions; typed scratch-only self bindings; grouped hex
literal. No lint was suppressed. Recompiled result follows below.

## Sketch compile, corrected

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge
Checking skein-forge v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge/crates/skein-forge)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 3.41s
```

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge d120820
Directory: /srv/data/git/runner/src/skein
Preparing worktree (detached HEAD d120820)
HEAD is now at d120820 The forge kit: five designs from draft/forge.md
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-01-forge
Directory: /srv/data/git/runner/src/skein
exit 0; scratch worktree removed
```

## Source counts

```text
$ python3 -c 'from pathlib import Path
p=Path('\''crates'\'')
files=['\''reads.rs'\'','\''reads/done.rs'\'','\''reads/size.rs'\'','\''reading.rs'\'','\''writes.rs'\'','\''contents.rs'\'','\''actions.rs'\'','\''plan.rs'\'','\''tables.rs'\'','\''failure.rs'\'','\''marker.rs'\'','\''hooks/hint.rs'\'']
total=0
for f in files:
 n=len((p/'\''temper-engine-protocol-forge/src'\''/f).read_text().splitlines());total+=n;print(f'\''{f}: {n}'\'')
print('\''01.3 modules:'\'',total,'\''plus Fields/Body/Failure/Heard extracted from connection.rs'\'')
a=(p/'\''temper-engine-domain-forge-client/src/bounds.rs'\'').read_text().splitlines()
print('\''01.1: api 417; locate'\'',len((p/'\''temper-engine-domain-forge-client/src/locate.rs'\'').read_text().splitlines()),'\''; sizing helpers'\'',len(a[9:25]+a[91:110]+a[116:200]+a[204:213]))
for crate in ['\''temper-forge-forgejo'\'']:
 files=list((p/crate/'\''src'\'').rglob('\''*.rs'\''));prod=sum(len(f.read_text().splitlines()) for f in files if '\''tests'\'' not in f.parts and f.name!='\''tests.rs'\'');print(crate,'\''production'\'',prod)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
reads.rs: 793
reads/done.rs: 331
reads/size.rs: 150
reading.rs: 253
writes.rs: 673
contents.rs: 391
actions.rs: 387
plan.rs: 97
tables.rs: 91
failure.rs: 91
marker.rs: 55
hooks/hint.rs: 32
01.3 modules: 3344 plus Fields/Body/Failure/Heard extracted from connection.rs
01.1: api 417; locate 35 ; sizing helpers 128
temper-forge-forgejo production 7827
```

01.1's measured source slices total 580 lines; its Size rounds to 600.
01.2's 7,827 count includes module declarations and the cfg(test) declaration;
the survey's 7,822 excludes those test lines. Its Size rounds to 7,820.
01.3's 3,344 module lines plus the exchange types round to 3,500. The moved
tests and their fixtures are excluded from these production estimates.
The per-file diff at implementation counts what remains after path renames;
the new-line allowance, not the copy's total size, is the increment limit.

## System-name scan, source form

Future command: rg -n -i temper crates/skein-forge. At the base there
is no destination, so run on the source regions instead. The expected base
matches are names to remove, not a failed promise that the base is neutral.

```text
$ rg -n -i temper crates/temper-forge-forgejo crates/temper-engine-domain-forge-client/src/api.rs crates/temper-engine-domain-forge-client/src/locate.rs crates/temper-engine-protocol-forge/src
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-protocol-forge/src/tests/writes.rs:58:        assert!(bytes::find(&written, br#""body":"<!-- temper:key abff -->\nbody""#).is_some(), "{name}: marked body");
crates/temper-engine-protocol-forge/src/tests/reads.rs:159:            d::Body::Kept(Box::from(&b"<!-- temper:key 01 -->\ntext"[..])),
crates/temper-engine-protocol-forge/src/tests/reads.rs:161:            Box::from(&b"<!-- temper:key 01 -->\ntext"[..]),
crates/temper-forge-forgejo/src/tests/fixtures.rs:13:    crate::document::User { id: 1, login: Box::from(*b"fixture"), is_admin: true, full_name: Box::from(*b"temper:bot") }
crates/temper-engine-protocol-forge/src/conventions.rs:6:pub const USER_AGENT: &str = "temper/0.0.0";
crates/temper-forge-forgejo/src/tests/accounts.rs:15:        br#"{"id":1,"login":"fixture","full_name":"temper:bot","email":"fixture@example.invalid","is_admin":true}"#;
crates/temper-forge-forgejo/src/tests/accounts.rs:28:    let body = br#"{"id":3,"user":{"id":1,"login":"fixture","is_admin":true,"full_name":"temper:bot"},"body":""}"#;
crates/temper-forge-forgejo/src/tests/accounts.rs:95:        full_name: Box::from(*b"temper:bot"),
crates/temper-forge-forgejo/src/tests/accounts.rs:108:    assert_eq!(encoded.body.as_deref(), Some(br#"{"username":"bot","email":"bot@example.invalid","password":"private-password","full_name":"temper:bot","must_change_password":false}"#.as_slice()));
crates/temper-engine-protocol-forge/src/tests/fixtures.rs:489:    temper_forge_forgejo::document::Body::Kept(Box::from(b"<!-- temper:key 0102 -->\nbody".as_slice()))
crates/temper-engine-protocol-forge/src/tests/connection.rs:272:    assert_eq!(h.sent.as_ref(), b"DELETE /api/v1/repos/fixture/specimen/pulls/7/requested_reviewers HTTP/1.1\r\nHost: 127.0.0.1:3000\r\nAuthorization: token 0123456789abcdef0123456789abcdef01234567\r\nUser-Agent: temper/0.0.0\r\nContent-Type: application/json\r\nContent-Length: 23\r\n\r\n{\"reviewers\":[\"alice\"]}", "encoded upload");
crates/temper-engine-protocol-forge/src/tests.rs:22:        ("text holds a marker", &[0xff], b"<!-- temper:key 01 -->\ntext"),
crates/temper-engine-protocol-forge/src/tests.rs:32:    assert_eq!(marker::encode(&[0xab, 0x01], b"body", 29), Some(Box::from(&b"<!-- temper:key ab01 -->\nbody"[..])));
crates/temper-engine-protocol-forge/src/tests.rs:39:        ("uppercase", b"<!-- temper:key AB -->\n"),
crates/temper-engine-protocol-forge/src/tests.rs:40:        ("odd", b"<!-- temper:key a -->\n"),
crates/temper-engine-protocol-forge/src/tests.rs:41:        ("no line feed", b"<!-- temper:key ab -->"),
crates/temper-engine-protocol-forge/src/tests.rs:42:        ("leading space", b" <!-- temper:key ab -->\n"),
crates/temper-engine-protocol-forge/src/tests.rs:43:        ("after text", b"text<!-- temper:key ab -->\n"),
crates/temper-engine-protocol-forge/src/marker.rs:5:pub const HEAD: &[u8] = b"<!-- temper:key ";
```

The complete scan has 111 matching lines; the rest are source imports,
manifest package names and document references. The destination must have
zero matches; no whitelist or alias preserves the old names.

## User search

All counts below are matching source lines, not distinct callsites. Column
order in tuple output is temper, skein, jig, smith, hearth. Qualified crate
paths have no downstream users outside temper. Short module/symbol names
also match unrelated modules (for example jig's reads); they are conservative
candidates, not dependencies. The source modules' imports and Cargo edges
identify the users listed in survey-01/03 and code-organization.md 3.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
roots=[Path('\''/srv/data/git/runner/src'\'')/r for r in ['\''temper'\'','\''skein'\'','\''jig'\'','\''smith'\'','\''hearth'\'']]
base=Path('\''/srv/data/git/runner/src/tmp/planning/temper-base/crates'\'')
groups={'\''vocabulary-path'\'':r'\''temper_engine_domain_forge_client::api::'\'','\''dialect-path'\'':r'\''temper_forge_forgejo::'\''}
for mod in ['\''reads'\'','\''reading'\'','\''writes'\'','\''contents'\'','\''actions'\'','\''plan'\'','\''tables'\'','\''failure'\'','\''marker'\'','\''hint'\'']:
 groups['\''plan-'\''+mod]=r'\''\b'\''+mod+r'\''::'\''
for key,files in [('\''vocabulary'\'',[(base/'\''temper-engine-domain-forge-client/src/api.rs'\'',417),(base/'\''temper-engine-domain-forge-client/src/locate.rs'\'',999)]),('\''dialect'\'',[(p,9999) for p in (base/'\''temper-forge-forgejo/src'\'').rglob('\''*.rs'\'') if '\''tests'\'' not in p.parts and p.name!='\''tests.rs'\'']),('\''plans'\'',[(base/'\''temper-engine-protocol-forge/src'\''/p,9999) for p in ['\''plan.rs'\'','\''reads.rs'\'','\''reads/done.rs'\'','\''reads/size.rs'\'','\''reading.rs'\'','\''writes.rs'\'','\''contents.rs'\'','\''actions.rs'\'','\''tables.rs'\'','\''failure.rs'\'','\''marker.rs'\'','\''hooks/hint.rs'\'']])]:
 names=set()
 for p,end in files:
  names.update(re.findall(r'\''\b(?:fn|struct|enum|type)\s+(\w+)'\'', '\''\n'\''.join(p.read_text().splitlines()[:end])))
 groups[key+'\''-all-symbols'\'']=r'\''\b('\''+'\''|'\''.join(sorted(names))+r'\'')\b'\''
groups['\''boundary-and-start'\'']=r'\''\b(Heard|Body|Fields|Failure|RepositoryEntry|UserEntry|answer_bytes_unbounded|start_request|drive_starts|page_size|largest_row|revision)\b'\''
for label,pattern in groups.items():
 counts=[]
 for root in roots:
  r=subprocess.run(['\''rg'\'','\''-n'\'','\''--glob'\'','\''*.rs'\'',pattern,str(root/'\''crates'\''),str(root/'\''testing'\''),str(root/'\''tests'\''),str(root/'\''tools'\''),str(root/'\''examples'\'')],capture_output=True,text=True)
  lines=r.stdout.splitlines();counts.append(str(len(lines)))
 print(label+'\'' '\''+'\'','\''.join(counts)+'\'' pattern='\''+pattern)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
vocabulary-path 58,0,0,0,0 pattern=temper_engine_domain_forge_client::api::
dialect-path 180,0,0,0,0 pattern=temper_forge_forgejo::
boundary-and-start 1233,1168,6068,860,1111 pattern=\b(Heard|Body|Fields|Failure|RepositoryEntry|UserEntry|answer_bytes_unbounded|start_request|drive_starts|page_size|largest_row|revision)\b
```

The same run's per-module counts (all moved module names searched; done/size
are covered by the plan symbol inventory, revision by reading):

| Pattern | temper | skein | jig | smith | hearth |
|---|---:|---:|---:|---:|---:|
| reads:: | 58 | 0 | 1584 | 0 | 0 |
| reading:: | 39 | 0 | 0 | 0 | 0 |
| writes:: | 20 | 0 | 0 | 0 | 0 |
| contents:: | 2 | 0 | 0 | 0 | 0 |
| actions:: | 24 | 0 | 30 | 0 | 0 |
| plan:: | 9 | 0 | 2 | 0 | 0 |
| tables:: | 13 | 0 | 0 | 0 | 0 |
| failure:: | 26 | 0 | 0 | 18 | 0 |
| marker:: | 13 | 3 | 0 | 0 | 0 |
| hint:: | 4 | 7 | 0 | 0 | 0 |

The complete symbol inventory and per-symbol counts follow; every moved
function/type name is extracted from its source, including private helpers.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
base=Path("/srv/data/git/runner/src/tmp/planning/temper-base/crates")
roots=[Path("/srv/data/git/runner/src")/r for r in ["temper","skein","jig","smith","hearth"]]
groups={"vocabulary":[(base/"temper-engine-domain-forge-client/src/api.rs",417),(base/"temper-engine-domain-forge-client/src/locate.rs",999)],"dialect":[(p,9999) for p in (base/"temper-forge-forgejo/src").rglob("*.rs") if "tests" not in p.parts and p.name!="tests.rs"],"plans":[(base/"temper-engine-protocol-forge/src"/p,9999) for p in ["plan.rs","reads.rs","reads/done.rs","reads/size.rs","reading.rs","writes.rs","contents.rs","actions.rs","tables.rs","failure.rs","marker.rs","hooks/hint.rs"]]}
cache={}
for label,files in groups.items():
 names=set()
 for p,end in files: names.update(re.findall(r"\b(?:fn|struct|enum|type)\s+(\w+)", "\n".join(p.read_text().splitlines()[:end])))
 if label=="vocabulary": names.update(["answer_bytes_unbounded","answer_heap","bytes","optional","rows","summary","files","pull"])
 if label=="plans": names.update(["Heard","Body","Fields","Failure","start_request","drive_starts","page_size","largest_row"])
 for name in sorted(names):
  if name not in cache:
   counts=[]
   for root in roots:
    paths=[str(root/p) for p in ["crates","testing","tests","tools","examples"] if (root/p).exists()]
    r=subprocess.run(["rg","-n","--glob","*.rs",r"\b"+name+r"\b",*paths],capture_output=True,text=True)
    if r.returncode not in [0,1]: raise RuntimeError(r.stderr)
    counts.append(str(len(r.stdout.splitlines())))
   cache[name]=",".join(counts)
 print(label+": "+"; ".join(name+"="+cache[name] for name in sorted(names)))
print("Each tuple: temper,skein,jig,smith,hearth matching lines; includes definitions and unrelated homonyms.")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
vocabulary: Answer=3021,146,2147,937,347; BranchCreation=50,0,0,0,0; Change=391,10,681,206,202; Check=297,101,61,299,67; Ci=277,0,0,0,0; Collaborator=57,0,0,0,0; Comment=326,29,1,0,0; Commit=690,54,112,40,39; Error=1552,3238,63,1129,1057; File=516,136,74,208,708; Hint=90,0,0,0,0; JobAttempt=64,0,0,0,0; Kind=768,823,921,393,1114; Op=1245,2582,120,399,936; Permission=399,74,15,8,15; Protection=351,0,0,0,0; Provenance=83,1,0,0,33; Pull=582,0,0,0,0; Read=1681,1588,2317,853,326; Remark=41,0,0,0,0; Repository=882,3,0,20,491; Review=344,0,21,16,0; Settings=518,204,394,629,125; State=746,1338,939,627,198; Status=548,64,228,111,2; Step=522,374,914,393,54; Summary=108,305,9,5,3; Verdict=300,26,870,592,6; Write=1690,292,648,242,84; accepts=42,103,31,54,3; answer_bytes_unbounded=7,0,0,0,0; answer_heap=4,0,0,0,0; bytes=4014,5036,19300,4418,2231; files=720,546,325,548,528; optional=42,63,276,49,150; pull=919,23,1,2,1; rows=1566,135,6028,29,144; summary=110,41,116,197,0
dialect: Access=137,20,1,1,0; AccessToken=19,4,0,18,0; ActionKind=37,0,0,0,0; Actions=23,1,0,0,0; Answer=3021,146,2147,937,347; ApiSettings=88,0,0,0,0; Body=255,620,966,61,11; BodyDecoder=16,0,0,0,0; BodyStep=18,0,0,0,0; Branch=665,9,0,4,26; BranchKind=45,0,0,0,0; BranchProtection=19,0,0,0,0; Branches=189,0,0,0,0; ChangedFile=34,0,0,0,0; CollaboratorPermission=11,0,0,0,0; CombinedStatus=91,0,0,0,0; Comment=326,29,1,0,0; CommitStatus=22,0,0,0,0; Compare=230,0,9,7,1; ContentRange=10,0,0,0,0; Decoder=23,31,5,0,84; DecoderState=33,0,0,0,0; Delivery=793,2,1236,628,6; DeliveryKind=8,0,0,0,0; Document=472,165,994,39,115; Encoded=74,4,30,0,0; Error=1552,3238,63,1129,1057; ErrorBody=14,10,0,19,0; Event=3396,4098,9003,4026,1079; Field=84,135,110,272,227; FileKind=24,6,0,0,0; FileStatus=30,0,0,0,0; Files=114,68,62,114,79; Frame=4,58,217,22,92; Hook=147,0,0,0,0; HookSpec=21,0,0,0,0; Hooks=52,0,0,0,0; Issue=341,0,1,3,0; IssueKind=18,0,0,0,0; IssueState=54,0,0,0,0; Item=372,396,230,108,391; Items=228,2,14,2,38; Job=409,0,0,493,39; JobEnd=21,0,0,210,0; Kind=768,823,921,393,1114; Limits=3099,2301,19363,3286,690; Malformed=276,273,295,251,122; MergeStyle=58,0,0,0,0; Method=180,216,80,1,8; Move=145,29,289,16,0; ObjectFormat=49,0,0,0,0; ObjectId=69,0,0,0,0; Org=97,0,0,0,0; Orgs=16,0,0,0,0; Page=144,204,552,0,15; Pair=11,55,0,0,62; Permission=399,74,15,8,15; Problem=1870,557,14503,1748,0; PullBranch=18,0,0,0,0; PullRequest=26,0,0,0,0; Read=1681,1588,2317,853,326; RefType=25,0,0,0,0; Refusal=503,597,2644,843,171; Repo=113,0,0,54,0; Repository=882,3,0,20,491; Request=4969,2653,6979,2603,869; Review=344,0,21,16,0; ReviewComment=22,0,0,0,0; ReviewEvent=45,0,0,0,0; ReviewKind=23,0,0,0,0; ReviewState=49,0,0,0,0; Reviews=203,0,0,0,0; Route=281,3,181,49,0; Row=304,110,260,0,164; Run=332,237,734,232,103; Sent=59,77,212,57,22; Setup=120,22,21,0,0; SetupKind=127,0,0,0,0; Sort=15,21,0,0,28; Start=340,174,299,457,36; StateFilter=18,0,0,0,0; StatusState=87,0,0,0,0; Step=522,374,914,393,54; StepStatus=74,0,0,0,0; Target=67,169,54,36,188; Team=76,0,0,0,0; TeamSpec=35,0,0,0,0; Units=30,0,0,4,0; Unread=44,61,77,57,33; Unrouted=119,0,0,0,0; UpdateStyle=13,0,0,0,0; User=296,86,10,182,68; Value=438,180,1055,707,939; Walk=34,4,2,0,10; Want=110,20,35,7,37; WriteKind=72,0,0,0,0; access=93,76,85,59,12; access_text=3,0,0,0,0; answer=1802,1119,3156,2065,261; before=824,1032,2444,1020,808; body=1961,1803,1320,516,355; body_first=5,0,0,0,0; branch=1855,36,0,70,38; check=692,493,4162,1056,222; comment=377,38,13,20,10; commit_status=3,0,0,0,0; config=1233,663,819,487,985; content_range=16,0,0,0,0; cut=260,351,2426,345,97; decoder=189,245,14,26,141; deleted=39,0,18,2,1; digit=38,67,26,39,59; document=873,1127,2851,370,431; element=16,266,4,5,26; empty=255,304,410,184,125; encode=1505,188,12132,1190,260; end=629,1508,1480,1277,382; error_decoder=7,0,0,0,0; error_fields=2,0,0,0,0; event=1600,1742,3079,1481,321; events=521,2063,819,588,124; field=2029,1317,12723,1699,997; fields=460,368,4405,1079,447; file=666,1393,663,1296,1127; finish=1243,651,8266,1506,181; finish_pull=2,0,0,0,0; finish_user=2,0,0,0,0; first=979,1288,2499,1298,558; flag=23,35,19,47,1; has_more=8,0,3,0,0; header=176,446,383,226,58; hex=64,65,5,31,136; hook=288,4,10,0,0; hook_forge=5,0,0,0,0; hook_target=6,0,0,0,0; http_date=4,0,0,0,0; issue=392,79,7,16,64; issue_query=2,0,0,0,0; issue_state=2,0,0,0,0; job=395,0,2,928,4; job_link=4,0,0,0,0; joined=84,85,71,38,26; keep_after=8,0,0,0,0; key=2176,1237,5700,749,1927; leap=3,7,6,0,0; member_target=3,0,0,0,0; merge_style=7,0,0,0,0; month_days=4,0,0,0,0; names_forgejo=10,0,0,0,0; names_runner=10,0,0,0,0; new=6624,5903,25717,8095,2337; number=3530,586,8914,870,92; number_value=33,0,2,0,0; numbered=47,15,121,21,0; object=398,144,2261,159,136; optional=42,63,276,49,150; org_name=2,0,0,0,0; org_target=3,0,0,0,0; page=902,890,2232,6,76; parent=450,653,1449,1161,262; permission=259,22,170,6,6; protection=265,1,0,0,0; pull=919,23,1,2,1; pull_branch=3,0,0,0,0; pull_for=4,0,0,0,0; pull_row=2,0,0,0,0; put=1167,574,6451,1218,879; query=141,389,650,303,1; query_value=16,0,0,0,0; read=1660,2553,3196,1799,573; read_route=2,0,0,0,0; refusal=314,325,740,617,21; repo=369,24,3,92,72; repository=2975,6,0,168,1050; repository_fields=2,0,0,0,0; retry_after=23,77,62,142,0; review=437,0,40,80,1; review_text=2,0,0,0,0; route=350,93,572,101,0; route_tail=2,0,0,0,0; run=1565,1745,3738,8284,594; scalar=26,9,22,44,39; segment=92,38,362,1,14; segment_length=4,0,0,0,0; set_id=2,0,0,0,0; settings_fields=2,0,0,0,0; skip_comment=2,0,0,0,0; skipped=63,90,7,26,1; sorting=2,0,0,1,1; specification=32,0,375,28,0; start=787,966,804,1119,317; state=1285,1321,3130,732,147; status=1002,416,525,292,23; status_state=4,0,0,0,0; step=1021,627,2188,1401,212; step_status=2,0,0,0,0; step_status_text=2,0,0,0,0; style=43,42,14,0,27; style_text=2,0,0,0,0; target=415,709,679,99,2374; team=184,0,1,0,0; team_text=3,0,0,0,0; text=1545,2050,4782,3318,1627; to=1127,1756,2915,1109,405; token=1138,1196,2431,619,175; unescape=4,2,0,0,0; unsigned=53,57,8,273,34; update=64,64,137,5,19; update_target=2,0,0,0,0; user=1019,238,25,122,209; value=5072,2074,40952,4667,2621; write=1088,730,453,599,182; zero=32,72,78,109,9
plans: Asked=165,75,73,220,223; Body=255,620,966,61,11; Classified=5,0,2,0,0; Drop=46,31,80,52,1; Exchange=74,241,155,22,0; Failure=313,518,460,775,113; Fields=14,11,34,17,28; Heard=137,19,184,7,0; Mode=85,82,2,111,368; Plan=89,180,216,12,29; Range=446,35,214,0,136; Repository=882,3,0,20,491; RepositoryEntry=5,0,0,0,0; Rows=155,0,140,0,8; Stage=195,111,273,170,20; Step=522,374,914,393,54; Tables=40,98,0,0,0; UserEntry=5,0,0,0,0; absolute=32,20,22,93,32; add_size=8,14,0,0,0; after=957,867,3867,901,823; answer=1802,1119,3156,2065,261; append=141,306,85,100,30; asked=231,271,518,317,178; at=2108,3340,4977,1890,2688; before_write=5,0,0,0,0; begin=74,29,102,48,7; body=1961,1803,1320,516,355; branch=1855,36,0,70,38; branch_done=2,0,0,0,0; check=692,493,4162,1056,222; check_pull_ids=2,0,0,0,0; check_size=4,0,0,0,0; ci=328,0,0,5,23; classify=16,29,89,4,1; comment=377,38,13,20,10; commit=2110,191,818,159,166; decode=1478,374,10429,1165,94; digit=38,67,26,39,59; done=346,432,392,461,51; drive_starts=2,0,0,0,0; drop_repository=5,0,0,0,0; encode=1505,188,12132,1190,260; error=364,1121,286,618,927; expected=889,960,1065,788,966; files_size=3,0,0,0,0; finish_rows=3,0,0,0,0; heard=193,39,95,33,0; hex=64,65,5,31,136; hint=124,25,34,2,2; id=1791,1876,2231,1178,135; initial=32,26,148,103,9; keep_document=5,0,0,0,0; keep_pull=3,0,0,0,0; keep_repository=7,0,0,0,0; keep_row=2,0,0,0,0; keep_user=23,0,0,0,0; largest_row=2,0,0,0,0; length=542,979,751,863,465; listed_done=2,0,0,0,0; log=183,50,47,43,40; long=94,211,64,127,15; long_body=8,0,0,0,0; lookup=14,14,47,15,17; lookup_row=2,0,0,0,0; marked=16,26,40,8,19; new=6624,5903,25717,8095,2337; next=901,1306,1617,808,429; next_check=11,0,0,0,0; next_permission=3,0,0,0,0; next_side=4,0,0,0,0; optional=42,63,276,49,150; optional_commit=3,0,0,0,0; optional_time=3,0,0,0,0; page_bound=2,0,0,0,0; page_size=28,0,0,0,0; paged=21,0,20,0,1; pending=887,573,1323,369,289; permission=259,22,170,6,6; protection=265,1,0,0,0; provenance=78,0,5,0,4; pull=919,23,1,2,1; pull_done=2,0,0,0,0; pull_status=5,0,0,0,0; push=2677,2058,6136,1938,1316; push_items=2,0,0,0,0; raw=526,423,582,282,61; raw_room=11,0,0,0,0; read=1660,2553,3196,1799,573; remark=33,0,0,0,0; repository=2975,6,0,168,1050; repository_done=2,0,0,0,0; review=437,0,40,80,1; reviewer_write=3,0,0,0,0; reviewers=198,0,0,0,0; revision=495,0,4445,0,959; row=2070,43,6631,20,806; rows=1566,135,6028,29,144; search_done=2,0,0,0,0; send=590,603,1211,144,15; set_pending=6,0,0,0,0; settings=1055,565,599,1557,576; side=102,575,744,41,542; start_request=4,0,0,0,0; state=1285,1321,3130,732,147; status=1002,416,525,292,23; step_name=11,0,0,0,0; summary=110,41,116,197,0; take_cut=4,0,0,0,0; terminal=544,575,802,1645,128; text=1545,2050,4782,3318,1627; tick=96,102,138,28,6; time=243,406,331,685,32; transport=238,163,721,122,17; user=1019,238,25,122,209; validate=6,31,44,80,35; verdict=268,17,492,280,22
Each tuple: temper,skein,jig,smith,hearth matching lines; includes definitions and unrelated homonyms.
```

## Verification of the draft

The session has five increments. The new-production allowances are
30, 60, 290, 180 and 90 lines. Test code does not count towards the
300-line production allowance, so 01.5 stays one increment.
Only the assigned session and findings files were written persistently;
no commit or landing was made. Base worktrees remained clean. The temporary
skein worktree was removed after the corrected sketch passed.






## Revision after the 2026-10-19 review

This section supersedes the draft-specific conclusions above. Read section 5
of development.md whole, the plan's README/organization/decisions, both
designer answer files, and the relevant design and base source. The supplied
bases are now temper dd12b8ed904edced9e65af9fa62cc769c5a09905 and skein
8938e16dd6381b34bb961e45b15734320aca2071; their code is the earlier base's.

Graph discovery used srv-data-git-runner-src-temper: the protocol-forge
host search returned zero symbols, and the broad search mixed old paths.
Used rg and the named base files for verification. The skein graph found
skein-http's server::response::put_date; verified it with rg and against
temper's pinned skein revision. No indexing or project-list tools were used.

### What changed

1. Needs now match the README: 01.2 needs only skein 01.1, 01.3 needs
   temper 01.1's vocabulary users AND temper 01.2's dialect users, 01.4
   needs temper 01.3's component on the moved plans/events. No temper
   dependency in 01.5. The README was already corrected and was not edited.
2. 01.5 names the downstream tests and the concrete undated Items fixture
   in the machine world. The fake protocol's omission of a Date field from
   its own Reply is not an omission on the wire: skein-http supplies it.
   See the pinned-source check below; adding Date to Reply would be wrong.
3. The approximately 190-line reads/keep.rs split belongs to 01.3,
   before threading Bounds/word can take the 793-line file past 800.
   The split does not add another 190 to the 3,500 moved-line estimate.
4. The measure stays wholly in skein 02.2 (decision 24). 01.3's Don't
   explicitly leaves reads.rs's Branches room and row widths,
   reads/size.rs including rows(), contents.rs's File/Commit widths,
   actions.rs's Status capacity and reads/done.rs's Comment capacity
   unchanged until that step.
5. No plan calls answer_bytes_unbounded or the client's bounds helpers.
   Removed both from 01.1; its two source slices total 452 lines, rounded
   to 450 moved. The client keeps its wrapper/helpers until skein 02.2;
   temper-engine-domain/src/store.rs's answer_bytes_unbounded call remains
   on the client until temper 02.1. The old organization table's 01.1 row
   and decisions 4 still mention the early port; they are outside this
   task's write scope, so the planner must reconcile them.
6. hex stays pub(crate). During the interval after temper 01.3 deletes
   marker.rs and before 01.4 takes the Host helper, conventions::host
   needs its own private digit expression, not an upstream public hex.
7. Step::Done keeps no cost; the component counts bytes-first-written
   requests as at the base. Only question 34 remains pending, at Shape.
8. read_calls_follow_canned_exchanges now explicitly checks
   Range::Tail(4) for Job and Range::Head(4) for File.
9. 01.2 names document.rs's production comment containing the system name
   and its neutral replacement, in addition to fixture bytes/citations.

01.4 now moves conventions::{host,put} and
host_names_include_only_nondefault_ports_and_bracket_ipv6. Its new name
argument covers configured plaintext and TLS names. Ports 80 and 443 are
both omitted per decision 25, including TLS on 80 and plaintext on 443.
The new test covers names and IP fallback, IPv4/IPv6 and port 3000 too.
The base callsites are temper-service/src/forge.rs and temper/src/forge.rs,
not connection.rs: connection.rs currently sends the already-built Forge.host.
Those owner callsites switch in temper 01.4; the Host field disappears with
skein 03.1/temper 03.1. There is no re-export or alias.

### Answers 7–12 and 35, traced against the base

- Q7: writes::Plan::heard maps Malformed::Bound to TooLarge; the read
  plans lump it with malformed answers as InvalidAnswer. The new test
  requires Timeout on writes, TooLarge on reads, retaining the pre-POST
  OpenPull exception. Client admission remains distinct in skein 02.2.
- Q8, B: contents::Plan::heard and actions::Plan::heard drop the path
  before deciding a 404 is an answer. Move that decision ahead of the
  drop for missing file sides and Checks' absent run/job. Terminal 404s,
  including Job's MissingJob, already drop it and must still do so.
- Q9: contents::Plan::raw returns TooLarge for a short remaining room
  before max_bytes. File must cut there. Job already relies on raw_room
  and the connection to retain the tail; its pure test supplies that
  bounded tail, not an impossible oversized connection result. Sides
  of PullFiles/Compare still fail TooLarge instead of cutting.
- Q10: reads::Plan::heard substitutes Time::ZERO when Date is absent;
  require it on the Items listing's own head, not a preceding resolve.
  contents::Plan::heard defaults missing X-HasMore to false; use page
  fullness instead, while explicit false stays authoritative. The
  no-Retry-After 429 fallback is already Duration::ZERO.
- Q11: exact success statuses already hold: reads/actions/contents
  reject the wrong status, writes::expected supplies the per-stage
  status and classify's default maps other 2xx to malformed failure.
  Added the explicit pure regression matrix; no invented production
  defect or broad status rewrite.
- Q12: reads/done.rs::repository_done accepts any nonzero returned id.
  It must compare a RepositoryById result before keeping any row. Its
  done method calls keep_document too early (caching the owner's login);
  writes has that same cache ordering before its existing id check.
  Check before either cache mutation. Locate still accepts another
  nonzero id; out-of-u32 ids retain TooLarge.
- Q35: classify(429) does return RateLimited, but writes::Plan::heard
  returns self.error(classified.error), and error overrides every
  Stage::Merged failure to Timeout. The base test already includes
  post-merge 429 and passes below. The settled result is retained and
  a pure test distinguishes pre-merge 429 from post-merge 429; there is
  no remaining terminal RateLimited bug at these supplied bases.

For Q7/Q8/Q9/Q12, rg and the bodies of the named component tests found no
existing expectation that must change; the repin adds their new boundary
rows while retaining names. The existing malformed-write test uses "{}",
not inner overflow; the raw-side TooLarge test concerns PullFiles, not
standalone File; resolution tests return the requested id; 404 tests do
not assert an unnecessary re-resolve after an answering 404.
The component harness's response helper already adds Date. The machine
world's fixture does not; its named long-body Items test bypasses HTTP
server response rendering and needs a dated scripted head at the repin.
No qualified old forge-crate paths were found in skein/jig/smith/hearth.

### Revised source and user search

```text
$ python3 -c 'from pathlib import Path
import re,subprocess
root=Path(".")
p=root/"crates/temper-engine-protocol-forge/src"
files=[p/f for f in ["reads.rs","reads/done.rs","reads/size.rs","reading.rs","writes.rs","contents.rs","actions.rs","plan.rs","tables.rs","failure.rs","marker.rs","hooks/hint.rs"]]
def matches(pattern,paths):
 r=subprocess.run(["rg","-n",pattern,*map(str,paths)],capture_output=True,text=True)
 if r.returncode not in (0,1): raise RuntimeError(r.stderr)
 return r.stdout.splitlines()
print("Plan answer_bytes_unbounded/client bounds references:",len(matches(r"answer_bytes_unbounded|bounds::",files)))
print("Vocabulary import files:",len(matches(r"^use temper_engine_domain_forge_client::api",files)))
print("Dialect import files:",len(matches(r"^use temper_forge_forgejo",files)))
print("reads.rs lines:",len((p/"reads.rs").read_text().splitlines()))
print("reads.rs split helpers:",", ".join(re.findall(r"fn (keep_user|keep_pull|keep_row|keep_document|page_bound|long|long_body)\(",(p/"reads.rs").read_text())))
for pattern,paths in [
 (r"repository units temper",[root/"crates/temper-forge-forgejo/src/document.rs"]),
 (r"pub\(crate\) fn hex",[p/"marker.rs"]),
 (r"fn host|let default",[p/"conventions.rs"]),
 (r"conventions::host\(",[root/"crates/temper-service/src/forge.rs",root/"crates/temper/src/forge.rs"]),
 (r"Stage::Merged => api::Error::Timeout|429 => api::Error::RateLimited",[p/"writes.rs",p/"failure.rs"]),
 (r"hint::hint",[p/"hooks/connection.rs"]),
 (r"Some\(Step::Done\(Err\(self.error\(classified.error\)\)\)\)",[p/"writes.rs"])
]:
 for line in matches(pattern,paths): print(line)
for f in ["response.rs","server.rs"]:
 print("Fake",f,"Date literal matches:",len(matches(r'\''(?i)"date"|Date:'\'',[root/"testing/temper-fake-forge-protocol/src"/f])))
print("host moved test:",len(matches(r"fn host_names_include_only_nondefault_ports_and_bracket_ipv6",[p/"tests.rs"])))
print("01.1 source vocabulary/locate lines:",417+len((root/"crates/temper-engine-domain-forge-client/src/locate.rs").read_text().splitlines()))
for repo in ["skein","jig","smith","hearth"]:
 base=Path("/srv/data/git/runner/src")/repo
 found=matches(r"temper_engine_protocol_forge::|temper_forge_forgejo::|temper_engine_domain_forge_client::", [base/d for d in ["crates","tests","testing","tools"] if (base/d).exists()])
 print(repo,"qualified old forge paths:",len(found))
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Plan answer_bytes_unbounded/client bounds references: 0
Vocabulary import files: 10
Dialect import files: 9
reads.rs lines: 793
reads.rs split helpers: keep_user, keep_pull, keep_row, keep_document, page_bound, long, long_body
417:/// The four repository units temper uses.
53:pub(crate) fn hex(byte: u8) -> u8 {
9:pub fn host(address: SocketAddr, server_name: Option<&[u8]>) -> Option<Box<[u8]>> {
55:    let default = if server_name.is_some() { 443 } else { 80 };
crates/temper/src/forge.rs:258:        host: conventions::host(address, None).expect("checked address"),
crates/temper-service/src/forge.rs:56:        let Some(host) = kit::conventions::host(endpoint.address, name) else {
crates/temper-engine-protocol-forge/src/failure.rs:43:        429 => api::Error::RateLimited { after: retry.unwrap_or(Duration::ZERO) },
crates/temper-engine-protocol-forge/src/writes.rs:349:            Stage::Merged => api::Error::Timeout,
211:                Some(delivery) => hint::hint(self.forge, &delivery),
249:                    return Ok(Some(Step::Done(Err(self.error(classified.error)))));
Fake response.rs Date literal matches: 0
Fake server.rs Date literal matches: 0
host moved test: 1
01.1 source vocabulary/locate lines: 452
skein qualified old forge paths: 0
jig qualified old forge paths: 0
smith qualified old forge paths: 0
hearth qualified old forge paths: 0
```

The zero Date matches above were only a candidate break. Following the
HTTP writer ruled it out for the fake protocol, at the lockfile's skein
8a51e10b873f39ea699853361408812eb80a5f87 as well as skein-base. The writer
adds Date and refuses an owner-supplied Date field:

```text
$ sh -c 'git show 8a51e10b873f39ea699853361408812eb80a5f87:crates/skein-http/src/server/response.rs | rg -n '\''put_date\(&mut head|header.is\(b"date"\)'\'''
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
161:    put_date(&mut head, wall);
205:            || header.is(b"date")
```

```text
$ rg -n 'fn a_body_past_body_bytes_is_carried_empty_without_its_key|fixtures::response\(200, format|Op::Read\(Read::Items|pub fn response|HTTP/1.1' tests/engine/protocol-forge/tests/machine.rs tests/engine/protocol-forge/src/fixtures.rs
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/src/fixtures.rs:109:pub fn response(code: u16, body: &[u8]) -> Vec<u8> {
tests/engine/protocol-forge/src/fixtures.rs:110:    let mut bytes = format!("HTTP/1.1 {code} Answer\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
tests/engine/protocol-forge/tests/machine.rs:133:fn a_body_past_body_bytes_is_carried_empty_without_its_key() {
tests/engine/protocol-forge/tests/machine.rs:144:                response: Response::Bytes(fixtures::response(200, format!("[{body}]").as_bytes())),
tests/engine/protocol-forge/tests/machine.rs:148:    world.call(1, fixtures::REPO, Op::Read(Read::Items { since: skein_lib::Time::ZERO, page: 1, kind: None }));
```

### New signature compile and focused base check

Only host(address, name)'s new boundary needed a new signature sketch;
the retained Bounds/plan/start/credential/reading signatures have their
clippy evidence above. The new scratch crate has core/alloc, a public host
module and the signature with a typed argument binding and unreachable
stub. No Host implementation or behavioural test pass is claimed from
this compile. hex's retained private signature is already the base's.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-01-revision 8938e16
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 8938e16)
HEAD is now at 8938e16 Answer the forge kit planner's other questions
```

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-01-revision
    Checking skein-forge v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-01-revision/crates/skein-forge)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.15s
```

```text
$ cargo nextest run -p temper-engine-protocol-forge -E 'test(every_failure_of_a_merges_read_is_timeout) | test(host_names_include_only_nondefault_ports_and_bracket_ipv6)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
    Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
    Starting 2 tests across 1 binary (115 tests skipped)
        PASS [   0.002s] (1/2) temper-engine-protocol-forge tests::host_names_include_only_nondefault_ports_and_bracket_ipv6
        PASS [   0.006s] (2/2) temper-engine-protocol-forge tests::writes::every_failure_of_a_merges_read_is_timeout
     Summary [   0.007s] 2 tests run: 2 passed, 115 skipped
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-01-revision
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; scratch worktree removed
```

### Final session audit

All prescribed proof and closing shell commands are unchanged from the
draft and have their base equivalents recorded above. The new tests do
not exist at the base. Cumulative planned counts are now 2/86/90/99/126:
01.4 adds one moved Host test and eight new tests; 01.5 adds 27
(twelve calls tests, fifteen edges tests). The base tests retain names,
budgets and unoptimized builds. No new production interface in 01.5.

01.5 is 112 lines, above the approximately 100-line target; none of the
answers was dropped to hide that. Its production estimate is 220 new
lines, below 300. The whole session remains five increments and 400 lines.
Only Q34 is pending. The organization/decisions early-wrapper wording
noted above remains outside this two-file revision's scope.

```text
$ python3 -c 'from pathlib import Path
p=Path("docs/development/plans/forge-kit/skein/01-forge.md")
lines=p.read_text().splitlines()
starts=[i for i,l in enumerate(lines) if l.startswith("### ")]+[lines.index("## Done when")]
assert len(lines)<=400 and len(starts)==6
print("Session:",len(lines),"lines; five increments")
for a,b in zip(starts,starts[1:]):
 block=lines[a:b]
 fields=["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]
 assert all(sum(x.startswith(k+":") for x in block)==1 for k in fields)
 print(block[0].split(":")[0],str(b-a)+" lines;",next(x for x in block if x.startswith("Size:")))
pending=[l.strip() for l in lines if "pending the designer" in l]
assert len(pending)==1 and "34" in pending[0]
assert not any(l!=l.rstrip() for l in lines)
assert "hex stays pub(crate)" in "\n".join(lines)
print("Pending only Q34; all nine fields present; no trailing whitespace")
print("Planned cumulative tests: 2 / 86 / 90 / 99 / 126")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
Session: 400 lines; five increments
### 01.1 The crate and vocabulary 30 lines; Size: about 450 lines moved, 30 new.
### 01.2 Forgejo's dialect 24 lines; Size: about 7,820 lines moved, 60 new.
### 01.3 The call plans, tables and marker 90 lines; Size: about 3,500 lines moved, 290 new.
### 01.4 The start reads, Host and credential kinds 96 lines; Size: about 110 lines moved, 195 new.
### 01.5 Pure plan tests and the settled differences 112 lines; Size: about 0 lines moved, 220 new.
Pending only Q34; all nine fields present; no trailing whitespace
Planned cumulative tests: 2 / 86 / 90 / 99 / 126
```

Only skein/01-forge.md and this appended findings section were written
persistently. Both base worktrees are clean; the scratch worktree is
removed. No commit or landing was made.

## Re-check revision (2026-10-19)

This section supersedes the earlier revision's size estimates, downstream
test list and pending-Q34 statements. Inputs: the whole requested section 5,
README, organization (including its revised section 6), decisions, designer
answers 1–3, the relevant forge designs, both reviews and skein-01b's
planner rulings. Base code remains temper dd12b8e and skein 25d2551.
All design questions used here, including 34 and 35, are settled.

Only skein/01-forge.md and this findings appendix persist. The scratch
copy below was explicitly requested for item 3; it is not a production
implementation or a commit. Graph searches in the supplied temper and
skein projects returned no matching symbols for the scoped searches;
named base files and rg supplied the missing coverage. No list/index
operation was called.

### The six rulings

1. The pre-POST OpenPull overflow now expects TooLarge in both Shape
   and Test shape. before_write converts only Timeout/InvalidAnswer.
2. 01.3's temper's side now names both hooks/connection.rs marker::hex
   calls and local digit expressions until temper 03.1 deletes that
   component crate. conventions::host keeps its earlier local-expression
   repair until temper 01.4. hex remains pub(crate).
3. The real, formatted scratch copy below measures 3,206 moved and 300
   new lines, including tests, under organization 6. reads/keep.rs pairs
   with reads.rs. 01.3's Size rounds only the moved total to 3,210.
4. 01.5 includes all new tests and fixtures: estimated 1,100 new lines.
   01.4's old 195 was production only too; its eight new tests and
   fixture/wiring additions add about 125, for 320. Its Size now says so.
   The smallest self-contained behaviour change, Items' Date check and
   its dated-resolve/undated-listing test, is about 35 new lines and would
   make 01.4 about 355. The ruling's conditional move therefore does not
   fit; both Sizes state the issue for the planner, keeping five increments
   and all coverage. This is an unresolved size ruling, not a design question.
5. Removed the unaffected component tests and the instruction to add rows
   at a repin. Only the machine world's undated Items response remains:
   tests/engine/protocol-forge/src/fixtures.rs::response feeds
   tests/engine/protocol-forge/tests/machine.rs::
   a_body_past_body_bytes_is_carried_empty_without_its_key.
6. Pure tests record sends and explicitly assert no cost. Done still
   carries only the result; component first-byte accounting is unchanged.

The copy compile also made two forced details concrete: reading::remark
calls the now word-taking body conversion, so its signature and caller
thread the same word; reads::long moves to public reads::keep::long, whose
consumer is component/calls.rs. The public plan enum needs public
reads/writes payload types. These are named in 01.3's Shape, with the
diagnostic path's temper repair. No new behaviour or alias was added.

```text
$ rg -n -A 3 'Body::Malformed\(temper_forge_forgejo::decode::Malformed::Bound|pub\(crate\) fn before_write' crates/temper-engine-protocol-forge/src/writes.rs crates/temper-engine-protocol-forge/src/failure.rs
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-protocol-forge/src/failure.rs:76:pub(crate) fn before_write(asked: Asked, error: api::Error) -> api::Error {
crates/temper-engine-protocol-forge/src/failure.rs-77-    if asked == Asked::BeforeWrite && (error == api::Error::Timeout || error == api::Error::InvalidAnswer) {
crates/temper-engine-protocol-forge/src/failure.rs-78-        api::Error::Unavailable
crates/temper-engine-protocol-forge/src/failure.rs-79-    } else {
--
crates/temper-engine-protocol-forge/src/writes.rs:254:                    Body::Malformed(temper_forge_forgejo::decode::Malformed::Bound { .. }) => {
crates/temper-engine-protocol-forge/src/writes.rs-255-                        return Err(self.error(api::Error::TooLarge));
crates/temper-engine-protocol-forge/src/writes.rs-256-                    }
crates/temper-engine-protocol-forge/src/writes.rs-257-                    Body::Malformed(_) | Body::Raw { .. } => return Err(self.error(api::Error::Timeout)),
```

```text
$ rg -n 'marker::hex|reads::long|fn remark|reading::remark|fn a_body_past_body_bytes_is_carried_empty_without_its_key|fixtures::response\(200, format|Op::Read\(Read::Items|pub fn response|HTTP/1.1' crates/temper-engine-protocol-forge/src/hooks/connection.rs crates/temper-engine-protocol-forge/src/conventions.rs crates/temper-engine-protocol-forge/src/component/calls.rs crates/temper-engine-protocol-forge/src/reading.rs crates/temper-engine-protocol-forge/src/reads.rs tests/engine/protocol-forge/tests/machine.rs tests/engine/protocol-forge/src/fixtures.rs
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/src/fixtures.rs:109:pub fn response(code: u16, body: &[u8]) -> Vec<u8> {
tests/engine/protocol-forge/src/fixtures.rs:110:    let mut bytes = format!("HTTP/1.1 {code} Answer\r\nContent-Length: {}\r\n\r\n", body.len()).into_bytes();
tests/engine/protocol-forge/tests/machine.rs:133:fn a_body_past_body_bytes_is_carried_empty_without_its_key() {
tests/engine/protocol-forge/tests/machine.rs:144:                response: Response::Bytes(fixtures::response(200, format!("[{body}]").as_bytes())),
tests/engine/protocol-forge/tests/machine.rs:148:    world.call(1, fixtures::REPO, Op::Read(Read::Items { since: skein_lib::Time::ZERO, page: 1, kind: None }));
crates/temper-engine-protocol-forge/src/reads.rs:388:                self.append(api::Answer::Remarks { remarks: Box::new([reading::remark(remark)]), more: false }, shared)
crates/temper-engine-protocol-forge/src/conventions.rs:47:                            put(&mut out, &[super::marker::hex(digit)])?;
crates/temper-engine-protocol-forge/src/component/calls.rs:238:                if crate::reads::long(&heard) {
crates/temper-engine-protocol-forge/src/hooks/connection.rs:199:            *high = marker::hex(byte / 16);
crates/temper-engine-protocol-forge/src/hooks/connection.rs:200:            *low = marker::hex(byte % 16);
crates/temper-engine-protocol-forge/src/reading.rs:162:pub fn remark(value: d::ReviewComment) -> api::Remark {
```

### Scratch pairs

Fresh detached worktree, created from the supplied skein-base:

```text
$ git worktree add --detach /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b 25d2551
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 25d2551)
HEAD is now at 25d2551 Have the forge component count a call's cost
```

copy01.py copied the 01.1 vocabulary and 01.2 dialect as prerequisites,
then the 01.3 files and the four named source tests. The latter changed
only paths, public visibility/must_use, plain Bounds and borrowed word
arguments, the neutral marker framing, sha256's digest representation,
and the reads helper split. No 01.5 behaviour was changed. The script and
its generated copy existed only in the scratch worktree.

```text
$ python3 copy01.py
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
Copied 01.1/01.2 prerequisites and the 01.3 move; four source tests retained.
```

The copy was compiled, not merely counted. The first three clippy attempts
exposed copy-script errors and the public boundary's lint requirements;
none was treated as a pass. There was no lint suppression.

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
Attempt 1: exit 101; incomplete path substitutions, two substring substitutions,
missing word on remark, stale multiline page_size/client accesses, test qualifications.
log /srv/data/git/runner/src/tmp/logs/20261019T014114-734.log
Attempt 2: exit 101; remaining missing word on remark.
log /srv/data/git/runner/src/tmp/logs/20261019T014139-733.log
Attempt 3: exit 101; private payload types, unreachable long, missing must_use,
and a closure in valid_word (replaced with the required for loop).
log /srv/data/git/runner/src/tmp/logs/20261019T014204-733.log
Corrected copy:
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.45s
```

```text
$ python3 count01.py
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
Before formatting: 3228 moved, 271 new (tests included).
Split reads.rs / keep.rs lines: 606 183
```

```text
$ cargo fmt -p skein-forge
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
```

count01.py runs git diff --no-index --numstat separately for every pair
below; an exit of 1 means differences. New = the first numstat column;
moved = destination lines minus new. No path normalization hides added
lines: even the vocabulary/dialect import changes that temper's preceding
steps will already have made are counted against the supplied raw base.

Source paths below are relative to
/srv/data/git/runner/src/tmp/planning/temper-base/crates/temper-engine-protocol-forge/src;
destination paths are relative to the scratch's crates/skein-forge/src.
The reads.rs source is paired with both reads.rs and reads/keep.rs;
only each destination's unchanged lines count, never the deleted source
lines twice. count-tests.rs is the four moved functions and their imports
extracted from the formatted src/tests.rs, excluding 01.1's existing two
tests. count-fixtures.rs is the added REPO/specimen slice and its import,
excluding 01.2's existing dialect fixtures. Both extracts sit at scratch
root. prior01.3/{lib,forgejo}.rs are snapshots of the prior prerequisite
state; their unchanged lines are already present, so moved is zero.

```text
$ python3 count01.py
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
source -> destination: added / removed / moved
reads.rs -> forgejo/plans/reads.rs: 64 / 253 / 540
reads/done.rs -> forgejo/plans/reads/done.rs: 23 / 32 / 299
reads/size.rs -> forgejo/plans/reads/size.rs: 1 / 1 / 149
reading.rs -> forgejo/plans/reading.rs: 20 / 20 / 233
writes.rs -> forgejo/plans/writes.rs: 31 / 23 / 650
contents.rs -> forgejo/plans/contents.rs: 25 / 27 / 364
actions.rs -> forgejo/plans/actions.rs: 17 / 18 / 369
plan.rs -> forgejo/plans/plan.rs: 37 / 27 / 70
tables.rs -> forgejo/plans/tables.rs: 5 / 5 / 86
failure.rs -> forgejo/plans/failure.rs: 4 / 4 / 87
reads.rs -> forgejo/plans/reads/keep.rs: 5 / 615 / 178
hooks/hint.rs -> forgejo/plans/hint.rs: 6 / 5 / 27
marker.rs -> marker.rs: 24 / 8 / 47
connection.rs -> forgejo/plans.rs: 11 / 696 / 50
tests.rs -> count-tests.rs: 19 / 304 / 51
tests/fixtures.rs -> count-fixtures.rs: 5 / 495 / 6
prior01.3/lib.rs -> lib.rs: 2 / 1 / 0
prior01.3/forgejo.rs -> forgejo.rs: 1 / 0 / 0
TOTAL: 3206 moved, 300 new (tests included)
Split reads.rs / keep.rs lines: 604 183
```

The measured total includes 24 added test/fixture lines (19 + 5), module
wiring, and formatting. No speculative extra commentary was inserted into
the copied production bodies. reads.rs is 604 lines and keep.rs 183, both
below 800; the split's 178 unchanged lines count as moved, its five added
lines as new. This is a base rehearsal; the run still copies origin/main
at the step's start and recounts that commit, as README requires.

The formatted copy's new/changed signatures and all targets compile:

```text
$ cargo clippy -p skein-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
    Checking skein-forge v0.0.0 (/srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b/crates/skein-forge)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 2.45s
```

```text
$ cargo nextest run -p skein-forge
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
    Finished `test` profile [unoptimized + debuginfo] target(s) in 2.34s
    Starting 90 tests across 1 binary
        PASS [   0.002s] (86/90) skein-forge tests::a_marker_round_trips_its_key_and_keeps_the_text
        PASS [   0.002s] (87/90) skein-forge tests::a_full_table_drops_the_entry_used_longest_ago
        PASS [   0.002s] (88/90) skein-forge tests::a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place
        PASS [   0.003s] (90/90) skein-forge tests::a_body_without_a_well_formed_marker_has_no_key
     Summary [   0.040s] 90 tests run: 90 passed, 0 skipped
```

```text
$ rg -n -i temper crates/skein-forge
Directory: /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
No matches (expected exit 1).
```

### Estimates including new tests

These are planning estimates, not measured code or tests claimed to run.
01.3 alone was measured with the real copy above.

| 01.4 addition | Estimated new lines |
|---|---:|
| Existing production estimate (starts, credential kinds, Host adaptation) | 195 |
| host_uses_names_and_omits_both_default_ports | 28 |
| identity_start_sends_user_and_returns_id | 12 |
| page_cap_start_returns_positive_cap | 12 |
| zero_page_cap_is_unanswered | 12 |
| start_status_preserves_status_and_retry | 12 |
| bad_start_answer_is_unanswered | 24 |
| token_authorization_has_the_dialect_prefix | 8 |
| credential_debug_contains_no_secret | 8 |
| Shared fixture additions, imports and test module wiring | 9 |
| Total | 320 |

The Host test that moves retains its name; it is not a ninth new test.
The 125 new test/fixture lines above were missing from the old Size.

| 01.5 addition | Estimated new lines |
|---|---:|
| Existing production estimate for the settled differences | 220 |
| calls.rs's 12 tests, canned cases and their shared fixture data | 430 |
| edges.rs's 15 tests, boundary matrices and their fixture data | 350 |
| Shared Run/Bounds/Heard feeding and remaining imports/module wiring | 100 |
| Total | 1,100 |

The 430 includes 32 read/write cases plus the other ten call tests. The
350 includes the bound, path-retention, raw-room, header, status, id,
post-merge, delivery and marker cases. Helpers/data belong in the one
shared fixtures module as the plan already requires; their estimates
are attributed once here, not once per test. None has a base source
for organization's move count. Keeping test functions under 40 lines
does not remove fixture code from Size.

Moving Items' Date change would need about 10 production lines and 25
test/fixture lines and would make 01.4 about 355, while 01.5 would still
exceed 1,000. No self-contained transfer into 01.4 satisfies the ruling's
at-most-about-300 condition. Splitting again violates the explicit five
increment limit; dropping coverage violates the design. The planner must
rule on the excess (or relax the increment limit); it is not silently
approved here. Production alone remains below 300 in both increments.

All existing Proof and Done when commands are unchanged; their base forms
are recorded earlier. The scratch also runs the skein-forge command itself
through 01.3, as recorded above. Planned cumulative counts remain
2 / 86 / 90 / 99 / 126. 01.5's planned tests have not been implemented here.

### Closing audit and cleanup

```text
$ python3 -c 'from pathlib import Path
p=Path("docs/development/plans/forge-kit/skein/01-forge.md")
lines=p.read_text().splitlines()
starts=[i for i,l in enumerate(lines) if l.startswith("### ")]+[lines.index("## Done when")]
assert len(lines)<=400 and len(starts)==6
print("Session:",len(lines),"lines; five increments")
for a,b in zip(starts,starts[1:]):
 block=lines[a:b]
 for field in ["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]:
  assert sum(l.startswith(field+":") for l in block)==1, field
 assert next(l for l in block if l.startswith("Proof:")).startswith("Proof: unit;")
 print(block[0].split()[1],str(b-a)+" lines;",next(l for l in block if l.startswith("Size:")))
s="\n".join(lines)
for text in ["OpenPull before POST -> TooLarge (Q7)","asserts no cost","marker::hex calls use local hex-digit expressions","reads::keep::long","pub fn remark(word: &[u8]","Range::Tail(4)","Range::Head(4)","hex stays pub(crate)","sends 2; POST 429"]:
 assert text in s, text
for text in ["pending the designer","Send count","count 2;","add the corresponding edge rows"]:
 assert text not in s, text
assert not any(l!=l.rstrip() for l in lines)
print("All nine fields; settled answers; six review corrections; no trailing whitespace")
print("Planned cumulative tests: 2 / 86 / 90 / 99 / 126")
print("Planner size ruling remains: 01.4 ~320 new; 01.5 ~1100 new")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
Session: 400 lines; five increments
01.1 30 lines; Size: about 450 lines moved, 30 new.
01.2 24 lines; Size: about 7,820 lines moved, 60 new.
01.3 95 lines; Size: about 3,210 lines moved, 300 new; measured 3,206/300, tests included
01.4 97 lines; Size: about 110 lines moved, 320 new: 195 production plus 125 tests/fixtures (re-check estimate).
01.5 106 lines; Size: about 0 lines moved, 1,100 new: 220 production plus 880 tests/fixtures.
All nine fields; settled answers; six review corrections; no trailing whitespace
Planned cumulative tests: 2 / 86 / 90 / 99 / 126
Planner size ruling remains: 01.4 ~320 new; 01.5 ~1100 new
```

```text
$ git worktree remove --force /srv/data/git/runner/.claude/jobs/ae1f7ad0/tmp/scratch-skein-01b
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; scratch worktree removed
```

`git worktree list --porcelain` confirms the scratch is absent.
Both base worktrees are clean. Only the assigned session file and this
appendix were written persistently; no commit, landing, or other plan
file change was made. The pre-existing STATUS.md edit and other untracked
plan files were left alone.

During this work another session advanced skein-base from 25d2551 to
064c55f (only docs/design/forge-connection.md changed). The scratch was
created explicitly at 25d2551 and all copy/compile evidence above uses
that requested base; no source code changed between those revisions.


