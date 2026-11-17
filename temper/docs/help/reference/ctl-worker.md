# temper ctl worker

Kind: reference

Workers: list, add, change places or remove.

## worker list

```
worker list
Read the workers not withdrawn, in number order
Prints: A line per worker: WORKER, its places, up since TIME, down since TIME or down, and its runs' count
--json: the record workers.
Key: none.
Runs are those of the connected host of the worker's name.
JSON gives each worker not withdrawn, with its runs' count.
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
