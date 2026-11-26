# QA pass 34, request 1: 1 findings at 47770ca2090e

Fix each finding, each fix with a test in the lowest world that reproduces it, starting from the finding's lines. Deliver what the cases expected and nothing past it.

## 6. party reissue's answer tells the owner to sign their own directory in as the service

Severity: medium

Surface: ctl

Page: party

Area: ctl party reissue

Landings:
- goal 11563: An owner reissues a service party's sign-in from ctl

Setup: temper/docs/development/qa/setups/ctl-hello/

Reproduction:

```text
ctl project make Hello --key qa-hello-project
ctl forge connect forgejo-admin {forgejo.admin} --first work/hello --project Hello --api {forgejo.api} --git {forgejo.git} --git-program {git} --key qa-hello
ctl forge list --until made:1
ctl account add-key 1 {model.key} --key qa-model
ctl account list --until usable
ctl party make --project Hello --role 2 --out qa43-svc1 "Builder" --key qa-svc-1
ctl party reissue 3 --out qa43-link1 --key qa-re-1
```

Expected: reissued: party N; its sign-ins have ended; sign its directory in with temper ctl DIRECTORY sign-in FILE, DIRECTORY standing for the service's directory, FILE as given (temper/docs/design/client/ctl/README.md, 7)

Seen: reissued: party 3; its sign-ins have ended; sign its directory in with temper ctl OWNER sign-in qa43-link1, where OWNER is the full path of the owner's own ctl directory, the one that ran the reissue. Following the printed line from a second owner directory signed that directory in as party 3 (me: party 3 · Builder), ending the owner's sign-in there; it could then no longer reissue (party 3 is this directory's own party). The same with --out -, on stderr.

Evidence (kept beside the QA ledger):
- evidence/6/reissue-hint.txt

Earlier goals: none
