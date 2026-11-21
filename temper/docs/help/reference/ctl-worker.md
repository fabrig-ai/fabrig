# temper ctl worker

Kind: reference

Workers: list, add, change places, remove, pause, drain or resume.

## worker list

```
worker list [--since DIGEST] [--until CONDITION] [--timeout DURATION]
  --since DIGEST: a digest, as a read prints it; optional. Wait until it differs from the read that printed DIGEST
  --until CONDITION: a condition; optional. Wait until it holds: drained:WORKER, paused:WORKER, or changed
  --timeout DURATION: a duration: seconds, or a number with s, m or h, at most 3600 seconds; optional. How long a wait lasts; 300 seconds when absent, at most 3600
Read the workers not withdrawn, in number order
Prints: A line per worker: WORKER, its places, up since TIME, down since TIME or down, its runs' count, then paused, draining or drained when not taking; and `digest: workers`
--json: the record workers.
Key: none.
Runs are those of the connected host of the worker's name.
JSON gives each worker not withdrawn, with its runs' count and pause:
taking, paused, draining or drained. The digest covers these rows.
Waits on the deployment's watch. paused:WORKER holds for paused,
draining or drained; drained:WORKER holds for drained. An unknown
live worker in a condition is a usage error (exit 1) before watching.
```

## worker add

```
worker add WORKER --places PLACES --digest SHA256 --key KEY
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The worker's name
  --places PLACES: a count from 1 to 4294967295; required. The worker's places
  --digest SHA256: 64 lowercase hexadecimal digits; required. The SHA-256 of the worker's secret
Add a worker, or bring a withdrawn one back at its number
Prints: `worker WORKER: revision R`, the new revision
--json: the record change_settings.
Key: --key KEY, required.
Make the secret on the worker's own machine as temper help server says;
the digest is its SHA-256. A worker started first redials until added.
Changes one setting against the newest revision read; running work stays.
```

## worker places

```
worker places WORKER PLACES --key KEY
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The live worker's name
  PLACES (places): a count from 1 to 4294967295; required. The worker's new places
Change a worker's places
Prints: `worker WORKER: revision R`, the new revision
--json: the record change_settings.
Key: --key KEY, required.
A linked worker takes its new places at its next start.
Changes one setting against the newest revision read; running work stays.
```

## worker remove

```
worker remove WORKER --key KEY
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The live worker's name
Remove a worker, withdrawn at its number
Prints: `worker WORKER: revision R`, the new revision
--json: the record change_settings.
Key: --key KEY, required.
Stop a linked worker first; its runs end interrupted.
Changes one setting against the newest revision read; running work stays.
```

## worker pause

```
worker pause WORKER
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The live worker's name
Ask a linked worker to take no new run or job; its runs go on
Prints: `worker WORKER: asked to pause`
--json: the record pause_worker.
Key: none.
A worker's restart ends its pause.
An ask is no settings change: no key, no revision, no retry.
```

## worker drain

```
worker drain WORKER
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The live worker's name
Ask a linked worker to take no new work and drain
Prints: `worker WORKER: asked to drain`, then `worker list --until drained:WORKER`
--json: the record pause_worker.
Key: none.
Each run stops at its turn's end; what is left is cut at the worker's
drain bound. A drained worker stays linked and takes nothing until
worker resume or its stop, which then finds nothing to drain.
An older build is kept from new work but not drained.
An ask is no settings change: no key, no revision, no retry.
```

## worker resume

```
worker resume WORKER
  WORKER (worker): a worker name: ASCII letters, digits, -, _ or ., starting with a letter or digit, at most 128 bytes; required. The live worker's name
Ask a linked worker to take work again
Prints: `worker WORKER: asked to take work again`
--json: the record pause_worker.
Key: none.
A run already asked to stop still stops at its turn's end.
An ask is no settings change: no key, no revision, no retry.
```
