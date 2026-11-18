# temper ctl deployment

Kind: reference

Read the deployment's store census or take a checkpoint.

## deployment census

```
deployment census
Read the store's census in the deployment's document
Prints: `census at commit N, writes since commit S`, then a line per keyspace in prefix order; `no census yet` before the first report
--json: the record census.
Key: none.
Needs a deployment role; jig's refusal exits 2 without one.
Prefixes print in lower-case hex, - for the empty prefix or no name.
Heads print H:R,H:R, - for an unnumbered head, heads - for none.
JSON keeps the census whole, or null before the first report.
```

## deployment checkpoint

```
deployment checkpoint [--timeout DURATION] [--key KEY]
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. Wait this long; 10 minutes when absent
Take a checkpoint of the serving deployment as its owner
Prints: The checkpoint's name, commit, secrets position and file figures; --json gives its fields
--json: the record checkpoint.
Key: --key KEY, optional.
Only a deployment owner takes one. A refusal exits 2. Past the wait,
it exits 3; the checkpoint goes on in the engine.
```
