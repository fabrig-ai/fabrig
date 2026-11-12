# temper ctl push

Kind: reference

Push devices: list, add or remove your subscriptions.

## push list

```
push list
List your push devices
Prints: Each device's number, label, added time and push service origin; never its whole endpoint or keys
--json: the record devices.
Key: none.
Only your own devices are shown.
```

## push add

```
push add FILE [--label LABEL] [--key KEY]
  FILE (file): a file; required. Browser subscription JSON, at most 4096 bytes
  --label LABEL: text, at most 1048576 bytes; optional. 1 to 60 UTF-8 bytes; temper ctl when absent
Register a browser's push subscription
Prints: `device NUMBER: added`, or a refusal (exit 2)
--json: the record device-added.
Key: --key KEY, optional.
FILE must be a private regular file owned by you. The subscription's keys are base64url without padding. Other JSON members are ignored.
```

## push remove

```
push remove NUMBER [--key KEY]
  NUMBER (number): a number from 1 to 4294967295; required. Your device's number
Remove one of your push devices
Prints: `device NUMBER: removed`, or `No device NUMBER.` (exit 2)
--json: the record device-removed.
Key: --key KEY, optional.
Removing a device stops pushes to that subscription.
```
