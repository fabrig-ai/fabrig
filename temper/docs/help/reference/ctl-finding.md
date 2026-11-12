# temper ctl finding

Kind: reference

Findings: file, read, hand on and triage them.

## finding list

```
finding list --project PROJECT [--state new|routed|backlog|dropped] [--mine]
  --project PROJECT: a project's name or number; required. The project
  --state new|routed|backlog|dropped: one of new, routed, backlog or dropped; optional. Only this state
  --mine: a switch. Only findings held by you or a role you hold
Read a project's areas, triager and findings, newest first
Prints: Each area's NAME owner OWNER, held by HOLDER: WORDS, the triager, then P/N, state, kind, severity, area or -, holder and title per finding
--json: the record findings.
Key: none.
Reads the findings subject once. At most 128 findings; a cut is marked.
```

## finding show

```
finding show FINDING
  FINDING (finding): P/N: a project name or number, then a finding number; required. P/N: the project's name or number and its finding number
Read one finding whole
Prints: Its line, filer, filing time, last hand-off's why, goal and drop reason, then its body
--json: the record finding.
Key: none.
FINDING is P/N, P the project's name or number.
```

## finding file

```
finding file --project PROJECT --kind qa|flake|red|review|feedback [--severity low|medium|high] [--area AREA] TEXT FILE --key KEY
  --project PROJECT: a project's name or number; required. The project
  --kind qa|flake|red|review|feedback: one of qa, flake, red, review or feedback; required. What kind of finding
  --severity low|medium|high: one of low, medium or high; optional. medium when absent
  --area AREA: text, at most 1048576 bytes; optional. The area's name; the triager's when absent
  TEXT (title): text, at most 1048576 bytes, or - to read stdin; required. Its title, 1 to 200 bytes
  FILE (body): a file, at most 4096 bytes; required. Its body, at most 4096 bytes; - reads stdin
File a finding, held by its area's owner or the triager
Prints: filed: P/N, held by HOLDER
--json: the record finding_filed.
Key: --key KEY, required.
An unknown area exits 2 naming the project's areas.
```

## finding hand

```
finding hand FINDING --area AREA TEXT --key KEY
  FINDING (finding): P/N: a project name or number, then a finding number; required. P/N: the project's name or number and its finding number
  --area AREA: text, at most 1048576 bytes; required. The area's name in the project
  TEXT (why): text, at most 1048576 bytes, or - to read stdin; required. Why, 1 to 200 bytes
Hand a finding on to an area's owner, with why
Prints: handed: P/N, held by HOLDER
--json: the record finding_handed.
Key: --key KEY, required.
Only its holder or a project owner hands it on. Its new owner hears why.
```

## finding route

```
finding route FINDING --goal TASK --key KEY
  FINDING (finding): P/N: a project name or number, then a finding number; required. P/N: the project's name or number and its finding number
  --goal TASK: a number; required. The live goal's task number
Route a finding to a live goal of its project
Prints: routed: P/N
--json: the record finding_routed.
Key: --key KEY, required.
The goal hears it. Only the holder or a project owner may triage.
```

## finding backlog

```
finding backlog FINDING --key KEY
  FINDING (finding): P/N: a project name or number, then a finding number; required. P/N: the project's name or number and its finding number
Put a finding in the backlog
Prints: backlog: P/N
--json: the record finding_backlog.
Key: --key KEY, required.
Only its holder or a project owner may triage.
```

## finding drop

```
finding drop FINDING REASON --key KEY
  FINDING (finding): P/N: a project name or number, then a finding number; required. P/N: the project's name or number and its finding number
  REASON (reason): text, at most 1048576 bytes, or - to read stdin; required. Why, 1 to 400 bytes
Drop a finding and keep the reason
Prints: dropped: P/N
--json: the record finding_dropped.
Key: --key KEY, required.
Only its holder or a project owner may triage. Its issue, if any, closes with the reason.
```
