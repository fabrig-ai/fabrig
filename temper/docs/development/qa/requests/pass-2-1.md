# QA pass 2, request 1: 1 findings at 437a3a8c4bb9

Fix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.

## 2. A settings apply of a charter past about 64 KB gets no answer: its refusal is lost, not given

Severity: medium

Surface: ctl

Page: settings

Area: temper/draft/qa

Landings:
- goal 5198: Carry out milestone T1 of temper/docs/design/draft/tools-by-role.execution.md (its row and its order entry), with tools-by-role.md as its source (section 6 for T1). Its draft is on main (goal 5069, 1911dad8fc). The user approved it (11-16 12:2xZ, via te…

Setup: fresh

Reproduction:

```text
ctl project make Hello --key qa-hello-project
ctl settings apply FILE --against 1 --key qa-apply-big
```

Expected: refused before anything changes, naming the charter, the wake reason and the figures and saying what to trim (tools-by-role.md, 6), as the same apply of a 60 KB charter answers: 'settings apply: refused; item 0; charter 2: charter 2 on a first wake is 88,946 bytes owned before its brief, past its allowance of 65,536; trim its text, parts or tool families'

Seen: for FILE, charter 2's document with its instructions at 82,227 bytes (the file 85,329 bytes, within the tool's 1,048,576-byte bound): 'temper ctl: the last answer to this act from the engine at http://127.0.0.1:PORT was lost; no outcome within 30 s; it is kept, and the same --key may be tried again', exit 1, twice (against revisions 1 and 2, new keys); 'ctl pending' then lists nothing, though the answer says the request is kept; the box's log says nothing of it. A person learns neither that it was refused nor why.

Evidence (kept beside the QA ledger):
- evidence/2/charter-2-too-large.json

Earlier goals: none
