# temper ctl tier

Kind: reference

Tiers: hold or release a repository's after-landing tier.

## tier hold

```
tier hold --project PROJECT [--owner OWNER] [--name NAME] TIER --reason TEXT --key KEY
  --project PROJECT: a project's name or number; required. The project
  --owner OWNER: text, at most 1048576 bytes; optional. The repository's owner; give --name too
  --name NAME: text, at most 1048576 bytes; optional. The repository's name; give --owner too
  TIER (tier): text, at most 1048576 bytes; required. The tier's name
  --reason TEXT: text, at most 1048576 bytes, or - to read stdin; required. Why the tier waits; 1 to 1024 UTF-8 bytes
Pause a repository's after-landing tier
Prints: `held: TIER of OWNER/NAME`, or a refusal (exit 2)
--json: the record tier.
Key: --key KEY, required.
Give --owner and --name together, or neither for the project's home. A run in flight is let finish.
```

## tier release

```
tier release --project PROJECT [--owner OWNER] [--name NAME] TIER --key KEY
  --project PROJECT: a project's name or number; required. The project
  --owner OWNER: text, at most 1048576 bytes; optional. The repository's owner; give --name too
  --name NAME: text, at most 1048576 bytes; optional. The repository's name; give --owner too
  TIER (tier): text, at most 1048576 bytes; required. The tier's name
Resume a repository's held tier
Prints: `released: TIER of OWNER/NAME`, or a refusal (exit 2)
--json: the record tier.
Key: --key KEY, required.
Starts the next due run at the newest tip, covering the landings made while held.
```
