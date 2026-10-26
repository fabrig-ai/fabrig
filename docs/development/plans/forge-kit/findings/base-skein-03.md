# Base evidence: skein 03, forge connection

2026-10-19. temper `27995fb75`; skein `d120820`. Commands below ran
through in the named base worktree; scratch-only changes are described
separately. The session copies origin/main at execution time, not this base.

## Discovery and additional dependencies

The supplied graph projects were queried with search_graph, search_code and
get_file_outline. The component's limits/settings and skein's TLS world paths
returned no entries; the broad Shared query returned older, unrelated paths.
No indexing/status/list operation was called. Names were verified with rg in
the base files, using the permitted fallback for incomplete coverage.

03.4 needs **temper 05.1** and **skein 06.2**, in addition to the README's
temper 03.1: `src/hooks.rs` imports the fake's `delivery::Post`, `signed` calls
`delivery::sign`, and `push` builds a fake `Service` and asks its `delivery`.
The listener's payload builders therefore retain the fake dependency. Taking
these files only after temper 05.1 keeps the copy on skein's already-moved fakes. `machine::case_bytes` reads conformance
transcripts, and the recorded-push test includes its delivery; skein 06.2 must
supply them first. Dropping these dependencies would require rewriting a moved
part or losing a test. No cycle: both needed steps precede temper 07's deletion.

`fixtures::hook_limits` reads the service's small profile. Its five values at
the base, checked against `crates/temper-service/src/forge.rs::small`, are
hooks=4, hook_head_bytes=8192, hook_body_bytes=1048576, hook_deadline=10 s,
hook_secret_bytes=128. The moved world records those plain values because a
kit world cannot depend on a service. Its other fixture limits already live
in its fixtures.rs; `Shared.client` is reduced to its five used numbers.

The full world has 53 tests, not all of which move in 03.4: machine 14,
hooks 18, fuzzy_machine 4 and fuzzy_hooks 1 = **37**. One of hooks.rs's 19
tests, `a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint`,
uses `joint::Joint::hooked`, not Listener. Keep its complete function, name
and assertions with joint.rs for 07.3. The remaining fake_protocol 5,
protocol 9 and fuzzy_protocol 1 also belong to 07.3: 16 tests in all.
This is a source partition, not a test filter in the resulting crate.

## Lists at the base

The JSON output is summarized by module/binary below (no test was filtered out
of its assigned profile). The raw output is retained in the test log.

```text
$ cargo nextest list -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world --message-format json
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
Component: root=17, connection=20, hooks=12, reads=27, writes=18, contents=23; total=117
World default: machine=14, hooks=19, fake_protocol=5, protocol=9; total=47
Log: /srv/data/git/runner/src/tmp/logs/20261019T003855-735.log
```

This is the base form of listing skein-forge-connection and
skein-forge-machine-world; the latter keeps only 32 machine/listener tests; the joint story stays for 07.3.

```text
$ cargo nextest list -p temper-engine-protocol-forge-world --profile fuzzy --message-format json
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
World fuzzy: fuzzy_machine=4, fuzzy_hooks=1, fuzzy_protocol=1; total=6
Log: /srv/data/git/runner/src/tmp/logs/20261019T004127-738.log
```

## Test runs

The component command is the base form of the session's connection command.
At 03.1 it must retain 117 tests and add 2; 03.2 adds 6, 03.3 adds 6:
119/125/131. New behavior is not claimed tested at this base.

```text
$ cargo nextest run -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.10s
     Summary [   0.082s] 117 tests run: 117 passed, 0 skipped
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
     Summary [   0.147s] 47 tests run: 47 passed, 0 skipped
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.11s
        PASS [   0.317s] (1/6) temper-engine-protocol-forge-world::fuzzy_protocol stories_through_cut_streams_answer_each_call_once
        PASS [   0.332s] (2/6) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   1.937s] (3/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   2.010s] (4/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   2.525s] (5/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
        PASS [   2.574s] (6/6) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
     Summary [   2.575s] 6 tests run: 6 passed, 0 skipped
```

The full-world runs preserve the prompt's base commands. The next two are
initial file-level base forms for skein-forge-machine-world. The focused one
still includes the joint story discovered in the final read, so a subsequent
32-test command below isolates the actual move. The new crate's unfiltered
command runs 32 focused/5 fuzzy tests; 03.5 adds 4 focused/2 fuzzy tests.

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test machine --test hooks
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
     Summary [   0.118s] 33 tests run: 33 passed, 0 skipped
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --profile fuzzy --test fuzzy_machine --test fuzzy_hooks
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.24s
        PASS [   0.372s] (1/5) temper-engine-protocol-forge-world::fuzzy_hooks a_delivery_split_at_every_point_is_answered_alike
        PASS [   4.513s] (2/5) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_2
        PASS [   4.791s] (3/5) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_3
        PASS [   4.960s] (4/5) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once_seed_1
        PASS [   5.166s] (5/5) temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
     Summary [   5.167s] 5 tests run: 5 passed, 0 skipped
```

### Moved module times

| Component module | Tests at base | Sum of PASS times | Largest test |
|---|---:|---:|---:|
| root | 17 | 0.043 s | 0.005 s |
| connection | 20 | 0.058 s | 0.006 s |
| contents | 23 | 0.185 s | 0.015 s |
| hooks | 12 | 0.026 s | 0.003 s |
| reads | 27 | 0.142 s | 0.014 s |
| writes | 18 | 0.145 s | 0.014 s |

The 68 harness plan tests are reads 27 + writes 18 + contents 23.

| Source world binary (including the joint story) | Tests | Sum of PASS times | Largest test |
|---|---:|---:|---:|
| hooks | 19 | 0.209 s | 0.097 s |
| machine | 14 | 0.509 s | 0.100 s |

The four fuzzy_machine tests rebuild a component and run both startup reads
and repository resolution at every byte cut of the recorded branch response.
They have independent seeds 0–3, and each iterates the entire response length.
This explains their repeated setup and decoding cost; no profile-based claim
of a particular hot function is made. The run report must retain each test's
cost and cause, without trimming cuts, changing seeds or optimizing builds.

The planning rerun was faster than base-runs.md (2.575 s full fuzzy, 5.167 s
selected fuzzy under different concurrent load). The existing accepted costs
remain 7.635/7.544/7.709/7.481 s for the unsuffixed/_seed_1/_seed_2/_seed_3
tests. Decisions 11 and 18 remain unchanged; these reruns do not reset the
run's base or its 15/60 s budgets.

## Pending design and ready-check limits

The session carries questions 1, 2, 3, 6 and 13's recommendations, with 13
also depending on 14's answer. No answer is settled here. The credential-kind
path in the sketch is provisional: skein 01.4 owns its final spelling and the
supervisor must compile against that landed interface. The scratch compiles
changed boundary shapes against base libraries; it cannot test the absent
upstream plan implementation. There are no new design questions from this
survey. The two extra 03.4 dependencies must be reconciled into the README
by its owner before landing the plan.

## Lint and neutrality commands

Base form of the session's two-package clippy command:

```text
$ cargo clippy -p temper-engine-protocol-forge -p temper-engine-protocol-forge-world --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.84s
```

Base form of the final no-service-name scan. Matches are expected at the
base; the future skein paths must print nothing and return rg's no-match
status 1. This base run is evidence of work to do, not a passing neutrality
check. There were 195 matching source/manifest lines.

```text
$ rg -n -i temper crates/temper-engine-protocol-forge tests/engine/protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests/engine/protocol-forge/tests/protocol.rs:2:use temper_engine_domain_forge as top;
tests/engine/protocol-forge/tests/protocol.rs:3:use temper_engine_domain_forge_change as change;
tests/engine/protocol-forge/tests/protocol.rs:4:use temper_engine_domain_forge_client as client;
tests/engine/protocol-forge/tests/protocol.rs:5:use temper_engine_domain_forge_issues as issues;
[remaining matches omitted]
```

## Compiled interface sketch

Only the authorized detached scratch worktree was changed. It contained one
new crate, forge-connection-sketch, with the session's complete Settings,
Forge and Shared boundary declarations, the Word/UserAgent/TokenValue variants,
and declaration-only full_heap/tls_cuts helper signatures. Moved Kind,
Transport and Webhooks use their base paths. CredentialKind::Token is an
explicit stand-in for skein 01.4's absent enum, not a proposed second enum
in the component. No implementation or behavior proof is claimed by this
compilation. The ready check must use the landed credential-kind path.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-03 27995fb75
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Preparing worktree (detached HEAD 27995fb75)
HEAD is now at 27995fb75 Point temper's forge documents to skein's forge kit
```

The first compile exposed a scratch manifest error: temper lists members
explicitly. The correction added only the scratch crate's member entry in
that worktree, then reran the same command.

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03
error: package ID specification `forge-connection-sketch` did not match any packages
```

Corrected run, with all declarations included:

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03
Checking forge-connection-sketch v0.0.0 (/srv/data/git/runner/src/tmp/planning/scratch-skein-03/crates/forge-connection-sketch)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 5.98s
```

The scratch worktree was removed after that pass:

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-03
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; no output
```

## Final source partition: the 32 focused tests that move

The source-files run had 33 tests; this base-only selection excludes the one
Joint story that 07.3 keeps. The moved crate itself has no filter or ignored
test. This supersedes the first file-level count, preserving that run as evidence.

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test machine --test hooks -E 'not test(=a_domain_push_reaches_the_component_as_a_signed_delivery_and_a_hint)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.17s
    Starting 32 tests across 2 binaries (1 test skipped)
     Summary [   0.082s] 32 tests run: 32 passed, 1 skipped
```

| Exact moved module | Count | Sum | Maximum |
|---|---:|---:|---:|
| hooks | 18 | 0.168 s | 0.072 s |
| machine | 14 | 0.363 s | 0.066 s |

## User and size searches

Counts below are lexical occurrences, including definitions, not graph call
edges. Common names such as Limits occur in unrelated components; the crate
and conventions searches identify the actual consumers. All five repositories
were searched. Generated/retired temper.old sources are excluded. Generic
name scans deliberately include other code so a copy is not missed.

The initial `python3 -` heredoc submissions were unusable: queued jobs
do not receive the caller's stdin. The running count job was stopped and its
queued counterpart removed, then both scripts were resubmitted with `-c`.
No result from those stdin-based submissions is used as count evidence.

```text
$ pueue kill 746 764
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Tasks are being killed: 746
The command failed for tasks: 764
```
```text
$ pueue remove 764
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Tasks removed from list: 764
```

The corrected command (output table below regroups its lines):

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
root=Path('\''/srv/data/git/runner/src'\'')
repos=[root/'\''tmp/planning/temper-base'\'',root/'\''tmp/planning/skein-base'\'',root/'\''jig'\'',root/'\''smith'\'',root/'\''hearth'\'']
patterns=[r'\''temper_engine_protocol_forge::'\'',r'\''conventions::host'\'',r'\''USER_AGENT'\'',r'\''temper_engine_protocol_forge_world::'\'',r'\''\b(Settings|Shared|Forge|Unusable|Limits|Component)\b'\'',r'\''\b(check|page_size|largest_row|worst_case|host|signature|heard)\b'\'',r'\''\b(word|user_agent|credential|token_bytes|answer_bytes|body_bytes|hook_limits|case_bytes)\b'\'']
for pat in patterns:
 print('\''pattern:'\'',pat)
 for repo in repos:
  p=subprocess.run(['\''rg'\'','\''-n'\'','\''--glob'\'','\''*.rs'\'','\''--glob'\'','\''!**/temper.old/**'\'',pat,str(repo)],capture_output=True,text=True)
  lines=p.stdout.splitlines(); print(repo.name, len(lines),'\''lines'\'',len({l.split('\'':'\'')[0] for l in lines}),'\''files'\'', '\''exit'\'',p.returncode)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
```

| Pattern | temper lines/files | skein lines/files | jig lines/files | smith lines/files | hearth lines/files |
|---|---:|---:|---:|---:|---:|
| `temper_engine_protocol_forge::` | 40/25 | 0/0 | 0/0 | 0/0 | 0/0 |
| `conventions::host` | 4/4 | 0/0 | 0/0 | 0/0 | 0/0 |
| `USER_AGENT` | 3/3 | 4/2 | 0/0 | 0/0 | 0/0 |
| `temper_engine_protocol_forge_world::` | 12/11 | 0/0 | 0/0 | 0/0 | 0/0 |
| `\b(Settings\|Shared\|Forge\|Unusable\|Limits\|Component)\b` | 4301/428 | 2629/320 | 19963/880 | 4098/345 | 999/247 |
| `\b(check\|page_size\|largest_row\|worst_case\|host\|signature\|heard)\b` | 2698/344 | 1682/342 | 7570/606 | 5689/467 | 523/154 |
| `\b(word\|user_agent\|credential\|token_bytes\|answer_bytes\|body_bytes\|hook_limits\|case_bytes)\b` | 561/162 | 437/104 | 1406/305 | 737/202 | 103/27 |

The inventory below searches every function/type/constant name in the moved
production and world-driver files. It also counts base lines per file. The
3,021 production lines still contain the portions skein 01 will extract from
connection/start; the session therefore estimates about 3,000 production
lines, plus 3,178 test lines. World drivers total 728 lines, plus roughly
200 lines of retained fixtures and its manifests = about 950 support lines.
The four test files total 539 lines; leaving the 14-line Joint story for
07.3 gives about 525 moved test lines. All counts precede rename edits.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess, collections
root=Path('\''/srv/data/git/runner/src'\''); base=root/'\''tmp/planning/temper-base'\''
k=base/'\''crates/temper-engine-protocol-forge/src'\''; w=base/'\''tests/engine/protocol-forge'\''
files=[k/x for x in ['\''lib.rs'\'','\''component.rs'\'','\''component/calls.rs'\'','\''connection.rs'\'','\''hooks.rs'\'','\''hooks/connection.rs'\'','\''start.rs'\'','\''limits.rs'\'','\''grow.rs'\'','\''boundary.rs'\'','\''settings.rs'\'','\''conventions.rs'\'']]
world=[w/'\''src'\''/x for x in ['\''machine.rs'\'','\''hooks.rs'\'','\''peer.rs'\'','\''pipe.rs'\'']]
names=sorted(set(re.findall(r'\''\b(?:fn|struct|enum|const)\s+(\w+)'\'', '\''\n'\''.join(p.read_text() for p in files+world))))
pattern=r'\''\b(?:'\''+'\''|'\''.join(names)+r'\'')\b'\''
for repo in [base,root/'\''tmp/planning/skein-base'\'',root/'\''jig'\'',root/'\''smith'\'',root/'\''hearth'\'']:
 out=subprocess.run(['\''rg'\'','\''-o'\'','\''--no-filename'\'','\''--glob'\'','\''*.rs'\'','\''--glob'\'','\''!**/temper.old/**'\'',pattern,str(repo)],capture_output=True,text=True)
 counts=collections.Counter(out.stdout.splitlines()); print(repo.name+'\'': '\''+'\'', '\''.join(n+'\''='\''+str(counts[n]) for n in names))
for group,paths in [('\''component production'\'',files),('\''component tests'\'',[k/'\''tests.rs'\'',*sorted((k/'\''tests'\'').glob('\''*.rs'\''))]),('\''world support'\'',world),('\''world tests'\'',[w/'\''tests'\''/x for x in ['\''machine.rs'\'','\''hooks.rs'\'','\''fuzzy_machine.rs'\'','\''fuzzy_hooks.rs'\'']])]:
 print(group+'\'': '\''+str(sum(len(p.read_text().splitlines()) for p in paths))+'\'' lines; '\''+'\'', '\''.join(p.name+'\''='\''+str(len(p.read_text().splitlines())) for p in paths))'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper-base: Answer=3169, Answered=193, Body=278, Closing=91, Component=72, Connect=43, Connection=34, Cut=91, Diagnostics=5, End=303, Event=3575, Failure=326, Fields=15, Forge=492, Full=76, Grow=12, Heard=145, Held=236, HintState=11, Hinted=13, Hooked=8, Hooks=54, Kind=835, Limits=3181, Listener=24, Listing=62, MAX_OUT=93, Machine=49, MaxOut=9, OWNER=64, Peer=79, Phase=872, Pipe=38, Receiving=32, Request=5080, Response=32, Settings=575, Shared=97, Stacked=23, Start=405, StartRead=21, Stopped=107, Tail=30, Transport=103, USER_AGENT=3, Unusable=117, Waiting=256, Webhooks=22, aborts=14, accept=51, answered=496, assign=26, backoff=67, begin=74, body=2308, busy=90, call=1386, call_step=3, capacity=374, carried=20, case=526, case_bytes=5, check=712, check_settings=2, client_down=3, client_up=5, close=407, closed=347, connect=25, connections_opened=16, connects=9, cut=285, deadline=485, debug=10, decoder=222, diagnostics=26, documents=219, down=464, drive=193, drive_calls=2, drive_room=5, drive_starts=2, dropped=96, end=661, events=545, expire=9, failed=507, field=2093, finish=1243, fire=168, fmt=92, has_work=9, heard=211, hinted_after_answer=3, hints=41, hook_documents=9, hook_heap=2, hook_http=7, host=1295, http=477, http_down=21, http_env=10, http_event=4, in_flight=6, io=1310, json_down=35, json_event=2, json_request=3, keep_after=8, largest_row=2, len=2223, lose=3, new=8769, next_deadline=189, offer=27, open_start=3, owns=89, page_size=31, peer=322, pending=954, pump=37, pump_call=2, push=2768, put=1167, raw_demand=4, raw_request=9, raw_room=12, read=1768, ready=296, reclaim=156, refuse=29, refused=466, request=2313, requests=348, respond=13, response=301, room=865, routing_bytes=2, run_for=215, script_done=8, send=599, sending=11, server_down=3, server_up=3, settle=382, signature=39, signed=55, stack=29, start=854, start_heard=3, start_request=4, status=1124, statuses=341, stopped=114, storage=4, stream=224, tables=156, tick=96, timeout=27, tls_event=2, up=599, upload=33, verified=44, with=751, worst_case=620, write=1210
skein-base: Answer=148, Answered=41, Body=668, Closing=354, Component=95, Connect=243, Connection=95, Cut=118, Diagnostics=0, End=362, Event=4380, Failure=556, Fields=11, Forge=0, Full=41, Grow=3, Heard=23, Held=144, HintState=0, Hinted=0, Hooked=0, Hooks=0, Kind=905, Limits=2388, Listener=182, Listing=13, MAX_OUT=230, Machine=418, MaxOut=128, OWNER=215, Peer=137, Phase=1104, Pipe=130, Receiving=60, Request=2733, Response=155, Settings=227, Shared=53, Stacked=5, Start=175, StartRead=0, Stopped=57, Tail=14, Transport=143, USER_AGENT=2, Unusable=25, Waiting=516, Webhooks=0, aborts=20, accept=314, answered=294, assign=0, backoff=17, begin=29, body=1977, busy=62, call=1591, call_step=0, capacity=574, carried=20, case=287, case_bytes=0, check=499, check_settings=0, client_down=13, client_up=5, close=1690, closed=737, connect=210, connections_opened=0, connects=60, cut=406, deadline=509, debug=6, decoder=255, diagnostics=11, documents=215, down=1738, drive=124, drive_calls=0, drive_room=0, drive_starts=0, dropped=104, end=1604, events=2128, expire=41, failed=711, field=1390, finish=654, fire=181, fmt=156, has_work=74, heard=39, hinted_after_answer=0, hints=0, hook_documents=0, hook_heap=0, hook_http=0, host=449, http=849, http_down=16, http_env=5, http_event=11, in_flight=40, io=3024, json_down=6, json_event=2, json_request=2, keep_after=0, largest_row=0, len=3818, lose=2, new=6432, next_deadline=289, offer=6, open_start=0, owns=102, page_size=0, peer=1030, pending=619, pump=21, pump_call=0, push=2059, put=583, raw_demand=0, raw_request=5, raw_room=0, read=2723, ready=358, reclaim=155, refuse=51, refused=435, request=2132, requests=1108, respond=61, response=1397, room=2145, routing_bytes=0, run_for=0, script_done=0, send=632, sending=53, server_down=20, server_up=0, settle=282, signature=101, signed=78, stack=124, start=1012, start_heard=0, start_request=0, status=473, statuses=4, stopped=151, storage=94, stream=2050, tables=445, tick=102, timeout=52, tls_event=0, up=1774, upload=233, verified=5, with=1177, worst_case=686, write=773
jig: Answer=2304, Answered=298, Body=979, Closing=850, Component=177, Connect=12, Connection=38, Cut=312, Diagnostics=0, End=940, Event=10206, Failure=515, Fields=35, Forge=0, Full=120, Grow=0, Heard=190, Held=1817, HintState=0, Hinted=0, Hooked=0, Hooks=0, Kind=995, Limits=19505, Listener=4, Listing=0, MAX_OUT=120, Machine=50, MaxOut=1, OWNER=132, Peer=109, Phase=2629, Pipe=4, Receiving=5, Request=7257, Response=14, Settings=439, Shared=40, Stacked=0, Start=306, StartRead=0, Stopped=307, Tail=2, Transport=90, USER_AGENT=0, Unusable=64, Waiting=1304, Webhooks=0, aborts=6, accept=97, answered=594, assign=84, backoff=98, begin=104, body=1491, busy=311, call=2593, call_step=0, capacity=2085, carried=101, case=160, case_bytes=0, check=4167, check_settings=0, client_down=0, client_up=0, close=606, closed=518, connect=35, connections_opened=0, connects=0, cut=3073, deadline=2262, debug=13, decoder=15, diagnostics=29, documents=2057, down=716, drive=1326, drive_calls=0, drive_room=0, drive_starts=0, dropped=187, end=1617, events=836, expire=45, failed=695, field=12799, finish=8271, fire=266, fmt=41, has_work=4, heard=103, hinted_after_answer=0, hints=17, hook_documents=0, hook_heap=0, hook_http=0, host=1993, http=327, http_down=0, http_env=3, http_event=0, in_flight=53, io=694, json_down=0, json_event=0, json_request=0, keep_after=0, largest_row=0, len=5268, lose=21, new=48114, next_deadline=236, offer=213, open_start=0, owns=190, page_size=0, peer=763, pending=1448, pump=51, pump_call=0, push=7401, put=6451, raw_demand=0, raw_request=0, raw_room=0, read=3503, ready=631, reclaim=172, refuse=163, refused=1196, request=4244, requests=1312, respond=83, response=144, room=2473, routing_bytes=0, run_for=0, script_done=0, send=1220, sending=45, server_down=0, server_up=0, settle=275, signature=0, signed=130, stack=35, start=841, start_heard=0, start_request=0, status=572, statuses=6, stopped=175, storage=51, stream=893, tables=15, tick=141, timeout=22, tls_event=0, up=955, upload=19, verified=31, with=2445, worst_case=1580, write=494
smith: Answer=1063, Answered=386, Body=61, Closing=292, Component=130, Connect=57, Connection=16, Cut=118, Diagnostics=1, End=976, Event=5395, Failure=843, Fields=17, Forge=0, Full=44, Grow=0, Heard=7, Held=102, HintState=0, Hinted=0, Hooked=0, Hooks=0, Kind=465, Limits=3378, Listener=0, Listing=30, MAX_OUT=91, Machine=132, MaxOut=18, OWNER=45, Peer=145, Phase=876, Pipe=23, Receiving=117, Request=2743, Response=226, Settings=689, Shared=55, Stacked=0, Start=480, StartRead=0, Stopped=103, Tail=0, Transport=56, USER_AGENT=0, Unusable=0, Waiting=355, Webhooks=0, aborts=7, accept=10, answered=783, assign=0, backoff=37, begin=48, body=573, busy=79, call=3482, call_step=0, capacity=281, carried=67, case=139, case_bytes=0, check=1813, check_settings=0, client_down=0, client_up=0, close=548, closed=320, connect=64, connections_opened=0, connects=1, cut=407, deadline=1298, debug=3, decoder=51, diagnostics=15, documents=46, down=270, drive=106, drive_calls=0, drive_room=0, drive_starts=0, dropped=416, end=1770, events=943, expire=11, failed=642, field=1862, finish=1637, fire=216, fmt=58, has_work=15, heard=36, hinted_after_answer=0, hints=0, hook_documents=0, hook_heap=0, hook_http=0, host=3913, http=128, http_down=0, http_env=0, http_event=0, in_flight=13, io=1419, json_down=0, json_event=0, json_request=0, keep_after=0, largest_row=0, len=2756, lose=2, new=10784, next_deadline=224, offer=45, open_start=0, owns=289, page_size=0, peer=1103, pending=438, pump=5, pump_call=0, push=2295, put=1219, raw_demand=0, raw_request=0, raw_room=0, read=2116, ready=175, reclaim=222, refuse=92, refused=513, request=1782, requests=548, respond=0, response=827, room=745, routing_bytes=0, run_for=0, script_done=0, send=153, sending=35, server_down=0, server_up=0, settle=482, signature=30, signed=23, stack=4, start=1374, start_heard=0, start_request=0, status=405, statuses=0, stopped=119, storage=35, stream=514, tables=5, tick=28, timeout=274, tls_event=0, up=420, upload=0, verified=12, with=1108, worst_case=672, write=835
hearth: Answer=446, Answered=0, Body=11, Closing=13, Component=128, Connect=3, Connection=2, Cut=39, Diagnostics=0, End=111, Event=1136, Failure=114, Fields=28, Forge=0, Full=33, Grow=0, Heard=0, Held=44, HintState=0, Hinted=0, Hooked=0, Hooks=0, Kind=1252, Limits=735, Listener=0, Listing=60, MAX_OUT=18, Machine=99, MaxOut=42, OWNER=87, Peer=0, Phase=50, Pipe=73, Receiving=0, Request=996, Response=0, Settings=135, Shared=73, Stacked=0, Start=41, StartRead=0, Stopped=7, Tail=31, Transport=0, USER_AGENT=0, Unusable=18, Waiting=99, Webhooks=0, aborts=0, accept=3, answered=5, assign=8, backoff=0, begin=7, body=384, busy=8, call=591, call_step=0, capacity=296, carried=0, case=441, case_bytes=0, check=222, check_settings=0, client_down=0, client_up=0, close=158, closed=68, connect=9, connections_opened=0, connects=0, cut=111, deadline=34, debug=4, decoder=146, diagnostics=3, documents=493, down=159, drive=10, drive_calls=0, drive_room=0, drive_starts=0, dropped=14, end=433, events=128, expire=2, failed=158, field=1098, finish=181, fire=21, fmt=28, has_work=0, heard=0, hinted_after_answer=0, hints=13, hook_documents=0, hook_heap=0, hook_http=0, host=158, http=69, http_down=0, http_env=0, http_event=0, in_flight=0, io=297, json_down=0, json_event=0, json_request=0, keep_after=0, largest_row=0, len=1362, lose=0, new=2462, next_deadline=41, offer=0, open_start=0, owns=30, page_size=0, peer=23, pending=304, pump=0, pump_call=0, push=1316, put=879, raw_demand=0, raw_request=0, raw_room=0, read=643, ready=93, reclaim=5, refuse=8, refused=155, request=835, requests=145, respond=0, response=7, room=285, routing_bytes=0, run_for=0, script_done=0, send=15, sending=8, server_down=0, server_up=0, settle=33, signature=7, signed=15, stack=106, start=338, start_heard=0, start_request=0, status=23, statuses=0, stopped=0, storage=10, stream=168, tables=2, tick=6, timeout=11, tls_event=0, up=195, upload=0, verified=1, with=124, worst_case=155, write=190
component production: 3021 lines; lib.rs=28, component.rs=581, calls.rs=322, connection.rs=746, hooks.rs=181, connection.rs=321, start.rs=59, limits.rs=494, grow.rs=104, boundary.rs=70, settings.rs=48, conventions.rs=67
component tests: 3178 lines; tests.rs=355, connection.rs=370, contents.rs=708, fixtures.rs=501, hooks.rs=135, reads.rs=658, writes.rs=451
world support: 728 lines; machine.rs=320, hooks.rs=243, peer.rs=35, pipe.rs=130
world tests: 539 lines; machine.rs=262, hooks.rs=215, fuzzy_machine.rs=43, fuzzy_hooks.rs=19
```

### Alias imports and affected owners

The exact `crate::` search misses `use ... as kit`; this last scan includes
those imports. The 51 temper lines below include 4 internal definitions/uses;
external owner files are in the listed crates/worlds. The world-crate prefix
search above also finds forge-setup/reconcile's borrowed fixtures and Pipe.
Temper 03 owns the component-user changes; 07 owns the residual shared-world
imports/deletion. None of jig/smith/hearth names this component. Skein's four
USER_AGENT lines are unrelated HTTP-request tests and stay untouched.

```text
$ python3 -c 'from pathlib import Path
import subprocess,collections
root=Path("/srv/data/git/runner/src")
for repo in [root/"tmp/planning/temper-base",root/"tmp/planning/skein-base",root/"jig",root/"smith",root/"hearth"]:
 p=subprocess.run(["rg","-n","--glob","*.rs","--glob","!**/temper.old/**",r"\btemper_engine_protocol_forge\b|conventions::host|USER_AGENT",str(repo)],capture_output=True,text=True)
 lines=p.stdout.splitlines(); print(repo.name,len(lines),"lines",len({x.split(":")[0] for x in lines}),"files")
 if repo.name=="temper-base":
  counts=collections.Counter("/".join(Path(x.split(":")[0]).relative_to(repo).parts[:2]) for x in lines)
  print("owners:",dict(sorted(counts.items())))
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
temper-base 51 lines 32 files
owners: {'crates/temper': 5, 'crates/temper-engine-protocol-forge': 4, 'crates/temper-forge-setup-shell': 2, 'crates/temper-service': 19, 'tests/codecs': 3, 'tests/engine': 13, 'tests/one-process': 1, 'tools/forgejo-conformance': 4}
skein-base 4 lines 2 files
jig 0 lines 0 files
smith 0 lines 0 files
hearth 0 lines 0 files
```

All terminal commands used for proofs, lists, counts and lint are recorded
above. Exploratory reads used cat/sed and rg on the named base files; the
additional review checked the one Joint story's complete function, the TLS
world's pki/server and the heap Meter signatures. Final document checks:
385 session lines, every required increment field present, each command block
at most 30 lines, scratch absent, and git diff --check exited 0. No commit or
landing command was run.

## Revision evidence: answers 1, 13/14 and session 03's boundary

2026-10-19. This section supersedes the draft counts, provisional credential
shape, Host recommendation, size estimate and 03.4/03.5 scope above. The
planning inputs are now skein `d9f3e39` and temper `b4cfb999`; their code
is still the original base's. Only skein/03-connection.md and this findings
file are edited. No commit or landing was made.

### Discovery and corrections

The supplied `srv-data-git-runner-src-temper` project was queried with
search_graph for largest_row/page_size/check_settings and the two existing
limit-test names, then search_code scoped to the component for largest_row,
limits_that_disagree and Transport. The graph returned only an unrelated
fake-domain page_size field, and the text query returned zero results.
Relevant component coverage is incomplete; exact base-file rg checks supplied
the names and callers. No list/index/status tool was called.

- Answer 1 is applied: Transport::Plaintext carries name; Forge owns Credential
  (Token(Secret)), not host, token plus a separate kind, or credential::Kind.
  The shared host helper belongs to skein 01.4; the component adds
  PlaintextName { forge } without retaining input bytes. The inherited
  unusable_forge_settings_name_the_forge gains the empty/control/DEL/high-byte,
  separator and adjacent-accepted-byte cases. Both check and send use the
  derived Host; the head check includes its full length and the user agent.
- The base really has 117 component tests. Four marker/table tests named in
  skein 01.3 leave root 13 and component 113. There is a count subtlety: moving
  the fifth, pure Host test away without retaining a component check would
  give 112. The revision preserves its inherited name for component Host
  wiring, while the pure helper vectors live in skein-forge's src/tests.rs in
  skein 01.4. This retains the requested 115/121/127 totals and adds exactly
  14 new test names (2+6+6). This is an explicit test adaptation to the new
  transport boundary, not a second production host helper. The neighboring
  01 draft still says not to build Host; answer 1 and decisions 7 settle that
  prerequisite, and this task cannot edit the neighboring session.
- 03.3 needs skein 02.2's vocabulary measure. The seven independently priced
  rows use forge.md 3.7 widths; private largest_row tests live in
  src/limits/tests.rs, sharing the existing fixtures via pub(crate). The
  existing page-minimum test retains its name and result 1, replacing 4096
  by ROW_ITEMS. The existing ("answer", ...) refusal uses ROW_ITEMS-1,
  still PageSize { listing: Items }. No pending 13/14 recommendation remains.
- Public limits::check checks all seven listings in one pass, returning the
  first failure. It cannot accept at each smaller listing's individual row
  size: for example Collaborators needs 16 but Items needs 2971. The new test
  therefore checks all seven together at MAX_ROW-1/at MAX_ROW (2970/2971),
  refusing Items/accepting; the separate seven-listing table proves each
  individual page_size's ROW-1/ROW/2*ROW -> 0/1/2 against measure::answer.
  The signature-length cases join 03.2's signature-shape test, leaving room
  for this sixth 03.3 test without changing the 14-new-test arithmetic.
- The body bound changes from 2*512+21=1045 to 2*512+3+15=1042 for kit.
  The moved BodyBytes row changes 1044/1045 to 1041/1042; fixtures::limits'
  body_bytes changes 1045 to 1042. The owner-word test raises body room for
  its valid 16-byte word so a different refusal cannot hide its assertion.
- Temper's side is named by file and symbol in each Shape. At the base,
  temper-service/src/forge.rs constructs Forge at 61, shared at 84,
  problems at 206 and indexed at 281; temper/src/forge.rs uses host and
  USER_AGENT at 258–259. Component runtime fixtures are service/tests/fixtures
  763, one-process/fixtures 243, conformance/component/hosted 124 and the
  component's tests/fixtures 98–100. system/src/composed/link.rs:34 is a
  different type: temper_service::settings::Transport, a configuration value.
  Its assignment caller is verified at repin; blindly changing it to the
  component's struct variant would be wrong. The page-size check is also
  consumed by service prepare/small/shared and the conformance fixtures.
- The 3021-line raw production sum includes connection.rs's exchange types
  (Fields/Body/Failure/Heard, roughly 50 lines), conventions.rs (67 lines,
  including the residual agent removed here), start::Heard (roughly 14) and
  component's start request/answer mapping (roughly 30). Those are removed
  from the estimate: about 2850 production lines moved, 240 new; not 3000.
  Test lines are about 3100 after the four root tests leave. Execution uses
  the actual origin/main source and the plan's per-file move-count rule.
- The baseline remains base-runs.md: component 117 at 0.149 s; full world
  47 focused at 0.404 s and 6 fuzzy at 7.709 s; fuzzy_machine
  7.635/7.544/7.709/7.481 s. Earlier faster runs here, and the run below,
  are lighter-load reruns. No moved test gets a lower budget from them.
- Decision 23 removes 03.4/03.5 and their commands from this session.
  Skein 08 owns the machine/listener, TLS/fault/deadline/closing and allocator
  proofs after temper 03.1, temper 05.1, skein 05.2 and skein 06.2.
  Its machine::case_bytes uses conformance transcripts; hooks imports the
  fake protocol delivery module. The earlier 07.3 attributions above are
  historical: decisions 22/23 assign component worlds to 08.
- Only Design questions 2, 3 and 6 remain pending; their recommendations
  are explicitly conditional. No serde, alias, re-export or system name
  is introduced into a skein implementation by this plan.

### Commands at the base

The session's future skein package does not exist at the planning base.
These are its base-equivalent commands; the supervisor's ready check runs
the future paths and real upstream interfaces after their prerequisites.
The new behavior and 127-test target are not claimed proved by a base run.

```text
$ cargo nextest run -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `test` profile [unoptimized + debuginfo] target(s) in 0.12s
Starting 117 tests across 1 binary
Summary [   0.073s] 117 tests run: 117 passed, 0 skipped
Log: /srv/data/git/runner/src/tmp/logs/20261019T010856-733.log
```

```text
$ cargo clippy -p temper-engine-protocol-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished `dev` profile [unoptimized + debuginfo] target(s) in 0.16s
Log: /srv/data/git/runner/src/tmp/logs/20261019T010742-734.log
```

The narrowed neutrality scan's base still contains the names to remove;
exit 0 here means matches, not neutrality. At the skein destination its
required result is no output, rg exit 1.

```text
$ rg -n -i temper crates/temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
crates/temper-engine-protocol-forge/Cargo.toml:2:name = "temper-engine-protocol-forge"
crates/temper-engine-protocol-forge/Cargo.toml:16:temper-forge-forgejo.workspace = true
crates/temper-engine-protocol-forge/Cargo.toml:17:temper-engine-domain-forge-client.workspace = true
[remaining matches omitted; complete output is in the test log]
Log: /srv/data/git/runner/src/tmp/logs/20261019T010742-733.log
```

The following count and consumer search also ran directly. Row values
are arithmetic from the settled design and adapted fixture, not execution
of the not-yet-landed measure. Its output is summarized after the command.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
root=Path("/srv/data/git/runner/src/tmp/planning/temper-base")
d=root/"crates/temper-engine-protocol-forge/src"
files=["lib.rs","component.rs","component/calls.rs","connection.rs","hooks.rs","hooks/connection.rs","start.rs","limits.rs","grow.rs","boundary.rs","settings.rs","conventions.rs"]
n=sum(len((d/f).read_text().splitlines()) for f in files)
print("Base production:",n,"lines; conventions:",len((d/"conventions.rs").read_text().splitlines()),"lines")
for f in [d/"tests.rs",*sorted((d/"tests").glob("*.rs"))]:
 print(f.name,len(re.findall(r"^#\[test\]",f.read_text(),re.M)),"tests")
print("Row widths at body=1042, string=128, inner=8:")
print(dict(Items=128+1042+521+128+8*(16+128),Reviews=128+1042+521,Statuses=160+3*128,Remarks=64+1042+128,Branches=16+128,Collaborators=16,PullFiles=64+128+2*1042))
for p in ["crates/temper-service/src/forge.rs","crates/temper/src/forge.rs","crates/temper-service/src/tests/fixtures.rs","tests/one-process/src/fixtures.rs","tools/forgejo-conformance/src/component/hosted.rs","tests/system/src/composed/link.rs","crates/temper-engine-protocol-forge/src/tests/fixtures.rs"]:
 r=subprocess.run(["rg","-n",r"conventions::host|USER_AGENT|Transport::Plaintext|kit::settings::Forge|fn (shared|problems|indexed)|body_bytes: 1045",str(root/p)],capture_output=True,text=True)
 print(p+": "+ " | ".join(line.split(":",1)[0]+":"+line.split(":",1)[1].strip() for line in r.stdout.splitlines()))
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Base production: 3021 lines; conventions: 67 lines
tests.rs 17 tests; connection.rs 20; contents.rs 23; fixtures.rs 0
hooks.rs 12; reads.rs 27; writes.rs 18
Row widths at body=1042, string=128, inner=8:
Items=2971, Reviews=1691, Statuses=544, Remarks=1234, Branches=144, Collaborators=16, PullFiles=2276
Consumer hits: service forge 56/61/84/206/281; binary forge 258/259
Runtime fixtures: service 763; one-process 243; conformance 124
Different configuration enum: system composed link 34
Component fixture: body_bytes 16; shared 46; plaintext 98
Log: /srv/data/git/runner/src/tmp/logs/20261019T010858-734.log
```

### Changed type sketch, compiled on skein-base

Unlike the original sketch, this scratch is a detached worktree of skein-base,
not temper. It compiled the complete changed Settings, Forge, Transport and
Shared declarations, and the added Word, PlaintextName, UserAgent and
TokenValue variants. It uses actual skein-lib/io/tls types, no component
implementation. A scratch-only skein-forge crate supplies exactly 01.4's
Credential::Token(skein_lib::Secret) declaration because that crate has not
landed at the base; this is not a second proposed credential kind.
Kind/Webhooks retain their base declarations. No function signature changes
in 03.2/03.3; unchanged page_size/largest_row are named, not reimplemented.
The ready check must still compile against the landed skein-forge.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-03-revision d9f3e39
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD d9f3e39)
HEAD is now at d9f3e39 Answer the forge kit planner's shape questions
```

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03-revision
Checking skein-forge v0.0.0 (scratch-only credential declaration)
Checking forge-connection-sketch v0.0.0 (scratch-only changed declarations)
Finished `dev` profile [unoptimized + debuginfo] target(s) in 4.01s
Log: /srv/data/git/runner/src/tmp/logs/20261019T010822-733.log
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-03-revision
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```






### Final document verification

```text
$ python3 -c 'from pathlib import Path
import re
root=Path("/srv/data/git/runner/src/tmp/planning/temper-forge-kit")
p=root/"docs/development/plans/forge-kit/skein/03-connection.md"
s=p.read_text()
assert len(s.splitlines()) <= 400
parts=re.split(r"(?=^### 03\.)",s,flags=re.M)[1:]
assert len(parts)==3
for part in parts:
 for label in ["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]:
  assert re.search("^"+re.escape(label)+":",part,re.M),label
assert all(line.startswith("Proof: world,") for line in s.splitlines() if line.startswith("Proof:"))
assert "113 inherited component test identities +14 new (2+6+6)" in s
for count in ["115 passed","121 passed","127 passed"]:
 assert count in s,count
for pattern in ["credential::Kind","pub host:","### 03.4","### 03.5"]:
 assert pattern not in s,pattern
for f in [p,p.parents[1]/"findings/base-skein-03.md"]:
 assert not any(line.rstrip()!=line for line in f.read_text().splitlines()),str(f)
assert not Path("/srv/data/git/runner/src/tmp/planning/scratch-skein-03-revision").exists()
print("Session:",len(s.splitlines()),"lines; 3 increments; all fields and 115/121/127 counts present")
print("No stale host field, credential::Kind or 03.4/03.5; whitespace clean; scratch absent")
print("Pending behavior remains Design questions 2, 3 and 6; future package checked when ready")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
Session: 308 lines; 3 increments; all fields and 115/121/127 counts present
No stale host field, credential::Kind or 03.4/03.5; whitespace clean; scratch absent
Pending behavior remains Design questions 2, 3 and 6; future package checked when ready
```

```text
$ git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
exit 0; no output (plan files are untracked; the explicit whitespace check above reads them)
```

```text
$ git status --short
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --short
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; no output
```

## Second revision: answers 2–6, retaining answer 1

2026-10-19. Inputs: skein-base `8938e16`, temper-base `dd12b8e`,
the whole planning section and current README/code-organization/decisions,
both designer answer files, and reviews/skein-03.txt. This section supersedes
every earlier pending mark for questions 1–6, the proposed UserAgent and
TokenValue variants, and the draft's allocation guard. The earlier Host,
measure, fixture-body, consumer, baseline-time and session-boundary fixes stay.

### Audit and resulting plan

- The supplied graph projects were queried with search_graph and search_code.
  The component query returned unrelated older symbols; regex text queries
  scoped to its current path, and skein-http's status literals, returned none.
  Exact-path rg and source reads supplied the missing coverage. No list,
  index or status operation was called.
- Answer 2: base limits::check_settings already rejects empty/oversize tokens
  as Token { forge } and user 0 as User { forge }. 03.2 adds forbidden-control
  checking to Token and uses the existing Request { forge } for an empty or
  unsafe owner agent, at each forge's head check. No new answer-2 variant.
  Tests cover both values independently: empty, NUL, CR, LF, 0x1f, DEL refused;
  tab, space, 0x7e and high bytes accepted; user 0 refused and 1 accepted.
  Token/Request's descriptions and temper-service's existing Request mapping
  reflect validity as well as size. No credential bytes appear in a refusal.
- Answer 3: hooks/connection.rs::signature uses is_ascii_hexdigit at the base,
  admitting uppercase until verified compares with lowercase rendered HMAC.
  03.2 explicitly changes this to a head refusal: 63/65 digits, uppercase
  (with at least one A–F) or nonhex g -> 403 without body demand. Correct
  lowercase 64 -> 204 and one hint; wrong lowercase 64 -> 403 after the body.
- Answer 4: Hooked::pump sends the HTTP server's queued output and closes on
  Refused, without replacing its answer. skein-http's Rejection and response
  mapping supply 501 for Method/Coding and 505 for Version, alongside 4xx.
  The existing HTTP table test asserts those status bytes; the component
  world's malformed-head test asserts 400 and close. Both passed below.
  The latter moves in skein 08. No HTTP production or test change is planned.
- Answer 5: API next_token begins at 1, but open_start guards the incremented
  counter only below 2^60, overlapping hooks::OWNER at 2^59. Hook connections
  begin at 2^59+1 and already assert below 2^60. 03.2 tightens only the API
  guard, checking before issuing a token; 2^59-1 is allowed, 2^59 asserts
  before any io request. A private child-module test exercises both edges
  without exposing a production test API. The owner's 60-bit tag still fits.
- Answer 6: start::heard already uses the longer Retry-After without a cap,
  saturates now+wait, and caps only the doubled next backoff. The moved
  the_start_stops_on_another_user_or_a_4xx_and_waits_otherwise tests only 5 s;
  a_call_waiting_for_its_forge_is_unavailable_at_its_deadline has no 429.
  The new test covers both start reads with 20 s versus the 8 s cap, a
  waiting call's one Unavailable at its 10 s deadline with cost 0, no early
  reconnect, and a wake saturated to Time::from_nanos(u64::MAX).
  Hooked::call already leaves decoder=None without EVENT_HEADER, verifies,
  answers 204 and produces no hint. The existing unknown-event world test
  supplies an event header, so it does not cover absence: a distinct new
  Hooked/Wire test omits the header completely and asserts 204/0/kept alive.
  Neither answer-6 behavior needs a production change.
- Answer 1 remains: no Settings/Forge Host field, and no owner-supplied Host.
  Both check and send call skein 01.4's skein_forge::host(address, name).
  The port wording now explicitly follows decision 25: omit both 443 and 80.
- The six new 03.2 test names are request_head_counts_user_agent,
  header_controls_are_refused_before_io, signature_shape_is_refused_at_the_head,
  a_verified_delivery_without_an_event_is_answered_204_without_a_hint,
  start_retry_after_wins_over_the_backoff_cap, and
  api_tokens_stop_before_the_listener_range. The two broader settings/stack
  tables extend the existing unusable_forge_settings_name_the_forge and
  zero_documents_and_stacked_demands_are_refused_in_order; no moved test is
  renamed or removed. The new private-file and hook-driver kinds have reference
  tests. Tests' fixtures visibility is changed in 03.2, before either consumer.
- Counts: 117 at base minus four root marker/table identities = 113 inherited.
  03.1 adds two root tests: 115. 03.2 adds two root, two hooks, one connection
  and one private component test: 121. 03.3 adds two private limits and four
  connection tests: 127. Thus 14 new remains correct, with different 03.2
  identities. Sizes: 2850 moved/240 new, 0/110, 0/60 production lines.
  Moved names, the 0.149 s original component baseline, and all budgets stay.
- All behavior changes are in 03.2 (head admission, signature, API range) or
  03.3 (the settled page measure). No session beyond 03.3 was reintroduced.
  Question 34 belongs to skein 01.3; no decision on it is taken here.
  Question 35's Merge read belongs to skein 01.5, not the component.

### Commands at the base

The future skein-forge-connection package and its prerequisites do not exist
at the base. The first two commands and the neutrality scan are their base
forms; the ready check must run the future paths on the landed interfaces.
These runs do not claim to prove the planned 127 tests or their new behavior.
The 0.084 s component result is a rerun, not a replacement baseline.

```text
$ cargo nextest run -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished test profile [unoptimized + debuginfo] in 0.14s
Starting 117 tests across 1 binary
Summary [0.084s] 117 tests run: 117 passed, 0 skipped
Log: /srv/data/git/runner/src/tmp/logs/20261019T011706-733.log
```

```text
$ cargo clippy -p temper-engine-protocol-forge --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished dev profile [unoptimized + debuginfo] in 0.21s
Log: /srv/data/git/runner/src/tmp/logs/20261019T011706-734.log
```

```text
$ cargo nextest run -p skein-http -E 'test(=tests::server::head::every_rejection_s_answer_fits_the_room_set_aside_and_names_its_status)'
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Starting 1 test across 1 binary (184 tests skipped)
PASS [0.001s] every_rejection_s_answer_fits_the_room_set_aside_and_names_its_status
Summary [0.002s] 1 test run: 1 passed, 184 skipped
Log: /srv/data/git/runner/src/tmp/logs/20261019T011707-735.log
```

```text
$ cargo nextest run -p temper-engine-protocol-forge-world --test hooks -E 'test(=a_malformed_head_is_answered_4xx_and_closed)'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Starting 1 test across 1 binary (18 tests skipped)
PASS [0.002s] a_malformed_head_is_answered_4xx_and_closed
Summary [0.002s] 1 test run: 1 passed, 18 skipped
Log: /srv/data/git/runner/src/tmp/logs/20261019T011941-736.log
```

The unanchored consumer search below includes world-crate imports too, unlike
the earlier exact component-name search. Existing consumer files and symbols
remain those inventoried in the preceding revision; no new downstream API
variant is introduced by this pass. Output below summarizes the long rg lines.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
root=Path("/srv/data/git/runner/src")
base=root/"tmp/planning/temper-base"
d=base/"crates/temper-engine-protocol-forge/src"
for f in [d/"tests.rs",*sorted((d/"tests").glob("*.rs"))]:
 print(f.relative_to(d),len(re.findall(r"^#\[test\]",f.read_text(),re.M)),"tests")
for file,pattern in [("limits.rs",r"Unusable::(Token|User|Request)"),("start.rs",r"after > next|saturating_add|start_backoff_max"),("component.rs",r"next_token: 1|connection token prefix"),("hooks.rs",r"const OWNER|next: OWNER|component owner prefix"),("hooks/connection.rs",r"is_ascii_hexdigit|None => None|Event::Refused")]:
 hits=subprocess.run(["rg","-n",pattern,str(d/file)],capture_output=True,text=True)
 print(file, " | ".join(hits.stdout.splitlines()))
for repo in [base,root/"tmp/planning/skein-base",root/"jig",root/"smith"]:
 hits=subprocess.run(["rg","-l","--glob","*.rs","--glob","!**/temper.old/**",r"temper_engine_protocol_forge|skein_forge_connection",str(repo)],capture_output=True,text=True)
 print(repo.name,len(hits.stdout.splitlines()),"consumer files")
print("new names per increment: 2 + 6 + 6; inherited 117 - 4 = 113; totals 115/121/127")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
tests.rs 17; connection 20; contents 23; fixtures 0; hooks 12; reads 27; writes 18
limits.rs: Token/User/Request refusals present
start.rs: doubled backoff capped; after > next; now.saturating_add(after)
component.rs: next_token 1; guard checked_shl(60)
hooks.rs: OWNER 1<<59; next OWNER+1; guard <1<<60
hooks/connection.rs: absent event -> None; Refused -> closing; is_ascii_hexdigit
temper-base 37 consumer files; skein-base 0; jig 0; smith 0
new names per increment: 2 + 6 + 6; inherited 117 - 4 = 113; totals 115/121/127
Log: /srv/data/git/runner/src/tmp/logs/20261019T011940-734.log
```

```text
$ rg -n -i temper crates/temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Cargo.toml:2:name = "temper-engine-protocol-forge"
Cargo.toml:16:temper-forge-forgejo.workspace = true
Cargo.toml:17:temper-engine-domain-forge-client.workspace = true
[other matches omitted; base matches are expected, not a passed neutrality check]
Log: /srv/data/git/runner/src/tmp/logs/20261019T011940-735.log
```

### Interface sketch and cleanup

A detached skein-base scratch compiled only the changed Settings, Forge,
Transport and Shared declarations, plus Word/PlaintextName and the reused
Token/User/Request refusal shapes. Kind/Webhooks are unchanged base types.
A scratch-only skein-forge supplies 01.4's not-yet-landed Credential::Token
declaration. There is no component implementation and no new answer-2 variant.
The first attempt caught missing Debug derives; the session sketch now shows
them (Shared also keeps Clone/Copy). The prerequisite stub received Debug too.
The ready check must still compile with the real landed upstream crate.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-03-answers-2 8938e16
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 8938e16)
HEAD is now at 8938e16 Answer the forge kit planner's other questions
```

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03-answers-2
error: type does not implement core::fmt::Debug
skein-forge/src/lib.rs: Credential
requested on the command line with -D missing-debug-implementations
Log: /srv/data/git/runner/src/tmp/logs/20261019T011911-733.log
```

After adding the derives:

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03-answers-2
Checking skein-forge v0.0.0
Checking forge-connection-sketch v0.0.0
Finished dev profile [unoptimized + debuginfo] in 0.24s
Log: /srv/data/git/runner/src/tmp/logs/20261019T011954-733.log
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-03-answers-2
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --short && git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --short && git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; no output
```

### Second revision document check

The plan files are untracked in the planning worktree. The explicit check
reads both files; git diff --check alone would not inspect them. No commit
or landing ran, and only this session file and its appended findings remain
changed by this task.

```text
$ python3 -c 'from pathlib import Path
import re
root=Path("/srv/data/git/runner/src/tmp/planning/temper-forge-kit")
p=root/"docs/development/plans/forge-kit/skein/03-connection.md"
s=p.read_text()
parts=re.split(r"(?=^### 03\.)",s,flags=re.M)[1:]
assert len(s.splitlines())<=400 and len(parts)==3
for part in parts:
 for label in ["Scope","Needs","Files","Shape","Test shape","Proof","Don'\''t","Size","Removes"]:
  assert re.search("^"+re.escape(label)+":",part,re.M),label
 assert int(re.search(r"Size:.*?, (\d+) new",part).group(1))<=300
assert all(x.startswith("Proof: world,") for x in s.splitlines() if x.startswith("Proof:"))
for bad in ["pending the designer","TokenValue","UserAgent","pub host:","### 03.4","settings_checks_accept_their_bounds","stack_and_document_checks_accept_their_bounds"]:
 assert bad not in s,bad
names=["request_head_counts_user_agent","header_controls_are_refused_before_io","signature_shape_is_refused_at_the_head","a_verified_delivery_without_an_event_is_answered_204_without_a_hint","start_retry_after_wins_over_the_backoff_cap","api_tokens_stop_before_the_listener_range"]
assert len(set(names))==6 and all(n in parts[1] for n in names)
for count in ["115 passed","121 passed","127 passed","113 inherited component test identities +14 new (2+6+6)"]:
 assert count in s,count
for f in [p,p.parents[1]/"findings/base-skein-03.md"]:
 assert not any(x.rstrip()!=x for x in f.read_text().splitlines()),f
assert not Path("/srv/data/git/runner/src/tmp/planning/scratch-skein-03-answers-2").exists()
print("Session:",len(s.splitlines()),"lines; 3 increments; new production sizes 240/110/60")
print("Six named 03.2 tests; 113 inherited + 2 + 6 + 6 = 127; proofs 115/121/127")
print("No pending marks, Host field or answer-2 variants; all required fields present")
print("Whitespace clean; scratch absent")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
Session: 365 lines; 3 increments; new production sizes 240/110/60
Six named 03.2 tests; 113 inherited + 2 + 6 + 6 = 127; proofs 115/121/127
No pending marks, Host field or answer-2 variants; all required fields present
Whitespace clean; scratch absent
Log: /srv/data/git/runner/src/tmp/logs/20261019T012236-733.log
```

```text
$ git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
exit 0; no output (explicit untracked-file checks above)
```



## Third revision: re-check rulings 1–6

2026-10-19. Inputs: skein-base `25d2551`, temper-base `dd12b8e`;
development.md section 5, the current README/code-organization/decisions,
all three designer answer files, both reviews and the session's prior evidence.
All design questions are answered. This append supersedes the earlier
113-inherited/14-new arithmetic, reused Host-test identity, Single harness
reference and claim that every adaptation changes no behavior. Earlier
answer-1–6, measure, body-bound, consumer, baseline and session-boundary fixes
remain; this revision follows all six rulings in reviews/skein-03b.txt.

### Audit and changes

1. The primary-checkout graph returned no component nodes for
   ConnectionHarness/Hooked/Shared/signature. Exact base-file rg and targeted
   reads supplied that missing coverage. The skein graph located the HTTP
   rejection test; rg confirmed its path. No list/index/status call ran.
   03.2 now lists src/hooks.rs, makes its connection module pub(crate), and
   keeps signature private. The new fixtures::hook, fixtures::signed_head
   and their Hooked/Wire driver exercise Hooked::stream/pump; their model
   is the existing ConnectionHarness::pump/answer, not nonexistent Single.
2. The timed Harness scenario answers only the identity read with 429 at
   1 s: its next connection is at 21 s, and the waiting call ends once at
   11 s, Unavailable, cost 0. Separate direct heard cases cover Checking
   and Settings at both 1 s and u64::MAX-1 ns, with the same 20 s retry and
   2 s next backoff. No ApiSettings response is scheduled during the wait.
3. The base Host test calls conventions::host directly. Its identity stays
   solely in skein-forge from 01.4; 03.1 drops the component copy.
   each_request_carries_the_host_its_transport_derives is a new wiring test,
   explicitly covering TLS on 80 and plaintext on 443 as well.
   The inherited partition is root 12, connection 20, hooks 12, reads 27,
   writes 18, contents 23 = 112. New counts are 3/6/6 = 15; Proof totals
   remain 115/121/127 and Done when says 112 + 15 = 127.
4. Shared keeps all six base derives: Clone, Copy, PartialEq, Eq, Hash,
   Debug. Its changed declaration compiled against real skein-lib/io below.
5. 03.1's Shape explicitly attributes Word and PlaintextName refusals and
   decision 25's port rule to the new home's owner-value boundary: the
   owner supplies the word and transport, the component knows no system's
   fixed word or host, and Host uses 01.4's helper. Their tests remain in
   03.1. The moved body fixture and refusal numbers remain 1042 and 1041.
6. The rejection-status test is explicitly skein-http's
   src/tests/server/head.rs test, outside the component's count and Proof
   command. No new HTTP command is prescribed; its prior passing run above
   remains the evidence. Skein 08 still owns the machine/listener worlds.

Sizes remain 2850 moved/240 new, 0/110 and 0/60 production lines;
03.1 also moves about 3100 test lines. No new session Proof or Done when
command is introduced. The future crate's commands still need the landed
prerequisites at the ready check. This base rerun proves only the existing
117 tests, not the 127 planned tests; its 0.119 s does not replace
base-runs.md's 0.149 s baseline.

### Commands and changed declaration

Only Shared's declaration changed in this pass. The detached scratch
compiled that declaration, with all its fields and derives, against the
base's real skein-lib and skein-io. It added no stub dependency, implementation
or test. The prior evidence covers the other unchanged boundary declarations.
The scratch was removed after clippy passed.

```text
$ git worktree add --detach /srv/data/git/runner/src/tmp/planning/scratch-skein-03-recheck 25d2551
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
Preparing worktree (detached HEAD 25d2551)
HEAD is now at 25d2551 Have the forge component count a call's cost
```

```text
$ cargo clippy -p forge-connection-sketch --all-targets -- -D warnings
Directory: /srv/data/git/runner/src/tmp/planning/scratch-skein-03-recheck
Checking skein-lib v0.0.0
Checking skein-io v0.0.0
Checking forge-connection-sketch v0.0.0
Finished dev profile [unoptimized + debuginfo] in 0.73s
Log: /srv/data/git/runner/src/tmp/logs/20261019T013323-751.log
```

```text
$ cargo nextest run -p temper-engine-protocol-forge
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Finished test profile [unoptimized + debuginfo] in 0.15s
Starting 117 tests across 1 binary
Summary [0.119s] 117 tests run: 117 passed, 0 skipped
Log: /srv/data/git/runner/src/tmp/logs/20261019T013324-752.log
```

```text
$ git worktree remove --force /srv/data/git/runner/src/tmp/planning/scratch-skein-03-recheck
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --short && git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/skein-base
exit 0; no output
```

```text
$ git status --short && git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
exit 0; no output
```

The source/count audit uses rg for every added existing name and confirms
that the new helpers/test have no component definitions at the base.

```text
$ python3 -c 'from pathlib import Path
import re, subprocess
root=Path("/srv/data/git/runner/src/tmp/planning")
base=root/"temper-base/crates/temper-engine-protocol-forge/src"
counts={str(f.relative_to(base)):len(re.findall(r"^#\[test\]",f.read_text(),re.M)) for f in [base/"tests.rs",*sorted((base/"tests").glob("*.rs"))]}
assert sum(counts.values())==117
print("Base tests:",counts,"total",sum(counts.values()))
removed=["a_marker_round_trips_its_key_and_keeps_the_text","a_body_without_a_well_formed_marker_has_no_key","a_full_table_drops_the_entry_used_longest_ago","a_full_user_table_keeps_the_most_recent_login_and_replaces_in_place","host_names_include_only_nondefault_ports_and_bracket_ipv6"]
for name in removed:
 assert subprocess.run(["rg","-q",r"fn "+name+r"\(",str(base/"tests.rs")]).returncode==0,name
print("Inherited: root",counts["tests.rs"]-len(removed),"; total",sum(counts.values())-len(removed))
checks=[(base/"tests/fixtures.rs",r"pub struct ConnectionHarness|pub fn pump|pub fn answer|Wire::new"),(base/"hooks.rs",r"^mod connection;"),(base/"hooks/connection.rs",r"pub\(crate\) struct Hooked|pub fn stream|pub fn pump|^fn signature"),(base/"limits.rs",r"derive\(Clone, Copy, PartialEq, Eq, Hash, Debug\)|pub struct Shared"),(base/"start.rs",r"pub fn heard|Start::Checking|Start::Settings|now.saturating_add"),(root/"skein-base/crates/skein-http/src/tests/server/head.rs",r"fn every_rejection_s_answer_fits_the_room_set_aside_and_names_its_status")]
for path,pattern in checks:
 hits=subprocess.run(["rg","-n",pattern,str(path)],capture_output=True,text=True)
 assert hits.returncode==0,path
 print(path.relative_to(root),":", " | ".join(hits.stdout.splitlines()))
for pattern in [r"fn hook\(",r"fn signed_head\(",r"fn each_request_carries_the_host_its_transport_derives\("]:
 assert subprocess.run(["rg","-q",pattern,str(base)]).returncode==1,pattern
print("hook, signed_head and Host wiring test are new component names")
print("Plan totals: 112 + 3 = 115; + 6 = 121; + 6 = 127")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Base tests: {'tests.rs': 17, 'tests/connection.rs': 20, 'tests/contents.rs': 23, 'tests/fixtures.rs': 0, 'tests/hooks.rs': 12, 'tests/reads.rs': 27, 'tests/writes.rs': 18} total 117
Inherited: root 12 ; total 112
temper-base/crates/temper-engine-protocol-forge/src/tests/fixtures.rs : 178:    pub fn answered(&self, call: u64) -> Option<&Answered> { | 216:    pub fn answer(&mut self, forge: u16, bytes: &[u8]) { | 291:                                        wire: skein_world::stream::Wire::new(owner.raw()), | 411:pub struct ConnectionHarness { | 429:    pub fn pump(&mut self) { | 432:    pub fn answer(&mut self, response: &[u8]) -> List<crate::connection::Heard> { | 433:        let mut wire = skein_world::stream::Wire::new(1);
temper-base/crates/temper-engine-protocol-forge/src/hooks.rs : 6:mod connection;
temper-base/crates/temper-engine-protocol-forge/src/hooks/connection.rs : 31:pub(crate) struct Hooked { | 87:    pub fn stream(&mut self, env: &Env<Limits>, event: Up) { | 236:    pub fn pump( | 313:fn signature(header: Option<&[u8]>) -> Option<[u8; 64]> {
temper-base/crates/temper-engine-protocol-forge/src/limits.rs : 9:#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] | 58:#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] | 59:pub struct Shared { | 71:#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] | 83:#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)] | 95:#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
temper-base/crates/temper-engine-protocol-forge/src/start.rs : 36:pub fn heard(start: Start, heard: Heard, user: u64, now: Time, limits: &Limits) -> Start { | 38:        Start::Checking { next } => (StartRead::User, next), | 39:        Start::Settings { next } => (StartRead::ApiSettings, next), | 44:        Heard::User { id } if read == StartRead::User && id == user => Start::Settings { next }, | 53:            Start::Waiting { until: now.saturating_add(after), next: doubled } | 56:            Start::Waiting { until: now.saturating_add(next), next: doubled }
skein-base/crates/skein-http/src/tests/server/head.rs : 320:fn every_rejection_s_answer_fits_the_room_set_aside_and_names_its_status() {
hook, signed_head and Host wiring test are new component names
Plan totals: 112 + 3 = 115; + 6 = 121; + 6 = 127
```

### Third revision document check

The check reads the session and findings directly because they are untracked
in the plan worktree. The unchanged Proof commands still target the future
crate, and all of their base forms were run in earlier evidence above.
No review item or design answer remains unsettled in this session.

```text
$ python3 -c 'from pathlib import Path
import re
root=Path("/srv/data/git/runner/src/tmp/planning")
p=root/"temper-forge-kit/docs/development/plans/forge-kit/skein/03-connection.md"
s=p.read_text(); parts=re.split(r"(?=^### 03\.)",s,flags=re.M)[1:]
assert len(s.splitlines())<=400 and len(parts)==3
for part in parts:
 for label in ["Scope","Needs","Files","Shape","Test shape","Proof","Don"+chr(39)+"t","Size","Removes"]:
  assert re.search("^"+re.escape(label)+":",part,re.M),label
assert [int(re.search(r"Size:.*?, (\d+) new",x).group(1)) for x in parts]==[240,110,60]
assert re.findall(r"Proof: world, .*?: (\d+) passed",s)==["115","121","127"]
assert "112 inherited component test identities +15 new (3+6+6)" in s
assert "#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]" in parts[0]
assert "hooks,hooks/connection" in parts[1] and "mod connection pub(crate)" in parts[1]
assert "fixtures::ConnectionHarness::{pump,answer}" in parts[1] and "keep signature private" in parts[1]
assert "identity start read only" in parts[1] and "directly for Start::Checking and Start::Settings" in parts[1]
assert "each_request_carries_the_host_its_transport_derives" in parts[0] and "drop this copy" in parts[0]
assert "new home" in parts[0] and "outside this crate" in parts[1]
for old in ["fixtures::Single", "113 inherited", "+14 new", "pending the designer", "pub host:", "TokenValue", "### 03.4", "### 03.5"]:
 assert old not in s,old
for kept in ["1041/needed 1042","body_bytes:1045 becomes 1042","skein_forge::measure::answer","MAX_ROW","fuzzy_machine 7.481–7.709 s","Temper'\''s side:"]:
 assert kept in s,kept
for f in [p,p.parents[1]/"findings/base-skein-03.md"]:
 assert all(line.rstrip()==line for line in f.read_text().splitlines()),f
assert not (root/"scratch-skein-03-recheck").exists()
print("Session:",len(s.splitlines()),"lines; increments 03.1–03.3; production new sizes 240/110/60")
print("112 inherited + 15 new (3+6+6) = 127; Proof totals 115/121/127")
print("All six corrections and earlier required fixes present; whitespace clean; scratch absent")
'
Directory: /srv/data/git/runner/src/tmp/planning/temper-base
Session: 380 lines; increments 03.1–03.3; production new sizes 240/110/60
112 inherited + 15 new (3+6+6) = 127; Proof totals 115/121/127
All six corrections and earlier required fixes present; whitespace clean; scratch absent
```

```text
$ git diff --check
Directory: /srv/data/git/runner/src/tmp/planning/temper-forge-kit
exit 0; no output (the explicit check above also reads the untracked files)
```

Only skein/03-connection.md and this appended findings file remain changed
by this task. No commit or landing ran. The scratch worktree is removed.
